//! Compatibility facade for fragment-owned scroll layout scopes.

#![allow(non_snake_case)]

#[path = "../../../layout_fragment/src/paint_layer_scrollable_area_scope.rs"]
mod layout_fragment_paint_layer_scrollable_area_scope;

pub use layout_fragment_paint_layer_scrollable_area_scope::*;
