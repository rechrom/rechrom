//! Compatibility facade for fragment-owned gap intersections.

#![allow(non_snake_case)]

#[path = "../../../../layout_fragment/src/native_gap_intersection.rs"]
mod layout_fragment_native_gap_intersection;

pub use layout_fragment_native_gap_intersection::*;
