use orderbook::{Order, OrderBook, Side};
use rand::Rng;
use std::{thread, time};
fn main() {
    let num_orders = 10_000;
    let mut rng = rand::rng();
    let mut ob = OrderBook::new();
    for i in 1..=num_orders {
        ob.add_limit_order(Order {
            side: Side::Bid,
            price: rng.random_range(95..105),
            qty: rng.random_range(1..=50),
            id: 2 * i as u64,
        });

        ob.add_limit_order(Order {
            side: Side::Ask,
            price: rng.random_range(95..105),
            qty: rng.random_range(1..=50),
            id: (2 * i + 1) as u64,
        });
        println!("{}", ob);
        thread::sleep(time::Duration::from_millis(16));
        print!("{esc}c", esc = 27 as char);
    }
}
