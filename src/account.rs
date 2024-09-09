#[derive(Default)]
pub struct TradingAccount {
    pub balance: i32,
}

impl TradingAccount {
    pub const fn new(initial_balance: i32) -> Self {
        Self {
            balance: initial_balance,
        }
    }
}
