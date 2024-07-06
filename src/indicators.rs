use crate::orderbook::OrderBook;

pub enum Indicator {
    Midprice,
}

impl Indicator {
    pub fn evaluate(&self, ob: &OrderBook) -> Option<f32> {
        match self {
            Indicator::Midprice => {
                let (bid, ask, _spread) = ob.get_bbo().ok()?;
                Some(midprice(bid, ask))
            }
        }
    }
}

pub fn midprice(bid: u64, ask: u64) -> f32 {
    (bid + ask) as f32 / 2.0
}
