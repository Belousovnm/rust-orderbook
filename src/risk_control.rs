use crate::backtest::Strategy;
use crate::management::OrderManagementSystem;
use crate::OrderBook;

impl<'a, S: Strategy> OrderManagementSystem<'a, S> {
    fn cancel_all_orders(&self, ob: &mut OrderBook) {
        if let Some(order) = self.active_buy_order {
            let _ = ob.cancel_order(order.id);
        };
        if let Some(order) = self.active_sell_order {
            let _ = ob.cancel_order(order.id);
        };
    }
}
