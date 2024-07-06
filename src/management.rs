use crate::account::TradingAccount;
use crate::dbgp;
use crate::{
    orderbook::{ExecutionReport, Order, Side},
    strategy::Strategy,
};

pub struct OrderManagementSystem {
    pub strategy: Strategy,
    pub account: TradingAccount,
    pub active_orders: Vec<Order>,
    pub strategy_signals: Vec<Order>,
}

impl<'a, 'b> OrderManagementSystem {
    pub fn new(strategy: Strategy, account: TradingAccount) -> Self {
        Self {
            strategy,
            account,
            active_orders: Vec::with_capacity(2),
            strategy_signals: Vec::with_capacity(2),
        }
    }

    pub fn calculate_buy_order(
        &'a self,
        ref_price: Option<f32>,
        id: u64,
    ) -> Result<Order, &'b str> {
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
            Ok(Order {
                id,
                side: Side::Bid,
                price,
                qty,
            })
        } else {
            Err("No Limit left")
        }
    }

    pub fn calculate_sell_order(
        &'a self,
        ref_price: Option<f32>,
        id: u64,
    ) -> Result<Order, &'b str> {
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
            Ok(Order {
                id,
                side: Side::Ask,
                price,
                qty,
            })
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
                "[ KEY ] qty = {:?}, price = {:?}",
                trader_filled_qty,
                trader_filled_price,
            );
            self.strategy.master_position += sign * trader_filled_qty as i32;
            self.account.balance -= sign * (trader_filled_qty * trader_filled_price) as i32;
            dbgp!("TRADER FILLED: {}", trader_filled_qty);
            // std::mem::swap(&mut self.strategy.master_position, &mut new_position);
        }
    }
}
