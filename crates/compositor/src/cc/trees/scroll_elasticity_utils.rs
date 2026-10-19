// Translation status: complete
// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/scroll_elasticity_utils.h:1-23
// Source: cc/trees/scroll_elasticity_utils.cc:1-26
use super::{property_tree::ScrollTree, scroll_node::ScrollNode};
use crate::adapter::scroll_elasticity::LayerTreeHostImpl;

// Source: cc/trees/scroll_elasticity_utils.h:15-18
// Source: cc/trees/scroll_elasticity_utils.cc:11-25
// Canonical LayerTreeHostImpl implements the live HostImpl facet directly;
// ScrollTree is the resident owner, not an adapter projection.
pub fn ShouldAllowOverscrollEffect(
    node: &ScrollNode,
    scroll_tree: &ScrollTree,
    host_impl: Option<&dyn LayerTreeHostImpl>,
) -> bool {
    if node.scrolls_inner_viewport || node.scrolls_outer_viewport {
        return true;
    }
    let allow_pending_tree = host_impl.is_none() || !host_impl.unwrap().CommitsToActiveTree();
    scroll_tree.CanRealizeScrollsOnActiveTree(node)
        || (allow_pending_tree && scroll_tree.CanRealizeScrollsOnPendingTree(node))
}
