use crate::{
    dbgp, management::OrderManagementSystem, place_body, Midprice, Order, OrderBook, Snap,
};
use readable::num::{Float, Unsigned};
use std::fmt;

use super::FixSpreadStrategy;

#[derive(Debug, PartialEq)]
pub struct StrategyMetrics {
    pub pnl_abs: f32,
    pub pnl_bps: f32,
    pub volume: f32,
    pub trade_count: u32,
}

impl fmt::Display for StrategyMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PnL abs     = {:.1}\nPnl bps     = {:.3}\nVolume      = {}\nTrade Count = {}",
            self.pnl_abs,
            self.pnl_bps,
            Float::from(self.volume),
            Unsigned::from(self.trade_count)
        )
    }
}

/// # Panics
///
/// Will panic if file read fails
pub fn strategy_flow(
    oms: &mut OrderManagementSystem<FixSpreadStrategy>,
    ob: &mut OrderBook,
    ob_path: &str,
    orders_path: &str,
) -> StrategyMetrics {
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
        *ob = ob.process(first_snap, oms, place_body(false));
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
                oms.update(&exec_report);
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
                *ob = ob.process(snap, oms, place_body(false));
                // Trader's move
                let m = Midprice::evaluate(&ob.get_raw(oms));
                trader_buy_id = Some(epoch + 3);
                trader_sell_id = Some(epoch + 7);
                oms.send_orders(ob, m, trader_buy_id, trader_sell_id);
                break;
            }
        }
    }
    dbgp!("{:#?}", ob);
    let _ = ob.get_bbo();
    let pnl = Midprice::evaluate(ob)
        .unwrap()
        .mul_add(oms.strategy.master_position as f32, oms.account.balance);
    let pnl_bps = match oms.account.cumulative_volume {
        | 0 => 0.0,
        | _ => (pnl / (oms.account.cumulative_volume as f32)) * 10000.0,
    };
    dbgp!("Done!");
    let metrics = StrategyMetrics {
        pnl_abs: pnl * oms.strategy.ticker.step_price,
        pnl_bps,
        volume: oms.account.cumulative_volume as f32 * oms.strategy.ticker.step_price,
        trade_count: oms.account.trade_count,
    };
    println!("{metrics}");
    metrics
}
