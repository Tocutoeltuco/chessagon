use rand::{rngs::SmallRng, SeedableRng};
use web_time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Gamemode {
    Solo,
    Online,
    Bot,
}

impl From<u8> for Gamemode {
    fn from(value: u8) -> Self {
        match value {
            0 => Gamemode::Solo,
            1 => Gamemode::Online,
            2 => Gamemode::Bot,
            _ => panic!("invalid gamemode"),
        }
    }
}

impl From<Gamemode> for u8 {
    fn from(value: Gamemode) -> u8 {
        match value {
            Gamemode::Solo => 0,
            Gamemode::Online => 1,
            Gamemode::Bot => 2,
        }
    }
}

pub fn new_rng() -> SmallRng {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time travel?")
        .as_millis() as u64;
    SmallRng::seed_from_u64(now)
}
