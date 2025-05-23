use orderbook::{
    backtest::{strategy_flow, FixSpreadStrategy},
    engine::{OrderBook, TradingAccount},
    management::OrderManagementSystem,
};

fn main() {
    let ob_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/ob/ob_ALRS.2024-01-29.csv";
    let orders_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/orders/orders_ALRS.2024-01-29.csv";
    let mut ob = OrderBook::default();
    let mut strat = FixSpreadStrategy::new(orderbook::utils::tick::ALRS);
    let initial_balance = 0.0;
    strat.buy_criterion = -0.0002;
    strat.sell_criterion = 0.0002;
    strat.buy_position_limit = 100;
    strat.sell_position_limit = -100;
    strat.qty = 100;
    strat.maker_range = (-f32::INFINITY, f32::INFINITY);

    let money_account = TradingAccount::new(initial_balance);
    let mut oms = OrderManagementSystem::new(&mut strat, money_account);

    strategy_flow(&mut oms, &mut ob, ob_path, orders_path);
}
