use core::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion};
use csv::Reader;
use orderbook::Snap;
use std::fs::File;

fn deserialize(snap_reader: &mut Reader<File>) {
    let _srdr = snap_reader.deserialize::<Snap>();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let ob_path = "data/ob.csv";
    let snap_reader = &mut csv::Reader::from_path(ob_path).unwrap();
    let mut group = c.benchmark_group("order-benchmark");
    group.sample_size(10);
    group.measurement_time(Duration::new(5, 0));
    group.bench_function("Match orders", |b| b.iter(|| deserialize(snap_reader)));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
