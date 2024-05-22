use rand::{seq::SliceRandom, Rng};

pub const LEFT: &[&str] = &include!(concat!(env!("OUT_DIR"), "/name-left.rs"));
pub const RIGHT: &[&str] = &include!(concat!(env!("OUT_DIR"), "/name-right.rs"));

pub fn new_name<R>(rng: &mut R) -> String
where
    R: Rng + ?Sized,
{
    let left = LEFT.choose(rng).unwrap();
    let right = RIGHT.choose(rng).unwrap();

    format!("{}-{}", left, right)
}
