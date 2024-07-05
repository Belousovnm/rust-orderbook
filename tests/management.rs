mod common;
use common::{empty_ob, full_ob};
use orderbook::account::TradingAccount;
use orderbook::indicators::Indicator;
use orderbook::management::OrderManagementSystem;
use orderbook::orderbook::{Order, OrderBook, Side};
use orderbook::strategy::{Strategy, StrategyName};
use pretty_assertions::assert_eq;
use rstest::rstest;

#[rstest]
#[case(empty_ob(), Err("Both bid and offer HalfBooks are empty"))]
#[case(full_ob(), Ok(Order{id: 777, side: Side::Bid, price: 100, qty: 10}))]
fn ref_price_to_order_test(#[case] ob: OrderBook, #[case] expected: Result<Order, &str>) {
    let mut strategy = Strategy::new(StrategyName::TestStrategy);
    let account = TradingAccount::new(0);
    strategy.buy_criterion = 0.0;
    strategy.buy_position_limit = 10;
    strategy.qty = 100;
    let trader_id = 777;
    let midprice = Indicator::Midprice;
    let m = midprice.evaluate(&ob);
    let oms = OrderManagementSystem::new(strategy, account);
    let trader_order = oms.calculate_buy_order(m, trader_id);

    assert_eq!(trader_order, expected);
}
