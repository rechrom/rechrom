/*
 * Copyright (C) 2011, 2013 Apple Inc. All rights reserved.
 * Copyright (C) 2014 Samsung Electronics. All rights reserved.
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 * 1. Redistributions of source code must retain the above copyright notice,
 *    this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 * THIS SOFTWARE IS PROVIDED BY APPLE AND ITS CONTRIBUTORS "AS IS" AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL APPLE OR ITS CONTRIBUTORS BE LIABLE FOR ANY
 * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
 * ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
 * THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// cpp: third_party/blink/renderer/core/css/selector_query.h
// cpp: third_party/blink/renderer/core/css/selector_query.cc
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source ledger (physical / effective / mapped / omitted / pending):
//   selector_query.h:   413 / 119 / 104 / 15 / 0.
//   selector_query.cc: 1074 / 613 / 599 / 14 / 0.
// Effective excludes copyright/comments/blank/preprocessor/include/namespace
// lines and lines consisting only of braces/parentheses/semicolons.
// Header omissions: 43-49,119,121-122,164,166,401,405,407 (forward declarations,
// C++ access/deleted-copy boilerplate and GC Trace). Implementation omissions:
// 170,172-177,190,192-197 (expensive debug-only fast/slow differential checks).
// Source output traits use const FIRST plus Vec<Rc<Element>> (zero or one
// entry for FIRST); their IsEmpty helpers map to slice::is_empty. Source
// CHECK/DCHECK invariants and debug query statistics are retained. Compound
// pointers are stable indices into the same compound array. Query errors carry
// the exact source SyntaxError message for the bindings layer to throw.
// DOM/tree-scope/lazy-attribute registries and query cache scopes are required
// typed collaborators. All compound transitions, traversal, filtering, ID
// anchoring, selector rechecking and parsed-query cache policy live here.
#![allow(non_snake_case)]

use crate::css_selector::{
    AttributeMatchType, CSSSelector, CSSSelectorComplex, MatchNth, MatchType, PseudoType,
    QualifiedName, RelationType,
};
use crate::css_selector_list::CSSSelectorList;
use crate::parser::css_nesting_type::CSSNestingType;
use crate::parser::css_parser::{CSSParser, CSSParserBackend};
use crate::parser::css_parser_context::{CSSParserContext, DocumentSnapshot};
use crate::selector_checker::{
    Mode, SelectorChecker, SelectorCheckerBackend, SelectorCheckingContext,
};
use crate::selector_filter::{
    AttributesToExcludeHashesFor, CollectSingleSelectorIdentifierHashes, SelectorQueryFilterBackend,
};
use foundation::{AtomicString, String};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

/// Container identity must be stable (the same DOM node always returns the
/// same Rc). Traversal remains within a tree scope, without entering shadows.
pub trait SelectorQueryBackend: SelectorCheckerBackend + SelectorQueryFilterBackend {
    type QueryNthIndexCache;
    fn QueryParentNode(&self, node: &Self::ContainerNode) -> Option<Rc<Self::ContainerNode>>;
    fn QueryPreviousSiblingNode(
        &self,
        node: &Self::ContainerNode,
    ) -> Option<Rc<Self::ContainerNode>>;
    fn QueryFirstChildElement(&self, node: &Self::ContainerNode) -> Option<Rc<Self::Element>>;
    fn QueryNextSiblingElement(&self, node: &Self::ContainerNode) -> Option<Rc<Self::Element>>;
    fn QueryIsShadowRoot(&self, node: &Self::ContainerNode) -> bool;
    fn QueryIsHTMLDocument(&self, node: &Self::ContainerNode) -> bool;
    fn QueryInQuirksMode(&self, node: &Self::ContainerNode) -> bool;
    fn QueryIsInTreeScope(&self, node: &Self::ContainerNode) -> bool;
    fn QueryTreeScope(&self, node: &Self::ContainerNode) -> Rc<Self::TreeScope>;
    fn QuerySubtreeBloomFilter(&self, element: &Self::Element) -> u32;
    fn QueryContainsMultipleElementsWithId(
        &self,
        scope: &Self::TreeScope,
        id: &AtomicString,
    ) -> bool;
    /// Returns elements in tree order, as TreeScope::GetAllElementsById does.
    fn QueryAllElementsById(
        &self,
        scope: &Self::TreeScope,
        id: &AtomicString,
    ) -> Vec<Rc<Self::Element>>;
    fn QueryElementById(
        &self,
        scope: &Self::TreeScope,
        id: &AtomicString,
    ) -> Option<Rc<Self::Element>>;
    fn EnterQueryHasCacheScope(&self, root: &Self::ContainerNode) -> Self::HasCacheScope;
    fn EnterQueryNthIndexCache(&self, root: &Self::ContainerNode) -> Self::QueryNthIndexCache;
    fn ExitQueryNthIndexCache(&self, cache: Self::QueryNthIndexCache);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QueryStats {
    pub elements_seen: u32,
    pub fast_id_roots: u32,
    pub check_id: u32,
    pub check_tag: u32,
    pub check_class: u32,
    pub check_attr: u32,
    pub check_nth_child: u32,
    pub recheck_selector: u32,
    pub skipped_subtree: u32,
    pub slow_scan: u32,
}
thread_local! { static QUERY_STATS: RefCell<QueryStats> = RefCell::new(QueryStats::default()); }
macro_rules! stat {
    ($field:ident) => {
        #[cfg(debug_assertions)]
        QUERY_STATS.with(|s| s.borrow_mut().$field += 1);
    };
}
impl std::fmt::Display for QueryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == Self::default() {
            return f.write_str("(empty)");
        }
        for (name, value) in [
            ("elements_seen", self.elements_seen),
            ("fast_id_roots", self.fast_id_roots),
            ("check_id", self.check_id),
            ("check_tag", self.check_tag),
            ("check_class", self.check_class),
            ("check_attr", self.check_attr),
            ("check_nth_child", self.check_nth_child),
            ("recheck_selector", self.recheck_selector),
            ("slow_scan", self.slow_scan),
            ("skipped_subtree", self.skipped_subtree),
        ] {
            if value != 0 {
                write!(f, ".{name} = {value}, ")?;
            }
        }
        Ok(())
    }
}

struct QueryCacheScope<'a, B: SelectorQueryBackend> {
    backend: &'a B,
    has: Option<B::HasCacheScope>,
    nth: Option<B::QueryNthIndexCache>,
}
impl<'a, B: SelectorQueryBackend> QueryCacheScope<'a, B> {
    fn new(backend: &'a B, root: &B::ContainerNode, nth: bool) -> Self {
        let has = Some(backend.EnterQueryHasCacheScope(root));
        let nth = nth.then(|| backend.EnterQueryNthIndexCache(root));
        Self { backend, has, nth }
    }
}
impl<B: SelectorQueryBackend> Drop for QueryCacheScope<'_, B> {
    fn drop(&mut self) {
        if let Some(nth) = self.nth.take() {
            self.backend.ExitQueryNthIndexCache(nth);
        }
        self.backend.ExitHasCacheScope(self.has.take().unwrap());
    }
}

#[derive(Default)]
struct Compound {
    id_needed: AtomicString,
    tag_needed: Option<QualifiedName>,
    class_needed: AtomicString,
    attr_needed: Option<QualifiedName>,
    attr_value: AtomicString,
    attr_case_insensitive: Cell<bool>,
    needs_synchronize_attribute: Cell<bool>,
    match_type_case_insensitive: bool,
    legacy_case_insensitive: bool,
    has_nth_child: bool,
    nth_a: i32,
    nth_b: i32,
    selector_filter: u32,
    skip_for_shadow_root: bool,
    next_compound_is_horizontal: bool,
    is_subject: bool,
    simple_traversal_from_here: bool,
    valid_for_progress: Cell<bool>,
    seen_before_root: Cell<bool>,
    next_compound_for_children_on_match: usize,
    next_compound_for_siblings_on_match: usize,
    next_compound_for_children_on_mismatch: usize,
}
impl Compound {
    fn new() -> Self {
        Self {
            skip_for_shadow_root: true,
            valid_for_progress: Cell::new(true),
            ..Self::default()
        }
    }
}

pub struct SelectorQuery {
    selector_list_: Rc<CSSSelectorList>,
    selector_start_offsets_: Vec<usize>,
    compounds_: Vec<Compound>,
    need_full_check_: bool,
    last_compound_with_id_selector_: Option<usize>,
}
impl SelectorQuery {
    const kUnknownSiblingIndex: u32 = 0x80000000;
    pub fn LastQueryStats() -> QueryStats {
        QUERY_STATS.with(|s| *s.borrow())
    }
    fn ResetStats() {
        QUERY_STATS.with(|s| *s.borrow_mut() = QueryStats::default());
    }
    pub fn new<B: SelectorQueryFilterBackend>(
        selector_list: Rc<CSSSelectorList>,
        backend: &B,
    ) -> Self {
        assert!(
            selector_list.IsValid(),
            "SelectorQuery requires a parsed valid selector list"
        );
        let selector_start_offsets = selector_list
            .ComplexSelectors()
            .filter(|s| !s.MatchesPseudoElement())
            .map(|s| selector_list.SelectorIndex(&s))
            .collect();
        let mut query = Self {
            selector_list_: selector_list,
            selector_start_offsets_: selector_start_offsets,
            compounds_: vec![],
            need_full_check_: false,
            last_compound_with_id_selector_: None,
        };
        if query.selector_start_offsets_.len() == 1 {
            query.BuildCompounds(backend);
        }
        query
    }
    fn OnlySelector(&self) -> CSSSelectorComplex<'_> {
        assert_eq!(self.selector_start_offsets_.len(), 1);
        self.selector_list_
            .ComplexAt(self.selector_start_offsets_[0])
    }
    fn BuildCompounds<B: SelectorQueryFilterBackend>(&mut self, backend: &B) {
        let mut current_compound = Compound::new();
        let mut level_filter = 0;
        self.compounds_.clear();
        self.need_full_check_ = false;
        self.last_compound_with_id_selector_ = None;
        // Iterate the actual CSSSelector array right-to-left, then reverse it.
        let selector_list = self.selector_list_.clone();
        for current in selector_list
            .ComplexAt(self.selector_start_offsets_[0])
            .SimpleSelectors()
        {
            match current.Match() {
                MatchType::kClass => {
                    if current_compound.class_needed.IsNull() {
                        current_compound.class_needed = (*current.Value()).clone();
                    } else {
                        self.need_full_check_ = true;
                    }
                }
                MatchType::kUniversalTag => {
                    if current.TagQName().NamespaceURI() != &AtomicString::from_str("*") {
                        self.need_full_check_ = true;
                    }
                }
                MatchType::kTag => {
                    if current_compound.tag_needed.is_none() {
                        current_compound.tag_needed = Some(current.TagQName().clone());
                        if current.TagQName().NamespaceURI() != &AtomicString::from_str("*") {
                            self.need_full_check_ = true;
                        }
                    } else {
                        self.need_full_check_ = true;
                    }
                }
                MatchType::kAttributeExact => {
                    if current_compound.attr_needed.is_none() {
                        current_compound.attr_needed = Some((*current.Attribute()).clone());
                        current_compound.attr_value = (*current.Value()).clone();
                        current_compound.match_type_case_insensitive =
                            current.AttributeMatch() == AttributeMatchType::kCaseInsensitive;
                        current_compound.legacy_case_insensitive =
                            current.LegacyCaseInsensitiveMatch();
                        if current_compound.legacy_case_insensitive
                            && !current_compound.match_type_case_insensitive
                        {
                            self.need_full_check_ = true;
                        }
                    } else {
                        self.need_full_check_ = true;
                    }
                }
                MatchType::kId => {
                    if current_compound.id_needed.IsNull() {
                        current_compound.id_needed = (*current.Value()).clone();
                    } else {
                        self.need_full_check_ = true;
                    }
                }
                MatchType::kPseudoClass => {
                    if current.GetPseudoType() == PseudoType::kPseudoNthChild
                        && current.SelectorList().is_none()
                        && !current_compound.has_nth_child
                    {
                        current_compound.has_nth_child = true;
                        current_compound.nth_a = current.NthAValue() as i32;
                        current_compound.nth_b = current.NthBValue() as i32;
                    } else if current.GetPseudoType() == PseudoType::kPseudoFirstChild
                        && !current_compound.has_nth_child
                    {
                        current_compound.has_nth_child = true;
                        current_compound.nth_a = 0;
                        current_compound.nth_b = 1;
                    } else {
                        self.need_full_check_ = true;
                    }
                }
                _ => self.need_full_check_ = true,
            }
            CollectSingleSelectorIdentifierHashes(
                current,
                AttributesToExcludeHashesFor::kExcludeAllLazilySynchronizedAttributes,
                &mut current_compound.selector_filter,
                backend,
            );
            level_filter |= current_compound.selector_filter;
            if !current.IsLastInComplexSelector()
                && current.Relation() != RelationType::kSubSelector
            {
                self.compounds_.push(current_compound);
                current_compound = Compound::new();
                if matches!(
                    current.Relation(),
                    RelationType::kDirectAdjacent | RelationType::kIndirectAdjacent
                ) {
                    current_compound.next_compound_is_horizontal = true;
                } else {
                    current_compound.selector_filter = level_filter;
                    level_filter = 0;
                }
                if matches!(
                    current.Relation(),
                    RelationType::kDirectAdjacent | RelationType::kChild
                ) {
                    self.need_full_check_ = true;
                }
            }
        }
        self.compounds_.push(current_compound);
        self.compounds_.reverse();
        for compound in &mut self.compounds_ {
            if !compound.id_needed.IsNull()
                || compound.tag_needed.is_some()
                || !compound.class_needed.IsNull()
                || compound.attr_needed.is_some()
            {
                compound.skip_for_shadow_root = false;
            }
        }
        let id_attr = QualifiedName::new(
            AtomicString::default(),
            AtomicString::from_str("id"),
            AtomicString::default(),
        );
        self.last_compound_with_id_selector_ = self.compounds_.iter().rposition(|c| {
            !c.id_needed.IsNull()
                || (c.attr_needed.as_ref() == Some(&id_attr)
                    && !c.attr_value.empty()
                    && !c.match_type_case_insensitive
                    && !c.legacy_case_insensitive)
        });
        for index in 0..self.compounds_.len() {
            let level_start = self.FindStartOfLevel(index);
            let last = index + 1 == self.compounds_.len();
            let compound = &mut self.compounds_[index];
            compound.next_compound_for_siblings_on_match = index;
            compound.next_compound_for_children_on_mismatch = level_start;
            compound.next_compound_for_children_on_match = level_start;
            if !last {
                if compound.next_compound_is_horizontal {
                    compound.next_compound_for_siblings_on_match = index + 1;
                } else {
                    compound.next_compound_for_children_on_match = index + 1;
                }
            }
        }
        let subject_index = self.compounds_.len() - 1;
        let subject = &mut self.compounds_[subject_index];
        subject.is_subject = true;
        subject.simple_traversal_from_here = subject.next_compound_for_children_on_mismatch
            == subject_index
            && !subject.has_nth_child;
    }
    fn FindStartOfLevel(&self, mut index: usize) -> usize {
        while index > 0 && self.compounds_[index - 1].next_compound_is_horizontal {
            index -= 1;
        }
        index
    }
    fn FillMissingData<B: SelectorQueryBackend>(&self, backend: &B, root: &B::ContainerNode) {
        let html = backend.QueryIsHTMLDocument(root);
        for c in &self.compounds_ {
            if let Some(attr) = &c.attr_needed {
                c.attr_case_insensitive
                    .set(c.match_type_case_insensitive || (c.legacy_case_insensitive && html));
                c.needs_synchronize_attribute
                    .set(backend.IsExcludedAttribute(
                    attr,
                    if html {
                        AttributesToExcludeHashesFor::kExcludeAllLazilySynchronizedAttributes
                    } else {
                        AttributesToExcludeHashesFor::kExcludeLowercaseLazilySynchronizedAttributes
                    },
                ));
            }
        }
    }
    fn MatchCompound<B: SelectorQueryBackend>(
        backend: &B,
        e: &B::Element,
        c: &Compound,
        sibling_index: u32,
        html: bool,
    ) -> bool {
        if !c.id_needed.IsNull() {
            stat!(check_id);
            if !backend.HasID(e) || backend.IdForStyleResolution(e) != c.id_needed {
                return false;
            }
        }
        if let Some(tag) = &c.tag_needed {
            stat!(check_tag);
            if *tag != QualifiedName::AnyQName()
                && backend.ElementLocalName(e) != *tag.LocalName()
                && !(!backend.IsHTMLElement(e)
                    && backend.IsInHTMLDocument(e)
                    && backend.ElementTagQName(e).LocalName().ToAsciiUpper()
                        == tag.LocalName().ToAsciiUpper())
            {
                return false;
            }
        }
        if !c.class_needed.IsNull() {
            stat!(check_class);
            if !backend.ClassNamesContain(e, &c.class_needed) {
                return false;
            }
        }
        if let Some(name) = &c.attr_needed {
            stat!(check_attr);
            if c.needs_synchronize_attribute.get() {
                backend.SynchronizeAttribute(e, name.LocalName());
            }
            let mut matched = false;
            for attr in backend.AttributesWithoutUpdate(e) {
                if !backend.AttributeMatchesName(&attr, name)
                    && (backend.IsHTMLElement(e)
                        || !html
                        || !backend.AttributeMatchesNameCaseInsensitive(&attr, name))
                {
                    continue;
                }
                let value = backend.AttributeValue(&attr);
                if !value.IsNull()
                    && (c.attr_value == value
                        || (c.attr_case_insensitive.get()
                            && c.attr_value.ToAsciiLower() == value.ToAsciiLower()))
                {
                    matched = true;
                    break;
                }
                if name.NamespaceURI() != &AtomicString::from_str("*") {
                    break;
                }
            }
            if !matched {
                return false;
            }
        }
        if c.has_nth_child && sibling_index & Self::kUnknownSiblingIndex == 0 {
            stat!(check_nth_child);
            if !MatchNth(c.nth_a, c.nth_b, sibling_index) {
                return false;
            }
        }
        true
    }
    fn SelectorMatches<B: SelectorQueryBackend>(
        &self,
        backend: &B,
        checker: &SelectorChecker<B>,
        selector: CSSSelectorComplex<'_>,
        element: Rc<B::Element>,
        root: Rc<B::ContainerNode>,
    ) -> bool {
        stat!(recheck_selector);
        let mut context = SelectorCheckingContext::new(element);
        context.selector = Some(selector);
        context.tree_scope = Some(backend.QueryTreeScope(&root));
        context.scope = Some(root);
        checker.MatchWithoutResult(&context)
    }
    fn SelectorListMatches<B: SelectorQueryBackend>(
        &self,
        backend: Rc<B>,
        root: Rc<B::ContainerNode>,
        element: Rc<B::Element>,
    ) -> bool {
        let checker = SelectorChecker::new(backend.clone(), Mode::kQueryingRules);
        self.selector_start_offsets_.iter().any(|&offset| {
            self.SelectorMatches(
                &*backend,
                &checker,
                self.selector_list_.ComplexAt(offset),
                element.clone(),
                root.clone(),
            )
        })
    }
    pub fn Matches<B: SelectorQueryBackend>(&self, backend: Rc<B>, target: Rc<B::Element>) -> bool {
        Self::ResetStats();
        let root = backend.ElementAsContainer(&target);
        let _scope = QueryCacheScope::new(&*backend, &root, false);
        if self.compounds_.len() == 1 && !self.need_full_check_ && !self.compounds_[0].has_nth_child
        {
            self.FillMissingData(&*backend, &root);
            Self::MatchCompound(
                &*backend,
                &target,
                &self.compounds_[0],
                Self::kUnknownSiblingIndex,
                backend.QueryIsHTMLDocument(&root),
            )
        } else {
            self.SelectorListMatches(backend.clone(), root, target)
        }
    }
    pub fn Specificity(&self) -> u32 {
        self.selector_list_.MaximumSpecificity()
    }
    /// Rule collection must use the specificity of selectors that actually
    /// match the candidate, not the maximum of unmatched list alternatives.
    pub fn MatchingSpecificity<B: SelectorQueryBackend>(
        &self,
        backend: Rc<B>,
        target: Rc<B::Element>,
    ) -> Option<u32> {
        let root = backend.ElementAsContainer(&target);
        let _scope = QueryCacheScope::new(&*backend, &root, true);
        let checker = SelectorChecker::new(backend.clone(), Mode::kQueryingRules);
        self.selector_start_offsets_
            .iter()
            .filter_map(|&offset| {
                let selector = self.selector_list_.ComplexAt(offset);
                self.SelectorMatches(&*backend, &checker, selector, target.clone(), root.clone())
                    .then(|| selector.Specificity())
            })
            .max()
    }
    pub fn Closest<B: SelectorQueryBackend>(
        &self,
        backend: Rc<B>,
        target: Rc<B::Element>,
    ) -> Option<Rc<B::Element>> {
        Self::ResetStats();
        let root = backend.ElementAsContainer(&target);
        let _scope = QueryCacheScope::new(&*backend, &root, false);
        if self.selector_start_offsets_.is_empty() {
            return None;
        }
        let fast = self.compounds_.len() == 1
            && !self.need_full_check_
            && !self.compounds_[0].has_nth_child;
        if fast {
            self.FillMissingData(&*backend, &root);
        }
        let html = backend.QueryIsHTMLDocument(&root);
        let mut current = Some(target);
        while let Some(element) = current {
            if if fast {
                Self::MatchCompound(
                    &*backend,
                    &element,
                    &self.compounds_[0],
                    Self::kUnknownSiblingIndex,
                    html,
                )
            } else {
                self.SelectorListMatches(backend.clone(), root.clone(), element.clone())
            } {
                return Some(element);
            }
            current = backend.ParentElement(&element);
        }
        None
    }
    pub fn QueryFirst<B: SelectorQueryBackend>(
        &self,
        backend: Rc<B>,
        root: Rc<B::ContainerNode>,
    ) -> Option<Rc<B::Element>> {
        self.Query::<B, true>(backend, root).into_iter().next()
    }
    pub fn QueryAll<B: SelectorQueryBackend>(
        &self,
        backend: Rc<B>,
        root: Rc<B::ContainerNode>,
    ) -> Vec<Rc<B::Element>> {
        self.Query::<B, false>(backend, root)
    }
    fn Query<B: SelectorQueryBackend, const FIRST: bool>(
        &self,
        backend: Rc<B>,
        root: Rc<B::ContainerNode>,
    ) -> Vec<Rc<B::Element>> {
        Self::ResetStats();
        let _scope = QueryCacheScope::new(&*backend, &root, true);
        let mut result = vec![];
        self.Execute::<B, FIRST>(&backend, &root, &mut result);
        result
    }
    fn FirstCompoundNotSeenBeforeRoot<B: SelectorQueryBackend>(
        &self,
        backend: &B,
        root: &B::ContainerNode,
        from: usize,
        to: usize,
        html: bool,
        horizontal: bool,
    ) -> usize {
        if from == to {
            return from;
        }
        let mut remaining = 0;
        for c in &self.compounds_[from..to] {
            let seen = !horizontal && c.next_compound_is_horizontal;
            c.seen_before_root.set(seen);
            if !seen {
                remaining += 1;
            }
        }
        let mut node = if horizontal {
            backend.QueryPreviousSiblingNode(root)
        } else {
            backend.QueryParentNode(root)
        };
        while let Some(n) = node {
            if remaining == 0 {
                break;
            }
            if let Some(element) = backend.ContainerAsElement(&n) {
                for c in &self.compounds_[from..to] {
                    if !c.seen_before_root.get()
                        && Self::MatchCompound(
                            backend,
                            &element,
                            c,
                            Self::kUnknownSiblingIndex,
                            html,
                        )
                    {
                        c.seen_before_root.set(true);
                        remaining -= 1;
                    }
                }
            }
            node = if horizontal {
                backend.QueryPreviousSiblingNode(&n)
            } else {
                backend.QueryParentNode(&n)
            };
        }
        (from..to)
            .find(|&i| !self.compounds_[i].seen_before_root.get())
            .unwrap_or(to)
    }
    fn CouldMatchFilter<B: SelectorQueryBackend>(
        backend: &B,
        e: &B::Element,
        c: &Compound,
    ) -> bool {
        backend.QuerySubtreeBloomFilter(e) & c.selector_filter == c.selector_filter
    }
    fn DescendantOf<B: SelectorQueryBackend>(
        backend: &B,
        element: &B::Element,
        root: &Rc<B::ContainerNode>,
    ) -> bool {
        let mut node = backend.QueryParentNode(&backend.ElementAsContainer(element));
        while let Some(n) = node {
            if Rc::ptr_eq(&n, root) {
                return true;
            }
            node = backend.QueryParentNode(&n);
        }
        false
    }
    fn ExecuteSlow<B: SelectorQueryBackend, const FIRST: bool>(
        &self,
        backend: &Rc<B>,
        root: &Rc<B::ContainerNode>,
        output: &mut Vec<Rc<B::Element>>,
    ) {
        let mut element = backend.QueryFirstChildElement(root);
        while let Some(e) = element {
            stat!(slow_scan);
            if self.SelectorListMatches(backend.clone(), root.clone(), e.clone()) {
                debug_assert!(!FIRST || output.is_empty());
                output.push(e.clone());
                if FIRST {
                    return;
                }
            }
            element = Self::NextDescendant(&**backend, root, &e, true);
        }
    }
    fn NextDescendant<B: SelectorQueryBackend>(
        backend: &B,
        root: &Rc<B::ContainerNode>,
        element: &B::Element,
        children: bool,
    ) -> Option<Rc<B::Element>> {
        let mut node = backend.ElementAsContainer(element);
        if children {
            if let Some(child) = backend.QueryFirstChildElement(&node) {
                return Some(child);
            }
        }
        loop {
            if Rc::ptr_eq(&node, root) {
                return None;
            }
            if let Some(sibling) = backend.QueryNextSiblingElement(&node) {
                return Some(sibling);
            }
            node = backend.QueryParentNode(&node)?;
        }
    }
    fn Execute<B: SelectorQueryBackend, const FIRST: bool>(
        &self,
        backend: &Rc<B>,
        root: &Rc<B::ContainerNode>,
        output: &mut Vec<Rc<B::Element>>,
    ) {
        if self.selector_start_offsets_.is_empty() {
            return;
        }
        if self.selector_start_offsets_.len() > 1 {
            self.ExecuteSlow::<B, FIRST>(backend, root, output);
            return;
        }
        assert_eq!(self.selector_start_offsets_.len(), 1);
        self.FillMissingData(&**backend, root);
        let html = backend.QueryIsHTMLDocument(root);
        let subject = self.compounds_.len() - 1;
        let mut start = self.FirstCompoundNotSeenBeforeRoot(
            &**backend,
            root,
            0,
            self.FindStartOfLevel(subject),
            html,
            false,
        );
        start = self.FindStartOfLevel(start);
        start = self.FirstCompoundNotSeenBeforeRoot(&**backend, root, start, subject, html, true);
        let full = self.need_full_check_ || start > 0;
        let checker = SelectorChecker::new(backend.clone(), Mode::kQueryingRules);
        if let Some(id_index) = self.last_compound_with_id_selector_.filter(|&i| {
            i >= start && backend.QueryIsInTreeScope(root) && !backend.QueryInQuirksMode(root)
        }) {
            let tree_scope = backend.QueryTreeScope(root);
            let c = &self.compounds_[id_index];
            let id = if !c.id_needed.IsNull() {
                &c.id_needed
            } else {
                &c.attr_value
            };
            for (i, c) in self.compounds_.iter().enumerate() {
                c.valid_for_progress.set(i > id_index);
            }
            let multiple = backend.QueryContainsMultipleElementsWithId(&tree_scope, id);
            if !multiple || id_index == subject {
                let candidates = if multiple {
                    backend.QueryAllElementsById(&tree_scope, id)
                } else {
                    backend
                        .QueryElementById(&tree_scope, id)
                        .into_iter()
                        .collect()
                };
                for e in candidates {
                    stat!(fast_id_roots);
                    let node = backend.ElementAsContainer(&e);
                    if !Rc::ptr_eq(&node, root) && !Self::DescendantOf(&**backend, &e, root) {
                        continue;
                    }
                    if self.ExecuteSearch::<B, FIRST, true>(
                        &**backend,
                        node,
                        root,
                        id_index,
                        Self::kUnknownSiblingIndex,
                        full || id_index > 0,
                        html,
                        &checker,
                        output,
                    ) {
                        break;
                    }
                }
                return;
            }
        }
        for c in &self.compounds_ {
            c.valid_for_progress.set(true);
        }
        if self.compounds_[start].simple_traversal_from_here {
            if let Some(first) = backend.QueryFirstChildElement(root) {
                self.ExecuteSearchSingleCompound::<B, FIRST>(
                    &**backend, root, first, root, start, full, html, &checker, output,
                );
            }
        } else {
            self.ExecuteSearch::<B, FIRST, true>(
                &**backend,
                root.clone(),
                root,
                start,
                Self::kUnknownSiblingIndex,
                full,
                html,
                &checker,
                output,
            );
        }
    }
    fn ExecuteSearchSingleCompound<B: SelectorQueryBackend, const FIRST: bool>(
        &self,
        backend: &B,
        root: &Rc<B::ContainerNode>,
        first: Rc<B::Element>,
        scope: &Rc<B::ContainerNode>,
        index: usize,
        full: bool,
        html: bool,
        checker: &SelectorChecker<B>,
        output: &mut Vec<Rc<B::Element>>,
    ) -> bool {
        let c = &self.compounds_[index];
        debug_assert!(c.simple_traversal_from_here);
        if backend
            .ContainerAsElement(root)
            .is_some_and(|e| !Self::CouldMatchFilter(backend, &e, c))
        {
            stat!(skipped_subtree);
            return false;
        }
        let mut element = Some(first);
        while let Some(e) = element {
            let possible = Self::CouldMatchFilter(backend, &e, c);
            if !possible {
                stat!(skipped_subtree);
            } else {
                stat!(elements_seen);
                if Self::MatchCompound(backend, &e, c, Self::kUnknownSiblingIndex, html)
                    && (!full
                        || self.SelectorMatches(
                            backend,
                            checker,
                            self.OnlySelector(),
                            e.clone(),
                            scope.clone(),
                        ))
                {
                    debug_assert!(!FIRST || output.is_empty());
                    output.push(e.clone());
                    if FIRST {
                        return true;
                    }
                }
            }
            element = Self::NextDescendant(backend, root, &e, possible);
        }
        false
    }
    fn ExecuteSearch<B: SelectorQueryBackend, const FIRST: bool, const TOP: bool>(
        &self,
        backend: &B,
        mut node: Rc<B::ContainerNode>,
        scope: &Rc<B::ContainerNode>,
        mut index: usize,
        mut sibling_index: u32,
        mut full: bool,
        html: bool,
        checker: &SelectorChecker<B>,
        output: &mut Vec<Rc<B::Element>>,
    ) -> bool {
        if TOP
            && backend
                .ContainerAsElement(&node)
                .is_some_and(|e| !Self::CouldMatchFilter(backend, &e, &self.compounds_[index]))
        {
            stat!(skipped_subtree);
            return false;
        }
        loop {
            let c = &self.compounds_[index];
            let element = backend.ContainerAsElement(&node);
            if element.is_some() {
                stat!(elements_seen);
            }
            let matched = !(c.is_subject && Rc::ptr_eq(&node, scope))
                && ((c.skip_for_shadow_root && backend.QueryIsShadowRoot(&node))
                    || element
                        .as_ref()
                        .is_some_and(|e| Self::MatchCompound(backend, e, c, sibling_index, html)));
            let (child_index, next_index) = if matched {
                if c.has_nth_child && sibling_index & Self::kUnknownSiblingIndex != 0 {
                    full = true;
                }
                if c.is_subject {
                    if let Some(e) = element {
                        if !full
                            || self.SelectorMatches(
                                backend,
                                checker,
                                self.OnlySelector(),
                                e.clone(),
                                scope.clone(),
                            )
                        {
                            debug_assert!(!FIRST || output.is_empty());
                            output.push(e);
                            if FIRST {
                                return true;
                            }
                        }
                    }
                }
                (
                    c.next_compound_for_children_on_match,
                    c.next_compound_for_siblings_on_match,
                )
            } else {
                (c.next_compound_for_children_on_mismatch, index)
            };
            let child_compound = &self.compounds_[child_index];
            let sibling_compound = &self.compounds_[next_index];
            let mut first_child = backend.QueryFirstChildElement(&node);
            let mut next_sibling = if Rc::ptr_eq(&node, scope) {
                None
            } else {
                backend.QueryNextSiblingElement(&node)
            };
            if TOP {
                if !child_compound.valid_for_progress.get() {
                    first_child = None;
                }
                if !sibling_compound.valid_for_progress.get() {
                    next_sibling = None;
                }
            } else {
                debug_assert!(child_compound.valid_for_progress.get());
                debug_assert!(sibling_compound.valid_for_progress.get());
            }
            let mut child_sibling_index: u32 = 1;
            let mut next_sibling_index = sibling_index.wrapping_add(1);
            while first_child
                .as_ref()
                .is_some_and(|e| !Self::CouldMatchFilter(backend, e, child_compound))
            {
                stat!(skipped_subtree);
                first_child = backend.QueryNextSiblingElement(
                    &backend.ElementAsContainer(first_child.as_ref().unwrap()),
                );
                child_sibling_index += 1;
            }
            while next_sibling
                .as_ref()
                .is_some_and(|e| !Self::CouldMatchFilter(backend, e, sibling_compound))
            {
                stat!(skipped_subtree);
                next_sibling = backend.QueryNextSiblingElement(
                    &backend.ElementAsContainer(next_sibling.as_ref().unwrap()),
                );
                next_sibling_index = next_sibling_index.wrapping_add(1);
            }
            if child_compound.simple_traversal_from_here {
                if let Some(first) = first_child.take() {
                    if self.ExecuteSearchSingleCompound::<B, FIRST>(
                        backend,
                        &node,
                        first,
                        scope,
                        child_index,
                        full,
                        html,
                        checker,
                        output,
                    ) {
                        return true;
                    }
                }
            }
            if let Some(next) = next_sibling {
                if let Some(first) = first_child {
                    if self.ExecuteSearch::<B, FIRST, false>(
                        backend,
                        backend.ElementAsContainer(&first),
                        scope,
                        child_index,
                        child_sibling_index,
                        full,
                        html,
                        checker,
                        output,
                    ) {
                        return true;
                    }
                }
                node = backend.ElementAsContainer(&next);
                index = next_index;
                sibling_index = next_sibling_index;
            } else if let Some(first) = first_child {
                node = backend.ElementAsContainer(&first);
                index = child_index;
                sibling_index = child_sibling_index;
            } else {
                return false;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorQuerySyntaxError {
    pub message: String,
}
#[derive(Default)]
pub struct SelectorQueryCache {
    entries_: HashMap<AtomicString, Rc<SelectorQuery>>,
}
impl SelectorQueryCache {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn Add<P: CSSParserBackend<CSSSelector = CSSSelector>, B: SelectorQueryFilterBackend>(
        &mut self,
        selectors: &AtomicString,
        document: &DocumentSnapshot<P>,
        backend: &B,
    ) -> Result<Rc<SelectorQuery>, SelectorQuerySyntaxError> {
        if selectors.empty() {
            return Err(SelectorQuerySyntaxError {
                message: "The provided selector is empty.".into(),
            });
        }
        if let Some(query) = self.entries_.get(selectors) {
            return Ok(query.clone());
        }
        let context = CSSParserContext::<P>::FromDocumentWithOptions(
            document,
            document.base_url.clone(),
            true,
            P::EmptyReferrer(),
            P::EmptyTextEncoding(),
            crate::css_resource_fetch_restriction::ResourceFetchRestriction::kNone,
        );
        let mut arena = vec![];
        let text = String::from_utf16(selectors.utf16_units().expect("nonempty selector atom"));
        let vector = CSSParser::<P>::ParseSelector(
            &context,
            CSSNestingType::kNone,
            None,
            None,
            &text,
            &mut arena,
        );
        if vector.is_empty() {
            let mut message = vec![b'\'' as u16];
            message.extend_from_slice(selectors.utf16_units().unwrap());
            message.extend("' is not a valid selector.".encode_utf16());
            return Err(SelectorQuerySyntaxError {
                message: String::from_utf16(&message),
            });
        }
        let list = CSSSelectorList::AdoptSelectorVector(vector.to_vec());
        let query = Rc::new(SelectorQuery::new(list, backend));
        if self.entries_.len() == 256 {
            let key = self.entries_.keys().next().unwrap().clone();
            self.entries_.remove(&key);
        }
        self.entries_.insert(selectors.clone(), query.clone());
        Ok(query)
    }
    /// Direct dependency used by Document's selector cache. The selector
    /// parser does not require declaration/color/stylesheet backend methods.
    pub fn AddForSelectorParser<B: SelectorQueryFilterBackend>(
        &mut self,
        selectors: &AtomicString,
        context: &crate::parser::css_selector_parser::SelectorParserContext,
        backend: &B,
    ) -> Result<Rc<SelectorQuery>, SelectorQuerySyntaxError> {
        if selectors.empty() {
            return Err(SelectorQuerySyntaxError {
                message: "The provided selector is empty.".into(),
            });
        }
        if let Some(query) = self.entries_.get(selectors) {
            return Ok(query.clone());
        }
        let text = String::from_utf16(selectors.utf16_units().unwrap());
        let vector =
            crate::parser::css_selector_parser::CSSSelectorParser::ParseSelector(&text, context);
        if vector.is_empty() {
            return Err(SelectorQuerySyntaxError {
                message: format!("'{}' is not a valid selector.", text.Utf8()).into(),
            });
        }
        let query = Rc::new(SelectorQuery::new(
            CSSSelectorList::AdoptSelectorVector(vector),
            backend,
        ));
        if self.entries_.len() == 256 {
            let key = self.entries_.keys().next().unwrap().clone();
            self.entries_.remove(&key);
        }
        self.entries_.insert(selectors.clone(), query.clone());
        Ok(query)
    }
    pub fn Invalidate(&mut self) {
        self.entries_.clear();
    }
}

#[cfg(test)]
#[path = "selector_query_test.rs"]
mod tests;
