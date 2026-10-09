// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license.
// Source ledger: comments and blank lines stripped; braces retained.
// Effective = mapped + omitted + production pending. All non-omitted
// effective source lines are mapped; omissions are listed exactly below.
// Omitted: includes/guards/access/forward/cast/allocation scaffolding,
// deleted copy/move/default destructors, GC Trace and inspector tracing.
// pending_invalidations.h: physical 101, effective 38, mapped 17,
// omitted 21, production pending 0.
// pending_invalidations.h omitted effective lines:5-6,8,10-13,15,17-18,70,72,74-76,90-92,94,99,101
// pending_invalidations.cc: physical 234, effective 189, mapped 152,
// omitted 37, production pending 0.
// pending_invalidations.cc omitted effective lines:5,7-17,19,24-25,32-33,43-47,50,56-58,60-61,152-154,161-162
// 165-166,172,234
// cpp: third_party/blink/renderer/core/css/invalidation/pending_invalidations.h
// cpp: third_party/blink/renderer/core/css/invalidation/pending_invalidations.cc
// All scheduling decisions map to Blink; DOM state lives in InvalidationDom.
// Box retains NodeInvalidationSets allocation identity across HashMap growth.

use super::invalidation_dom::{InvalidationDom, StyleChangeType};
use super::invalidation_set::{InvalidationElement, InvalidationLists};
use super::node_invalidation_sets::{Contains, NodeInvalidationSets};
use std::collections::HashMap;
use std::hash::Hash;

pub type PendingInvalidationMap<N> = HashMap<N, Box<NodeInvalidationSets>>;
pub struct PendingInvalidations<N> {
    pending_invalidation_map_: PendingInvalidationMap<N>,
}
impl<N> Default for PendingInvalidations<N> {
    fn default() -> Self {
        Self {
            pending_invalidation_map_: HashMap::new(),
        }
    }
}

impl<N: Clone + Eq + Hash> PendingInvalidations<N> {
    pub fn GetPendingInvalidationMap(&mut self) -> &mut PendingInvalidationMap<N> {
        &mut self.pending_invalidation_map_
    }

    // cpp: pending_invalidations.cc:21-128
    pub fn ScheduleInvalidationSetsForNode<D: InvalidationDom<Node = N>>(
        &mut self,
        invalidation_lists: &InvalidationLists,
        node: &N,
        dom: &mut D,
    ) {
        let mut requires_descendant_invalidation = false;
        if dom.style_change_type(node) < StyleChangeType::SubtreeStyleChange {
            for set in &invalidation_lists.descendants {
                let invalidation_set = set.borrow();
                if invalidation_set.InvalidatesNth() {
                    dom.possibly_schedule_nth_pseudo_invalidations(node);
                }
                if invalidation_set.WholeSubtreeInvalid() {
                    let subtree_root = dom.shadow_host(node).unwrap_or_else(|| node.clone());
                    dom.set_needs_style_recalc(&subtree_root, StyleChangeType::SubtreeStyleChange);
                    requires_descendant_invalidation = false;
                    break;
                }
                if invalidation_set.InvalidatesSelf() && dom.element(node).is_some() {
                    dom.set_needs_style_recalc(node, StyleChangeType::LocalStyleChange);
                }
                if !invalidation_set.IsEmpty() {
                    requires_descendant_invalidation = true;
                }
            }
            if requires_descendant_invalidation
                && dom.element(node).is_some_and(|e| !e.HasComputedStyle())
            {
                requires_descendant_invalidation = false;
            }
        }
        if !requires_descendant_invalidation && invalidation_lists.siblings.is_empty() {
            return;
        }

        let nth_only = dom.next_sibling(node).is_none();
        let mut requires_sibling_invalidation = false;
        // Do not allocate an entry before finding a schedulable sibling set:
        // otherwise DOM removal cannot clear an orphaned entry (crbug 40257823).
        for set in &invalidation_lists.siblings {
            let invalidation_set = set.borrow();
            if nth_only && !invalidation_set.IsNthSiblingInvalidationSet() {
                continue;
            }
            let pending = self.EnsurePendingInvalidations(node);
            if Contains(pending.Siblings(), set) {
                continue;
            }
            if invalidation_set.InvalidatesNth() {
                dom.possibly_schedule_nth_pseudo_invalidations(node);
            }
            pending.SiblingsMut().push(set.clone());
            requires_sibling_invalidation = true;
        }
        if requires_sibling_invalidation || requires_descendant_invalidation {
            dom.set_needs_style_invalidation(node);
        }
        if !requires_descendant_invalidation {
            return;
        }
        let pending = self.EnsurePendingInvalidations(node);
        for set in &invalidation_lists.descendants {
            let invalidation_set = set.borrow();
            debug_assert!(!invalidation_set.WholeSubtreeInvalid());
            if invalidation_set.IsEmpty() || Contains(pending.Descendants(), set) {
                continue;
            }
            pending.DescendantsMut().push(set.clone());
        }
    }

