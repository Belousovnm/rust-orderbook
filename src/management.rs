use crate::{
    orderbook::{ExecutionReport, Order, OrderBook, Side},
    strategy::Strategy,
};

pub struct OrderManagementSystem {
    pub strategy: Strategy,
    pub active_orders: Vec<Order>,
    pub strategy_signals: Vec<Order>,
}

impl OrderManagementSystem {
    pub fn send_buy_order(&self, ob: &mut OrderBook, ref_price: f32) -> ExecutionReport {
        let price = (ref_price * (1.0 + self.strategy.buy_criterion)).floor() as u64;
        let qty = 10;
        let order = Order {
            id: 666,
            side: Side::Bid,
            price,
            qty,
        };
        ob.add_limit_order(order)
    }

    pub fn send_sell_order(&self, ob: &mut OrderBook, ref_price: f32) -> ExecutionReport {
        let price = (ref_price * (1.0 + self.strategy.sell_criterion)).ceil() as u64;
        let qty = 10;
        let order = Order {
            id: 999,
            side: Side::Ask,
            price,
            qty,
        };
        ob.add_limit_order(order)
    }
}
