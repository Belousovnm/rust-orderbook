use orderbook::{
    account::TradingAccount,
    backtest::{strategy_flow, FixSpreadStrategy, StrategyMetrics},
    management::OrderManagementSystem,
    tick::Ticker,
    OrderBook,
};
use pretty_assertions::assert_eq;
use rstest::rstest;

fn metrics_1() -> StrategyMetrics {
    StrategyMetrics {
        pnl_abs: -351.0,
        pnl_bps: -0.14447868,
        volume: 24294244.0,
        trade_count: 1068,
    }
}

#[rstest]
#[case((-0.0002, 0.0002), metrics_1())]
fn snap_to_event_test(#[case] criterions: (f32, f32), #[case] expected: StrategyMetrics) {
    let ob_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/ob/ob_ALRS.2024-01-29.csv";
    let orders_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/orders/orders_ALRS.2024-01-29.csv";
    let mut ob = OrderBook::new();
    let ticker = Ticker {
        ticker_id: 0,
        tick_size: 1.0,
        step_price: 0.1,
        taker_fee: 0.0,
        maker_fee: 0.0,
    };
    let mut strat = FixSpreadStrategy::new(ticker);
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
