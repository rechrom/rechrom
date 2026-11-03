//! Compatibility facade for the fragment-owned scrollable-overflow calculator.

#![allow(non_snake_case)]

#[path = "../../../layout_fragment/src/scrollable_overflow_calculator.rs"]
mod layout_fragment_scrollable_overflow_calculator;

pub use layout_fragment_scrollable_overflow_calculator::*;
