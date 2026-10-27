//! Compatibility facade for fragment-owned sticky constraint geometry.

#![allow(non_snake_case)]

#[path = "../../../layout_fragment/src/native_layout_sticky_constraints.rs"]
mod layout_fragment_native_layout_sticky_constraints;

pub use layout_fragment_native_layout_sticky_constraints::*;
