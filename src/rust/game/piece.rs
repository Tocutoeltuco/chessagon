use crate::{
    game::directions::{DirectionIterator, BISHOP, KING, KNIGHT, QUEEN, ROOK},
    glue::promotePieces,
};

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

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum Color {
    Light = 0,
    Dark = 1,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::Light => Color::Dark,
            Color::Dark => Color::Light,
        }
    }

    pub fn is_light(&self) -> bool {
        *self == Color::Light
    }
}

impl From<u8> for PieceKind {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::King,
            1 => Self::Queen,
            2 => Self::Rook,
            3 => Self::Bishop,
            4 => Self::Knight,
            5 => Self::Pawn,
            _ => panic!("invalid piece kind"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Piece {
    pub idx: u8,
    pub kind: PieceKind,
    pub color: Color,
    pub q: u8,
    pub r: u8,
}

impl Piece {
    pub fn from_desc(idx: u8, desc: u16) -> Self {
        let q = (desc >> 4 & 0xf) as u8;
        let r = (desc & 0xf) as u8;
        let kind = (desc >> 8 & 0x7) as u8;
        let dark = (desc >> 11) > 0;
        Piece {
            idx,
            kind: kind.into(),
            color: if dark { Color::Dark } else { Color::Light },
            q,
            r,
        }
    }

    pub fn describe(&self) -> u16 {
        let q: u16 = self.q.into();
        let r: u16 = self.r.into();
        let kind: u16 = (self.kind as u8).into();
        let dark: u16 = (self.color as u8).into();

        dark << 11 | kind << 8 | q << 4 & 0xf0 | r & 0xf
    }

    pub fn movement(&mut self, q: u8, r: u8) -> u16 {
        self.q = q;
        self.r = r;

        let q: u16 = q.into();
        let r: u16 = r.into();
        let idx: u16 = self.idx.into();

        idx << 8 | q << 4 & 0xf0 | r & 0xf
    }

    pub fn is_captured(&self) -> bool {
        self.r == 0 && self.q == 0
    }

    pub fn can_promote(&self) -> bool {
        if self.kind != PieceKind::Pawn {
            return false;
        }
        if self.is_captured() {
            return false;
        }

        let row;
        let col;
        if self.color.is_light() {
            row = 0;
            col = 5;
        } else {
            row = 10;
            col = 15;
        }

        self.r == row || (self.q + self.r) == col
    }

    pub fn promote(&mut self, kind: PieceKind) {
        self.kind = kind;
        let idx: u16 = self.idx.into();
        let kind: u16 = (kind as u8).into();

        promotePieces(&[kind << 8 | idx]);
    }

    pub fn available(&self) -> DirectionIterator {
        if self.kind == PieceKind::Pawn {
            let direction = if self.color.is_light() { -1 } else { 1 };
            let repeat = if self.color.is_light() {
                // Light pawn starting hexes
                (self.r == 6 && self.q > 4) || (self.r > 6 && self.q + self.r == 11)
            } else {
                // Dark pawn starting hexes
                (self.r == 4 && self.q < 6) || (self.r < 5 && self.q + self.r == 9)
            };
            let repeat = if repeat { 2 } else { 1 };

            return DirectionIterator::new(self.q, self.r, vec![(0, direction, repeat)]);
        }

        let container = match self.kind {
            PieceKind::King => KING,
            PieceKind::Queen => QUEEN,
            PieceKind::Rook => ROOK,
            PieceKind::Bishop => BISHOP,
            PieceKind::Knight => KNIGHT,
            PieceKind::Pawn => panic!("shouldnt be here"),
        };

        DirectionIterator::new(self.q, self.r, Vec::from(container))
    }
}
