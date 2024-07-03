use orderbook::dbgp;
use orderbook::indicators::Indicator;
use orderbook::management::OrderManagementSystem;
use orderbook::orderbook::{Order, OrderBook};
use orderbook::snap::Snap;
use orderbook::strategy::{Strategy, StrategyName};

// #[allow(unused_variables)]
fn snap_to_event() {
    dbgp!("Crafting Orderbook");
    let mut ob = OrderBook::new("SecName".to_string());
    let mut snap_reader = csv::Reader::from_path("data/ob.csv").unwrap();
    let mut trade_reader = csv::Reader::from_path("data/orders.csv").unwrap();
    let mut srdr = snap_reader.deserialize::<Snap>();
    let mut trdr = trade_reader.deserialize::<Order>();
    let mut epoch = 0;
    let mut next_order = Order::default();
    let trader_id = 777;

    // Setup Strat
    let mut strat = Strategy::new(StrategyName::TestStrategy);
    strat.buy_criterion = 0.0;

    // Setup OMS
    let oms = OrderManagementSystem {
        strategy: strat,
        active_orders: Vec::with_capacity(2),
        strategy_signals: Vec::with_capacity(2),
    };

    // Setup Indicator
    let midprice = Indicator::Midprice;

    // Load first snapshot
    if let Some(Ok(first_snap)) = srdr.next() {
        epoch = first_snap.exch_epoch;
        ob = ob.process(first_snap, trader_id);
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

                // if trader was filled
                if let Some(key) = exec_report
                    .filled_orders
                    .iter()
                    .position(|&o| o.0 == trader_id)
                {
                    dbgp!(
                        "[ KEY ] qty = {:?}, price = {:?}",
                        exec_report.filled_orders[key].1,
                        exec_report.filled_orders[key].2
                    );
                }

                // exec_report.filled_orders

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
                ob = ob.process(snap, trader_id);
                // Trader's move
                if let Ok(m) = midprice.evaluate(&ob) {
                    let trader_order = oms.calculate_buy_order(m, trader_id);
                    match ob.order_loc.get(&trader_id) {
                        None => {
                            dbgp!("[ STRAT] Order not found, place new order");
                            ob.add_limit_order(trader_order);
                        }
                        Some((_, _, price)) if *price == trader_order.price => {
                            dbgp!("[ STRAT] Order found, passing");
                        }
                        Some((_, _, price)) => {
                            dbgp!("[ STRAT] Order found, need price update, place new order");
                            dbgp!(
                                "[ STRAT] Old price {}, New Price {}",
                                *price,
                                trader_order.price
                            );
                            ob.add_limit_order(trader_order);
                        }
                    }
                }
                break;
            }
        }
    }
    dbgp!("{:#?}", ob);
    let _ = ob.get_bbo();
    dbgp!("Done!");
}

fn main() {
    snap_to_event()
}
