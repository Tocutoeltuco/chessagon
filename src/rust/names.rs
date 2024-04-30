use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};
use web_time::{SystemTime, UNIX_EPOCH};

pub const LEFT: &[&str] = &include!(concat!(env!("OUT_DIR"), "/name-left.rs"));
pub const RIGHT: &[&str] = &include!(concat!(env!("OUT_DIR"), "/name-right.rs"));

pub struct NameGenerator {
    rng: SmallRng,
}

impl NameGenerator {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time travel?")
            .as_secs();
        NameGenerator {
            rng: SmallRng::seed_from_u64(now),
        }
    }
}

impl Iterator for NameGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let left = LEFT.choose(&mut self.rng).unwrap();
        let right = RIGHT.choose(&mut self.rng).unwrap();

        Some(format!("{}-{}", left, right))
    }
}
