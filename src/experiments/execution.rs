use crate::BestBidOffer;
use crate::{dbgp, management::OrderManagementSystem, Order, OrderBook, Snap};

use crate::backtest::FixPriceStrategy;

/// # Panics
///
/// Will panic if File IO fails
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
                oms.update(&exec_report);
                if oms.active_buy_order.is_none() {
                    oms.strategy.buy_price = None;
                } else if oms.active_sell_order.is_none() {
                    oms.strategy.sell_price = None;
                };
                oms.strategy.master_position = 0;
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
                *ob = ob.process(snap, oms);
                // Trader's move
                let bbo = BestBidOffer::evaluate(&ob.get_raw(oms));
                oms.strategy.buy_price = oms.lock_bid_price(bbo).ok();
                oms.strategy.sell_price = oms.lock_ask_price(bbo).ok();
                dbgp!("LOCK BID: {:?}", oms.strategy.buy_price);
                dbgp!("LOCK ASK: {:?}", oms.strategy.sell_price);
                trader_buy_id = 10 * epoch + 3;
                trader_sell_id = 10 * epoch + 7;
                oms.send_orders(ob, bbo, trader_buy_id, trader_sell_id);
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
