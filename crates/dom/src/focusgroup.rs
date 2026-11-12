// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/dom/element.cc:11978-11990,13364-13410.
//! ElementRareData's canonical parsed value and last-focused weak identity.
#![allow(non_snake_case)]
use super::{DOMNamespace, DOMNodeType, PersistentDocument};
use foundation::focusgroup_flags::{
    FocusgroupData, FocusgroupFeatures, FocusgroupFlags, ParseFocusgroup,
};

#[derive(Clone, Debug, Default)]
pub(super) struct ElementFocusgroupData {
    parsed: FocusgroupData,
    last_focused: Option<u64>,
}
impl PersistentDocument {
    // core/page/focusgroup_controller_utils.cc:301-339, using canonical
    // FlatTreeTraversal relations (including assignment and shadow hosts).
    pub fn FlatTreeNextElement(&self, node: usize, skip_subtree: bool) -> Option<usize> {
        let mut next = if skip_subtree {
            self.FocusNextSkippingChildren(node, true)
        } else {
            self.FocusNextNode(node, true)
        };
        while let Some(node) = next {
            if self.Node(node).Type() == DOMNodeType::kElement {
                return Some(node);
            }
            next = self.FocusNextNode(node, true);
        }
        None
    }
    pub fn FlatTreePreviousElement(&self, node: usize, skip_subtree: bool) -> Option<usize> {
        let mut previous = if skip_subtree {
            let mut ancestor = Some(node);
            let mut sibling = None;
            while let Some(node) = ancestor {
                sibling = self.FlatTreePreviousSibling(node);
                if sibling.is_some() {
                    break;
                }
                ancestor = self.FlatTreeParent(node);
            }
            sibling
        } else {
            self.FocusPreviousNode(node, true)
        };
        while let Some(node) = previous {
            if self.Node(node).Type() == DOMNodeType::kElement {
                return Some(node);
            }
            previous = self.FocusPreviousNode(node, true);
        }
        None
    }

    // HTMLElement::isContentEditableForBinding/IsEditableOrEditingHost,
    // html_element.cc:191-216,3666-3668. This is the binding's attribute-based
    // query, deliberately distinct from editing's computed UserModify query.
    pub fn IsContentEditableForBinding(&self, node: usize) -> bool {
        let value = self.Node(node);
        let html =
            value.Type() == DOMNodeType::kElement && value.Namespace() == DOMNamespace::kHTML;
        if html {
            let attr = value
                .FindAttribute("contenteditable")
                .map(|attr| attr.value.as_str());
            if attr.is_some_and(|attr| {
                attr.is_empty()
                    || attr.eq_ignore_ascii_case("true")
                    || attr.eq_ignore_ascii_case("plaintext-only")
            }) {
                return true;
            }
            if self.DocumentInDesignMode(self.NodeDocumentHandle(node))
                && self.IsConnectedToDocument(node)
            {
                return true;
            }
            if attr.is_some_and(|attr| attr.eq_ignore_ascii_case("false")) {
                return false;
            }
        }
        let Some(parent) = value.Parent() else {
            return false;
        };
        if !self.IsContentEditableForBinding(parent) {
            return false;
        }
        html || value.IsElement("svg", DOMNamespace::kSVG)
            || value.IsElement("math", DOMNamespace::kMathML)
            || (value.Type() != DOMNodeType::kElement
                && self.Node(parent).Namespace() == DOMNamespace::kHTML
                && self.Node(parent).Type() == DOMNodeType::kElement)
    }
    // core/dom/flat_tree_traversal.cc:125-159. Keep assigned-node sibling
    // decisions with DOM's live assignment owner, not in Page traversal.
    pub fn FlatTreeNextSibling(&self, node: usize) -> Option<usize> {
        self.FlatTreeSibling(node, true)
    }
    pub fn FlatTreePreviousSibling(&self, node: usize) -> Option<usize> {
        self.FlatTreeSibling(node, false)
    }
    fn FlatTreeSibling(&self, node: usize, forward: bool) -> Option<usize> {
        let parent = self.arena[node].parent?;
        let assigned;
        let siblings = if self.ShadowRoot(parent).is_some() {
            let slot = self.AssignedSlot(node)?;
            assigned = self.AssignedNodes(slot);
            assigned.as_slice()
        } else {
            self.arena[parent].Children()
        };
        let index = siblings.iter().position(|&sibling| sibling == node)?;
        siblings
            .get(if forward {
                index + 1
            } else {
                index.checked_sub(1)?
            })
            .copied()
    }
    /// Runtime configuration is document input, set at construction/admission.
    /// Changing it reparses existing attributes once, never during navigation.
    pub fn SetFocusgroupFeatures(&mut self, features: FocusgroupFeatures) {
        if self.focusgroup_features == features {
            return;
        }
        self.focusgroup_features = features;
        if features.enabled {
            for node in 0..self.arena.len() {
                if self.arena[node].node_type == DOMNodeType::kElement {
                    self.UpdateFocusgroup(node);
                }
            }
        }
    }
    pub fn FocusgroupFeatures(&self) -> FocusgroupFeatures {
        self.focusgroup_features
    }
    pub(super) fn UpdateFocusgroup(&mut self, node: usize) {
        if !self.focusgroup_features.enabled {
            return;
        }
        let parsed = self.arena[node]
            .FindAttribute("focusgroup")
            .map(|attribute| ParseFocusgroup(&attribute.value, self.focusgroup_features.v2))
            .unwrap_or_default();
        if parsed == FocusgroupData::default() && self.arena[node].focusgroup.is_none() {
            return;
        }
        self.arena[node]
            .focusgroup
            .get_or_insert_with(Default::default)
            .parsed = parsed;
    }
    pub fn FocusgroupData(&self, node: usize) -> FocusgroupData {
        if !self.focusgroup_features.enabled {
            return FocusgroupData::default();
        }
        self.arena[node]
            .focusgroup
            .as_ref()
            .map_or(FocusgroupData::default(), |owner| owner.parsed)
    }
    pub fn FocusgroupLastFocused(&self, owner: usize) -> Option<u64> {
        let data = self.FocusgroupData(owner);
        if !data.IsActualFocusgroup() || data.flags.Intersects(FocusgroupFlags::kNoMemory) {
            return None;
        }
        self.arena[owner].focusgroup.as_ref()?.last_focused
    }
    pub fn SetFocusgroupLastFocused(&mut self, owner: usize, node: u64) {
        let data = self.FocusgroupData(owner);
        assert!(data.IsActualFocusgroup() && !data.flags.Intersects(FocusgroupFlags::kNoMemory));
        assert!(self.FindNodeById(node).is_some());
        self.arena[owner]
            .focusgroup
            .get_or_insert_with(Default::default)
            .last_focused = Some(node);
    }
    pub fn ClearFocusgroupLastFocused(&mut self, owner: usize) {
        if !self.focusgroup_features.enabled {
            return;
        }
        if let Some(owner) = self.arena[owner].focusgroup.as_mut() {
            owner.last_focused = None;
        }
    }

