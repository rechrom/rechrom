// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/input/mouse_wheel_event_manager.h:1-55
// Source: third_party/blink/renderer/core/input/mouse_wheel_event_manager.cc:1-254
// Rust modules replace includes/namespaces/GC annotations. Shared handles keep
// the same frame, target, scroll manager and scrollable identities.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use foundation::{gfx, PhysicalOffset};
use interaction_dispatch::core::dom::events::event::{EventRef, EventTargetRef};
use interaction_dispatch::core::dom::events::event_dispatch_result::DispatchEventResult;
use interaction_dispatch::core::events::wheel_event::{WheelEvent, WheelEventWindow};
use interaction_dispatch::core::input::event_handling_util::ToWebInputEventResult;
pub use interaction_dispatch::public::platform::web_input_event_result::WebInputEventResult;
use interaction_hit_test::core::layout::hit_test_location::HitTestLocation;
use interaction_hit_test::core::layout::hit_test_request::HitTestRequest;
use interaction_input::public::common::input::web_mouse_wheel_event::{
    EventAction, WebMouseWheelEvent,
};

// Source: mouse_wheel_event_manager.cc:43-49,60-62,180-185
// The frame projection retains only the document/view owners and the two
// existing wheel routes used by this manager.
pub trait WheelFrame {
    fn Document(&self) -> Option<Rc<dyn WheelDocument>>;
    fn View(&self) -> Option<Rc<dyn WheelFrameView>>;
    fn PointerLockedElement(&self) -> Option<EventTargetRef>;
    fn ParentLocalWheelRoute(&self) -> Option<Rc<dyn WheelTargetRoute>>;
}

// Source: mouse_wheel_event_manager.cc:44,98,102-103,140-143,156-157
// HitTest returns the two facts read by this source without publishing a
// second Layout or DOM tree. The document provides its canonical live query.
pub trait WheelDocument {
    fn HasLayoutView(&self) -> bool;
    fn Window(&self) -> WheelEventWindow;
    fn IsVerticalScrollEnforced(&self) -> bool;
    fn HitTest(
        &self,
        location: &HitTestLocation,
        request: &HitTestRequest<'_, (), ()>,
    ) -> WheelHitTestResult;
    fn DocumentElement(&self) -> Option<EventTargetRef>;
}

// Source: mouse_wheel_event_manager.cc:137-138
pub trait WheelFrameView {
    fn ConvertFromRootFrame(&self, point: gfx::Point) -> PhysicalOffset;
}

// Source: mouse_wheel_event_manager.cc:145-157
pub struct WheelHitTestResult {
    pub inner_node: Option<EventTargetRef>,
    pub over_scrollbar: bool,
}

// The production Page lends its resident fragment cursor only for the current
// synchronous input transaction. Keep the source target-selection algorithm in
// this manager while borrowing its Document/View/Node owners call-scoped; the
// manager retains only the selected EventTarget identity across wheel phases.
// Source: mouse_wheel_event_manager.cc:41-62,96-160
pub trait WheelTargetQuery {
    fn Document(&self) -> &dyn WheelDocument;
    fn View(&self) -> &dyn WheelFrameView;
    fn Nodes(&self) -> &dyn WheelNodeQueries;
    fn TargetWindow(&self, target: &EventTargetRef) -> WheelEventWindow;
    fn IsVerticalScrollEnforced(&self, target: &EventTargetRef) -> bool;
    fn DispatchEvent(&self, target: &EventTargetRef, event: &EventRef) -> DispatchEventResult;
}

// Source: mouse_wheel_event_manager.cc:98,102-103,145-152
// These projections read only the target's document, type, and ancestor
// layout eligibility. Node identity always remains the DOM-owned handle.
pub trait WheelNodeQueries {
    fn IsTextNode(&self, node: &EventTargetRef) -> bool;
    fn FlatTreeParent(&self, node: &EventTargetRef) -> Option<EventTargetRef>;
    fn HasLayoutObject(&self, node: &EventTargetRef) -> bool;
}

