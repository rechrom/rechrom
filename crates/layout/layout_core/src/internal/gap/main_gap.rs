//! Compatibility facade for fragment-owned main-gap state.

#![allow(non_snake_case)]

#[path = "../../../../layout_fragment/src/main_gap.rs"]
mod layout_fragment_main_gap;

pub use layout_fragment_main_gap::*;
