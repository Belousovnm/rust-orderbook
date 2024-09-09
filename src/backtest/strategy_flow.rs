use std::collections::HashMap;
use crate::{dbgp, management::OrderManagementSystem, Indicator, Order, OrderBook, Snap, Side};
use readable::num::Unsigned;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct StrategyMetrics {
    pub pnl_abs: f32,
    pub pnl_bps: f32,
    pub volume: u32,
    pub trade_count: u32,
    pub fill_times_bid: HashMap<u64, (bool, u64)>,
    pub fill_times_ask: HashMap<u64, (bool, u64)>,
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
/// Will panic if File IO fails
pub fn snap_to_event(
    midprice: &Indicator,
    oms: &mut OrderManagementSystem,
    ob: &mut OrderBook,
    ob_path: &str,
    orders_path: &str
) -> StrategyMetrics {
    let mut snap_reader = csv::Reader::from_path(ob_path).unwrap();
    let mut trade_reader = csv::Reader::from_path(orders_path).unwrap();
    let mut srdr = snap_reader.deserialize::<Snap>();
    let mut trdr = trade_reader.deserialize::<Order>();
    let mut epoch = 0;
    let mut trader_buy_id=333;
    let mut trader_sell_id=777;
    let mut next_order = Order::default();
    let mut trading_volume: u32 = 0;
    let mut trade_count: u32 = 0;

    let mut  fill_times_bid: HashMap<u64, (bool, u64)> = HashMap::new();
    let mut  fill_times_ask: HashMap<u64, (bool, u64)> = HashMap::new();

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
                let exec_report = ob.add_limit_order(next_order, snap.exch_epoch);

                if oms.is_fp_tracking {
                    for o in &exec_report.filled_orders {
                        if o.0 == trader_buy_id {
                            fill_times_bid.insert(o.3, (true, snap.exch_epoch - o.3));
                        } else if o.0 == trader_sell_id {
                            fill_times_ask.insert(o.3, (true, snap.exch_epoch - o.3));
                        }
                    }
                }
                dbgp!("{:#?}", exec_report);
                let prev_account_balance = oms.account.balance;
                oms.update(&exec_report);
                dbgp!("POS {:#?}", oms.strategy.master_position);
                dbgp!("ACC {:#?}", oms.account.balance);
                if prev_account_balance != oms.account.balance {
                    trading_volume += (oms.account.balance - prev_account_balance).unsigned_abs();
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

                if oms.is_fp_tracking {
                    let buy_offset = ob.get_offset(oms, Side::Bid);
                    let sell_offset = ob.get_offset(oms, Side::Ask);

                    if let Ok((Side::Bid, _price_bid, _qty_head_bid, _qty_bid, _qty_tail_bid, _id_bid, ts_create_bid)) = buy_offset {
                        let epoch = snap.exch_epoch;
                        let live_time = epoch - ts_create_bid;

                        if live_time > 10_000_000_000 {
                            // log bid expiry
                            oms.filled_times_bid.insert(ts_create_bid, (false, 0));
                        }
                    }

                    if let Ok((Side::Ask, _price_ask, _qty_head_ask, _qty_ask, _qty_tail_ask, _id_ask, ts_create_ask)) = sell_offset {
                        let epoch = snap.exch_epoch;
                        let live_time = epoch - ts_create_ask;

                        if live_time > 10_000_000_000 {
                            // log bid expiry
                            oms.filled_times_ask.insert(ts_create_ask, (false, 0));
                        }
                    }

                }
                trader_buy_id = 1000 * epoch + 333;
                trader_sell_id = 1000 * epoch + 777;
                *ob = ob.process(snap, oms);
                // Trader's move
                let m = midprice.evaluate(ob);
                oms.send_orders(ob, m, trader_buy_id, trader_sell_id, epoch);
                // dbgp!("{:?}", ob.get_order(trader_buy_id));
                // dbgp!("{:?}", ob.get_order(trader_sell_id));

                break;
                // } else if strategy_epoch < epoch.min(next_order.id) {
            }
        }
    }
    dbgp!("{:#?}", ob);
    let _ = ob.get_bbo();
    let pnl = midprice.evaluate(ob).unwrap().mul_add(
        oms.strategy.master_position as f32,
        oms.account.balance as f32,
    );
    let pnl_bps = match trading_volume {
        0 => 0.0,
        _ => (pnl / (trading_volume as f32)) * 10000.0,
    };
    dbgp!("Done!");
    let metrics = StrategyMetrics {
        pnl_abs: pnl,
        pnl_bps,
        volume: trading_volume,
        trade_count,
        fill_times_bid,
        fill_times_ask
    };
    println!("{metrics}");
    metrics
}
