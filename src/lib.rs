// PERF: contig Array > BTreeMap?
// PERF: Stack alloc
// PERF: VecDeque is not contigous?
// PEPR: SNAP -> L3 -> shortest update
// TODO: TUI orderbook
// Store total qty on level
// Add Strategy builder
// Add typestate

pub mod account;
pub mod diff;
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
