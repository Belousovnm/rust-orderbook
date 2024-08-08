use orderbook::{Indicator, Order, OrderBook, Side};
use pretty_assertions::assert_eq;
use rstest::rstest;

fn empty_ob() -> OrderBook {
    OrderBook::new()
}

fn full_ob(bid: u64, ask: u64) -> OrderBook {
    let mut ob = OrderBook::new();
    let buy_order = Order {
        id: 666,
        side: Side::Bid,
        price: bid,
        qty: 10,
    };
    ob.add_limit_order(buy_order);
    let sell_order = Order {
        id: 999,
        side: Side::Ask,
        price: ask,
        qty: 10,
    };
    ob.add_limit_order(sell_order);
    ob
}

#[rstest]
#[case(empty_ob(), None)]
#[case(full_ob(99, 101), Some(100.0))]
fn midprice_test(#[case] input: OrderBook, #[case] expected: Option<f32>) {
    let midprice = Indicator::Midprice;
    assert_eq!(midprice.evaluate(&input), expected);
}
