//! Compatibility facade for fragment-owned retained gap geometry.

#![allow(non_snake_case)]

#[path = "../../../../layout_fragment/src/native_gap_geometry.rs"]
mod layout_fragment_native_gap_geometry;

pub use layout_fragment_native_gap_geometry::*;
