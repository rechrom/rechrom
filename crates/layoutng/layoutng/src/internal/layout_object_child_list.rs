//! Compatibility facade for invalidation-owned LayoutObject child mutation.

#![allow(non_snake_case)]

#[path = "../../../layout_invalidation/src/native_layout_object_child_list.rs"]
mod layout_invalidation_native_layout_object_child_list;

pub use layout_invalidation_native_layout_object_child_list::*;
