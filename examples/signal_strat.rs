use orderbook::{
    backtest::{signal_flow, SignalStrategy},
    engine::OrderBook,
    engine::Ticker,
    engine::TradingAccount,
    management::OrderManagementSystem,
};

fn main() {
    let ob_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/ob/ob_MMH5.2025-02-13.csv";
    let orders_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/orders/orders_MMH5.2025-02-13.csv";
    let signals_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/signals/CVD_signal.2025-02-13.csv";
    let mut ob = OrderBook::default();
    let mm = Ticker {
        ticker_id: 0,
        tick_size: 5.0,
        step_price: 0.5,
        taker_fee: 0.000066,
        maker_fee: 0.0,
    };
    let mut strat = SignalStrategy {
        buy_open_criterion: 0.0001,
        sell_open_criterion: -0.0001,
        buy_close_criterion: -0.0001,
        sell_close_criterion: 0.0001,
        buy_position_limit: 50,
        sell_position_limit: -50,
        qty: 10,
        ticker: mm,
        maker_range: (-f32::INFINITY, f32::INFINITY),
        taker_range: (-f32::INFINITY, f32::INFINITY),
        ..Default::default()
    };

    let money_account = TradingAccount::new(0.0);
    let mut oms = OrderManagementSystem::new(&mut strat, money_account);

    let _ = signal_flow(&mut oms, &mut ob, ob_path, orders_path, signals_path);
}
