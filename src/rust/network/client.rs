use rand::rngs::SmallRng;
use rand::RngCore;
use wasm_bindgen::prelude::*;
use web_time::Instant;

use super::connector::Connector;
use super::p2p::Connection;
use super::packet::{
    ChatMessage, ChessPacket, Handshake, Movement, Ping, Promote, Resign, SetBoard, SetSettings,
    Start,
};
use crate::chat::Chat;
use crate::glue::{addRTT, setPlayerName, Button, Event};
use crate::utils::new_rng;
use crate::Context;

#[wasm_bindgen]
extern "C" {
    fn setInterval(closure: &Closure<dyn FnMut()>, millis: u32) -> f64;

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

struct PingRequest {
    id: u16,
    sent_at: Instant,
}

pub struct Client {
    ctx: Context,
    conn: Option<Connection>,
    name: String,
    is_host: bool,
    queue: Vec<ChessPacket>,
    ping: Option<PingRequest>,
    rng: SmallRng,
}

impl Client {
    pub fn new(ctx: &Context) -> Self {
        Client {
            ctx: ctx.clone(),
            conn: None,
            name: "unknown".to_owned(),
            is_host: false,
            queue: vec![],
            ping: None,
            rng: new_rng(),
        }
    }

    fn send_when_ready(&mut self, packet: ChessPacket) {
        match &self.conn {
            Some(c) => c.send(packet.write()),
            None => self.queue.push(packet),
        };
    }

    fn new_conn(&self, is_host: bool) -> Connector {
        let mut net = Connector::new();

        net.set_onestablishing(Box::new(move || {
            Chat::new_peer();
        }));

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

            ctx.handle(Event::PacketReceived(packet));
        }));

        net
    }

    pub fn handle_packet(&mut self, packet: &ChessPacket) {
        let conn = self.conn.as_ref().unwrap();
        match packet {
            ChessPacket::Handshake(p) => {
                setPlayerName(false, p.name.clone());
                self.ctx.handle(Event::Handshake(p.name.clone()));
            }
            ChessPacket::Start(_) => {
                if self.is_host {
                    error("guest can't start match.");
                    conn.close();
                    return;
                }

                self.ctx.handle(Event::GameStart);
            }
            ChessPacket::ChatMessage(p) => {
                self.ctx.handle(Event::ChatMessage {
                    is_local: false,
                    content: p.content.clone(),
                });
            }
            ChessPacket::Movement(p) => {
                self.ctx.handle(Event::Movement {
                    piece: p.idx,
                    to: (p.q, p.r),
                    is_local: false,
                });
            }
            ChessPacket::Resign(_) => {
                self.ctx.handle(Event::Resign(false));
            }
            ChessPacket::Ping(p) => {
                if let Some(id) = p.request {
                    conn.send(
                        ChessPacket::Ping(Ping {
                            request: None,
                            reply_to: Some(id),
                        })
                        .write(),
                    );
                }

                if let Some(id) = p.reply_to {
                    // Ping response
                    if let Some(req) = &self.ping {
                        if req.id == id {
                            let ping = Instant::now() - req.sent_at;
                            self.ping.take();
                            addRTT(ping.as_millis().try_into().unwrap());
                        }
                    }
                }
            }
            ChessPacket::SetBoard(p) => {
                if self.is_host {
                    error("guest can't load board");
                    conn.close();
                    return;
                }

                self.ctx.handle(Event::LoadedBoard(p.board.clone()));
            }
            ChessPacket::SetSettings(p) => {
                if self.is_host {
                    error("guest can't set settings");
                    conn.close();
                    return;
                }

                self.ctx.handle(Event::SetSettings {
                    timer: p.timer,
                    host_as_light: p.host_as_light,
                });
            }
            ChessPacket::Promote(p) => {
                self.ctx.handle(Event::Promotion {
                    piece: p.idx,
                    kind: p.kind,
                    is_local: false,
                });
            }
        };
    }

    pub fn on_event(&mut self, evt: &Event) {
        match evt {
            Event::Start => {
                let ctx = self.ctx.clone();
                let closure: Closure<dyn FnMut()> =
                    Closure::new(move || ctx.handle(Event::PingRequest {}));
                setInterval(&closure, 5_000);
                closure.forget();
            }
            Event::JoinRoom(code) => {
                self.is_host = false;
                self.new_conn(false).start_as_guest(code.to_owned());
            }
            Event::CreateRoom => {
                self.is_host = true;
                self.new_conn(true).start_as_host();
            }
            Event::Connected(conn) => {
                self.conn = Some(conn.clone());

                for packet in self.queue.drain(0..) {
                    conn.send(packet.write());
                }
            }
            Event::Register(name) => {
                self.name = name.clone();
            }
            Event::ChatMessage { is_local, content } => {
                if !is_local {
                    return;
                }
                if let Some(conn) = &self.conn {
                    conn.send(
                        ChessPacket::ChatMessage(ChatMessage {
                            content: content.clone(),
                        })
                        .write(),
                    );
                }
            }
            Event::Disconnected => {
                if let Some(conn) = self.conn.take() {
                    conn.close();
                }
            }
            Event::GameStart => {
                if self.is_host {
                    self.conn
                        .as_ref()
                        .unwrap()
                        .send(ChessPacket::Start(Start {}).write());
                }
            }
            Event::SetSettings {
                timer,
                host_as_light,
            } => {
                if !self.is_host {
                    return;
                }

                self.send_when_ready(ChessPacket::SetSettings(SetSettings {
                    timer: *timer,
                    host_as_light: *host_as_light,
                }));
            }
            Event::LoadedBoard(board) => {
                if !self.is_host {
                    return;
                }

                self.send_when_ready(ChessPacket::SetBoard(SetBoard {
                    board: board.clone(),
                }));
            }
            Event::PingRequest => {
                let conn = match &self.conn {
                    Some(c) => c,
                    None => return,
                };

                if self.ping.is_some() {
                    error("unhandled ping: is peer disconnected?");
                    self.ctx.handle(Event::Disconnected);
                    return;
                }

                let id = self.rng.next_u32() as u16;
                self.ping = Some(PingRequest {
                    id,
                    sent_at: Instant::now(),
                });
                conn.send(
                    ChessPacket::Ping(Ping {
                        request: Some(id),
                        reply_to: None,
                    })
                    .write(),
                );
            }
            Event::PacketReceived(packet) => {
                self.handle_packet(packet);
            }
            Event::Movement {
                piece,
                to,
                is_local,
            } => {
                if !*is_local {
                    return;
                }

                if let Some(conn) = &self.conn {
                    conn.send(
                        ChessPacket::Movement(Movement {
                            idx: *piece,
                            q: to.0,
                            r: to.1,
                            time_left: None,
                        })
                        .write(),
                    );
                }
            }
            Event::Resign(local) => {
                if !local {
                    return;
                }

                if let Some(conn) = &self.conn {
                    conn.send(ChessPacket::Resign(Resign {}).write());
                }
            }
            Event::Promotion {
                piece,
                kind,
                is_local,
            } => {
                if !*is_local {
                    return;
                }

                if let Some(conn) = &self.conn {
                    conn.send(
                        ChessPacket::Promote(Promote {
                            idx: *piece,
                            kind: *kind,
                        })
                        .write(),
                    )
                }
            }
            Event::GameButtonClick(Button::LeaveRoom) => {
                self.ctx.handle(Event::Disconnected);
            }
            _ => {}
        };
    }
}
