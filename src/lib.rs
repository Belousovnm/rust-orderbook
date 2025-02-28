//
// developed by nbelousov
//

// PERF: contig Array > BTreeMap?
// PERF: Stack alloc
// PERF: VecDeque is not contigous?
// PEPR: SNAP -> L3 -> shortest update
// TODO: TUI orderbook
// TODO: Double OB strats
// TODO: Cover all mutants
// TODO: Make FixPrice 1st class citizen
// Store total qty on level
// Add Strategy builder
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::used_underscore_binding,
    clippy::cast_possible_wrap
)]

extern crate proc_macro;

pub mod backtest;
pub mod engine;
pub mod error;
pub mod experiments;
pub mod management;
pub mod utils;
