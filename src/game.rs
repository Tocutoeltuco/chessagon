fn is_in_bounds(q: u8, r: u8) -> bool {
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

    pub fn available(&self) -> Vec<Vec<(u8, u8)>> {
        // q, r, repeat
        let mut dirs: Vec<(i8, i8, u8)> = vec![(0, 0, 1)];

        match self.kind {
            PieceKind::King => {
                for q in -1..2 {
                    for r in -1..2 {
                        dirs.push((q, r, 1));
                    }
                }

                dirs.push((-2, 1, 1));
                dirs.push((-1, 2, 1));
                dirs.push((1, -2, 1));
                dirs.push((2, -1, 1));
            }
            PieceKind::Queen => {
                for q in -1..2 {
                    for r in -1..2 {
                        dirs.push((q, r, 10));
                    }
                }

                dirs.push((-2, 1, 10));
                dirs.push((-1, 2, 10));
                dirs.push((1, -2, 10));
                dirs.push((2, -1, 10));
            }
            PieceKind::Rook => {
                for q in -1..2 {
                    for r in -1..2 {
                        if q != r {
                            dirs.push((q, r, 10));
                        }
                    }
                }
            }
            PieceKind::Bishop => {
                dirs.push((-2, 1, 5));
                dirs.push((2, -1, 5));

                dirs.push((1, 1, 5));
                dirs.push((-1, -1, 5));

                dirs.push((-1, 2, 5));
                dirs.push((1, -2, 5));
            }
            PieceKind::Knight => {
                dirs.push((-2, -1, 1));
                dirs.push((-3, 1, 1));

                dirs.push((-3, 2, 1));
                dirs.push((-2, 3, 1));

                dirs.push((-1, 3, 1));
                dirs.push((1, 2, 1));

                // Mirror
                dirs.push((2, 1, 1));
                dirs.push((3, -1, 1));

                dirs.push((3, -2, 1));
                dirs.push((2, -3, 1));

                dirs.push((1, -3, 1));
                dirs.push((-1, -2, 1));
            }
            PieceKind::Pawn => {
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

                dirs.push((0, direction, repeat));
            }
        };

        let mut moves: Vec<Vec<(u8, u8)>> = vec![];
        for dir in dirs.iter() {
            let mut cont: Vec<(i8, i8)> = vec![];
            let mut q = 0;
            let mut r = 0;

            for _ in 0..dir.2 {
                q += dir.0;
                r += dir.1;
                cont.push((q, r));
            }

            moves.push(
                cont.iter()
                    // Convert to positions
                    .map(|(q, r)| (self.q as i8 + q, self.r as i8 + r))
                    .filter(|(q, r)| q >= &0 && r >= &0)
                    // Cast to u8
                    .map(|(q, r)| (q as u8, r as u8))
                    // Check in bounds
                    .filter(|(q, r)| is_in_bounds(*q, *r))
                    .collect(),
            );
        }

        moves
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

        for dir in piece.available().iter() {
            for pos in dir.iter() {
                if let Some(other) = self.get_at(pos.0, pos.1) {
                    let is_same = other.idx == piece.idx;
                    let capture = other.light != piece.light;

                    if is_same || (capture && piece.kind != PieceKind::Pawn) {
                        moves.push((pos.0, pos.1));
                    }
                    break;
                } else {
                    moves.push((pos.0, pos.1));
                }
            }
        }

        if piece.kind == PieceKind::Pawn {
            let direction = if piece.light { 1 } else { -1 };
            for delta in [(-1, 0), (1, -1)] {
                let q = piece.q as i8 + delta.0 * direction;
                let r = piece.r as i8 + delta.1 * direction;

                if q < 0 || r < 0 {
                    continue;
                }

                let q = q as u8;
                let r = r as u8;

                if let Some(other) = self.get_at(q, r) {
                    if other.light != piece.light {
                        moves.push((q, r));
                    }
                }
            }
        }

        moves
    }
}
