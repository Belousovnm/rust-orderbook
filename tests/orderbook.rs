mod common;
use common::{empty_ob, full_ob, taker_buy_order};
use orderbook::{Order, OrderBook, Side};
use pretty_assertions::assert_eq;
use rstest::rstest;

#[rstest]
#[case(empty_ob(), None)]
#[case(full_ob(), Some(102.0))]
fn avg_fill_price_test(#[case] mut ob: OrderBook, #[case] expected: Option<f32>) {
    let exec_report = ob.add_limit_order(taker_buy_order(), 0);
    let filled_price = exec_report.avg_fill_price();
    assert_eq!(filled_price, expected)
}

#[rstest]
#[case(empty_ob(), None)]
#[case(full_ob(), Some(&Order {id: 666, side: Side::Bid, price: 99, qty: 10, ts_create: 0 }))]
fn get_order_test(#[case] ob: OrderBook, #[case] expected: Option<&Order>) {
    let order = ob.get_order(666);
    assert_eq!(order, expected)
}
