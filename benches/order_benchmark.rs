use core::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion};
use orderbook::engine::{Order, OrderBook, Side};
use rand::Rng;

fn run_orders(num_orders: u32, rng: &mut rand::prelude::ThreadRng) -> OrderBook {
    let mut ob = OrderBook::new();
    (1..num_orders).for_each(|id| {
        ob.add_limit_order(Order {
            side: Side::Bid,
            price: rng.random_range(95..105),
            qty: rng.random_range(1..=50),
            id: 2 * id as u64,
        });

        ob.add_limit_order(Order {
            side: Side::Ask,
            price: rng.random_range(95..105),
            qty: rng.random_range(1..=50),
            id: (2 * id + 1) as u64,
        });
    });

    ob
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::rng();
    let mut group = c.benchmark_group("order-benchmark");
    // let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    // group.plot_config(plot_config);
    group.bench_function("Match orders", |b| {
        b.iter(|| run_orders(1_000_000, &mut rng))
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
             .sample_size(10)
             .measurement_time(Duration::from_secs(10));
    targets = criterion_benchmark
}
criterion_main!(benches);
