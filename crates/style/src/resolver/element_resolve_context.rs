//! Source: /Users/zhenghuaiyu/chromium/src/third_party/blink/renderer/core/css/resolver/
//! Chromium commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Ledger (physical / effective / mapped / omitted / pending):
//! element_resolve_context.h: 102 / 38 / 33 / 5 / 0.
//! element_resolve_context.cc: 116 / 43 / 41 / 2 / 0.
//! Effective excludes copyright/comments, blank/preprocessor/include/namespace
//! lines and lines containing only brackets/punctuation. Header omissions are
//! forward declarations 34-35, STACK_ALLOCATED 40 and access labels 42,76;
//! cc omissions are pure debug checks 66-67. All remaining declarations and
//! production behavior, including cc:49-114, map to this module. Required trait
//! operations denote DOM/traversal/probe/link owners, never matching algorithms.
#![allow(non_snake_case)]
use super::style_resolver_state::{ComputedStyleHandle, StyleResolverStateBackend};
use crate::css_selector::PseudoType;
use foundation::EInsideLink;
use std::rc::Rc;

/// Calls owned by DOM, LayoutTreeBuilderTraversal, probes and VisitedLinkState.
/// The associated Element and computed styles are the existing resolver types.
pub trait ElementResolveContextBackend: StyleResolverStateBackend {
    fn ContextIsPseudoElement(&self, element: &Self::Element) -> bool;
    fn ContextIsViewTransitionPseudoElement(&self, element: &Self::Element) -> bool;
    fn ContextUltimateOriginatingElement(&self, element: &Self::Element) -> Rc<Self::Element>;
    fn ContextDOMParentElement(&self, element: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ContextTraversalParentElement(&self, element: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ContextTraversalLayoutParentElement(
        &self,
        element: &Self::Element,
    ) -> Option<Rc<Self::Element>>;
    fn ContextDocumentIsActive(&self, element: &Self::Element) -> bool;
    fn ContextForcePseudoState(&self, element: &Self::Element, pseudo: PseudoType) -> bool;
    fn ContextDetermineLinkState(&self, element: &Self::Element) -> EInsideLink;
    fn ContextDocumentElement(&self, element: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ContextComputedStyle(&self, element: &Self::Element) -> Option<ComputedStyleHandle>;
}

pub const kMaxPseudoElementsNesting: usize = 5;
pub type PseudoElementAncestors<B> =
    [Option<Rc<<B as StyleResolverStateBackend>::Element>>; kMaxPseudoElementsNesting];

/// A per-resolution snapshot of element identities. Parent styles are read at
/// access time, just as in the source; the root style is captured on creation.
pub struct ElementResolveContext<B: ElementResolveContextBackend> {
    backend: Rc<B>,
    element_: Rc<B::Element>,
    ultimate_originating_element_: Rc<B::Element>,
    pseudo_element_: Option<Rc<B::Element>>,
    parent_element_: Option<Rc<B::Element>>,
    layout_parent_: Option<Rc<B::Element>>,
    root_element_style_: Option<ComputedStyleHandle>,
    element_link_state_: EInsideLink,
    pseudo_element_ancestors_size_: usize,
    pseudo_element_ancestors_: PseudoElementAncestors<B>,
}
impl<B: ElementResolveContextBackend> ElementResolveContext<B> {
    pub fn new(backend: Rc<B>, element: Rc<B::Element>) -> Self {
        let is_pseudo = backend.ContextIsPseudoElement(&element);
        let ultimate_originating_element_ = if is_pseudo {
            backend.ContextUltimateOriginatingElement(&element)
        } else {
            element.clone()
        };
        let element_link_state_ = if !backend.ContextDocumentIsActive(&element) {
            EInsideLink::kNotInsideLink
        } else if backend.ContextForcePseudoState(&element, PseudoType::kPseudoVisited) {
            EInsideLink::kInsideVisitedLink
        } else if backend.ContextForcePseudoState(&element, PseudoType::kPseudoLink) {
            EInsideLink::kInsideUnvisitedLink
        } else {
            backend.ContextDetermineLinkState(&element)
        };
        let mut result = Self {
            backend,
            element_: element.clone(),
            ultimate_originating_element_,
            pseudo_element_: is_pseudo.then_some(element),
            parent_element_: None,
            layout_parent_: None,
            root_element_style_: None,
            element_link_state_,
            pseudo_element_ancestors_size_: kMaxPseudoElementsNesting,
            pseudo_element_ancestors_: std::array::from_fn(|_| None),
        };
        result.BuildPseudoElementAncestors();
        result.parent_element_ = result
            .backend
            .ContextTraversalParentElement(&result.element_);
        result.layout_parent_ = result
            .backend
            .ContextTraversalLayoutParentElement(&result.element_);
        result.root_element_style_ = result
            .backend
            .ContextDocumentElement(&result.element_)
            .filter(|root| !Rc::ptr_eq(root, &result.element_))
            .and_then(|root| result.backend.ContextComputedStyle(&root));
        result
    }
    fn BuildPseudoElementAncestors(&mut self) {
        let mut element = self.element_.clone();
        if !self.backend.ContextIsPseudoElement(&element) {
            return;
        }
        if self.backend.ContextIsViewTransitionPseudoElement(&element) {
            self.pseudo_element_ancestors_size_ -= 1;
            self.pseudo_element_ancestors_[self.pseudo_element_ancestors_size_] = Some(element);
            return;
        }
        while self.backend.ContextIsPseudoElement(&element) {
            assert!(
                self.pseudo_element_ancestors_size_ > 0,
                "pseudo-element nesting exceeds source limit"
            );
            self.pseudo_element_ancestors_size_ -= 1;
            self.pseudo_element_ancestors_[self.pseudo_element_ancestors_size_] =
                Some(element.clone());
            element = self
                .backend
                .ContextDOMParentElement(&element)
                .expect("pseudo-element has originating parent");
        }
    }
    pub fn GetElement(&self) -> &Rc<B::Element> {
        &self.element_
    }
    pub fn GetUltimateOriginatingElementOrSelf(&self) -> &Rc<B::Element> {
        &self.ultimate_originating_element_
    }
    pub fn GetPseudoElement(&self) -> Option<&Rc<B::Element>> {
        self.pseudo_element_.as_ref()
    }
    pub fn ParentElement(&self) -> Option<&Rc<B::Element>> {
        self.parent_element_.as_ref()
    }
    pub fn LayoutParentElement(&self) -> Option<&Rc<B::Element>> {
        self.layout_parent_.as_ref()
    }
    pub fn RootElementStyle(&self) -> Option<&ComputedStyleHandle> {
        self.root_element_style_.as_ref()
    }
    pub fn ParentStyle(&self) -> Option<ComputedStyleHandle> {
        self.parent_element_
            .as_ref()
            .and_then(|element| self.backend.ContextComputedStyle(element))
    }
    pub fn LayoutParentStyle(&self) -> Option<ComputedStyleHandle> {
        self.layout_parent_
            .as_ref()
            .and_then(|element| self.backend.ContextComputedStyle(element))
    }
    pub fn ElementLinkState(&self) -> EInsideLink {
        self.element_link_state_
    }
    pub fn GetPseudoElementAncestors(&self) -> &[Option<Rc<B::Element>>] {
        &self.pseudo_element_ancestors_[self.pseudo_element_ancestors_size_..]
    }
    pub fn PseudoElementAncestorsSize(&self) -> usize {
        kMaxPseudoElementsNesting - self.pseudo_element_ancestors_size_
    }
}
