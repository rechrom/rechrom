// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/editing_utilities.cc:221-318.
// Shared source query over value-only facts. DOM, Style and Interaction use
// this one implementation without reverse dependencies or copied DOM/style.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UserModify {
    ReadOnly,
    ReadWrite,
    ReadWritePlaintextOnly,
}
#[derive(Clone, Copy)]
pub struct EditableFeatures {
    pub use_shadow_host_style_check_editable: bool,
    pub inert_element_non_editable: bool,
}
impl Default for EditableFeatures {
    fn default() -> Self {
        // runtime_enabled_features.json5: both features are stable.
        Self {
            use_shadow_host_style_check_editable: true,
            inert_element_non_editable: true,
        }
    }
}
/// Short-lived facts from the canonical DOM/style owner. No descendant tree or
/// copied ComputedStyle crosses this interface.
pub trait EditableDOM {
    type Node: Copy + Eq;
    fn IsActive(&self) -> bool;
    fn Parent(&self, node: Self::Node) -> Option<Self::Node>;
    fn IsPseudoElement(&self, node: Self::Node) -> bool;
    fn IsElement(&self, node: Self::Node) -> bool;
    fn IsHTMLElementOrDocument(&self, node: Self::Node) -> bool;
    fn IsBody(&self, node: Self::Node) -> bool;
    fn IsUnassignedChildOfShadowHost(&self, node: Self::Node) -> bool;
    fn UsedUserModify(&self, node: Self::Node) -> Option<UserModify>;
    fn IsInertRoot(&self, node: Self::Node) -> bool;
}
fn HasEditableLevel<D: EditableDOM>(
    dom: &D,
    node: D::Node,
    richly: bool,
    features: EditableFeatures,
) -> bool {
    if dom.IsPseudoElement(node) {
        return false;
    }
    let mut ancestor = Some(node);
    while let Some(node) = ancestor {
        ancestor = dom.Parent(node);
        if !dom.IsHTMLElementOrDocument(node) {
            continue;
        }
        if features.use_shadow_host_style_check_editable && dom.IsUnassignedChildOfShadowHost(node)
        {
            continue;
        }
        if let Some(modify) = dom.UsedUserModify(node) {
            return match modify {
                UserModify::ReadOnly => false,
                UserModify::ReadWrite => true,
                UserModify::ReadWritePlaintextOnly => !richly,
            };
        }
        if features.inert_element_non_editable && dom.IsInertRoot(node) {
            return false;
        }
    }
    false
}
pub fn IsEditable<D: EditableDOM>(dom: &D, node: D::Node, features: EditableFeatures) -> bool {
    dom.IsActive() && HasEditableLevel(dom, node, false, features)
}
pub fn IsRichlyEditable<D: EditableDOM>(
    dom: &D,
    node: D::Node,
    features: EditableFeatures,
) -> bool {
    dom.IsActive() && HasEditableLevel(dom, node, true, features)
}
pub fn IsRootEditableElement<D: EditableDOM>(
    dom: &D,
    node: D::Node,
    features: EditableFeatures,
) -> bool {
    IsEditable(dom, node, features)
        && dom.IsElement(node)
        && (dom.IsBody(node)
            || dom
                .Parent(node)
                .is_none_or(|parent| !dom.IsElement(parent) || !IsEditable(dom, parent, features)))
}
pub fn RootEditableElement<D: EditableDOM>(
    dom: &D,
    node: D::Node,
    features: EditableFeatures,
) -> Option<D::Node> {
    let mut result = None;
    let mut ancestor = Some(node);
    while let Some(node) = ancestor {
        if !IsEditable(dom, node, features) {
            break;
        }
        if dom.IsElement(node) {
            result = Some(node);
        }
        if dom.IsBody(node) {
            break;
        }
        ancestor = dom.Parent(node);
    }
    result
}
