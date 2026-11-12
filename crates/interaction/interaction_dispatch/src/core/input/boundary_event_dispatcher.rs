// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/input/boundary_event_dispatcher.h:1-49
// Source: third_party/blink/renderer/core/input/boundary_event_dispatcher.cc:1-141
// Rust modules replace the includes, namespace, and stack-allocation marker.

use std::rc::Rc;

use crate::core::dom::events::event::{EventTargetKind, EventTargetRef};

// Source: boundary_event_dispatcher.cc:15-25,75-78
// DOM supplies this immutable flat-tree ancestor chain from its existing
// tree/slot owner. Index zero is the target and the last entry is its root.
// Holding the snapshot keeps the original node identities alive while the
// boundary listeners mutate the live tree. No tree traversal occurs here.
pub trait BoundaryAncestorChainSnapshot {
    fn Ancestors(&self) -> &[EventTargetRef];
}

pub type BoundaryAncestorChainSnapshotRef = Rc<dyn BoundaryAncestorChainSnapshot>;

// Source: boundary_event_dispatcher.cc:17-24,67-68,107,120,128
// IsInDocument projects event_handling_util.cc:84-86: a non-null Node whose
// isConnected() is true. Listener checks read the retained nodes' current
// listener registrations; they are deliberately not frozen in the snapshot.
pub trait BoundaryEventDOM {
    fn IsInDocument(&self, target: &EventTargetRef) -> bool;
    fn FlatTreeAncestorChain(&self, target: &EventTargetRef) -> BoundaryAncestorChainSnapshotRef;
    fn HasCapturingEventListeners(&self, target: &EventTargetRef, event_type: &str) -> bool;
}

// Source: boundary_event_dispatcher.h:34-38
// The concrete mouse/pointer boundary sender constructs and dispatches its
// Event with the original target and related-target owners. The source's
// check_for_listener optimization is carried through without deciding it here.
pub trait BoundaryEventDispatch {
    fn Dispatch(
        &self,
        target: &EventTargetRef,
        related_target: Option<&EventTargetRef>,
        event_type: &str,
        check_for_listener: bool,
    );
}

// Source: boundary_event_dispatcher.h:13-28,40-44
// Borrowed names retain the source's references to the event-type strings.
pub struct BoundaryEventDispatcher<'a> {
    over_event_: &'a str,
    out_event_: &'a str,
    enter_event_: &'a str,
    leave_event_: &'a str,
}

impl<'a> BoundaryEventDispatcher<'a> {
    // Source: boundary_event_dispatcher.h:20-28
    pub fn new(
        over_event: &'a str,
        out_event: &'a str,
        enter_event: &'a str,
        leave_event: &'a str,
    ) -> Self {
        Self {
            over_event_: over_event,
            out_event_: out_event,
            enter_event_: enter_event,
            leave_event_: leave_event,
        }
    }
}

// Source: boundary_event_dispatcher.cc:15-25
fn BuildAncestorChain(
    dom: &dyn BoundaryEventDOM,
    target: Option<&EventTargetRef>,
) -> Option<BoundaryAncestorChainSnapshotRef> {
    let target = target.filter(|target| dom.IsInDocument(target))?;
    debug_assert_eq!(target.Kind(), EventTargetKind::Node);
    Some(dom.FlatTreeAncestorChain(target))
}

// Source: boundary_event_dispatcher.cc:27-54
fn BuildAncestorChainsAndFindCommonAncestors(
    dom: &dyn BoundaryEventDOM,
    exited_target: Option<&EventTargetRef>,
    entered_target: Option<&EventTargetRef>,
) -> (
    Option<BoundaryAncestorChainSnapshotRef>,
    Option<BoundaryAncestorChainSnapshotRef>,
    usize,
    usize,
) {
    let exited_ancestors = BuildAncestorChain(dom, exited_target);
    let entered_ancestors = BuildAncestorChain(dom, entered_target);
    let exited = exited_ancestors
        .as_ref()
        .map_or(&[][..], |chain| chain.Ancestors());
    let entered = entered_ancestors
        .as_ref()
        .map_or(&[][..], |chain| chain.Ancestors());

    let mut exited_common_parent_index = exited.len();
    let mut entered_common_parent_index = entered.len();
    while exited_common_parent_index > 0 && entered_common_parent_index > 0 {
        if !Rc::ptr_eq(
            &exited[exited_common_parent_index - 1],
            &entered[entered_common_parent_index - 1],
        ) {
            break;
        }
        exited_common_parent_index -= 1;
        entered_common_parent_index -= 1;
    }
    (
        exited_ancestors,
        entered_ancestors,
        exited_common_parent_index,
        entered_common_parent_index,
    )
}

