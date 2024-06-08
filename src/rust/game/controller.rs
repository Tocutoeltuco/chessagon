use web_time::{Duration, SystemTime};

use wasm_bindgen::prelude::wasm_bindgen;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

use super::{
    board::Board,
    highlight::{Effect, HighlightController},
    piece::{Color, PieceKind},
};
use crate::{
    chat::Chat,
    glue::{
        hideChat, movePieces, removeTimers, setBoardPerspective, setPieces, setTimers, showButtons,
        showChat, showPromotionPrompt, Button, Event,
    },
    utils::Gamemode,
    Context,
};

struct Side {
    color: Color,
    time_left: Option<Duration>,
    time_active_at: Option<SystemTime>,
}

impl Side {
    fn update_timer(&mut self) {
        if let Some(since) = self.time_active_at.take() {
            let duration = SystemTime::now()
                .duration_since(since)
                .expect("time travel");
            self.time_left = Some(self.time_left.unwrap() - duration);
        }
    }
}

pub struct Controller {
    ctx: Context,
    board: Board,
    is_host: bool,
    is_solo: bool,
    is_connected: bool,
    loaded_board: bool,
    color: Color,
    turn: Option<Color>,
    timer: Option<Duration>,
    light: Side,
    dark: Side,
    name: String,
    opp_name: String,
    highlight: HighlightController,
    selected_hex: Option<(u8, u8)>,
    promoting: Option<u8>,
}

impl Controller {
    pub fn new(ctx: &Context) -> Self {
        Controller {
            ctx: ctx.clone(),
            board: Board::new(),
            is_host: false,
            is_solo: false,
            is_connected: false,
            loaded_board: false,
            color: Color::Light,
            turn: None,
            timer: None,
            light: Side {
                color: Color::Light,
                time_left: None,
                time_active_at: None,
            },
            dark: Side {
                color: Color::Dark,
                time_left: None,
                time_active_at: None,
            },
            name: "".to_owned(),
            opp_name: "".to_owned(),
            highlight: HighlightController::new(),
            selected_hex: None,
            promoting: None,
        }
    }

    fn get_color(&self, is_local: bool) -> Color {
        if is_local {
            self.color
        } else {
            self.color.opposite()
        }
    }

    fn try_start(&self) {
        if self.turn.is_some() {
            return;
        }

        if !self.is_solo {
            if !self.is_host {
                return;
            }

            if !self.is_connected {
                return;
            }

            if !self.loaded_board {
                return;
            }
        }

        self.ctx.handle(Event::GameStart);
    }

    fn check_winner(&mut self) -> Option<Color> {
        let mut winner = None;
        self.highlight.remove(Effect::Check);

        for color in [Color::Light, Color::Dark].iter() {
            match self.board.get_king(*color) {
                Some(king) => {
                    if self.board.is_threatened(king.q, king.r, *color) {
                        self.highlight.add(Effect::Check, [(king.q, king.r)].iter());
                    }
                }
                None => {
                    winner = Some(color.opposite());
                }
            };
        }

        self.highlight.send();
        winner
    }

    fn send_timers(&self) {
        if self.timer.is_none() {
            removeTimers();
            return;
        }

        let active = self
            .turn
            .map(|c| {
                if c.is_light() {
                    &self.light
                } else {
                    &self.dark
                }
            })
            .filter(|s| s.time_active_at.is_some())
            .map(|s| if s.color.is_light() { 0 } else { 1 })
            .unwrap_or(-1);

        setTimers(
            self.light.time_left.unwrap().as_secs().try_into().unwrap(),
            self.dark.time_left.unwrap().as_secs().try_into().unwrap(),
            active,
        );
    }

    fn switch_turns(&mut self) {
        let (active, inactive) = match self.turn.unwrap() {
            Color::Light => (&mut self.light, &mut self.dark),
            Color::Dark => (&mut self.dark, &mut self.light),
        };
        self.turn = Some(inactive.color);
        if self.is_solo {
            self.color = inactive.color;
        }

        if self.timer.is_none() {
            return;
        }

        inactive.time_active_at = Some(SystemTime::now());
        active.update_timer();
        self.send_timers();
    }

