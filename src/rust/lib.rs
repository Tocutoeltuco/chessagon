mod directions;
mod game;
mod names;
mod piece;

use game::Game;
use names::NameGenerator;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/rust/glue.js")]
extern "C" {
    pub fn setScene(idx: i8);
    pub fn joinResponse(resp: String);

    pub fn setPieces(pieces: &[u16]);
    pub fn movePieces(pieces: &[u16]);
    pub fn highlight(hexes: &[u16]);
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Gamemode {
    Solo,
    Online,
    Bot,
}

impl From<u8> for Gamemode {
    fn from(value: u8) -> Self {
        match value {
            0 => Gamemode::Solo,
            1 => Gamemode::Online,
            2 => Gamemode::Bot,
            _ => panic!("invalid gamemode"),
        }
    }
}

impl From<Gamemode> for u8 {
    fn from(value: Gamemode) -> u8 {
        match value {
            Gamemode::Solo => 0,
            Gamemode::Online => 1,
            Gamemode::Bot => 2,
        }
    }
}

#[wasm_bindgen]
pub struct Context {
    game: Game,
    selected: Option<(u8, u8)>,
    gamemode: Gamemode,
    generator: NameGenerator,
}

#[wasm_bindgen]
pub fn create_context() -> Context {
    Context {
        game: Game::new(),
        selected: None,
        gamemode: Gamemode::Solo,
        generator: NameGenerator::new(),
    }
}

#[wasm_bindgen]
pub fn start(ctx: &mut Context) {
    setPieces(ctx.game.describe().as_slice());
    highlight(&[]);

    setScene(0);
}

#[wasm_bindgen]
pub fn new_player_name(ctx: &mut Context) -> String {
    ctx.generator.next().unwrap()
}

#[wasm_bindgen]
pub fn set_gamemode(ctx: &mut Context, mode: u8) {
    ctx.gamemode = mode.into();

    if ctx.gamemode == Gamemode::Solo {
        // Show settings
        setScene(3);
    } else if ctx.gamemode == Gamemode::Online {
        // Show register
        setScene(1);
    }
}

#[wasm_bindgen]
pub fn registered(ctx: &mut Context, name: String) {
    // Show online lobby
    setScene(2);
}

#[wasm_bindgen]
pub fn create_room(ctx: &mut Context) {
    // Show settings
    setScene(3);
}

#[wasm_bindgen]
pub fn join_room(ctx: &mut Context, room: String) {}

#[wasm_bindgen]
pub fn set_settings(ctx: &mut Context, timer: Option<u32>, play_light: bool) {
    setScene(-1);
}

#[wasm_bindgen]
pub fn on_menu_hidden(ctx: &mut Context, menu: u8) {
    // gamemode selection
    if menu == 0 {
        panic!("wasn't supposed to close this menu");
    }

    // register menu
    if menu == 1 {
        // Send back to gamemode selection
        setScene(0);
    // online menu
    } else if menu == 2 {
        // Send back to registration
        setScene(1);
    // settings menu
    } else if menu == 3 {
        if ctx.gamemode == Gamemode::Solo {
            // Send back to gamemode selection
            setScene(0);
        } else {
            // Send back to online menu
            setScene(2);
        }
    }
}

#[wasm_bindgen]
pub fn on_hex_clicked(ctx: &mut Context, q: u8, r: u8) {
    if let Some(piece) = ctx.game.get_at(q, r) {
        ctx.selected = Some((q, r));

        let mut moves = ctx.game.available_moves(piece);
        moves.push((piece.q, piece.r));
        let moves: Vec<u16> = moves
            .iter()
            .map(|(q, r)| q << 4 & 0xf0 | r & 0xf)
            .map(|hex| hex as u16)
            .map(|hex| 1 << 8 | hex) // light effect
            .collect();
        highlight(moves.as_slice());
        return;
    }

    if let Some(pos) = ctx.selected {
        if let Some(packet) = ctx.game.move_piece(pos, (q, r)) {
            movePieces(packet.as_slice());
        }
    }

    ctx.selected = None;
    highlight(&[]);
}
