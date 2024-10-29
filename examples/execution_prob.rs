use orderbook::{
    account::TradingAccount, backtest::FixPriceStrategy, experiments::execution_flow,
    management::OrderManagementSystem, OrderBook,
};

fn main() {
    let ob_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/ob_SBER.2023-11-10.csv";
    let orders_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/orders_SBER.2023-11-10.csv";
    let mut ob = OrderBook::default();
    let mut strat = FixPriceStrategy {
        buy_tick_criterion: Some(-2),
        sell_tick_criterion: None,
        qty: 1,
        ..Default::default()
    };

    let money_account = TradingAccount::new(0);
    let mut oms = OrderManagementSystem::new(&mut strat, money_account);

    execution_flow(&mut oms, &mut ob, ob_path, orders_path);
}
