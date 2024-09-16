//! Stategy calculates buy and sell criterions based on
//! input indicators. Strategy does not rightaway send
//! orders to the matchning engine, instead it communicates
//! with `OrderManagementSystem`, whichs abstracts away Risk control
//! and queue optimization  

// use std::collections::HashSet;

pub trait Strategy {}

impl Strategy for TestStrategy {}
impl Strategy for FixPriceStrategy {}

impl TestStrategy {
    pub const fn get_master_position(&self) -> i32 {
        self.master_position
    }
    pub fn increment_master_position(&mut self, incr: i32) {
        self.master_position += incr;
    }
}

pub struct TestStrategy {
    pub qty: u32,
    pub buy_criterion: f32,
    pub sell_criterion: f32,
    pub master_position: i32,
    pub buy_position_limit: i32,
    pub sell_position_limit: i32,
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
#[derive(Default)]
pub struct FixPriceStrategy {
    pub qty: u32,
    pub buy_price: Option<u32>,
    pub sell_price: Option<u32>,
    pub buy_tick_criterion: Option<u8>,
    pub sell_tick_criterion: Option<u8>,
}
