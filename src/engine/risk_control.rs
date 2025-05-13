use crate::{backtest::Strategy, engine::OrderBook, management::OrderManagementSystem};

#[allow(dead_code)]
impl<S: Strategy> OrderManagementSystem<'_, S> {
    pub fn cancel_all_orders(&mut self, ob: &mut OrderBook) {
        if let Some(order) = self.active_buy_order {
            let _ = ob.cancel_order(order.id);
            self.active_buy_order = None;
        }
        if let Some(order) = self.active_sell_order {
            let _ = ob.cancel_order(order.id);
            self.active_sell_order = None;
        }
    }
}
