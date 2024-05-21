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
    piece::Color,
};
use crate::{
    glue::{movePieces, removeTimers, setPieces, setTimers, Event},
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
    color: Color,
    turn: Option<Color>,
    timer: Option<Duration>,
    light: Side,
    dark: Side,
    highlight: HighlightController,
    selected_hex: Option<(u8, u8)>,
}

impl Controller {
    pub fn new(ctx: &Context) -> Self {
        Controller {
            ctx: ctx.clone(),
            board: Board::new(),
            is_host: false,
            is_solo: false,
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
            highlight: HighlightController::new(),
            selected_hex: None,
        }
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
            Event::SetGamemode(mode) => {
                let mode: Gamemode = (*mode).into();
                self.is_solo = mode == Gamemode::Solo;
            }
            Event::JoinedRoom { is_host, .. } => {
                self.is_host = *is_host;
            }
            Event::SetSettings {
                timer,
                host_as_light,
            } => {
                self.turn = Some(Color::Light);
                self.color = if *host_as_light {
                    Color::Light
                } else {
                    Color::Dark
                };
                self.timer = if *timer > 0 {
                    Some(Duration::from_secs((*timer).into()))
                } else {
                    None
                };

                if self.is_host || self.is_solo {
                    self.board.load_default();
                    self.ctx.handle(Event::LoadedBoard(self.board.describe()))
                }
            }
            Event::LoadedBoard(board) => {
                setPieces(board.as_slice());
                self.highlight.reset();

                if self.is_host || self.is_solo {
                    self.ctx.handle(Event::GameStart);
                }
            }
            Event::HexClicked { q, r } => {
                self.highlight.remove(Effect::Light);
                if self.turn.is_none() {
                    return;
                }

                // Is there a selected piece?
                if let Some(from) = self.selected_hex.take() {
                    let piece = self.board.get_at(from.0, from.1).unwrap();
                    if self.board.can_move(piece, *q, *r) {
                        // Move it if we can
                        self.ctx.handle(Event::Movement {
                            from,
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
                        self.selected_hex = Some((*q, *r));
                    }
                }

                self.highlight.send();
            }
            Event::Movement { from, to, is_local } => {
                if !is_local {
                    let valid = match self.turn {
                        Some(color) => color != self.color,
                        None => false,
                    };

                    if !valid {
                        // Not peer's turn.
                        self.ctx.handle(Event::Disconnected);
                        return;
                    }
                }

                let piece = match self.board.get_at(from.0, from.1) {
                    Some(p) => p,
                    None => {
                        // No piece at starting position.
                        self.ctx.handle(Event::Disconnected);
                        return;
                    }
                };

                if !self.board.can_move(piece, to.0, to.1) {
                    // Illegal move
                    self.ctx.handle(Event::Disconnected);
                    return;
                }

                movePieces(self.board.move_piece(*from, *to).as_slice());

                if let Some(winner) = self.check_winner() {
                    self.ctx.handle(Event::GameEnded {
                        won_light: winner.is_light(),
                    });
                    return;
                }
                self.switch_turns();
            }
            Event::TimerExpired => {
                if let Some(loser) = self.turn {
                    self.ctx.handle(Event::GameEnded {
                        won_light: !loser.is_light(),
                    });
                }
            }
            Event::GameStart => {
                removeTimers();
                self.light.time_left = self.timer;
                self.dark.time_left = self.timer;
                self.send_timers();
            }
            Event::GameEnded { won_light } => {
                self.turn = None;

                self.light.update_timer();
                self.dark.update_timer();
                self.send_timers();

                let color = if *won_light {
                    Color::Light
                } else {
                    Color::Dark
                };
                log(&format!("game ended: {:?} won", color));
            }
            _ => {}
        };
    }
}
