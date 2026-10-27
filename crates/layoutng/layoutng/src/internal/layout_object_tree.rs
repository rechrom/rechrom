//! Compatibility facade for invalidation-owned LayoutObject tree topology.

#![allow(non_snake_case)]

#[path = "../../../layout_invalidation/src/native_layout_object_tree.rs"]
mod layout_invalidation_native_layout_object_tree;

pub use layout_invalidation_native_layout_object_tree::*;
