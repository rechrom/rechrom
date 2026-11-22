//! Compatibility facade for the invalidation-owned source tree adapter.

#[path = "../../../layout_invalidation/src/layout_tree_invalidation.rs"]
mod layout_invalidation_tree;

pub(super) use layout_invalidation_tree::LayoutObjectInvalidationTree;
