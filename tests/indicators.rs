use orderbook::{indicators::Midprice, Order, OrderBook, Side};
use pretty_assertions::assert_eq;
use rstest::rstest;

fn empty_ob() -> OrderBook {
    OrderBook::new()
}

fn full_ob(bid: u32, ask: u32) -> OrderBook {
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
#[case(full_ob(42, 43), Some(42.5))]
fn midprice_test(#[case] input: OrderBook, #[case] expected: Option<f32>) {
    assert_eq!(Midprice::evaluate(&input), expected);
}
