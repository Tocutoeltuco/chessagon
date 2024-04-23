use crate::{
    directions::DirectionIterator,
    piece::{Piece, PieceKind},
};

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

    pub fn available_moves(&self, piece: &Piece) -> Vec<(u8, u8)> {
        let mut moves: Vec<(u8, u8)> = vec![];
        let is_pawn = piece.kind == PieceKind::Pawn;

        for dir in piece.available().iter_mut() {
            for pos in dir {
                if let Some(other) = self.get_at(pos.0, pos.1) {
                    let capture = other.light != piece.light;

                    if capture && !is_pawn {
                        moves.push((pos.0, pos.1));
                    }
                    break;
                } else {
                    moves.push((pos.0, pos.1));
                }
            }
        }

        if is_pawn {
            // Check if we can capture
            let direction = if piece.light { 1 } else { -1 };

            for mut dir in [
                // Left and right "diagonals"
                DirectionIterator::new(piece.q, piece.r, (-direction, 0, 1)),
                DirectionIterator::new(piece.q, piece.r, (direction, -1, 1)),
            ] {
                let pos = dir.next().unwrap();
                if let Some(other) = self.get_at(pos.0, pos.1) {
                    if other.light != piece.light {
                        moves.push((pos.0, pos.1));
                    }
                }
            }
        }

        moves
    }
}
