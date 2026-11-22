//! Base facade for the scrollable-overflow calculator (LayoutBox behaviour).

#![allow(non_snake_case)]

#[path = "fragment/scrollable_overflow_calculator.rs"]
mod layout_core_scrollable_overflow_calculator;

pub use layout_core_scrollable_overflow_calculator::*;
