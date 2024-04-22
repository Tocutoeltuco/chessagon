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

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum PieceKind {
    King = 0,
    Queen = 1,
    Rook = 2,
    Bishop = 3,
    Knight = 4,
    Pawn = 5,
}

#[derive(Copy, Clone, Debug)]
struct Piece {
    idx: u8,
    kind: PieceKind,
    light: bool,
    q: u8,
    r: u8,
}

impl Piece {
    fn describe(&self) -> u16 {
        let q: u16 = self.q.into();
        let r: u16 = self.r.into();
        let kind: u16 = (self.kind as u8).into();
        let mut light: u16 = 0;

        if self.light {
            light = 1 << 11;
        }

        light | kind << 8 | q << 4 & 0xf0 | r & 0xf
    }

    fn movement(&mut self, q: u8, r: u8) -> u16 {
        self.q = q;
        self.r = r;

        let q: u16 = q.into();
        let r: u16 = r.into();
        let idx: u16 = self.idx.into();

        idx << 8 | q << 4 & 0xf0 | r & 0xf
    }
}

#[derive(Debug)]
struct Game {
    pieces: Vec<Piece>,
}

macro_rules! add_piece {
    ($pieces:expr, $side:expr, $kind:expr, $q:literal, $r_l:literal, $r_d:literal) => {
        $pieces.push(Piece {
            idx: $pieces.len() as u8,
            light: $side == 0,
            q: $q,
            r: if $side == 0 { $r_l } else { $r_d },
            kind: $kind,
        });
    };
}

impl Game {
    fn new() -> Self {
        let mut pieces = vec![];

        for side in 0..2 {
            add_piece!(pieces, side, PieceKind::Queen, 4, 10, 1);
            add_piece!(pieces, side, PieceKind::King, 6, 9, 0);

            add_piece!(pieces, side, PieceKind::Knight, 3, 10, 2);
            add_piece!(pieces, side, PieceKind::Knight, 7, 8, 0);

            add_piece!(pieces, side, PieceKind::Rook, 2, 10, 3);
            add_piece!(pieces, side, PieceKind::Rook, 8, 7, 0);

            add_piece!(pieces, side, PieceKind::Bishop, 5, 10, 0);
            add_piece!(pieces, side, PieceKind::Bishop, 5, 9, 1);
            add_piece!(pieces, side, PieceKind::Bishop, 5, 8, 2);

            add_piece!(pieces, side, PieceKind::Pawn, 1, 10, 4);
            add_piece!(pieces, side, PieceKind::Pawn, 2, 9, 4);
            add_piece!(pieces, side, PieceKind::Pawn, 3, 8, 4);
            add_piece!(pieces, side, PieceKind::Pawn, 4, 7, 4);
            add_piece!(pieces, side, PieceKind::Pawn, 5, 6, 4);
            add_piece!(pieces, side, PieceKind::Pawn, 6, 6, 3);
            add_piece!(pieces, side, PieceKind::Pawn, 7, 6, 2);
            add_piece!(pieces, side, PieceKind::Pawn, 8, 6, 1);
            add_piece!(pieces, side, PieceKind::Pawn, 9, 6, 0);
        }

        Game { pieces }
    }

    fn describe(&self) -> Vec<u16> {
        let mut result = vec![];
        for piece in self.pieces.iter() {
            result.push(piece.describe());
        }
        result
    }
}

#[wasm_bindgen]
pub fn start() {
    loadAssets();
}

#[wasm_bindgen]
pub fn on_assets_ready() {
    resetBoard();
    let game = Game::new();
    setPieces(game.describe().as_slice());
    resumeBoard();
}

#[wasm_bindgen]
pub fn on_hex_clicked(q: u8, r: u8) {
    let mut king = Piece {
        idx: 0,
        q: 3,
        r: 5,
        light: true,
        kind: PieceKind::King,
    };
    movePieces(&[king.movement(q, r)])
}
