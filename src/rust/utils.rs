use futures::StreamExt;
use rand::{rngs::SmallRng, SeedableRng};
use wasm_bindgen::closure::Closure;
use web_time::{Instant, SystemTime, UNIX_EPOCH};

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

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    fn setTimeout(closure: &Closure<dyn FnMut()>, millis: u32) -> f64;
}

pub async fn wait_until(at: Instant) {
    let delay = at - Instant::now();
    let delay = delay.as_millis();
    if delay == 0 {
        return;
    }

    let (mut tx, mut rx) = futures_channel::mpsc::channel(1);
    let handler: Box<dyn FnMut()> = Box::new(move || tx.try_send(()).unwrap());
    let handler = Closure::wrap(handler);

    setTimeout(&handler, delay as u32);

    rx.next().await;
}
