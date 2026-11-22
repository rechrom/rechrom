//! Compatibility facade for the inline-algorithm node interface.

#![allow(non_snake_case)]

#[path = "inline/inline_node_interface.rs"]
mod layout_algo_inline_inline_node_interface;

pub use layout_algo_inline_inline_node_interface::*;