// Source: mouse_wheel_event_manager.cc:81-85,167-171,181-184
pub trait WheelTargetRouting {
    fn SubframeForTargetNode(
        &self,
        target: Option<&EventTargetRef>,
    ) -> Option<Rc<dyn WheelTargetRoute>>;
}

pub trait WheelTargetRoute {
    fn HandleWheelEvent(&self, event: &WebMouseWheelEvent) -> WebInputEventResult;
    fn ClearWheelTarget(&self);
}

// Source: mouse_wheel_event_manager.cc:104-105
// The dispatcher receives the same WheelEvent owner that the manager keeps
// across the synchronous call, including listener reentry and cancellation.
pub trait WheelDOMDispatch {
    fn DispatchEvent(&self, target: &EventTargetRef, event: &EventRef) -> DispatchEventResult;
}

// Source: mouse_wheel_event_manager.cc:207-209,238-240,245-249
pub trait WheelScrollableArea {
    // Shared source ScrollableArea identity used by HitTestResult as well as
    // wheel traversal; keep it on the same resident area owner.
    fn GetScrollElementId(&self) -> foundation::CompositorElementId;
    fn ScrollOffset(&self) -> (f64, f64);
    fn MinimumScrollOffset(&self) -> (f64, f64);
    fn MaximumScrollOffset(&self) -> (f64, f64);
    fn FadeInScrollbarIfExists(&self, horizontal: bool, vertical: bool) -> bool;
    fn FadeOutScrollbarIfNeeded(&self);
}

// Source: mouse_wheel_event_manager.cc:202
// This is the existing ScrollableAreaTraversal owner/cursor, whose source
// implementation lives in scrollable_area.cc:1526-1551. No layer/fragment or
// scrollable list is materialized at the interaction boundary.
pub trait WheelScrollableTraversal {
    fn ScrollableAreas<'a>(
        &'a self,
        target: Option<&EventTargetRef>,
    ) -> Box<dyn Iterator<Item = Rc<dyn WheelScrollableArea>> + 'a>;
}

// Source: mouse_wheel_event_manager.cc:190-193,238-240
// Feature values are resolved by the embedding before this source owner runs.
#[derive(Clone, Copy, Debug)]
pub struct MouseWheelFeatures {
    pub fade_in_scrollbar_when_mouse_wheel_may_begin: bool,
    pub defer_fade_out_scrollbar_until_mouse_wheel_ended: bool,
}

struct RetainedWheelTargetContext {
    nodes: Rc<dyn WheelNodeQueries>,
    dispatch: Rc<dyn WheelDOMDispatch>,
}

pub struct MouseWheelEventManagerContext {
    retained_target: Option<RetainedWheelTargetContext>,
    pub routing: Rc<dyn WheelTargetRouting>,
    pub scrollables: Rc<dyn WheelScrollableTraversal>,
    pub features: MouseWheelFeatures,
}

impl MouseWheelEventManagerContext {
    /// Source-compatible retained target services. Embeddings whose fragment
    /// lease is call-scoped use `CallScoped` and supply a WheelTargetQuery to
    /// the complete synchronous transaction instead.
    pub fn Retained(
        nodes: Rc<dyn WheelNodeQueries>,
        routing: Rc<dyn WheelTargetRouting>,
        dispatch: Rc<dyn WheelDOMDispatch>,
        scrollables: Rc<dyn WheelScrollableTraversal>,
        features: MouseWheelFeatures,
    ) -> Self {
        Self {
            retained_target: Some(RetainedWheelTargetContext { nodes, dispatch }),
            routing,
            scrollables,
            features,
        }
    }

    /// Page's resident fragments are lent for one synchronous dispatch. No
    /// retained DOM dispatcher or copied hit-test tree is manufactured merely
    /// to satisfy the compatibility entry point.
    pub fn CallScoped(
        routing: Rc<dyn WheelTargetRouting>,
        scrollables: Rc<dyn WheelScrollableTraversal>,
        features: MouseWheelFeatures,
    ) -> Self {
        Self {
            retained_target: None,
            routing,
            scrollables,
            features,
        }
    }
}

