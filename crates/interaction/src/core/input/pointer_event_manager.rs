// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/input/pointer_event_manager.h:25-326
// Source: third_party/blink/renderer/core/input/pointer_event_manager.cc:48-1571
// Field-trial strings at .cc:42-46 and resolution at .cc:81-92 are supplied
// as resolved construction inputs. Includes, namespace and GC tracing are
// represented by Rust modules and shared/weak concrete owners.
// Ports retain the existing DOM, frame, touch and mouse owners. They
// are dependency projections, not replacement implementations of those owners.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::{Rc, Weak};

use interaction_dispatch::core::dom::events::event::{EventRef, EventTargetKind, EventTargetRef};
use interaction_dispatch::core::dom::events::event_dispatch_result::DispatchEventResult;
use interaction_dispatch::core::events::pointer_event::PointerEventRef;
use interaction_dispatch::core::events::pointer_event_factory::{
    PointerEventFactory, PointerPseudoElement, PointerTarget, PointerTargetRef,
};
use interaction_dispatch::core::events::ui_event::AbstractViewRef;
use interaction_dispatch::core::input::boundary_event_dispatcher::{
    BoundaryEventDOM, BoundaryEventDispatch, BoundaryEventDispatcher,
};
use interaction_dispatch::core::input::event_handling_util::{
    MergeEventResult, ToWebInputEventResult, WebInputEventResult,
};
use interaction_input::base::time::time::TimeTicks;
use interaction_input::public::common::input::pointer_id::PointerId;
use interaction_input::public::common::input::web_input_event::{Type, WebInputEvent};
use interaction_input::public::common::input::web_mouse_event::WebMouseEvent;
use interaction_input::public::common::input::web_pointer_event::WebPointerEvent;
use interaction_input::public::common::input::web_pointer_properties::{PointF, PointerType};

use super::mouse_event_manager::{MouseEventManagerState, MousePointerTarget};

// Source: pointer_event_manager.h:25-28
pub const kStylusWritableAdjustmentSizeDip: usize = 30;

// Source: pointer_event_manager.cc:99,343-347,693-695,1506-1509,1555-1557
pub trait PointerTouchManager {
    fn Clear(&self);
    fn HandlePseudoElementRemoval(&self, pseudo: &EventTargetRef);
    fn FlushEvents(&self) -> WebInputEventResult;
    fn IsAnyTouchActive(&self) -> bool;
    fn CurrentTouchDownElement(&self) -> Option<EventTargetRef>;
    // Source: pointer_event_manager.cc:816-817,852-861,867-868
    fn GetTouchPointerNode(
        &self,
        event: &WebPointerEvent,
        target: &PointerEventTarget,
    ) -> Option<EventTargetRef>;
    fn UpdateTouchAttributeMapsForPointerDown(
        &self,
        event: &WebPointerEvent,
        node: &EventTargetRef,
        action: PointerTouchAction,
    );
    fn HandleTouchPoint(
        &self,
        event: &WebPointerEvent,
        coalesced: &[WebPointerEvent],
        target: &PointerEventTarget,
    );
}

// Source: pointer_event_manager.cc:100,261,1012-1015,1214,1221-1226,
// 1274-1284,1360
pub trait PointerMouseManager {
    fn Clear(&self);
    fn SetElementUnderMouse(&self, target: Option<EventTargetRef>, event: &WebMouseEvent);
    fn GetElementUnderMouse(&self) -> Option<EventTargetRef>;
    fn DispatchMouseEvent(
        &self,
        target: Option<EventTargetRef>,
        event_type: &str,
        event: &WebMouseEvent,
        last_position: &PointF,
    ) -> (Option<EventRef>, WebInputEventResult);
    fn DispatchMouseClickIfNeeded(
        &self,
        target: Option<EventTargetRef>,
        captured_target: Option<EventTargetRef>,
        event: &WebMouseEvent,
        pointer_id: PointerId,
        pointer_type: &str,
        down_target: Option<PointerTargetRef>,
        up_target: Option<PointerTargetRef>,
    );
}

// Source: pointer_event_manager.cc:52-55,174-223,284-289,315-369,580-600,
// 1081-1095,1173-1183,1261-1264,1382-1407,1499-1503,1537-1538
// Read-only queries borrow the live owning subsystems. Synchronous operations
// below may execute script and reenter this manager: no RefCell or event borrow
// may be retained across them.
pub trait PointerEventDOMPort: BoundaryEventDOM {
    fn CoordinateContext(
        &self,
    ) -> interaction_dispatch::core::events::mouse_event::MouseCoordinateContext;
    fn HasPointerEventHandlers(&self) -> bool;
    fn HasPointerRawUpdateEventHandlers(&self) -> bool;
    fn HasEventListeners(&self, target: &EventTargetRef, event_type: &str) -> bool;
    fn IsElement(&self, target: &EventTargetRef) -> bool;
    fn ParentNode(&self, target: &EventTargetRef) -> Option<EventTargetRef>;
    fn ParentOrShadowHostElement(&self, target: &EventTargetRef) -> Option<EventTargetRef>;
    fn IsShadowIncludingInclusiveAncestorOf(
        &self,
        ancestor: &EventTargetRef,
        target: &EventTargetRef,
    ) -> bool;
    fn OwnerDocument(&self, target: &EventTargetRef) -> EventTargetRef;
    fn FrameForElement(&self, target: &EventTargetRef) -> Option<Rc<()>>;
    fn View(&self) -> Option<AbstractViewRef>;
    fn ViewForElement(&self, target: &EventTargetRef) -> Option<AbstractViewRef>;
    fn IsSecureContext(&self) -> bool;
    fn NeedsUnbufferedInputEvents(&self, target: &EventTargetRef) -> bool;
    fn RequestUnbufferedInputEvents(&self);
    fn AnchorElementInteractionTrackerOnPointerEvent(
        &self,
        target: &EventTargetRef,
        event: &PointerEventRef,
    );
    fn HandlePopoverLightDismiss(&self, event: &PointerEventRef, target: &EventTargetRef);
    fn HandleDialogLightDismiss(&self, event: &PointerEventRef, target: &EventTargetRef);
    fn HandlePointerEventsForInterestFor(&self, target: &EventTargetRef, event_type: &str);
    // Concrete virtual dispatch needs PointerEvent::DispatchHook, including
    // its coalesced/predicted target updates. event_ref projects that very
    // same owner; implementations release the concrete borrow before entry.
    fn DispatchEvent(
        &self,
        target: &EventTargetRef,
        pointer_event: &PointerEventRef,
        event_ref: EventRef,
    ) -> DispatchEventResult;
    fn HitTestTouchPointer(&self, event: &WebPointerEvent) -> PointerEventTarget;
    fn HitTestMouseRelease(&self, event: &WebMouseEvent) -> Option<EventTargetRef>;
    fn NodeEventContexts(&self, event: &EventRef) -> Rc<dyn PointerNodeEventContexts>;
    fn AppendTouchIdForCanceledPointerDown(&self, frame: &Rc<()>, unique_touch_event_id: u32);
}

// Source: pointer_event_manager.cc:1090-1096
// Contexts belong to the dispatched EventPath. Indexed projection avoids
// constructing a second vector of nodes from its resident NodeEventContexts.
pub trait PointerNodeEventContexts {
    fn Len(&self) -> usize;
    fn GetNode(&self, index: usize) -> EventTargetRef;
}

// Source: cc/input/touch_action.h:22-58
// The canonical bit value crosses this port unchanged. This local value-only
// projection has no touch-action computation or independent policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerTouchAction(pub u32);
impl PointerTouchAction {
    pub const kNone: Self = Self(0);
    pub const kPan: Self = Self(0x0f);
    pub const kInternalPanXScrolls: Self = Self(0x40);
    pub const kInternalNotWritable: Self = Self(0x80);
    pub const kAuto: Self = Self(0xff);
}

// Source: third_party/blink/public/mojom/input/input_handler.mojom:239-249
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerPanAction {
    kNone,
    kScroll,
    kMoveCursorOrScroll,
    kStylusWritable,
}

// Source: pointer_event_manager.cc:884-898
pub trait PointerScrollbar {
    fn HandlePointerEvent(&self, event: &WebPointerEvent) -> bool;
}

// Source: pointer_event_manager.cc:920-945
// This opaque transform is the area owner's canonical gfx::Transform value;
// no matrix or scrollable-area model is retained by the manager.
pub trait PointerResizeTransform {
    fn Transform(&self) -> &foundation::gfx::Transform;
}
pub trait PointerResizeArea {
    fn IsAbsolutePointInResizeControl(&self, point: PointF) -> bool;
    fn SetInResizeMode(&self, value: bool);
    fn InResizeMode(&self) -> bool;
    fn HasLayerAndLayoutBox(&self) -> bool;
    fn InitializeResizeTransform(&self, point: PointF) -> Rc<dyn PointerResizeTransform>;
    fn Resize(&self, point: PointF, transform: &dyn PointerResizeTransform);
}

// Source: pointer_event_manager.cc:503-542
// The hit-test owner retains its existing location/result pair and returns only
// its narrow result owner. The manager never stores or duplicates a hit tree.
pub trait PointerAdjustmentHitTest {
    fn AsAny(&self) -> &dyn std::any::Any;
}
#[derive(Clone, Copy)]
pub enum PointerAdjustmentCandidateType {
    kClickable,
    kStylusWritable,
}
pub struct PointerAdjustment {
    pub point: PointF,
    pub node: Option<EventTargetRef>,
}

// Source: pointer_event_manager.cc:486-542,726-733,790-815,834-862,
// 886-888,906-925,1038-1067
// The embedding supplies native/frame and existing hit-test/touch-action leaf
// operations. All event ordering and manager-owned state stay in this module.
pub trait PointerEventEnvironment {
    fn CurrentFrame(&self) -> Rc<()>;
    fn PointerLockedElement(&self) -> Option<EventTargetRef>;
    fn HandlePointerEventOnFrame(
        &self,
        frame: &Rc<()>,
        event: &WebPointerEvent,
        coalesced: &[WebPointerEvent],
        predicted: &[WebPointerEvent],
    );
    fn ShouldDiscardEventTargetingFrame(&self, event: &WebPointerEvent, frame: &Rc<()>) -> bool;
    fn DomNodeId(&self, node: &EventTargetRef) -> i32;
    fn NotifyUserActivation(&self, frame: &Rc<()>);
    fn EffectiveTouchActionAtPointerDown(
        &self,
        event: &WebPointerEvent,
        node: &EventTargetRef,
    ) -> PointerTouchAction;
    fn EffectiveTouchActionAtPointer(
        &self,
        event: &WebPointerEvent,
        node: &EventTargetRef,
    ) -> PointerTouchAction;
    fn SetTouchAction(&self, action: PointerTouchAction);
    fn SetPanAction(&self, action: PointerPanAction);
    fn EnclosingLayerScrollableArea(
        &self,
        node: &EventTargetRef,
    ) -> Option<Rc<dyn PointerResizeArea>>;
    fn ConvertFromRootFrame(&self, frame: &Rc<()>, point: PointF) -> PointF;
    fn DeviceScaleFactor(&self) -> f32;
    fn PageScaleFactor(&self) -> f32;
    fn HandwritingRadius(&self) -> i32;
    // LayoutUnit rounding and source GetHitTestRectForAdjustment occur at this
    // typed geometry boundary, before the manager's emptiness check.
    fn GetHitTestRectForAdjustment(&self, width: f32, height: f32) -> (f32, f32);
    // Performs root-frame kTouchEvent|kReadOnly|kActive|kListBased hit testing,
    // at the rounded input point minus half the supplied hit rectangle.
    fn HitTestAdjustmentRect(
        &self,
        point: PointF,
        size: (f32, f32),
    ) -> Rc<dyn PointerAdjustmentHitTest>;
    fn BestNodeForHitTestResult(
        &self,
        candidate_type: PointerAdjustmentCandidateType,
        hit: &dyn PointerAdjustmentHitTest,
    ) -> Option<PointerAdjustment>;
    fn CacheTouchAdjustmentResult(&self, unique_touch_event_id: u32, point: PointF);
}

