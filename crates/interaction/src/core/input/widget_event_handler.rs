// Copyright 2022 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/input/widget_event_handler.h:1-57
// Source: third_party/blink/renderer/core/input/widget_event_handler.cc:1-220
// Rust modules replace includes, guards, export annotations and namespaces.

use interaction_dispatch::public::platform::web_input_event_result::WebInputEventResult;
use interaction_hit_test::core::layout::hit_test_location::HitTestLocation;
use interaction_hit_test::core::layout::hit_test_request::RequestType;
use interaction_hit_test::core::layout::hit_test_result::{FrameRef, HitTestResult};
use interaction_input::base::time::time::TimeTicks;
use interaction_input::public::common::input::web_coalesced_input_event::WebCoalescedInputEvent;
use interaction_input::public::common::input::web_input_event::{
    Type, WebInputEvent, WebInputEventPayload,
};
use interaction_input::public::common::input::web_keyboard_event::WebKeyboardEvent;
use interaction_input::public::common::input::web_mouse_event::WebMouseEvent;
use interaction_input::public::common::input::web_mouse_wheel_event::WebMouseWheelEvent;
use interaction_input::public::common::input::web_pointer_event::WebPointerEvent;

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: widget_event_handler.h:16-23; widget_event_handler.cc:24-54,146-217
// Narrow ports to the existing frame/document/view/event-handler owners. The
// frame lookup resolves the existing HitTestResult FrameRef identity; it must
// not construct another frame or document. Integer coordinate pairs carry
// gfx::Point values without adding a dependency on a geometry owner.
pub trait WidgetLocalFrame {
    fn GetDocument(&self) -> Option<&dyn WidgetDocument>;
    fn View(&self) -> Option<&dyn WidgetFrameView>;
    fn SetCurrentInteractionEventQueuedTimestamp(&self, timestamp: TimeTicks);
    fn GetEventHandler(&mut self) -> &mut dyn WidgetFrameEventHandler;
    fn GetDocumentForFrame(&self, frame: &FrameRef) -> Option<&dyn WidgetDocument>;
}

pub trait WidgetDocument {
    fn View(&self) -> Option<&dyn WidgetDocumentView>;
    fn ExistingAXObjectCache(&self) -> Option<&dyn WidgetAXObjectCache>;
}

pub trait WidgetDocumentView {
    fn NotifyInput(&self, event: &WebInputEvent);
}

pub trait WidgetAXObjectCache {
    fn OnTouchAccessibilityHover(&self, rounded_point_in_inner_node_frame: [i32; 2]);
}

pub trait WidgetFrameView {
    fn TransformWebMouseEvent(&self, event: &WebMouseEvent) -> WebMouseEvent;
    fn TransformWebMouseEventVector(
        &self,
        events: &[Box<dyn WebInputEventPayload>],
    ) -> Vec<WebMouseEvent>;
    fn TransformWebMouseWheelEvent(&self, event: &WebMouseWheelEvent) -> WebMouseWheelEvent;
    fn TransformWebPointerEvent(&self, event: &WebPointerEvent) -> WebPointerEvent;
    fn TransformWebPointerEventVector(
        &self,
        events: &[Box<dyn WebInputEventPayload>],
    ) -> Vec<WebPointerEvent>;
    // Converts the already-floored root point, then constructs HitTestLocation
    // from that local gfx::Point, exactly as the source expression does.
    fn ConvertFromRootFrame(&self, floored_point: [i32; 2]) -> HitTestLocation;
}

pub trait WidgetFrameEventHandler {
    // This request has no borrowed stop node/layer; its returned result keeps
    // the existing shared node/frame owners rather than borrowing this port.
    fn HitTestResultAtLocation(
        &mut self,
        location: &HitTestLocation,
        request_type: u32,
    ) -> HitTestResult<'static, ()>;
    fn HandleMouseMoveEvent(
        &mut self,
        event: WebMouseEvent,
        coalesced_events: Vec<WebMouseEvent>,
        predicted_events: Vec<WebMouseEvent>,
    );
    fn HandleMouseLeaveEvent(&mut self, event: WebMouseEvent);
    fn CancelDelayedNavigationTask(&mut self);
    fn HandleMousePressEvent(&mut self, event: WebMouseEvent);
    fn HandleMouseReleaseEvent(&mut self, event: WebMouseEvent) -> WebInputEventResult;
    fn HandleWheelEvent(&mut self, event: WebMouseWheelEvent) -> WebInputEventResult;
    fn HandlePointerEvent(
        &mut self,
        event: WebPointerEvent,
        coalesced_events: Vec<WebPointerEvent>,
        predicted_events: Vec<WebPointerEvent>,
    ) -> WebInputEventResult;
}

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: widget_event_handler.h:25-53
// Frame and popup widgets override the same virtual entry points. Default
// bodies below follow the .cc definition order. GestureEvent retains the
// embedding's concrete WebGestureEvent payload until that input unit lands;
// this source unit only downcasts and forwards it, never projects its fields.
pub trait WidgetEventHandler<F: WidgetLocalFrame> {
    type GestureEvent: WebInputEventPayload;

