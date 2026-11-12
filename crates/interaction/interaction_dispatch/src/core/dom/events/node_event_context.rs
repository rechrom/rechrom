// Copyright (C) 2014 Google Inc. All Rights Reserved.
// Use of this source code is governed by the BSD-style license retained in
// third_party/NOTICE.chromium-event-path.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/node_event_context.h:1-86
// Source: third_party/blink/renderer/core/dom/events/node_event_context.cc:1-65

use std::cell::RefCell;
use std::rc::Rc;

use super::event::{EventRef, EventTargetRef};
use super::event_path::{EventPathEntry, LocalEventDispatch, TouchEventContextRef};
use super::tree_scope_event_context::TreeScopeEventContext;

pub type TreeScopeEventContextRef = Rc<RefCell<TreeScopeEventContext>>;

// Source: node_event_context.h:39-75
// These are dispatch-context handles into the retained DOM snapshot, never
// copies of DOM nodes. The source Member owners map to Rc handles.
#[derive(Clone)]
pub struct NodeEventContext {
    node_: EventTargetRef,
    current_target_: EventTargetRef,
    tree_scope_event_context_: Option<TreeScopeEventContextRef>,
    invocation_target_in_shadow_tree_: bool,
}

impl NodeEventContext {
    // Source: node_event_context.h:44; node_event_context.cc:38-45
    pub fn new(entry: &EventPathEntry) -> Self {
        Self {
            node_: entry.node.clone(),
            current_target_: entry.current_target.clone(),
            tree_scope_event_context_: None,
            invocation_target_in_shadow_tree_: entry.invocation_target_in_shadow_tree,
        }
    }

    // Source: node_event_context.h:45; node_event_context.cc:47-51
    // Rc/Drop retain and release precisely the three source Member owners.

    // Source: node_event_context.h:47
    pub fn GetNode(&self) -> EventTargetRef {
        self.node_.clone()
    }

    // Source: node_event_context.h:49-56
    pub fn SetTreeScopeEventContext(&mut self, context: TreeScopeEventContextRef) {
        self.tree_scope_event_context_ = Some(context);
    }
    pub fn GetTreeScopeEventContext(&self) -> TreeScopeEventContextRef {
        self.tree_scope_event_context_
            .as_ref()
            .expect("node dispatch context has a tree scope")
            .clone()
    }

    // Source: node_event_context.h:58-64
    pub fn Target(&self) -> Option<EventTargetRef> {
        self.GetTreeScopeEventContext().borrow().Target()
    }
    pub fn RelatedTarget(&self) -> Option<EventTargetRef> {
        self.GetTreeScopeEventContext().borrow().RelatedTarget()
    }
    pub fn GetTouchEventContext(&self) -> Option<TouchEventContextRef> {
        self.GetTreeScopeEventContext()
            .borrow()
            .GetTouchEventContext()
    }

    // Source: node_event_context.h:66-68
    pub fn CurrentTargetSameAsTarget(&self) -> bool {
        self.Target()
            .is_some_and(|target| Rc::ptr_eq(&self.current_target_, &target))
    }

    // Source: node_event_context.h:69; node_event_context.cc:53-63
    pub fn HandleLocalEvents(&self, event: &EventRef, dispatch: &dyn LocalEventDispatch) {
        if let Some(touch_context) = self.GetTouchEventContext() {
            touch_context.HandleLocalEvents(event);
        } else if let Some(related) = self.RelatedTarget() {
            event.borrow_mut().SetRelatedTarget(Some(related));
        }
        {
            let mut owner = event.borrow_mut();
            owner.SetTarget(self.Target());
            let base = owner.BaseEventMut();
            base.SetCurrentTarget(Some(self.current_target_.clone()));
            base.SetInvocationTargetInShadowTree(self.invocation_target_in_shadow_tree_);
        }
        // Release every Event/context borrow before listener invocation so a
        // synchronous listener may reenter dispatch with the same Event owner.
        dispatch.HandleNodeLocalEvents(&self.node_, event);
    }
}

// Source: node_event_context.h:78-82
// Rust moves/Rc field ownership replace VectorTraits and concurrent GC tracing.
