#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum PieceKind {
    King = 0,
    Queen = 1,
    Rook = 2,
    Bishop = 3,
    Knight = 4,
    Pawn = 5,
}

#[derive(Copy, Clone, Debug)]
pub struct Piece {
    pub idx: u8,
    pub kind: PieceKind,
    pub light: bool,
    pub q: u8,
    pub r: u8,
}

impl Piece {
    pub fn describe(&self) -> u16 {
        let q: u16 = self.q.into();
        let r: u16 = self.r.into();
        let kind: u16 = (self.kind as u8).into();
        let mut light: u16 = 0;

        if self.light {
            light = 1 << 11;
        }

        light | kind << 8 | q << 4 & 0xf0 | r & 0xf
    }

    pub fn movement(&mut self, q: u8, r: u8) -> u16 {
        self.q = q;
        self.r = r;

        let q: u16 = q.into();
        let r: u16 = r.into();
        let idx: u16 = self.idx.into();

        idx << 8 | q << 4 & 0xf0 | r & 0xf
    }
}

#[derive(Debug)]
pub struct Game {
    pub pieces: Vec<Piece>,
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
    pub fn new() -> Self {
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

    pub fn describe(&self) -> Vec<u16> {
        let mut result = vec![];
        for piece in self.pieces.iter() {
            result.push(piece.describe());
        }
        result
    }

    pub fn get_at(&mut self, q: u8, r: u8) -> Option<&mut Piece> {
        self.pieces
            .iter_mut()
            .find(|piece| piece.q == q && piece.r == r)
    }
}
