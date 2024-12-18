mod common;
use common::{empty_ob, full_ob};
use orderbook::{
    account::TradingAccount, backtest::FixSpreadStrategy, management::OrderManagementSystem,
    tick::Ticker, Midprice, Order, OrderBook, Side,
};
use pretty_assertions::assert_eq;
use rstest::rstest;

#[rstest]
#[case(empty_ob(), Side::Bid, Err("Missing Ref Price".to_owned()))]
#[case(full_ob(), Side::Bid, Ok(Order{id: 3, side: Side::Bid, price: 99, qty: 10}))]
#[case(full_ob(), Side::Ask, Ok(Order{id: 7, side: Side::Ask, price: 101, qty: 10}))]
fn calculate_order_test(
    #[case] ob: OrderBook,
    #[case] side: Side,
    #[case] expected: Result<Order, String>,
) {
    let mut strat = FixSpreadStrategy::new(Ticker::default());
    let account = TradingAccount::new(0.0);
    strat.buy_criterion = -0.01;
    strat.sell_criterion = 0.01;
    strat.buy_position_limit = 10;
    strat.sell_position_limit = -10;
    strat.qty = 100;
    let oms = OrderManagementSystem::new(&mut strat, account);
    let m = Midprice::evaluate(&ob);
    match side {
        | Side::Bid => {
            let trader_order = oms.calculate_buy_order(m, Some(3));
            assert_eq!(trader_order, expected);
        }
        | Side::Ask => {
            let trader_order = oms.calculate_sell_order(m, Some(7));
            assert_eq!(trader_order, expected);
        }
    }
}

#[rstest]
#[case(full_ob(), Side::Bid, 1, 99, Some(Order {
        id: 666,
        side: Side::Bid,
        price: 99,
        qty: 9,
    }
))]
#[case(full_ob(), Side::Ask, 1, 1010, None)]
fn update_test(
    #[case] mut ob: OrderBook,
    #[case] side: Side,
    #[case] expected_count: u32,
    #[case] expected_volume: u32,
    #[case] expected_order: Option<Order>,
) {
    let mut strat = FixSpreadStrategy::new(Ticker::default());
    let account = TradingAccount::new(0.0);
    strat.buy_criterion = -0.01;
    strat.sell_criterion = 0.01;
    strat.buy_position_limit = 10;
    strat.sell_position_limit = -10;
    strat.qty = 100;
    let mut oms = OrderManagementSystem::new(&mut strat, account);
    match side {
        | Side::Bid => {
            oms.active_buy_order = Some(Order {
                id: 666,
                side: Side::Bid,
                price: 99,
                qty: 10,
            });

            let next_order = Order {
                id: 0,
                side: Side::Ask,
                price: 99,
                qty: 1,
            };
            let exec_report = ob.add_limit_order(next_order);
            oms.update(&exec_report);
            assert_eq!(expected_count, oms.account.trade_count);
            assert_eq!(expected_volume, oms.account.cumulative_volume);
            assert_eq!(expected_order, oms.active_buy_order);
        }
        | Side::Ask => {
            oms.active_sell_order = Some(Order {
                id: 999,
                side: Side::Ask,
                price: 101,
                qty: 10,
            });

            let next_order = Order {
                id: 0,
                side: Side::Bid,
                price: 101,
                qty: 10,
            };
            let exec_report = ob.add_limit_order(next_order);
            oms.update(&exec_report);
            assert_eq!(expected_count, oms.account.trade_count);
            assert_eq!(expected_volume, oms.account.cumulative_volume);
            assert_eq!(expected_order, oms.active_buy_order);
        }
    }
}

#[rstest]
#[case(full_ob(), Some(100.0), -0.01, 0.01, 0)]
#[case(full_ob(), None, 0.1, 0.2, 0)]
#[case(full_ob(), Some(100.0), 0.01, 0.02, 1)]
#[case(full_ob(), Some(100.0), -0.01, -0.02, -1)]
fn send_orders_test(
    #[case] mut ob: OrderBook,
    #[case] m: Option<f32>,
    #[case] buy_criterion: f32,
    #[case] sell_criterion: f32,
    #[case] exp_position: i32,
) {
    let mut strat = FixSpreadStrategy::new(Ticker::default());
    strat.buy_criterion = buy_criterion;
    strat.sell_criterion = sell_criterion;
    strat.buy_position_limit = 10;
    strat.sell_position_limit = -10;
    strat.qty = 1;
    let buy_id = Some(333);
    let sell_id = Some(777);
    let account = TradingAccount::new(0.0);
    let mut oms = OrderManagementSystem::new(&mut strat, account);
    oms.send_orders(&mut ob, m, buy_id, sell_id);
    assert_eq!(exp_position, oms.strategy.master_position);
}
