// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Retained DOM facts for PendingInvalidations and StyleInvalidator. Scheduling
// and traversal are performed by the translated engines, never by this adapter.
use crate::invalidation::{
    invalidation_dom::{InvalidationDom, StyleChangeType},
    invalidation_set::InvalidationElement,
};
use dom::{persistent_document::DOMNodeType, Document};
use foundation::AtomicString;
use layoutng_style::style::computed_style::ComputedStyle;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

pub(crate) struct ElementFacts {
    name: AtomicString,
    id: Option<AtomicString>,
    classes: Vec<AtomicString>,
    attributes: Vec<AtomicString>,
    shadow_pseudo: AtomicString,
    has_part: bool,
    style: *const ComputedStyle,
}
impl InvalidationElement for ElementFacts {
    fn LocalNameForSelectorMatching(&self) -> &AtomicString {
        &self.name
    }
    fn IdForStyleResolution(&self) -> Option<&AtomicString> {
        self.id.as_ref()
    }
    fn ClassNames(&self) -> &[AtomicString] {
        &self.classes
    }
    fn ShadowPseudoId(&self) -> &AtomicString {
        &self.shadow_pseudo
    }
    fn HasAttributes(&self) -> bool {
        !self.attributes.is_empty()
    }
    fn HasAttributeIgnoringNamespace(&self, name: &AtomicString) -> bool {
        self.attributes.contains(name)
    }
    fn HasPart(&self) -> bool {
        self.has_part
    }
    fn HasComputedStyle(&self) -> bool {
        !self.style.is_null()
    }
    fn HasSiblingFunctions(&self) -> bool {
        unsafe { self.style.as_ref() }.is_some_and(|style| style.HasSiblingFunctions())
    }
    fn PseudoElementStylesDependOnSiblingFunctions(&self) -> bool {
        unreachable!("nth dependency queries are not admitted on the descendant user-action path")
    }
    fn ContainerQueryDependsOnTreeCounting(&self) -> bool {
        unreachable!("this adapter admits only descendant user-action sets, not nth sets")
    }
}

// A stable fact keeps its document borrow: the computed-style pointer below
// remains owned by that immutable document. The Rc never crosses this view.
pub(crate) struct ElementFactsRef<'a> {
    facts: Rc<ElementFacts>,
    document: PhantomData<&'a Document>,
}
impl Deref for ElementFactsRef<'_> {
    type Target = ElementFacts;
    fn deref(&self) -> &ElementFacts {
        &self.facts
    }
}

