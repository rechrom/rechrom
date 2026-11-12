// Copyright (C) 2014 Google Inc. All Rights Reserved.
// Use of this source code is governed by the BSD-style license retained in
// third_party/NOTICE.chromium-event-path.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/tree_scope_event_context.h:1-162
// Source: third_party/blink/renderer/core/dom/events/tree_scope_event_context.cc:1-124

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use super::event::EventTargetRef;
use super::event_path::{
    DOMEventPathSnapshotRef, EventPath, EventPathScope, TouchEventContextFactory,
    TouchEventContextRef, TreeScopeRef,
};
use super::node_event_context::TreeScopeEventContextRef;

// Source: tree_scope_event_context.h:49-103
// The DOM snapshot retains scope/node ownership. This object contains only
// per-dispatch retarget/cache state and references to other dispatch contexts.
pub struct TreeScopeEventContext {
    snapshot_: DOMEventPathSnapshotRef,
    scope_index_: usize,
    target_: Option<EventTargetRef>,
    related_target_: Option<EventTargetRef>,
    event_path_: RefCell<Option<Rc<Vec<EventTargetRef>>>>,
    touch_event_context_: Option<TouchEventContextRef>,
    containing_closed_shadow_tree_: Weak<RefCell<TreeScopeEventContext>>,
    children_: Vec<TreeScopeEventContextRef>,
    pre_order_: i32,
    post_order_: i32,
}

impl TreeScopeEventContext {
    // Source: tree_scope_event_context.h:52; tree_scope_event_context.cc:90-94
    pub fn new(snapshot: DOMEventPathSnapshotRef, scope_index: usize) -> Self {
        Self {
            snapshot_: snapshot,
            scope_index_: scope_index,
            target_: None,
            related_target_: None,
            event_path_: RefCell::new(None),
            touch_event_context_: None,
            containing_closed_shadow_tree_: Weak::new(),
            children_: Vec::new(),
            pre_order_: -1,
            post_order_: -1,
        }
    }

    // Source: tree_scope_event_context.h:53; tree_scope_event_context.cc:96-104
    // Rc/Weak/Drop replace tracing; the closed ancestor backlink is weak to
    // retain source self/ancestor identity without creating Rc cycles.

    // Source: tree_scope_event_context.h:55-56
    pub fn GetTreeScope(&self) -> TreeScopeRef {
        self.snapshot_.TreeScopes()[self.scope_index_]
            .identity
            .clone()
    }
    pub fn RootNode(&self) -> EventTargetRef {
        self.snapshot_.TreeScopes()[self.scope_index_]
            .root_node
            .clone()
    }

    // Source: tree_scope_event_context.h:58-62,119-128
    pub fn Target(&self) -> Option<EventTargetRef> {
        self.target_.clone()
    }
    pub fn SetTarget(&mut self, target: EventTargetRef) {
        self.CheckReachableNode(&target);
        self.target_ = Some(target);
    }
    pub fn RelatedTarget(&self) -> Option<EventTargetRef> {
        self.related_target_.clone()
    }
    pub fn SetRelatedTarget(&mut self, target: EventTargetRef) {
        self.CheckReachableNode(&target);
        self.related_target_ = Some(target);
    }

    // Source: tree_scope_event_context.h:64-67; tree_scope_event_context.cc:84-88
    pub fn GetTouchEventContext(&self) -> Option<TouchEventContextRef> {
        self.touch_event_context_.clone()
    }
    pub fn EnsureTouchEventContext(
        &mut self,
        factory: &dyn TouchEventContextFactory,
    ) -> TouchEventContextRef {
        self.touch_event_context_
            .get_or_insert_with(|| factory.Create())
            .clone()
    }

    // Source: tree_scope_event_context.h:69; tree_scope_event_context.cc:66-82
    pub fn EnsureEventPath(&self, path: &EventPath) -> Rc<Vec<EventTargetRef>> {
        if let Some(path) = self.event_path_.borrow().as_ref() {
            return path.clone();
        }
        let window = path.GetWindowEventContext().Window();
        let mut result = Vec::with_capacity(path.size() + usize::from(window.is_some()));
        for context in path.NodeEventContexts() {
            if context
                .GetTreeScopeEventContext()
                .borrow()
                .IsUnclosedTreeOf(self)
            {
                result.push(context.GetNode());
            }
        }
        if let Some(window) = window {
            result.push(window);
        }
        let result = Rc::new(result);
        *self.event_path_.borrow_mut() = Some(result.clone());
        result
    }

