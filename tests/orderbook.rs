mod common;
use common::{empty_ob, full_ob, taker_buy_order};
use orderbook::engine::{ExecutionReport, Order, OrderBook, OrderStatus, Side};
use pretty_assertions::assert_eq;
use rstest::rstest;

#[rstest]
#[case(empty_ob(), None, Err("Offer HalfBook is empty"))]
#[case(full_ob(), Some(101.5), Ok((99, 103, 4)))]
fn avg_fill_price_test(
    #[case] mut ob: OrderBook,
    #[case] expected: Option<f32>,
    #[case] bbo: Result<(u32, u32, u32), &str>,
) {
    let exec_report = ob.add_limit_order(taker_buy_order());
    let filled_price = exec_report.avg_fill_price();
    assert_eq!(filled_price, expected);
    assert_eq!(ob.get_bbo(), bbo);
}

#[rstest]
#[case(empty_ob(), None)]
#[case(full_ob(), Some(&Order {id: 666, side: Side::Bid, price: 99, qty: 10 }))]
fn get_order_test(#[case] ob: OrderBook, #[case] expected: Option<&Order>) {
    let order = ob.get_order(666);
    assert_eq!(order, expected)
}

#[rstest]
#[case(full_ob(), Order {id:222, side: Side::Bid, price: 100, qty: 20}, ExecutionReport {filled_orders: vec!(), own_id: 222, own_side: Side::Bid, remaining_qty: 20, status: OrderStatus::Created})]
#[case(full_ob(), Order {id:222, side: Side::Bid, price: 101, qty: 10}, ExecutionReport {filled_orders: vec![(999,10,101)], own_id: 222, own_side: Side::Bid, remaining_qty: 0, status: OrderStatus::Filled})]
#[case(full_ob(), Order {id:222, side: Side::Bid, price: 101, qty: 15}, ExecutionReport {filled_orders: vec![(999,10,101)], own_id: 222, own_side: Side::Bid, remaining_qty: 5, status: OrderStatus::PartiallyFilled})]
fn add_limit_order_test(
    #[case] mut ob: OrderBook,
    #[case] order: Order,
    #[case] expected: ExecutionReport,
) {
    assert_eq!(ob.add_limit_order(order), expected);
}
