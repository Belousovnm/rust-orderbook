use orderbook::account::TradingAccount;
use orderbook::dbgp;
use orderbook::management::OrderManagementSystem;
use orderbook::strategy::{Strategy, StrategyName};
use orderbook::Indicator;
use orderbook::{Order, OrderBook, Snap};
use readable::num::*;

fn snap_to_event() {
    let ob_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/ob.csv";
    let orders_path = "/opt/Zenpy/jupyter/data/voskhod/RUST_OB/orders.csv";
    dbgp!("Crafting Orderbook");
    let mut ob = OrderBook::new("SecName".to_string());
    let mut snap_reader = csv::Reader::from_path(ob_path).unwrap();
    let mut trade_reader = csv::Reader::from_path(orders_path).unwrap();
    let mut srdr = snap_reader.deserialize::<Snap>();
    let mut trdr = trade_reader.deserialize::<Order>();
    let mut epoch = 0;
    let mut next_order = Order::default();
    let trader_buy_id = 333;
    let trader_sell_id = 777;
    let initial_balance = 0;
    let mut trading_volume = 0;
    let mut trade_count = 0;

    // Setup Strat
    let mut strat = Strategy::new(StrategyName::TestStrategy);
    strat.buy_criterion = -0.0002;
    strat.sell_criterion = 0.0002;
    strat.buy_position_limit = 100;
    strat.sell_position_limit = -100;
    strat.qty = 100;

    // Setup account
    let money_account = TradingAccount::new(initial_balance);

    // Setup OMS
    let mut oms = OrderManagementSystem::new(strat, money_account);

    // Setup Indicator
    let midprice = Indicator::Midprice;

    // Load first snapshot
    if let Some(Ok(first_snap)) = srdr.next() {
        epoch = first_snap.exch_epoch;
        dbgp!("[ EPCH ] snap {:?}", epoch);
        ob = ob.process(first_snap, (trader_buy_id, trader_sell_id));
    }

    // Skip all trades that occured before the first snapshot
    while next_order.id < epoch {
        if let Some(Ok(order)) = trdr.next() {
            next_order = order;
        }
    }

    'a: while let Some(Ok(snap)) = srdr.next() {
        epoch = snap.exch_epoch;
        loop {
            if next_order.id <= epoch {
                // Apply order
                dbgp!("[ EPCH ] order {:?}", next_order.id);
                let exec_report = ob.add_limit_order(next_order);
                dbgp!("{:#?}", exec_report);
                let prev_account_balance = oms.account.balance;
                oms.update(exec_report, (trader_buy_id, trader_sell_id));
                // dbgp!("POS {:#?}", oms.strategy.master_position);
                // dbgp!("ACC {:#?}", oms.account.balance);
                if prev_account_balance != oms.account.balance {
                    trading_volume += (oms.account.balance - prev_account_balance).abs();
                    trade_count += 1;
                }
                // Load next order
                if let Some(Ok(order)) = trdr.next() {
                    next_order = order;
                } else {
                    // Replay until last order
                    break 'a;
                }
            // If next snap before order
            } else if next_order.id > epoch {
                // Load next snap
                dbgp!("[ EPCH ] snap {:?}", epoch);
                ob = ob.process(snap, (trader_buy_id, trader_sell_id));
                // Trader's move
                let m = midprice.evaluate(&ob);
                oms.send_orders(&mut ob, m);
                dbgp!("{:?}", ob.get_order(trader_buy_id));
                dbgp!("{:?}", ob.get_order(trader_sell_id));
                break;
            }
        }
    }
    dbgp!("{:#?}", ob);
    let _ = ob.get_bbo();
    let pnl = midprice.evaluate(&ob).unwrap() * oms.strategy.master_position as f32
        + oms.account.balance as f32;
    println!("PnL abs = {}", Int::from(pnl as i32));
    println!("PnL bps = {:.3}", pnl / trading_volume as f32 * 10000.0);
    println!("Volume = {}", Int::from(trading_volume));
    println!("Trade Count = {}", Int::from(trade_count));
    dbgp!("Done!");
}

fn main() {
    snap_to_event()
}
