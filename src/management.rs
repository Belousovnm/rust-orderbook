#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

use crate::{
    account::TradingAccount,
    backtest::{FixPriceStrategy, Strategy, TestStrategy},
    dbgp,
    experiments::Schedule,
    matching_engine::{ExecutionReport, Order, OrderBook, OrderStatus, Side},
};

pub struct OrderManagementSystem<'a, S: Strategy> {
    pub strategy: &'a mut S,
    pub account: TradingAccount,
    pub active_buy_order: Option<Order>,
    pub active_sell_order: Option<Order>,
    pub strategy_buy_signal: Option<Order>,
    pub strategy_sell_signal: Option<Order>,
    pub schedule: Schedule,
}

impl<'a, S: Strategy> OrderManagementSystem<'a, S> {
    pub fn new(strategy: &'a mut S, account: TradingAccount) -> Self {
        Self {
            strategy,
            account,
            active_buy_order: None,
            active_sell_order: None,
            strategy_buy_signal: None,
            strategy_sell_signal: None,
            schedule: Schedule::default(),
        }
    }

    pub fn get_order_id(&self, side: Side) -> Option<u64> {
        match side {
            | Side::Bid => self.active_buy_order.map(|order| order.id),
            | Side::Ask => self.active_sell_order.map(|order| order.id),
        }
    }
}

impl<'a> OrderManagementSystem<'a, TestStrategy> {
    fn send_buy_order(&mut self, ob: &mut OrderBook) {
        let exec_report;
        if let Some(order) = self.active_buy_order {
            dbgp!("{} {:?}", order.id, ob.get_order(order.id));
            exec_report = ob
                .amend_limit_order(order.id, self.strategy_buy_signal.unwrap())
                .unwrap();
            dbgp!("Amend buy order {:?}", exec_report);
        } else {
            exec_report = ob.add_limit_order(self.strategy_buy_signal.unwrap());
            dbgp!("New buy order {:?}", exec_report);
        }
        if exec_report.status == OrderStatus::Created {
            self.active_buy_order = self.strategy_buy_signal;
        } else {
            // Only maker orders allowed
            unreachable!();
        }
    }