    // Source: widget_event_handler.h:29-33
    fn HandleMouseMove(
        &mut self,
        main_frame: &mut F,
        event: &WebMouseEvent,
        coalesced_events: &[Box<dyn WebInputEventPayload>],
        predicted_events: &[Box<dyn WebInputEventPayload>],
    ) {
        HandleMouseMove(main_frame, event, coalesced_events, predicted_events);
    }
    // Source: widget_event_handler.h:34
    fn HandleMouseLeave(&mut self, local_root: &mut F, event: &WebMouseEvent) {
        HandleMouseLeave(local_root, event);
    }
    // Source: widget_event_handler.h:35
    fn HandleMouseDown(&mut self, local_root: &mut F, event: &WebMouseEvent) {
        HandleMouseDown(local_root, event);
    }
    // Source: widget_event_handler.h:36-37
    fn HandleMouseUp(&mut self, local_root: &mut F, event: &WebMouseEvent) -> WebInputEventResult {
        HandleMouseUp(local_root, event)
    }
    // Source: widget_event_handler.h:38-39
    fn HandleMouseWheel(
        &mut self,
        local_root: &mut F,
        event: &WebMouseWheelEvent,
    ) -> WebInputEventResult {
        HandleMouseWheel(local_root, event)
    }
    // Source: widget_event_handler.h:40
    fn HandleKeyEvent(&mut self, event: &WebKeyboardEvent) -> WebInputEventResult;
    // Source: widget_event_handler.h:41
    fn HandleCharEvent(&mut self, event: &WebKeyboardEvent) -> WebInputEventResult;
    // Source: widget_event_handler.h:42
    fn HandleGestureEvent(&mut self, event: &Self::GestureEvent) -> WebInputEventResult;
    // Source: widget_event_handler.h:43-47
    fn HandlePointerEvent(
        &mut self,
        main_frame: &mut F,
        event: &WebPointerEvent,
        coalesced_events: &[Box<dyn WebInputEventPayload>],
        predicted_events: &[Box<dyn WebInputEventPayload>],
    ) -> WebInputEventResult {
        HandlePointerEvent(main_frame, event, coalesced_events, predicted_events)
    }
    // Source: widget_event_handler.h:48
    // Rust destroys the concrete widget owner and its fields automatically.

