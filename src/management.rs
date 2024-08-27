//! TOBE design
//! MD -> Indicator -> caclulate order ----> oms-send -> ob-apply order  
//!                                     ^
//!                                     |
//!                               Active Orders   
use crate::{
    account::TradingAccount,
    dbgp,
    orderbook::{ExecutionReport, Order, OrderBook, Side},
    strategy::Strategy,
};

pub struct OrderManagementSystem<'a> {
    pub strategy: &'a mut Strategy,
    pub account: TradingAccount,
    pub active_buy_order: Option<Order>,
    pub active_sell_order: Option<Order>,
    pub strategy_buy_signal: Option<Order>,
    pub strategy_sell_signal: Option<Order>,
}

impl<'a, 'b> OrderManagementSystem<'b> {
    pub fn new(strategy: &'b mut Strategy, account: TradingAccount) -> Self {
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
            .floor() as u32;
        let free_qty = if self.strategy.buy_position_limit - self.strategy.master_position > 0 {
            (self.strategy.buy_position_limit - self.strategy.master_position) as u32
        } else {
            0
        };
        let qty = self.strategy.qty.min(free_qty);
        // dbgp!(
        //     "free_qty = {}, strategy_qty = {}, qty = {}",
        //     free_qty,
        //     self.strategy.qty,
        //     qty
        // );
        if qty > 0 {
            let order = Order {
                id,
                side,
                price,
                qty,
                is_synth: false,
                send_time: 0,
                fill_time: 0,
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
            .ceil() as u32;
        let free_qty = if -self.strategy.sell_position_limit + self.strategy.master_position > 0 {
            (-self.strategy.sell_position_limit + self.strategy.master_position) as u32
        } else {
            0
        };
        let qty = self.strategy.qty.min(free_qty);
        // dbgp!(
        //     "free_qty = {}, strategy_qty = {}, qty = {}",
        //     free_qty,
        //     self.strategy.qty,
        //     qty
        // );
        if qty > 0 {
            let order = Order {
                id,
                side,
                price,
                qty,
                is_synth: false,
                send_time: 0,
                fill_time: 0,
            };
            Ok(order)
        } else {
            Err("No Limit left")
        }
    }

    pub fn update(&mut self, exec_report: ExecutionReport, ids: (u64, u64)) {
        if exec_report.taker_side == Side::Ask {
            if let Some(key) = exec_report.filled_orders.iter().position(|&o| o.0 == ids.0) {
                let trader_filled_qty = exec_report.filled_orders[key].1;
                let trader_filled_price = exec_report.filled_orders[key].2;
                dbgp!(
                    "[TRADE ] qty = {:?}, price = {:?}",
                    trader_filled_qty,
                    trader_filled_price,
                );
                self.strategy.master_position += trader_filled_qty as i32;
                self.account.balance -= (trader_filled_qty * trader_filled_price) as i32;
                dbgp!("TRADER FILLED: {}", trader_filled_qty);
                if trader_filled_qty == self.active_buy_order.unwrap().qty {
                    self.active_buy_order = None;
                } else {
                    self.active_buy_order.unwrap().qty -= trader_filled_qty;
                }
                if let Some(active_buy) = self.active_buy_order {
                    if trader_filled_qty == active_buy.qty {
                        self.active_buy_order = None;
                    } else {
                        let qty = self.active_buy_order.unwrap().qty;
                        // dbgp!("BEFORE FILLED: {:?}", self.active_buy_order);
                        self.active_buy_order = Some(Order {
                            id: ids.0,
                            side: Side::Bid,
                            price: trader_filled_price,
                            qty: qty - trader_filled_qty,
                            is_synth: false,
                            send_time: 0,
                            fill_time: 0,
                        });
                        // dbgp!("AFTER FILLED: {:?}", self.active_buy_order);
                    }
                }
            }
        } else if let Some(key) = exec_report.filled_orders.iter().position(|&o| o.0 == ids.1) {
            let trader_filled_qty = exec_report.filled_orders[key].1;
            let trader_filled_price = exec_report.filled_orders[key].2;
            dbgp!(
                "[TRADE ] qty = {:?}, price = {:?}",
                trader_filled_qty,
                trader_filled_price,
            );
            self.strategy.master_position -= trader_filled_qty as i32;
            self.account.balance += (trader_filled_qty * trader_filled_price) as i32;
            dbgp!("TRADER FILLED: {}", trader_filled_qty);
            if let Some(active_sell) = self.active_sell_order {
                if trader_filled_qty == active_sell.qty {
                    self.active_sell_order = None;
                } else {
                    let qty = self.active_sell_order.unwrap().qty;
                    // dbgp!("BEFORE FILLED: {:?}", self.active_sell_order);
                    self.active_sell_order = Some(Order {
                        id: ids.1,
                        side: Side::Ask,
                        price: trader_filled_price,
                        qty: qty - trader_filled_qty,
                        is_synth: false,
                        send_time: 0,
                        fill_time: 0,
                    });
                    // dbgp!("AFTER FILLED: {:?}", self.active_sell_order);
                }
            }
            // dbgp!(
            //     "Active Orders {:?}, {:?}",
            //     self.active_buy_order,
            //     self.active_sell_order
            // );
        };
        // std::mem::swap(&mut self.strategy.master_position, &mut new_position);
    }
    fn send_buy_order(&mut self, ob: &mut OrderBook) {
        // if self.active_buy_order.is_none() {
        let _ = ob.cancel_order(333);
        let _exec_report = ob.add_limit_order(self.strategy_buy_signal.unwrap());
        // dbgp!("New buy order {:?}", exec_report);
        // } else {
        //     let _exec_report = ob.replace_limit_order(333, self.strategy_buy_signal.unwrap());
        //     dbgp!("Amend buy order {:?}", _exec_report);
        // }
        // if _exec_report.status != OrderStatus::Created {
        //     unreachable!();
        // }
        self.active_buy_order = self.strategy_buy_signal;
    }
    fn send_sell_order(&mut self, ob: &mut OrderBook) {
        // if self.active_sell_order.is_none() {
        let _ = ob.cancel_order(777);
        let _exec_report = ob.add_limit_order(self.strategy_sell_signal.unwrap());
        // dbgp!("New buy order {:?}", exec_report);
        // } else {
        // let _exec_report = ob.replace_limit_order(777, self.strategy_sell_signal.unwrap());
        // dbgp!("Amend buy order {:?}", _exec_report);
        // }
        // if _exec_report.status != OrderStatus::Created {
        //     unreachable!();
        // }
        self.active_sell_order = self.strategy_sell_signal;
    }

    pub fn send_orders(&mut self, ob: &mut OrderBook, m: Option<f32>) {
        let trader_buy_id = 333;
        let trader_sell_id = 777;
        let mut send_buy_order = false;
        let mut send_sell_order = false;
        // dbgp!(
        //     "Active Orders {:?}, {:?}",
        //     self.active_buy_order,
        //     self.active_sell_order
        // );
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
                    qty: _qty,
                    ..
                }) if price == buy_order.price => {
                    dbgp!("[ STRAT] Order found, passing");
                    dbgp!("[ STRAT] price = {}", price);
                    self.strategy_buy_signal = Some(buy_order);
                }
                Some(Order {
                    id: _id,
                    side: Side::Bid,
                    price: _price,
                    qty: _qty,
                    ..
                }) => {
                    dbgp!("[ STRAT] Order found, need amend");
                    dbgp!(
                        "[ STRAT] Old price {}, New Price {}",
                        _price,
                        buy_order.price
                    );
                    dbgp!("[ STRAT] Old qty {}, New qty {}", _qty, buy_order.qty);
                    dbgp!("[ STRAT] send {:#?}", buy_order);
                    self.strategy_buy_signal = Some(buy_order);
                    send_buy_order = true;
                }
                Some(Order {
                    id: _id,
                    side: Side::Ask,
                    price: _price,
                    qty: _qty,
                    ..
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
                    qty: _qty, ..
                    // }) if price == sell_order.price && qty == sell_order.qty => {
                }) if price == sell_order.price => {
                    dbgp!("[ STRAT] Order found, passing");
                    dbgp!("[ STRAT] price = {}", price);
                    self.strategy_sell_signal = Some(sell_order);
                }
                Some(Order {
                    id: _id,
                    side: Side::Ask,
                    price: _price,
                    qty: _qty, ..
                }) => {
                    dbgp!("[ STRAT] Order found, need amend");
                    dbgp!(
                        "[ STRAT] Old price {}, New Price {}",
                        _price,
                        sell_order.price
                    );
                    dbgp!("[ STRAT] Old qty {}, New qty {}", _qty, sell_order.qty);
                    dbgp!("[ STRAT] send {:#?}", sell_order);
                    self.strategy_sell_signal = Some(sell_order);
                    send_sell_order = true;
                }
                Some(Order {
                    id: _id,
                    side: Side::Bid,
                    price: _price,
                    qty: _qty, ..
                }) => unreachable!(),
            }
        } else {
            let _ = ob.cancel_order(777);
            self.strategy_sell_signal = None;
        }
        match (send_buy_order, send_sell_order) {
            (true, true) => match self.active_sell_order {
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
            },
            (true, false) => {
                self.send_buy_order(ob);
            }
            (false, true) => {
                self.send_sell_order(ob);
            }
            (false, false) => {}
        }
    }

    pub fn calculate_buy_order_fp(
        &'a self,
        ref_price: Option<f32>,
        id: u64,
        send_time: u64,
        ind: u8,
    ) -> Result<Order, &'b str> {
        let side = Side::Bid;
        let price = (ref_price.ok_or("Missing Ref Price")?
            * (1.0 + (ind as f32) * self.strategy.buy_criterion))
            .floor() as u32;
        let free_qty = if self.strategy.buy_position_limit - self.strategy.master_position > 0 {
            (self.strategy.buy_position_limit - self.strategy.master_position) as u32
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
                is_synth: true,
                send_time,
                fill_time: 0,
            };
            Ok(order)
        } else {
            Err("No Limit left")
        }
    }

    pub fn calculate_sell_order_fp(
        &'a self,
        ref_price: Option<f32>,
        id: u64,
        send_time: u64,
        ind: u8,
    ) -> Result<Order, &'b str> {
        let side = Side::Ask;
        let price = (ref_price.ok_or("Missing Ref Price")?
            * (1.0 + (ind as f32) * self.strategy.sell_criterion))
            .ceil() as u32;
        let free_qty = if -self.strategy.sell_position_limit + self.strategy.master_position > 0 {
            (-self.strategy.sell_position_limit + self.strategy.master_position) as u32
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
                is_synth: true,
                send_time,
                fill_time: 0,
            };
            Ok(order)
        } else {
            Err("No Limit left")
        }
    }

    fn send_buy_order_fp(&mut self, ob: &mut OrderBook, order: Order) {
        ob.add_limit_order(order);
    }
    fn send_sell_order_fp(&mut self, ob: &mut OrderBook, order: Order) {
        ob.add_limit_order(order);
    }

    pub fn send_orders_fp(
        &mut self,
        ob: &mut OrderBook,
        m: Option<f32>,
        send_time: u64,
        ladder_num: u8,
    ) {
        let trader_buy_id = 333;
        let trader_sell_id = 777;

        for i in 1..ladder_num {
            let buy_order = self.calculate_buy_order_fp(m, trader_buy_id, send_time, i);
            let sell_order = self.calculate_sell_order_fp(m, trader_sell_id, send_time, i);

            self.send_buy_order_fp(ob, buy_order.unwrap());
            self.send_sell_order_fp(ob, sell_order.unwrap());
        }
    }
}
