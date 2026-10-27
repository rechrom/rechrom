//! Compatibility facade for the block-algorithm node interface.

#![allow(non_snake_case)]

#[path = "../../../layout_algo_block/src/native_block_node.rs"]
mod layout_algo_block_native_block_node;

pub use layout_algo_block_native_block_node::*;
