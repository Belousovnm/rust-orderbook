#![allow(clippy::too_many_arguments)]

use std::collections::VecDeque;
use crate::{
    dbgp,
    event::LimitOrder,
    orderbook::{Order, OrderBook, Side, HalfBook},
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
            is_synth: false,
            send_time: 0,
            fill_time: 0
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
            is_synth: false,
            send_time: 0,
            fill_time: 0
        });
    }
    if let Some(best_offer_price) = ob.best_offer_price {
        if side == Side::Bid && price < best_offer_price {
            let _ = ob.add_limit_order(Order {
                id,
                side,
                price,
                qty,
                is_synth: false,
                send_time: 0,
                fill_time: 0
            });
        }
    } else if side == Side::Bid {
        let _ = ob.add_limit_order(Order {
            id,
            side,
            price,
            qty,
            is_synth: false,
            send_time: 0,
            fill_time: 0
        });
    }
    if let Some(best_bid_price) = ob.best_bid_price {
        if side == Side::Ask && price > best_bid_price {
            let _ = ob.add_limit_order(Order {
                id,
                side,
                price,
                qty,
                is_synth: false,
                send_time: 0,
                fill_time: 0,
            });
        }
    } else if side == Side::Ask {
        let _ = ob.add_limit_order(Order {
            id,
            side,
            price,
            qty,
            is_synth: false,
            send_time: 0,
            fill_time: 0
        });
    }
    if qty_tail > 0 {
        let _ = ob.add_limit_order(Order {
            id: id + 111,
            side,
            price,
            qty: qty_tail,
            is_synth: false,
            send_time: 0,
            fill_time: 0
        });
    }
}


fn process_stale_level(level : &VecDeque<Order>)
{
    for o in level {
        if o.is_synth {
            // mark order as not done
        }
    }
}

fn is_stale_level(level : &VecDeque<Order>, side: Side, boundary_price : u32) -> bool {
    let cur_price = level.front().map_or(u32::MAX, |order| order.price);

    if side == Side::Bid && cur_price >= boundary_price {
        return true;
    }

    if side == Side::Ask && cur_price <= boundary_price {
        return true;
    }

    false
}

fn filter_stale_synth_orders(level: &VecDeque<Order>, cur_epoch : u64) -> VecDeque<Order> {

    let filtered_result: VecDeque<Order> = level
        .iter()
        .filter(|order| ((order.is_synth && cur_epoch.abs_diff(order.send_time) < 10000000000 && order.qty > 0) || !order.is_synth))
        .cloned() // Clone the orders to create a new VecDeque
        .collect();

    filtered_result
}

fn merge_level_with_qty(level: &mut VecDeque<Order>, qty : u32, cur_epoch : u64) -> VecDeque<Order> {

    let mut need_retain = true;
    let mut tot_qty = 0;
    for o in &mut *level {

        if !o.is_synth {
            if !need_retain
            {
                o.fill_time = u64::MAX;
            } else {
                tot_qty += o.qty;
                if tot_qty >= qty {
                    need_retain = false;
                    o.qty -= tot_qty - qty;
                }
            }
        }
    }

    let filtered_result: VecDeque<Order> = level
        .iter()
        .filter(|order| ((order.is_synth && cur_epoch.abs_diff(order.send_time) < 10000000000 && order.qty > 0) || (!order.is_synth && order.fill_time != u64::MAX)))
        .cloned() // Clone the orders to create a new VecDeque
        .collect();

    filtered_result
}

