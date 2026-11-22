//! Compatibility facade for fragment-owned retained gap geometry.

#![allow(non_snake_case)]

#[path = "fragment/gap_geometry.rs"]
mod layout_fragment_gap_geometry;

pub use layout_fragment_gap_geometry::*;
