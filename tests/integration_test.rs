mod common;
use orderbook_lib::event::Event;
use orderbook_lib::orderbook::{OrderBook, Side};
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};

#[fixture]
fn deser() -> Event {
    let data = "bid_price,ask_price,bid_qty,ask_qty\n100,101,16,13";
    let mut reader = csv::Reader::from_reader(data.as_bytes());
    reader.deserialize::<Event>().next().unwrap().unwrap()
}

#[rstest]
fn deser_level(deser: Event) {
    assert_eq!(deser.bid_price, 100);
    assert_eq!(deser.ask_price, 101);
    assert_eq!(deser.bid_qty, 16);
    assert_eq!(deser.ask_qty, 13);
}

#[rstest]
fn deser_to_ob(deser: Event) {
    let mut ob = OrderBook::new("test".to_string());

    let snap = vec![
        (Side::Bid, deser.bid_price, deser.bid_qty),
        (Side::Ask, deser.ask_price, deser.ask_qty),
    ];
    ob.process(snap, Err("test"));
    assert_eq!(ob.get_bbo(), Ok((100, 101, 1)))
}