    // Source: widget_event_handler.h:50-52
    fn HandleInputEvent<L: Clone>(
        &mut self,
        coalesced_event: &WebCoalescedInputEvent<L>,
        root: Option<&mut F>,
    ) -> WebInputEventResult {
        HandleInputEvent(self, coalesced_event, root)
    }
}

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: widget_event_handler.cc:20-144
pub fn HandleInputEvent<F: WidgetLocalFrame, H: WidgetEventHandler<F> + ?Sized, L: Clone>(
    handler: &mut H,
    coalesced_event: &WebCoalescedInputEvent<L>,
    mut root: Option<&mut F>,
) -> WebInputEventResult {
    let payload = coalesced_event.Event();
    let event = payload.Event();
    // Source: widget_event_handler.cc:24-33
    if let Some(root) = root.as_deref() {
        let document = root.GetDocument().expect("a local root has a document");
        if let Some(view) = document.View() {
            view.NotifyInput(event);
        }
        root.SetCurrentInteractionEventQueuedTimestamp(event.QueuedTimeStamp());
    }

    // Source: widget_event_handler.cc:35-55
    if event.GetModifiers() & WebInputEvent::kIsTouchAccessibility != 0
        && WebInputEvent::IsMouseEventType(event.GetType())
    {
        // The source requires a root/view here, before the switch's detached
        // frame guards. Preserve that precondition rather than dropping hover.
        let root = root
            .as_deref_mut()
            .expect("touch accessibility requires a root");
        let view = root.View().expect("touch accessibility requires a view");
        let mouse_event = view.TransformWebMouseEvent(
            payload
                .AsAny()
                .downcast_ref::<WebMouseEvent>()
                .expect("mouse payload"),
        );
        let point = mouse_event.PositionInRootFrame();
        let location =
            view.ConvertFromRootFrame([point.x().floor() as i32, point.y().floor() as i32]);
        let mut result = root.GetEventHandler().HitTestResultAtLocation(
            &location,
            RequestType::kReadOnly as u32 | RequestType::kActive as u32,
        );
        result.SetToShadowHostIfInUAShadowRoot();
        if let Some(frame) = result.InnerNodeFrame() {
            if let Some(document) = root.GetDocumentForFrame(&frame) {
                if let Some(cache) = document.ExistingAXObjectCache() {
                    let point = result.RoundedPointInInnerNodeFrame();
                    cache.OnTouchAccessibilityHover([point.x(), point.y()]);
                }
            }
        }
    }

    // Source: widget_event_handler.cc:57-143
    match event.GetType() {
        // Source assumes mouse processing succeeded; detached mouse roots
        // are suppressed, while wheel and pointer roots are not handled.
        Type::kMouseMove => {
            let Some(root) = root.filter(|root| root.View().is_some()) else {
                return WebInputEventResult::kHandledSuppressed;
            };
            handler.HandleMouseMove(
                root,
                payload
                    .AsAny()
                    .downcast_ref::<WebMouseEvent>()
                    .expect("mouse payload"),
                coalesced_event.GetCoalescedEventsPointers(),
                coalesced_event.GetPredictedEventsPointers(),
            );
            WebInputEventResult::kHandledSystem
        }
        Type::kMouseLeave => {
            let Some(root) = root.filter(|root| root.View().is_some()) else {
                return WebInputEventResult::kHandledSuppressed;
            };
            handler.HandleMouseLeave(
                root,
                payload
                    .AsAny()
                    .downcast_ref::<WebMouseEvent>()
                    .expect("mouse payload"),
            );
            WebInputEventResult::kHandledSystem
        }
        Type::kMouseDown => {
            let Some(root) = root.filter(|root| root.View().is_some()) else {
                return WebInputEventResult::kHandledSuppressed;
            };
            handler.HandleMouseDown(
                root,
                payload
                    .AsAny()
                    .downcast_ref::<WebMouseEvent>()
                    .expect("mouse payload"),
            );
            WebInputEventResult::kHandledSystem
        }
        Type::kMouseUp => {
            let Some(root) = root.filter(|root| root.View().is_some()) else {
                return WebInputEventResult::kHandledSuppressed;
            };
            handler.HandleMouseUp(
                root,
                payload
                    .AsAny()
                    .downcast_ref::<WebMouseEvent>()
                    .expect("mouse payload"),
            )
        }
        Type::kMouseWheel => {
            let Some(root) = root.filter(|root| root.View().is_some()) else {
                return WebInputEventResult::kNotHandled;
            };
            handler.HandleMouseWheel(
                root,
                payload
                    .AsAny()
                    .downcast_ref::<WebMouseWheelEvent>()
                    .expect("wheel payload"),
            )
        }
        Type::kRawKeyDown | Type::kKeyDown | Type::kKeyUp => handler.HandleKeyEvent(
            payload
                .AsAny()
                .downcast_ref::<WebKeyboardEvent>()
                .expect("keyboard payload"),
        ),
        Type::kChar => handler.HandleCharEvent(
            payload
                .AsAny()
                .downcast_ref::<WebKeyboardEvent>()
                .expect("keyboard payload"),
        ),
        Type::kGestureScrollBegin
        | Type::kGestureScrollEnd
        | Type::kGestureScrollUpdate
        | Type::kGestureFlingStart
        | Type::kGestureFlingCancel
        | Type::kGestureTap
        | Type::kGestureTapUnconfirmed
        | Type::kGestureTapDown
        | Type::kGestureShowPress
        | Type::kGestureTapCancel
        | Type::kGestureDoubleTap
        | Type::kGestureTwoFingerTap
        | Type::kGestureShortPress
        | Type::kGestureLongPress
        | Type::kGestureLongTap => handler.HandleGestureEvent(
            payload
                .AsAny()
                .downcast_ref::<H::GestureEvent>()
                .expect("gesture payload"),
        ),
        Type::kPointerDown
        | Type::kPointerUp
        | Type::kPointerMove
        | Type::kPointerRawUpdate
        | Type::kPointerCancel
        | Type::kPointerCausedUaAction => {
            let Some(root) = root.filter(|root| root.View().is_some()) else {
                return WebInputEventResult::kNotHandled;
            };
            handler.HandlePointerEvent(
                root,
                payload
                    .AsAny()
                    .downcast_ref::<WebPointerEvent>()
                    .expect("pointer payload"),
                coalesced_event.GetCoalescedEventsPointers(),
                coalesced_event.GetPredictedEventsPointers(),
            )
        }
        Type::kTouchStart
        | Type::kTouchMove
        | Type::kTouchEnd
        | Type::kTouchCancel
        | Type::kTouchScrollStarted => {
            unreachable!("touch events must not reach WidgetEventHandler")
        }
        Type::kGesturePinchBegin => {
            // Source: widget_event_handler.cc:135-137
            // TODO: instrumentation: DLOG pinch ignored by main thread.
            WebInputEventResult::kNotHandled
        }
        Type::kGesturePinchEnd | Type::kGesturePinchUpdate => WebInputEventResult::kNotHandled,
        _ => WebInputEventResult::kNotHandled,
    }
}

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: widget_event_handler.cc:146-157
pub fn HandleMouseMove(
    local_root: &mut impl WidgetLocalFrame,
    event: &WebMouseEvent,
    coalesced_events: &[Box<dyn WebInputEventPayload>],
    predicted_events: &[Box<dyn WebInputEventPayload>],
) {
    let view = local_root.View().expect("mouse move requires a view");
    let transformed_event = view.TransformWebMouseEvent(event);
    let coalesced_events = view.TransformWebMouseEventVector(coalesced_events);
    let predicted_events = view.TransformWebMouseEventVector(predicted_events);
    local_root.GetEventHandler().HandleMouseMoveEvent(
        transformed_event,
        coalesced_events,
        predicted_events,
    );
}

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: widget_event_handler.cc:159-164
pub fn HandleMouseLeave(local_root: &mut impl WidgetLocalFrame, event: &WebMouseEvent) {
    let transformed_event = local_root
        .View()
        .expect("mouse leave requires a view")
        .TransformWebMouseEvent(event);
    local_root
        .GetEventHandler()
        .HandleMouseLeaveEvent(transformed_event);
}

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: widget_event_handler.cc:166-177
pub(crate) fn IsDoubleAltClick(mouse_event: &WebMouseEvent) -> bool {
    let is_alt_pressed = mouse_event.GetModifiers() & WebInputEvent::kAltKey != 0;
    if !is_alt_pressed {
        return false;
    }
    mouse_event.click_count == 2
}

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: widget_event_handler.cc:179-188
pub fn HandleMouseDown(local_root: &mut impl WidgetLocalFrame, event: &WebMouseEvent) {
    if IsDoubleAltClick(event) {
        local_root.GetEventHandler().CancelDelayedNavigationTask();
    }
    let transformed_event = local_root
        .View()
        .expect("mouse down requires a view")
        .TransformWebMouseEvent(event);
    local_root
        .GetEventHandler()
        .HandleMousePressEvent(transformed_event);
}

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: widget_event_handler.cc:190-197
pub fn HandleMouseUp(
    local_root: &mut impl WidgetLocalFrame,
    event: &WebMouseEvent,
) -> WebInputEventResult {
    let transformed_event = local_root
        .View()
        .expect("mouse up requires a view")
        .TransformWebMouseEvent(event);
    local_root
        .GetEventHandler()
        .HandleMouseReleaseEvent(transformed_event)
}

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: widget_event_handler.cc:199-205
pub fn HandleMouseWheel(
    local_root: &mut impl WidgetLocalFrame,
    event: &WebMouseWheelEvent,
) -> WebInputEventResult {
    let transformed_event = local_root
        .View()
        .expect("wheel requires a view")
        .TransformWebMouseWheelEvent(event);
    local_root
        .GetEventHandler()
        .HandleWheelEvent(transformed_event)
}

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: widget_event_handler.cc:207-218
pub fn HandlePointerEvent(
    local_root: &mut impl WidgetLocalFrame,
    event: &WebPointerEvent,
    coalesced_events: &[Box<dyn WebInputEventPayload>],
    predicted_events: &[Box<dyn WebInputEventPayload>],
) -> WebInputEventResult {
    let view = local_root.View().expect("pointer event requires a view");
    let transformed_event = view.TransformWebPointerEvent(event);
    let coalesced_events = view.TransformWebPointerEventVector(coalesced_events);
    let predicted_events = view.TransformWebPointerEventVector(predicted_events);
    local_root.GetEventHandler().HandlePointerEvent(
        transformed_event,
        coalesced_events,
        predicted_events,
    )
}
