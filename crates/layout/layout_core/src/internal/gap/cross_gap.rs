//! Compatibility facade for fragment-owned cross-gap state.

#![allow(non_snake_case)]

#[path = "../../../../layout_fragment/src/cross_gap.rs"]
mod layout_fragment_cross_gap;

pub use layout_fragment_cross_gap::*;
