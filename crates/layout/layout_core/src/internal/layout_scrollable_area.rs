//! Compatibility facade for fragment-owned scrollable-area state.

#![allow(non_snake_case)]

#[path = "../../../layout_fragment/src/paint_layer_scrollable_area.rs"]
mod layout_fragment_paint_layer_scrollable_area;

pub use layout_fragment_paint_layer_scrollable_area::*;
