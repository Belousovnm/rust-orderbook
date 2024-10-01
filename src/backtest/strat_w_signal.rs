use crate::{
    dbgp,
    experiments::{Ready, Schedule},
    management::OrderManagementSystem,
    place_body, BestBidOffer, Order, OrderBook, Signal, Snap,
};

use crate::backtest::FixPriceStrategy;

/// # Panics
///
/// Will panic if file read fails
pub fn signal_flow(
    oms: &mut OrderManagementSystem<FixPriceStrategy>,
    ob: &mut OrderBook,
    ob_path: &str,
    orders_path: &str,
    signals_path: &str,
) {
    let mut snap_reader = csv::Reader::from_path(ob_path).unwrap();
    let mut trade_reader = csv::Reader::from_path(orders_path).unwrap();
    let mut signals_reader = csv::Reader::from_path(signals_path).unwrap();
    let mut srdr = snap_reader.deserialize::<Snap>();
    let mut trdr = trade_reader.deserialize::<Order>();
    let mut sigrdr = signals_reader.deserialize::<Signal>();
    let mut epoch = 0;
    let mut trader_buy_id;
    let mut trader_sell_id;
    let mut next_order = Order::default();
    let mut next_signal = Signal::default();
    dbgp!("Crafting Orderbook");
    // Load first snapshot
    if let Some(Ok(first_snap)) = srdr.next() {
        epoch = first_snap.exch_epoch;
        dbgp!("[ EPCH ] snap {:?}", epoch);
        *ob = ob.process(first_snap, oms, place_body(true));
    }

    // Skip all trades that occured before the first snapshot
    while next_order.id < epoch {
        if let Some(Ok(order)) = trdr.next() {
            next_order = order;
        }
    }
    // Skip all signals that occured before the first snapshot
    while next_signal.exch_epoch < epoch {
        if let Some(Ok(signal)) = sigrdr.next() {
            next_signal = signal;
        }
    }

    'a: while let Some(Ok(snap)) = srdr.next() {
        epoch = snap.exch_epoch;
        // let strategy_epoch = epoch + 100;
        loop {
            if next_order.id <= epoch.min(next_signal.exch_epoch) {
                // Apply order
                dbgp!("[ EPCH ] order {:?}", next_order.id);
                let exec_report = ob.add_limit_order(next_order);
                dbgp!("{:#?}", exec_report);
                // Updates active order when filled, releases price lock, restarts scheduler
                oms.update(&exec_report);
                // Load next order
                if let Some(Ok(order)) = trdr.next() {
                    next_order = order;
                } else {
                    // Replay until last order
                    break 'a;
                }
            // If next snap before order
            } else if epoch < next_order.id.min(next_signal.exch_epoch) {
                // Load next snap
                dbgp!("[ EPCH ] snap {:?}", epoch);
                if let Some(order) = oms.active_buy_order.or(oms.active_sell_order) {
                    // 10s censoring
                    if epoch - order.id >= 10_000_000_000 {
                        oms.cancel_all_orders(ob);
                        println!("[  DB  ];{};{};{};{};", order.id, epoch, 10_000_000, 1);
                        oms.lock_release();
                        oms.schedule = Schedule::new();
                    } else {
                        *ob = ob.process_w_takers(snap, oms, place_body(true));
                        trader_buy_id = epoch + 3;
                        trader_sell_id = epoch + 7;
                        oms.send_orders(ob, epoch, trader_buy_id, trader_sell_id);
                    }
                // No active orders
                } else {
                    match oms.schedule.ready() {
                        | Ready::Yes => {
                            // Lock new price after cooldown
                            dbgp!("!!!! READY !!!!!");
                            oms.schedule.set_counter(0);
                            *ob = ob.process_w_takers(snap, oms, place_body(true));
                            let bbo = BestBidOffer::evaluate(ob);
                            dbgp!("bbo = {:?}", bbo);
                            oms.strategy.buy_price = oms.lock_bid_price(bbo).ok();
                            oms.strategy.sell_price = oms.lock_ask_price(bbo).ok();
                            dbgp!("[ LOCK ] BID: {:?}", oms.strategy.buy_price);
                            dbgp!("[ LOCK ] ASK: {:?}", oms.strategy.sell_price);
                            trader_buy_id = epoch + 3;
                            trader_sell_id = epoch + 7;
                            oms.send_orders(ob, epoch, trader_buy_id, trader_sell_id);
                        }
                        | Ready::No => oms.schedule.incr_counter(),
                    };
                }
                // dbgp!("{:?}", ob.get_order(oms.active_buy_order));
                // dbgp!("{:?}", ob.get_order(oms.active_sell_order));
                break;
            } else if next_signal.exch_epoch < epoch.min(next_order.id) {
                todo!()
            }
        }
    }
    dbgp!("{:#?}", ob);
    let _ = ob.get_bbo();
    dbgp!("Done!");
}
