use crate::{
    backtest::StrategyMetrics, dbgp, management::OrderManagementSystem, place_body, Midprice,
    Order, OrderBook, Side, Signal, Snap,
};

use crate::backtest::FixSpreadStrategy;

/// # Panics
///
/// Will panic if file read fails
pub fn signal_flow(
    oms: &mut OrderManagementSystem<FixSpreadStrategy>,
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
            dbgp!("[ SGNL ] {:?}", next_signal);
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
                *ob = ob.process(snap, oms, place_body(true));
                break;
            } else if next_signal.exch_epoch < epoch.min(next_order.id) {
                dbgp!("[ SGNL ] {:?}", next_signal);
                oms.cancel_all_orders(ob);
                let m = Midprice::evaluate(&ob.get_raw(oms));
                if next_signal.side == Side::Bid {
                    oms.strategy.buy_criterion = 0.0005;
                    oms.strategy.sell_criterion = 0.0010;
                } else if next_signal.side == Side::Ask {
                    oms.strategy.buy_criterion = -0.0010;
                    oms.strategy.sell_criterion = -0.0005;
                }
                trader_buy_id = next_signal.exch_epoch + 3;
                trader_sell_id = next_signal.exch_epoch + 7;
                oms.send_orders(ob, m, trader_buy_id, trader_sell_id);

                if let Some(Ok(signal)) = sigrdr.next() {
                    next_signal = signal;
                }
            }
        }
    }
    dbgp!("{:#?}", ob);
    let _ = ob.get_bbo();
    let pnl = Midprice::evaluate(ob).unwrap().mul_add(
        oms.strategy.master_position as f32,
        oms.account.balance as f32,
    );
    let pnl_bps = match oms.account.cumulative_volume {
        | 0 => 0.0,
        | _ => (pnl / (oms.account.cumulative_volume as f32)) * 10000.0,
    };
    dbgp!("Done!");
    let metrics = StrategyMetrics {
        pnl_abs: pnl,
        pnl_bps,
        volume: oms.account.cumulative_volume,
        trade_count: oms.account.trade_count,
    };
    println!("{metrics}");
}
