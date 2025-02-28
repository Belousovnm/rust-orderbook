// TODO: Err instead of unreachable panic
use crate::{
    backtest::Strategy,
    dbgp,
    engine::TradingAccount,
    engine::{ExecutionReport, OrderBook, OrderStatus},
    engine::{Order, Side},
    experiments::Schedule,
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

    /// # Panics
    ///
    /// Will panic
    pub fn send_buy_maker(&mut self, ob: &mut OrderBook) -> ExecutionReport {
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
        exec_report
    }

    /// # Panics
    ///
    /// Will panic
    pub fn send_sell_maker(&mut self, ob: &mut OrderBook) -> ExecutionReport {
        let exec_report;
        if let Some(order) = self.active_sell_order {
            exec_report = ob
                .amend_limit_order(order.id, self.strategy_sell_signal.unwrap())
                .unwrap();
            dbgp!("Amend sell order {:?}", exec_report);
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
        exec_report
    }
}
