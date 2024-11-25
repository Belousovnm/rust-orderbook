use orderbook::{
    account::TradingAccount,
    backtest::{signal_flow, FixSpreadStrategy},
    management::OrderManagementSystem,
    tick::Ticker,
    OrderBook,
};

fn main() {
    let ob_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/ob/ob_MMZ4.2024-09-25.csv";
    let orders_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/orders/orders_MMZ4.2024-09-25.csv";
    let signals_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/signals/MM_signal.2024-09-25.csv";
    let mut ob = OrderBook::default();
    let mmz4 = Ticker {
        ticker_id: 0,
        tick_size: 5.0,
        step_price: 0.1,
        taker_fee: 0.0,
        maker_fee: 0.0,
    };
    let mut strat = FixSpreadStrategy {
        buy_criterion: 0.0005,
        sell_criterion: -0.0005,
        buy_position_limit: 10,
        sell_position_limit: -10,
        qty: 1,
        ticker: mmz4,
        ..Default::default()
    };

    let money_account = TradingAccount::new(0);
    let mut oms = OrderManagementSystem::new(&mut strat, money_account);

    signal_flow(&mut oms, &mut ob, ob_path, orders_path, signals_path);
}
