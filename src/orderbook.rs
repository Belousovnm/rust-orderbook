use crate::{
    backtest::TestStrategy,
    dbgp,
    management::OrderManagementSystem,
    snap::{next_snap, Snap},
};
use rand::Rng;
use serde::Serialize;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap, VecDeque},
};

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Default)]
pub enum Side {
    #[default]
    Bid,
    Ask,
}

#[derive(Debug, Eq, PartialEq, Default)]
pub enum OrderStatus {
    #[default]
    Uninitialized,
    Created,
    Filled,
    PartiallyFilled,
    Cancelled,
}

#[derive(Debug, Default)]
pub struct ExecutionReport {
    // Orders filled (id, qty, price)
    pub taker_side: Side,
    pub filled_orders: Vec<(u64, u32, u32)>,
    pub remaining_qty: u32,
    pub status: OrderStatus,
}

impl ExecutionReport {
    pub const fn new() -> Self {
        Self {
            taker_side: Side::Bid,
            filled_orders: Vec::new(),
            remaining_qty: u32::MAX,
            status: OrderStatus::Uninitialized,
        }
    }

    #[allow(dead_code)]
    pub fn avg_fill_price(&self) -> Option<f32> {
        if self.filled_orders.is_empty() {
            return None;
        }
        let mut total_sum_paid = 0;
        let mut total_qty = 0;
        for (_, q, p) in &self.filled_orders {
            total_sum_paid += p * q;
            total_qty += q;
        }
        Some(total_sum_paid as f32 / total_qty as f32)
    }
}

