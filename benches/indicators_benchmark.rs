// only in nightly
#![feature(test)]

use orderbook::{midprice, utils::better_black_box};
use std::hint::black_box;
extern crate test;

fn midprice_bench(bid: u32, ask: u32) -> f32 {
    black_box(midprice(black_box(bid), black_box(ask)) as u64) as f32
}

fn better_midprice_bench(bid: u32, ask: u32) -> f32 {
    better_black_box(midprice(better_black_box(bid), better_black_box(ask)) as u32) as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_midprice(b: &mut Bencher) {
        b.iter(|| midprice_bench(100, 101));
    }

    #[bench]
    fn better_bench_midprice(b: &mut Bencher) {
        b.iter(|| better_midprice_bench(100, 101));
    }
}
