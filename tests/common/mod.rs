use orderbook::{Order, OrderBook, Side};

#[allow(dead_code)]
pub fn empty_ob() -> OrderBook {
    OrderBook::new()
}

#[allow(dead_code)]
pub fn full_ob() -> OrderBook {
    let mut ob = OrderBook::new();
    let buy_order = Order {
        id: 666,
        side: Side::Bid,
        price: 99,
        qty: 10,
        ts_create: 0
    };
    ob.add_limit_order(buy_order, 0);
    let sell_order = Order {
        id: 999,
        side: Side::Ask,
        price: 101,
        qty: 10,
        ts_create: 0
    };
    ob.add_limit_order(sell_order, 0);
    let sell_order = Order {
        id: 1000,
        side: Side::Ask,
        price: 102,
        qty: 10,
        ts_create: 0
    };
    ob.add_limit_order(sell_order, 0);
    let sell_order = Order {
        id: 1001,
        side: Side::Ask,
        price: 103,
        qty: 10,
        ts_create: 0
    };
    ob.add_limit_order(sell_order, 0);
    ob
}

#[allow(dead_code)]
pub fn taker_buy_order() -> Order {
    Order {
        id: 1,
        side: Side::Bid,
        price: 9999,
        qty: 30,
        ts_create: 0
    }
}
