// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license.
// Source ledger: comments and blank lines stripped; braces retained.
// Effective = mapped + omitted + production pending. All non-omitted
// effective source lines are mapped; omissions are listed exactly below.
// Omitted: includes/guards/access/forward/cast/allocation scaffolding,
// deleted copy/move/default destructors, GC Trace and inspector tracing.
// style_invalidator.h: physical 153, effective 111, mapped 81,
// omitted 30, production pending 0.
// style_invalidator.h omitted effective lines:5-6,8-12,14,16-20,27,29,32,35-36,100,102,111,113,120,132,134,138
// 144,147,151,153
// style_invalidator.cc: physical 349, effective 281, mapped 246,
// omitted 35, production pending 0.
// style_invalidator.cc omitted effective lines:5,7-16,18,20-22,56,161-162,165,182-184,210-216,247-248,301-302
// 344,349
// cpp: third_party/blink/renderer/core/css/invalidation/style_invalidator.h
// cpp: third_party/blink/renderer/core/css/invalidation/style_invalidator.cc
// Recursive checkpoints restore the source's vector size and flags. Rc retains
// sets while the queue is traversed; tracing and platform scaffolding are omitted.

use super::invalidation_dom::{InvalidationDom, StyleChangeType};
use super::invalidation_flags::InvalidationFlags;
use super::invalidation_set::{InvalidationElement, InvalidationSetRef};
use super::pending_invalidations::PendingInvalidationMap;
use std::hash::Hash;

pub struct StyleInvalidator<'a, N> {
    pending_invalidation_map_: &'a mut PendingInvalidationMap<N>,
    invalidation_sets_: Vec<InvalidationSetRef>,
    pending_nth_sets_: Vec<InvalidationSetRef>,
    invalidation_flags_: InvalidationFlags,
}

#[derive(Default)]
struct SiblingData {
    invalidation_entries_: Vec<SiblingEntry>,
    element_index_: u32,
}
struct SiblingEntry {
    invalidation_set_: InvalidationSetRef,
    invalidation_limit_: u32,
}
// cpp: style_invalidator.h:131-148; scope restoration is explicit at the end of
// each recursive function (there are no early returns inside either checkpoint).
struct RecursionCheckpoint {
    prev_invalidation_sets_size_: usize,
    prev_invalidation_flags_: InvalidationFlags,
}

impl SiblingData {
    // cpp: style_invalidator.h:110-116; style_invalidator.cc:119-129
    fn IsEmpty(&self) -> bool {
        self.invalidation_entries_.is_empty()
    }
    fn Advance(&mut self) {
        self.element_index_ = self.element_index_.wrapping_add(1);
    }
    fn PushInvalidationSet(&mut self, set: &InvalidationSetRef) {
        let maximum = set.borrow().MaxDirectAdjacentSelectors();
        let limit = if maximum == u32::MAX {
            u32::MAX
        } else {
            self.element_index_.wrapping_add(maximum)
        };
        self.invalidation_entries_.push(SiblingEntry {
            invalidation_set_: set.clone(),
            invalidation_limit_: limit,
        });
    }

    // cpp: style_invalidator.cc:131-175
    fn MatchCurrentInvalidationSets<N: Clone + Eq + Hash, D: InvalidationDom<Node = N>>(
        &mut self,
        element: &N,
        invalidator: &mut StyleInvalidator<'_, N>,
        dom: &mut D,
    ) -> bool {
        let mut needs_recalc = false;
        debug_assert!(!invalidator.WholeSubtreeInvalid());
        let mut index = 0;
        while index < self.invalidation_entries_.len() {
            if self.element_index_ > self.invalidation_entries_[index].invalidation_limit_ {
                self.invalidation_entries_.swap_remove(index);
                continue;
            }
            let set = self.invalidation_entries_[index].invalidation_set_.borrow();
            index += 1;
            if !set.InvalidatesElement(dom.element(element).expect("element traversal")) {
                continue;
            }
            if set.InvalidatesSelf() {
                needs_recalc = true;
            }
            if let Some(descendants) = set.SiblingDescendants() {
                if descendants.borrow().WholeSubtreeInvalid() {
                    dom.set_needs_style_recalc(element, StyleChangeType::SubtreeStyleChange);
                    return true;
                }
                if !descendants.borrow().IsEmpty() {
                    invalidator.PushInvalidationSet(descendants);
                }
            }
        }
        needs_recalc
    }
}

