use crate::glue::highlight;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Effect {
    Light = 0,
    Check = 1,
}

pub struct HighlightController([[u8; 11]; 11]);

impl Default for HighlightController {
    fn default() -> Self {
        HighlightController::new()
    }
}

impl HighlightController {
    pub fn new() -> Self {
        HighlightController(Default::default())
    }

    pub fn send(&self) {
        let mut packet = vec![];
        for q in 0..11 {
            for r in 0..11 {
                if self.0[q][r] == 0 {
                    continue;
                }

                let effects = self.0[q][r] as u16;
                let q = q as u16;
                let r = r as u16;
                packet.push(effects << 8 | q << 4 | r);
            }
        }
        highlight(packet.as_slice());
    }

    pub fn reset(&mut self) {
        self.0 = Default::default();
        highlight(&[]);
    }

    pub fn add<'a, I>(&mut self, effect: Effect, hexes: I)
    where
        I: Iterator<Item = &'a (u8, u8)>,
    {
        let effect: u8 = 1 << (effect as u8);
        for (q, r) in hexes {
            self.0[*q as usize][*r as usize] |= effect;
        }
    }

    pub fn remove(&mut self, effect: Effect) {
        let mask: u8 = !(1 << (effect as u8));
        for q in 0..11 {
            for r in 0..11 {
                self.0[q][r] &= mask;
            }
        }
    }
}