    pub fn on_event(&mut self, evt: &Event) {
        match evt {
            Event::Connected(..) => {
                self.is_connected = true;
                self.try_start();
            }
            Event::Disconnected => {
                if self.is_connected {
                    self.is_connected = false;
                    self.turn = None;
                    Chat::disconnected();
                    hideChat();
                }
            }
            Event::SetGamemode(mode) => {
                let mode: Gamemode = (*mode).into();
                self.is_solo = mode == Gamemode::Solo;
                hideChat();
            }
            Event::Register(name) => {
                self.name = name.clone();
            }
            Event::Handshake(name) => {
                Chat::connected(name);
                showChat();
                self.opp_name = name.clone();
            }
            Event::JoinedRoom { is_host, .. } => {
                self.is_host = *is_host;
            }
            Event::SetSettings {
                timer,
                host_as_light,
            } => {
                self.color = if *host_as_light {
                    Color::Light
                } else {
                    Color::Dark
                };
                if !self.is_solo && !self.is_host {
                    self.color = self.color.opposite();
                }
                self.timer = if *timer > 0 {
                    Some(Duration::from_secs((*timer).into()))
                } else {
                    None
                };
                setBoardPerspective(self.is_solo || self.color.is_light());

                if self.is_host || self.is_solo {
                    self.board.load_default();
                    self.ctx.handle(Event::LoadedBoard(self.board.describe()))
                }
            }
            Event::LoadedBoard(board) => {
                if !self.is_host {
                    self.board.load_desc(board.clone());
                }
                setPieces(board.as_slice());
                self.highlight.reset();
                self.loaded_board = true;
                self.try_start();
            }
            Event::HexClicked { q, r } => {
                self.highlight.remove(Effect::Light);
                if self.promoting.is_some() {
                    return;
                }
                let turn = match self.turn {
                    Some(c) => c,
                    None => return,
                };
                if turn != self.color {
                    return;
                }

                // Is there a selected piece?
                if let Some(from) = self.selected_hex.take() {
                    if from.0 == *q && from.1 == *r {
                        // Unselect piece
                        self.highlight.send();
                        return;
                    }

                    let piece = self.board.get_at(from.0, from.1).unwrap();
                    if self.board.can_move(piece, *q, *r) {
                        // Move it if we can
                        self.ctx.handle(Event::Movement {
                            piece: piece.idx,
                            to: (*q, *r),
                            is_local: true,
                        });
                    }
                }

                if let Some(piece) = self.board.get_at(*q, *r) {
                    // Are we trying to select a piece of our color?
                    if piece.color == self.color {
                        let moves = self.board.available_moves(piece);
                        self.highlight.add(Effect::Light, moves.iter());
                        self.highlight.add(Effect::Light, [(*q, *r)].iter());
                        self.selected_hex = Some((*q, *r));
                    }
                }

                self.highlight.send();
            }
            Event::Movement {
                piece: idx,
                to,
                is_local,
            } => {
                let turn = match self.turn {
                    Some(color) => color,
                    None => {
                        if !is_local {
                            self.ctx.handle(Event::Disconnected);
                        }
                        return;
                    }
                };
                if !is_local && turn == self.color {
                    // Not peer's turn.
                    self.ctx.handle(Event::Disconnected);
                    return;
                }

                let piece = match self.board.get_piece(*idx) {
                    Some(p) if p.color == turn => p,
                    _ => {
                        // Invalid piece
                        self.ctx.handle(Event::Disconnected);
                        return;
                    }
                };

                if !self.board.can_move(piece, to.0, to.1) {
                    // Illegal move
                    self.ctx.handle(Event::Disconnected);
                    return;
                }

                movePieces(self.board.move_piece((piece.q, piece.r), *to).as_slice());

                if let Some(winner) = self.check_winner() {
                    self.ctx.handle(Event::GameEnded {
                        won_light: winner.is_light(),
                    });
                    return;
                }

                let piece = self.board.get_piece(*idx).unwrap();
                if piece.can_promote() {
                    if *is_local {
                        self.ctx.handle(Event::PromotionPrompt(*idx));
                    }
                } else {
                    self.switch_turns();
                }
            }
            Event::Promotion {
                piece,
                kind,
                is_local,
            } => {
                let turn = match self.turn {
                    Some(color) => color,
                    None => {
                        if !is_local {
                            self.ctx.handle(Event::Disconnected);
                        }
                        return;
                    }
                };
                if !is_local && turn == self.color {
                    // Not peer's turn.
                    self.ctx.handle(Event::Disconnected);
                    return;
                }

                let kind: PieceKind = (*kind).into();
                if !matches!(
                    kind,
                    PieceKind::Queen | PieceKind::Knight | PieceKind::Rook | PieceKind::Bishop
                ) {
                    // Invalid promotion.
                    self.ctx.handle(Event::Disconnected);
                    return;
                }

                let piece = match self.board.get_piece_mut(*piece) {
                    Some(p) if p.color == turn => p,
                    _ => {
                        // Invalid piece
                        self.ctx.handle(Event::Disconnected);
                        return;
                    }
                };

                if !piece.can_promote() {
                    // Invalid promotion.
                    self.ctx.handle(Event::Disconnected);
                    return;
                }
                piece.promote(kind);
                self.switch_turns();
            }
            Event::TimerExpired => {
                if let Some(loser) = self.turn {
                    let loser = loser.is_light();
                    Chat::timer_expired(loser);
                    self.ctx.handle(Event::GameEnded { won_light: !loser });
                }
            }
            Event::Resign(local) => {
                let is_light = self.get_color(*local).is_light();
                Chat::resign(is_light);
                self.ctx.handle(Event::GameEnded {
                    won_light: !is_light,
                });
            }
            Event::ChatMessage { is_local, content } => {
                let color = self.get_color(*is_local);
                let name = if *is_local {
                    &self.name
                } else {
                    &self.opp_name
                };
                Chat::player_message(color.is_light(), name, content);
            }
            Event::GameStart => {
                removeTimers();
                self.light.time_left = self.timer;
                self.dark.time_left = self.timer;
                self.send_timers();
                self.promoting = None;
                self.turn = Some(Color::Light);

                Chat::game_start();
                if !self.is_solo {
                    showButtons(&[Button::LeaveRoom.into(), Button::Resign.into()]);
                } else {
                    self.color = Color::Light;
                    showButtons(&[Button::LeaveRoom.into()]);
                }
            }
            Event::GameEnded { won_light } => {
                self.turn = None;
                self.promoting = None;

                self.light.update_timer();
                self.dark.update_timer();
                self.send_timers();

                Chat::game_end(*won_light);
                if self.is_solo || self.is_host {
                    showButtons(&[Button::LeaveRoom.into(), Button::PlayAgain.into()]);
                } else {
                    showButtons(&[Button::LeaveRoom.into()]);
                }
            }
            Event::GameButtonClick(btn) => match btn {
                Button::Resign => {
                    self.ctx.handle(Event::Resign(true));
                }
                Button::LeaveRoom => {
                    self.turn = None;
                    self.promoting = None;
                }
                _ => {}
            },
            Event::PromotionPrompt(idx) => {
                let piece = match self.board.get_piece(*idx) {
                    Some(piece) => piece,
                    None => return,
                };

                self.promoting = Some(*idx);
                showPromotionPrompt(piece.color as u8, piece.q, piece.r);
            }
            Event::PromotionResponse(kind) => {
                let piece = match self.promoting.take().and_then(|i| self.board.get_piece(i)) {
                    Some(p) => p,
                    None => return,
                };
                if !piece.can_promote() {
                    return;
                }
                self.ctx.handle(Event::Promotion {
                    piece: piece.idx,
                    kind: *kind,
                    is_local: true,
                });
            }
            _ => {}
        };
    }
}
