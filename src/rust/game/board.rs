use crate::game::{
    directions::{DirectionIterator, MovementIterator},
    piece::{Piece, PieceKind},
};

use super::piece::Color;

#[derive(Debug)]
pub struct Board {
    pub pieces: Vec<Piece>,
}

macro_rules! add_piece {
    ($pieces:expr, $color:expr, $kind:expr, $q:literal, $r_l:literal, $r_d:literal) => {
        $pieces.push(Piece {
            idx: $pieces.len() as u8,
            color: $color,
            q: $q,
            r: if $color.is_light() { $r_l } else { $r_d },
            kind: $kind,
        });
    };
}

impl Board {
    pub fn new() -> Self {
        Board { pieces: vec![] }
    }

    pub fn load_default(&mut self) {
        self.pieces = vec![];
        for side in 0..2 {
            let color = if side == 0 { Color::Light } else { Color::Dark };

            add_piece!(self.pieces, color, PieceKind::Queen, 4, 10, 1);
            add_piece!(self.pieces, color, PieceKind::King, 6, 9, 0);

            add_piece!(self.pieces, color, PieceKind::Knight, 3, 10, 2);
            add_piece!(self.pieces, color, PieceKind::Knight, 7, 8, 0);

            add_piece!(self.pieces, color, PieceKind::Rook, 2, 10, 3);
            add_piece!(self.pieces, color, PieceKind::Rook, 8, 7, 0);

            add_piece!(self.pieces, color, PieceKind::Bishop, 5, 10, 0);
            add_piece!(self.pieces, color, PieceKind::Bishop, 5, 9, 1);
            add_piece!(self.pieces, color, PieceKind::Bishop, 5, 8, 2);

            add_piece!(self.pieces, color, PieceKind::Pawn, 1, 10, 4);
            add_piece!(self.pieces, color, PieceKind::Pawn, 2, 9, 4);
            add_piece!(self.pieces, color, PieceKind::Pawn, 3, 8, 4);
            add_piece!(self.pieces, color, PieceKind::Pawn, 4, 7, 4);
            add_piece!(self.pieces, color, PieceKind::Pawn, 5, 6, 4);
            add_piece!(self.pieces, color, PieceKind::Pawn, 6, 6, 3);
            add_piece!(self.pieces, color, PieceKind::Pawn, 7, 6, 2);
            add_piece!(self.pieces, color, PieceKind::Pawn, 8, 6, 1);
            add_piece!(self.pieces, color, PieceKind::Pawn, 9, 6, 0);
        }
    }

    pub fn load_desc(&mut self, desc: Vec<u16>) {
        self.pieces = desc
            .iter()
            .enumerate()
            .map(|(idx, piece)| Piece::from_desc(idx as u8, *piece))
            .collect();
    }

    pub fn describe(&self) -> Vec<u16> {
        let mut result = vec![];
        for piece in self.pieces.iter() {
            result.push(piece.describe());
        }
        result
    }

    pub fn get_at(&self, q: u8, r: u8) -> Option<&Piece> {
        self.pieces
            .iter()
            .find(|piece| piece.q == q && piece.r == r)
    }

    pub fn get_at_mut(&mut self, q: u8, r: u8) -> Option<&mut Piece> {
        self.pieces
            .iter_mut()
            .find(|piece| piece.q == q && piece.r == r)
    }

    fn avoid_checks<'a>(
        &'a self,
        piece: &'a Piece,
    ) -> impl Fn(&(u8, u8, (i8, i8, u8))) -> bool + 'a {
        move |pos| {
            if piece.kind == PieceKind::King {
                return !self.is_threatened(pos.0, pos.1, piece.color);
            }

            // TODO: Discovered checks
            true
        }
    }

    pub fn is_threatened(&self, q: u8, r: u8, color: Color) -> bool {
        for pos in MovementIterator::threatened(&self.pieces, color, q, r) {
            let piece = self.get_at(pos.0, pos.1).unwrap();
            if self.can_capture(piece, q, r, pos.2) {
                return true;
            }
        }
        false
    }

    fn can_capture(&self, piece: &Piece, q: u8, r: u8, dir: (i8, i8, u8)) -> bool {
        if piece.kind == PieceKind::Pawn {
            // Normal pawn moves (piece.available()) can't capture.
            return DirectionIterator::pawn_capture(piece.q, piece.r, piece.color)
                .any(|pos| pos.0 == q && pos.1 == r);
        }

        // Only check if the piece can move in the opposite direction
        // in which it was discovered.
        piece.available().directions.iter().any(|other| {
            let opposite = other.0 == -dir.0 && other.1 == -dir.1;
            let count = other.2 == 0 || other.2 >= dir.2;
            opposite && count
        })
    }

    pub fn can_move(&self, piece: &Piece, q: u8, r: u8) -> bool {
        MovementIterator::new(&self.pieces, piece)
            .filter(self.avoid_checks(piece))
            .any(|pos| pos.0 == q && pos.1 == r)
    }

    pub fn available_moves(&self, piece: &Piece) -> Vec<(u8, u8)> {
        MovementIterator::new(&self.pieces, piece)
            .filter(self.avoid_checks(piece))
            .map(|pos| (pos.0, pos.1))
            .collect()
    }

    pub fn move_piece(&mut self, from: (u8, u8), to: (u8, u8)) -> Vec<u16> {
        let mut packet = vec![];
        if let Some(piece) = self.get_at_mut(to.0, to.1) {
            packet.push(piece.movement(0, 0));
        }

        let piece = self.get_at_mut(from.0, from.1).unwrap();
        packet.push(piece.movement(to.0, to.1));
        packet
    }

    pub fn get_king(&self, color: Color) -> Option<&Piece> {
        self.pieces
            .iter()
            .find(|p| p.color == color && p.kind == PieceKind::King)
            .filter(|p| p.q != 0 || p.r != 0)
    }
}
