use std::env;
use orderbook::{
    account::TradingAccount, management::OrderManagementSystem, snap_to_event, snap_to_event_fp, Indicator,
    OrderBook, Strategy, StrategyName,
};

fn main() {
    let ob_path = "data/ob.csv";
    let orders_path = "data/orders.csv";
    let mut ob = OrderBook::new();
    let mut strat = Strategy::new(StrategyName::TestStrategy);
    let initial_balance = 0;
    strat.buy_criterion = -0.0002;
    strat.sell_criterion = 0.0002;
    strat.buy_position_limit = 100;
    strat.sell_position_limit = -100;
    strat.qty = 100;

    let args: Vec<String> = env::args().collect();

    let mut is_fp = false;
    if args.len() > 1 {
        // Access the second argument
        let second_arg = &args[1];

        // Compare the second argument with "fp"
        if second_arg == "fp" {
            is_fp = true;
        }
    }

    let money_account = TradingAccount::new(initial_balance);
    let midprice = Indicator::Midprice;
    let mut oms = OrderManagementSystem::new(&mut strat, money_account);


    if is_fp {
        snap_to_event_fp(midprice, &mut oms, &mut ob, ob_path, orders_path);
    } else {
        snap_to_event(midprice, &mut oms, &mut ob, ob_path, orders_path);
    }
}
