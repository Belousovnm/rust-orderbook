// TODO: pure Array > BTreeMap?
// FEATURE: Replace order type
// store total qty on level
// Problem: VecDeque is not contigous?

pub mod event;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::collections::{BTreeMap, HashMap, VecDeque};

#[macro_export]
macro_rules! dbgp {
    ($($arg:tt)*) => (#[cfg(debug_assertions)] println!($($arg)*));
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Side {
    Bid,
    Ask,
}

impl Distribution<Side> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Side {
        match rng.gen_range(0..=1) {
            0 => Side::Bid,
            1 => Side::Ask,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum OrderStatus {
    Uninitialized,
    Created,
    Filled,
    PartiallyFilled,
}

#[derive(Debug)]
pub struct ExecutionReport {
    // Orders filled (qty, price)
    pub filled_orders: Vec<(u64, u64)>,
    pub remaining_qty: u64,
    pub status: OrderStatus,
}

impl ExecutionReport {
    fn new() -> Self {
        ExecutionReport {
            filled_orders: Vec::new(),
            remaining_qty: u64::MAX,
            status: OrderStatus::Uninitialized,
        }
    }

    pub fn avg_fill_price(&self) -> f32 {
        let mut total_price_paid = 0;
        let mut total_qty = 0;
        for (q, p) in &self.filled_orders {
            total_price_paid += p * q;
            total_qty += q;
        }
        total_price_paid as f32 / total_qty as f32
    }
}

#[derive(Debug, PartialEq)]
pub struct Order {
    pub id: u64,
    pub side: Side,
    pub price: u64,
    pub qty: u64,
}

#[derive(Debug)]
struct HalfBook {
    side: Side,
    price_map: BTreeMap<u64, usize>,
    price_levels: Vec<VecDeque<Order>>,
}

impl HalfBook {
    pub fn new(side: Side) -> Self {
        HalfBook {
            side,
            price_map: BTreeMap::new(),
            price_levels: Vec::with_capacity(50_000),
        }
    }

    pub fn get_total_qty(&self, price: u64) -> u64 {
        self.price_levels[self.price_map[&price]]
            .iter()
            .map(|s| s.qty)
            .sum()
    }
}

#[derive(Debug)]
pub struct OrderBook {
    symbol: String,
    best_bid_price: Option<u64>,
    best_offer_price: Option<u64>,
    bid_book: HalfBook,
    ask_book: HalfBook,
    order_loc: HashMap<u64, (Side, usize, u64)>,
}

impl OrderBook {
    pub fn new(symbol: String) -> Self {
        OrderBook {
            symbol,
            best_bid_price: None,
            best_offer_price: None,
            bid_book: HalfBook::new(Side::Bid),
            ask_book: HalfBook::new(Side::Ask),
            order_loc: HashMap::with_capacity(50_000),
        }
    }

    pub fn cancel_order(&mut self, order_id: u64) -> Result<&str, &str> {
        if let Some((side, price_level, price)) = self.order_loc.get(&order_id) {
            let book = match side {
                Side::Bid => &mut self.bid_book,
                Side::Ask => &mut self.ask_book,
            };
            let currdeque = book.price_levels.get_mut(*price_level).unwrap();
            currdeque.retain(|x| x.id != order_id);
            if currdeque.is_empty() {
                book.price_map.remove(price);
            }
            self.order_loc.remove(&order_id);
            self.update_bbo();
            Ok("Successfully cancelled order")
        } else {
            Err("No such order id")
        }
    }

    #[inline]
    fn create_new_limit_order(
        &mut self,
        side: Side,
        price: u64,
        qty: u64,
        order_id: Option<u64>,
    ) -> u64 {
        let mut rng = rand::thread_rng();
        let order_id = order_id.unwrap_or(rng.gen());
        let book = match side {
            Side::Ask => &mut self.ask_book,
            Side::Bid => &mut self.bid_book,
        };
        let order = Order {
            id: order_id,
            side,
            price,
            qty,
        };

        if let Some(val) = book.price_map.get(&price) {
            // If price level already exists
            // Add order to the back of the queue
            self.order_loc.insert(order_id, (side, *val, price));
            book.price_levels[*val].push_back(order);
        } else {
            let new_loc = book.price_levels.len();
            book.price_map.insert(price, new_loc);
            let mut vec_deq = VecDeque::new();
            vec_deq.push_back(order);
            book.price_levels.push(vec_deq);
            self.order_loc.insert(order_id, (side, new_loc, price));
        }
        order_id
    }

    #[inline]
    fn update_bbo(&mut self) {
        let mut best_bid_price = None;
        for (p, u) in &self.bid_book.price_map {
            if !self.bid_book.price_levels[*u].is_empty() {
                best_bid_price = match best_bid_price {
                    None => Some(*p),
                    Some(v) if v < *p => Some(*p),
                    Some(v) if v >= *p => Some(v),
                    _ => unreachable!(),
                }
            }
        }
        let mut best_offer_price = None;
        for (p, u) in &self.ask_book.price_map {
            if !self.ask_book.price_levels[*u].is_empty() {
                best_offer_price = match best_offer_price {
                    None => Some(*p),
                    Some(v) if v > *p => Some(*p),
                    Some(v) if v <= *p => Some(v),
                    _ => unreachable!(),
                }
            }
        }
        self.best_bid_price = best_bid_price;
        self.best_offer_price = best_offer_price;
        dbgp!(
            "[ INFO ]    Updating bbo {:?} {:?}",
            self.best_bid_price,
            self.best_offer_price
        );
    }

    #[inline]
    fn match_at_price_level(
        price_level: &mut VecDeque<Order>,
        incoming_order_qty: &mut u64,
        order_loc: &mut HashMap<u64, (Side, usize, u64)>,
    ) -> u64 {
        let mut done_qty = 0;
        let mut incomplete_fills: usize = 0;
        let mut front_dec = 0;
        let iter = price_level.iter();
        for o in iter {
            if *incoming_order_qty > 0 {
                if o.qty < *incoming_order_qty {
                    dbgp!("[ FILL ]    Incomplete");
                    *incoming_order_qty -= o.qty;
                    done_qty += o.qty;
                    incomplete_fills += 1;
                } else {
                    dbgp!("[ FILL ]    Complete");
                    done_qty += *incoming_order_qty;
                    front_dec = *incoming_order_qty;
                    *incoming_order_qty = 0;
                }
            } else {
                break;
            }
        }
        for _ in 1..=incomplete_fills {
            let pop = price_level.pop_front();
            order_loc.remove(&pop.unwrap().id);
        }
        if front_dec > 0 {
            price_level.front_mut().unwrap().qty -= front_dec;
        };
        done_qty
    }
    #[inline]
    pub fn add_limit_order(&mut self, order: Order) -> ExecutionReport {
        let order_qty = order.qty;
        let order_id = order.id;
        let side = order.side;
        let price = order.price;
        let mut remaining_order_qty = order_qty;
        dbgp!(
            "[ INFO ] Booked {:?} {}@{}",
            side,
            remaining_order_qty,
            price
        );
        let mut exec_report = ExecutionReport::new();
        match side {
            Side::Bid => {
                let askbook = &mut self.ask_book;
                let price_map = &mut askbook.price_map;
                let price_levels = &mut askbook.price_levels;
                let mut price_map_iter = price_map.iter();

                if let Some((mut x, _)) = price_map_iter.next() {
                    while price >= *x {
                        let curr_level = price_map[x];
                        let matched_qty = Self::match_at_price_level(
                            &mut price_levels[curr_level],
                            &mut remaining_order_qty,
                            &mut self.order_loc,
                        );
                        if matched_qty != 0 {
                            dbgp!("[ INFO ]    Matched {}@{}", matched_qty, x);
                            exec_report.filled_orders.push((matched_qty, *x));
                        }
                        if let Some((a, _)) = price_map_iter.next() {
                            x = a;
                        } else {
                            break;
                        }
                    }
                }
            }
            Side::Ask => {
                let bidbook = &mut self.bid_book;
                let price_map = &mut bidbook.price_map;
                let price_levels = &mut bidbook.price_levels;
                let mut price_map_iter = price_map.iter();

                if let Some((mut x, _)) = price_map_iter.next_back() {
                    while price <= *x {
                        let curr_level = price_map[x];
                        let matched_qty = Self::match_at_price_level(
                            &mut price_levels[curr_level],
                            &mut remaining_order_qty,
                            &mut self.order_loc,
                        );
                        if matched_qty != 0 {
                            dbgp!("[ INFO ]    Matched {}@{}", matched_qty, x);
                            exec_report.filled_orders.push((matched_qty, *x));
                        }
                        if let Some((a, _)) = price_map_iter.next_back() {
                            x = a;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        exec_report.remaining_qty = remaining_order_qty;
        if remaining_order_qty != 0 {
            dbgp!("[ INFO ]    Remaining {}@{}", remaining_order_qty, price);
            if remaining_order_qty == order_qty {
                exec_report.status = OrderStatus::Created;
            } else {
                exec_report.status = OrderStatus::PartiallyFilled;
            }
            self.create_new_limit_order(side, price, remaining_order_qty, Some(order_id));
        } else {
            exec_report.status = OrderStatus::Filled;
        }
        self.update_bbo();

        exec_report
    }
    #[inline]
    pub fn get_bbo(&self) -> Result<(u64, u64, u64), &str> {
        match (self.best_bid_price, self.best_offer_price) {
            (None, None) => Err("Both bid and offer HalfBooks are empty"),
            (Some(_), None) => Err("Offer HalfBook is empty"),
            (None, Some(_)) => Err("Bid HalfBook is empty"),
            (Some(_), Some(_)) => {
                let total_bid_qty = self.bid_book.get_total_qty(self.best_bid_price.unwrap());
                let total_ask_qty = self.ask_book.get_total_qty(self.best_offer_price.unwrap());
                dbgp!(
                    "[ BBO  ] {:?}@{} x {:?}@{}",
                    total_bid_qty,
                    self.best_bid_price.unwrap(),
                    total_ask_qty,
                    self.best_offer_price.unwrap(),
                );
                let spread = self.best_offer_price.unwrap() - self.best_bid_price.unwrap();
                dbgp!("[ BBO  ] Spread is {:.6},", spread);
                Ok((
                    self.best_bid_price.unwrap(),
                    self.best_offer_price.unwrap(),
                    spread,
                ))
            }
        }
    }

    pub fn get_offset(&self, order_id: u64) -> Result<(Side, u64, u64, u64, u64, u64), &str> {
        if let Some((side, price_level, price)) = self.order_loc.get(&order_id) {
            let book = match side {
                Side::Bid => &self.bid_book,
                Side::Ask => &self.ask_book,
            };
            let mut qty_head = 0;
            let mut qty_tail = 0;
            let mut qty = 0;
            let mut order_met = false;
            let currdeque = book.price_levels.get(*price_level).unwrap();
            for o in currdeque.iter() {
                match o.id == order_id {
                    false if !order_met => qty_head += o.qty,
                    true => {
                        qty = o.qty;
                        order_met = true;
                    }
                    false if order_met => qty_tail += o.qty,
                    _ => (),
                };
            }
            Ok((*side, *price, qty_head, qty, qty_tail, order_id))
        } else {
            Err("No such order id")
        }
    }
}

fn place_order_from_snap(snap: Vec<(Side, u64, u64)>, ob: &mut OrderBook) {
    for (id, level) in snap.iter().enumerate() {
        let _ = ob.add_limit_order(Order {
            id: id as u64,
            side: level.0,
            price: level.1,
            qty: level.2,
        });
    }
}

pub fn next_snap(
    snap: Vec<(Side, u64, u64)>,
    ob: &mut OrderBook,
    offset: Result<(Side, u64, u64, u64, u64, u64), &str>,
) {
    match offset.ok() {
        Some((side, price, qty_head, qty, qty_tail, id)) => {
            let mut filtered_snap = Vec::with_capacity(11);
            let mut new_qty = qty_head + qty_tail;
            for level in snap.iter() {
                if level.1 == price {
                    new_qty = level.2;
                } else {
                    filtered_snap.push(*level)
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

#[cfg(test)]
mod tests {

    // use crate::{next_snap, OrderBook, Side};
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_orders_from_snapshot() {
        let snap = vec![(Side::Bid, 99, 1), (Side::Ask, 101, 1)];
        // let offset = Ok((Side::Bid, 101, 0, 1, 0, 999));
        let offset = Err("unittest");
        let mut ob = OrderBook::new("SPB".to_string());
        next_snap(snap, &mut ob, offset);
        assert_eq!(ob.get_bbo().unwrap(), (99, 101, 2));
    }
}