    // Source: tree_scope_event_context.h:71,130-137
    pub fn IsInclusiveAncestorOf(&self, other: &Self) -> bool {
        self.CheckOrder(other);
        self.pre_order_ <= other.pre_order_ && other.post_order_ <= self.post_order_
    }

    // Source: tree_scope_event_context.h:72,139-146
    pub fn IsDescendantOf(&self, other: &Self) -> bool {
        self.CheckOrder(other);
        other.pre_order_ < self.pre_order_ && self.post_order_ < other.post_order_
    }

    // Source: tree_scope_event_context.h:73-75,148-158
    pub fn IsExclusivePartOf(&self, other: &Self) -> bool {
        self.CheckOrder(other);
        (self.pre_order_ < other.pre_order_ && self.post_order_ < other.pre_order_)
            || (self.pre_order_ > other.pre_order_ && self.pre_order_ > other.post_order_)
    }

    fn CheckOrder(&self, other: &Self) {
        debug_assert_ne!(self.pre_order_, -1);
        debug_assert_ne!(self.post_order_, -1);
        debug_assert_ne!(other.pre_order_, -1);
        debug_assert_ne!(other.post_order_, -1);
    }

    // Source: tree_scope_event_context.h:76
    pub fn AddChild(&mut self, child: TreeScopeEventContextRef) {
        self.children_.push(child);
    }

    // Source: tree_scope_event_context.h:78-82; tree_scope_event_context.cc:106-122
    // This source helper operates only on dispatch contexts, never the DOM.
    // Production imports the order already retained by DOM's immutable path.
    pub fn CalculateTreeOrderAndSetNearestAncestorClosedTree(
        this: &TreeScopeEventContextRef,
        mut order_number: i32,
        nearest_closed: Option<TreeScopeEventContextRef>,
    ) -> i32 {
        let (children, closed) = {
            let mut context = this.borrow_mut();
            context.pre_order_ = order_number;
            let is_closed =
                context.snapshot_.TreeScopes()[context.scope_index_].is_closed_shadow_root;
            let closed = if is_closed {
                Some(this.clone())
            } else {
                nearest_closed
            };
            context.containing_closed_shadow_tree_ =
                closed.as_ref().map_or_else(Weak::new, Rc::downgrade);
            (context.children_.clone(), closed)
        };
        for child in children {
            order_number = Self::CalculateTreeOrderAndSetNearestAncestorClosedTree(
                &child,
                order_number + 1,
                closed.clone(),
            );
        }
        this.borrow_mut().post_order_ = order_number + 1;
        order_number + 1
    }

    // Source: tree_scope_event_context.h:84-86
    pub fn ContainingClosedShadowTree(&self) -> Option<TreeScopeEventContextRef> {
        self.containing_closed_shadow_tree_.upgrade()
    }

    // Source: tree_scope_event_context.h:89,105-117
    fn CheckReachableNode(&self, target: &EventTargetRef) {
        #[cfg(debug_assertions)]
        if let Some(node) = self.snapshot_.NodeFacts(target) {
            if !node.is_svg_element {
                debug_assert!(self
                    .snapshot_
                    .IsInclusiveAncestorTreeScopeOf(&node.tree_scope, &self.GetTreeScope(),));
            }
        }
    }

    // Source: tree_scope_event_context.h:91; tree_scope_event_context.cc:39-64
    fn IsUnclosedTreeOf(&self, other: &Self) -> bool {
        if self.IsInclusiveAncestorOf(other) {
            return true;
        }
        let Some(closed) = self.ContainingClosedShadowTree() else {
            return true;
        };
        if self.IsDescendantOf(other) {
            return !closed.borrow().IsDescendantOf(other);
        }
        debug_assert!(other.IsExclusivePartOf(self));
        false
    }

    // Boundary projection for source cc:106-122. These values are borrowed
    // from the same DOM snapshot that supplied the path and retarget values.
    pub(crate) fn ImportSnapshotOrder(
        &mut self,
        scope: &EventPathScope,
        closed: Option<&TreeScopeEventContextRef>,
    ) {
        self.pre_order_ = scope.pre_order;
        self.post_order_ = scope.post_order;
        self.containing_closed_shadow_tree_ = closed.map_or_else(Weak::new, Rc::downgrade);
    }
}
