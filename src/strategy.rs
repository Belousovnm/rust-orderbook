//! Stategy calculates buy and sell criterions based on
//! input indicators. Strategy does not rightaway send
//! orders to the matchning engine, instead it communicates
//! with OrderManagementSystem, whichs abstracts away Risk control
//! and queue optimization  

pub enum StrategyName {
    TestStrategy,
}

#[allow(dead_code)]
pub struct Strategy {
    name: StrategyName,
    pub buy_criterion: f32,
    pub sell_criterion: f32,
    pub master_position: i32,
    pub buy_position_limit: i32,
    pub sell_position_limit: i32,
    pub qty: u32,
}

impl Strategy {
    pub fn new(name: StrategyName) -> Self {
        Strategy {
            name,
            buy_criterion: f32::INFINITY,
            sell_criterion: -f32::INFINITY,
            master_position: 0,
            buy_position_limit: 0,
            sell_position_limit: 0,
            qty: 0,
        }
    }
}
