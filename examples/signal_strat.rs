use orderbook::{
    account::TradingAccount,
    backtest::{signal_flow, FixSpreadStrategy},
    management::OrderManagementSystem,
    OrderBook,
};

fn main() {
    let ob_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/ob_MMZ4.2024-10-10.csv";
    let orders_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/orders_MMZ4.2024-10-10.csv";
    let signals_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/MM_signal.csv";
    let mut ob = OrderBook::default();
    let mut strat = FixSpreadStrategy {
        buy_criterion: 0.0005,
        sell_criterion: -0.0005,
        buy_position_limit: 50,
        sell_position_limit: -50,
        qty: 1,
        ..Default::default()
    };

    let money_account = TradingAccount::new(0);
    let mut oms = OrderManagementSystem::new(&mut strat, money_account);

    signal_flow(&mut oms, &mut ob, ob_path, orders_path, signals_path);
}
