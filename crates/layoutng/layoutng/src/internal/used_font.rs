//! Compatibility facade for inline-algorithm used-font state.

#![allow(non_snake_case)]

#[path = "../../../layout_algo_inline/src/native_used_font.rs"]
mod layout_algo_inline_native_used_font;

pub use layout_algo_inline_native_used_font::*;
