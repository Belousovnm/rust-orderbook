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

// use orderbook_lib::dbgp;
use orderbook_lib::orderbook::{Order, OrderBook};
use orderbook_lib::snap::Snap;

fn snap_to_event() {
    println!("Crafting Orderbook");
    let mut ob = OrderBook::new("SBER".to_string());
    let mut snap_reader =
        csv::Reader::from_path("/opt/Zenpy/jupyter/data/voskhod/TQBR/SBER/ob.csv").unwrap();
    let mut trade_reader =
        csv::Reader::from_path("/opt/Zenpy/jupyter/data/voskhod/TQBR/SBER/trades.csv").unwrap();
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

    while let Some(Ok(snap)) = srdr.next() {
        let epoch = snap.exch_epoch;
        loop {
            println!("{} {}", next_order.id, epoch);
            if next_order.id <= epoch {
                let exec_report = ob.add_limit_order(next_order);
                println!("{:#?}", exec_report);
                if let Some(Ok(order)) = trdr.next() {
                    next_order = order;
                } else {
                    break;
                }
            } else if next_order.id > epoch {
                ob.process(snap, Err(""));
                break;
            }
        }
    }
    println!("{:#?}", ob);
    println!("{:#?}", ob.get_bbo());
    println!("Done!");
}

fn main() {
    snap_to_event()
}