// Source: pointer_event_manager.cc:567-604,606-647
#[derive(Default)]
pub struct PointerEventTarget {
    pub target_element: Option<EventTargetRef>,
    pub target_frame: Option<Rc<()>>,
    pub scrollbar: Option<Rc<dyn PointerScrollbar>>,
}

// Source: pointer_event_manager.cc:81-91,175,281-282,321-322,352-354,
// 419-435,1177-1183,1213,1265-1266,1296-1300
#[derive(Clone, Copy, Default)]
pub struct PointerEventFeatures {
    pub skip_touch_filter_discrete: bool,
    pub skip_touch_filter_all: bool,
    pub light_dismiss_from_click: bool,
    pub boundary_event_dispatch_tracks_node_removal: bool,
    pub suppress_pointer_stream_after_drag: bool,
    pub pointer_raw_update_only_in_secure_context: bool,
}

/// Call-scoped services for Chromium's mouse pointer down/up transaction. The
/// retained manager below owns factory, compatibility suppression and capture;
/// the Page adapter only projects the current DOM dispatcher.
// Source: pointer_event_manager.cc:1103-1305,1327-1410
pub trait MousePointerEventDispatchPort {
    fn View(&self) -> Option<AbstractViewRef>;
    fn IsInDocument(&self, target: &EventTargetRef) -> bool;
    fn OwnerDocument(&self, target: &EventTargetRef) -> EventTargetRef;
    fn HasPointerEventListeners(&self, target: &EventTargetRef, event_type: &str) -> bool;
    fn CoordinateContext(
        &self,
    ) -> interaction_dispatch::core::events::mouse_event::MouseCoordinateContext;
    fn DispatchPointerEvent(
        &self,
        target: &EventTargetRef,
        event: &PointerEventRef,
    ) -> WebInputEventResult;
    fn DispatchMouseEvent(
        &self,
        mouse: &Rc<MouseEventManagerState>,
        target: &EventTargetRef,
        event_type: &str,
        event: &WebMouseEvent,
        last_position: PointF,
    ) -> WebInputEventResult;
    fn SetElementUnderMouse(
        &self,
        mouse: &Rc<MouseEventManagerState>,
        target: Option<&EventTargetRef>,
        event: &WebMouseEvent,
    );
    fn DispatchMouseClickIfNeeded(
        &self,
        mouse: &Rc<MouseEventManagerState>,
        target: Option<&EventTargetRef>,
        captured_target: Option<&EventTargetRef>,
        event: &WebMouseEvent,
        pointer_id: PointerId,
        pointer_type: &str,
        down_target: Option<&dyn MousePointerTarget>,
        up_target: Option<&dyn MousePointerTarget>,
    );
}

impl MousePointerTarget for PointerTargetRef {
    fn Node(&self) -> Option<EventTargetRef> {
        self.borrow().node.as_ref().and_then(Weak::upgrade)
    }

    fn ClientPosition(&self) -> (f64, f64) {
        let target = self.borrow();
        (target.client_x, target.client_y)
    }
}

// Source: pointer_event_manager.h:161-180; pointer_event_manager.cc:128-149
struct MousePointerBoundaryEventDispatcher<'a> {
    manager: &'a MousePointerEventManager,
    dispatch: &'a dyn MousePointerEventDispatchPort,
    pointer_event: &'a PointerEventRef,
}

impl BoundaryEventDispatch for MousePointerBoundaryEventDispatcher<'_> {
    fn Dispatch(
        &self,
        target: &EventTargetRef,
        related_target: Option<&EventTargetRef>,
        event_type: &str,
        check_for_listener: bool,
    ) {
        let event = self
            .manager
            .factory
            .borrow()
            .as_ref()
            .expect("pointer factory initialized")
            .CreatePointerBoundaryEvent(
                &self.pointer_event.borrow(),
                event_type,
                related_target.cloned(),
            );
        if !check_for_listener || self.dispatch.HasPointerEventListeners(target, event_type) {
            self.dispatch.DispatchPointerEvent(target, &event);
        }
    }
}

/// The retained mouse portion of PointerEventManager used by production while
/// touch/pen assembly is still pending. This is one owner for PointerEventFactory,
/// mouse pointer boundary/position state, capture and compatibility-mouse
/// suppression across move/down/up events.
// Source: pointer_event_manager.h:161-180,273-309;
// Source: pointer_event_manager.cc:128-149,228-305,1103-1305,1327-1473,1493-1504
pub struct PointerEventManagerState {
    factory: RefCell<Option<PointerEventFactory>>,
    element_under_pointer_: RefCell<HashMap<PointerId, EventTargetRef>>,
    original_element_under_pointer_removed_: RefCell<HashSet<PointerId>>,
    pointer_capture_target_: RefCell<HashMap<PointerId, EventTargetRef>>,
    pending_pointer_capture_target_: RefCell<HashMap<PointerId, EventTargetRef>>,
    prevent_mouse_event_for_pointer_type_: RefCell<[bool; PointerType::kMaxValue as usize + 1]>,
    light_dismiss_from_click_: bool,
    resize_scrollable_area_: RefCell<Option<Weak<dyn PointerResizeArea>>>,
    resize_position_to_size_transform_: RefCell<Option<Rc<dyn PointerResizeTransform>>>,
    captured_scrollbar_: RefCell<Option<Weak<dyn PointerScrollbar>>>,
    handwriting_radius_: Cell<i32>,
    skip_touch_filter_discrete_: Cell<bool>,
    skip_touch_filter_all_: Cell<bool>,
    discarded_event_: Cell<(Option<i32>, TimeTicks)>,
    non_hovering_pointers_canceled_: Cell<bool>,
    touch_ids_for_canceled_pointerdowns_: RefCell<VecDeque<u32>>,
    source_features_initialized: Cell<bool>,
}

// Existing mouse assembly retains the same canonical source owner while the
// complete manager borrows its call-scoped document/fragment service ports.
pub type MousePointerEventManager = PointerEventManagerState;

impl MousePointerEventManager {
    pub fn RemoveLastMousePosition(&self) {
        if let Some(factory) = self.factory.borrow_mut().as_mut() {
            factory.RemoveLastPosition(PointerEventFactory::kMouseId);
        }
    }
    /// ComputePointerEventTarget's capture lookup before dispatch is forwarded
    /// to another local Frame. Uses the original factory's incoming-id map.
    pub fn HasRoutingCapture(
        &self,
        properties: &interaction_input::public::common::input::web_pointer_properties::WebPointerProperties,
    ) -> bool {
        let factory = self.factory.borrow();
        let Some(factory) = factory.as_ref() else {
            return false;
        };
        let id = factory.GetPointerEventId(properties);
        self.pending_pointer_capture_target_
            .borrow()
            .contains_key(&id)
            || self.pointer_capture_target_.borrow().contains_key(&id)
    }
    // Source pointer_event_manager.cc:1484-1487.
    pub fn AppendTouchIdForCanceledPointerDown(&self, id: u32) {
        self.touch_ids_for_canceled_pointerdowns_
            .borrow_mut()
            .push_back(id);
    }
    pub fn new(light_dismiss_from_click: bool) -> Self {
        Self {
            factory: RefCell::new(None),
            element_under_pointer_: RefCell::new(HashMap::new()),
            original_element_under_pointer_removed_: RefCell::new(HashSet::new()),
            pointer_capture_target_: RefCell::new(HashMap::new()),
            pending_pointer_capture_target_: RefCell::new(HashMap::new()),
            prevent_mouse_event_for_pointer_type_: RefCell::new(
                [false; PointerType::kMaxValue as usize + 1],
            ),
            light_dismiss_from_click_: light_dismiss_from_click,
            resize_scrollable_area_: RefCell::new(None),
            resize_position_to_size_transform_: RefCell::new(None),
            captured_scrollbar_: RefCell::new(None),
            handwriting_radius_: Cell::new(kStylusWritableAdjustmentSizeDip as i32),
            skip_touch_filter_discrete_: Cell::new(false),
            skip_touch_filter_all_: Cell::new(false),
            discarded_event_: Cell::new((None, TimeTicks::new())),
            non_hovering_pointers_canceled_: Cell::new(false),
            touch_ids_for_canceled_pointerdowns_: RefCell::new(VecDeque::new()),
            source_features_initialized: Cell::new(false),
        }
    }

    pub fn Clear(&self) {
        self.factory.borrow_mut().take();
        self.element_under_pointer_.borrow_mut().clear();
        self.original_element_under_pointer_removed_
            .borrow_mut()
            .clear();
        self.pointer_capture_target_.borrow_mut().clear();
        self.pending_pointer_capture_target_.borrow_mut().clear();
        self.prevent_mouse_event_for_pointer_type_
            .borrow_mut()
            .fill(false);
        self.resize_scrollable_area_.borrow_mut().take();
        self.resize_position_to_size_transform_.borrow_mut().take();
        self.captured_scrollbar_.borrow_mut().take();
        self.discarded_event_.set((None, TimeTicks::new()));
        self.non_hovering_pointers_canceled_.set(false);
        self.touch_ids_for_canceled_pointerdowns_
            .borrow_mut()
            .clear();
        self.source_features_initialized.set(false);
    }

    fn EnsureFactory(&self, dispatch: &dyn MousePointerEventDispatchPort) {
        self.EnsureFactoryWithCoordinates(dispatch.CoordinateContext());
    }

    fn EnsureFactoryWithCoordinates(
        &self,
        coordinates: interaction_dispatch::core::events::mouse_event::MouseCoordinateContext,
    ) {
        if self.factory.borrow().is_none() {
            self.factory
                .borrow_mut()
                .replace(PointerEventFactory::new(coordinates));
        }
    }

