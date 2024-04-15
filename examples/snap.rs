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

use orderbook_lib::event::Event;
use orderbook_lib::{dbgp, next_snap, OrderBook, Side};

fn event_to_snap() {
    dbgp!("Crafting new Orderbook");
    let mut ob = OrderBook::new("SBER".to_string());
    let mut reader =
        csv::Reader::from_path("/opt/Zenpy/jupyter/data/voskhod/TQBR/SBER/ob.csv").unwrap();

    let mut rdr = reader.deserialize::<Event>();

    while let Some(Ok(event)) = rdr.next() {
        dbgp!("{:?}", event);

        let snap = vec![
            (Side::Bid, event.bid_price, event.bid_qty),
            (Side::Ask, event.ask_price, event.ask_qty),
        ];
        next_snap(snap, &mut ob, Err("mock"));
    }
    dbgp!("{:#?}", ob);
    dbgp!("{:?}", ob.get_bbo());
    dbgp!("Done!");
}

fn main() {
    event_to_snap()
}
