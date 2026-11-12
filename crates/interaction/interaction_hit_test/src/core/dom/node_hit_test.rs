// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Source: third_party/blink/renderer/core/dom/node.cc:3018-3034
use crate::core::layout::hit_test_result::{ElementKind, NodeRef};

pub fn EnclosingLinkEventParentOrSelf(node: &NodeRef) -> Option<NodeRef> {
    let mut current = Some(node.clone());
    while let Some(node) = current {
        if node
            .Element()
            .is_some_and(|element| element.IsLink() && element.Kind() != ElementKind::Image)
        {
            return Some(node);
        }
        current = node.FlatTreeParent();
    }
    None
}

/// Read-only Node::CanStartSelection facts. A caller encountering a locked
/// ancestor must prepare that source layout-tree update before retrying.
/// Source: node.cc:2038-2063.
pub trait SelectionStartNode {
    fn LockedAncestorPreventingPaint(&self) -> bool;
    fn IsEditable(&self) -> bool;
    fn LayoutSelectionStyle(&self) -> Option<(foundation::EUserSelect, foundation::EUserDrag)>;
    fn FlatTreeParentForSelection(&self) -> Option<Box<dyn SelectionStartNode + '_>>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectionStartNeedsLayoutTreeUpdate;
pub fn CanStartSelection(
    node: &dyn SelectionStartNode,
) -> Result<bool, SelectionStartNeedsLayoutTreeUpdate> {
    if node.LockedAncestorPreventingPaint() {
        return Err(SelectionStartNeedsLayoutTreeUpdate);
    }
    Ok(CanStartSelectionAfterLayoutTreeUpdate(node))
}
/// The caller has completed Node::CanStartSelection's synchronous locked
/// ancestor style-tree barrier. This is also the continuation of the ordinary
/// entry above; selection policy remains in one source implementation.
pub fn CanStartSelectionAfterLayoutTreeUpdate(node: &dyn SelectionStartNode) -> bool {
    if node.IsEditable() {
        return true;
    }
    if let Some((user_select, user_drag)) = node.LayoutSelectionStyle() {
        if user_select == foundation::EUserSelect::kNone {
            return false;
        }
        if user_drag != foundation::EUserDrag::kElement
            && matches!(
                user_select,
                foundation::EUserSelect::kText | foundation::EUserSelect::kAll
            )
        {
            return true;
        }
    }
    node.FlatTreeParentForSelection()
        .is_none_or(|parent| CanStartSelectionAfterLayoutTreeUpdate(parent.as_ref()))
}
