//! Compatibility facade for block-level list-marker placement.

#![allow(non_snake_case)]

#[path = "block/unpositioned_list_marker_interface.rs"]
mod layout_algo_block_unpositioned_list_marker_interface;

pub use layout_algo_block_unpositioned_list_marker_interface::*;