// Source: mouse_wheel_event_manager.h:23-50
// M is the same ScrollManager owner supplied by the facade. This source only
// retains/traces that member; it never calls an operation on it.
pub struct MouseWheelEventManager<M: ?Sized> {
    frame_: Rc<dyn WheelFrame>,
    wheel_target_: RefCell<Option<EventTargetRef>>,
    _scroll_manager: Rc<M>,
    fade_out_deferred_scrollables_: RefCell<HashMap<usize, Weak<dyn WheelScrollableArea>>>,
    context_: MouseWheelEventManagerContext,
}

impl<M: ?Sized> MouseWheelEventManager<M> {
    // Source: mouse_wheel_event_manager.h:26-29;
    // mouse_wheel_event_manager.cc:26-35
    // Rc/Weak destruction supplies the traced Member/WeakMember ownership.
    // The absent Clone implementation preserves the deleted source copying.
    pub fn new(
        frame: Rc<dyn WheelFrame>,
        scroll_manager: Rc<M>,
        context: MouseWheelEventManagerContext,
    ) -> Self {
        Self {
            frame_: frame,
            wheel_target_: RefCell::new(None),
            _scroll_manager: scroll_manager,
            fade_out_deferred_scrollables_: RefCell::new(HashMap::new()),
            context_: context,
        }
    }

    // Source: mouse_wheel_event_manager.cc:37-39
    pub fn HasWheelTarget(&self) -> bool {
        self.wheel_target_.borrow().is_some()
    }
    pub fn Clear(&self) {
        self.UpdateWheelTarget(None);
    }