fn merge_halfbook_with_snap(halfbook: &mut HalfBook, snap: &Snap, side:Side, boundary_price : u32) -> HalfBook {

    let mut result_ob = HalfBook::new(side);
    let mut i_book:usize = 0;  // Pointer for price_levels
    let mut i_snap:usize = 0;  // Pointer for snap
    let mut i_result:usize = 0; // pointer to result

    let snap_len = snap.vec.len();
    let book_len = halfbook.price_levels.len();

    while i_book < book_len && i_snap < snap_len {

        let level = &halfbook.price_levels[i_book];

        if is_stale_level(level, side, boundary_price)
        {
            process_stale_level(level);
            i_book += 1;
            continue;
        }

        let cur_book_price = halfbook.price_levels[i_book].front().map_or(u32::MAX, |order| order.price);
        let cur_snap_price = snap.vec[i_snap].price;

        let snap_ahead = ((side == Side::Ask) && (cur_book_price < cur_snap_price)) || ((side == Side::Bid) && (cur_book_price > cur_snap_price));

        if snap_ahead {
            // Copy the whole level into the new order book
            let filtered_level = filter_stale_synth_orders(&halfbook.price_levels[i_book], snap.exch_epoch);
            let l = filtered_level.len();

            if l > 0 {
                result_ob.price_levels.push(filtered_level);
                result_ob.price_map.insert(cur_book_price, i_result);
                i_result += 1;
                i_book += 1;
            } else {
                i_book += 1;
            }
        } else if cur_book_price == cur_snap_price {
            // Merge the level with Snap's qty
            let merged_level = merge_level_with_qty(& mut halfbook.price_levels[i_book], snap.vec[i_snap].qty, snap.exch_epoch);
            result_ob.price_levels.push(merged_level);
            result_ob.price_map.insert(cur_book_price, i_result);
            i_book += 1;
            i_snap += 1;
            i_result += 1;
        } else {
            // Generate a single order from Snap
            let lo = snap.vec[i_snap];
            let price = lo.price;
            let qty = lo.qty;
            let o = Order {
                id: i_result as u64,
                side: side,
                price: price,
                qty: qty,
                is_synth: false,
                send_time: 0,
                fill_time: 0,
            };

            let mut level: VecDeque<Order> = VecDeque::new();
            level.push_back(o);
            result_ob.price_levels.push(level);
            result_ob.price_map.insert(price, i_result);
            i_snap += 1;
            i_result += 1;
        }
    }

    // If there are remaining levels in a book, copy them to the new order book
    while i_book < book_len {

        let level = &halfbook.price_levels[i_book];

        if is_stale_level(level, side, boundary_price)
        {
            process_stale_level(level);
            break;
        }
        let cur_book_price = halfbook.price_levels[i_book].front().map_or(u32::MAX, |order| order.price);
        let filtered_level = filter_stale_synth_orders(&halfbook.price_levels[i_book], snap.exch_epoch);

        let l = filtered_level.len();

        if l > 0 {
            result_ob.price_levels.push(filtered_level);
            result_ob.price_map.insert(cur_book_price, i_result);
            i_result += 1;
            i_book += 1;
        } else
        {
            i_book += 1;
        }
    }

    while i_snap < snap.vec.len() {
        // Generate a single order from Snap
        let lo = snap.vec[i_snap];
        let price = lo.price;
        let qty = lo.qty;
        let o = Order {
            id: i_result as u64,
            side: side,
            price: price,
            qty: qty,
            is_synth: false,
            send_time: 0,
            fill_time: 0,
        };

        let mut level: VecDeque<Order> = VecDeque::new();
        level.push_back(o);
        result_ob.price_levels.push(level);
        result_ob.price_map.insert(price, i_result);
        i_snap += 1;
        i_result += 1;
    }

    result_ob
}

unsafe fn next_snap_fp(snap: Snap, ob : & mut OrderBook) -> OrderBook {

    let mut ob_res = OrderBook::new();

    let mid = snap.vec.len() / 2; // Calculate the midpoint
    let (bid_orders, ask_orders) = snap.vec.split_at(mid);

    let bid_snap = Snap {
        exch_epoch: snap.exch_epoch,
        vec: bid_orders.to_vec(), // Convert the slice back to Vec
    };

    let ask_snap = Snap {
        exch_epoch: snap.exch_epoch,
        vec: ask_orders.to_vec(), // Convert the slice back to Vec
    };

    let ask_book = merge_halfbook_with_snap(& mut ob.ask_book, &ask_snap, Side::Ask, bid_orders[0].price);
    let bid_book = merge_halfbook_with_snap(& mut ob.bid_book, &bid_snap, Side::Bid, ask_orders[0].price);

    let best_ask_price = ask_book.price_levels[0].front().map_or(u32::MAX, |order| order.price);
    //let best_bid_price = bid_book.price_levels[0].front().map_or(u64::MAX, |order| order.price);
    let best_bid_price = bid_book.price_levels[0].front().map_or(u32::MAX, |order| order.price);

    ob_res.best_bid_price = Some(best_bid_price);
    ob_res.best_offer_price = Some(best_ask_price);

    ob_res.ask_book = ask_book;
    ob_res.bid_book = bid_book;

    for o in  &ob_res.bid_book.price_levels[1] {
        let _s_t = o.send_time;
    }

    ob_res
}

fn next_snap(snap: Snap, offsets: (Result<Offset, &str>, Result<Offset, &str>)) -> OrderBook {
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
        (None, None) => {
            place_order_from_snap(snap, &mut ob);
        }
        (_, _) => unreachable!(),
    }
    ob
}

impl OrderBook {
    pub fn process(&self, snap: Snap, ids: (u64, u64)) -> Self {
        let buy_offset = self.get_offset(ids.0);
        let sell_offset = self.get_offset(ids.1);
        dbgp!("OFFSET {:?}", (buy_offset, sell_offset));
        next_snap(snap, (buy_offset, sell_offset))
    }

    pub unsafe fn process_fp(& mut self, snap: Snap) -> OrderBook {
        dbgp!("snap::order_book::process {:?}", 100);
        next_snap_fp(snap, self)
    }
}

#[cfg(test)]
mod tests {

    // use crate::{next_snap, OrderBook, Side};
    use super::*;
    use crate::orderbook::Side;
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
        ob = ob.process(snap, (0, 0));
        assert_eq!(ob.get_bbo().unwrap(), (99, 101, 2));
    }
}