    fn borrow(&self) -> std::cell::Ref<'_, PointerEventFactory> {
        std::cell::Ref::map(self.factory.borrow(), |factory| {
            factory
                .as_ref()
                .expect("canonical pointer factory initialized")
        })
    }
    fn borrow_mut(&self) -> std::cell::RefMut<'_, PointerEventFactory> {
        std::cell::RefMut::map(self.factory.borrow_mut(), |factory| {
            factory
                .as_mut()
                .expect("canonical pointer factory initialized")
        })
    }

    // Source: pointer_event_manager.cc:228-236
    fn GetEffectiveTargetForPointerEvent(
        &self,
        target: Option<EventTargetRef>,
        pointer_id: PointerId,
    ) -> Option<EventTargetRef> {
        self.pointer_capture_target_
            .borrow()
            .get(&pointer_id)
            .cloned()
            .or(target)
    }

    // Source: pointer_event_manager.cc:265-305
    fn SetElementUnderPointer(
        &self,
        dispatch: &dyn MousePointerEventDispatchPort,
        dom: &dyn BoundaryEventDOM,
        pointer_event: &PointerEventRef,
        target: Option<EventTargetRef>,
    ) {
        let pointer_id = pointer_event.borrow().pointerId();
        let exited = self
            .element_under_pointer_
            .borrow()
            .get(&pointer_id)
            .cloned();
        let original_exited_target_removed = self
            .original_element_under_pointer_removed_
            .borrow_mut()
            .remove(&pointer_id);
        match target.as_ref() {
            Some(target) if !same_target(exited.as_ref(), Some(target)) => {
                self.element_under_pointer_
                    .borrow_mut()
                    .insert(pointer_id, target.clone());
            }
            None if exited.is_some() => {
                self.element_under_pointer_.borrow_mut().remove(&pointer_id);
            }
            _ => {}
        }
        let sender = MousePointerBoundaryEventDispatcher {
            manager: self,
            dispatch,
            pointer_event,
        };
        BoundaryEventDispatcher::new("pointerover", "pointerout", "pointerenter", "pointerleave")
            .SendBoundaryEvents(
                dom,
                &sender,
                exited.as_ref(),
                original_exited_target_removed,
                target.as_ref(),
            );
    }

    // Source: pointer_event_manager.cc:1327-1347
    fn GetPointerCaptureState(
        &self,
        id: PointerId,
    ) -> (bool, Option<EventTargetRef>, Option<EventTargetRef>) {
        let capture = self.pointer_capture_target_.borrow().get(&id).cloned();
        let pending = self
            .pending_pointer_capture_target_
            .borrow()
            .get(&id)
            .cloned();
        (
            !same_target(capture.as_ref(), pending.as_ref()),
            capture,
            pending,
        )
    }

    // Source: pointer_event_manager.cc:1365-1410
    fn ProcessPendingPointerCapture(
        &self,
        dispatch: &dyn MousePointerEventDispatchPort,
        dom: &dyn BoundaryEventDOM,
        pointer: &PointerEventRef,
    ) {
        let id = pointer.borrow().pointerId();
        let (changed, capture, pending) = self.GetPointerCaptureState(id);
        if !changed {
            return;
        }
        if let Some(capture) = capture {
            let target = if dispatch.IsInDocument(&capture) {
                capture
            } else {
                dispatch.OwnerDocument(&capture)
            };
            self.pointer_capture_target_.borrow_mut().remove(&id);
            let lost = self
                .factory
                .borrow()
                .as_ref()
                .expect("pointer factory initialized")
                .CreatePointerCaptureEvent(&pointer.borrow(), "lostpointercapture");
            dispatch.DispatchPointerEvent(&target, &lost);
        }
        if let Some(pending) = pending.filter(|target| dispatch.IsInDocument(target)) {
            self.SetElementUnderPointer(dispatch, dom, pointer, Some(pending.clone()));
            let got = self
                .factory
                .borrow()
                .as_ref()
                .expect("pointer factory initialized")
                .CreatePointerCaptureEvent(&pointer.borrow(), "gotpointercapture");
            dispatch.DispatchPointerEvent(&pending, &got);
            if dispatch.IsInDocument(&pending) {
                self.pointer_capture_target_
                    .borrow_mut()
                    .insert(id, pending);
            } else {
                let lost = self
                    .factory
                    .borrow()
                    .as_ref()
                    .expect("pointer factory initialized")
                    .CreatePointerCaptureEvent(&pointer.borrow(), "lostpointercapture");
                dispatch.DispatchPointerEvent(&dispatch.OwnerDocument(&pending), &lost);
            }
        }
    }

    // Source: pointer_event_manager.cc:1349-1363
    fn ProcessCaptureAndPositionOfPointerEvent(
        &self,
        dispatch: &dyn MousePointerEventDispatchPort,
        dom: &dyn BoundaryEventDOM,
        mouse: &Rc<MouseEventManagerState>,
        pointer: &PointerEventRef,
        hit_test_target: Option<EventTargetRef>,
        mouse_event: &WebMouseEvent,
    ) -> Option<EventTargetRef> {
        self.ProcessPendingPointerCapture(dispatch, dom, pointer);
        let id = pointer.borrow().pointerId();
        let effective = self.GetEffectiveTargetForPointerEvent(hit_test_target, id);
        self.SetElementUnderPointer(dispatch, dom, pointer, effective.clone());
        dispatch.SetElementUnderMouse(mouse, effective.as_ref(), mouse_event);
        effective
    }

    // Source: pointer_event_manager.cc:238-305
    pub fn SendMousePointerBoundaryEvents(
        &self,
        dispatch: &dyn MousePointerEventDispatchPort,
        dom: &dyn BoundaryEventDOM,
        entered_element: Option<EventTargetRef>,
        mouse_event: &WebMouseEvent,
    ) {
        self.EnsureFactory(dispatch);
        let web_pointer_event = WebPointerEvent::FromMouse(Type::kPointerMove, mouse_event);
        let pointer = self
            .factory
            .borrow_mut()
            .as_mut()
            .expect("pointer factory initialized")
            .Create(&web_pointer_event, &[], &[], dispatch.View())
            .expect("mouse pointer event");
        let (id, reset) = {
            let pointer = pointer.borrow();
            (
                pointer.pointerId(),
                pointer.buttons() == 0 && pointer.isPrimary(),
            )
        };
        if reset {
            self.prevent_mouse_event_for_pointer_type_.borrow_mut()
                [ToPointerTypeIndex(mouse_event.pointer_properties.pointer_type)] = false;
        }
        self.ProcessPendingPointerCapture(dispatch, dom, &pointer);
        let effective = self.GetEffectiveTargetForPointerEvent(entered_element, id);
        self.SetElementUnderPointer(dispatch, dom, &pointer, effective);
    }

    // Source: pointer_event_manager.cc:307-369
    pub fn NodeWillBeRemovedWithRelations(
        &self,
        relations: &dyn super::mouse_event_manager::MouseRemovalNodeRelations,
        boundary_event_dispatch_tracks_node_removal: bool,
        node: &EventTargetRef,
    ) {
        if !boundary_event_dispatch_tracks_node_removal {
            return;
        }
        let remaining = relations.ParentNode(node);
        let remaining_element = remaining.and_then(|remaining| {
            if relations.IsElement(&remaining) {
                Some(remaining)
            } else {
                relations.ParentOrShadowHostElement(&remaining)
            }
        });
        self.element_under_pointer_
            .borrow_mut()
            .retain(|id, element| {
                if !relations.IsShadowIncludingInclusiveAncestorOf(node, element) {
                    return true;
                }
                self.original_element_under_pointer_removed_
                    .borrow_mut()
                    .insert(*id);
                if let Some(remaining) = &remaining_element {
                    *element = remaining.clone();
                    true
                } else {
                    false
                }
            });
    }

    // Source: pointer_event_manager.cc:1412-1434
    pub fn ElementRemoved(&self, target: &EventTargetRef) {
        self.pending_pointer_capture_target_
            .borrow_mut()
            .retain(|_, value| !Rc::ptr_eq(value, target));
    }

    /// Source-aligned mouse move/down/up path. Touch/pen input continues to use
    /// the complete PointerEventManager once its TouchEventManager owner exists.
    // Source: pointer_event_manager.cc:1103-1305
    pub fn SendMousePointerEvent(
        &self,
        dispatch: &dyn MousePointerEventDispatchPort,
        dom: &dyn BoundaryEventDOM,
        mouse: &Rc<MouseEventManagerState>,
        target: EventTargetRef,
        event_type: Type,
        mouse_event: &WebMouseEvent,
    ) -> WebInputEventResult {
        debug_assert!(matches!(
            event_type,
            Type::kPointerDown | Type::kPointerMove | Type::kPointerUp
        ));
        self.EnsureFactory(dispatch);
        let web_pointer = WebPointerEvent::FromMouse(event_type, mouse_event);
        let pointer_type_index = ToPointerTypeIndex(mouse_event.pointer_properties.pointer_type);
        let (pointer, last) = {
            let mut factory = self.factory.borrow_mut();
            let factory = factory.as_mut().expect("pointer factory initialized");
            let pointer_id = factory.GetPointerEventId(&mouse_event.pointer_properties);
            let last = factory.GetLastPointerPosition(
                pointer_id,
                &mouse_event.pointer_properties,
                event_type,
            );
            let pointer = factory
                .Create(&web_pointer, &[], &[], dispatch.View())
                .expect("mouse pointer event");
            (pointer, last)
        };
        let pointer_id = pointer.borrow().pointerId();
        let is_primary = pointer.borrow().isPrimary();
        if event_type == Type::kPointerMove && pointer.borrow().buttons() == 0 {
            self.ReleasePointerCaptureById(pointer_id);
            self.ProcessPendingPointerCapture(dispatch, dom, &pointer);
            if is_primary {
                self.prevent_mouse_event_for_pointer_type_.borrow_mut()[pointer_type_index] = false;
            }
        }
        let effective_target = self
            .ProcessCaptureAndPositionOfPointerEvent(
                dispatch,
                dom,
                mouse,
                &pointer,
                Some(target.clone()),
                mouse_event,
            )
            .unwrap_or_else(|| target.clone());
        if self.light_dismiss_from_click_
            && matches!(event_type, Type::kPointerDown | Type::kPointerUp)
        {
            let pointer = pointer.borrow();
            let pointer_target = Rc::new(RefCell::new(PointerTarget::new(
                Some(&effective_target),
                pointer.clientX(),
                pointer.clientY(),
            )));
            let mut factory = self.factory.borrow_mut();
            let factory = factory.as_mut().expect("pointer factory initialized");
            if event_type == Type::kPointerDown {
                factory.SetPointerDownTarget(pointer_id, Some(pointer_target));
            } else {
                factory.SetPointerUpTarget(pointer_id, Some(pointer_target));
            }
        }

        let mut result = dispatch.DispatchPointerEvent(&effective_target, &pointer);
        if result != WebInputEventResult::kNotHandled
            && pointer.borrow().r#type() == "pointerdown"
            && is_primary
        {
            self.prevent_mouse_event_for_pointer_type_.borrow_mut()[pointer_type_index] = true;
        }
        let send_compat =
            is_primary && !self.prevent_mouse_event_for_pointer_type_.borrow()[pointer_type_index];
        if send_compat {
            result = MergeEventResult(
                result,
                dispatch.DispatchMouseEvent(
                    mouse,
                    &effective_target,
                    MouseEventNameForPointerEventInputType(event_type),
                    mouse_event,
                    last,
                ),
            );
        }

        if event_type == Type::kPointerUp && pointer.borrow().r#type() == "pointerup" {
            let captured_click_target = self
                .pointer_capture_target_
                .borrow()
                .get(&pointer_id)
                .cloned();
            self.ReleasePointerCaptureById(pointer_id);
            self.prevent_mouse_event_for_pointer_type_.borrow_mut()[pointer_type_index] = false;
            let (down_target, up_target) = {
                let factory = self.factory.borrow();
                let factory = factory.as_ref().expect("pointer factory initialized");
                (
                    factory.GetPointerDownTarget(pointer_id),
                    factory.GetPointerUpTarget(pointer_id),
                )
            };
            let pointer_type = pointer.borrow().pointerType().to_owned();
            self.ProcessPendingPointerCapture(dispatch, dom, &pointer);
            dispatch.DispatchMouseClickIfNeeded(
                mouse,
                Some(&effective_target),
                captured_click_target.as_ref(),
                mouse_event,
                pointer_id,
                &pointer_type,
                down_target
                    .as_ref()
                    .map(|target| target as &dyn MousePointerTarget),
                up_target
                    .as_ref()
                    .map(|target| target as &dyn MousePointerTarget),
            );
            self.factory
                .borrow_mut()
                .as_mut()
                .expect("pointer factory initialized")
                .RemovePointerTargets(pointer_id);
            self.ProcessCaptureAndPositionOfPointerEvent(
                dispatch,
                dom,
                mouse,
                &pointer,
                Some(target),
                mouse_event,
            );
        }
        result
    }

    // Source: pointer_event_manager.cc:1436-1443
    pub fn SetPointerCapture(&self, id: PointerId, target: EventTargetRef) -> bool {
        if self
            .factory
            .borrow()
            .as_ref()
            .is_some_and(|factory| factory.IsActiveButtonsState(id))
        {
            self.pending_pointer_capture_target_
                .borrow_mut()
                .insert(id, target);
            true
        } else {
            false
        }
    }

    // Source: pointer_event_manager.cc:1445-1459
    pub fn ReleasePointerCapture(&self, id: PointerId, target: &EventTargetRef) -> bool {
        if self.HasPointerCapture(id, target) {
            self.ReleasePointerCaptureById(id);
            true
        } else {
            false
        }
    }

    // Source: pointer_event_manager.cc:1461-1463
    pub fn ReleaseMousePointerCapture(&self) {
        self.ReleasePointerCaptureById(PointerEventFactory::kMouseId);
    }

    // Source: pointer_event_manager.cc:1465-1469
    pub fn HasPointerCapture(&self, id: PointerId, target: &EventTargetRef) -> bool {
        self.pending_pointer_capture_target_
            .borrow()
            .get(&id)
            .is_some_and(|value| Rc::ptr_eq(value, target))
    }

    // Source: pointer_event_manager.cc:1493-1504
    pub fn IsActive(&self, id: PointerId) -> bool {
        self.factory
            .borrow()
            .as_ref()
            .is_some_and(|factory| factory.IsActive(id))
    }

    // Source: pointer_event_manager.cc:1471-1473
    fn ReleasePointerCaptureById(&self, id: PointerId) {
        self.pending_pointer_capture_target_
            .borrow_mut()
            .remove(&id);
    }
}

