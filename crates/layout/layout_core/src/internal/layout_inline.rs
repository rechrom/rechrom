//! Compatibility facade for the inline LayoutObject interface.

#![allow(non_snake_case)]

#[path = "../inline/layout_inline.rs"]
mod layout_algo_inline_layout_inline;

pub use layout_algo_inline_layout_inline::*;
