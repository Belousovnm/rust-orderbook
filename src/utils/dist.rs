use crate::orderbook::Side;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

impl Distribution<Side> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Side {
        match rng.gen_range(0..=1) {
            0 => Side::Bid,
            1 => Side::Ask,
            _ => unreachable!(),
        }
    }
}
