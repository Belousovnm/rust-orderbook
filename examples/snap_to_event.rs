use orderbook::dbgp;
use orderbook::indicators::Indicator;
use orderbook::management::OrderManagementSystem;
use orderbook::orderbook::{Order, OrderBook};
use orderbook::snap::Snap;
use orderbook::strategy::{Strategy, StrategyName};

fn snap_to_event() {
    dbgp!("Crafting Orderbook");
    let mut ob = OrderBook::new("SecName".to_string());
    let mut snap_reader = csv::Reader::from_path("data/ob.csv").unwrap();
    let mut trade_reader = csv::Reader::from_path("data/orders.csv").unwrap();
    let mut srdr = snap_reader.deserialize::<Snap>();
    let mut trdr = trade_reader.deserialize::<Order>();
    let mut epoch = 0;
    let mut next_order = Order::default();
    let offset = Err("First pass");

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
        ob.process(first_snap, Err("First snap"));
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
                ob.process(snap, offset);

                // Trader's move
                if let Ok(m) = midprice.evaluate(&ob) {
                    let trader_exec_report = oms.send_buy_order(&mut ob, m);
                    dbgp!("{:#?}", trader_exec_report);
                    let offset = ob.get_offset(666);
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
