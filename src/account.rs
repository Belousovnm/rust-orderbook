#[derive(Default)]
pub struct TradingAccount {
    pub balance: f32,
    pub cumulative_volume: u32,
    pub trade_count: u32,
}

impl TradingAccount {
    pub const fn new(initial_balance: f32) -> Self {
        Self {
            balance: initial_balance,
            cumulative_volume: 0,
            trade_count: 0,
        }
    }
}
