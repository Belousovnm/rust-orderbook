#![allow(clippy::too_many_arguments)]
use crate::{
    dbgp,
    event::LimitOrder,
    orderbook::{Order, OrderBook, Side},
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

fn place_head_tail(
    ob: &mut OrderBook,
    qty_head: u32,
    qty_tail: u32,
    qty: u32,
    new_qty: u32,
    id: u64,
    side: Side,
    price: u32,
) {
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
            id: id - 111,
            side,
            price,
            qty: qty_head,
        });
    }
    if let Some(best_offer_price) = ob.best_offer_price {
        if side == Side::Bid && price < best_offer_price {
            let _ = ob.add_limit_order(Order {
                id,
                side,
                price,
                qty,
            });
        }
    } else if side == Side::Bid {
        let _ = ob.add_limit_order(Order {
            id,
            side,
            price,
            qty,
        });
    }
    if let Some(best_bid_price) = ob.best_bid_price {
        if side == Side::Ask && price > best_bid_price {
            let _ = ob.add_limit_order(Order {
                id,
                side,
                price,
                qty,
            });
        }
    } else if side == Side::Ask {
        let _ = ob.add_limit_order(Order {
            id,
            side,
            price,
            qty,
        });
    }
    if qty_tail > 0 {
        let _ = ob.add_limit_order(Order {
            id: id + 111,
            side,
            price,
            qty: qty_tail,
        });
    }
}

pub fn next_snap(snap: Snap, offsets: (Result<Offset, &str>, Result<Offset, &str>)) -> OrderBook {
    let mut ob = OrderBook::new();
    match (offsets.0.ok(), offsets.1.ok()) {
        (
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
            place_head_tail(
                &mut ob,
                qty_head_bid,
                qty_tail_bid,
                qty_bid,
                new_qty_bid,
                id_bid,
                Side::Bid,
                price_bid,
            );
            place_head_tail(
                &mut ob,
                qty_head_ask,
                qty_tail_ask,
                qty_ask,
                new_qty_ask,
                id_ask,
                Side::Ask,
                price_ask,
            );
        }
        (Some((side, bid_price, qty_head, qty, qty_tail, id)), None) => {
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
            place_head_tail(
                &mut ob, qty_head, qty_tail, qty, new_qty, id, side, bid_price,
            );
        }
        (None, Some((side, ask_price, qty_head, qty, qty_tail, id))) => {
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
            place_head_tail(
                &mut ob, qty_head, qty_tail, qty, new_qty, id, side, ask_price,
            );
        }
        (None, None) => place_order_from_snap(snap, &mut ob),
        (_, _) => unreachable!(),
    }
    ob
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        account::TradingAccount, backtest::TestStrategy, management::OrderManagementSystem,
        orderbook::Side,
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
        let strat = &mut TestStrategy::new();
        let oms = &mut OrderManagementSystem::new(strat, TradingAccount::new(0));
        ob = ob.process(snap, oms);
        assert_eq!(ob.get_bbo().unwrap(), (99, 101, 2));
    }
}
