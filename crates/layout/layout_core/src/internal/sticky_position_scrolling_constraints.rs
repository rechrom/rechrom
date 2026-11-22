//! Compatibility facade for fragment-owned sticky scrolling constraints.

#![allow(non_snake_case)]

#[path = "../../../layout_fragment/src/sticky_position_scrolling_constraints.rs"]
mod layout_fragment_sticky_position_scrolling_constraints;

pub use layout_fragment_sticky_position_scrolling_constraints::*;
