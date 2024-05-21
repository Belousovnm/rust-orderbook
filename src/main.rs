// use std::time::SystemTime;
mod event;
mod orderbook;
mod snap;
mod utils;
use event::LimitOrder;
use orderbook::{Order, OrderBook, Side};
use snap::Snap;

fn main() {
    dbgp!("Crafting Orderbook");
    let mut ob = OrderBook::new("SPB".to_string());
    let snap = Snap {
        exch_epoch: 0,
        vec: vec![
            LimitOrder {
                side: Side::Bid,
                price: 99,
                qty: 10,
            },
            LimitOrder {
                side: Side::Bid,
                price: 100,
                qty: 10,
            },
            LimitOrder {
                side: Side::Bid,
                price: 101,
                qty: 10,
            },
        ],
    };
    // println!("{:?}", SystemTime::now());
    ob.process(snap, Err("mock"));
    // if matches! {fr.status, OrderStatus::Filled} {
    //     dbgp!("{:#?}, avg_fill_price {}", fr, fr.avg_fill_price());
    // }
    // println!("{:?}", SystemTime::now());
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
    let snap = Snap {
        exch_epoch: 0,
        vec: vec![
            LimitOrder {
                side: Side::Bid,
                price: 99,
                qty: 10,
            },
            LimitOrder {
                side: Side::Bid,
                price: 100,
                qty: 10,
            },
            LimitOrder {
                side: Side::Bid,
                price: 101,
                qty: 5,
            },
        ],
    };
    ob.process(snap, offset);

    dbgp!("{:#?}", ob);
    let _ = ob.get_bbo();
    dbgp!("Done!");
}
