pub struct TradingAccount {
    pub balance: i32,
}

impl TradingAccount {
    pub fn new(initial_balance: i32) -> Self {
        TradingAccount {
            balance: initial_balance,
        }
    }
}
