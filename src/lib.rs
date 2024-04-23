mod directions;
mod game;
mod piece;

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
    selected: Option<(u8, u8)>,
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
    if let Some(piece) = ctx.game.get_at(q, r) {
        ctx.selected = Some((q, r));

        let mut moves = ctx.game.available_moves(piece);
        moves.push((piece.q, piece.r));
        let moves: Vec<u8> = moves.iter().map(|(q, r)| q << 4 & 0xf0 | r & 0xf).collect();
        highlight(moves.as_slice());
        return;
    }

    if let Some(prev) = ctx.selected {
        if let Some(piece) = ctx.game.get_at_mut(prev.0, prev.1) {
            movePieces(&[piece.movement(q, r)]);
        }
    }

    ctx.selected = None;
    highlight(&[]);
}
