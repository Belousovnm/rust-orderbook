use crate::account::TradingAccount;
use crate::dbgp;
use crate::{
    orderbook::{ExecutionReport, Order, OrderBook, Side},
    strategy::Strategy,
};

pub struct OrderManagementSystem {
    pub strategy: Strategy,
    pub account: TradingAccount,
    pub active_buy_order: Option<Order>,
    pub active_sell_order: Option<Order>,
    pub strategy_buy_signal: Option<Order>,
    pub strategy_sell_signal: Option<Order>,
}

impl<'a, 'b> OrderManagementSystem {
    pub fn new(strategy: Strategy, account: TradingAccount) -> Self {
        Self {
            strategy,
            account,
            active_buy_order: None,
            active_sell_order: None,
            strategy_buy_signal: None,
            strategy_sell_signal: None,
        }
    }

    pub fn calculate_buy_order(
        &'a self,
        ref_price: Option<f32>,
        id: u64,
    ) -> Result<Order, &'b str> {
        let side = Side::Bid;
        let price = (ref_price.ok_or("Missing Ref Price")? * (1.0 + self.strategy.buy_criterion))
            .floor() as u64;
        let free_qty = if self.strategy.buy_position_limit - self.strategy.master_position > 0 {
            (self.strategy.buy_position_limit - self.strategy.master_position) as u64
        } else {
            0
        };
        let qty = self.strategy.qty.min(free_qty);
        dbgp!(
            "free_qty = {}, strategy_qty = {}, qty = {}",
            free_qty,
            self.strategy.qty,
            qty
        );
        if qty > 0 {
            let order = Order {
                id,
                side,
                price,
                qty,
            };
            Ok(order)
        } else {
            Err("No Limit left")
        }
    }

    pub fn calculate_sell_order(
        &'a self,
        ref_price: Option<f32>,
        id: u64,
    ) -> Result<Order, &'b str> {
        let side = Side::Ask;
        let price = (ref_price.ok_or("Missing Ref Price")? * (1.0 + self.strategy.sell_criterion))
            .ceil() as u64;
        let free_qty = if -self.strategy.sell_position_limit + self.strategy.master_position > 0 {
            (-self.strategy.sell_position_limit + self.strategy.master_position) as u64
        } else {
            0
        };
        let qty = self.strategy.qty.min(free_qty);
        dbgp!(
            "free_qty = {}, strategy_qty = {}, qty = {}",
            free_qty,
            self.strategy.qty,
            qty
        );
        if qty > 0 {
            let order = Order {
                id,
                side,
                price,
                qty,
            };
            Ok(order)
        } else {
            Err("No Limit left")
        }
    }

    pub fn update(&mut self, exec_report: ExecutionReport, ids: (u64, u64)) {
        let sign;
        let trader_id;
        if exec_report.taker_side == Side::Ask {
            sign = 1;
            trader_id = ids.0;
        } else {
            sign = -1;
            trader_id = ids.1;
        };
        // if trader was filled
        if let Some(key) = exec_report
            .filled_orders
            .iter()
            .position(|&o| o.0 == trader_id)
        {
            let trader_filled_qty = exec_report.filled_orders[key].1;
            let trader_filled_price = exec_report.filled_orders[key].2;
            dbgp!(
                "[TRADE ] qty = {:?}, price = {:?}",
                trader_filled_qty,
                trader_filled_price,
            );
            self.strategy.master_position += sign * trader_filled_qty as i32;
            self.account.balance -= sign * (trader_filled_qty * trader_filled_price) as i32;
            dbgp!("TRADER FILLED: {}", trader_filled_qty);
            // std::mem::swap(&mut self.strategy.master_position, &mut new_position);
        }
    }
    fn send_buy_order(&mut self, ob: &mut OrderBook) {
        let _ = ob.cancel_order(333);
        let _ = ob.add_limit_order(self.strategy_buy_signal.unwrap());
        self.active_buy_order = self.strategy_buy_signal;
    }
    fn send_sell_order(&mut self, ob: &mut OrderBook) {
        let _ = ob.cancel_order(777);
        let _ = ob.add_limit_order(self.strategy_sell_signal.unwrap());
        self.active_sell_order = self.strategy_sell_signal;
    }

    pub fn send_orders(&mut self, ob: &mut OrderBook, m: Option<f32>) {
        let trader_buy_id = 333;
        let trader_sell_id = 777;
        let mut send_buy_order = false;
        let mut send_sell_order = false;
        if let Ok(buy_order) = self.calculate_buy_order(m, trader_buy_id) {
            match self.active_buy_order {
                None => {
                    dbgp!("[ STRAT] Order not found, place new order");
                    dbgp!("[ STRAT] send {:#?}", buy_order);
                    self.strategy_buy_signal = Some(buy_order);
                    send_buy_order = true;
                }
                Some(Order {
                    id: _id,
                    side: Side::Bid,
                    price,
                    qty,
                }) if price == buy_order.price && qty == buy_order.qty => {
                    dbgp!("[ STRAT] Order found, passing");
                    dbgp!("[ STRAT] price = {}", price);
                }
                Some(Order {
                    id: _id,
                    side: Side::Bid,
                    price,
                    qty,
                }) => {
                    dbgp!("[ STRAT] Order found, need replace");
                    dbgp!(
                        "[ STRAT] Old price {}, New Price {}",
                        price,
                        buy_order.price
                    );
                    dbgp!("[ STRAT] Old qty {}, New qty {}", qty, buy_order.qty);
                    dbgp!("[ STRAT] send {:#?}", buy_order);
                    self.strategy_buy_signal = Some(buy_order);
                    send_buy_order = true;
                }
                Some(Order {
                    id: _id,
                    side: Side::Ask,
                    price: _price,
                    qty: _qty,
                }) => unreachable!(),
            }
        } else {
            let _ = ob.cancel_order(333);
            self.strategy_sell_signal = None;
        }

        if let Ok(sell_order) = self.calculate_sell_order(m, trader_sell_id) {
            match self.active_sell_order {
                None => {
                    dbgp!("[ STRAT] Order not found, place new order");
                    dbgp!("[ STRAT] send {:#?}", sell_order);
                    self.strategy_sell_signal = Some(sell_order);
                    send_sell_order = true;
                }
                Some(Order {
                    id: _id,
                    side: Side::Ask,
                    price,
                    qty,
                    // }) if price == sell_order.price && qty == sell_order.qty => {
                }) if price == sell_order.price => {
                    dbgp!("[ STRAT] Order found, passing");
                    dbgp!("[ STRAT] price = {}", price);
                }
                Some(Order {
                    id: _id,
                    side: Side::Ask,
                    price,
                    qty,
                }) => {
                    dbgp!("[ STRAT] Order found, need replace");
                    dbgp!(
                        "[ STRAT] Old price {}, New Price {}",
                        price,
                        sell_order.price
                    );
                    dbgp!("[ STRAT] Old qty {}, New qty {}", qty, sell_order.qty);
                    dbgp!("[ STRAT] send {:#?}", sell_order);
                    self.strategy_sell_signal = Some(sell_order);
                    send_sell_order = true;
                }
                Some(Order {
                    id: _id,
                    side: Side::Bid,
                    price: _price,
                    qty: _qty,
                }) => unreachable!(),
            }
        } else {
            let _ = ob.cancel_order(777);
            self.strategy_sell_signal = None;
        }
        if send_buy_order && send_sell_order {
            match self.active_sell_order {
                Some(active_sell) => {
                    if self.strategy_buy_signal.unwrap().price < active_sell.price {
                        self.send_buy_order(ob);
                        self.send_sell_order(ob);
                    } else {
                        self.send_sell_order(ob);
                        self.send_buy_order(ob);
                    }
                }
                None => {
                    self.send_buy_order(ob);
                    self.send_sell_order(ob);
                }
            }
        }
        if send_buy_order {
            self.send_buy_order(ob);
        }
        if send_sell_order {
            self.send_sell_order(ob);
        }
    }
}
