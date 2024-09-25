use core::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion};
use orderbook::{Order, OrderBook, Side};
use rand::Rng;

fn run_orders(
    num_orders: i32,
    rng: &mut rand::prelude::ThreadRng,
) -> OrderBook {
    let mut ob = OrderBook::new();
    for i in 1..=num_orders {
        ob.add_limit_order(Order {
            side: Side::Bid,
            price: rng.gen_range(95..102),
            qty: rng.gen_range(1..=50),
            id: 2 * i as u64,
        });

        ob.add_limit_order(Order {
            side: Side::Ask,
            price: rng.gen_range(98..105),
            qty: rng.gen_range(1..=50),
            id: (2 * i + 1) as u64,
        });
    }

    ob
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("order-benchmark");
    // let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    // group.plot_config(plot_config);
    group.bench_function("Match orders", |b| {
        b.iter(|| run_orders(1_000, &mut rng))
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
             .sample_size(10_000)
             .measurement_time(Duration::from_secs(10));
    targets = criterion_benchmark
}
criterion_main!(benches);
