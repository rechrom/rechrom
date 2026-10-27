//! Compatibility facade for fragment-owned PaintLayer identity.

#![allow(non_snake_case)]

#[path = "../../../layout_fragment/src/native_paint_layer.rs"]
mod layout_fragment_native_paint_layer;

pub use layout_fragment_native_paint_layer::*;
