use crate::{
    directions::{DirectionIterator, MovementIterator},
    piece::{Piece, PieceKind},
};

#[derive(Debug)]
pub struct Game {
    pub pieces: Vec<Piece>,
}

macro_rules! moves_iter {
    ($self: expr, $piece: expr) => {
        MovementIterator::new(&$self.pieces, $piece).filter(|pos| {
            let is_king = $piece.kind == PieceKind::King;
            !is_king || !$self.is_threatened(pos.0, pos.1, $piece.light)
        })
    };
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

    pub fn is_threatened(&self, q: u8, r: u8, threatened_light: bool) -> bool {
        for pos in MovementIterator::threatened(&self.pieces, threatened_light, q, r) {
            let piece = self.get_at(pos.0, pos.1).unwrap();
            if self.can_capture(piece, q, r, pos.2) {
                return true;
            }
        }
        false
    }

    pub fn can_capture(&self, piece: &Piece, q: u8, r: u8, dir: (i8, i8, u8)) -> bool {
        if piece.kind == PieceKind::Pawn {
            // Normal pawn moves (piece.available()) can't capture.
            return DirectionIterator::pawn_capture(piece.q, piece.r, piece.light)
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
        moves_iter!(self, piece).any(|pos| pos.0 == q && pos.1 == r)
    }

    pub fn available_moves(&self, piece: &Piece) -> Vec<(u8, u8)> {
        moves_iter!(self, piece).map(|pos| (pos.0, pos.1)).collect()
    }

    pub fn move_piece(&mut self, from: (u8, u8), to: (u8, u8)) -> Option<Vec<u16>> {
        let piece = self.get_at(from.0, from.1)?;

        if !self.can_move(piece, to.0, to.1) {
            return None;
        }

        let idx = piece.idx as usize;
        let mut packet = vec![];
        if let Some(other) = self.get_at_mut(to.0, to.1) {
            packet.push(other.movement(0, 0));
        }

        let piece = self.pieces.get_mut(idx).unwrap();
        packet.push(piece.movement(to.0, to.1));

        Some(packet)
    }
}
