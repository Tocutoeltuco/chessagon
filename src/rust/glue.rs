use crate::network::{buffer::Buffer, p2p::Connection};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/rust/glue.js")]
extern "C" {
    pub fn setScene(idx: i8);
    pub fn setPlayerName(is_self: bool, name: String);
    pub fn joinResponse(resp: String);
    pub fn addChatMessage(kind: String, name: String, content: String);
    pub fn setPieces(pieces: &[u16]);
    pub fn movePieces(pieces: &[u16]);
    pub fn highlight(hexes: &[u16]);
    pub fn setTimers(light: u16, dark: u16, active: i8);
    pub fn removeTimers();
    pub fn addRTT(rtt: i32);
}

#[wasm_bindgen]
#[derive(Debug)]
pub enum JsEvent {
    Start,
    SetGamemode,
    Register,
    CreateRoom,
    JoinRoom,
    SetSettings,
    SendMessage,
    MenuHidden,
    HexClicked,
    TimerExpired,
}

#[derive(Debug)]
pub enum Event {
    Start,
    SetGamemode(u8),
    Register(String),
    CreateRoom,
    JoinRoom(String),
    SetSettings {
        timer: u16,
        host_as_light: bool,
    },
    SendMessage(String),
    MenuHidden(u8),
    HexClicked {
        q: u8,
        r: u8,
    },
    JoinedRoom {
        code: String,
        is_host: bool,
    },
    NetError(JsValue),
    Connected(Connection),
    Disconnected,
    LoadedBoard(Vec<u16>),
    Movement {
        from: (u8, u8),
        to: (u8, u8),
        is_local: bool,
    },
    TimerExpired,
    GameStart,
    GameEnded {
        won_light: bool,
    },
}

impl Event {
    pub fn from_js(evt: JsEvent, data: &[u8]) -> Self {
        let mut buf = Buffer::from_slice(data);
        match evt {
            JsEvent::Start => Self::Start,
            JsEvent::SetGamemode => Self::SetGamemode(buf.read_u8().unwrap()),
            JsEvent::Register => Self::Register(buf.read_js_string().unwrap()),
            JsEvent::CreateRoom => Self::CreateRoom,
            JsEvent::JoinRoom => Self::JoinRoom(buf.read_js_string().unwrap()),
            JsEvent::SetSettings => Self::SetSettings {
                timer: buf.read_u16().unwrap(),
                host_as_light: buf.read_bool().unwrap(),
            },
            JsEvent::SendMessage => Self::SendMessage(buf.read_js_string().unwrap()),
            JsEvent::MenuHidden => Self::MenuHidden(buf.read_u8().unwrap()),
            JsEvent::HexClicked => Self::HexClicked {
                q: buf.read_u8().unwrap(),
                r: buf.read_u8().unwrap(),
            },
            JsEvent::TimerExpired => Self::TimerExpired,
        }
    }
}
