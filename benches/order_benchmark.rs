use core::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion};
use orderbook::{OrderBook, Side};
use rand::Rng;

fn run_orders(num_orders: i32, rng: &mut rand::prelude::ThreadRng) -> OrderBook {
    let mut ob = OrderBook::new("SPB".to_string());
    let mut order_id = 0;
    for _ in 0..num_orders {
        order_id += 1;
        ob.add_limit_order(
            Side::Bid,
            rng.gen_range(90..102),
            rng.gen_range(10..=50),
            Some(order_id),
        );
        ob.add_limit_order(
            Side::Ask,
            rng.gen_range(98..110),
            rng.gen_range(1..=500),
            Some(order_id),
        );
    }
    ob
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
    let mut group = c.benchmark_group("order-benchmark");
    group.sample_size(10);
    group.measurement_time(Duration::new(20, 0));
    group.bench_function("Match orders", |b| {
        b.iter(|| run_orders(1_000_000, &mut rng))
    });
    /*
    group.bench_function("match 10000 orders on orderbook with 100k orders", |b| {
        b.iter(|| match_orders(&mut ob, &mut rng, normal))
    });
    */
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
