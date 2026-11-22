//! Compatibility facade for invalidation-owned LayoutObject child mutation.

#![allow(non_snake_case)]

#[path = "invalidation/layout_object_child_list.rs"]
mod layout_invalidation_layout_object_child_list;

pub use layout_invalidation_layout_object_child_list::*;
