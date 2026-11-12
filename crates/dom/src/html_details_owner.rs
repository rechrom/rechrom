//! Canonical HTMLDetailsElement shadow/slot and label-related HTML facts.
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
#![allow(non_snake_case)]
use super::{
    DOMAttribute, DOMNamespace, DOMNodeType, DOMShadowRootMode, DOMSlotAssignmentMode,
    PersistentDocument,
};

#[derive(Clone, Debug, Default)]
pub(super) struct HTMLDetailsElementData {
    summary_slot: Option<usize>,
    content_slot: Option<usize>,
    fallback_summary: Option<usize>,
}
impl PersistentDocument {
    /// Install the embedding's locale and source lifecycle before parser feed.
    /// Subsequent details allocation uses this document-owned locale value.
    pub fn InstallDetailsSummaryLabel(&mut self, label: String) {
        self.details_summary_label = Some(label);
        let details = self
            .arena
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                matches!(
                    node.class,
                    super::editing_node::DOMNodeClass::HTMLDetailsElement(_)
                )
                .then_some(index)
            })
            .collect::<Vec<_>>();
        assert!(
            details.is_empty(),
            "bind concrete HTML lifecycle owners before parser feed allocates details elements"
        );
    }
    // html_details_element.cc:72-122.
    pub fn EnsureDetailsUserAgentShadowRoot(&mut self, element: usize) {
        let data = match &self.arena[element].class {
            super::editing_node::DOMNodeClass::HTMLDetailsElement(data) => data,
            _ => return,
        };
        if data.summary_slot.is_some() {
            return;
        }
        let label = self.details_summary_label.clone().expect("install the document's localized details label before constructing details shadow content");
        let root = self.ShadowRoot(element).unwrap_or_else(|| {
            self.AttachShadowRootWithSlotAssignment(
                element,
                DOMShadowRootMode::kUserAgent,
                DOMSlotAssignmentMode::kManual,
            )
        });
        assert_eq!(
            self.ShadowRootMode(root),
            Some(DOMShadowRootMode::kUserAgent)
        );
        let summary = self.CreateElementDefault(DOMNamespace::kHTML, "summary".into());
        let text = self.CreateText(label);
        self.AppendChild(summary, text);
        let summary_slot = self.CreateElementDefault(DOMNamespace::kHTML, "slot".into());
        self.SetAttribute(
            summary_slot,
            DOMAttribute {
                local_name: "id".into(),
                value: "details-summary".into(),
                ..Default::default()
            },
        );
        self.AppendChild(summary_slot, summary);
        self.AppendChild(root, summary_slot);
        let content_slot = self.CreateElementDefault(DOMNamespace::kHTML, "slot".into());
        self.SetAttribute(
            content_slot,
            DOMAttribute {
                local_name: "id".into(),
                value: "details-content".into(),
                ..Default::default()
            },
        );
        self.SetUAShadowPseudoElementIdentity(element, content_slot, "details-content");
        self.SetAttribute(
            content_slot,
            DOMAttribute {
                local_name: "style".into(),
                value: if self.Node(element).FindAttribute("open").is_some() {
                    "display:block"
                } else {
                    "content-visibility:hidden;display:block"
                }
                .into(),
                ..Default::default()
            },
        );
        self.SetIsDetailsSlotElement(content_slot);
        self.AppendChild(root, content_slot);
        let style = self.CreateElementDefault(DOMNamespace::kHTML, "style".into());
        let text = self.CreateText(":host summary{display:list-item;counter-increment:list-item 0;list-style:disclosure-closed inside}:host([open]) summary{list-style-type:disclosure-open}".into());
        self.AppendChild(style, text);
        self.AppendChild(root, style);
        let super::editing_node::DOMNodeClass::HTMLDetailsElement(data) =
            &mut self.arena[element].class
        else {
            unreachable!()
        };
        data.summary_slot = Some(summary_slot);
        data.content_slot = Some(content_slot);
        data.fallback_summary = Some(summary);
        self.AssignDetailsSlots(element);
    }
    // html_details_element.cc:124-133,135-153.
    pub fn DetailsMainSummary(&self, element: usize) -> Option<usize> {
        if let Some(summary) = self
            .Node(element)
            .Children()
            .iter()
            .copied()
            .find(|&node| self.Node(node).IsHTMLElement("summary"))
        {
            return Some(summary);
        }
        match &self.arena[element].class {
            super::editing_node::DOMNodeClass::HTMLDetailsElement(data) => data.fallback_summary,
            _ => None,
        }
    }
    pub fn IsDetailsMainSummary(&self, summary: usize) -> bool {
        if !self.Node(summary).IsHTMLElement("summary") {
            return false;
        }
        if let Some(parent) = self
            .Node(summary)
            .Parent()
            .filter(|&parent| self.Node(parent).IsHTMLElement("details"))
        {
            return self.DetailsMainSummary(parent) == Some(summary);
        }
        self.OwnerShadowHost(summary)
            .filter(|&host| self.Node(host).IsHTMLElement("details"))
            .is_some_and(|host| self.DetailsMainSummary(host) == Some(summary))
    }
    pub(super) fn AssignDetailsSlots(&mut self, element: usize) {
        let (summary_slot, content_slot) = match &self.arena[element].class {
            super::editing_node::DOMNodeClass::HTMLDetailsElement(data) => {
                match (data.summary_slot, data.content_slot) {
                    (Some(a), Some(b)) => (a, b),
                    _ => return,
                }
            }
            _ => return,
        };
        let mut summary_nodes = Vec::new();
        let mut content_nodes = Vec::new();
        for &child in self.Node(element).Children() {
            if !matches!(
                self.Node(child).Type(),
                DOMNodeType::kElement | DOMNodeType::kText
            ) {
                continue;
            }
            if summary_nodes.is_empty() && self.Node(child).IsHTMLElement("summary") {
                summary_nodes.push(child);
            } else {
                content_nodes.push(child);
            }
        }
        self.AssignSlotNodes(summary_slot, summary_nodes);
        self.AssignSlotNodes(content_slot, content_nodes);
    }
    pub fn DetailsContentSlot(&self, element: usize) -> Option<usize> {
        match &self.arena[element].class {
            super::editing_node::DOMNodeClass::HTMLDetailsElement(data) => data.content_slot,
            _ => None,
        }
    }
    pub fn SetDetailsToggleDuringParsing(&mut self, value: bool) {
        self.details_toggle_during_parsing = value;
    }
    pub fn DetailsToggleDuringParsing(&self) -> bool {
        self.details_toggle_during_parsing
    }
    // NodeRareData/DisplayLockContext::SetIsDetailsSlotElement. This is the
    // original context flag, not a fabricated actual lock/visibility result.
    pub fn SetIsDetailsSlotElement(&mut self, slot: usize) {
        self.EnsureDisplayLockContext(slot).is_details_slot_element = true;
    }
    pub fn IsDetailsSlotElement(&self, slot: usize) -> bool {
        self.arena[slot]
            .display_lock_context
            .borrow()
            .as_ref()
            .is_some_and(|context| context.is_details_slot_element)
    }
    /// Source virtual HTML interactive-content facts for classes whose default
    /// algorithms have no retained interaction state.
    pub fn IsNativeHTMLInteractiveContent(&self, element: usize) -> bool {
        use super::editing_node::DOMNodeClass as C;
        let node = self.Node(element);
        match &node.class {
            // HTMLAnchorElementBase::IsInteractiveContent cc:535-537 also
            // owns HTMLAreaElement; href presence is its native IsLink fact.
            C::HTMLAnchorElement { .. } | C::HTMLAreaElement { .. } => {
                node.FindAttribute("href").is_some()
            }
            C::HTMLInputElement | C::MediaControlInput { .. } => {
                self.InputType(element) != Some(super::DOMInputType::kHidden)
            }
            C::HTMLSelectElement(_) | C::HTMLTextAreaElement | C::HTMLDetailsElement(_) => true,
            C::HTMLMediaElement { .. } => node.FindAttribute("controls").is_some(),
            C::HTMLImageElement => node.FindAttribute("usemap").is_some(),
            C::HTMLFrameElementBase => node.IsHTMLElement("iframe"),
            C::HTMLPlugInElement => true,
            _ => false,
        }
    }
    // Document::ProcessBaseElement, document.cc:5155-5179,5232-5249.
    // Retain the derived value on the canonical document. The DOM's mutation
    // revision invalidates this query, so repeated default actions do not walk.
    pub fn HTMLDocumentBaseTarget(&self) -> String {
        if let Some((revision, value)) = self.html_base_target.borrow().as_ref() {
            if *revision == self.collection_membership_revision && *revision != u64::MAX {
                return value.clone();
            }
        }
        let mut pending = vec![self.Root()];
        let mut target = String::new();
        while let Some(node) = pending.pop() {
            let element = self.Node(node);
            if element.IsHTMLElement("base") {
                if let Some(value) = element.FindAttribute("target") {
                    target = value.value.clone();
                    break;
                }
            }
            pending.extend(element.Children().iter().rev().copied());
        }
        *self.html_base_target.borrow_mut() =
            Some((self.collection_membership_revision, target.clone()));
        target
    }
    // Element::GetShadowReferenceTarget, element.cc:1420-1439.
    pub fn SetShadowReferenceTarget(&mut self, root: usize, target: Option<String>) {
        if !self.shadow_root_reference_target_enabled {
            return;
        }
        assert!(
            self.ShadowHost(root).is_some(),
            "reference target belongs to a ShadowRoot"
        );
        if let Some(target) = target {
            self.shadow_reference_targets.insert(root, target);
        } else {
            self.shadow_reference_targets.remove(&root);
        }
    }
    /// TreeScope::Retarget, tree_scope.cc:662-689. Only ancestor scope
    /// identities are collected, borrowing the canonical DOM throughout.
    pub fn RetargetEventSource(&self, target: usize, context: usize) -> usize {
        let target_scope = self.TreeScopeRoot(target);
        if self.ShadowHost(target_scope).is_none() {
            return target;
        }
        let ancestors = |first| {
            let mut scopes = Vec::new();
            let mut current = Some(first);
            while let Some(scope) = current {
                scopes.push(scope);
                current = self.ShadowHost(scope).map(|host| self.TreeScopeRoot(host));
            }
            scopes
        };
        let target_scopes = ancestors(target_scope);
        let context_scopes = ancestors(self.TreeScopeRoot(context));
        let mut remaining = target_scopes.len();
        let mut context_remaining = context_scopes.len();
        while remaining > 0
            && context_remaining > 0
            && target_scopes[remaining - 1] == context_scopes[context_remaining - 1]
        {
            remaining -= 1;
            context_remaining -= 1;
        }
        if remaining == 0 {
            target
        } else {
            self.ShadowHost(target_scopes[remaining - 1])
                .expect("different target scope is a shadow root")
        }
    }
    pub fn ShadowReferenceTarget(&self, root: usize) -> Option<&str> {
        self.shadow_reference_targets.get(&root).map(String::as_str)
    }
    pub fn SetShadowRootReferenceTargetEnabled(&mut self, enabled: bool) {
        self.shadow_root_reference_target_enabled = enabled;
    }
    pub fn ShadowRootReferenceTargetEnabled(&self) -> bool {
        self.shadow_root_reference_target_enabled
    }
    pub fn ShadowReferenceTargetElement(&self, element: usize) -> Option<usize> {
        let root = self.ShadowRoot(element)?;
        let id = self.shadow_reference_targets.get(&root)?;
        let target = self.FindElementByIdInTreeScope(root, id)?;
        Some(self.ShadowReferenceTargetElement(target).unwrap_or(target))
    }
    /// ListedElement::RecalcWillValidate for the Button base. The button's
    /// translated subclass still owns its submit/reset/button type branch.
    // listed_element.cc:364-380; listed_element.h:225.
    pub fn ButtonBaseRecalcWillValidate(&self, element: usize) -> bool {
        assert!(self.Node(element).IsHTMLElement("button"));
        let mut parent = self.Node(element).Parent();
        while let Some(node) = parent {
            if self.Node(node).IsHTMLElement("datalist") {
                return false;
            }
            parent = self.Node(node).Parent();
        }
        !self.IsDisabledFormControl(element)
        // Button's ReadOnlyPreventsConstraintValidation is false at this
        // revision, and ElementSpecificReadOnlyConstraintValidation is stable.
    }
}
