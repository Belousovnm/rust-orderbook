// TODO: pure Array > BTreeMap?
// TODO: TUI orderbook
// FEATURE: "Replace" order Type
// store total qty on level
// Problem: VecDeque is not contigous?
// Updates from deltas
// Add Strategy builder

pub mod account;
pub mod management;
pub mod strategy;
pub mod utils;

mod event;
mod indicators;
mod orderbook;
mod snap;

pub use event::*;
pub use indicators::*;
pub use orderbook::*;
pub use snap::*;
