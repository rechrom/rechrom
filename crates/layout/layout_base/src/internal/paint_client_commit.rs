//! ABI-shaped entry into the layout_commit-owned source client traversal.
#![allow(non_snake_case)]

use std::collections::HashSet;

use crate::fragment_tree::LayoutResultHandle;
use crate::internal::layout_object_builder::LayoutObjectTree;

impl LayoutObjectTree {
    /// Commit only clients reachable from this document's source tree. IDs in
    /// exported snapshots are comparison keys, never pointers to dereference.
    /// `source_clients` is how many of `used_clients` can be source clients
    /// (the used set less the caret clients); the walk stops once it has
    /// validated that many.
    // cpp: platform/graphics/paint/paint_controller.cc:127-143
    // cpp: platform/graphics/paint/paint_chunker.cc:22-28
    pub fn CommitPaintClientState(
        &self,
        used_clients: &HashSet<u64>,
        source_clients: usize,
        fragments: &mut LayoutResultHandle,
    ) {
        crate::layout_commit_package::commit_paint_client_state(
            self,
            used_clients,
            source_clients,
            fragments,
        )
    }
}
