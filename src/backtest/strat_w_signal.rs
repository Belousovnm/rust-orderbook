use crate::{
    backtest::StrategyMetrics,
    dbgp,
    error::MyError,
    experiments::{Ready, Schedule},
    indicators::{EmaMidprice, Midprice},
    management::OrderManagementSystem,
    place_body, Order, OrderBook, OrderStatus, Side, Signal, Snap,
};
use log::info;
use log4rs::{self, config::Deserializers};

use crate::backtest::SignalStrategy;

/// # Panics
///
/// Will panic if file read fails
///
/// # Errors
///
/// ...
pub fn signal_flow(
    oms: &mut OrderManagementSystem<SignalStrategy>,
    ob: &mut OrderBook,
    ob_path: &str,
    orders_path: &str,
    signals_path: &str,
) -> Result<(), MyError> {
    log4rs::init_file("logging_config.yaml", Deserializers::default()).unwrap();
    let mut snap_reader = csv::Reader::from_path(ob_path)?;
    let mut orders_reader = csv::Reader::from_path(orders_path)?;
    let mut signals_reader = csv::Reader::from_path(signals_path)?;
    let mut srdr = snap_reader.deserialize::<Snap>();
    let mut trdr = orders_reader.deserialize::<Order>();
    let mut sigrdr = signals_reader.deserialize::<Signal>();
    let mut epoch = 0;
    let mut trader_buy_id = None;
    let mut trader_sell_id = None;
    let mut next_order = Order::default();
    let mut next_signal = Signal::default();
    let mut clock = 0;
    let mut schedule_soft = Schedule::new(5_000_000_000);
    let mut schedule_hard = Schedule::new(u64::MAX);
    let mut ema = EmaMidprice::new(1.0);
    let default_qty = oms.strategy.qty;
    dbgp!("Crafting Orderbook");
    // Load first snapshot
    if let Some(Ok(first_snap)) = srdr.next() {
        epoch = first_snap.exch_epoch;
        dbgp!("[ EPCH ] snap {:?}", epoch);
        *ob = ob.process(first_snap, oms, place_body(true));
    }

    // Skip all trades that occured before the first snapshot
    while next_order.id < epoch {
        dbgp!("{:?}", next_order);
        if let Some(Ok(order)) = trdr.next() {
            next_order = order;
        }
    }
    // Skip all signals that occured before the first snapshot
    while next_signal.exch_epoch < epoch {
        if let Some(Ok(signal)) = sigrdr.next() {
            next_signal = signal;
            dbgp!("[ SGNL ] {:?}", next_signal);
        }
    }

    'a: while let Some(Ok(snap)) = srdr.next() {
        epoch = snap.exch_epoch;
        loop {
            if next_order.id <= epoch.min(next_signal.exch_epoch) {
                // Apply order
                dbgp!("[ EPCH ] order {:?}", next_order.id);
                let exec_report = ob.add_limit_order(next_order);
                dbgp!("{:#?}", exec_report);
                oms.update(&exec_report);
                info!(target: "pnl", "{};{:?}", next_order.id, oms.get_pnl(Midprice::evaluate(ob), false));
                info!(target: "pos", "{};{:?}", next_order.id, oms.strategy.master_position);
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
                *ob = ob.process(snap, oms, place_body(true));
                info!(target: "pnl", "{};{:?}", epoch, oms.get_pnl(Midprice::evaluate(ob), false));
                info!(target: "pos", "{};{:?}", epoch, oms.strategy.master_position);
                // hedging
                // dbgp!("counter {:?}", oms.schedule.counter);
                schedule_soft.set_counter(epoch - clock);
                schedule_hard.set_counter(epoch - clock);
                match (schedule_soft.ready(), schedule_hard.ready()) {
                    | (Ready::Yes, Ready::No) => {
                        dbgp!("Hedging as Maker, time passed={}", epoch - clock);
                        let m = ema.evaluate(&ob.get_raw(oms));
                        match oms.strategy.master_position.cmp(&0) {
                            // ??? untested ???
                            | std::cmp::Ordering::Less => {
                                oms.strategy.qty = oms.strategy.master_position.unsigned_abs();
                                oms.send_close_orders(ob, m, Some(epoch + 3), None);
                                oms.strategy.qty = default_qty;
                            }
                            | std::cmp::Ordering::Greater => {
                                oms.strategy.qty = oms.strategy.master_position.unsigned_abs();
                                oms.send_close_orders(ob, m, None, Some(epoch + 7));
                                oms.strategy.qty = default_qty;
                            }
                            | std::cmp::Ordering::Equal => {}
                        }
                    }
                    | (Ready::Yes, Ready::Yes) => {
                        dbgp!("Hedging as Taker (actually no)");
                        let m = Midprice::evaluate(&ob.get_raw(oms));
                        match oms.strategy.master_position.cmp(&0) {
                            | std::cmp::Ordering::Less => {
                                oms.strategy.qty = oms.strategy.master_position.unsigned_abs();
                                let crit = oms.strategy.buy_close_criterion;
                                oms.strategy.buy_close_criterion = 0.0;
                                oms.send_close_orders(ob, m, Some(epoch + 3), None);
                                oms.strategy.buy_close_criterion = crit;
                                oms.strategy.qty = default_qty;
                                // info!(target: "pnl", "{};{:?}", epoch, oms.get_pnl(Midprice::evaluate(ob), false));
                            }
                            | std::cmp::Ordering::Greater => {
                                oms.strategy.qty = oms.strategy.master_position.unsigned_abs();
                                let crit = oms.strategy.buy_close_criterion;
                                oms.strategy.sell_close_criterion = -0.0;
                                oms.send_close_orders(ob, m, None, Some(epoch + 7));
                                oms.strategy.sell_close_criterion = crit;
                                oms.strategy.qty = default_qty;
                                // info!(target: "pnl", "{};{:?}", epoch, oms.get_pnl(Midprice::evaluate(ob), false));
                            }
                            | std::cmp::Ordering::Equal => {}
                        }
                    }
                    | (Ready::No, Ready::Yes) => {
                        unreachable!()
                    }
                    | (Ready::No, Ready::No) => {}
                }
                break;
            } else if next_signal.exch_epoch <= epoch.min(next_order.id) {
                if schedule_soft.ready() == Ready::Yes {
                    dbgp!("[ SGNL ] {:?}", next_signal);
                    let m = Midprice::evaluate(&ob.get_raw(oms));
                    if next_signal.side == Side::Bid {
                        trader_buy_id = Some(next_signal.exch_epoch + 3);
                        trader_sell_id = None;
                    } else if next_signal.side == Side::Ask {
                        trader_buy_id = None;
                        trader_sell_id = Some(next_signal.exch_epoch + 7);
                    }
                    let (buy_exec_report, sell_exec_report) =
                        oms.send_open_orders(ob, m, trader_buy_id, trader_sell_id);
                    if buy_exec_report.is_some_and(|e| {
                        e.status == OrderStatus::Filled || e.status == OrderStatus::PartiallyFilled
                    }) || sell_exec_report.is_some_and(|e| {
                        e.status == OrderStatus::Filled || e.status == OrderStatus::PartiallyFilled
                    }) {
                        dbgp!("Cooldown started!");
                        schedule_soft.counter = 0;
                        schedule_hard.counter = 0;
                    };
                    info!(target: "pnl", "{};{:?}", next_signal.exch_epoch, oms.get_pnl(Midprice::evaluate(ob), false));
                    info!(target: "pos", "{};{:?}", next_signal.exch_epoch, oms.strategy.master_position);
                    clock = next_signal.exch_epoch;
                }

                if let Some(Ok(signal)) = sigrdr.next() {
                    next_signal = signal;
                    // next_signal.exch_epoch += 150_000_000;
                } else {
                    // break;
                    next_signal.exch_epoch = u64::MAX;
                }
            } else {
                unreachable!()
            }
        }
    }
    dbgp!("{:#?}", ob);
    let _ = ob.get_bbo();
    let ref_price = Midprice::evaluate(ob);
    let pnl_abs = oms.get_pnl(ref_price, false).unwrap();
    let pnl_bps = oms.get_pnl(ref_price, true).unwrap();
    dbgp!("Done!");
    let metrics = StrategyMetrics {
        pnl_abs,
        pnl_bps,
        volume: oms.account.cumulative_volume as f32 * oms.strategy.ticker.step_price,
        trade_count: oms.account.trade_count,
    };
    println!("{metrics}");
    Ok(())
}