#[derive(Debug, Eq, PartialEq, Default, Clone, Copy)]
pub struct Order {
    pub id: u64,
    pub side: Side,
    pub price: u32,
    pub qty: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HalfBook {
    side: Side,
    price_map: BTreeMap<u32, usize>,
    price_levels: Vec<VecDeque<Order>>,
}

impl HalfBook {
    pub fn new(side: Side) -> Self {
        Self {
            side,
            price_map: BTreeMap::new(),
            price_levels: Vec::with_capacity(16),
        }
    }
    #[allow(unused)]
    pub fn get_total_qty(&self, price: u32) -> u32 {
        self.price_levels[self.price_map[&price]]
            .iter()
            .map(|s| s.qty)
            .sum()
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
#[must_use]
pub struct OrderBook {
    pub best_bid_price: Option<u32>,
    pub best_offer_price: Option<u32>,
    bid_book: HalfBook,
    ask_book: HalfBook,
    // id, (side, price_level, price)
    pub order_loc: HashMap<u64, (Side, usize, u32)>,
}

#[allow(dead_code)]
impl OrderBook {
    pub fn new() -> Self {
        Self {
            best_bid_price: None,
            best_offer_price: None,
            bid_book: HalfBook::new(Side::Bid),
            ask_book: HalfBook::new(Side::Ask),
            order_loc: HashMap::with_capacity(32),
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if `order_id` is not found in `OrderBook`
    ///
    /// # Panics
    ///
    /// Will panic if `OrderBook` state was corrupted
    pub fn cancel_order(&mut self, order_id: u64) -> Result<ExecutionReport, String> {
        if let Some((side, price_level, price)) = self.order_loc.get(&order_id) {
            let book = match side {
                Side::Bid => &mut self.bid_book,
                Side::Ask => &mut self.ask_book,
            };
            let currdeque = book
                .price_levels
                .get_mut(*price_level)
                .expect("price level is missing");
            currdeque.retain(|x| x.id != order_id);
            if currdeque.is_empty() {
                book.price_map.remove(price);
            }
            if self.best_bid_price.is_some_and(|b| b == *price)
                | self.best_offer_price.is_some_and(|a| a == *price)
            {
                self.update_bbo();
            }
            self.order_loc.remove(&order_id);
            Ok(ExecutionReport {
                status: OrderStatus::Cancelled,
                ..Default::default()
            })
        } else {
            Err("No such order id".to_owned())
        }
    }

    fn create_new_limit_order(
        &mut self,
        side: Side,
        price: u32,
        qty: u32,
        order_id: Option<u64>,
    ) -> u64 {
        let mut rng = rand::thread_rng();
        let order_id = order_id.unwrap_or_else(|| rng.gen());
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

    fn match_at_price_level(
        price_level: &mut VecDeque<Order>,
        incoming_order_qty: &mut u32,
        order_loc: &mut HashMap<u64, (Side, usize, u32)>,
    ) -> (Vec<u64>, Vec<u32>) {
        let mut done_qty = Vec::new();
        let mut ids = Vec::new();
        let mut incomplete_fills: usize = 0;
        let mut front_dec = 0;
        for o in price_level.iter() {
            if *incoming_order_qty > 0 {
                match o.qty.cmp(incoming_order_qty) {
                    Ordering::Less => {
                        dbgp!("[ FILL ]    Incomplete {}", o.price);
                        *incoming_order_qty -= o.qty;
                        done_qty.push(o.qty);
                        incomplete_fills += 1;
                    }
                    Ordering::Equal => {
                        dbgp!("[ FILL ]    Complete {}", o.price);
                        done_qty.push(o.qty);
                        incomplete_fills += 1;
                        *incoming_order_qty = 0;
                    }
                    Ordering::Greater => {
                        dbgp!("[ FILL ]    Complete {}", o.price);
                        done_qty.push(*incoming_order_qty);
                        front_dec = *incoming_order_qty;
                        *incoming_order_qty = 0;
                    }
                }
            } else {
                break;
            }
        }
        for _ in 0..incomplete_fills {
            let pop = price_level.pop_front();
            let id = &pop.unwrap().id;
            order_loc.remove(id);
            // dbgp!("MATCHING ENGINE removed order {}", id);
            ids.push(*id);
        }
        if front_dec > 0 {
            let id = price_level.front().unwrap().id;
            price_level.front_mut().unwrap().qty -= front_dec;
            ids.push(id);
        };
        (ids, done_qty)
    }

    pub fn add_limit_order(&mut self, order: Order) -> ExecutionReport {
        let order_qty = order.qty;
        let order_id = order.id;
        let side = order.side;
        let price = order.price;
        let mut remaining_order_qty = order_qty;
        dbgp!(
            "[ INFO ] Booked {:?} {}@{} id={}",
            side,
            remaining_order_qty,
            price,
            order_id,
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
                        let (id_vec, qty_vec) = Self::match_at_price_level(
                            &mut price_levels[curr_level],
                            &mut remaining_order_qty,
                            &mut self.order_loc,
                        );
                        for i in 0..id_vec.len() {
                            dbgp!("[ INFO ]    Matched {}@{} id={}", qty_vec[i], x, id_vec[i]);
                            exec_report.filled_orders.push((id_vec[i], qty_vec[i], *x));
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
                        let (id_vec, qty_vec) = Self::match_at_price_level(
                            &mut price_levels[curr_level],
                            &mut remaining_order_qty,
                            &mut self.order_loc,
                        );
                        for i in 0..id_vec.len() {
                            dbgp!("[ INFO ]    Matched {}@{} {}", qty_vec[i], x, id_vec[i]);
                            exec_report.filled_orders.push((id_vec[i], qty_vec[i], *x));
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
        let status = match remaining_order_qty {
            qty if qty == order_qty => {
                self.create_new_limit_order(side, price, remaining_order_qty, Some(order_id));
                OrderStatus::Created
            }

            qty if qty > 0 => {
                self.create_new_limit_order(side, price, remaining_order_qty, Some(order_id));
                OrderStatus::PartiallyFilled
            }
            0 => OrderStatus::Filled,
            _ => unreachable!(),
        };

        if side == Side::Bid {
            if self.best_bid_price.is_none() | self.best_bid_price.is_some_and(|b| price > b) {
                self.update_bbo();
            }
        } else if side == Side::Ask
            && self.best_offer_price.is_none() | self.best_offer_price.is_some_and(|a| price < a)
        {
            self.update_bbo();
        }
        exec_report.taker_side = side;
        exec_report.status = status;
        exec_report.remaining_qty = remaining_order_qty;
        if order_qty == 0 {
            dbgp!("WTF ORDER QTY 0! IN ORDER {}", order_id);
        }
        exec_report
    }

    /// # Errors
    ///
    /// Will return `Err` if atleast one `HalfBook` in `OrderBook` is empty
    pub fn get_bbo(&self) -> Result<(u32, u32, u32), &str> {
        match (self.best_bid_price, self.best_offer_price) {
            (None, None) => Err("Both bid and offer HalfBooks are empty"),
            (Some(_bid), None) => Err("Offer HalfBook is empty"),
            (None, Some(_ask)) => Err("Bid HalfBook is empty"),
            (Some(bid_price), Some(ask_price)) => {
                dbgp!(
                    "[ BBO  ] {:?}@{} x {:?}@{}",
                    self.bid_book.get_total_qty(bid_price),
                    bid_price,
                    self.ask_book.get_total_qty(ask_price),
                    ask_price,
                );
                let spread = ask_price - bid_price;
                dbgp!("[ BBO  ] Spread is {:.6}", spread);
                Ok((bid_price, ask_price, spread))
            }
        }
    }

    pub fn get_order(&self, order_id: u64) -> Option<&Order> {
        let (side, price_level, _) = self.order_loc.get(&order_id)?;
        let book = match side {
            Side::Bid => &self.bid_book,
            Side::Ask => &self.ask_book,
        };
        // let currdeque = book.price_levels.get(*price_level).unwrap();
        let currdeque = &book.price_levels[*price_level];
        let mut order = currdeque.iter().filter(|o| o.id == order_id);
        order.next()
    }

    /// # Errors
    ///
    /// Will return `Err` if `order_id` is not found in `OrderBook`
    pub fn amend_limit_order(
        &mut self,
        order_id: u64,
        new_order: Order,
    ) -> Result<ExecutionReport, String> {
        self.cancel_order(order_id)?;
        Ok(self.add_limit_order(new_order))
    }
}

impl OrderBook {
    /// # Errors
    ///
    /// Will return `Err` if `order_id` is not found in `OrderBook`
    pub fn get_offset(
        &self,
        oms: &mut OrderManagementSystem<TestStrategy>,
        side: Side,
    ) -> Result<(Side, u32, u32, u32, u32, u64), &str> {
        let order_id = oms.get_order_id(side).ok_or("No such order id")?;
        if let Some((side, price_level, price)) = self.order_loc.get(&order_id) {
            let book = match side {
                Side::Bid => &self.bid_book,
                Side::Ask => &self.ask_book,
            };
            let mut qty_head = 0;
            let mut qty_tail = 0;
            let mut qty = 0;
            let mut order_met = false;
            // let currdeque = book.price_levels.get(*price_level).unwrap();
            let currdeque = &book.price_levels[*price_level];
            for o in currdeque {
                // match o.id == order_id {
                //     false if !order_met => qty_head += o.qty,
                //     true => {
                //         qty = o.qty;
                //         order_met = true;
                //     }
                //     false if order_met => qty_tail += o.qty,
                //     _ => (),
                if o.id == order_id {
                    qty = o.qty;
                    order_met = true;
                } else if o.id != order_id && !order_met {
                    qty_head += o.qty;
                } else if o.id != order_id && order_met {
                    qty_tail += o.qty;
                };
            }
            Ok((*side, *price, qty_head, qty, qty_tail, order_id))
        } else {
            Err("No such order id")
        }
    }
    pub fn get_raw(&self, oms: &OrderManagementSystem<TestStrategy>) -> Self {
        let mut raw_ob = self.clone();
        if let Some(order) = oms.active_buy_order {
            let _ = raw_ob.cancel_order(order.id);
        }
        if let Some(order) = oms.active_sell_order {
            let _ = raw_ob.cancel_order(order.id);
        }
        raw_ob
    }

    pub fn process(&self, snap: Snap, oms: &mut OrderManagementSystem<TestStrategy>) -> Self {
        let buy_offset = self.get_offset(oms, Side::Bid);
        let sell_offset = self.get_offset(oms, Side::Ask);
        dbgp!("OFFSET {:?}", (buy_offset, sell_offset));
        let ob = next_snap(snap, (buy_offset, sell_offset));
        if let Some(id) = oms.get_order_id(Side::Bid) {
            if ob.get_order(id).is_none() {
                oms.active_buy_order = None;
            }
        }
        if let Some(id) = oms.get_order_id(Side::Ask) {
            if ob.get_order(id).is_none() {
                oms.active_sell_order = None;
            }
        }
        ob
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}
