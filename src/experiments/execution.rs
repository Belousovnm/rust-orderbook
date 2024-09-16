use crate::{
    dbgp,
    experiments::{Ready, Schedule},
    management::OrderManagementSystem,
    BestBidOffer, Order, OrderBook, Snap,
};

use crate::backtest::FixPriceStrategy;

/// # Panics
///
/// Will panic if file read fails
pub fn execution_flow(
    oms: &mut OrderManagementSystem<FixPriceStrategy>,
    ob: &mut OrderBook,
    ob_path: &str,
    orders_path: &str,
) {
    let mut snap_reader = csv::Reader::from_path(ob_path).unwrap();
    let mut trade_reader = csv::Reader::from_path(orders_path).unwrap();
    let mut srdr = snap_reader.deserialize::<Snap>();
    let mut trdr = trade_reader.deserialize::<Order>();
    let mut epoch = 0;
    let mut schedule = Schedule {
        cooldown: 0,
        counter: 0,
    };
    let mut trader_buy_id;
    let mut trader_sell_id;
    let mut next_order = Order::default();
    dbgp!("Crafting Orderbook");
    // Load first snapshot
    if let Some(Ok(first_snap)) = srdr.next() {
        epoch = first_snap.exch_epoch;
        dbgp!("[ EPCH ] snap {:?}", epoch);
        *ob = ob.process(first_snap, oms);
    }

    // Skip all trades that occured before the first snapshot
    while next_order.id < epoch {
        if let Some(Ok(order)) = trdr.next() {
            next_order = order;
        }
    }

    'a: while let Some(Ok(snap)) = srdr.next() {
        epoch = snap.exch_epoch;
        // let strategy_epoch = epoch + 100;
        loop {
            if next_order.id <= epoch {
                // Apply order
                dbgp!("[ EPCH ] order {:?}", next_order.id);
                let exec_report = ob.add_limit_order(next_order);
                dbgp!("{:#?}", exec_report);
                // Updates active order when filled, releases price lock, restarts scheduler
                oms.update(&exec_report, &next_order.id, &mut schedule);
                // Load next order
                if let Some(Ok(order)) = trdr.next() {
                    next_order = order;
                } else {
                    // Replay until last order
                    break 'a;
                }
            // If next snap before order
            } else if epoch < next_order.id {
                // Load next snap
                dbgp!("[ EPCH ] snap {:?}", epoch);
                // Trader's move
                // Experiment is live
                dbgp!(
                    "cooldown={}, counter={}",
                    schedule.cooldown,
                    schedule.counter
                );
                // Active orders
                if let Some(order) = oms.active_buy_order.or(oms.active_sell_order) {
                    // 10s censoring
                    if epoch - order.id >= 10_000_000_000 {
                        oms.cancel_all_orders(ob);
                        dbgp!(
                            "[  DB  ] epoch_start={} epoch_end={} censored={}",
                            order.id,
                            epoch,
                            1
                        );
                        schedule = Schedule::new();
                    } else {
                        *ob = ob.process(snap, oms);
                        trader_buy_id = epoch + 3;
                        trader_sell_id = epoch + 7;
                        oms.send_orders(ob, None, trader_buy_id, trader_sell_id);
                    }
                // No active orders
                } else {
                    match schedule.ready() {
                        Ready::Yes => {
                            // Lock new price after cooldown
                            dbgp!("!!!! READY !!!!!");
                            schedule.set_counter(0);
                            *ob = ob.process(snap, oms);
                            let bbo = BestBidOffer::evaluate(&ob.get_raw(oms));
                            oms.strategy.buy_price = oms.lock_bid_price(bbo).ok();
                            oms.strategy.sell_price = oms.lock_ask_price(bbo).ok();
                            dbgp!("[ LOCK ] BID: {:?}", oms.strategy.buy_price);
                            dbgp!("[ LOCK ] ASK: {:?}", oms.strategy.sell_price);
                            trader_buy_id = epoch + 3;
                            trader_sell_id = epoch + 7;
                            oms.send_orders(ob, None, trader_buy_id, trader_sell_id);
                        }
                        Ready::No => schedule.incr_counter(),
                    };
                }
                // dbgp!("{:?}", ob.get_order(oms.active_buy_order));
                // dbgp!("{:?}", ob.get_order(oms.active_sell_order));
                break;
                // } else if strategy_epoch < epoch.min(next_order.id) {
            }
        }
    }
    dbgp!("{:#?}", ob);
    let _ = ob.get_bbo();
    dbgp!("Done!");
}
