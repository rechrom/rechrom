//! Compatibility facade for the block LayoutObject interface.

#![allow(non_snake_case)]

#[path = "../../../layout_algo_block/src/native_layout_block.rs"]
mod layout_algo_block_native_layout_block;

pub use layout_algo_block_native_layout_block::*;
