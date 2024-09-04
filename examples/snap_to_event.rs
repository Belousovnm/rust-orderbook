use std::collections::HashMap;
use std::env;
use orderbook::backtest::snap_to_event;
use orderbook::backtest::{Strategy, StrategyName};
use orderbook::{account::TradingAccount, management::OrderManagementSystem, Indicator, OrderBook};

fn main() {
    let ob_path = "data/ob.csv";
    let orders_path = "data/orders.csv";
    let mut ob = OrderBook::new();
    let mut strat = Strategy::new(StrategyName::TestStrategy);
    let initial_balance = 0;

    let mut  fill_times_bid_all: HashMap<(u8,u8), HashMap<u64, u64>> = HashMap::new();
    let mut  fill_times_ask_all: HashMap<(u8,u8), HashMap<u64, u64>> = HashMap::new();

    let money_account = TradingAccount::new(initial_balance);
    let midprice = Indicator::Midprice;
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

    strat.buy_criterion = -0.0002;
    strat.sell_criterion = 0.0002;
    strat.qty = 100;
    strat.buy_position_limit = 100;
    strat.sell_position_limit = -100;

    if is_fp {
        // walk through levels from 1 to 5
        for level in 1..5 {
            // walk through qty factors from 1 to 10
            for qty_mul in 1..10 {
                strat.buy_criterion = -0.0001 * level as f32;
                strat.sell_criterion = 0.0001 * level as f32;
                strat.qty = 10 * qty_mul;

                let money_account = TradingAccount::new(initial_balance);
                let midprice = Indicator::Midprice;
                let mut oms = OrderManagementSystem::new(&mut strat, money_account);
                oms.is_fp_tracking = true;

                let metrics = snap_to_event(midprice, &mut oms, &mut ob, ob_path, orders_path);

                fill_times_bid_all.insert((level, qty_mul as u8), metrics.fill_times_bid);
                fill_times_ask_all.insert((level, qty_mul as u8), metrics.fill_times_ask);
            }
        }
    } else {
        let mut oms = OrderManagementSystem::new(&mut strat, money_account);
        snap_to_event(midprice, &mut oms, &mut ob, ob_path, orders_path);
    }

    // save fill_times to data base
}