impl Default for MousePointerEventManager {
    fn default() -> Self {
        Self::new(false)
    }
}

// Source: pointer_event_manager.h:156-159,273-309
// Interior mutability is limited to manager-owned scalar/maps. Every dispatch
// releases those borrows first, so set/release capture during a listener acts
// synchronously on this same manager and uses the same target identities.
pub struct PointerEventManager<'a> {
    dom: Rc<dyn PointerEventDOMPort + 'a>,
    factory: Rc<PointerEventManagerState>,
    touch: Rc<dyn PointerTouchManager + 'a>,
    mouse: Rc<dyn PointerMouseManager + 'a>,
    features: PointerEventFeatures,
    environment: Rc<dyn PointerEventEnvironment + 'a>,
}

impl std::ops::Deref for PointerEventManager<'_> {
    type Target = PointerEventManagerState;
    fn deref(&self) -> &Self::Target {
        self.factory.as_ref()
    }
}

// Source: pointer_event_manager.cc:48-50
fn ToPointerTypeIndex(t: PointerType) -> usize {
    t as usize
}

// Source: pointer_event_manager.cc:52-56
fn HasPointerEventListener(dom: &dyn PointerEventDOMPort) -> bool {
    dom.HasPointerEventHandlers() || dom.HasPointerRawUpdateEventHandlers()
}

// Source: pointer_event_manager.cc:58-70
fn MouseEventNameForPointerEventInputType(event_type: Type) -> &'static str {
    match event_type {
        Type::kPointerDown => "mousedown",
        Type::kPointerUp => "mouseup",
        Type::kPointerMove => "mousemove",
        _ => unreachable!("pointer input type has no compatibility mouse event"),
    }
}

impl<'a> PointerEventManager<'a> {
    // Source: pointer_event_manager.h:35-37; pointer_event_manager.cc:74-80
    // Features are the already resolved field-trial values, supplied at assembly.
    pub fn new(
        dom: Rc<dyn PointerEventDOMPort + 'a>,
        touch: Rc<dyn PointerTouchManager + 'a>,
        mouse: Rc<dyn PointerMouseManager + 'a>,
        environment: Rc<dyn PointerEventEnvironment + 'a>,
        features: PointerEventFeatures,
    ) -> Self {
        let state = Rc::new(PointerEventManagerState::new(
            features.light_dismiss_from_click,
        ));
        Self::WithState(state, dom, touch, mouse, environment, features)
    }

    pub fn WithState(
        state: Rc<PointerEventManagerState>,
        dom: Rc<dyn PointerEventDOMPort + 'a>,
        touch: Rc<dyn PointerTouchManager + 'a>,
        mouse: Rc<dyn PointerMouseManager + 'a>,
        environment: Rc<dyn PointerEventEnvironment + 'a>,
        features: PointerEventFeatures,
    ) -> Self {
        state.EnsureFactoryWithCoordinates(dom.CoordinateContext());
        let manager = Self {
            dom,
            factory: state,
            touch,
            mouse,
            environment,
            features,
        };
        // Source: pointer_event_manager.cc:81-92 (resolved field-trial values).
        if !manager.source_features_initialized.replace(true) {
            manager
                .skip_touch_filter_discrete_
                .set(features.skip_touch_filter_discrete);
            manager
                .skip_touch_filter_all_
                .set(features.skip_touch_filter_all);
        }
        manager
    }

    // Source: pointer_event_manager.cc:95-114
    pub fn Clear(&self) {
        self.prevent_mouse_event_for_pointer_type_
            .borrow_mut()
            .fill(false);
        self.touch.Clear();
        self.mouse.Clear();
        self.non_hovering_pointers_canceled_.set(false);
        self.factory.borrow_mut().Clear();
        self.touch_ids_for_canceled_pointerdowns_
            .borrow_mut()
            .clear();
        self.element_under_pointer_.borrow_mut().clear();
        self.original_element_under_pointer_removed_
            .borrow_mut()
            .clear();
        self.pointer_capture_target_.borrow_mut().clear();
        self.pending_pointer_capture_target_.borrow_mut().clear();
        self.resize_scrollable_area_.borrow_mut().take();
        self.resize_position_to_size_transform_.borrow_mut().take();
        self.skip_touch_filter_discrete_.set(false);
        self.skip_touch_filter_all_.set(false);
        self.discarded_event_.set((None, TimeTicks::new()));
    }

    // Source: pointer_event_manager.cc:116-126
    // Rc/Drop retain and trace the original live owners in place of Oilpan.
}

// Source: pointer_event_manager.h:161-180; pointer_event_manager.cc:128-149
struct PointerEventBoundaryEventDispatcher<'a> {
    pointer_event_manager_: &'a PointerEventManager<'a>,
    pointer_event_: &'a PointerEventRef,
}
impl BoundaryEventDispatch for PointerEventBoundaryEventDispatcher<'_> {
    fn Dispatch(
        &self,
        target: &EventTargetRef,
        related_target: Option<&EventTargetRef>,
        event_type: &str,
        check_for_listener: bool,
    ) {
        let event = self
            .pointer_event_manager_
            .factory
            .borrow()
            .CreatePointerBoundaryEvent(
                &self.pointer_event_.borrow(),
                event_type,
                related_target.cloned(),
            );
        self.pointer_event_manager_.DispatchPointerEvent(
            Some(target.clone()),
            &event,
            check_for_listener,
        );
    }
}

