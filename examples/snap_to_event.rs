use orderbook::{
    account::TradingAccount, management::OrderManagementSystem, snap_to_event, Indicator,
    OrderBook, Strategy, StrategyName,
};

fn main() {
    let ob_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/ob_ALRS.2024-01-29.csv";
    let orders_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/orders_ALRS.2024-01-29.csv";
    let mut ob = OrderBook::new("SecName".to_string());
    let mut strat = Strategy::new(StrategyName::TestStrategy);
    let initial_balance = 0;
    strat.buy_criterion = -0.0002;
    strat.sell_criterion = 0.0002;
    strat.buy_position_limit = 100;
    strat.sell_position_limit = -100;
    strat.qty = 100;

    let money_account = TradingAccount::new(initial_balance);
    let midprice = Indicator::Midprice;
    let mut oms = OrderManagementSystem::new(&mut strat, money_account);

    snap_to_event(midprice, &mut oms, &mut ob, ob_path, orders_path);
}
