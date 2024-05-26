use crate::game::piece::{Piece, PieceKind};

use super::piece::Color;

pub const KING: &[(i8, i8, u8)] = &[
    (-1, -1, 1),
    (-1, 0, 1),
    (-1, 1, 1),
    (0, -1, 1),
    (0, 1, 1),
    (1, -1, 1),
    (1, 0, 1),
    (1, 1, 1),
    (-2, 1, 1),
    (-1, 2, 1),
    (1, -2, 1),
    (2, -1, 1),
];
pub const QUEEN: &[(i8, i8, u8)] = &[
    (-1, -1, 0),
    (-1, 0, 0),
    (-1, 1, 0),
    (0, -1, 0),
    (0, 1, 0),
    (1, -1, 0),
    (1, 0, 0),
    (1, 1, 0),
    (-2, 1, 0),
    (-1, 2, 0),
    (1, -2, 0),
    (2, -1, 0),
];
pub const ROOK: &[(i8, i8, u8)] = &[
    (-1, 0, 0),
    (-1, 1, 0),
    (0, -1, 0),
    (0, 1, 0),
    (1, -1, 0),
    (1, 0, 0),
];
pub const BISHOP: &[(i8, i8, u8)] = &[
    (-2, 1, 0),
    (2, -1, 0),
    (1, 1, 0),
    (-1, -1, 0),
    (-1, 2, 0),
    (1, -2, 0),
];
pub const KNIGHT: &[(i8, i8, u8)] = &[
    (-2, -1, 1),
    (-3, 1, 1),
    (-3, 2, 1),
    (-2, 3, 1),
    (-1, 3, 1),
    (1, 2, 1),
    (2, 1, 1),
    (3, -1, 1),
    (3, -2, 1),
    (2, -3, 1),
    (1, -3, 1),
    (-1, -2, 1),
];
// Directions to check for pieces during threatening check
pub const THREATEN_CHECK: &[(i8, i8, u8)] = &[
    (-1, -1, 0),
    (-1, 0, 0),
    (-1, 1, 0),
    (0, -1, 0),
    (0, 1, 0),
    (1, -1, 0),
    (1, 0, 0),
    (1, 1, 0),
    (-2, 1, 0),
    (-1, 2, 0),
    (1, -2, 0),
    (2, -1, 0),
    (-2, -1, 1),
    (-3, 1, 1),
    (-3, 2, 1),
    (-2, 3, 1),
    (-1, 3, 1),
    (1, 2, 1),
    (2, 1, 1),
    (3, -1, 1),
    (3, -2, 1),
    (2, -3, 1),
    (1, -3, 1),
    (-1, -2, 1),
];

fn is_in_bounds(q: i8, r: i8) -> bool {
    if q < 0 || r < 0 {
        return false;
    }
    if q > 10 || r > 10 {
        // Bigger than the board
        return false;
    }
    if q < 5 && r < 5 - q {
        // Top left corner is out of bounds.
        // We are in a hexagon.
        return false;
    }
    if r > 15 - q {
        // Bottom right corner is out of bounds.
        // We are in a hexagon.
        return false;
    }
    true
}

struct SingleDirectionIterator {
    q: u8,
    r: u8,
    dir: (i8, i8, u8),
    i: u8,
}

impl SingleDirectionIterator {
    pub fn new(q: u8, r: u8, dir: (i8, i8, u8)) -> Self {
        SingleDirectionIterator { q, r, dir, i: 0 }
    }
}

impl Iterator for SingleDirectionIterator {
    type Item = (u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.dir.2 != 0 && self.i >= self.dir.2 {
            return None;
        }
        self.i += 1;

        let q = self.q as i8 + self.dir.0;
        let r = self.r as i8 + self.dir.1;

        if !is_in_bounds(q, r) {
            return None;
        }

        self.q = q as u8;
        self.r = r as u8;

        Some((self.q, self.r))
    }
}

