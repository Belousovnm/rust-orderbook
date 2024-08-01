// TODO: pure Array > BTreeMap?
// TODO: TUI orderbook
// FEATURE: "Replace" order Type
// store total qty on level
// Problem: VecDeque is not contigous?
// Updates from deltas
// Add Strategy builder
// Add typestate

pub mod account;
pub mod management;
pub mod utils;

mod event;
mod indicators;
mod orderbook;
mod snap;
mod strategy;
mod strategy_flow;

pub use event::*;
pub use indicators::*;
pub use orderbook::*;
pub use snap::*;
pub use strategy::*;
pub use strategy_flow::*;
