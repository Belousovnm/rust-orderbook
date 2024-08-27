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
            qty: rng.gen_range(10..=50),
            id: order_id,
            is_synth: false,
            send_time: 0,
            fill_time: 0
        });

        ob.add_limit_order(Order {
            side: Side::Ask,
            price: rng.gen_range(98..110),
            qty: rng.gen_range(1..=500),
            id: order_id,
            is_synth: false,
            send_time: 0,
            fill_time: 0
        });
        if order_id > 100 {
            let _ = ob.cancel_order(order_id - 100);
        };
    }

    ob
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("order-benchmark");
    group.sample_size(10);
    group.measurement_time(Duration::new(5, 0));
    group.bench_function("Match orders", |b| b.iter(|| run_orders(100_000, &mut rng)));
    /*
    group.bench_function("match 10000 orders on orderbook with 100k orders", |b| {
        b.iter(|| match_orders(&mut ob, &mut rng, normal))
    });
    */
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