impl<'a, N: Clone + Eq + Hash> StyleInvalidator<'a, N> {
    // cpp: style_invalidator.cc:52-56
    pub fn new(pending_invalidation_map: &'a mut PendingInvalidationMap<N>) -> Self {
        Self {
            pending_invalidation_map_: pending_invalidation_map,
            invalidation_sets_: Vec::new(),
            pending_nth_sets_: Vec::new(),
            invalidation_flags_: InvalidationFlags::default(),
        }
    }

    // cpp: style_invalidator.cc:24-50
    pub fn Invalidate<D: InvalidationDom<Node = N>>(
        &mut self,
        document: &N,
        root_element: Option<N>,
        dom: &mut D,
    ) {
        let mut siblings = SiblingData::default();
        if dom.needs_style_invalidation(document) {
            debug_assert!(root_element == dom.document_element(document));
            self.PushInvalidationSetsForContainerNode(document, &mut siblings, dom);
            dom.clear_needs_style_invalidation(document);
            debug_assert!(siblings.IsEmpty());
        }
        if let Some(root) = root_element {
            self.InvalidateElement(&root, &mut siblings, dom);
            if !siblings.IsEmpty() {
                let mut child = dom.next_element_sibling(&root);
                while let Some(element) = child {
                    self.InvalidateElement(&element, &mut siblings, dom);
                    child = dom.next_element_sibling(&element);
                }
            }
            let mut ancestor = Some(root);
            while let Some(node) = ancestor {
                dom.clear_child_needs_style_invalidation(&node);
                ancestor = dom.parent_or_shadow_host_node(&node);
            }
        }
        dom.clear_child_needs_style_invalidation(document);
        self.pending_invalidation_map_.clear();
        self.pending_nth_sets_.clear();
    }

    fn WholeSubtreeInvalid(&self) -> bool {
        self.invalidation_flags_.WholeSubtreeInvalid()
    }
    fn SetWholeSubtreeInvalid(&mut self) {
        self.invalidation_flags_.SetWholeSubtreeInvalid(true);
    }
    fn HasInvalidationSets(&self) -> bool {
        !self.WholeSubtreeInvalid()
            && (!self.invalidation_sets_.is_empty() || !self.pending_nth_sets_.is_empty())
    }
    fn TreeBoundaryCrossing(&self) -> bool {
        self.invalidation_flags_.TreeBoundaryCrossing()
    }
    fn InsertionPointCrossing(&self) -> bool {
        self.invalidation_flags_.InsertionPointCrossing()
    }
    fn InvalidatesSlotted(&self) -> bool {
        self.invalidation_flags_.InvalidatesSlotted()
    }
    fn InvalidatesParts(&self) -> bool {
        self.invalidation_flags_.InvalidatesParts()
    }
    // cpp: style_invalidator.h:78-88
    fn AddPendingNthSiblingInvalidationSet(&mut self, set: &InvalidationSetRef) {
        self.pending_nth_sets_.push(set.clone());
    }
    fn PushNthSiblingInvalidationSets(&mut self, siblings: &mut SiblingData) {
        for set in &self.pending_nth_sets_ {
            siblings.PushInvalidationSet(set);
        }
        self.ClearPendingNthSiblingInvalidationSets();
    }
    fn ClearPendingNthSiblingInvalidationSets(&mut self) {
        self.pending_nth_sets_.clear();
    }

