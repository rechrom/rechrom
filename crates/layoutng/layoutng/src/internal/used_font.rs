//! Compatibility facade for inline-algorithm used-font state.

#![allow(non_snake_case)]

#[path = "../../../layout_algo_inline/src/used_font_interface.rs"]
mod layout_algo_inline_used_font_interface;

pub use layout_algo_inline_used_font_interface::*;
