use orderbook::{
    account::TradingAccount,
    backtest::{strategy_flow, StrategyMetrics, TestStrategy},
    management::OrderManagementSystem,
    OrderBook,
};
use pretty_assertions::assert_eq;
use rstest::rstest;

fn metrics_1() -> StrategyMetrics {
    StrategyMetrics {
        pnl_abs: -150.0,
        pnl_bps: -0.29717612,
        volume: 5047512,
        trade_count: 3,
    }
}

#[rstest]
#[case((-0.000001, 0.000001), metrics_1())]
fn snap_to_event_test(
    #[case] criterions: (f32, f32),
    #[case] expected: StrategyMetrics,
) {
    let ob_path = "data/ob.csv";
    let orders_path = "data/orders.csv";
    let mut ob = OrderBook::new();
    let mut strat = TestStrategy::new();
    let initial_balance = 0;
    strat.buy_criterion = criterions.0;
    strat.sell_criterion = criterions.1;
    strat.buy_position_limit = 100;
    strat.sell_position_limit = -100;
    strat.qty = 100;

    // Setup account
    let money_account = TradingAccount::new(initial_balance);
    // Setup OMS
    let mut oms = OrderManagementSystem::new(&mut strat, money_account);

    let metrics = strategy_flow(&mut oms, &mut ob, ob_path, orders_path);
    assert_eq!(metrics, expected);
}
