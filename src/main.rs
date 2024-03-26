use orderbook::{dbgp, OrderBook, OrderStatus};
use rand::Rng;

fn main() {
    println!("Crafting new Orderbook");
    let mut ob = OrderBook::new("SPB".to_string());
    let mut rng = rand::thread_rng();
    let mut order_id = 0;
    let trader_order_id = 4;
    for _ in 1..=10 {
        order_id += 1;
        let fr = ob.add_limit_order(
            rng.gen(),
            rng.gen_range(95..=105),
            rng.gen_range(1..=100),
            Some(order_id),
        );
        if matches! {fr.status, OrderStatus::Filled} {
            dbgp!("{:#?}, avg_fill_price {}", fr, fr.avg_fill_price());
        }
        let _ = ob.get_bbo();
    }

    let to_keep = ob.get_offset(trader_order_id);
    dbgp!("{:#?}", to_keep);

    dbgp!("{:#?}", ob);
    dbgp!("{:?}", ob.get_bbo());
    println!("Done!");
}
