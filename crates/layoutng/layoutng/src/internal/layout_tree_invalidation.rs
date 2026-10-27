//! Compatibility facade for the invalidation-owned native tree adapter.

#[path = "../../../layout_invalidation/src/native_layout_tree_invalidation.rs"]
mod layout_invalidation_native_tree;

pub(super) use layout_invalidation_native_tree::LayoutObjectInvalidationTree;