impl PointerEventManager<'_> {
    // Source: pointer_event_manager.cc:151-226
    pub fn DispatchPointerEvent(
        &self,
        target: Option<EventTargetRef>,
        pointer_event: &PointerEventRef,
        check_for_listener: bool,
    ) -> WebInputEventResult {
        let Some(target) = target else {
            return WebInputEventResult::kNotHandled;
        };
        let event_type = pointer_event.borrow().r#type().to_owned();
        let should_filter = self.ShouldFilterEvent(pointer_event);
        pointer_event.borrow_mut().SetTrusted(true);
        // TODO: .cc:162 UIEventTiming instrumentation.
        if matches!(
            event_type.as_str(),
            "pointerdown" | "pointerover" | "pointerout"
        ) {
            self.dom
                .AnchorElementInteractionTrackerOnPointerEvent(&target, pointer_event);
        }
        if target.Kind() == EventTargetKind::Node {
            if self.features.light_dismiss_from_click {
                let pointer = pointer_event.borrow();
                let id = pointer.pointerId();
                let pointer_target = Rc::new(RefCell::new(PointerTarget::new(
                    Some(&target),
                    pointer.clientX(),
                    pointer.clientY(),
                )));
                drop(pointer);
                if event_type == "pointerdown" {
                    self.factory
                        .borrow_mut()
                        .SetPointerDownTarget(id, Some(pointer_target));
                } else if event_type == "pointerup" {
                    self.factory
                        .borrow_mut()
                        .SetPointerUpTarget(id, Some(pointer_target));
                }
            } else if matches!(event_type.as_str(), "pointerdown" | "pointerup") {
                self.dom.HandlePopoverLightDismiss(pointer_event, &target);
                self.dom.HandleDialogLightDismiss(pointer_event, &target);
            }
            let from_touch = pointer_event.borrow().FromTouch();
            if self.dom.IsElement(&target)
                && !from_touch
                && matches!(event_type.as_str(), "pointerover" | "pointerout")
            {
                self.dom
                    .HandlePointerEventsForInterestFor(&target, &event_type);
            }
        }
        if should_filter && !HasPointerEventListener(self.dom.as_ref()) {
            return WebInputEventResult::kNotHandled;
        }
        if event_type == "pointerdown" && self.dom.NeedsUnbufferedInputEvents(&target) {
            self.dom.RequestUnbufferedInputEvents();
        }
        let listeners_exist =
            !check_for_listener || self.dom.HasEventListeners(&target, &event_type);
        if !should_filter || listeners_exist {
            return ToWebInputEventResult(self.dom.DispatchEvent(
                &target,
                pointer_event,
                EventRef::FromShared(pointer_event.clone()),
            ));
        }
        WebInputEventResult::kNotHandled
    }

    // Source: pointer_event_manager.cc:228-236
    pub fn GetEffectiveTargetForPointerEvent(
        &self,
        target: Option<EventTargetRef>,
        pointer_id: PointerId,
    ) -> Option<EventTargetRef> {
        self.pointer_capture_target_
            .borrow()
            .get(&pointer_id)
            .cloned()
            .or(target)
    }

    // Source: pointer_event_manager.cc:238-263
    pub fn SendMouseAndPointerBoundaryEvents(
        &self,
        entered_element: Option<EventTargetRef>,
        mouse_event: &WebMouseEvent,
    ) {
        let web_pointer_event = WebPointerEvent::FromMouse(Type::kPointerMove, mouse_event);
        let dummy_pointer_event = self
            .factory
            .borrow_mut()
            .Create(&web_pointer_event, &[], &[], self.dom.View())
            .expect("mouse pointer event");
        let reset = {
            let event = dummy_pointer_event.borrow();
            event.buttons() == 0 && event.isPrimary()
        };
        if reset {
            self.prevent_mouse_event_for_pointer_type_.borrow_mut()
                [ToPointerTypeIndex(mouse_event.pointer_properties.pointer_type)] = false;
        }
        self.ProcessCaptureAndPositionOfPointerEvent(
            &dummy_pointer_event,
            entered_element,
            Some(mouse_event),
        );
    }

    // Source: pointer_event_manager.cc:265-274
    fn SendBoundaryEvents(
        &self,
        exited_target: Option<EventTargetRef>,
        original_exited_target_removed: bool,
        entered_target: Option<EventTargetRef>,
        pointer_event: &PointerEventRef,
    ) {
        let sender = PointerEventBoundaryEventDispatcher {
            pointer_event_manager_: self,
            pointer_event_: pointer_event,
        };
        BoundaryEventDispatcher::new("pointerover", "pointerout", "pointerenter", "pointerleave")
            .SendBoundaryEvents(
                self.dom.as_ref(),
                &sender,
                exited_target.as_ref(),
                original_exited_target_removed,
                entered_target.as_ref(),
            );
    }

    // Source: pointer_event_manager.cc:276-305
    fn SetElementUnderPointer(
        &self,
        pointer_event: &PointerEventRef,
        target: Option<EventTargetRef>,
    ) {
        let id = pointer_event.borrow().pointerId();
        let removed = self
            .original_element_under_pointer_removed_
            .borrow()
            .contains(&id);
        assert!(!removed || self.features.boundary_event_dispatch_tracks_node_removal);
        let exited = self.element_under_pointer_.borrow().get(&id).cloned();
        if let Some(target) = &target {
            if !same_target(exited.as_ref(), Some(target)) {
                self.element_under_pointer_
                    .borrow_mut()
                    .insert(id, target.clone());
            }
        } else if exited.is_some() {
            self.element_under_pointer_.borrow_mut().remove(&id);
        }
        self.original_element_under_pointer_removed_
            .borrow_mut()
            .remove(&id);
        self.SendBoundaryEvents(exited, removed, target, pointer_event);
    }

    // Source: pointer_event_manager.cc:307-313
    pub fn NodeChildrenWillBeRemoved(&self, container: &EventTargetRef) {
        self.HandleRemoveSubtree(container, false);
    }
    pub fn NodeWillBeRemoved(&self, node: &EventTargetRef) {
        self.HandleRemoveSubtree(node, true);
    }

    // Source: pointer_event_manager.cc:315-349
    pub fn HandlePseudoElementRemoval(&self, pseudo: &EventTargetRef) {
        let parent = self.dom.ParentOrShadowHostElement(pseudo);
        self.element_under_pointer_
            .borrow_mut()
            .retain(|id, target| {
                if target.IsPseudoElement()
                    && self
                        .dom
                        .IsShadowIncludingInclusiveAncestorOf(pseudo, target)
                {
                    if self.features.boundary_event_dispatch_tracks_node_removal {
                        self.original_element_under_pointer_removed_
                            .borrow_mut()
                            .insert(*id);
                    }
                    if let Some(parent) = &parent {
                        *target = parent.clone();
                    } else {
                        return false;
                    }
                }
                true
            });
        for map in [
            &self.pointer_capture_target_,
            &self.pending_pointer_capture_target_,
        ] {
            map.borrow_mut().retain(|_, target| {
                if target.IsPseudoElement()
                    && self
                        .dom
                        .IsShadowIncludingInclusiveAncestorOf(pseudo, target)
                {
                    if let Some(parent) = &parent {
                        *target = parent.clone();
                    } else {
                        return false;
                    }
                }
                true
            });
        }
        self.factory
            .borrow_mut()
            .HandlePseudoElementRemoved(&PointerPseudoProjection {
                dom: self.dom.as_ref(),
                pseudo,
            });
        self.touch.HandlePseudoElementRemoval(pseudo);
    }

    // Source: pointer_event_manager.cc:351-369
    fn HandleRemoveSubtree(&self, node: &EventTargetRef, include_root: bool) {
        if !self.features.boundary_event_dispatch_tracks_node_removal {
            return;
        }
        let remaining = if include_root {
            self.dom.ParentNode(node)
        } else {
            Some(node.clone())
        };
        let remaining_element = remaining.and_then(|remaining| {
            if self.dom.IsElement(&remaining) {
                Some(remaining)
            } else {
                self.dom.ParentOrShadowHostElement(&remaining)
            }
        });
        self.element_under_pointer_
            .borrow_mut()
            .retain(|id, element| {
                if self.dom.IsShadowIncludingInclusiveAncestorOf(node, element) {
                    self.original_element_under_pointer_removed_
                        .borrow_mut()
                        .insert(*id);
                    if let Some(remaining) = &remaining_element {
                        *element = remaining.clone();
                    } else {
                        return false;
                    }
                }
                true
            });
    }

    // Source: pointer_event_manager.cc:371-439
    pub fn HandlePointerInterruption(&self, event: &WebPointerEvent) {
        debug_assert_eq!(event.GetType(), Type::kPointerCausedUaAction);
        let mut canceled = Vec::new();
        if event.pointer_properties.pointer_type == PointerType::kMouse {
            canceled.push(self.factory.borrow_mut().CreatePointerCancelEvent(
                PointerEventFactory::kMouseId,
                event.TimeStamp(),
                event.pointer_properties.device_id,
            ));
        } else if !self.non_hovering_pointers_canceled_.get() {
            let ids = self.factory.borrow().GetPointerIdsOfNonHoveringPointers();
            for id in ids {
                canceled.push(self.factory.borrow_mut().CreatePointerCancelEvent(
                    id,
                    event.TimeStamp(),
                    event.pointer_properties.device_id,
                ));
            }
            self.non_hovering_pointers_canceled_.set(true);
        }
        for pointer_event in canceled {
            let id = pointer_event.borrow().pointerId();
            let target = self.element_under_pointer_.borrow().get(&id).cloned();
            self.DispatchPointerEvent(
                self.GetEffectiveTargetForPointerEvent(target, id),
                &pointer_event,
                false,
            );
            self.ReleasePointerCaptureById(id);
            if self.features.suppress_pointer_stream_after_drag
                || event.pointer_properties.pointer_type != PointerType::kMouse
            {
                self.ProcessCaptureAndPositionOfPointerEvent(&pointer_event, None, None);
            } else {
                self.ProcessPendingPointerCapture(&pointer_event);
            }
            self.RemovePointer(&pointer_event);
        }
    }

    // Source: pointer_event_manager.cc:441-448
    fn ShouldAdjustPointerEvent(&self, event: &WebPointerEvent) -> bool {
        (event.pointer_properties.pointer_type == PointerType::kTouch
            || self.ShouldAdjustStylusPointerEvent(event))
            && event.GetType() == Type::kPointerDown
            && self.factory.borrow().IsPrimary(&event.pointer_properties)
    }

    // Source: pointer_event_manager.cc:450-456
    fn ShouldAdjustStylusPointerEvent(&self, event: &WebPointerEvent) -> bool {
        matches!(
            event.pointer_properties.pointer_type,
            PointerType::kPen | PointerType::kEraser
        )
    }

    // Source: pointer_event_manager.cc:458-465
    pub fn SetHandwritingRadius(&self, radius: i32) {
        if self.handwriting_radius_.get() != radius {
            self.handwriting_radius_.set(radius);
        }
    }

    // Source: pointer_event_manager.cc:467-475
    fn AdjustPointerEvent(&self, event: &mut WebPointerEvent) {
        debug_assert!(matches!(
            event.pointer_properties.pointer_type,
            PointerType::kTouch | PointerType::kPen | PointerType::kEraser
        ));
        self.AdjustPointerEventWithNode(event);
    }

    // Source: pointer_event_manager.cc:477-544
    fn AdjustPointerEventWithNode(&self, event: &mut WebPointerEvent) -> Option<EventTargetRef> {
        let (width, height) = if event.pointer_properties.pointer_type == PointerType::kTouch {
            (event.width, event.height)
        } else {
            let scale = self.environment.DeviceScaleFactor();
            self.SetHandwritingRadius(self.environment.HandwritingRadius());
            debug_assert!(self.ShouldAdjustStylusPointerEvent(event));
            let size = self.handwriting_radius_.get() as f32
                * (scale / self.environment.PageScaleFactor());
            (size, size)
        };
        let size = self.environment.GetHitTestRectForAdjustment(width, height);
        if size.0 <= 0.0 || size.1 <= 0.0 {
            return None;
        }
        let hit = self
            .environment
            .HitTestAdjustmentRect(*event.pointer_properties.PositionInWidget(), size);
        let mut adjusted_node = None;
        if event.pointer_properties.pointer_type == PointerType::kTouch {
            if let Some(adjustment) = self
                .environment
                .BestNodeForHitTestResult(PointerAdjustmentCandidateType::kClickable, hit.as_ref())
            {
                event
                    .pointer_properties
                    .SetPositionInWidget(adjustment.point);
                adjusted_node = adjustment.node;
            }
            self.environment.CacheTouchAdjustmentResult(
                event.unique_touch_event_id,
                *event.pointer_properties.PositionInWidget(),
            );
        } else if self.ShouldAdjustStylusPointerEvent(event) {
            if let Some(adjustment) = self.environment.BestNodeForHitTestResult(
                PointerAdjustmentCandidateType::kStylusWritable,
                hit.as_ref(),
            ) {
                event
                    .pointer_properties
                    .SetPositionInWidget(adjustment.point);
                adjusted_node = adjustment.node;
            }
        }
        adjusted_node
    }

    // Source: pointer_event_manager.cc:546-565
    fn ShouldFilterEvent(&self, pointer_event: &PointerEventRef) -> bool {
        if !self.skip_touch_filter_discrete_.get() {
            return true;
        }
        let pointer = pointer_event.borrow();
        if matches!(pointer.r#type(), "pointerdown" | "pointerup") {
            return false;
        }
        !(self.skip_touch_filter_all_.get() && pointer.r#type() == "pointermove")
    }

    // Source: pointer_event_manager.cc:567-604
    pub fn ComputePointerEventTarget(&self, event: &WebPointerEvent) -> PointerEventTarget {
        let id = self
            .factory
            .borrow()
            .GetPointerEventId(&event.pointer_properties);
        let pending = self
            .pending_pointer_capture_target_
            .borrow()
            .get(&id)
            .cloned();
        if event.GetType() == Type::kPointerDown || pending.is_none() {
            self.dom.HitTestTouchPointer(event)
        } else {
            let frame = pending
                .as_ref()
                .and_then(|target| self.dom.FrameForElement(target));
            PointerEventTarget {
                target_element: pending,
                target_frame: frame,
                scrollbar: None,
            }
        }
    }

    // Source: pointer_event_manager.cc:606-648
    pub fn DispatchTouchPointerEvent(
        &self,
        event: &WebPointerEvent,
        coalesced: &[WebPointerEvent],
        predicted: &[WebPointerEvent],
        target: &PointerEventTarget,
    ) -> WebInputEventResult {
        debug_assert_ne!(event.GetType(), Type::kPointerCausedUaAction);
        let mut result = WebInputEventResult::kHandledSystem;
        if let (Some(element), Some(frame)) = (&target.target_element, &target.target_frame) {
            if !self.non_hovering_pointers_canceled_.get() {
                self.SetLastPointerPositionForFrameBoundary(event, Some(element.clone()));
                let pointer = self.factory.borrow_mut().Create(
                    event,
                    coalesced,
                    predicted,
                    self.dom.ViewForElement(element),
                );
                result = if let Some(pointer) = &pointer {
                    self.SendTouchPointerEvent(Some(element.clone()), pointer, event.hovering)
                } else {
                    WebInputEventResult::kNotHandled
                };
                if let Some(pointer) = pointer {
                    let canceled_down = {
                        let pointer = pointer.borrow();
                        result != WebInputEventResult::kNotHandled
                            && pointer.r#type() == "pointerdown"
                            && pointer.isPrimary()
                    };
                    if canceled_down {
                        self.dom.AppendTouchIdForCanceledPointerDown(
                            frame,
                            event.unique_touch_event_id,
                        );
                    }
                }
            }
        }
        result
    }

    // Source: pointer_event_manager.cc:650-691
    pub fn SendTouchPointerEvent(
        &self,
        target: Option<EventTargetRef>,
        pointer_event: &PointerEventRef,
        hovering: bool,
    ) -> WebInputEventResult {
        if self.non_hovering_pointers_canceled_.get() {
            return WebInputEventResult::kNotHandled;
        }
        let target = self.ProcessCaptureAndPositionOfPointerEvent(pointer_event, target, None);
        let id = pointer_event.borrow().pointerId();
        let is_down = pointer_event.borrow().r#type() == "pointerdown";
        if is_down {
            if let Some(target) = &target {
                self.SetPointerCapture(id, target.clone());
            }
        }
        let result = self.DispatchPointerEvent(target, pointer_event, false);
        let ends = matches!(
            pointer_event.borrow().r#type(),
            "pointerup" | "pointercancel"
        );
        if ends {
            self.ReleasePointerCaptureById(id);
            if !hovering {
                self.ProcessCaptureAndPositionOfPointerEvent(pointer_event, None, None);
                self.RemovePointer(pointer_event);
            }
        }
        result
    }

    // Source: pointer_event_manager.cc:693-696
    pub fn FlushEvents(&self) -> WebInputEventResult {
        self.touch.FlushEvents()
    }

    // Source: pointer_event_manager.cc:698-874
    pub fn HandlePointerEvent(
        &self,
        event: &WebPointerEvent,
        coalesced: &[WebPointerEvent],
        predicted: &[WebPointerEvent],
    ) -> WebInputEventResult {
        if event.GetType() == Type::kPointerRawUpdate {
            if !self.dom.HasPointerRawUpdateEventHandlers() {
                return WebInputEventResult::kHandledSystem;
            }
            let secure = self.dom.IsSecureContext();
            if !secure {
                // TODO: .cc:712-713 insecure-context UseCounter.
                if self.features.pointer_raw_update_only_in_secure_context {
                    return WebInputEventResult::kHandledSystem;
                }
            }
            let locked = self.environment.PointerLockedElement();
            let target = if let Some(locked) =
                locked.filter(|_| event.pointer_properties.pointer_type == PointerType::kMouse)
            {
                let Some(frame) = self.dom.FrameForElement(&locked) else {
                    return WebInputEventResult::kHandledSystem;
                };
                if !Rc::ptr_eq(&frame, &self.environment.CurrentFrame()) {
                    self.environment
                        .HandlePointerEventOnFrame(&frame, event, coalesced, predicted);
                    return WebInputEventResult::kHandledSystem;
                }
                Some(locked)
            } else {
                self.ComputePointerEventTarget(event).target_element
            };
            let pointer =
                self.factory
                    .borrow_mut()
                    .Create(event, coalesced, predicted, self.dom.View());
            // Create must precede relative-motion filtering: it updates position.
            if event.GetModifiers() & WebInputEvent::kRelativeMotionEvent != 0 {
                return WebInputEventResult::kHandledSuppressed;
            }
            if let Some(pointer) = pointer {
                self.DispatchPointerEvent(target, &pointer, false);
            }
            return WebInputEventResult::kHandledSystem;
        }
        if event.GetType() == Type::kPointerCausedUaAction {
            self.HandlePointerInterruption(event);
            return WebInputEventResult::kHandledSystem;
        }
        let mut pointer = event.WebPointerEventInRootFrame();
        if self.ShouldAdjustPointerEvent(event) {
            self.AdjustPointerEvent(&mut pointer);
        }
        let target = self.ComputePointerEventTarget(&pointer);
        let is_down = event.GetType() == Type::kPointerDown;
        // TODO: .cc:783-790 recently moved iframe mistaken-discard UseCounter.
        // Its retained target/time state is still updated below in source order.
        let discard = target.target_frame.as_ref().is_some_and(|frame| {
            self.environment
                .ShouldDiscardEventTargetingFrame(event, frame)
        });
        if discard {
            if is_down {
                let id = target
                    .target_element
                    .as_ref()
                    .map(|target| self.environment.DomNodeId(target));
                self.discarded_event_.set((id, pointer.TimeStamp()));
            }
            let core = self.factory.borrow_mut().Create(
                event,
                coalesced,
                predicted,
                target
                    .target_element
                    .as_ref()
                    .and_then(|target| self.dom.ViewForElement(target)),
            );
            if let Some(core) = core {
                let (id, device) = {
                    let core = core.borrow();
                    (core.pointerId(), core.persistentDeviceId())
                };
                let cancel = self.factory.borrow_mut().CreatePointerCancelEvent(
                    id,
                    event.TimeStamp(),
                    device,
                );
                self.SendTouchPointerEvent(target.target_element.clone(), &cancel, event.hovering);
            }
            let mut cancel = WebPointerEvent::default();
            cancel.pointer_properties.pointer_type = event.pointer_properties.pointer_type;
            cancel.SetTimeStamp(event.TimeStamp());
            cancel.SetType(Type::kPointerCancel);
            self.touch.HandleTouchPoint(&cancel, coalesced, &target);
            return WebInputEventResult::kHandledSuppressed;
        }
        if is_down {
            self.discarded_event_.set((None, TimeTicks::new()));
        }
        if self.HandleScrollbarTouchDrag(event, target.scrollbar.as_ref()) {
            return WebInputEventResult::kHandledSuppressed;
        }
        if self.HandleResizerDrag(&pointer, &target) {
            return WebInputEventResult::kHandledSuppressed;
        }
        if event.GetType() == Type::kPointerUp && !self.non_hovering_pointers_canceled_.get() {
            if let Some(frame) = &target.target_frame {
                self.environment.NotifyUserActivation(frame);
            }
        }
        if !event.hovering && !self.IsAnyTouchActive() {
            self.non_hovering_pointers_canceled_.set(false);
        }
        let pointerdown_node = if is_down {
            self.touch.GetTouchPointerNode(event, &target)
        } else {
            None
        };
        if let Some(node) = pointerdown_node {
            let action = self
                .environment
                .EffectiveTouchActionAtPointerDown(event, &node);
            self.touch
                .UpdateTouchAttributeMapsForPointerDown(event, &node, action);
        }
        let result = self.DispatchTouchPointerEvent(event, coalesced, predicted, &target);
        self.touch.HandleTouchPoint(event, coalesced, &target);
        result
    }

    // Source: pointer_event_manager.cc:876-899
    fn HandleScrollbarTouchDrag(
        &self,
        event: &WebPointerEvent,
        scrollbar: Option<&Rc<dyn PointerScrollbar>>,
    ) -> bool {
        let Some(scrollbar) = scrollbar else {
            return false;
        };
        if !matches!(
            event.pointer_properties.pointer_type,
            PointerType::kTouch | PointerType::kPen | PointerType::kEraser
        ) {
            return false;
        }
        if event.GetType() == Type::kPointerDown {
            *self.captured_scrollbar_.borrow_mut() = Some(Rc::downgrade(scrollbar));
            self.environment.SetTouchAction(PointerTouchAction::kNone);
        }
        let captured = self
            .captured_scrollbar_
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade);
        let Some(captured) = captured else {
            return false;
        };
        let handled = captured.HandlePointerEvent(event);
        if event.GetType() == Type::kPointerUp {
            self.captured_scrollbar_.borrow_mut().take();
        }
        handled
    }

    // Source: pointer_event_manager.cc:901-954
    fn HandleResizerDrag(&self, event: &WebPointerEvent, target: &PointerEventTarget) -> bool {
        match event.GetType() {
            Type::kPointerDown => {
                let Some(node) = &target.target_element else {
                    return false;
                };
                let Some(area) = self.environment.EnclosingLayerScrollableArea(node) else {
                    return false;
                };
                let frame = target.target_frame.as_ref().expect("layout resizer frame");
                let point = event.pointer_properties.PositionInWidget();
                let point = self
                    .environment
                    .ConvertFromRootFrame(frame, PointF::new(point.x().floor(), point.y().floor()));
                if area.IsAbsolutePointInResizeControl(point) {
                    *self.resize_scrollable_area_.borrow_mut() = Some(Rc::downgrade(&area));
                    area.SetInResizeMode(true);
                    self.environment.SetTouchAction(PointerTouchAction::kNone);
                    let transform = area.InitializeResizeTransform(point);
                    *self.resize_position_to_size_transform_.borrow_mut() = Some(transform);
                    return true;
                }
            }
            Type::kPointerMove => {
                let area = self
                    .resize_scrollable_area_
                    .borrow()
                    .as_ref()
                    .and_then(Weak::upgrade);
                if let Some(area) =
                    area.filter(|area| area.HasLayerAndLayoutBox() && area.InResizeMode())
                {
                    let point = event.pointer_properties.PositionInWidget();
                    let point = PointF::new(point.x().round(), point.y().round());
                    let transform = self
                        .resize_position_to_size_transform_
                        .borrow()
                        .clone()
                        .expect("active resize transform");
                    area.Resize(point, transform.as_ref());
                    return true;
                }
            }
            Type::kPointerUp => {
                let area = self
                    .resize_scrollable_area_
                    .borrow()
                    .as_ref()
                    .and_then(Weak::upgrade);
                if let Some(area) = area.filter(|area| area.InResizeMode()) {
                    area.SetInResizeMode(false);
                    self.resize_scrollable_area_.borrow_mut().take();
                    self.resize_position_to_size_transform_.borrow_mut().take();
                    return true;
                }
            }
            _ => return false,
        }
        false
    }

    // Source: pointer_event_manager.cc:956-990
    fn CreateAndDispatchPointerEvent(
        &self,
        target: EventTargetRef,
        mouse_event_name: &str,
        event: &WebMouseEvent,
        coalesced: &[WebMouseEvent],
        predicted: &[WebMouseEvent],
    ) -> WebInputEventResult {
        let event_type = match mouse_event_name {
            "mousemove" => Type::kPointerMove,
            "mousedown" => Type::kPointerDown,
            "mouseup" => Type::kPointerUp,
            _ => return WebInputEventResult::kNotHandled,
        };
        let pointer = WebPointerEvent::FromMouse(event_type, event);
        let coalesced: Vec<_> = coalesced
            .iter()
            .map(|e| WebPointerEvent::FromMouse(event_type, e))
            .collect();
        let predicted: Vec<_> = predicted
            .iter()
            .map(|e| WebPointerEvent::FromMouse(event_type, e))
            .collect();
        let pointer = self
            .factory
            .borrow_mut()
            .Create(
                &pointer,
                &coalesced,
                &predicted,
                self.dom.ViewForElement(&target),
            )
            .expect("mouse pointer event");
        self.ProcessCaptureAndPositionOfPointerEvent(&pointer, Some(target.clone()), Some(event));
        self.DispatchPointerEvent(Some(target), &pointer, false)
    }

    // Source: pointer_event_manager.cc:992-1023
    pub fn DirectDispatchMousePointerEvent(
        &self,
        target: EventTargetRef,
        event: &WebMouseEvent,
        event_type: &str,
        coalesced: &[WebMouseEvent],
        predicted: &[WebMouseEvent],
    ) -> WebInputEventResult {
        if event.GetModifiers() & WebInputEvent::kRelativeMotionEvent == 0 {
            let last = self.factory.borrow().GetLastPointerPosition(
                PointerEventFactory::kMouseId,
                &event.pointer_properties,
                event.GetType(),
            );
            let result = self.CreateAndDispatchPointerEvent(
                target.clone(),
                event_type,
                event,
                coalesced,
                predicted,
            );
            return MergeEventResult(
                result,
                self.mouse
                    .DispatchMouseEvent(Some(target), event_type, event, &last)
                    .1,
            );
        }
        let pointer_id = self
            .factory
            .borrow()
            .GetPointerEventId(&event.pointer_properties);
        self.factory.borrow_mut().SetLastPosition(
            pointer_id,
            *event.pointer_properties.PositionInScreen(),
            event.GetType(),
        );
        WebInputEventResult::kHandledSuppressed
    }

    // Source: pointer_event_manager.cc:1025-1068
    pub fn SendEffectivePanActionAtPointer(
        &self,
        event: &WebPointerEvent,
        node_at_pointer: Option<EventTargetRef>,
    ) {
        if self.IsAnyTouchActive() {
            return;
        }
        let mut node = node_at_pointer;
        if self.ShouldAdjustStylusPointerEvent(event) {
            let mut cloned = event.clone();
            if let Some(adjusted) = self.AdjustPointerEventWithNode(&mut cloned) {
                node = Some(adjusted);
            }
        }
        let action = if let Some(node) = node {
            self.environment.EffectiveTouchActionAtPointer(event, &node)
        } else {
            PointerTouchAction::kAuto
        };
        let pan = if action.0 & PointerTouchAction::kPan.0 == PointerTouchAction::kNone.0 {
            PointerPanAction::kNone
        } else if action.0 & PointerTouchAction::kInternalNotWritable.0
            != PointerTouchAction::kInternalNotWritable.0
        {
            PointerPanAction::kStylusWritable
        } else if action.0 & PointerTouchAction::kInternalPanXScrolls.0
            != PointerTouchAction::kInternalPanXScrolls.0
        {
            PointerPanAction::kMoveCursorOrScroll
        } else {
            PointerPanAction::kScroll
        };
        self.environment.SetPanAction(pan);
    }
}

