use orderbook::{dbgp, OrderBook, OrderStatus};
use rand::Rng;

fn main() {
    println!("Crafting new Orderbook");
    let mut ob = OrderBook::new("SPB".to_string());
    let mut rng = rand::thread_rng();
    let mut order_id = 0;

    for _ in 1..=10 {
        order_id += 1;
        let fr = ob.add_limit_order(
            rng.gen(),
            rng.gen_range(75..125),
            rng.gen_range(1..=100),
            Some(order_id),
        );
        if matches! {fr.status, OrderStatus::Filled} {
            dbgp!("{:#?}, avg_fill_price {}", fr, fr.avg_fill_price());
        }
        let _ = ob.get_bbo();
    }
    dbgp!("{:#?}", ob);
    dbgp!("{:?}", ob.get_bbo());
    println!("Done!");
}
