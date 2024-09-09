#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
use crate::{
    account::TradingAccount,
    backtest::Strategy,
    dbgp,
    orderbook::{ExecutionReport, Order, OrderBook, OrderStatus, Side},
};
use std::collections::HashMap;

pub struct OrderManagementSystem<'a> {
    pub strategy: &'a mut Strategy,
    pub account: TradingAccount,
    pub active_buy_order: Option<Order>,
    pub active_sell_order: Option<Order>,
    pub strategy_buy_signal: Option<Order>,
    pub strategy_sell_signal: Option<Order>,
    pub is_fp_tracking: bool,
    pub filled_times_bid: HashMap<u64, (bool, u64)>,
    pub filled_times_ask: HashMap<u64, (bool, u64)>,
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
            is_fp_tracking: false,
            filled_times_ask: HashMap::new(),
            filled_times_bid: HashMap::new(),
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if either `Indicator` fails to provide reference price
    /// or `Strategy` has no limit left for this side
    pub fn calculate_buy_order(
        &'a self,
        ref_price: Option<f32>,
        id: u64,
        epoch: u64,
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
                ts_create: epoch,
            };
            Ok(order)
        } else {
            Err("No Limit left")
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if either `Indicator` fails to provide reference price
    /// or `Strategy` has no limit left for this side
    pub fn calculate_sell_order(
        &'a self,
        ref_price: Option<f32>,
        id: u64,
        epoch: u64,
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
                ts_create: epoch,
            };
            Ok(order)
        } else {
            Err("No Limit left")
        }
    }

    pub fn update(&mut self, exec_report: &ExecutionReport) {
        if let Some(order) = self.active_buy_order {
            if exec_report.taker_side == Side::Ask {
                if let Some(key) = exec_report
                    .filled_orders
                    .iter()
                    .position(|&o| o.0 == order.id)
                {
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
                    if let Some(active_buy) = self.active_buy_order {
                        if trader_filled_qty == active_buy.qty {
                            self.active_buy_order = None;
                        } else {
                            let qty = order.qty;
                            // dbgp!("BEFORE FILLED: {:?}", self.active_buy_order);
                            let ts_create = active_buy.ts_create;
                            self.active_buy_order = Some(Order {
                                id: order.id,
                                side: Side::Bid,
                                price: trader_filled_price,
                                qty: qty - trader_filled_qty,
                                ts_create
                            });
                            // dbgp!("AFTER FILLED: {:?}", self.active_buy_order);
                        }
                    }
                }
            }
        }
        if let Some(order) = self.active_sell_order {
            if let Some(key) = exec_report
                .filled_orders
                .iter()
                .position(|&o| o.0 == order.id)
            {
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

                        let qty = order.qty;
                        // dbgp!("BEFORE FILLED: {:?}", self.active_sell_order);
                        let ts_create = active_sell.ts_create;
                        self.active_sell_order = Some(Order {
                            id: order.id,
                            side: Side::Ask,
                            price: trader_filled_price,
                            qty: qty - trader_filled_qty,
                            ts_create,
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
    }

    fn send_buy_order(&mut self, ob: &mut OrderBook, epoch: u64) {
        let exec_report;
        if let Some(order) = self.active_buy_order {
            dbgp!("{} {:?}", order.id, ob.get_order(order.id));
            exec_report = ob
                .amend_limit_order(order.id, self.strategy_buy_signal.unwrap(), epoch)
                .unwrap();
            dbgp!("Amend buy order {:?}", exec_report);
        } else {
            exec_report = ob.add_limit_order(self.strategy_buy_signal.unwrap(), epoch);
            dbgp!("New buy order {:?}", exec_report);
        }
        if exec_report.status == OrderStatus::Created {
            self.active_buy_order = self.strategy_buy_signal;
        }
    }

    fn send_sell_order(&mut self, ob: &mut OrderBook, epoch: u64) {
        let exec_report;
        if let Some(order) = self.active_sell_order {
            exec_report = ob
                .amend_limit_order(order.id, self.strategy_sell_signal.unwrap(), epoch)
                .unwrap();
            dbgp!("Amend buy order {:?}", exec_report);
        } else {
            exec_report = ob.add_limit_order(self.strategy_sell_signal.unwrap(), epoch);
            dbgp!("New buy order {:?}", exec_report);
        }
        if exec_report.status == OrderStatus::Created {
            self.active_sell_order = self.strategy_sell_signal;
        }
    }

    /// # Panics
    ///
    /// Will panick
    pub fn send_orders(
        &mut self,
        ob: &mut OrderBook,
        m: Option<f32>,
        trader_buy_id: u64,
        trader_sell_id: u64,
        epoch: u64
    ) {
        let mut send_buy_order = false;
        let mut send_sell_order = false;
        // dbgp!(
        //     "Active Orders {:?}, {:?}",
        //     self.active_buy_order,
        //     self.active_sell_order
        // );
        if let Ok(mut buy_order) = self.calculate_buy_order(m, trader_buy_id, epoch) {
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
                    ts_create: _ts_create,
                }) if price == buy_order.price => {
                    if !self.is_fp_tracking || (epoch - _ts_create) < 10_000_000_000 {
                        dbgp!("[ STRAT] Order found, passing");
                        dbgp!("[ STRAT] price = {}", price);
                        self.strategy_buy_signal = Some(buy_order);
                    } else {
                        dbgp!("[ STRAT] Order is stale, place new order");
                        dbgp!("[ STRAT] send {:#?}", buy_order);
                        self.strategy_buy_signal = Some(buy_order);
                        send_buy_order = true;
                    }
                }
                Some(Order {
                    id: _id,
                    side: Side::Bid,
                    price: _price,
                    qty: _qty,
                    ts_create: _ts_create,
                }) => {
                    if !self.is_fp_tracking || (epoch - _ts_create) < 10_000_000_000 {
                        dbgp!("[ STRAT] Order found, need amend");
                        dbgp!(
                            "[ STRAT] Old price {}, New Price {}",
                            _price,
                            buy_order.price
                        );

                        if self.is_fp_tracking {
                            buy_order.qty = _qty;
                        }

                        dbgp!("[ STRAT] Old qty {}, New qty {}", _qty, buy_order.qty);
                    } else {
                        dbgp!("[ STRAT] Order is stale, place new order");
                    }
                    dbgp!("[ STRAT] send {:#?}", buy_order);
                    self.strategy_buy_signal = Some(buy_order);
                    send_buy_order = true;
                }
                Some(Order {
                    id: _id,
                    side: Side::Ask,
                    price: _price,
                    qty: _qty,
                    ts_create: _ts_create,
                }) => unreachable!(),
            }
        }

        if let Ok(mut sell_order) = self.calculate_sell_order(m, trader_sell_id, epoch) {
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
                    qty: _qty,
                    ts_create: _ts_create,
                    // }) if price == sell_order.price && qty == sell_order.qty => {
                }) if price == sell_order.price => {
                    if !self.is_fp_tracking || (epoch - _ts_create) < 10_000_000_000 {
                        dbgp!("[ STRAT] Order found, passing");
                        dbgp!("[ STRAT] price = {}", price);
                        self.strategy_sell_signal = Some(sell_order);
                    } else {
                        dbgp!("[ STRAT] Order is stale, place new order");
                        dbgp!("[ STRAT] send {:#?}", sell_order);
                        self.strategy_sell_signal = Some(sell_order);
                        send_sell_order = true;
                    }
                }
                Some(Order {
                    id: _id,
                    side: Side::Ask,
                    price: _price,
                    qty: _qty,
                    ts_create: _ts_create,
                }) => {
                    if !self.is_fp_tracking || (epoch - _ts_create) < 10_000_000_000 {
                        dbgp!("[ STRAT] Order found, need amend");
                        dbgp!(
                            "[ STRAT] Old price {}, New Price {}",
                            _price,
                            sell_order.price
                        );

                        if self.is_fp_tracking {
                            sell_order.qty = _qty;
                        }

                        dbgp!("[ STRAT] Old qty {}, New qty {}", _qty, sell_order.qty);
                    } else {
                        dbgp!("[ STRAT] Order is stale, place new order");
                    }
                    dbgp!("[ STRAT] send {:#?}", sell_order);
                    self.strategy_sell_signal = Some(sell_order);
                    send_sell_order = true;
                }
                Some(Order {
                    id: _id,
                    side: Side::Bid,
                    price: _price,
                    qty: _qty,
                    ts_create: _ts_create,
                }) => unreachable!(),
            }
        }
        match (send_buy_order, send_sell_order) {
            (true, true) => {
                if let Some(active_sell) = self.active_sell_order {
                    if self.strategy_buy_signal.unwrap().price < active_sell.price {
                        self.send_buy_order(ob, epoch);
                        self.send_sell_order(ob, epoch);
                    } else {
                        self.send_sell_order(ob, epoch);
                        self.send_buy_order(ob, epoch);
                    }

                } else {
                    self.send_buy_order(ob,epoch);
                    self.send_sell_order(ob, epoch);
                }
            }
            (true, false) => {
                self.send_buy_order(ob, epoch);
            }
            (false, true) => {
                self.send_sell_order(ob, epoch);
            }
            (false, false) => {}
        }
    }

    pub fn get_order_id(&self, side: Side) -> Option<u64> {
        match side {
            Side::Bid => self.active_buy_order.map(|order| order.id),
            Side::Ask => self.active_sell_order.map(|order| order.id),
        }
    }
}
