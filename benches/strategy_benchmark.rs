#![feature(test)]
extern crate test;
use orderbook::Snap;
use std::hint::black_box;

fn deserialize_bench() {
    let ob_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/ob_ALRS.2024-01-29.csv";
    let snap_reader = &mut csv::Reader::from_path(ob_path).unwrap();
    black_box(black_box(snap_reader).deserialize::<Snap>());
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_midprice(b: &mut Bencher) {
        b.iter(|| deserialize_bench());
    }
}
