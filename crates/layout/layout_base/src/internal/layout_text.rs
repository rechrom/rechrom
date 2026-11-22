//! Compatibility facade for the inline LayoutText interface.

#![allow(non_snake_case)]

#[path = "inline/layout_text_interface.rs"]
mod layout_algo_inline_layout_text_interface;

pub use layout_algo_inline_layout_text_interface::*;
