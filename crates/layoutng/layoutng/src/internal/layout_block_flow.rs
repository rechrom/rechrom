//! Compatibility facade for the block-flow LayoutObject interface.

#![allow(non_snake_case)]

#[path = "../../../layout_algo_block/src/native_layout_block_flow.rs"]
mod layout_algo_block_native_layout_block_flow;

pub use layout_algo_block_native_layout_block_flow::*;
