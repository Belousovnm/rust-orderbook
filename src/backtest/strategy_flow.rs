use crate::{
    dbgp, management::OrderManagementSystem, place_body, Midprice, Order,
    OrderBook, Snap,
};
use readable::num::Unsigned;
use std::fmt;

use super::TestStrategy;

#[derive(Debug, PartialEq)]
pub struct StrategyMetrics {
    pub pnl_abs: f32,
    pub pnl_bps: f32,
    pub volume: u32,
    pub trade_count: u32,
}

impl fmt::Display for StrategyMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
			f,
			"PnL abs     = {:.1}\nPnl bps     = {:.3}\nVolume      = {}\nTrade Count = {}",
			self.pnl_abs,
			self.pnl_bps,
			Unsigned::from(self.volume),
			Unsigned::from(self.trade_count)
		)
    }
}

/// # Panics
///
/// Will panic if file read fails
pub fn strategy_flow(
    oms: &mut OrderManagementSystem<TestStrategy>,
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
    let mut trading_volume: u32 = 0;
    let mut trade_count: u32 = 0;
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
        // let strategy_epoch = epoch + 100;
        loop {
            if next_order.id <= epoch {
                // Apply order
                dbgp!("[ EPCH ] order {:?}", next_order.id);
                let exec_report = ob.add_limit_order(next_order);
                dbgp!("{:#?}", exec_report);
                let prev_account_balance = oms.account.balance;
                oms.update(&exec_report);
                dbgp!("POS {:#?}", oms.strategy.master_position);
                dbgp!("ACC {:#?}", oms.account.balance);
                if prev_account_balance != oms.account.balance {
                    trading_volume += (oms.account.balance
                        - prev_account_balance)
                        .unsigned_abs();
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
            } else if epoch < next_order.id {
                // Load next snap
                dbgp!("[ EPCH ] snap {:?}", epoch);
                *ob = ob.process(snap, oms, place_body(false));
                // Trader's move
                let m = Midprice::evaluate(&ob.get_raw(oms));
                trader_buy_id = 10 * epoch + 3;
                trader_sell_id = 10 * epoch + 7;
                oms.send_orders(ob, m, trader_buy_id, trader_sell_id);
                // dbgp!("{:?}", ob.get_order(oms.active_buy_order));
                // dbgp!("{:?}", ob.get_order(oms.active_sell_order));
                break;
                // } else if strategy_epoch < epoch.min(next_order.id) {
            }
        }
    }
    dbgp!("{:#?}", ob);
    let _ = ob.get_bbo();
    let pnl = Midprice::evaluate(ob).unwrap().mul_add(
        oms.strategy.master_position as f32,
        oms.account.balance as f32,
    );
    let pnl_bps = match trading_volume {
        | 0 => 0.0,
        | _ => (pnl / (trading_volume as f32)) * 10000.0,
    };
    dbgp!("Done!");
    let metrics = StrategyMetrics {
        pnl_abs: pnl,
        pnl_bps,
        volume: trading_volume,
        trade_count,
    };
    println!("{metrics}");
    metrics
}
