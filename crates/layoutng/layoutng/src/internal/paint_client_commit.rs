//! ABI-shaped entry into the layout_commit-owned native client traversal.
#![allow(non_snake_case)]

use std::collections::HashSet;

use crate::fragment_tree::LayoutResultHandle;
use crate::internal::layout_object_builder::LayoutObjectTree;

impl LayoutObjectTree {
    /// Commit only clients reachable from this document's native tree. IDs in
    /// exported snapshots are comparison keys, never pointers to dereference.
    // cpp: platform/graphics/paint/paint_controller.cc:127-143
    // cpp: platform/graphics/paint/paint_chunker.cc:22-28
    pub fn CommitPaintClientState(
        &self,
        used_clients: &HashSet<u64>,
        fragments: &mut LayoutResultHandle,
    ) {
        crate::layout_commit_package::commit_paint_client_state(self, used_clients, fragments)
    }
}
