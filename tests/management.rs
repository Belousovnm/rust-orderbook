mod common;
use common::{empty_ob, full_ob};
use orderbook_lib::{
    account::TradingAccount, backtest::TestStrategy, management::OrderManagementSystem, Indicator,
    Order, OrderBook, Side,
};
use pretty_assertions::assert_eq;
use rstest::rstest;

#[rstest]
#[case(empty_ob(), Err("Missing Ref Price".to_owned()))]
#[case(full_ob(), Ok(Order{id: 777, side: Side::Bid, price: 100, qty: 10}))]
fn ref_price_to_order_test(#[case] ob: OrderBook, #[case] expected: Result<Order, String>) {
    let mut strategy = TestStrategy::new();
    let account = TradingAccount::new(0);
    strategy.buy_criterion = 0.0;
    strategy.buy_position_limit = 10;
    strategy.qty = 100;
    let trader_id = 777;
    let midprice = Indicator::Midprice;
    let m = midprice.evaluate(&ob);
    let oms = OrderManagementSystem::new(&mut strategy, account);
    let trader_order = oms.calculate_buy_order(m, trader_id);

    assert_eq!(trader_order, expected);
}