// Source: pointer_event_manager.cc:1072-1099
fn NonDeletedElementTarget(
    dom: &dyn PointerEventDOMPort,
    target: Option<EventTargetRef>,
    pointer: &PointerEventRef,
    mouse: Option<&EventRef>,
) -> Option<EventTargetRef> {
    let pointer_event = EventRef::FromShared(pointer.clone());
    let pointer_has_path = pointer_event.borrow().BaseEvent().HasEventPath();
    let dispatched = if pointer_has_path {
        Some(&pointer_event)
    } else {
        mouse.filter(|event| event.borrow().BaseEvent().HasEventPath())
    };
    if !target
        .as_ref()
        .is_some_and(|target| dom.IsInDocument(target))
    {
        if let Some(event) = dispatched {
            let contexts = dom.NodeEventContexts(event);
            for index in 0..contexts.Len() {
                let node = contexts.GetNode(index);
                if dom.IsElement(&node) && dom.IsInDocument(&node) {
                    return Some(node);
                }
            }
        }
    }
    target
}

impl PointerEventManager<'_> {
    // Source: pointer_event_manager.cc:1103-1325
    pub fn SendMousePointerEvent(
        &self,
        mut target: Option<EventTargetRef>,
        event_type: Type,
        mouse_event: &WebMouseEvent,
        coalesced: &[WebMouseEvent],
        predicted: &[WebMouseEvent],
        skip_click_dispatch: bool,
    ) -> WebInputEventResult {
        debug_assert!(matches!(
            event_type,
            Type::kPointerDown | Type::kPointerMove | Type::kPointerUp
        ));
        let web_pointer = WebPointerEvent::FromMouse(event_type, mouse_event);
        let coalesced: Vec<_> = coalesced
            .iter()
            .map(|e| WebPointerEvent::FromMouse(event_type, e))
            .collect();
        let predicted: Vec<_> = predicted
            .iter()
            .map(|e| WebPointerEvent::FromMouse(event_type, e))
            .collect();
        let last = self.factory.borrow().GetLastPointerPosition(
            self.factory
                .borrow()
                .GetPointerEventId(&mouse_event.pointer_properties),
            &mouse_event.pointer_properties,
            event_type,
        );
        let fake = web_pointer.GetModifiers() & WebInputEvent::kRelativeMotionEvent != 0;
        debug_assert!(!fake || event_type == Type::kPointerMove);
        let pointer = self
            .factory
            .borrow_mut()
            .Create(&web_pointer, &coalesced, &predicted, self.dom.View())
            .expect("mouse pointer event");
        let id = pointer.borrow().pointerId();
        let pointer_type_index = ToPointerTypeIndex(mouse_event.pointer_properties.pointer_type);
        let no_buttons = pointer.borrow().buttons() == 0;
        if !fake && event_type == Type::kPointerMove && no_buttons {
            self.ReleasePointerCaptureById(id);
            self.ProcessPendingPointerCapture(&pointer);
            if pointer.borrow().isPrimary() {
                self.prevent_mouse_event_for_pointer_type_.borrow_mut()[pointer_type_index] = false;
            }
        }
        let effective = self.ProcessCaptureAndPositionOfPointerEvent(
            &pointer,
            target.clone(),
            Some(mouse_event),
        );
        if fake {
            return WebInputEventResult::kHandledSuppressed;
        }
        let is_move = pointer.borrow().r#type() == "pointermove";
        if matches!(event_type, Type::kPointerDown | Type::kPointerUp)
            && is_move
            && self.dom.HasPointerRawUpdateEventHandlers()
        {
            let secure = self.dom.IsSecureContext();
            // TODO: .cc:1177-1179 insecure-context UseCounter.
            if !self.features.pointer_raw_update_only_in_secure_context || secure {
                let raw = self
                    .factory
                    .borrow()
                    .CreatePointerRawUpdateEvent(&pointer.borrow());
                self.DispatchPointerEvent(effective.clone(), &raw, false);
            }
        }
        let mut result = self.DispatchPointerEvent(effective.clone(), &pointer, false);
        let (is_primary, pointer_event_type) = {
            let pointer = pointer.borrow();
            (pointer.isPrimary(), pointer.r#type().to_owned())
        };
        if result != WebInputEventResult::kNotHandled
            && pointer_event_type == "pointerdown"
            && is_primary
        {
            self.prevent_mouse_event_for_pointer_type_.borrow_mut()[pointer_type_index] = true;
        }
        let send_compat =
            is_primary && !self.prevent_mouse_event_for_pointer_type_.borrow()[pointer_type_index];
        let mut consider_click =
            !skip_click_dispatch && is_primary && event_type == Type::kPointerUp;
        let mouse_target = if send_compat || consider_click {
            if self.features.boundary_event_dispatch_tracks_node_removal {
                self.mouse.GetElementUnderMouse()
            } else {
                NonDeletedElementTarget(self.dom.as_ref(), effective, &pointer, None)
            }
        } else {
            None
        };
        let mut dispatched_mouse = None;
        if send_compat {
            let dispatch = self.mouse.DispatchMouseEvent(
                mouse_target.clone(),
                MouseEventNameForPointerEventInputType(event_type),
                mouse_event,
                &last,
            );
            dispatched_mouse = dispatch.0;
            result = MergeEventResult(result, dispatch.1);
        }
        if mouse_target.is_none() {
            consider_click = false;
        }
        let captured_click_target = if consider_click {
            self.GetEffectiveTargetForPointerEvent(None, id)
        } else {
            None
        };
        if matches!(pointer_event_type.as_str(), "pointerup" | "pointercancel") {
            self.ReleasePointerCaptureById(id);
            if is_primary {
                self.prevent_mouse_event_for_pointer_type_.borrow_mut()[pointer_type_index] = false;
            }
        }
        if pointer_event_type == "pointerup" {
            let has_capture = self.pointer_capture_target_.borrow().contains_key(&id);
            if has_capture {
                target = self.dom.HitTestMouseRelease(mouse_event);
            } else if self.features.boundary_event_dispatch_tracks_node_removal {
                target = NonDeletedElementTarget(
                    self.dom.as_ref(),
                    target,
                    &pointer,
                    dispatched_mouse.as_ref(),
                );
            }
        }
        if consider_click {
            self.ProcessPendingPointerCapture(&pointer);
            let mut click_target = mouse_target;
            if click_target
                .as_ref()
                .is_some_and(|target| target.IsPseudoElement() && !self.dom.IsInDocument(target))
            {
                click_target = self.mouse.GetElementUnderMouse();
            }
            let pointer_type = pointer.borrow().pointerType().to_owned();
            let down_target = self.factory.borrow().GetPointerDownTarget(id);
            let up_target = self.factory.borrow().GetPointerUpTarget(id);
            self.mouse.DispatchMouseClickIfNeeded(
                click_target,
                captured_click_target,
                mouse_event,
                id,
                &pointer_type,
                down_target,
                up_target,
            );
            self.factory.borrow_mut().RemovePointerTargets(id);
        }
        if pointer_event_type == "pointerup" {
            if consider_click && self.features.boundary_event_dispatch_tracks_node_removal {
                target = NonDeletedElementTarget(
                    self.dom.as_ref(),
                    target,
                    &pointer,
                    dispatched_mouse.as_ref(),
                );
            }
            self.ProcessCaptureAndPositionOfPointerEvent(&pointer, target, Some(mouse_event));
        } else if pointer_event_type == "pointercancel" {
            self.ProcessPendingPointerCapture(&pointer);
        }
        if mouse_event.GetType() == Type::kMouseLeave
            && matches!(
                mouse_event.pointer_properties.pointer_type,
                PointerType::kPen | PointerType::kEraser
            )
        {
            self.factory.borrow_mut().Remove(id);
        }
        result
    }

    // Source: pointer_event_manager.cc:1327-1347
    fn GetPointerCaptureState(
        &self,
        id: PointerId,
    ) -> (bool, Option<EventTargetRef>, Option<EventTargetRef>) {
        let capture = self.pointer_capture_target_.borrow().get(&id).cloned();
        let pending = self
            .pending_pointer_capture_target_
            .borrow()
            .get(&id)
            .cloned();
        (
            !same_target(capture.as_ref(), pending.as_ref()),
            capture,
            pending,
        )
    }

    // Source: pointer_event_manager.cc:1349-1363
    fn ProcessCaptureAndPositionOfPointerEvent(
        &self,
        pointer: &PointerEventRef,
        hit_test_target: Option<EventTargetRef>,
        mouse_event: Option<&WebMouseEvent>,
    ) -> Option<EventTargetRef> {
        self.ProcessPendingPointerCapture(pointer);
        let id = pointer.borrow().pointerId();
        let effective = self.GetEffectiveTargetForPointerEvent(hit_test_target, id);
        self.SetElementUnderPointer(pointer, effective.clone());
        if let Some(mouse_event) = mouse_event {
            self.mouse
                .SetElementUnderMouse(effective.clone(), mouse_event);
        }
        effective
    }

    // Source: pointer_event_manager.cc:1365-1410
    fn ProcessPendingPointerCapture(&self, pointer: &PointerEventRef) {
        let id = pointer.borrow().pointerId();
        let (changed, capture, pending) = self.GetPointerCaptureState(id);
        if !changed {
            return;
        }
        if let Some(capture) = capture {
            let target = if self.dom.IsInDocument(&capture) {
                capture.clone()
            } else {
                self.dom.OwnerDocument(&capture)
            };
            self.pointer_capture_target_.borrow_mut().remove(&id);
            let lost = self
                .factory
                .borrow()
                .CreatePointerCaptureEvent(&pointer.borrow(), "lostpointercapture");
            self.DispatchPointerEvent(Some(target), &lost, false);
        }
        if let Some(pending) = pending.filter(|pending| self.dom.IsInDocument(pending)) {
            self.SetElementUnderPointer(pointer, Some(pending.clone()));
            let got = self
                .factory
                .borrow()
                .CreatePointerCaptureEvent(&pointer.borrow(), "gotpointercapture");
            self.DispatchPointerEvent(Some(pending.clone()), &got, false);
            if self.dom.IsInDocument(&pending) {
                self.pointer_capture_target_
                    .borrow_mut()
                    .insert(id, pending);
            } else {
                let lost = self
                    .factory
                    .borrow()
                    .CreatePointerCaptureEvent(&pointer.borrow(), "lostpointercapture");
                self.DispatchPointerEvent(Some(self.dom.OwnerDocument(&pending)), &lost, false);
            }
        }
    }

    // Source: pointer_event_manager.cc:1412-1419
    fn RemoveTargetFromPointerCapturingMapping(
        map: &RefCell<HashMap<PointerId, EventTargetRef>>,
        target: &EventTargetRef,
    ) {
        map.borrow_mut()
            .retain(|_, value| !Rc::ptr_eq(value, target));
    }

    // Source: pointer_event_manager.cc:1421-1429
    fn RemovePointer(&self, pointer: &PointerEventRef) {
        let id = pointer.borrow().pointerId();
        if self.factory.borrow_mut().Remove(id) {
            self.pending_pointer_capture_target_
                .borrow_mut()
                .remove(&id);
            self.pointer_capture_target_.borrow_mut().remove(&id);
            self.element_under_pointer_.borrow_mut().remove(&id);
            self.original_element_under_pointer_removed_
                .borrow_mut()
                .remove(&id);
        }
    }

    // Source: pointer_event_manager.cc:1431-1434
    pub fn ElementRemoved(&self, target: &EventTargetRef) {
        Self::RemoveTargetFromPointerCapturingMapping(
            &self.pending_pointer_capture_target_,
            target,
        );
    }

    // Source: pointer_event_manager.cc:1436-1443
    pub fn SetPointerCapture(&self, id: PointerId, target: EventTargetRef) -> bool {
        if self.factory.borrow().IsActiveButtonsState(id) {
            self.pending_pointer_capture_target_
                .borrow_mut()
                .insert(id, target);
            true
        } else {
            false
        }
    }

    // Source: pointer_event_manager.cc:1445-1459
    pub fn ReleasePointerCapture(&self, id: PointerId, target: &EventTargetRef) -> bool {
        if self.HasPointerCapture(id, target) {
            self.ReleasePointerCaptureById(id);
            true
        } else {
            false
        }
    }

    // Source: pointer_event_manager.cc:1461-1463
    pub fn ReleaseMousePointerCapture(&self) {
        self.ReleasePointerCaptureById(PointerEventFactory::kMouseId);
    }

    // Source: pointer_event_manager.cc:1465-1469
    pub fn HasPointerCapture(&self, id: PointerId, target: &EventTargetRef) -> bool {
        self.pending_pointer_capture_target_
            .borrow()
            .get(&id)
            .is_some_and(|value| Rc::ptr_eq(value, target))
    }

    // Source: pointer_event_manager.cc:1471-1473
    fn ReleasePointerCaptureById(&self, id: PointerId) {
        self.pending_pointer_capture_target_
            .borrow_mut()
            .remove(&id);
    }

    // Source: pointer_event_manager.cc:1475-1482
    pub fn GetMouseCaptureTarget(&self) -> Option<EventTargetRef> {
        self.pending_pointer_capture_target_
            .borrow()
            .get(&PointerEventFactory::kMouseId)
            .cloned()
    }

    // Source: pointer_event_manager.cc:1484-1487
    pub fn AppendTouchIdForCanceledPointerDown(&self, id: u32) {
        self.factory.AppendTouchIdForCanceledPointerDown(id);
    }

    // Source: pointer_event_manager.cc:1489-1491
    pub fn IsActive(&self, id: PointerId) -> bool {
        self.factory.borrow().IsActive(id)
    }

    // Source: pointer_event_manager.cc:1493-1504
    pub fn IsPointerIdActiveOnFrame(&self, id: PointerId, frame: &Rc<()>) -> bool {
        self.element_under_pointer_
            .borrow()
            .get(&id)
            .and_then(|target| self.dom.FrameForElement(target))
            .is_some_and(|target_frame| Rc::ptr_eq(&target_frame, frame))
    }

    // Source: pointer_event_manager.cc:1506-1509
    pub fn IsAnyTouchActive(&self) -> bool {
        self.touch.IsAnyTouchActive()
    }

    // Source: pointer_event_manager.cc:1511-1525
    pub fn PrimaryPointerdownCanceled(&self, id: u32) -> bool {
        let mut ids = self.touch_ids_for_canceled_pointerdowns_.borrow_mut();
        while let Some(first) = ids.front().copied() {
            if first > id {
                return false;
            }
            ids.pop_front();
            if first == id {
                return true;
            }
        }
        false
    }

    // Source: pointer_event_manager.cc:1527-1543
    fn SetLastPointerPositionForFrameBoundary(
        &self,
        event: &WebPointerEvent,
        new_target: Option<EventTargetRef>,
    ) {
        let id = self
            .factory
            .borrow()
            .GetPointerEventId(&event.pointer_properties);
        let last = self.element_under_pointer_.borrow().get(&id).cloned();
        if let Some(new_target) = new_target {
            let new_frame = self.dom.FrameForElement(&new_target);
            let last_frame = last
                .as_ref()
                .and_then(|last| self.dom.FrameForElement(last));
            if last.is_none() || !same_frame(new_frame.as_ref(), last_frame.as_ref()) {
                self.factory.borrow_mut().SetLastPosition(
                    id,
                    *event.pointer_properties.PositionInScreen(),
                    event.GetType(),
                );
            }
        } else {
            self.factory.borrow_mut().RemoveLastPosition(id);
        }
    }

    // Source: pointer_event_manager.cc:1545-1547
    pub fn RemoveLastMousePosition(&self) {
        self.factory
            .borrow_mut()
            .RemoveLastPosition(PointerEventFactory::kMouseId);
    }

    // Source: pointer_event_manager.cc:1549-1553
    pub fn GetPointerIdForTouchGesture(&self, id: u32) -> PointerId {
        self.factory.borrow_mut().GetPointerIdForTouchGesture(id)
    }

    // Source: pointer_event_manager.cc:1555-1557
    pub fn CurrentTouchDownElement(&self) -> Option<EventTargetRef> {
        self.touch.CurrentTouchDownElement()
    }

    // Source: pointer_event_manager.cc:1559-1562
    pub fn GetPointerDownTarget(&self, id: PointerId) -> Option<PointerTargetRef> {
        self.factory.borrow().GetPointerDownTarget(id)
    }

    // Source: pointer_event_manager.cc:1564-1567
    pub fn GetPointerUpTarget(&self, id: PointerId) -> Option<PointerTargetRef> {
        self.factory.borrow().GetPointerUpTarget(id)
    }

    // Source: pointer_event_manager.cc:1569-1571
    pub fn RemovePointerTargets(&self, id: PointerId) {
        self.factory.borrow_mut().RemovePointerTargets(id);
    }
}

// Source: pointer_event_manager.cc:342-344
struct PointerPseudoProjection<'a> {
    dom: &'a dyn PointerEventDOMPort,
    pseudo: &'a EventTargetRef,
}
impl PointerPseudoElement for PointerPseudoProjection<'_> {
    fn ParentOrShadowHostElement(&self) -> Option<EventTargetRef> {
        self.dom.ParentOrShadowHostElement(self.pseudo)
    }
    fn IsShadowIncludingInclusiveAncestorOf(&self, target: &EventTargetRef) -> bool {
        self.dom
            .IsShadowIncludingInclusiveAncestorOf(self.pseudo, target)
    }
}

// Source: pointer_event_manager.cc:294,1346 (C++ owner-identity comparisons).
fn same_target(a: Option<&EventTargetRef>, b: Option<&EventTargetRef>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

// Source: pointer_event_manager.cc:1537-1538 (LocalFrame identity comparison).
fn same_frame(a: Option<&Rc<()>>, b: Option<&Rc<()>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}
