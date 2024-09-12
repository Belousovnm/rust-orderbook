use crate::orderbook::OrderBook;

// pub trait Indicator {}

pub enum Midprice {}
pub enum BestBidOffer {}

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
pub fn midprice(bid: u32, ask: u32) -> f32 {
    (bid + ask) as f32 / 2.0
}
