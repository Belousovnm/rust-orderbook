#![allow(clippy::too_many_arguments)]
use crate::{
    dbgp,
    engine::event::LimitOrder,
    engine::matching_engine::{Order, OrderBook, Side},
    engine::ExecutionReport,
};

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Snap {
    pub exch_epoch: u64,
    pub vec: Vec<LimitOrder>,
}
impl Snap {
    fn new() -> Self {
        Self {
            exch_epoch: 0,
            vec: Vec::with_capacity(16),
        }
    }

    fn push(&mut self, item: LimitOrder) {
        self.vec.push(item);
    }
}

impl IntoIterator for Snap {
    type Item = LimitOrder;
    type IntoIter = <Vec<LimitOrder> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

type Offset = (Side, u32, u32, u32, u32, u64);

fn place_order_from_snap(snap: Snap, ob: &mut OrderBook) {
    for (id, level) in snap.into_iter().enumerate() {
        let _ = ob.add_limit_order(Order {
            id: u64::try_from(id).expect("ID CONVERTION FAILED"),
            // id: id as u64,
            side: level.side,
            price: level.price,
            qty: level.qty,
        });
    }
}

pub fn place_body(allow_fill: bool) -> impl Fn(&mut OrderBook, Order) -> ExecutionReport {
    move |ob: &mut OrderBook, order: Order| {
        let mut _exec_report = ExecutionReport::default();
        if allow_fill {
            // TODO unload strategy crit, log to db
            _exec_report = ob.add_limit_order(order);
        } else if order.side == Side::Ask {
            if let Some(best_bid_price) = ob.best_bid_price {
                if order.price > best_bid_price {
                    _exec_report = ob.add_limit_order(order);
                }
            } else {
                _exec_report = ob.add_limit_order(order);
            }
        } else if order.side == Side::Bid {
            if let Some(best_offer_price) = ob.best_offer_price {
                if order.price < best_offer_price {
                    _exec_report = ob.add_limit_order(order);
                }
            } else {
                _exec_report = ob.add_limit_order(order);
            }
        }
        _exec_report
    }
}

fn place_head_tail(
    ob: &mut OrderBook,
    body_f: impl Fn(&mut OrderBook, Order) -> ExecutionReport,
    qty_head: u32,
    qty_tail: u32,
    qty: u32,
    new_qty: u32,
    id: u64,
    side: Side,
    price: u32,
) -> ExecutionReport {
    dbgp!("{} {} {} {:?} {}", qty_head, qty, qty_tail, side, price);
    let (qty_head, qty_tail) = if new_qty < qty_head + qty_tail {
        let need_to_cut = qty_tail + qty_head - new_qty;
        let cut_qty_tail = qty_tail.min(need_to_cut);
        (
            qty_head + cut_qty_tail - need_to_cut,
            qty_tail - cut_qty_tail,
        )
    } else if new_qty >= qty_head + qty_tail {
        (qty_head, new_qty - qty_head)
    } else {
        unreachable!()
    };
    if qty_head > 0 {
        let _ = ob.add_limit_order(Order {
            id: id - 1,
            side,
            price,
            qty: qty_head,
        });
    }
    let order = Order {
        id,
        side,
        price,
        qty,
    };

    let exec_report = body_f(ob, order);

    if qty_tail > 0 {
        let _ = ob.add_limit_order(Order {
            id: id + 1,
            side,
            price,
            qty: qty_tail,
        });
    }
    exec_report
}

pub fn next_snap(
    snap: Snap,
    offsets: (Result<Offset, &str>, Result<Offset, &str>),
    body_f: impl Fn(&mut OrderBook, Order) -> ExecutionReport,
) -> (OrderBook, Option<ExecutionReport>, Option<ExecutionReport>) {
    let mut ob = OrderBook::new();
    let mut exec_report_bid = None;
    let mut exec_report_ask = None;
    match (offsets.0.ok(), offsets.1.ok()) {
        | (
            Some((Side::Bid, price_bid, qty_head_bid, qty_bid, qty_tail_bid, id_bid)),
            Some((Side::Ask, price_ask, qty_head_ask, qty_ask, qty_tail_ask, id_ask)),
        ) => {
            let mut filtered_snap = Snap::new();
            let mut new_qty_bid = 0;
            let mut new_qty_ask = 0;
            for level in snap {
                if level.price == price_bid && level.side == Side::Bid {
                    new_qty_bid = level.qty;
                } else if level.price == price_ask && level.side == Side::Ask {
                    new_qty_ask = level.qty;
                } else {
                    filtered_snap.push(level);
                }
            }
            place_order_from_snap(filtered_snap, &mut ob);
            exec_report_bid = Some(place_head_tail(
                &mut ob,
                &body_f,
                qty_head_bid,
                qty_tail_bid,
                qty_bid,
                new_qty_bid,
                id_bid,
                Side::Bid,
                price_bid,
            ));
            exec_report_ask = Some(place_head_tail(
                &mut ob,
                &body_f,
                qty_head_ask,
                qty_tail_ask,
                qty_ask,
                new_qty_ask,
                id_ask,
                Side::Ask,
                price_ask,
            ));
        }
        | (Some((side, bid_price, qty_head, qty, qty_tail, id)), None) => {
            let mut filtered_snap = Snap::new();
            let mut new_qty = 0;
            for level in snap {
                if level.price == bid_price && level.side == Side::Bid {
                    new_qty = level.qty;
                } else {
                    filtered_snap.push(level);
                }
            }
            place_order_from_snap(filtered_snap, &mut ob);
            exec_report_bid = Some(place_head_tail(
                &mut ob, body_f, qty_head, qty_tail, qty, new_qty, id, side, bid_price,
            ));
        }
        | (None, Some((side, ask_price, qty_head, qty, qty_tail, id))) => {
            let mut filtered_snap = Snap::new();
            let mut new_qty = 0;
            for level in snap {
                if level.price == ask_price && level.side == Side::Ask {
                    new_qty = level.qty;
                } else {
                    filtered_snap.push(level);
                }
            }
            place_order_from_snap(filtered_snap, &mut ob);
            exec_report_ask = Some(place_head_tail(
                &mut ob, body_f, qty_head, qty_tail, qty, new_qty, id, side, ask_price,
            ));
        }
        | (None, None) => place_order_from_snap(snap, &mut ob),
        | (_, _) => unreachable!(),
    }
    (ob, exec_report_bid, exec_report_ask)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::engine::Ticker;
    use crate::{
        backtest::FixSpreadStrategy, engine::account::TradingAccount,
        engine::matching_engine::Side, management::OrderManagementSystem,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn test_orders_from_snapshot() {
        let snap = Snap {
            exch_epoch: 0,
            vec: vec![
                LimitOrder {
                    side: Side::Bid,
                    price: 99,
                    qty: 1,
                },
                LimitOrder {
                    side: Side::Ask,
                    price: 101,
                    qty: 1,
                },
            ],
        };
        // let offset = Ok((Side::Bid, 101, 0, 1, 0, 999));
        let mut ob = OrderBook::new();
        let strat = &mut FixSpreadStrategy::new(Ticker::default());
        let oms = &mut OrderManagementSystem::new(strat, TradingAccount::new(0.0));
        ob = ob.process(snap, oms, place_body(false));
        assert_eq!(ob.get_bbo().unwrap(), (99, 101, 2));
    }
}
