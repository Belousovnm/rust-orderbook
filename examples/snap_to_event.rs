use orderbook::{
    account::TradingAccount, management::OrderManagementSystem, snap_to_event, Indicator,
    OrderBook, Strategy, StrategyName,
};

fn main() {
    let ob_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/ob_SBER.2023-11-16.csv";
    let orders_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/ob_SBER.2023-11-16.csv";
    let mut ob = OrderBook::new();
    let mut strat = Strategy::new(StrategyName::TestStrategy);
    let initial_balance = 0;
    strat.buy_criterion = -0.0001;
    strat.sell_criterion = 0.0001;
    strat.buy_position_limit = 100;
    strat.sell_position_limit = -100;
    strat.qty = 100;

    let money_account = TradingAccount::new(initial_balance);
    let midprice = Indicator::Midprice;
    let mut oms = OrderManagementSystem::new(&mut strat, money_account);

    snap_to_event(midprice, &mut oms, &mut ob, ob_path, orders_path);
}
