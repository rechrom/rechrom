//! Compatibility facade for the block-flow LayoutObject interface.

#![allow(non_snake_case)]

#[path = "../block/layout_block_flow.rs"]
mod layout_algo_block_layout_block_flow;

pub use layout_algo_block_layout_block_flow::*;
