//! Compatibility facade for the fragment-owned scrollable-overflow calculator.

#![allow(non_snake_case)]

#[path = "../../../layout_fragment/src/native_scrollable_overflow_calculator.rs"]
mod layout_fragment_native_scrollable_overflow_calculator;

pub use layout_fragment_native_scrollable_overflow_calculator::*;
