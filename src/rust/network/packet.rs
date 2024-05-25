use super::buffer::Buffer;
use std::fmt::Display;

const NET_VERSION: u8 = 0;

#[derive(Debug, Clone)]
pub enum ParseError {
    MissingFields(String),
    UnknownPacket(u8),
    VersionMismatch(u8, u8),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Start {}
impl Packet for Start {
    const CODE: u8 = 1;

    fn read(_: Buffer) -> Result<Self, ParseError> {
        Ok(Start {})
    }
    fn write(&self, _: &mut Buffer) {}
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Resign {}
impl Packet for Resign {
    const CODE: u8 = 4;

    fn read(_: Buffer) -> Result<Self, ParseError> {
        Ok(Resign {})
    }
    fn write(&self, _: &mut Buffer) {}
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct SetBoard {
    pub board: Vec<u16>,
}
impl Packet for SetBoard {
    const CODE: u8 = 6;

    fn read(mut data: Buffer) -> Result<Self, ParseError> {
        let mut board = Vec::new();
        for _ in 0..read!(data, read_u8) {
            board.push(read!(data, read_u16));
        }

        Ok(SetBoard { board })
    }
    fn write(&self, data: &mut Buffer) {
        data.write_u8(
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

#[derive(Debug)]
pub struct SetSettings {
    pub timer: u16,
    pub host_as_light: bool,
}
impl Packet for SetSettings {
    const CODE: u8 = 7;

    fn read(mut data: Buffer) -> Result<Self, ParseError> {
        Ok(SetSettings {
            timer: read!(data, read_u16),
            host_as_light: read!(data, read_bool),
        })
    }
    fn write(&self, data: &mut Buffer) {
        data.write_u16(self.timer).write_bool(self.host_as_light);
    }
}

#[derive(Debug)]
pub struct Promote {
    pub idx: u8,
    pub kind: u8,
}
impl Packet for Promote {
    const CODE: u8 = 8;

    fn read(mut data: Buffer) -> Result<Self, ParseError> {
        Ok(Promote {
            idx: read!(data, read_u8),
            kind: read!(data, read_u8),
        })
    }
    fn write(&self, data: &mut Buffer) {
        data.write_u8(self.idx).write_u8(self.kind);
    }
}

#[derive(Debug)]
pub enum ChessPacket {
    Handshake(Handshake),
    Start(Start),
    ChatMessage(ChatMessage),
    Movement(Movement),
    Resign(Resign),
    Ping(Ping),
    SetBoard(SetBoard),
    SetSettings(SetSettings),
    Promote(Promote),
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
            SetBoard::CODE => ChessPacket::SetBoard(SetBoard::read(data)?),
            SetSettings::CODE => ChessPacket::SetSettings(SetSettings::read(data)?),
            Promote::CODE => ChessPacket::Promote(Promote::read(data)?),
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
            ChessPacket::SetBoard(p) => p.write(data.write_u8(SetBoard::CODE)),
            ChessPacket::SetSettings(p) => p.write(data.write_u8(SetSettings::CODE)),
            ChessPacket::Promote(p) => p.write(data.write_u8(Promote::CODE)),
        };

        data
    }
}