    fn send_sell_order(&mut self, ob: &mut OrderBook) {
        let exec_report;
        if let Some(order) = self.active_sell_order {
            exec_report = ob
                .amend_limit_order(order.id, self.strategy_sell_signal.unwrap())
                .unwrap();
            dbgp!("Amend buy order {:?}", exec_report);
        } else {
            exec_report = ob.add_limit_order(self.strategy_sell_signal.unwrap());
            dbgp!("New buy order {:?}", exec_report);
        }
        if exec_report.status == OrderStatus::Created {
            self.active_sell_order = self.strategy_sell_signal;
        } else {
            // Only maker orders allowed
            unreachable!();
        }
    }
    /// # Errors
    ///
    /// Will return `Err` if either `Indicator` fails to provide reference price
    /// or `Strategy` has no limit left for this side
    pub fn calculate_buy_order(&self, ref_price: Option<f32>, id: u64) -> Result<Order, String> {
        let side = Side::Bid;
        let price = (ref_price.ok_or_else(|| "Missing Ref Price".to_owned())?
            * (1.0 + self.strategy.buy_criterion))
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
            };
            Ok(order)
        } else {
            Err("No Limit left".to_owned())
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if either `Indicator` fails to provide reference price
    /// or `Strategy` has no limit left for this side
    pub fn calculate_sell_order(&self, ref_price: Option<f32>, id: u64) -> Result<Order, String> {
        let side = Side::Ask;
        let price = (ref_price.ok_or_else(|| "Missing Ref Price".to_owned())?
            * (1.0 + self.strategy.sell_criterion))
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
            };
            Ok(order)
        } else {
            Err("No Limit left".to_owned())
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
    ) {
        let mut send_buy_order = false;
        let mut send_sell_order = false;
        // dbgp!(
        //     "Active Orders {:?}, {:?}",
        //     self.active_buy_order,
        //     self.active_sell_order
        // );
        if let Ok(buy_order) = self.calculate_buy_order(m, trader_buy_id) {
            match self.active_buy_order {
                | None => {
                    dbgp!("[ STRAT] Order not found, place new order");
                    dbgp!("[ STRAT] send {:#?}", buy_order);
                    self.strategy_buy_signal = Some(buy_order);
                    send_buy_order = true;
                }
                | Some(Order {
                    id: _id,
                    side: Side::Bid,
                    price,
                    qty: _qty,
                }) if price == buy_order.price => {
                    dbgp!("[ STRAT] Order found, passing");
                    dbgp!("[ STRAT] price = {}", price);
                    self.strategy_buy_signal = Some(buy_order);
                }
                | Some(Order {
                    id: _id,
                    side: Side::Bid,
                    price: _price,
                    qty: _qty,
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
                | Some(Order {
                    id: _id,
                    side: Side::Ask,
                    price: _price,
                    qty: _qty,
                }) => unreachable!(),
            }
        } else if let Some(order) = self.active_buy_order {
            let _ = ob.cancel_order(order.id);
        };

        if let Ok(sell_order) = self.calculate_sell_order(m, trader_sell_id) {
            match self.active_sell_order {
                | None => {
                    dbgp!("[ STRAT] Order not found, place new order");
                    dbgp!("[ STRAT] send {:#?}", sell_order);
                    self.strategy_sell_signal = Some(sell_order);
                    send_sell_order = true;
                }
                | Some(Order {
                    id: _id,
                    side: Side::Ask,
                    price,
                    qty: _qty,
                    // }) if price == sell_order.price && qty == sell_order.qty => {
                }) if price == sell_order.price => {
                    dbgp!("[ STRAT] Order found, passing");
                    dbgp!("[ STRAT] price = {}", price);
                    self.strategy_sell_signal = Some(sell_order);
                }
                | Some(Order {
                    id: _id,
                    side: Side::Ask,
                    price: _price,
                    qty: _qty,
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
                | Some(Order {
                    id: _id,
                    side: Side::Bid,
                    price: _price,
                    qty: _qty,
                }) => unreachable!(),
            }
        } else if let Some(order) = self.active_sell_order {
            let _ = ob.cancel_order(order.id);
        };

        match (send_buy_order, send_sell_order) {
            | (true, true) => {
                if let Some(active_sell) = self.active_sell_order {
                    if self.strategy_buy_signal.unwrap().price < active_sell.price {
                        self.send_buy_order(ob);
                        self.send_sell_order(ob);
                    } else {
                        self.send_sell_order(ob);
                        self.send_buy_order(ob);
                    }
                } else {
                    self.send_buy_order(ob);
                    self.send_sell_order(ob);
                }
            }
            | (true, false) => {
                self.send_buy_order(ob);
            }
            | (false, true) => {
                self.send_sell_order(ob);
            }
            | (false, false) => {}
        }
    }
    pub fn update(&mut self, exec_report: &ExecutionReport) {
        if let Some(order) = self.active_buy_order {
            if exec_report.own_side == Side::Ask {
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
                    self.strategy
                        .increment_master_position(trader_filled_qty as i32);
                    self.account.balance -= (trader_filled_qty * trader_filled_price) as i32;
                    dbgp!("TRADER FILLED: {}", trader_filled_qty);
                    if let Some(active_buy) = self.active_buy_order {
                        if trader_filled_qty == active_buy.qty {
                            self.active_buy_order = None;
                        } else {
                            let qty = order.qty;
                            // dbgp!("BEFORE FILLED: {:?}", self.active_buy_order);
                            self.active_buy_order = Some(Order {
                                id: order.id,
                                side: Side::Bid,
                                price: trader_filled_price,
                                qty: qty - trader_filled_qty,
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
                self.strategy
                    .increment_master_position(-(trader_filled_qty as i32));
                self.account.balance += (trader_filled_qty * trader_filled_price) as i32;
                dbgp!("TRADER FILLED: {}", trader_filled_qty);
                if let Some(active_sell) = self.active_sell_order {
                    if trader_filled_qty == active_sell.qty {
                        self.active_sell_order = None;
                    } else {
                        let qty = order.qty;
                        // dbgp!("BEFORE FILLED: {:?}", self.active_sell_order);
                        self.active_sell_order = Some(Order {
                            id: order.id,
                            side: Side::Ask,
                            price: trader_filled_price,
                            qty: qty - trader_filled_qty,
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
}

impl<'a> OrderManagementSystem<'a, FixPriceStrategy> {
    fn send_buy_order(&mut self, ob: &mut OrderBook, epoch: u64) {
        let exec_report;
        if let Some(order) = self.active_buy_order {
            dbgp!("{} {:?}", order.id, ob.get_order(order.id));
            exec_report = ob
                .amend_limit_order(order.id, self.strategy_buy_signal.unwrap())
                .unwrap();
            dbgp!("Amend buy order {:?}", exec_report);
        } else {
            exec_report = ob.add_limit_order(self.strategy_buy_signal.unwrap());
            dbgp!("New buy order {:?}", exec_report);
        }
        if exec_report.status == OrderStatus::Filled {
            println!(
                "[  DB  ];{};{};{};{};",
                exec_report.own_id,
                epoch,
                (epoch + 3 - exec_report.own_id) / 1000,
                0
            );
            self.lock_release();
            self.schedule = Schedule::new();
        } else {
            self.active_buy_order = self.strategy_buy_signal;
        }
    }

    fn send_sell_order(&mut self, ob: &mut OrderBook, epoch: u64) {
        let exec_report;
        if let Some(order) = self.active_sell_order {
            exec_report = ob
                .amend_limit_order(order.id, self.strategy_sell_signal.unwrap())
                .unwrap();
            dbgp!("Amend buy order {:?}", exec_report);
        } else {
            exec_report = ob.add_limit_order(self.strategy_sell_signal.unwrap());
            dbgp!("New buy order {:?}", exec_report);
        }
        if exec_report.status == OrderStatus::Filled {
            println!(
                "[  DB  ];{};{};{};{};",
                exec_report.own_id,
                epoch,
                (epoch + 7 - exec_report.own_id) / 1000,
                0
            );
            self.lock_release();
            self.schedule = Schedule::new();
        } else {
            self.active_sell_order = self.strategy_sell_signal;
        }
    }
    /// # Errors
    ///
    /// Will return `Err` if either reference price
    /// or criterion are None
    pub fn lock_bid_price(&self, bbo: Option<(u32, u32)>) -> Result<u32, String> {
        let bid_price = match self.strategy.buy_price {
            | None => {
                (bbo.ok_or_else(|| "Missing ref price".to_string())?.0 as i32
                    + self
                        .strategy
                        .buy_tick_criterion
                        .ok_or_else(|| "Missing buy criterion".to_string())?) as u32
            }
            | Some(price) => price,
        };
        Ok(bid_price)
    }

    /// # Errors
    ///
    /// Will return `Err` if either reference price
    /// or criterion are None
    pub fn lock_ask_price(&self, bbo: Option<(u32, u32)>) -> Result<u32, String> {
        let ask_price = match self.strategy.sell_price {
            | None => {
                (bbo.ok_or_else(|| "Missing ref price".to_string())?.1 as i32
                    + self
                        .strategy
                        .sell_tick_criterion
                        .ok_or_else(|| "Missing sell criterion".to_string())?)
                    as u32
            }

            | Some(price) => price,
        };
        Ok(ask_price)
    }

    pub fn lock_release(&mut self) {
        self.strategy.buy_price = None;
        self.strategy.sell_price = None;
    }

    /// # Errors
    ///
    /// Will return `Err` if `Indicator` fails to provide reference price
    pub fn calculate_buy_order(&self, id: u64) -> Result<Order, String> {
        let price = self
            .strategy
            .buy_price
            .ok_or_else(|| "Missing Buy Price".to_owned())?;
        let order = Order {
            id,
            side: Side::Bid,
            price,
            qty: self.strategy.qty,
        };
        Ok(order)
    }

    /// # Errors
    ///
    /// Will return `Err` if `Indicator` fails to provide reference price
    pub fn calculate_sell_order(&self, id: u64) -> Result<Order, String> {
        let price = self
            .strategy
            .sell_price
            .ok_or_else(|| "Missing Buy Price".to_owned())?;
        let order = Order {
            id,
            side: Side::Ask,
            price,
            qty: self.strategy.qty,
        };
        Ok(order)
    }

    /// # Panics
    ///
    /// Will panick
    pub fn send_orders(
        &mut self,
        ob: &mut OrderBook,
        epoch: u64,
        trader_buy_id: u64,
        trader_sell_id: u64,
    ) {
        let mut send_buy_order = false;
        let mut send_sell_order = false;
        // dbgp!(
        //     "Active Orders {:?}, {:?}",
        //     self.active_buy_order,
        //     self.active_sell_order
        // );

        if let Ok(buy_order) = self.calculate_buy_order(trader_buy_id) {
            match self.active_buy_order {
                | None => {
                    dbgp!("[ STRAT] Order not found, place new order");
                    dbgp!("[ STRAT] send {:#?}", buy_order);
                    self.strategy_buy_signal = Some(buy_order);
                    send_buy_order = true;
                }
                | Some(Order {
                    id: _id,
                    side: Side::Bid,
                    price,
                    qty: _qty,
                }) if price == buy_order.price => {
                    dbgp!("[ STRAT] Order found, passing");
                    dbgp!("[ STRAT] price = {}", price);
                    self.strategy_buy_signal = Some(buy_order);
                }
                | Some(Order {
                    id: _id,
                    side: Side::Bid,
                    price: _price,
                    qty: _qty,
                }) => {
                    dbgp!("[ STRAT] Order found, need amend");
                    dbgp!(
                        "[ STRAT] Old price {}, New Price {}",
                        _price,
                        buy_order.price
                    );
                    dbgp!("[ STRAT] Old qty {}, New qty {}", _qty, buy_order.qty);
                    dbgp!("[ STRAT] send {:#?}", buy_order);
                    // FixPrice specific
                    unreachable!()
                }
                | Some(Order {
                    id: _id,
                    side: Side::Ask,
                    price: _price,
                    qty: _qty,
                }) => unreachable!(),
            }
        } else if let Some(order) = self.active_buy_order {
            let _ = ob.cancel_order(order.id);
        };

        if let Ok(sell_order) = self.calculate_sell_order(trader_sell_id) {
            match self.active_sell_order {
                | None => {
                    dbgp!("[ STRAT] Order not found, place new order");
                    dbgp!("[ STRAT] send {:#?}", sell_order);
                    self.strategy_sell_signal = Some(sell_order);
                    send_sell_order = true;
                }
                | Some(Order {
                    id: _id,
                    side: Side::Ask,
                    price,
                    qty: _qty,
                    // }) if price == sell_order.price && qty == sell_order.qty => {
                }) if price == sell_order.price => {
                    dbgp!("[ STRAT] Order found, passing");
                    dbgp!("[ STRAT] price = {}", price);
                    self.strategy_sell_signal = Some(sell_order);
                }
                | Some(Order {
                    id: _id,
                    side: Side::Ask,
                    price: _price,
                    qty: _qty,
                }) => {
                    dbgp!("[ STRAT] Order found, need amend");
                    dbgp!(
                        "[ STRAT] Old price {}, New Price {}",
                        _price,
                        sell_order.price
                    );
                    dbgp!("[ STRAT] Old qty {}, New qty {}", _qty, sell_order.qty);
                    dbgp!("[ STRAT] send {:#?}", sell_order);
                    // FixPrice specific
                    unreachable!();
                }
                | Some(Order {
                    id: _id,
                    side: Side::Bid,
                    price: _price,
                    qty: _qty,
                }) => unreachable!(),
            }
        } else if let Some(order) = self.active_sell_order {
            let _ = ob.cancel_order(order.id);
        };
        match (send_buy_order, send_sell_order) {
            | (true, true) => {
                if let Some(active_sell) = self.active_sell_order {
                    if self.strategy_buy_signal.unwrap().price < active_sell.price {
                        self.send_buy_order(ob, epoch);
                        self.send_sell_order(ob, epoch);
                    } else {
                        self.send_sell_order(ob, epoch);
                        self.send_buy_order(ob, epoch);
                    }
                } else {
                    self.send_buy_order(ob, epoch);
                    self.send_sell_order(ob, epoch);
                }
            }
            | (true, false) => {
                self.send_buy_order(ob, epoch);
            }
            | (false, true) => {
                self.send_sell_order(ob, epoch);
            }
            | (false, false) => {}
        }
    }

    pub fn update(&mut self, exec_report: &ExecutionReport) {
        //Filled as Maker
        if let Some(order) = self.active_buy_order {
            if exec_report.own_side == Side::Ask {
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
                    self.account.balance -= (trader_filled_qty * trader_filled_price) as i32;
                    dbgp!("TRADER FILLED: {}", trader_filled_qty);
                    if let Some(active_buy) = self.active_buy_order {
                        if trader_filled_qty == active_buy.qty {
                            self.active_buy_order = None;
                            self.strategy.buy_price = None;
                            println!(
                                "[  DB  ];{};{};{};{};",
                                active_buy.id,
                                exec_report.own_id,
                                (exec_report.own_id - active_buy.id + 3) / 1000,
                                0
                            );
                            self.lock_release();
                            self.schedule = Schedule::new();
                        } else {
                            let qty = order.qty;
                            // dbgp!("BEFORE FILLED: {:?}", self.active_buy_order);
                            self.active_buy_order = Some(Order {
                                id: order.id,
                                side: Side::Bid,
                                price: trader_filled_price,
                                qty: qty - trader_filled_qty,
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
                self.account.balance += (trader_filled_qty * trader_filled_price) as i32;
                dbgp!("TRADER FILLED: {}", trader_filled_qty);
                if let Some(active_sell) = self.active_sell_order {
                    if trader_filled_qty == active_sell.qty {
                        self.active_sell_order = None;
                        self.strategy.sell_price = None;
                        println!(
                            "[  DB  ];{};{};{};{};",
                            active_sell.id,
                            exec_report.own_id,
                            (exec_report.own_id - active_sell.id + 3) / 1000,
                            0
                        );
                        self.lock_release();
                        self.schedule = Schedule::new();
                    } else {
                        let qty = order.qty;
                        // dbgp!("BEFORE FILLED: {:?}", self.active_sell_order);
                        self.active_sell_order = Some(Order {
                            id: order.id,
                            side: Side::Ask,
                            price: trader_filled_price,
                            qty: qty - trader_filled_qty,
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
}
