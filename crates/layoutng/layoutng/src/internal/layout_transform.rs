//! Compatibility facade for fragment-owned retained transform geometry.

#![allow(non_snake_case)]

#[path = "../../../layout_fragment/src/native_layout_transform.rs"]
mod layout_fragment_native_layout_transform;

pub use layout_fragment_native_layout_transform::*;