pub(crate) struct PersistentInvalidationDom<'a> {
    document: &'a Document,
    elements: RefCell<HashMap<usize, Rc<ElementFacts>>>,
    pub(crate) changes: HashMap<usize, StyleChangeType>,
    needs_invalidation: HashSet<usize>,
    child_needs_invalidation: HashSet<usize>,
}
impl<'a> PersistentInvalidationDom<'a> {
    pub(crate) fn new(document: &'a Document) -> Self {
        // StyleInvalidator reads live Element facts only as it visits them in
        // Blink. Do not materialize the entire DOM arena for each scheduled
        // attribute/user-action invalidation; detached retained nodes are not
        // part of that traversal. Owned handles keep each visited fact stable
        // while the sparse map grows; no arena-sized slot array is needed.
        let elements = RefCell::new(HashMap::new());
        Self {
            document,
            elements,
            changes: HashMap::new(),
            needs_invalidation: HashSet::new(),
            child_needs_invalidation: HashSet::new(),
        }
    }
    fn next(&self, node: usize) -> Option<usize> {
        let siblings = self
            .document
            .Node(self.document.Node(node).Parent()?)
            .Children();
        siblings
            .get(siblings.iter().position(|&n| n == node)? + 1)
            .copied()
    }
}
impl InvalidationDom for PersistentInvalidationDom<'_> {
    type Node = usize;
    type Element = ElementFacts;
    type ElementRef<'a>
        = ElementFactsRef<'a>
    where
        Self: 'a;
    fn element(&self, node: &usize) -> Option<ElementFactsRef<'_>> {
        let index = *node;
        let node = self.document.Node(index);
        if node.Type() != DOMNodeType::kElement {
            return None;
        }
        if let Some(element) = self.elements.borrow().get(&index) {
            return Some(ElementFactsRef {
                facts: element.clone(),
                document: PhantomData,
            });
        }
        let element = Rc::new({
            let _trace = browser_tracing::span("style", "StyleInvalidation.ElementFacts");
            let quirks = self.document.GetCompatibilityMode()
                == dom::persistent_document::CompatibilityMode::kQuirks;
            let atom = |s: &str| {
                let atom = AtomicString::from_str(s);
                if quirks {
                    atom.ToAsciiLower()
                } else {
                    atom
                }
            };
            ElementFacts {
                name: AtomicString::from_str(node.Name()),
                id: node.FindAttribute("id").map(|a| atom(&a.value)),
                classes: node
                    .FindAttribute("class")
                    .map(|a| a.value.split_ascii_whitespace().map(atom).collect())
                    .unwrap_or_default(),
                attributes: node
                    .Attributes()
                    .iter()
                    .map(|a| AtomicString::from_str(&a.local_name))
                    .collect(),
                shadow_pseudo: AtomicString::from_str(
                    self.document.ShadowPseudoId(index).unwrap_or_default(),
                ),
                has_part: node
                    .FindAttribute("part")
                    .is_some_and(|a| a.value.split_ascii_whitespace().next().is_some()),
                style: self
                    .document
                    .ResolvedStyleFor(index)
                    .map_or(std::ptr::null(), |s| s.native_style.Get()),
            }
        });
        self.elements.borrow_mut().insert(index, element.clone());
        Some(ElementFactsRef {
            facts: element,
            document: PhantomData,
        })
    }
    fn is_document(&self, node: &usize) -> bool {
        self.document.Node(*node).Type() == DOMNodeType::kDocument
    }
    fn shadow_host(&self, node: &usize) -> Option<usize> {
        self.document.ShadowHost(*node)
    }
    fn parent_node(&self, node: &usize) -> Option<usize> {
        self.document.Node(*node).Parent()
    }
    fn parent_or_shadow_host_node(&self, node: &usize) -> Option<usize> {
        self.parent_node(node).or_else(|| self.shadow_host(node))
    }
    fn next_sibling(&self, node: &usize) -> Option<usize> {
        self.next(*node)
    }
    fn next_element_sibling(&self, node: &usize) -> Option<usize> {
        let mut next = self.next(*node);
        while let Some(n) = next {
            if self.document.Node(n).Type() == DOMNodeType::kElement {
                return Some(n);
            }
            next = self.next(n);
        }
        None
    }
    fn element_children(&self, node: &usize) -> Vec<usize> {
        self.document
            .Node(*node)
            .Children()
            .iter()
            .copied()
            .filter(|n| self.document.Node(*n).Type() == DOMNodeType::kElement)
            .collect()
    }
    fn shadow_root(&self, node: &usize) -> Option<usize> {
        self.document.ShadowRoot(*node)
    }
    fn document_element(&self, node: &usize) -> Option<usize> {
        self.element_children(node).first().copied()
    }
    fn style_change_type(&self, node: &usize) -> StyleChangeType {
        self.changes.get(node).copied().unwrap_or_default()
    }
    fn set_needs_style_recalc(&mut self, node: &usize, change: StyleChangeType) {
        let value = self.changes.entry(*node).or_default();
        *value = (*value).max(change);
    }
    fn needs_style_recalc(&self, node: &usize) -> bool {
        self.style_change_type(node) != StyleChangeType::NoStyleChange
    }
    fn needs_style_invalidation(&self, node: &usize) -> bool {
        self.needs_invalidation.contains(node)
    }
    fn child_needs_style_invalidation(&self, node: &usize) -> bool {
        self.child_needs_invalidation.contains(node)
    }
    fn set_needs_style_invalidation(&mut self, node: &usize) {
        self.needs_invalidation.insert(*node);
        let mut ancestor = self.parent_or_shadow_host_node(node);
        while let Some(n) = ancestor {
            self.child_needs_invalidation.insert(n);
            ancestor = self.parent_or_shadow_host_node(&n);
        }
    }
    fn clear_needs_style_invalidation(&mut self, node: &usize) {
        self.needs_invalidation.remove(node);
    }
    fn clear_child_needs_style_invalidation(&mut self, node: &usize) {
        self.child_needs_invalidation.remove(node);
    }
    fn is_html_slot(&self, node: &usize) -> bool {
        self.document.IsSlotSupportingAssignment(*node)
    }
    fn flattened_assigned_nodes(&self, node: &usize) -> Vec<usize> {
        self.document.FlattenedAssignedNodes(*node)
    }
    fn possibly_schedule_nth_pseudo_invalidations(&mut self, _: &usize) {
        unreachable!("nth and sibling invalidation are retained on the owner's separate path")
    }
}
