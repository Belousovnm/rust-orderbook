pub struct Ticker {
    pub ticker_id: u64,
    // Minimal increment
    pub tick_size: f32,
    // Cost of tick_size in cash
    pub step_price: f32,
    pub taker_fee: f32,
    pub maker_fee: f32,
}

impl Default for Ticker {
    fn default() -> Self {
        Self {
            ticker_id: 0,
            tick_size: 1.0,
            step_price: 0.1,
            taker_fee: 0.0,
            maker_fee: 0.0,
        }
    }
}
