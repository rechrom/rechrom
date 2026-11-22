//! Element reflection uses the original element/tree-scope owners.
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Source: element.cc:548-563,1367-1418,1434-1525.
use super::{DOMAttribute, DOMNodeClass, DOMNodeType, PersistentDocument};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HTMLElementCommandClass {
    Element,
    HTMLElement,
    Input,
    Select,
    Details,
    Dialog,
    Media,
}
pub struct ContextMenuAutofillDOMFacts {
    pub control_class: Option<super::DOMFormControlClass>,
    pub field_id: u64,
    pub owning_form_id: Option<u64>,
    pub has_been_password: bool,
    pub root_editable_id: Option<u64>,
    pub root_is_contenteditable: bool,
    pub is_form: bool,
    pub is_form_control: bool,
}

impl PersistentDocument {
    pub fn ContextMenuIsHTMLAnchor(&self, node: usize) -> bool {
        matches!(
            self.arena[node].class,
            DOMNodeClass::HTMLAnchorElement { .. }
        )
    }
    pub fn ContextMenuIsSVGAnchor(&self, node: usize) -> bool {
        matches!(self.arena[node].class, DOMNodeClass::SVGAElement { .. })
    }
    pub fn AnchorLinkRelations(&self, node: usize) -> Option<u32> {
        self.arena[node].class.AnchorLinkRelations()
    }
    pub(super) fn AnchorRelAttributeChanged(&mut self, node: usize, name: &str, namespace: &str) {
        if name != "rel"
            || !namespace.is_empty()
            || self.arena[node].class.AnchorLinkRelations().is_none()
        {
            return;
        }
        let value = self.arena[node]
            .attributes
            .iter()
            .find(|a| a.namespace_uri.is_empty() && a.local_name == "rel")
            .map_or("", |a| a.value.as_str());
        let parsed = foundation::anchor_element_utils::ParseRelAttribute(value);
        *self.arena[node]
            .class
            .AnchorLinkRelationsMut()
            .expect("source native anchor") = parsed;
    }
    pub fn ContextMenuAutofillDOMFacts(&self, node: usize) -> ContextMenuAutofillDOMFacts {
        let class = self
            .FormControlClass(node)
            .filter(|kind| *kind != super::DOMFormControlClass::Object);
        let root = foundation::editable::RootEditableElement(
            &super::DOMEditableFacts(self),
            self.Node(node).Id(),
            foundation::editable::EditableFeatures::default(),
        )
        .and_then(|id| self.FindNodeById(id))
        .filter(|&root| self.Node(root).Namespace() == super::DOMNamespace::kHTML);
        let root_is_contenteditable = root.is_some_and(|root| {
            self.arena[root]
                .attributes
                .iter()
                .find(|a| a.namespace_uri.is_empty() && a.local_name == "contenteditable")
                .is_some_and(|a| {
                    a.value.is_empty()
                        || a.value.eq_ignore_ascii_case("true")
                        || a.value.eq_ignore_ascii_case("plaintext-only")
                })
        });
        ContextMenuAutofillDOMFacts {
            control_class: class,
            field_id: self.Node(node).Id(),
            owning_form_id: class
                .and_then(|_| self.FormOwner(node))
                .map(|form| self.Node(form).Id()),
            has_been_password: self.arena[node].class.IsHTMLInputElement()
                && self.InputHasBeenPasswordField(node),
            root_editable_id: root.map(|root| self.Node(root).Id()),
            root_is_contenteditable,
            is_form: matches!(self.arena[node].class, DOMNodeClass::HTMLFormElement),
            is_form_control: class.is_some(),
        }
    }
    pub fn ContextMenuIsSupportedAnchor(&self, node: usize) -> bool {
        matches!(
            self.arena[node].class,
            DOMNodeClass::HTMLAnchorElement { .. }
                | DOMNodeClass::SVGAElement { .. }
                | DOMNodeClass::MathMLAnchorElement { .. }
        )
    }
    pub fn ContextMenuIsCanvasElement(&self, node: usize) -> bool {
        matches!(self.arena[node].class, DOMNodeClass::HTMLCanvasElement)
    }
    pub fn ContextMenuIsPluginElement(&self, node: usize) -> bool {
        matches!(
            self.arena[node].class,
            DOMNodeClass::HTMLObjectElement(_) | DOMNodeClass::HTMLPlugInElement
        )
    }
    // Source: node.cc:3018-3033. This uses FlatTreeTraversal, including slot
    // assignment, and excludes the source HTMLImageElement link virtual.
    pub fn EnclosingLinkEventParentOrSelf(&self, node: usize) -> Option<usize> {
        let mut current = Some(node);
        while let Some(node) = current {
            if self.IsLinkForEditing(node)
                && !matches!(self.arena[node].class, DOMNodeClass::HTMLImageElement)
            {
                return Some(node);
            }
            current = self.FlatTreeParent(node);
        }
        None
    }
    /// Dynamic class projection, selected once at DOM allocation.
    pub fn HTMLElementCommandClass(&self, node: usize) -> HTMLElementCommandClass {
        use HTMLElementCommandClass as C;
        match self.arena[node].class {
            DOMNodeClass::HTMLInputElement | DOMNodeClass::MediaControlInput { .. } => C::Input,
            DOMNodeClass::HTMLSelectElement(_) => C::Select,
            DOMNodeClass::HTMLDetailsElement(_) => C::Details,
            DOMNodeClass::HTMLDialogElement => C::Dialog,
            DOMNodeClass::HTMLMediaElement { .. } => C::Media,
            _ if self.arena[node].Namespace() == super::DOMNamespace::kHTML => C::HTMLElement,
            _ => C::Element,
        }
    }
    pub fn IsInTreeScope(&self, node: usize) -> bool {
        self.RegisteredTreeScope(node).is_some()
    }
    fn ElementIsDescendantOfShadowIncludingAncestor(
        &self,
        element: usize,
        candidate: usize,
    ) -> bool {
        let candidate_root = self.TreeScopeRoot(candidate);
        let mut element_root = self.TreeScopeRoot(element);
        loop {
            if candidate_root == element_root {
                return true;
            }
            let Some(host) = self.ShadowHost(element_root) else {
                return false;
            };
            element_root = self.TreeScopeRoot(host);
        }
    }
    pub fn SetElementAttribute(&mut self, element: usize, name: &str, target: Option<usize>) {
        assert_eq!(self.Node(element).Type(), DOMNodeType::kElement);
        if let Some(target) = target {
            assert_eq!(self.Node(target).Type(), DOMNodeType::kElement);
            self.SetAttribute(
                element,
                DOMAttribute {
                    local_name: name.into(),
                    value: String::new(),
                    ..DOMAttribute::default()
                },
            );
            self.explicitly_set_element_attributes
                .insert((element, name.into()), target);
        } else {
            self.explicitly_set_element_attributes
                .remove(&(element, name.into()));
            self.RemoveAttribute(element, name, "");
        }
    }
    pub fn GetElementAttribute(&self, element: usize, name: &str) -> Option<usize> {
        if let Some(&target) = self
            .explicitly_set_element_attributes
            .get(&(element, name.into()))
        {
            return self
                .ElementIsDescendantOfShadowIncludingAncestor(element, target)
                .then_some(target);
        }
        let id = &self
            .Node(element)
            .attributes
            .iter()
            .find(|attribute| attribute.namespace_uri.is_empty() && attribute.local_name == name)?
            .value;
        if id.is_empty() {
            return None;
        }
        if self.IsConnectedToDocument(element) {
            return self.FindElementByIdInTreeScope(element, id);
        }
        // Source permits disconnected roots. The tree-ordered ID map indexes
        // connected elements, so use its required ElementTraversal fallback.
        fn find(document: &PersistentDocument, root: usize, id: &str) -> Option<usize> {
            for &child in document.Node(root).Children() {
                let node = document.Node(child);
                if node.Type() == DOMNodeType::kElement
                    && node
                        .attributes
                        .iter()
                        .find(|a| a.namespace_uri.is_empty() && a.local_name == "id")
                        .is_some_and(|a| a.value == id)
                {
                    return Some(child);
                }
                if let Some(found) = find(document, child, id) {
                    return Some(found);
                }
            }
            None
        }
        find(self, self.TreeScopeRoot(element), id)
    }
    pub fn GetShadowReferenceTargetOrSelf(&self, element: usize) -> Option<usize> {
        if !self.ShadowRootReferenceTargetEnabled() {
            return Some(element);
        }
        let Some(root) = self.ShadowRoot(element) else {
            return Some(element);
        };
        let Some(id) = self.ShadowReferenceTarget(root) else {
            return Some(element);
        };
        let target = self.FindElementByIdInTreeScope(root, id)?;
        self.GetShadowReferenceTargetOrSelf(target)
    }
    pub fn GetElementAttributeResolvingReferenceTarget(
        &self,
        element: usize,
        name: &str,
    ) -> Option<usize> {
        self.GetShadowReferenceTargetOrSelf(self.GetElementAttribute(element, name)?)
    }
}