    // cpp: style_invalidator.cc:58-76
    fn PushInvalidationSet(&mut self, set: &InvalidationSetRef) {
        let value = set.borrow();
        debug_assert!(!self.WholeSubtreeInvalid());
        debug_assert!(!value.WholeSubtreeInvalid());
        debug_assert!(!value.IsEmpty());
        if value.TreeBoundaryCrossing() {
            self.invalidation_flags_.SetTreeBoundaryCrossing(true);
        }
        if value.InsertionPointCrossing() {
            self.invalidation_flags_.SetInsertionPointCrossing(true);
        }
        if value.InvalidatesSlotted() {
            self.invalidation_flags_.SetInvalidatesSlotted(true);
        }
        if value.InvalidatesParts() {
            self.invalidation_flags_.SetInvalidatesParts(true);
        }
        self.invalidation_sets_.push(set.clone());
    }

    // cpp: style_invalidator.cc:78-117
    fn MatchesCurrentInvalidationSets<D: InvalidationDom<Node = N>>(
        &self,
        element: &N,
        dom: &D,
    ) -> bool {
        self.invalidation_sets_.iter().any(|set| {
            set.borrow()
                .InvalidatesElement(dom.element(element).expect("element traversal"))
        })
    }
    fn MatchesCurrentInvalidationSetsAsSlotted<D: InvalidationDom<Node = N>>(
        &self,
        element: &N,
        dom: &D,
    ) -> bool {
        debug_assert!(self.InvalidatesSlotted());
        self.invalidation_sets_.iter().any(|set| {
            let value = set.borrow();
            value.InvalidatesSlotted()
                && value.InvalidatesElement(dom.element(element).expect("assigned element"))
        })
    }
    fn MatchesCurrentInvalidationSetsAsParts<D: InvalidationDom<Node = N>>(
        &self,
        element: &N,
        dom: &D,
    ) -> bool {
        debug_assert!(self.InvalidatesParts());
        self.invalidation_sets_.iter().any(|set| {
            let value = set.borrow();
            value.InvalidatesParts()
                && value.InvalidatesElement(dom.element(element).expect("part element"))
        })
    }

    // cpp: style_invalidator.cc:177-218
    fn PushInvalidationSetsForContainerNode<D: InvalidationDom<Node = N>>(
        &mut self,
        node: &N,
        siblings: &mut SiblingData,
        dom: &D,
    ) {
        let Some(pending) = self.pending_invalidation_map_.get(node) else {
            // Blink emits a diagnostic, then returns; retain its recovery branch.
            return;
        };
        let sibling_sets = pending.Siblings().clone();
        let descendant_sets = pending.Descendants().clone();
        debug_assert!(self.pending_nth_sets_.is_empty());
        for set in &sibling_sets {
            if set.borrow().IsNthSiblingInvalidationSet() {
                self.AddPendingNthSiblingInvalidationSet(set);
            } else {
                siblings.PushInvalidationSet(set);
            }
        }
        if dom.style_change_type(node) == StyleChangeType::SubtreeStyleChange {
            return;
        }
        for set in &descendant_sets {
            self.PushInvalidationSet(set);
        }
    }

    // cpp: style_invalidator.cc:220-234. Evaluate both: siblings can mark a
    // subtree even when the descendant match already returned true.
    fn CheckInvalidationSetsAgainstElement<D: InvalidationDom<Node = N>>(
        &mut self,
        element: &N,
        siblings: &mut SiblingData,
        dom: &mut D,
    ) -> bool {
        let current = self.MatchesCurrentInvalidationSets(element, dom);
        let sibling =
            !siblings.IsEmpty() && siblings.MatchCurrentInvalidationSets(element, self, dom);
        current || sibling
    }

    fn Checkpoint(&self) -> RecursionCheckpoint {
        RecursionCheckpoint {
            prev_invalidation_sets_size_: self.invalidation_sets_.len(),
            prev_invalidation_flags_: self.invalidation_flags_,
        }
    }
    fn Restore(&mut self, checkpoint: RecursionCheckpoint) {
        self.invalidation_sets_
            .truncate(checkpoint.prev_invalidation_sets_size_);
        self.invalidation_flags_ = checkpoint.prev_invalidation_flags_;
    }

