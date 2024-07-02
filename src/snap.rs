use crate::event::LimitOrder;
use crate::orderbook::{Order, OrderBook, Side};

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Snap {
    pub exch_epoch: u64,
    pub vec: Vec<LimitOrder>,
}
impl Snap {
    fn new() -> Self {
        Snap {
            exch_epoch: 0,
            vec: Vec::with_capacity(11),
        }
    }

    fn push(&mut self, item: LimitOrder) {
        self.vec.push(item)
    }
}

impl IntoIterator for Snap {
    type Item = LimitOrder;
    type IntoIter = <Vec<LimitOrder> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

type Offset = (Side, u64, u64, u64, u64, u64);

// impl From<Snap> for Order {
//     fn from(id: u64, value: Snap) -> Self {
//         Self {
//             id,
//             side: value.0,
//             price: value.1,
//             qty: value.2,
//         }
//     }
// }

fn place_order_from_snap(snap: Snap, ob: &mut OrderBook) {
    for (id, level) in snap.into_iter().enumerate() {
        let _ = ob.add_limit_order(Order {
            id: id as u64,
            side: level.side,
            price: level.price,
            qty: level.qty,
        });
    }
}

fn next_snap(snap: Snap, ob: &mut OrderBook, offset: Result<Offset, &str>) {
    *ob = OrderBook::new("SPB".to_string());
    match offset.ok() {
        Some((side, price, qty_head, qty, qty_tail, id)) => {
            let mut filtered_snap = Snap::new();
            let mut new_qty = qty_head + qty_tail;
            for level in snap.into_iter() {
                if level.price == price {
                    new_qty = level.qty;
                } else {
                    filtered_snap.push(level)
                }
            }
            place_order_from_snap(filtered_snap, ob);
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

            let _ = ob.add_limit_order(Order {
                id: 666,
                side,
                price,
                qty: qty_head,
            });
            let _ = ob.add_limit_order(Order {
                id,
                side,
                price,
                qty,
            });
            let _ = ob.add_limit_order(Order {
                id: 999,
                side,
                price,
                qty: qty_tail,
            });
        }
        None => place_order_from_snap(snap, ob),
    }
}

impl OrderBook {
    pub fn process(&mut self, snap: Snap, offset: Result<Offset, &str>) {
        next_snap(snap, self, offset);
    }
}

#[cfg(test)]
mod tests {

    // use crate::{next_snap, OrderBook, Side};
    use super::*;
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
        let offset = Err("unittest");
        let mut ob = OrderBook::new("SPB".to_string());
        ob.process(snap, offset);
        assert_eq!(ob.get_bbo().unwrap(), (99, 101, 2));
    }
}
