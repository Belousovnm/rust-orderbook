mod common;
use orderbook_lib::{
    account::TradingAccount, backtest::TestStrategy, management::OrderManagementSystem, LimitOrder,
    Order, OrderBook, Side, Snap,
};
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
    let mut snap = snap_iter.next().unwrap();
    assert_eq!(snap.price, 1);
    assert_eq!(snap.qty, 51);
    snap = snap_iter.next().unwrap();
    assert_eq!(snap.price, 2);
    assert_eq!(snap.qty, 52);
}

#[rstest]
fn deser_to_ob(deser: Snap) {
    let mut ob = OrderBook::new();
    let strat = &mut TestStrategy::new();
    let oms = &mut OrderManagementSystem::new(strat, TradingAccount::new(0));

    let snap = deser;
    ob = ob.process(snap, oms);
    assert_eq!(ob.get_bbo(), Ok((10, 11, 1)))
}

#[test]
fn exec_report_test() {
    let trader_order_id = 333;
    let mut ob = OrderBook::new();
    let strat = &mut TestStrategy::new();
    let oms = &mut OrderManagementSystem::new(strat, TradingAccount::new(0));
    let snap = Snap {
        exch_epoch: 0,
        vec: vec![
            LimitOrder {
                side: Side::Ask,
                price: 99,
                qty: 100,
            },
            LimitOrder {
                side: Side::Ask,
                price: 100,
                qty: 10,
            },
            LimitOrder {
                side: Side::Ask,
                price: 101,
                qty: 10,
            },
        ],
    };
    ob = ob.process(snap, oms);
    // if matches! {fr.status, OrderStatus::Filled} {
    //     dbgp!("{:#?}, avg_fill_price {}", fr, fr.avg_fill_price());
    // }
    // println!("{:?}", SystemTime::now());
    oms.active_sell_order = Some(Order {
        side: Side::Ask,
        price: 99,
        qty: 10,
        id: trader_order_id,
    });
    let _ = ob.add_limit_order(oms.active_sell_order.unwrap());

    let snap = Snap {
        exch_epoch: 0,
        vec: vec![
            LimitOrder {
                side: Side::Ask,
                price: 99,
                qty: 150,
            },
            LimitOrder {
                side: Side::Ask,
                price: 100,
                qty: 10,
            },
            LimitOrder {
                side: Side::Ask,
                price: 101,
                qty: 5,
            },
        ],
    };
    ob = ob.process(snap, oms);

    let exec_report = ob.add_limit_order(Order {
        side: Side::Bid,
        price: 99,
        qty: 135,
        id: 1010,
    });
    let filled_orders = vec![(332, 100, 99), (333, 10, 99), (334, 25, 99)];
    assert_eq!(exec_report.filled_orders, filled_orders);
}
