//! Compatibility facade for the inline LayoutText interface.

#![allow(non_snake_case)]

#[path = "../../../layout_algo_inline/src/native_layout_text.rs"]
mod layout_algo_inline_native_layout_text;

pub use layout_algo_inline_native_layout_text::*;
