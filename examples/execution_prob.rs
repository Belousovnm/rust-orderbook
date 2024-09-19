use orderbook::{
    account::TradingAccount, backtest::FixPriceStrategy, experiments::execution_flow,
    management::OrderManagementSystem, OrderBook,
};

fn main() {
    let ob_path = "data/ob.csv";
    let orders_path = "data/orders.csv";
    let mut ob = OrderBook::default();
    let mut strat = FixPriceStrategy {
        buy_tick_criterion: Some(100),
        sell_tick_criterion: None,
        qty: 1,
        ..Default::default()
    };

    let money_account = TradingAccount::new(0);
    let mut oms = OrderManagementSystem::new(&mut strat, money_account);

    execution_flow(&mut oms, &mut ob, ob_path, orders_path);
}
