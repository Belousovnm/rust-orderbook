mod common;
use orderbook::orderbook::OrderBook;
use orderbook::snap::Snap;
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};

#[fixture]
fn deser() -> Snap {
    let data = "exch_epoch,bid_1_price,bid_2_price,bid_3_price,bid_4_price,bid_5_price,bid_6_price,bid_7_price,bid_8_price,bid_9_price,bid_10_price,ask_1_price,ask_2_price,ask_3_price,ask_4_price,ask_5_price,ask_6_price,ask_7_price,ask_8_price,ask_9_price,ask_10_price,bid_1_qty,bid_2_qty,bid_3_qty,bid_4_qty,bid_5_qty,bid_6_qty,bid_7_qty,bid_8_qty,bid_9_qty,bid_10_qty,ask_1_qty,ask_2_qty,ask_3_qty,ask_4_qty,ask_5_qty,ask_6_qty,ask_7_qty,ask_8_qty,ask_9_qty,ask_10_qty\n170000,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70";
    let mut reader = csv::Reader::from_reader(data.as_bytes());
    reader.deserialize::<Snap>().next().unwrap().unwrap()
}

#[rstest]
fn deser_level(deser: Snap) {
    let mut snap_iter = deser.into_iter();
    assert_eq!(snap_iter.next().unwrap().price, 1);
    assert_eq!(snap_iter.next().unwrap().qty, 52);
    assert_eq!(snap_iter.next().unwrap().price, 3);
    assert_eq!(snap_iter.next().unwrap().qty, 54);
}

#[rstest]
fn deser_to_ob(deser: Snap) {
    let mut ob = OrderBook::new("test".to_string());

    let snap = deser;
    ob.process(snap, Err("test"));
    assert_eq!(ob.get_bbo(), Ok((10, 11, 1)))
}
