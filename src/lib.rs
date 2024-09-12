// PERF: contig Array > BTreeMap?
// PERF: Stack alloc
// PERF: VecDeque is not contigous?
// PEPR: SNAP -> L3 -> shortest update
// TODO: TUI orderbook
// Store total qty on level
// Add Strategy builder
// Add typestate
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::cast_possible_wrap)]

pub mod account;
pub mod backtest;
pub mod diff;
pub mod experiments;
pub mod management;
pub mod utils;

mod event;
mod indicators;
mod orderbook;
mod snap;

pub use event::*;
pub use indicators::*;
pub use orderbook::*;
pub use snap::*;
