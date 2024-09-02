use core::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion};
use orderbook::{Order, OrderBook, Side};
use rand::Rng;

fn run_orders(num_orders: i32, rng: &mut rand::prelude::ThreadRng) -> OrderBook {
    let mut ob = OrderBook::new();
    let mut order_id = 0;
    for _ in 0..num_orders {
        order_id += 1;
        ob.add_limit_order(Order {
            side: Side::Bid,
            price: rng.gen_range(90..102),
            qty: rng.gen_range(1..=50),
            id: order_id,
        });

        ob.add_limit_order(Order {
            side: Side::Ask,
            price: rng.gen_range(98..110),
            qty: rng.gen_range(1..=50),
            id: order_id,
        });
    }

    ob
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("order-benchmark");
    group.sample_size(10);
    group.measurement_time(Duration::new(10, 0));
    group.bench_function("Match orders", |b| {
        b.iter(|| run_orders(1_000_000, &mut rng))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
