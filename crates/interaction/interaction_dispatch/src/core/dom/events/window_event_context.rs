// Copyright (C) 2010 Google Inc. All Rights Reserved.
// Use of this source code is governed by the BSD-style license retained in
// third_party/NOTICE.chromium-event-path.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/window_event_context.h:1-75
// Source: third_party/blink/renderer/core/dom/events/window_event_context.cc:1-82

use super::event::{EventRef, EventTargetRef};
use super::event_path::{EventPathEntry, LocalEventDispatch, TouchEventContextRef};
use super::node_event_context::NodeEventContext;

// Source: window_event_context.h:41-59
// No Clone implementation preserves deleted source copying.
// Rust dispatch snapshots clone only retained context handles so the EventPath
// borrow is released before listener reentry; no DOM/path storage is copied.
#[derive(Clone)]
pub struct WindowEventContext {
    window_: Option<EventTargetRef>,
    target_: Option<EventTargetRef>,
    related_target_: Option<EventTargetRef>,
    touch_event_context_: Option<TouchEventContextRef>,
}

impl WindowEventContext {
    // Source: window_event_context.h:43; window_event_context.cc:39-58
    pub fn new(
        event_type: &str,
        top: &NodeEventContext,
        top_snapshot: &EventPathEntry,
        clear_target_only_if_in_shadow_tree: bool,
    ) -> Self {
        let mut context = Self {
            window_: None,
            target_: None,
            related_target_: None,
            touch_event_context_: None,
        };
        if event_type == "load" {
            return context;
        }
        let document = top_snapshot.is_document;
        if if clear_target_only_if_in_shadow_tree {
            top_snapshot.invocation_target_in_shadow_tree
        } else {
            !document
        } {
            return context;
        }
        if document {
            context.window_ = top_snapshot.dom_window.clone();
        }
        context.target_ = top.Target();
        context.related_target_ = top.RelatedTarget();
        context.touch_event_context_ = top.GetTouchEventContext();
        context
    }

    // Source: window_event_context.h:47-49,61-71
    pub fn Window(&self) -> Option<EventTargetRef> {
        self.window_.clone()
    }
    pub fn Target(&self) -> Option<EventTargetRef> {
        self.target_.clone()
    }
    pub fn RelatedTarget(&self) -> Option<EventTargetRef> {
        self.related_target_.clone()
    }

    // Source: window_event_context.h:50; window_event_context.cc:60-73
    pub fn HandleLocalEvents(&self, event: &EventRef, dispatch: &dyn LocalEventDispatch) -> bool {
        let Some(window) = self.Window() else {
            return false;
        };
        if let Some(touch_context) = &self.touch_event_context_ {
            touch_context.HandleLocalEvents(event);
        }
        {
            let mut owner = event.borrow_mut();
            owner.SetTarget(self.Target());
            owner.BaseEventMut().SetCurrentTarget(Some(window.clone()));
            if let Some(related) = self.RelatedTarget() {
                owner.SetRelatedTarget(Some(related));
            }
        }
        dispatch.FireWindowEventListeners(&window, event);
        true
    }

    // Source: window_event_context.h:52; window_event_context.cc:75-80
    // Rc/Drop replace GC tracing of the four retained source Member owners.
}