    // cpp: style_invalidator.cc:236-268
    fn InvalidateShadowRootChildren<D: InvalidationDom<Node = N>>(
        &mut self,
        element: &N,
        dom: &mut D,
    ) {
        let Some(root) = dom.shadow_root(element) else {
            return;
        };
        if !self.TreeBoundaryCrossing()
            && !dom.child_needs_style_invalidation(&root)
            && !dom.needs_style_invalidation(&root)
        {
            return;
        }
        let checkpoint = self.Checkpoint();
        let mut siblings = SiblingData::default();
        if !self.WholeSubtreeInvalid() && dom.needs_style_invalidation(&root) {
            debug_assert!(siblings.IsEmpty());
            self.PushInvalidationSetsForContainerNode(&root, &mut siblings, dom);
        }
        self.PushNthSiblingInvalidationSets(&mut siblings);
        for child in dom.element_children(&root) {
            self.InvalidateElement(&child, &mut siblings, dom);
        }
        dom.clear_child_needs_style_invalidation(&root);
        dom.clear_needs_style_invalidation(&root);
        self.Restore(checkpoint);
    }

    // cpp: style_invalidator.cc:270-284
    fn InvalidateChildren<D: InvalidationDom<Node = N>>(&mut self, element: &N, dom: &mut D) {
        if dom.shadow_root(element).is_some() {
            self.InvalidateShadowRootChildren(element, dom);
        }
        let mut siblings = SiblingData::default();
        self.PushNthSiblingInvalidationSets(&mut siblings);
        for child in dom.element_children(element) {
            self.InvalidateElement(&child, &mut siblings, dom);
        }
    }

    // cpp: style_invalidator.cc:286-329
    fn InvalidateElement<D: InvalidationDom<Node = N>>(
        &mut self,
        element: &N,
        siblings: &mut SiblingData,
        dom: &mut D,
    ) {
        siblings.Advance();
        let checkpoint = self.Checkpoint();
        if !self.WholeSubtreeInvalid() {
            if dom.style_change_type(element) == StyleChangeType::SubtreeStyleChange {
                self.SetWholeSubtreeInvalid();
            } else if self.CheckInvalidationSetsAgainstElement(element, siblings, dom) {
                dom.set_needs_style_recalc(element, StyleChangeType::LocalStyleChange);
            }
            if dom.needs_style_invalidation(element) {
                self.PushInvalidationSetsForContainerNode(element, siblings, dom);
            }
            if dom.is_html_slot(element) && self.InvalidatesSlotted() {
                self.InvalidateSlotDistributedElements(element, dom);
            }
        }
        if (!self.WholeSubtreeInvalid()
            && self.HasInvalidationSets()
            && dom
                .element(element)
                .expect("element traversal")
                .HasComputedStyle())
            || dom.child_needs_style_invalidation(element)
        {
            self.InvalidateChildren(element, dom);
        } else {
            self.ClearPendingNthSiblingInvalidationSets();
        }
        dom.clear_child_needs_style_invalidation(element);
        dom.clear_needs_style_invalidation(element);
        self.Restore(checkpoint);
    }

    // cpp: style_invalidator.cc:331-347
    fn InvalidateSlotDistributedElements<D: InvalidationDom<Node = N>>(
        &self,
        slot: &N,
        dom: &mut D,
    ) {
        for node in dom.flattened_assigned_nodes(slot) {
            if dom.needs_style_recalc(&node) || dom.element(&node).is_none() {
                continue;
            }
            if self.MatchesCurrentInvalidationSetsAsSlotted(&node, dom) {
                dom.set_needs_style_recalc(&node, StyleChangeType::LocalStyleChange);
            }
        }
    }
}
