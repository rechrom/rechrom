//! Compatibility facade for the inline-algorithm node interface.

#![allow(non_snake_case)]

#[path = "../../../layout_algo_inline/src/native_inline_node.rs"]
mod layout_algo_inline_native_inline_node;

pub use layout_algo_inline_native_inline_node::*;