    // cpp: pending_invalidations.cc:130-186
    pub fn ScheduleSiblingInvalidationsAsDescendants<D: InvalidationDom<Node = N>>(
        &mut self,
        invalidation_lists: &InvalidationLists,
        scheduling_parent: &N,
        dom: &mut D,
    ) {
        debug_assert!(invalidation_lists.descendants.is_empty());
        if invalidation_lists.siblings.is_empty() {
            return;
        }
        let pending = self.EnsurePendingInvalidations(scheduling_parent);
        dom.set_needs_style_invalidation(scheduling_parent);
        let subtree_root = if dom.element(scheduling_parent).is_some() {
            scheduling_parent.clone()
        } else {
            dom.shadow_host(scheduling_parent)
                .expect("scheduling parent is an element or shadow root")
        };
        for set in &invalidation_lists.siblings {
            let invalidation_set = set.borrow();
            let descendants = invalidation_set.SiblingDescendants();
            if invalidation_set.WholeSubtreeInvalid()
                || descendants.is_some_and(|d| d.borrow().WholeSubtreeInvalid())
            {
                dom.set_needs_style_recalc(&subtree_root, StyleChangeType::SubtreeStyleChange);
                return;
            }
            if invalidation_set.InvalidatesSelf() && !Contains(pending.Descendants(), set) {
                pending.DescendantsMut().push(set.clone());
            }
            if let Some(descendants) = descendants {
                if !Contains(pending.Descendants(), descendants) {
                    pending.DescendantsMut().push(descendants.clone());
                }
            }
        }
    }

    // cpp: pending_invalidations.cc:188-214
    pub fn RescheduleSiblingInvalidationsAsDescendants<D: InvalidationDom<Node = N>>(
        &mut self,
        element: &N,
        dom: &mut D,
    ) {
        let parent = dom
            .parent_node(element)
            .expect("removed element still has its parent");
        if dom.is_document(&parent) {
            return;
        }
        let Some(pending) = self.pending_invalidation_map_.get(element) else {
            return;
        };
        if pending.Siblings().is_empty() {
            return;
        }
        let mut lists = InvalidationLists::default();
        for set in pending.Siblings() {
            lists.descendants.push(set.clone());
            if let Some(descendants) = set.borrow().SiblingDescendants() {
                lists.descendants.push(descendants.clone());
            }
        }
        self.ScheduleInvalidationSetsForNode(&lists, &parent, dom);
    }

    // cpp: pending_invalidations.cc:216-232
    pub fn ClearInvalidation<D: InvalidationDom<Node = N>>(&mut self, node: &N, dom: &mut D) {
        debug_assert!(dom.needs_style_invalidation(node));
        self.pending_invalidation_map_.remove(node);
        dom.clear_needs_style_invalidation(node);
    }
    fn EnsurePendingInvalidations(&mut self, node: &N) -> &mut NodeInvalidationSets {
        self.pending_invalidation_map_
            .entry(node.clone())
            .or_default()
    }
}
