use wasm_bindgen::prelude::*;

use super::connector::Connector;
use super::p2p::Connection;
use super::packet::ChessPacket;
use crate::glue::Event;
use crate::Context;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

pub struct Client {
    ctx: Context,
    conn: Option<Connection>,
}

impl Client {
    pub fn new(ctx: &Context) -> Self {
        Client {
            ctx: ctx.clone(),
            conn: None,
        }
    }

    fn new_conn(&self, is_host: bool) -> Connector {
        let mut net = Connector::new();

        let ctx = self.ctx.clone();
        net.set_onopen(Box::new(move |conn| {
            ctx.handle(Event::Connected(conn.clone()));
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
            Event::Disconnected => {
                if let Some(conn) = self.conn.take() {
                    conn.close();
                }
            }
            _ => {}
        };
    }
}
