//! Compatibility facade for fragment-owned PaintLayer identity.

#![allow(non_snake_case)]

#[path = "../fragment/paint_layer.rs"]
mod layout_fragment_paint_layer;

pub use layout_fragment_paint_layer::*;
