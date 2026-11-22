//! Compatibility facade for invalidation-owned LayoutObject tree topology.

#![allow(non_snake_case)]

#[path = "../invalidation/layout_object_tree.rs"]
mod layout_invalidation_layout_object_tree;

pub use layout_invalidation_layout_object_tree::*;
