use wasm_bindgen::prelude::*;

use super::connector::Connector;
use super::p2p::Connection;
use super::packet::{ChessPacket, Handshake, Ping};
use crate::glue::{setPlayerName, Event};
use crate::Context;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

pub struct Client {
    ctx: Context,
    conn: Option<Connection>,
    name: String,
    is_host: bool,
    board: Vec<u16>,
}

impl Client {
    pub fn new(ctx: &Context) -> Self {
        Client {
            ctx: ctx.clone(),
            conn: None,
            name: "unknown".to_owned(),
            is_host: false,
            board: vec![],
        }
    }

    fn new_conn(&self, is_host: bool) -> Connector {
        let mut net = Connector::new();

        let name = self.name.clone();
        let ctx = self.ctx.clone();
        net.set_onopen(Box::new(move |conn| {
            ctx.handle(Event::Connected(conn.clone()));
            conn.send(ChessPacket::Handshake(Handshake { name: name.clone() }).write());
        }));

        let ctx = self.ctx.clone();
        net.set_onroom(Box::new(move |code| {
            ctx.handle(Event::JoinedRoom { code, is_host });
        }));

        let ctx = self.ctx.clone();
        net.set_onerror(Box::new(move |err| {
            ctx.handle(Event::NetError(err));
        }));

        let ctx = self.ctx.clone();
        net.set_onclose(Box::new(move || {
            ctx.handle(Event::Disconnected);
        }));

        let ctx = self.ctx.clone();
        net.set_onmessage(Box::new(move |conn, data| {
            let packet = match ChessPacket::read(data) {
                Ok(p) => p,
                Err(e) => {
                    error(&e.to_string());
                    conn.close();
                    return;
                }
            };

            match packet {
                ChessPacket::Handshake(p) => {
                    setPlayerName(false, p.name);
                }
                ChessPacket::Start(p) => {
                    if is_host {
                        error("guest can't start match.");
                        conn.close();
                        return;
                    }
                    //
                }
                ChessPacket::ChatMessage(p) => {}
                ChessPacket::Movement(p) => {}
                ChessPacket::Resign(p) => {}
                ChessPacket::Ping(p) => {
                    if let Some(req) = p.request {
                        conn.send(
                            ChessPacket::Ping(Ping {
                                request: None,
                                reply_to: Some(req),
                            })
                            .write(),
                        );
                    }

                    if let Some(req) = p.reply_to {
                        // Ping response
                    }
                }
            };
        }));

        net
    }

    pub fn on_event(&mut self, evt: &Event) {
        match evt {
            Event::JoinRoom(code) => {
                self.new_conn(false).start_as_guest(code.to_owned());
            }
            Event::CreateRoom => {
                self.new_conn(true).start_as_host();
            }
            Event::Connected(conn) => {
                self.conn = Some(conn.clone());
            }
            Event::Register(name) => {
                self.name = name.clone();
            }
            Event::Disconnected => {
                if let Some(conn) = self.conn.take() {
                    conn.close();
                }
            }
            Event::LoadedBoard(board) => {
                if self.is_host {
                    self.board = board.clone();
                }
            }
            _ => {}
        };
    }
}
