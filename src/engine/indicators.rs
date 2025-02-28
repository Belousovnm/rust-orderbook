use crate::engine::matching_engine::OrderBook;

// pub trait Indicator {}

pub enum Midprice {}
pub enum BestBidOffer {}
pub struct EmaMidprice {
    pub alpha: f32,
    value: Option<f32>,
}

impl Midprice {
    pub fn evaluate(ob: &OrderBook) -> Option<f32> {
        let (bid, ask, _spread) = ob.get_bbo().ok()?;
        Some(midprice(bid, ask))
    }
}
impl BestBidOffer {
    pub fn evaluate(ob: &OrderBook) -> Option<(u32, u32)> {
        let (bid, ask, _spread) = ob.get_bbo().ok()?;
        Some((bid, ask))
    }
}

impl EmaMidprice {
    pub fn evaluate(&mut self, ob: &OrderBook) -> Option<f32> {
        let (bid, ask, _spread) = ob.get_bbo().ok()?;
        self.value = if let Some(value) = self.value {
            Some(
                self.alpha
                    .mul_add(midprice(bid, ask), (1.0 - self.alpha) * value),
            )
        } else {
            Some(midprice(bid, ask))
        };
        self.value
    }

    pub const fn new(alpha: f32) -> Self {
        Self {
            alpha,
            value: Some(0.0),
        }
    }
}

pub fn midprice(bid: u32, ask: u32) -> f32 {
    (bid + ask) as f32 / 2.0
}
