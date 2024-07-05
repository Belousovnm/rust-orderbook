// Problem: midprice need to be reusable for buy/send
// OverLimit
use orderbook::account::TradingAccount;
use orderbook::dbgp;
use orderbook::indicators::Indicator;
use orderbook::management::OrderManagementSystem;
use orderbook::orderbook::{Order, OrderBook};
use orderbook::snap::Snap;
use orderbook::strategy::{Strategy, StrategyName};

// #[allow(unused_variables)]
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

    // Setup Strat
    let mut strat = Strategy::new(StrategyName::TestStrategy);
    strat.buy_criterion = -0.0001;
    strat.sell_criterion = 0.0001;
    strat.buy_position_limit = 0;
    strat.sell_position_limit = 0;
    strat.qty = 10;

    // Setup account
    let money_account = TradingAccount::new(initial_balance);

    // Setup OMS
    let mut oms = OrderManagementSystem::new(strat, money_account);

    // Setup Indicator
    let midprice = Indicator::Midprice;

    // Load first snapshot
    if let Some(Ok(first_snap)) = srdr.next() {
        epoch = first_snap.exch_epoch;
        ob = ob.process(first_snap, (trader_buy_id, trader_sell_id));
    }

    // Skip all trades that occured before the first snapshot
    while next_order.id < epoch {
        if let Some(Ok(order)) = trdr.next() {
            next_order = order;
        }
    }

    'a: while let Some(Ok(snap)) = srdr.next() {
        let epoch = snap.exch_epoch;
        loop {
            // If order before next update
            if next_order.id <= epoch {
                // Apply order
                let exec_report = ob.add_limit_order(next_order);
                dbgp!("{:#?}", exec_report);
                oms.update(exec_report, (trader_buy_id, trader_sell_id));
                dbgp!("POS {:#?}", oms.strategy.master_position);
                dbgp!("ACC {:#?}", oms.account.balance);

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
                ob = ob.process(snap, (trader_buy_id, trader_sell_id));
                // Trader's move
                let m = midprice.evaluate(&ob);
                dbgp!("POS {:#?}", oms.strategy.master_position);
                if let Ok(buy_order) = oms.calculate_buy_order(m, trader_buy_id) {
                    match ob.order_loc.get(&trader_buy_id) {
                        None => {
                            dbgp!("[ STRAT] Order not found, place new order");
                            dbgp!("[ STRAT] sent {:#?}", buy_order);
                            ob.add_limit_order(buy_order);
                        }
                        Some((_, _, price)) if *price == buy_order.price => {
                            dbgp!("[ STRAT] Order found, passing");
                            dbgp!("[ STRAT] price = {}", *price);
                        }
                        Some((_, _, price)) => {
                            dbgp!("[ STRAT] Order found, need price update, place new order");
                            dbgp!(
                                "[ STRAT] Old price {}, New Price {}",
                                *price,
                                buy_order.price
                            );
                            dbgp!("[ STRAT] sent {:#?}", buy_order);
                            ob.add_limit_order(buy_order);
                        }
                    }
                }

                let m = midprice.evaluate(&ob);
                dbgp!("POS {:#?}", oms.strategy.master_position);
                if let Ok(sell_order) = oms.calculate_sell_order(m, trader_sell_id) {
                    match ob.order_loc.get(&trader_sell_id) {
                        None => {
                            dbgp!("[ STRAT] Order not found, place new order");
                            dbgp!("[ STRAT] sent {:#?}", sell_order);
                            ob.add_limit_order(sell_order);
                        }
                        Some((_, _, price)) if *price == sell_order.price => {
                            dbgp!("[ STRAT] Order found, passing");
                        }
                        Some((_, _, price)) => {
                            dbgp!("[ STRAT] Order found, need price update, place new order");
                            dbgp!(
                                "[ STRAT] Old price {}, New Price {}",
                                *price,
                                sell_order.price
                            );
                            dbgp!("[ STRAT] sent {:#?}", sell_order);
                            ob.add_limit_order(sell_order);
                        }
                    }
                }
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
    dbgp!("{}", pnl);
    dbgp!("Done!");
}

fn main() {
    snap_to_event()
}
