//! Compatibility facade for inline text-combine layout.

#![allow(non_snake_case)]

#[path = "../../../layout_algo_inline/src/native_layout_text_combine.rs"]
mod layout_algo_inline_native_layout_text_combine;

pub use layout_algo_inline_native_layout_text_combine::*;
