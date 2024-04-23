use crate::directions::{DirectionIterator, BISHOP, KING, KNIGHT, QUEEN, ROOK};

#[derive(Copy, Clone, Debug, PartialEq)]
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
        let light: u16 = if self.light { 1 << 11 } else { 0 };

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

    pub fn available(&self) -> Vec<DirectionIterator> {
        if self.kind == PieceKind::Pawn {
            let direction = if self.light { -1 } else { 1 };
            let repeat = match (self.light, self.q, self.r) {
                // Light pawn starting squares
                (true, q, 6) if q > 4 => 2,
                (true, q, r) if r == 11 - q => 2,
                // Dark pawn starting squares
                (false, q, 4) if q < 6 => 2,
                (false, q, r) if r == 9 - q => 2,
                _ => 1,
            };

            return vec![DirectionIterator::new(
                self.q,
                self.r,
                (0, direction, repeat),
            )];
        }

        let container = match self.kind {
            PieceKind::King => KING,
            PieceKind::Queen => QUEEN,
            PieceKind::Rook => ROOK,
            PieceKind::Bishop => BISHOP,
            PieceKind::Knight => KNIGHT,
            PieceKind::Pawn => panic!("shouldnt be here"),
        };

        container
            .iter()
            .map(|dir| DirectionIterator::new(self.q, self.r, *dir))
            .collect()
    }
}
