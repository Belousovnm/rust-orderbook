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
use orderbook_lib::dbgp;
use orderbook_lib::event::Event;
use orderbook_lib::orderbook::{OrderBook, Side};
use orderbook_lib::snap::next_snap;

fn snap_to_event() {
    dbgp!("Crafting new Orderbook");
    let mut ob = OrderBook::new("SBER".to_string());
    let mut reader =
        csv::Reader::from_path("/opt/Zenpy/jupyter/data/voskhod/TQBR/SBER/ob.csv").unwrap();

    let mut rdr = reader.deserialize::<Event>();
    // let mut event_counter = 0;
    while let Some(Ok(event)) = rdr.next() {
        dbgp!("{:?}", event);

        // if event_counter == 10 {
        //     break;
        // } else {
        //     event_counter += 1;
        // }
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
    snap_to_event()
}
