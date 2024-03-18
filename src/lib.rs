// TODO: get rid of price map
// store total qty on level
// Problem: VecDeque is not contigous?
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::collections::{BTreeMap, HashMap, VecDeque};

#[macro_export]
macro_rules! dbgp {
    ($($arg:tt)*) => (#[cfg(debug_assertions)] println!($($arg)*));
}
#[derive(Debug)]
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
pub struct FillResult {
    // Orders filled (qty, price)
    pub filled_orders: Vec<(u64, u64)>,
    pub remaining_qty: u64,
    pub status: OrderStatus,
}

impl FillResult {
    fn new() -> Self {
        FillResult {
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

#[derive(Debug)]
pub struct Order {
    pub order_id: u64,
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
            currdeque.retain(|x| x.order_id != order_id);
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
            order_id,
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
            "    Updating bbo {:?} {:?}",
            self.best_bid_price,
            self.best_offer_price
        );
    }

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
                    dbgp!("    Incomplete Fill");
                    *incoming_order_qty -= o.qty;
                    done_qty += o.qty;
                    incomplete_fills += 1;
                } else {
                    dbgp!("    Complete Fill");
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
            order_loc.remove(&pop.unwrap().order_id);
        }
        if front_dec > 0 {
            price_level.front_mut().unwrap().qty -= front_dec;
        };
        done_qty
    }
    pub fn add_limit_order(
        &mut self,
        side: Side,
        price: u64,
        order_qty: u64,
        order_id: Option<u64>,
    ) -> FillResult {
        let mut remaining_order_qty = order_qty;
        dbgp!("Got {:?} {}@{}", side, remaining_order_qty, price);
        let mut fill_result = FillResult::new();
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
                            dbgp!("    Matched {}@{}", matched_qty, x);
                            fill_result.filled_orders.push((matched_qty, *x));
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
                            dbgp!("    Matched {}@{}", matched_qty, x);
                            fill_result.filled_orders.push((matched_qty, *x));
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
        fill_result.remaining_qty = remaining_order_qty;
        if remaining_order_qty != 0 {
            dbgp!("    Remaining {}@{}", remaining_order_qty, price);
            if remaining_order_qty == order_qty {
                fill_result.status = OrderStatus::Created;
            } else {
                fill_result.status = OrderStatus::PartiallyFilled;
            }
            self.create_new_limit_order(side, price, remaining_order_qty, order_id);
        } else {
            fill_result.status = OrderStatus::Filled;
        }
        self.update_bbo();

        fill_result
    }
    pub fn get_bbo(&self) -> Result<(u64, u64, u64), &str> {
        let result = match (self.best_bid_price, self.best_offer_price) {
            (None, None) => Err("Both bid and offer HalfBooks are empty"),
            (Some(_), None) => Err("Offer HalfBook is empty"),
            (None, Some(_)) => Err("Bid HalfBook is empty"),
            (Some(_), Some(_)) => {
                let total_bid_qty = self.bid_book.get_total_qty(self.best_bid_price.unwrap());
                let total_ask_qty = self.ask_book.get_total_qty(self.best_offer_price.unwrap());
                dbgp!("---------------");
                dbgp!(
                    "Best bid {:?}, qty {}",
                    self.best_bid_price.unwrap(),
                    total_bid_qty
                );
                dbgp!(
                    "Best ask {:?}, qty {}",
                    self.best_offer_price.unwrap(),
                    total_ask_qty
                );
                let spread = self.best_offer_price.unwrap() - self.best_bid_price.unwrap();
                dbgp!("Spread is {:.6},", spread);
                dbgp!("---------------");
                Ok((
                    self.best_bid_price.unwrap(),
                    self.best_offer_price.unwrap(),
                    spread,
                ))
            }
        };
        result
    }
}
