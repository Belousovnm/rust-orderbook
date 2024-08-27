use crate::orderbook::OrderBook;

pub enum Indicator {
    Midprice,
}
// Cloning is depressing
impl Indicator {
    pub fn evaluate(&self, ob: &OrderBook) -> Option<f32> {
        let mut raw_ob = ob.clone();
        let _ = raw_ob.cancel_order(333);
        let _ = raw_ob.cancel_order(777);
        match self {
            Indicator::Midprice => {
                let (bid, ask, _spread) = raw_ob.get_bbo().ok()?;
                Some(midprice(bid, ask))
            }
        }
    }
}

pub fn midprice(bid: u32, ask: u32) -> f32 {
    (bid + ask) as f32 / 2.0
}