    // Source: mouse_wheel_event_manager.cc:41-125
    pub fn HandleWheelEvent(&self, event: &WebMouseWheelEvent) -> WebInputEventResult {
        let Some(target_context) = self.context_.retained_target.as_ref() else {
            return WebInputEventResult::kNotHandled;
        };
        let Some(document) = self.frame_.Document() else {
            return WebInputEventResult::kNotHandled;
        };
        if !document.HasLayoutView() {
            return WebInputEventResult::kNotHandled;
        }
        let Some(view) = self.frame_.View() else {
            return WebInputEventResult::kNotHandled;
        };
        struct FrameTargetQuery<'a> {
            document: &'a dyn WheelDocument,
            view: &'a dyn WheelFrameView,
            nodes: &'a dyn WheelNodeQueries,
            dispatch: &'a dyn WheelDOMDispatch,
        }
        impl WheelTargetQuery for FrameTargetQuery<'_> {
            fn Document(&self) -> &dyn WheelDocument {
                self.document
            }
            fn View(&self) -> &dyn WheelFrameView {
                self.view
            }
            fn Nodes(&self) -> &dyn WheelNodeQueries {
                self.nodes
            }
            fn TargetWindow(&self, target: &EventTargetRef) -> WheelEventWindow {
                let _ = target;
                self.document.Window()
            }
            fn IsVerticalScrollEnforced(&self, target: &EventTargetRef) -> bool {
                let _ = target;
                self.document.IsVerticalScrollEnforced()
            }
            fn DispatchEvent(
                &self,
                target: &EventTargetRef,
                event: &EventRef,
            ) -> DispatchEventResult {
                self.dispatch.DispatchEvent(target, event)
            }
        }
        self.HandleWheelEventWithTargetQuery(
            event,
            &FrameTargetQuery {
                document: document.as_ref(),
                view: view.as_ref(),
                nodes: target_context.nodes.as_ref(),
                dispatch: target_context.dispatch.as_ref(),
            },
        )
    }

    /// The same source transaction with call-scoped target-query owners. This
    /// permits a Page to lend its current resident fragments without retaining
    /// a copied hit-test tree or changing target-latch ownership.
    // Source: mouse_wheel_event_manager.cc:41-125
    pub fn HandleWheelEventWithTargetQuery(
        &self,
        event: &WebMouseWheelEvent,
        target_query: &dyn WheelTargetQuery,
    ) -> WebInputEventResult {
        if !target_query.Document().HasLayoutView() {
            return WebInputEventResult::kNotHandled;
        }
        let ended_mask = WebMouseWheelEvent::kPhaseEnded.0 | WebMouseWheelEvent::kPhaseCancelled.0;
        if event.phase.0 & ended_mask != 0 || event.momentum_phase.0 & ended_mask != 0 {
            self.UpdateWheelTarget(None);
            return WebInputEventResult::kNotHandled;
        }

        if let Some(locked) = self.frame_.PointerLockedElement() {
            self.UpdateWheelTarget(Some(locked));
        } else {
            let has_phase_info = event.phase != WebMouseWheelEvent::kPhaseNone
                || event.momentum_phase != WebMouseWheelEvent::kPhaseNone;
            let started_mask =
                WebMouseWheelEvent::kPhaseBegan.0 | WebMouseWheelEvent::kPhaseMayBegin.0;
            let needs_target = event.phase.0 & started_mask != 0
                || self.wheel_target_.borrow().is_none()
                || !has_phase_info;
            if needs_target {
                self.UpdateWheelTarget(self.FindTargetNode(event, target_query));
            }
        }

        let target = self.wheel_target_.borrow().clone();
        if let Some(subframe) = self.context_.routing.SubframeForTargetNode(target.as_ref()) {
            return subframe.HandleWheelEvent(event);
        }
        if event.phase == WebMouseWheelEvent::kPhaseMayBegin {
            self.FadeInChainedScrollbarsAndDeferFadeOut();
            return WebInputEventResult::kNotHandled;
        }
        self.FadeOutScrollbarsIfNeeded();

        // Read the current target after fade-out, just as the source member is
        // read here. No RefCell borrow crosses the synchronous DOM dispatch.
        let target = self.wheel_target_.borrow().clone();
        if let Some(target) = target {
            let dom_event = Rc::new(RefCell::new(WheelEvent::CreateFromNative(
                event,
                &target_query.TargetWindow(&target),
            )));
            let dispatch_event = EventRef::FromShared(Rc::clone(&dom_event));
            // This fact must survive removal of the target by its listener.
            let should_enforce_vertical_scroll = target_query.IsVerticalScrollEnforced(&target);
            let result = target_query.DispatchEvent(&target, &dispatch_event);
            if result != DispatchEventResult::kNotCanceled {
                self.UpdateWheelTarget(None);
                let is_vertical =
                    dom_event.borrow().NativeEvent().event_action == EventAction::kScrollVertical;
                return if should_enforce_vertical_scroll && is_vertical {
                    WebInputEventResult::kNotHandled
                } else {
                    ToWebInputEventResult(result)
                };
            }
        }
        WebInputEventResult::kNotHandled
    }

    // Source: mouse_wheel_event_manager.cc:127-131
    pub fn ElementRemoved(&self, target: &EventTargetRef) {
        let is_target = self
            .wheel_target_
            .borrow()
            .as_ref()
            .is_some_and(|current| Rc::ptr_eq(current, target));
        if is_target {
            self.UpdateWheelTarget(None);
        }
    }

    // Source: mouse_wheel_event_manager.cc:133-160
    fn FindTargetNode(
        &self,
        event: &WebMouseWheelEvent,
        target_query: &dyn WheelTargetQuery,
    ) -> Option<EventTargetRef> {
        let document = target_query.Document();
        let view = target_query.View();
        let nodes = target_query.Nodes();
        debug_assert!(document.HasLayoutView());
        let root = event.PositionInRootFrame();
        let floored = gfx::Point::new(root.x().floor() as i32, root.y().floor() as i32);
        let point = view.ConvertFromRootFrame(floored);
        let request =
            HitTestRequest::<(), ()>::new(HitTestRequest::<(), ()>::kReadOnly, None, None);
        let location = HitTestLocation::FromPhysicalOffset(point);
        let result = document.HitTest(&location, &request);
        let mut node = result.inner_node;
        if node.as_ref().is_some_and(|node| nodes.IsTextNode(node)) {
            node = node.as_ref().and_then(|node| nodes.FlatTreeParent(node));
            while let Some(current) = node.as_ref() {
                if nodes.HasLayoutObject(current) {
                    break;
                }
                node = nodes.FlatTreeParent(current);
            }
        }
        if node.is_none() && result.over_scrollbar {
            node = document.DocumentElement();
        }
        node
    }

    // Source: mouse_wheel_event_manager.cc:162-187
    fn UpdateWheelTarget(&self, target: Option<EventTargetRef>) {
        let old_target = self.wheel_target_.borrow().clone();
        let unchanged = match (&old_target, &target) {
            (None, None) => true,
            (Some(old), Some(new)) => Rc::ptr_eq(old, new),
            _ => false,
        };
        if unchanged {
            return;
        }
        if let Some(subframe) = self
            .context_
            .routing
            .SubframeForTargetNode(old_target.as_ref())
        {
            subframe.ClearWheelTarget();
        } else {
            self.FadeOutScrollbarsIfNeeded();
        }
        *self.wheel_target_.borrow_mut() = target;
        if self.wheel_target_.borrow().is_none() {
            if let Some(parent) = self.frame_.ParentLocalWheelRoute() {
                parent.ClearWheelTarget();
            }
        }
    }

    // Source: mouse_wheel_event_manager.cc:189-243
    fn FadeInChainedScrollbarsAndDeferFadeOut(&self) {
        if !self
            .context_
            .features
            .fade_in_scrollbar_when_mouse_wheel_may_begin
        {
            return;
        }
        const kLeft: u8 = 1 << 0;
        const kRight: u8 = 1 << 1;
        const kUp: u8 = 1 << 2;
        const kDown: u8 = 1 << 3;
        const kAllDirections: u8 = kLeft | kRight | kUp | kDown;
        let mut consumed = 0;
        let target = self.wheel_target_.borrow().clone();
        for scrollable in self.context_.scrollables.ScrollableAreas(target.as_ref()) {
            if consumed == kAllDirections {
                break;
            }
            let offset = scrollable.ScrollOffset();
            let minimum = scrollable.MinimumScrollOffset();
            let maximum = scrollable.MaximumScrollOffset();
            let mut horizontal = false;
            let mut vertical = false;
            if consumed & kLeft == 0 && offset.0 > minimum.0 {
                horizontal = true;
                consumed |= kLeft;
            }
            if consumed & kRight == 0 && offset.0 < maximum.0 {
                horizontal = true;
                consumed |= kRight;
            }
            if consumed & kUp == 0 && offset.1 > minimum.1 {
                vertical = true;
                consumed |= kUp;
            }
            if consumed & kDown == 0 && offset.1 < maximum.1 {
                vertical = true;
                consumed |= kDown;
            }
            if !horizontal && !vertical {
                continue;
            }
            if scrollable.FadeInScrollbarIfExists(horizontal, vertical)
                && self
                    .context_
                    .features
                    .defer_fade_out_scrollbar_until_mouse_wheel_ended
            {
                let identity = Rc::as_ptr(&scrollable) as *const () as usize;
                self.fade_out_deferred_scrollables_
                    .borrow_mut()
                    .insert(identity, Rc::downgrade(&scrollable));
            }
        }
    }

    // Source: mouse_wheel_event_manager.cc:245-252
    fn FadeOutScrollbarsIfNeeded(&self) {
        // Copy only weak set entries so a scrollbar effect never holds an
        // interaction-state borrow. Scrollable owners/trees are not copied.
        let pending: Vec<_> = self
            .fade_out_deferred_scrollables_
            .borrow()
            .values()
            .cloned()
            .collect();
        for scrollable in pending {
            if let Some(scrollable) = scrollable.upgrade() {
                scrollable.FadeOutScrollbarIfNeeded();
            }
        }
        self.fade_out_deferred_scrollables_.borrow_mut().clear();
    }
}

impl<M: ?Sized> WheelTargetRoute for MouseWheelEventManager<M> {
    fn HandleWheelEvent(&self, event: &WebMouseWheelEvent) -> WebInputEventResult {
        MouseWheelEventManager::HandleWheelEvent(self, event)
    }
    fn ClearWheelTarget(&self) {
        self.UpdateWheelTarget(None);
    }
}
