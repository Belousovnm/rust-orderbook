use orderbook::{
    backtest::{signal_flow, SignalStrategy},
    engine::OrderBook,
    engine::TradingAccount,
    management::OrderManagementSystem,
};

fn main() {
    let ob_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/ob/ob_MMM5.2025-03-28.csv";
    let orders_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/orders/orders_MMM5.2025-03-28.csv";
    let signals_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/signals/CVD_signal.2025-03-28.csv";
    let mut ob = OrderBook::default();
    let mut strat = SignalStrategy {
        buy_open_criterion: 0.0001,
        sell_open_criterion: -0.0001,
        buy_close_criterion: -0.0001,
        sell_close_criterion: 0.0001,
        buy_position_limit: 50,
        sell_position_limit: -50,
        qty: 10,
        ticker: orderbook::utils::tick::MM,
        maker_range: (-f32::INFINITY, f32::INFINITY),
        taker_range: (-f32::INFINITY, f32::INFINITY),
        ..Default::default()
    };

    let money_account = TradingAccount::new(0.0);
    let mut oms = OrderManagementSystem::new(&mut strat, money_account);

    let _ = signal_flow(&mut oms, &mut ob, ob_path, orders_path, signals_path);
}
