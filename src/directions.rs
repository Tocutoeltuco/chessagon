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

pub struct DirectionIterator {
    q: u8,
    r: u8,
    dir: (i8, i8, u8),
    count: u8,
}

impl DirectionIterator {
    pub fn new(q: u8, r: u8, dir: (i8, i8, u8)) -> Self {
        DirectionIterator {
            q,
            r,
            dir,
            count: 0,
        }
    }
}

impl Iterator for DirectionIterator {
    type Item = (u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.dir.2 != 0 {
            if self.count >= self.dir.2 {
                return None;
            }
            self.count += 1;
        }

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
