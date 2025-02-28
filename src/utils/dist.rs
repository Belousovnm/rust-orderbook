use crate::engine::Side;
use rand::{distr::StandardUniform, prelude::Distribution, Rng};

impl Distribution<Side> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Side {
        match rng.random_range(0..=1) {
            | 0 => Side::Bid,
            | 1 => Side::Ask,
            | _ => unreachable!(),
        }
    }
}
