//! Stategy calculates buy and sell criterions based on
//! input indicators. Strategy does not rightaway send
//! orders to the matchning engine, instead it communicates
//! with `OrderManagementSystem`, whichs abstracts away Risk control
//! and queue optimization  

// use std::collections::HashSet;

pub trait Strategy {
    fn get_master_position(&self) -> i32;
    fn increment_master_position(&mut self, incr: i32);
}

impl Strategy for TestStrategy {
    fn get_master_position(&self) -> i32 {
        self.master_position
    }
    fn increment_master_position(&mut self, incr: i32) {
        self.master_position += incr;
    }
}
impl Strategy for FixPriceStrategy {
    fn get_master_position(&self) -> i32 {
        self.master_position
    }
    fn increment_master_position(&mut self, incr: i32) {
        self.master_position += incr;
    }
}

pub struct TestStrategy {
    pub buy_criterion: f32,
    pub sell_criterion: f32,
    pub master_position: i32,
    pub buy_position_limit: i32,
    pub sell_position_limit: i32,
    pub qty: u32,
}

impl TestStrategy {
    pub fn new() -> Self {
        Self {
            buy_criterion: f32::INFINITY,
            sell_criterion: -f32::INFINITY,
            master_position: 0,
            buy_position_limit: 0,
            sell_position_limit: 0,
            qty: 0,
        }
    }
}

impl Default for TestStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub struct FixPriceStrategy {
    pub master_position: i32,
}