impl BoundaryEventDispatcher<'_> {
    // Source: boundary_event_dispatcher.h:30-32
    // Source: boundary_event_dispatcher.cc:58-139
    pub fn SendBoundaryEvents(
        &self,
        dom: &dyn BoundaryEventDOM,
        sender: &dyn BoundaryEventDispatch,
        exited_target: Option<&EventTargetRef>,
        original_exited_target_removed: bool,
        entered_target: Option<&EventTargetRef>,
    ) {
        // Source: boundary_event_dispatcher.cc:62-64
        let same_target = match (exited_target, entered_target) {
            (Some(exited), Some(entered)) => Rc::ptr_eq(exited, entered),
            (None, None) => true,
            _ => false,
        };
        if same_target && !original_exited_target_removed {
            return;
        }

        // Source: boundary_event_dispatcher.cc:66-70
        if let Some(exited) = exited_target.filter(|target| dom.IsInDocument(target)) {
            if !original_exited_target_removed {
                sender.Dispatch(exited, entered_target, self.out_event_, false);
            }
        }

        // Source: boundary_event_dispatcher.cc:72-103
        // The snapshots are acquired after out dispatch, matching the source
        // mutation point. Their node order remains frozen for leave and enter.
        let (
            exited_ancestors_owner,
            entered_ancestors_owner,
            exited_common_parent_index,
            entered_common_parent_index,
        ) = BuildAncestorChainsAndFindCommonAncestors(dom, exited_target, entered_target);
        let exited_ancestors = exited_ancestors_owner
            .as_ref()
            .map_or(&[][..], |chain| chain.Ancestors());
        let entered_ancestors = entered_ancestors_owner
            .as_ref()
            .map_or(&[][..], |chain| chain.Ancestors());

        // Source: boundary_event_dispatcher.cc:105-111
        let mut exited_node_has_capturing_ancestor = false;
        for ancestor in exited_ancestors {
            if dom.HasCapturingEventListeners(ancestor, self.leave_event_) {
                exited_node_has_capturing_ancestor = true;
                break;
            }
        }

        // Source: boundary_event_dispatcher.cc:113-117
        for ancestor in &exited_ancestors[..exited_common_parent_index] {
            sender.Dispatch(
                ancestor,
                entered_target,
                self.leave_event_,
                !exited_node_has_capturing_ancestor,
            );
        }

        // Source: boundary_event_dispatcher.cc:119-122
        if let Some(entered) = entered_target.filter(|target| dom.IsInDocument(target)) {
            sender.Dispatch(entered, exited_target, self.over_event_, false);
        }

        // Source: boundary_event_dispatcher.cc:124-132
        // Leave/over listeners may install a capturing enter listener, so this
        // check intentionally occurs after those synchronous dispatches.
        let mut entered_node_has_capturing_ancestor = false;
        for ancestor in entered_ancestors {
            if dom.HasCapturingEventListeners(ancestor, self.enter_event_) {
                entered_node_has_capturing_ancestor = true;
                break;
            }
        }

        // Source: boundary_event_dispatcher.cc:134-138
        for ancestor in entered_ancestors[..entered_common_parent_index]
            .iter()
            .rev()
        {
            sender.Dispatch(
                ancestor,
                exited_target,
                self.enter_event_,
                !entered_node_has_capturing_ancestor,
            );
        }
    }
}
