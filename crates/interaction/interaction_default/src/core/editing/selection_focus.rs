// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/frame_selection.cc:727-815.
use std::cmp::Ordering;

/// FrameSelection::SetFocusedNodeIfNeeded, frame_selection.cc:1283-1313.
/// The host supplies only the existing visible selection and focus owners.
pub trait FocusedNodeSelectionSource {
    type Element: Copy;
    fn UpdateStyleAndLayoutForSelection(&self);
    fn VisibleSelectionIsNone(&self) -> bool;
    fn FrameIsFocused(&self) -> bool;
    fn VisibleSelectionRootEditableElement(&self) -> Option<Self::Element>;
    fn UpdateStyleAndLayoutTree(&self);
    fn IsFocusable(&self, element: Self::Element) -> bool;
    fn IsFrameElement(&self, element: Self::Element) -> bool;
    fn FlatTreeParentElement(&self, element: Self::Element) -> Option<Self::Element>;
    fn SetFocusedElement(&self, element: Self::Element);
    fn ClearFocusedElement(&self);
}
pub fn SetFocusedNodeIfNeeded<D: FocusedNodeSelectionSource>(source: &D) {
    source.UpdateStyleAndLayoutForSelection();
    if source.VisibleSelectionIsNone() || !source.FrameIsFocused() {
        return;
    }
    let Some(mut target) = source.VisibleSelectionRootEditableElement() else {
        return;
    };
    source.UpdateStyleAndLayoutTree();
    loop {
        if source.IsFocusable(target) && !source.IsFrameElement(target) {
            source.SetFocusedElement(target);
            return;
        }
        let Some(parent) = source.FlatTreeParentElement(target) else {
            break;
        };
        target = parent;
    }
    source.ClearFocusedElement();
}

/// A visible selection contributes only its original two flat-tree positions.
/// Positions retain native node/offset/anchor identity, not a projected tree.
pub struct SelectionFocusRange<P> {
    pub start: P,
    pub end: P,
}
pub trait SelectionFocusDOM {
    type Node: Copy + Eq;
    type Scope: Copy + Eq;
    type Position: Clone + PartialEq;
    fn UpdateStyleAndLayoutForSelection(&self);
    fn VisibleFlatSelection(&self) -> Option<SelectionFocusRange<Self::Position>>;
    fn ContainerNode(&self, position: &Self::Position) -> Option<Self::Node>;
    fn FocusedElement(&self) -> Option<Self::Node>;
    fn DocumentElement(&self) -> Option<Self::Node>;
    fn IsScrollControlOrMarkerGroupPseudo(&self, node: Self::Node) -> bool;
    fn IsTextControl(&self, node: Self::Node) -> bool;
    fn ContainsIncludingHostElements(&self, ancestor: Self::Node, node: Self::Node) -> bool;
    fn FirstPositionInNode(&self, node: Self::Node) -> Self::Position;
    fn ComparePositions(&self, first: &Self::Position, second: &Self::Position)
        -> Option<Ordering>;
    fn SelectionAndFocusedVisiblePositionMatchEnabled(&self) -> bool;
    fn IsFocusable(&self, node: Self::Node) -> bool;
    fn IsEditable(&self, node: Self::Node) -> bool;
    fn VisiblePositionDeepEquivalent(&self, position: &Self::Position) -> Option<Self::Position>;
    fn TreeScope(&self, node: Self::Node) -> Self::Scope;
    fn IsElement(&self, node: Self::Node) -> bool;
    fn DelegatesFocus(&self, node: Self::Node) -> bool;
    fn GetFocusableArea(&self, node: Self::Node) -> Option<Self::Node>;
    fn ParentIsShadowRoot(&self, node: Self::Node) -> bool;
    fn FlatTreeParent(&self, node: Self::Node) -> Option<Self::Node>;
}
pub fn SelectionHasFocus<D: SelectionFocusDOM>(dom: &D) -> bool {
    dom.UpdateStyleAndLayoutForSelection();
    let Some(selection) = dom.VisibleFlatSelection() else {
        return false;
    };
    let Some(mut current) = dom.ContainerNode(&selection.start) else {
        return false;
    };
    let focused_element = dom.FocusedElement();
    let Some(focused) = focused_element.or_else(|| dom.DocumentElement()) else {
        return false;
    };
    if dom.IsScrollControlOrMarkerGroupPseudo(focused) {
        return false;
    }
    if dom.IsTextControl(focused) && dom.ContainsIncludingHostElements(focused, current) {
        return true;
    }
    let focused_position = dom.FirstPositionInNode(focused);
    if dom
        .ComparePositions(&selection.start, &focused_position)
        .is_some_and(|order| order != Ordering::Greater)
        && dom
            .ComparePositions(&selection.end, &focused_position)
            .is_some_and(|order| order != Ordering::Less)
    {
        return true;
    }
    if dom.SelectionAndFocusedVisiblePositionMatchEnabled()
        && focused_element.is_some()
        && dom.IsFocusable(focused)
        && !dom.IsEditable(focused)
        && dom
            .VisiblePositionDeepEquivalent(&focused_position)
            .as_ref()
            == Some(&selection.start)
    {
        return true;
    }
    let editable = dom.IsEditable(current);
    let mut tree_scope = dom.TreeScope(current);
    loop {
        if editable && !dom.IsEditable(current) && tree_scope == dom.TreeScope(current) {
            return false;
        }
        if current == focused {
            return true;
        }
        if dom.IsElement(current)
            && dom.DelegatesFocus(current)
            && dom.GetFocusableArea(current) == Some(focused)
        {
            return true;
        }
        let stepping_out = tree_scope == dom.TreeScope(current) && dom.ParentIsShadowRoot(current);
        let Some(parent) = dom.FlatTreeParent(current) else {
            return false;
        };
        current = parent;
        if stepping_out {
            tree_scope = dom.TreeScope(current);
        }
    }
}

/// Source frame_selection.cc:818-837. Queries borrow the same visible and raw
/// selection identities; no selection model is copied into focus calculation.
pub trait SelectionHiddenDOM: SelectionFocusDOM {
    fn RawSelectionIsRange(&self) -> bool;
    fn VisibleDomSelectionStartContainerNode(&self) -> Option<Self::Node>;
    fn EnclosingTextControl(&self, node: Self::Node) -> bool;
}
pub fn IsHidden<D: SelectionHiddenDOM>(dom: &D) -> bool {
    if SelectionHasFocus(dom) {
        return false;
    }
    let Some(start) = dom.VisibleDomSelectionStartContainerNode() else {
        return true;
    };
    if !dom.RawSelectionIsRange() {
        return true;
    }
    dom.EnclosingTextControl(start)
}