    // Source: Document::UpdateFocusgroupLastFocused,
    // document.cc:6129-6143; FocusgroupControllerUtils::
    // GetFocusgroupOwnerOfItem/IsExcludedFromAncestorFocusgroup,
    // focusgroup_controller_utils.cc:426-436,872-925. Interaction supplies the
    // current Element virtual result after SetFocused/listener reentry; an
    // earlier eligibility result is not sufficient for GetFocusgroupOwnerOfItem.
    pub fn UpdateFocusgroupLastFocused(&mut self, focused: usize, is_focusable: bool) {
        if !self.focusgroup_features.enabled || !is_focusable || self.IsInTopLayer(focused) {
            return;
        }

        // FindExcludedSubtreeRoot stops at the first actual focusgroup so a
        // top-layer focusgroup can still own its descendants.
        let mut current = Some(focused);
        while let Some(node) = current {
            let data = self.FocusgroupData(node);
            if data.behavior == foundation::focusgroup_flags::FocusgroupBehavior::kOptOut
                || (self.IsInTopLayer(node) && !data.IsActualFocusgroup())
            {
                return;
            }
            if data.IsActualFocusgroup() {
                break;
            }
            current = self.FlatTreeParentElement(node);
        }

        // focusgroup::FindFocusgroupOwner starts at the parent: an element is
        // never its own owner, including when it defines a nested focusgroup.
        let mut ancestor = self.FlatTreeParentElement(focused);
        while let Some(node) = ancestor {
            let data = self.FocusgroupData(node);
            if data.behavior == foundation::focusgroup_flags::FocusgroupBehavior::kOptOut {
                return;
            }
            if data.IsActualFocusgroup() {
                if !data.flags.Intersects(FocusgroupFlags::kNoMemory) {
                    let id = self.Node(focused).Id();
                    self.SetFocusgroupLastFocused(node, id);
                }
                return;
            }
            ancestor = self.FlatTreeParentElement(node);
        }
    }
}
