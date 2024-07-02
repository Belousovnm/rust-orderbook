//! Stategy calculates buy and sell criterions based on
//! input indicators. Strategy does not rightaway send
//! orders to the matchning engine, instead it communicates
//! with OrderManagementSystem, whichs abstracts away Risk control
//! and queue optimization  
use std::f32::INFINITY;

pub enum StrategyName {
    TestStrategy,
}

#[allow(dead_code)]
pub struct Strategy {
    name: StrategyName,
    pub buy_criterion: f32,
    pub sell_criterion: f32,
    master_position: i32,
    buy_position_limit: i32,
    sell_position_limit: i32,
}

impl Strategy {
    pub fn new(name: StrategyName) -> Self {
        Strategy {
            name,
            buy_criterion: INFINITY,
            sell_criterion: -INFINITY,
            master_position: 0,
            buy_position_limit: 0,
            sell_position_limit: 0,
        }
    }
}
