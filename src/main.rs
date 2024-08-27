// use std::time::SystemTime;
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_precision_loss)]
mod event;
mod orderbook;
mod snap;
mod utils;
use event::LimitOrder;
use orderbook::{Order, OrderBook, Side};
use snap::Snap;

fn main() {
    dbgp!("Crafting Orderbook");
    let trader_order_id = 333;
    let mut ob = OrderBook::new();

    let snap = Snap {
        exch_epoch: 0,
        vec: vec![
            LimitOrder {
                side: Side::Ask,
                price: 99,
                qty: 100,
            },
            LimitOrder {
                side: Side::Ask,
                price: 100,
                qty: 10,
            },
            LimitOrder {
                side: Side::Ask,
                price: 101,
                qty: 10,
            },
        ],
    };
    // println!("{:?}", SystemTime::now());
    if true {
        ob = ob.process(snap, (909, trader_order_id));
    } else {
        ob = ob.process_fp(&snap);
    }
    // if matches! {fr.status, OrderStatus::Filled} {
    //     dbgp!("{:#?}, avg_fill_price {}", fr, fr.avg_fill_price());
    // }
    // println!("{:?}", SystemTime::now());
    let _ = ob.add_limit_order(Order {
        side: Side::Ask,
        price: 99,
        qty: 10,
        id: trader_order_id,
        is_synth: false,
        send_time: 0,
        fill_time: 0,
    });

    let snap = Snap {
        exch_epoch: 0,
        vec: vec![
            LimitOrder {
                side: Side::Ask,
                price: 99,
                qty: 150,
            },
            LimitOrder {
                side: Side::Ask,
                price: 100,
                qty: 10,
            },
            LimitOrder {
                side: Side::Ask,
                price: 101,
                qty: 5,
            },
        ],
    };
    ob = ob.process(snap, (909, trader_order_id));

    let exec_report = ob.add_limit_order(Order {
        side: Side::Bid,
        price: 99,
        qty: 135,
        id: 1010,
        is_synth: false,
        send_time: 0,
        fill_time: 0,
    });
    dbgp!("{:#?}", exec_report);

    dbgp!("{:#?}", ob);
    let _ = ob.get_bbo();
    dbgp!("Done!");
}
