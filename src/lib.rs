mod utils;

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
pub fn start() {
    loadAssets();
}

#[wasm_bindgen]
pub fn on_assets_ready() {
    resetBoard();
    setPieces(&[2101]);
    resumeBoard();
}

#[wasm_bindgen]
pub fn on_hex_clicked(q: u8, r: u8) {
    let pos = ((q & 0xf) << 4) | (r & 0xf);
    movePieces(&[pos.into()])
}
