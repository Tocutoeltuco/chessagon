use super::buffer::Buffer;
use std::fmt::Display;

const NET_VERSION: u8 = 0;

#[derive(Debug, Clone)]
pub enum ParseError {
    EmptyPacket,
    MissingFields(String),
    UnknownPacket(u8),
    VersionMismatch(u8, u8),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPacket => write!(f, "packet is empty, EOL?"),
            Self::MissingFields(k) => write!(f, "missing a field of kind {}", k),
            Self::UnknownPacket(c) => write!(f, "unknown packet with code {}", c),
            Self::VersionMismatch(got, exp) => {
                write!(f, "net version mismatch: got {}, expected {}", got, exp)
            }
        }
    }
}

macro_rules! read {
    ($data: expr, $method: ident) => {
        $data
            .$method()
            .ok_or(ParseError::MissingFields(stringify!($method).to_owned()))?
    };
}

trait Packet: Sized {
    const CODE: u8;

    fn read(buf: Buffer) -> Result<Self, ParseError>;
    fn write(&self, buf: &mut Buffer);
}

pub struct Handshake {
    pub name: String,
}
impl Packet for Handshake {
    const CODE: u8 = 0;

    fn read(mut data: Buffer) -> Result<Self, ParseError> {
        Ok(Handshake {
            name: read!(data, read_string),
        })
    }
    fn write(&self, data: &mut Buffer) {
        data.write_string(&self.name);
    }
}

pub struct Start {
    pub timer: Option<u16>,
    pub host_as_light: bool,
    pub board: Vec<u16>,
}
impl Packet for Start {
    const CODE: u8 = 1;

    fn read(mut data: Buffer) -> Result<Self, ParseError> {
        let timer = match read!(data, read_u16) {
            0 => None,
            t => Some(t),
        };
        let host_as_light = read!(data, read_bool);

        let mut board = Vec::new();
        for _ in 0..read!(data, read_u8) {
            board.push(read!(data, read_u16));
        }

        Ok(Start {
            timer,
            host_as_light,
            board,
        })
    }
    fn write(&self, data: &mut Buffer) {
        data.write_u16(self.timer.unwrap_or(0))
            .write_bool(self.host_as_light)
            .write_u8(
                self.board
                    .len()
                    .try_into()
                    .expect("too many pieces in board"),
            );

        for piece in self.board.iter() {
            data.write_u16(*piece);
        }
    }
}

pub struct ChatMessage {
    pub content: String,
}
impl Packet for ChatMessage {
    const CODE: u8 = 2;

    fn read(mut data: Buffer) -> Result<Self, ParseError> {
        Ok(ChatMessage {
            content: read!(data, read_string),
        })
    }
    fn write(&self, data: &mut Buffer) {
        data.write_string(&self.content);
    }
}

pub struct Movement {
    pub idx: u8,
    pub q: u8,
    pub r: u8,
    pub time_left: Option<u16>,
}
impl Packet for Movement {
    const CODE: u8 = 3;

    fn read(mut data: Buffer) -> Result<Self, ParseError> {
        Ok(Movement {
            idx: read!(data, read_u8),
            q: read!(data, read_u8),
            r: read!(data, read_u8),
            time_left: match read!(data, read_u16) {
                0 => None,
                t => Some(t),
            },
        })
    }
    fn write(&self, data: &mut Buffer) {
        data.write_u8(self.idx)
            .write_u8(self.q)
            .write_u8(self.r)
            .write_u16(self.time_left.unwrap_or(0));
    }
}

pub struct Resign {}
impl Packet for Resign {
    const CODE: u8 = 4;

    fn read(_: Buffer) -> Result<Self, ParseError> {
        Ok(Resign {})
    }
    fn write(&self, _: &mut Buffer) {}
}

pub struct Ping {
    pub request: Option<u16>,
    pub reply_to: Option<u16>,
}
impl Packet for Ping {
    const CODE: u8 = 5;

    fn read(mut data: Buffer) -> Result<Self, ParseError> {
        let request = if read!(data, read_bool) {
            Some(read!(data, read_u16))
        } else {
            None
        };
        let reply_to = if read!(data, read_bool) {
            Some(read!(data, read_u16))
        } else {
            None
        };

        Ok(Ping { request, reply_to })
    }
    fn write(&self, data: &mut Buffer) {
        match self.request {
            Some(id) => data.write_bool(true).write_u16(id),
            None => data.write_bool(false),
        };
        match self.reply_to {
            Some(id) => data.write_bool(true).write_u16(id),
            None => data.write_bool(false),
        };
    }
}

pub enum ChessPacket {
    Handshake(Handshake),
    Start(Start),
    ChatMessage(ChatMessage),
    Movement(Movement),
    Resign(Resign),
    Ping(Ping),
}
impl ChessPacket {
    pub fn read(mut data: Buffer) -> Result<ChessPacket, ParseError> {
        let version = read!(data, read_u8);
        if version != NET_VERSION {
            return Err(ParseError::VersionMismatch(version, NET_VERSION));
        }

        let packet = match read!(data, read_u8) {
            Handshake::CODE => ChessPacket::Handshake(Handshake::read(data)?),
            Start::CODE => ChessPacket::Start(Start::read(data)?),
            ChatMessage::CODE => ChessPacket::ChatMessage(ChatMessage::read(data)?),
            Movement::CODE => ChessPacket::Movement(Movement::read(data)?),
            Resign::CODE => ChessPacket::Resign(Resign::read(data)?),
            Ping::CODE => ChessPacket::Ping(Ping::read(data)?),
            code => {
                return Err(ParseError::UnknownPacket(code));
            }
        };
        Ok(packet)
    }
    pub fn write(&self) -> Buffer {
        let mut data = Buffer::new();
        data.write_u8(NET_VERSION);

        match self {
            ChessPacket::Handshake(p) => p.write(data.write_u8(Handshake::CODE)),
            ChessPacket::Start(p) => p.write(data.write_u8(Start::CODE)),
            ChessPacket::ChatMessage(p) => p.write(data.write_u8(ChatMessage::CODE)),
            ChessPacket::Movement(p) => p.write(data.write_u8(Movement::CODE)),
            ChessPacket::Resign(p) => p.write(data.write_u8(Resign::CODE)),
            ChessPacket::Ping(p) => p.write(data.write_u8(Ping::CODE)),
        };

        data
    }
}
