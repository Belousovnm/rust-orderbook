//! Stategy calculates buy and sell criterions based on
//! input indicators. Strategy does not rightaway send
//! orders to the matchning engine, instead it communicates
//! with `OrderManagementSystem`, whichs abstracts away Risk control
//! and queue optimization  

// use std::collections::HashSet;

use core::f32;

use crate::engine::Ticker;

pub trait Strategy {}

impl Strategy for FixSpreadStrategy {}
impl Strategy for FixPriceStrategy {}
impl Strategy for SignalStrategy {}

pub struct FixSpreadStrategy {
    pub ticker: Ticker,
    pub qty: u32,
    pub buy_criterion: f32,
    pub sell_criterion: f32,
    pub master_position: i32,
    pub buy_position_limit: i32,
    pub sell_position_limit: i32,
    pub maker_range: (f32, f32),
    pub taker_range: (f32, f32),
}

impl FixSpreadStrategy {
    pub fn new(ticker: Ticker) -> Self {
        Self {
            ticker,
            buy_criterion: f32::INFINITY,
            sell_criterion: -f32::INFINITY,
            maker_range: (-f32::INFINITY, f32::INFINITY),
            taker_range: (0.0, f32::INFINITY),
            master_position: 0,
            buy_position_limit: 0,
            sell_position_limit: 0,
            qty: 0,
        }
    }
}

impl Default for FixSpreadStrategy {
    fn default() -> Self {
        Self::new(Ticker::default())
    }
}

#[derive(Default)]
pub struct FixPriceStrategy {
    pub qty: u32,
    pub buy_price: Option<u32>,
    pub sell_price: Option<u32>,
    pub buy_tick_criterion: Option<i32>,
    pub sell_tick_criterion: Option<i32>,
}

#[derive(Default)]
pub struct SignalStrategy {
    pub ticker: Ticker,
    pub qty: u32,
    pub buy_open_criterion: f32,
    pub sell_open_criterion: f32,
    pub buy_close_criterion: f32,
    pub sell_close_criterion: f32,
    pub master_position: i32,
    pub buy_position_limit: i32,
    pub sell_position_limit: i32,
    pub maker_range: (f32, f32),
    pub taker_range: (f32, f32),
}
