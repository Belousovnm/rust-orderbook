use orderbook::{dbgp, Order, OrderBook, Side};
// use rand::Rng;
use orderbook::next_snap;

fn main() {
    println!("Crafting new Orderbook");
    let mut ob = OrderBook::new("SPB".to_string());
    // let mut rng = rand::thread_rng();
    let snap = vec![
        (Side::Bid, 99, 10),
        (Side::Bid, 100, 10),
        (Side::Bid, 101, 10),
    ];
    next_snap(snap.clone(), &mut ob, Err("mock"));
    // if matches! {fr.status, OrderStatus::Filled} {
    //     dbgp!("{:#?}, avg_fill_price {}", fr, fr.avg_fill_price());
    // }
    let trader_order_id = 777;
    let _ = ob.add_limit_order(Order {
        side: Side::Bid,
        price: 101,
        qty: 1,
        id: trader_order_id,
    });
    let _ = ob.add_limit_order(Order {
        side: Side::Bid,
        price: 101,
        qty: 10,
        id: 999,
    });
    let offset = ob.get_offset(trader_order_id);

    let mut ob = OrderBook::new("SPB".to_string());
    let snap = vec![
        (Side::Bid, 99, 10),
        (Side::Bid, 100, 10),
        (Side::Bid, 101, 20),
    ];
    next_snap(snap, &mut ob, offset);

    dbgp!("{:#?}", ob);
    dbgp!("{:?}", ob.get_bbo());
    println!("Done!");
}
