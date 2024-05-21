// use polars::prelude::*;
// const PATH: &'static str = "/opt/Zenpy/jupyter/data/voskhod/TQBR/CBOM/CBOM.2023-11-20.parquet";
//
// fn main() -> PolarsResult<()> {
//     let df = LazyFrame::scan_parquet(PATH, ScanArgsParquet::default())?
//         .select([all()])
//         .collect()?;
//     dbg!(df);
//     Ok(())
// };
use orderbook::dbgp;
use orderbook::orderbook::{Order, OrderBook};
use orderbook::snap::Snap;

fn snap_to_event() {
    dbgp!("Crafting Orderbook");
    let mut ob = OrderBook::new("SecName".to_string());
    let mut snap_reader =
        csv::Reader::from_path("/opt/Zenpy/jupyter/data/voskhod/ob_GAZP.csv").unwrap();
    let mut trade_reader =
        csv::Reader::from_path("/opt/Zenpy/jupyter/data/voskhod/orders_GAZP.csv").unwrap();
    let mut srdr = snap_reader.deserialize::<Snap>();
    let mut trdr = trade_reader.deserialize::<Order>();
    let mut epoch = 0;
    let mut next_order = Order::default();

    if let Some(Ok(first_snap)) = srdr.next() {
        epoch = first_snap.exch_epoch;
        ob.process(first_snap, Err("First snap"));
    }

    while next_order.id < epoch {
        if let Some(Ok(order)) = trdr.next() {
            next_order = order;
        }
    }

    'a: while let Some(Ok(snap)) = srdr.next() {
        let epoch = snap.exch_epoch;
        loop {
            if next_order.id <= epoch {
                let exec_report = ob.add_limit_order(next_order);
                dbgp!("{:#?}", exec_report);
                if let Some(Ok(order)) = trdr.next() {
                    next_order = order;
                } else {
                    break 'a;
                }
            } else if next_order.id > epoch {
                ob.process(snap, Err(""));
                let _ = ob.get_bbo();
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
