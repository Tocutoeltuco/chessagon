mod game;

use game::Game;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/glue.js")]
extern "C" {
    pub fn loadAssets();
    pub fn resetBoard();
    pub fn resumeBoard();
    pub fn pauseBoard();
    pub fn setPieces(pieces: &[u16]);
    pub fn movePieces(pieces: &[u16]);
    pub fn highlight(hexes: &[u8]);
}

#[wasm_bindgen]
pub struct Context {
    game: Game,
    selected: Option<[u8; 2]>,
}

#[wasm_bindgen]
pub fn create_context() -> Context {
    Context {
        game: Game::new(),
        selected: None,
    }
}

#[wasm_bindgen]
pub fn start(_ctx: &mut Context) {
    loadAssets();
}

#[wasm_bindgen]
pub fn on_assets_ready(ctx: &mut Context) {
    resetBoard();
    setPieces(ctx.game.describe().as_slice());
    resumeBoard();
}

#[wasm_bindgen]
pub fn on_hex_clicked(ctx: &mut Context, q: u8, r: u8) {
    if ctx.game.get_at(q, r).is_some() {
        ctx.selected = Some([q, r]);
        highlight(&[q << 4 & 0xf0 | r & 0xf]);
        return;
    }

    if let Some(prev) = ctx.selected {
        if let Some(piece) = ctx.game.get_at(prev[0], prev[1]) {
            movePieces(&[piece.movement(q, r)]);
        }
    }

    ctx.selected = None;
    highlight(&[]);
}
