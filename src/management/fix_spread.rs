#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

use crate::{
    backtest::FixSpreadStrategy,
    dbgp,
    engine::indicators::BestBidOffer,
    engine::OrderStatus,
    engine::{ExecutionReport, Order, OrderBook, Side},
    management::OrderManagementSystem,
};

impl OrderManagementSystem<'_, FixSpreadStrategy> {
    /// # Errors
    ///
    /// Will return `Err` if either `Indicator` fails to provide reference price
    /// or `Strategy` has no limit left for this side
    pub fn calculate_buy_order(
        &self,
        ref_price: Option<f32>,
        id: Option<u64>,
    ) -> Result<Order, String> {
        if let Some(id) = id {
            let side = Side::Bid;
            let price = ((ref_price.ok_or_else(|| "Missing Ref Price".to_owned())?
                * (1.0 + self.strategy.buy_criterion)
                / self.strategy.ticker.tick_size)
                .floor()
                * self.strategy.ticker.tick_size) as u32;
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
        } else {
            Err("No order id".to_owned())
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if either `Indicator` fails to provide reference price
    /// or `Strategy` has no limit left for this side
    pub fn calculate_sell_order(
        &self,
        ref_price: Option<f32>,
        id: Option<u64>,
    ) -> Result<Order, String> {
        if let Some(id) = id {
            let side = Side::Ask;
            let price = ((ref_price.ok_or_else(|| "Missing Ref Price".to_owned())?
                * (1.0 + self.strategy.sell_criterion)
                / self.strategy.ticker.tick_size)
                .ceil()
                * self.strategy.ticker.tick_size) as u32;
            let free_qty = if -self.strategy.sell_position_limit + self.strategy.master_position > 0
            {
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
                };
                Ok(order)
            } else {
                Err("No Limit left".to_owned())
            }
        } else {
            Err("No order id".to_owned())
        }
    }

    /// # Panics
    ///
    /// Will panic
    pub fn send_orders(
        &mut self,
        ob: &mut OrderBook,
        m: Option<f32>,
        trader_buy_id: Option<u64>,
        trader_sell_id: Option<u64>,
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
            self.active_buy_order = None;
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
            self.active_sell_order = None;
        };

        match (send_buy_order, send_sell_order) {
            | (true, true) => {
                if let Some(active_sell) = self.active_sell_order {
                    if self.strategy_buy_signal.unwrap().price < active_sell.price {
                        self.send_buy(ob);
                        self.send_sell(ob);
                    } else {
                        self.send_sell(ob);
                        self.send_buy(ob);
                    }
                } else {
                    self.send_buy(ob);
                    self.send_sell(ob);
                }
            }
            | (true, false) => {
                let _ = self.send_buy(ob);
            }
            | (false, true) => {
                let _ = self.send_sell(ob);
            }
            | (false, false) => {}
        }
    }

    fn send_buy(&mut self, ob: &mut OrderBook) -> ExecutionReport {
        let strat_price = self.strategy_buy_signal.unwrap().price as f32;
        let (bid, ask) = BestBidOffer::evaluate(ob).expect("Empty orderbook");
        if strat_price >= bid as f32 * (1.0 + self.strategy.maker_range.0)
            && strat_price <= bid as f32 * (1.0 + self.strategy.maker_range.1)
            && strat_price < ask as f32
        {
            self.send_buy_maker(ob)
        } else if strat_price >= ask as f32
            && strat_price <= bid as f32 * (1.0 + self.strategy.taker_range.1)
        {
            self.send_buy_taker(ob)
        } else {
            unreachable!()
        }
    }
    fn send_sell_taker(&mut self, ob: &mut OrderBook) -> ExecutionReport {
        let exec_report;
        let taker_order = self.strategy_sell_signal.unwrap();
        if let Some(order) = self.active_sell_order {
            exec_report = ob.amend_limit_order(order.id, taker_order).unwrap();
        } else {
            exec_report = ob.add_limit_order(taker_order);
        }
        if exec_report.status == OrderStatus::Filled {
            self.update_taker(&exec_report);
            self.active_sell_order = None;
        } else if exec_report.status == OrderStatus::PartiallyFilled {
            self.update_taker(&exec_report);
        } else {
            // Only taker orders allowed
            unreachable!();
        }
        exec_report
    }

    fn send_buy_taker(&mut self, ob: &mut OrderBook) -> ExecutionReport {
        let exec_report;
        let taker_order = self.strategy_buy_signal.unwrap();
        if let Some(order) = self.active_buy_order {
            exec_report = ob.amend_limit_order(order.id, taker_order).unwrap();
        } else {
            exec_report = ob.add_limit_order(taker_order);
        }
        if exec_report.status == OrderStatus::Filled {
            self.update_taker(&exec_report);
            self.active_buy_order = None;
        } else if exec_report.status == OrderStatus::PartiallyFilled {
            self.update_taker(&exec_report);
        } else {
            // Only taker orders allowed
            unreachable!();
        }
        exec_report
    }

    fn send_sell(&mut self, ob: &mut OrderBook) -> ExecutionReport {
        let strat_price = self.strategy_sell_signal.unwrap().price as f32;
        let (bid, ask) = BestBidOffer::evaluate(ob).unwrap();
        if strat_price <= ask as f32 * (1.0 - self.strategy.maker_range.0)
            && strat_price >= ask as f32 * (1.0 + self.strategy.maker_range.0)
            && strat_price > bid as f32
        {
            self.send_sell_maker(ob)
        } else if strat_price <= bid as f32
            && strat_price >= ask as f32 * (1.0 - self.strategy.taker_range.1)
        {
            self.send_sell_taker(ob)
        } else {
            unreachable!()
        }
    }

    pub fn update(&mut self, exec_report: &ExecutionReport) {
        let mut trader_filled_qty;
        let mut traded_volume = 0;
        if let Some(order) = self.active_buy_order {
            if exec_report.own_side == Side::Ask {
                if let Some(key) = exec_report
                    .filled_orders
                    .iter()
                    .position(|&o| o.0 == order.id)
                {
                    trader_filled_qty = exec_report.filled_orders[key].1;
                    let trader_filled_price = exec_report.filled_orders[key].2;
                    dbgp!(
                        "[TRADE ] qty = {:?}, price = {:?}",
                        trader_filled_qty,
                        trader_filled_price,
                    );
                    self.strategy.master_position += trader_filled_qty as i32;
                    traded_volume = trader_filled_qty * trader_filled_price;
                    self.account.balance -=
                        traded_volume as f32 * (1.0 + self.strategy.ticker.maker_fee);
                    dbgp!("TRADER FILLED: {}", trader_filled_qty);
                    if let Some(active_buy) = self.active_buy_order {
                        if trader_filled_qty == active_buy.qty {
                            self.active_buy_order = None;
                        } else {
                            let qty = order.qty;
                            self.active_buy_order = Some(Order {
                                id: order.id,
                                side: Side::Bid,
                                price: trader_filled_price,
                                qty: qty - trader_filled_qty,
                            });
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
                trader_filled_qty = exec_report.filled_orders[key].1;
                let trader_filled_price = exec_report.filled_orders[key].2;
                dbgp!(
                    "[TRADE ] qty = {:?}, price = {:?}",
                    trader_filled_qty,
                    trader_filled_price,
                );
                self.strategy.master_position -= trader_filled_qty as i32;
                traded_volume = trader_filled_qty * trader_filled_price;
                self.account.balance +=
                    traded_volume as f32 * (1.0 - self.strategy.ticker.maker_fee);
                dbgp!("TRADER FILLED: {}", trader_filled_qty);
                if let Some(active_sell) = self.active_sell_order {
                    if trader_filled_qty == active_sell.qty {
                        self.active_sell_order = None;
                    } else {
                        let qty = order.qty;
                        self.active_sell_order = Some(Order {
                            id: order.id,
                            side: Side::Ask,
                            price: trader_filled_price,
                            qty: qty - trader_filled_qty,
                        });
                    }
                }
            };
            // std::mem::swap(&mut self.strategy.master_position, &mut new_position);
        }
        self.account.cumulative_volume += traded_volume;
        if traded_volume != 0 {
            self.account.trade_count += 1;
        }
        dbgp!("POS {:#?}", self.strategy.master_position);
        dbgp!("ACC {:#?}", self.account.balance);
        dbgp!("#TRADES {:#?}", self.account.trade_count);
    }

    pub fn get_pnl(&self, ref_price: Option<f32>, in_bps: bool) -> Option<f32> {
        let pnl_abs =
            ref_price?.mul_add(self.strategy.master_position as f32, self.account.balance);
        let pnl_bps = match self.account.cumulative_volume {
            | 0 => 0.0,
            | _ => (pnl_abs / (self.account.cumulative_volume as f32)) * 10000.0,
        };
        if in_bps {
            Some(pnl_bps)
        } else {
            Some(pnl_abs * self.strategy.ticker.step_price)
        }
    }

    pub fn update_taker(&mut self, exec_report: &ExecutionReport) {
        let (traded_volume, traded_qty) =
            exec_report
                .filled_orders
                .iter()
                .fold((0, 0), |(mut volume, mut qty), x| {
                    volume += x.1 * x.2;
                    qty += x.1;
                    (volume, qty)
                });
        if exec_report.own_side == Side::Bid {
            self.strategy.master_position += traded_qty as i32;
            self.account.balance -= traded_volume as f32 * (1.0 + self.strategy.ticker.taker_fee);
            self.account.cumulative_volume += traded_volume;
            self.account.trade_count += 1;
            dbgp!("[TRADE ] qty = {:?}", traded_qty,);
            dbgp!("POS {:#?}", self.strategy.master_position);
            dbgp!("ACC {:#?}", self.account.balance);
            dbgp!("#TRADES {:#?}", self.account.trade_count);
        } else if exec_report.own_side == Side::Ask {
            self.strategy.master_position -= traded_qty as i32;
            self.account.balance += traded_volume as f32 * (1.0 - self.strategy.ticker.taker_fee);
            self.account.cumulative_volume += traded_volume;
            self.account.trade_count += 1;
            dbgp!("[TRADE ] qty = {:?}", traded_qty,);
            dbgp!("POS {:#?}", self.strategy.master_position);
            dbgp!("ACC {:#?}", self.account.balance);
            dbgp!("#TRADES {:#?}", self.account.trade_count);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::engine::OrderStatus;
    use crate::engine::Ticker;
    use crate::engine::TradingAccount;
    use crate::utils;
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use utils::tests::full_ob;

    #[rstest]
    #[case(full_ob(), 99, 10, ExecutionReport {own_id: 3,
                                                  own_side: Side::Bid,
                                                  remaining_qty: 10,
                                                  filled_orders: Vec::new(),
                                                  status: OrderStatus::Created })]
    #[case(full_ob(), 101, 15, ExecutionReport {own_id: 3,
                                                  own_side: Side::Bid,
                                                  remaining_qty: 5,
                                                  filled_orders: vec![(999, 10, 101)],
                                                  status: OrderStatus::PartiallyFilled})]
    #[case(full_ob(), 103, 30, ExecutionReport {own_id: 3,
                                                  own_side: Side::Bid,
                                                  remaining_qty: 0,
                                                  filled_orders: vec![(999, 10, 101), (1000, 10, 102), (1001, 10, 103)],
                                                  status: OrderStatus::Filled})]
    fn send_buy_test(
        #[case] mut ob: OrderBook,
        #[case] price: u32,
        #[case] qty: u32,
        #[case] exp_report: ExecutionReport,
    ) {
        let mut strat = FixSpreadStrategy::new(Ticker::default());
        let account = TradingAccount::new(0.0);
        let mut oms = OrderManagementSystem::new(&mut strat, account);
        oms.strategy_buy_signal = Some(Order {
            id: 3,
            side: Side::Bid,
            price,
            qty,
        });
        let exec_report = oms.send_buy(&mut ob);
        assert_eq!(exec_report, exp_report);
    }

    #[rstest]
    #[case(full_ob(), 101, 10, ExecutionReport {own_id: 7,
                                                  own_side: Side::Ask,
                                                  remaining_qty: 10,
                                                  filled_orders: Vec::new(),
                                                  status: OrderStatus::Created })]
    #[case(full_ob(), 99, 15, ExecutionReport {own_id: 7,
                                                  own_side: Side::Ask,
                                                  remaining_qty: 5,
                                                  filled_orders: vec![(666, 10, 99)],
                                                  status: OrderStatus::PartiallyFilled})]
    #[case(full_ob(), 97, 30, ExecutionReport {own_id: 7,
                                                  own_side: Side::Ask,
                                                  remaining_qty: 0,
                                                  filled_orders: vec![(666, 10, 99), (555, 10, 98), (444, 10, 97)],
                                                  status: OrderStatus::Filled})]
    fn send_sell_test(
        #[case] mut ob: OrderBook,
        #[case] price: u32,
        #[case] qty: u32,
        #[case] exp_report: ExecutionReport,
    ) {
        let mut strat = FixSpreadStrategy::new(Ticker::default());
        let account = TradingAccount::new(0.0);
        let mut oms = OrderManagementSystem::new(&mut strat, account);
        oms.strategy_sell_signal = Some(Order {
            id: 7,
            side: Side::Ask,
            price,
            qty,
        });
        let exec_report = oms.send_sell(&mut ob);
        assert_eq!(exec_report, exp_report);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::engine::OrderStatus;
    use crate::engine::Ticker;
    use crate::engine::TradingAccount;

    #[test]
    fn send_buy_test() {
        let mut strat = FixSpreadStrategy::new(Ticker::default());
        strat.buy_criterion = -0.0001;
        strat.sell_criterion = 0.0001;
        strat.buy_position_limit = 10;
        strat.sell_position_limit = -10;
        strat.qty = 1;
        let account = TradingAccount::new(0.0);
        let mut ob = OrderBook::new();
        let _ = ob.add_limit_order(Order {
            id: 1,
            side: Side::Bid,
            price: 99,
            qty: 1,
        });
        let _ = ob.add_limit_order(Order {
            id: 2,
            side: Side::Ask,
            price: 100,
            qty: 1,
        });
        let mut oms = OrderManagementSystem::new(&mut strat, account);
        oms.strategy_buy_signal = Some(Order {
            id: 333,
            price: 100,
            qty: 2,
            side: Side::Bid,
        });
        let exec_report = oms.send_buy(&mut ob);
        let exp_report = ExecutionReport {
            own_id: 333,
            own_side: Side::Bid,
            filled_orders: [(2, 1, 100)].to_vec(),
            remaining_qty: 1,
            status: OrderStatus::PartiallyFilled,
        };
        assert_eq!(exp_report, exec_report);
    }
}