pub struct DirectionIterator {
    q: u8,
    r: u8,
    pub directions: Vec<(i8, i8, u8)>,
    idx: usize,
    current: Option<SingleDirectionIterator>,
}

impl DirectionIterator {
    pub fn new(q: u8, r: u8, directions: Vec<(i8, i8, u8)>) -> Self {
        let mut obj = DirectionIterator {
            q,
            r,
            directions,
            idx: 0,
            current: None,
        };
        obj.next_dir();
        obj
    }

    pub fn pawn_capture(q: u8, r: u8, color: Color) -> Self {
        let dir = if color.is_light() { -1 } else { 1 };
        DirectionIterator::new(q, r, vec![(dir, 0, 1), (-dir, dir, 1)])
    }

    pub fn next_dir(&mut self) {
        self.current = self
            .directions
            .get(self.idx)
            .map(|dir| SingleDirectionIterator::new(self.q, self.r, *dir));
        self.idx += 1;
    }
}

impl Iterator for DirectionIterator {
    type Item = (u8, u8, (i8, i8, u8));

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut iter = self.current.take()?;
            if let Some(pos) = iter.next() {
                let dir = (iter.dir.0, iter.dir.1, iter.i);
                self.current = Some(iter);
                return Some((pos.0, pos.1, dir));
            }

            self.next_dir();
        }
    }
}

pub struct MovementIterator<'a> {
    pieces: &'a Vec<Piece>,
    directions: DirectionIterator,
    pawn: bool,
    color: Color,
    only_pieces: bool,
    extra: DirectionIterator,
    passant: Option<(u8, u8)>,
}

impl<'a> MovementIterator<'a> {
    pub fn new(pieces: &'a Vec<Piece>, piece: &Piece, passant: Option<(u8, u8, u8)>) -> Self {
        let pawn = piece.kind == PieceKind::Pawn;
        let extra = if pawn {
            DirectionIterator::pawn_capture(piece.q, piece.r, piece.color)
        } else {
            DirectionIterator::new(0, 0, vec![])
        };

        let passant = passant
            .map(|(idx, q, r)| (pieces.get(idx as usize), q, r))
            .filter(|(pass, _, _)| pass.is_some())
            .map(|(pass, q, r)| (pass.unwrap(), q, r))
            .filter(|(pass, _, _)| pass.color != piece.color)
            .map(|(_, q, r)| (q, r));

        MovementIterator {
            pieces,
            directions: piece.available(),
            pawn,
            color: piece.color,
            only_pieces: false,
            extra,
            passant,
        }
    }

    pub fn threatened(pieces: &'a Vec<Piece>, color: Color, q: u8, r: u8) -> Self {
        MovementIterator {
            pieces,
            directions: DirectionIterator::new(q, r, Vec::from(THREATEN_CHECK)),
            pawn: false,
            color,
            only_pieces: true,
            extra: DirectionIterator::new(0, 0, vec![]),
            passant: None,
        }
    }

    fn get_at(&self, q: u8, r: u8) -> Option<&Piece> {
        self.pieces
            .iter()
            .find(|piece| piece.q == q && piece.r == r)
    }
}

impl<'a> Iterator for MovementIterator<'a> {
    type Item = (u8, u8, (i8, i8, u8));

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(pos) = self.extra.next() {
            if let Some((q, r)) = self.passant {
                if q == pos.0 && r == pos.1 {
                    return Some(pos);
                }
            }

            if let Some(piece) = self.get_at(pos.0, pos.1) {
                if piece.color != self.color {
                    return Some(pos);
                }
            }
        }

        if self.pawn && self.only_pieces {
            // Already checked capture moves (extra)
            return None;
        }

        loop {
            let pos = self.directions.next()?;

            if let Some(other) = self.get_at(pos.0, pos.1) {
                let same_color = other.color == self.color;

                self.directions.next_dir();
                if self.pawn || same_color {
                    continue;
                }
            } else if self.only_pieces {
                continue;
            }

            return Some(pos);
        }
    }
}
