//! Compatibility facade for fragment-owned retained transform geometry.

#![allow(non_snake_case)]

#[path = "../../../layout_fragment/src/paint_layer_transform.rs"]
mod layout_fragment_paint_layer_transform;

pub use layout_fragment_paint_layer_transform::*;
