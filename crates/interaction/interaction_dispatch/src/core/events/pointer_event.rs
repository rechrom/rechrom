// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/pointer_event.h:1-152
// Source: third_party/blink/renderer/core/events/pointer_event.cc:1-213
// Rust modules, embedded base state and shared owners replace includes,
// namespaces, wrapper annotations, inheritance and GC tracing.

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use interaction_input::base::time::time::TimeTicks;
use interaction_input::public::common::input::pointer_id::PointerId;

use crate::core::dom::events::event::{Event, EventClock, EventInterface, EventTargetRef};
use crate::core::dom::events::event_dispatch_result::DispatchEventResult;
use crate::core::dom::events::event_dispatcher::{EventDispatchHook, EventDispatcher};

use super::mouse_event::{
    MenuSourceType, MouseCoordinateContext, MouseEvent, MouseEventDispatchHook,
    MouseEventInterface, MouseNestedDispatch, SyntheticEventType,
};
pub use super::pointer_event_init::PointerEventInit;
use super::pointer_event_util::PointerEventUtil;
use super::ui_event::{AbstractViewRef, UIEvent, UIEventInterface};
use super::ui_event_with_key_state::{UIEventWithKeyState, UIEventWithKeyStateInterface};

// Source: pointer_event.h:135-137; pointer_event.cc:63-71,155-160
// Each list entry retains the concrete event owner, including its live base
// target and timestamp. Cloning a list clones handles, never event contents.
pub type PointerEventRef = Rc<RefCell<PointerEvent>>;

// Source: pointer_event.cc:207-210
// The view owner supplies only the LocalDOMWindow-to-Document projection.
pub trait PointerEventDocument {}
pub type PointerEventDocumentRef = Rc<dyn PointerEventDocument>;
pub trait PointerEventDocumentQuery {
    fn DocumentForLocalDOMWindow(&self, view: &AbstractViewRef) -> Option<PointerEventDocumentRef>;
}

// Source: pointer_event.h:18-19,122-142
pub struct PointerEvent {
    mouse_event: MouseEvent,
    pointer_id_: PointerId,
    width_: f64,
    height_: f64,
    pressure_: f32,
    tilt_x_: i32,
    tilt_y_: i32,
    azimuth_angle_: f64,
    altitude_angle_: f64,
    tangential_pressure_: f32,
    twist_: i32,
    pointer_type_: String,
    is_primary_: bool,
    coalesced_events_: Vec<PointerEventRef>,
    predicted_events_: Vec<PointerEventRef>,
    persistent_device_id_: i32,
    prevent_counting_as_interaction_: bool,
}

impl PointerEvent {
    // Source: pointer_event.h:22-34
    pub fn Create(
        event_type: impl Into<String>,
        initializer: &PointerEventInit,
        clock: &dyn EventClock,
        synthetic_event_type: SyntheticEventType,
        menu_source_type: MenuSourceType,
        prevent_counting_as_interaction: bool,
        coordinates: MouseCoordinateContext,
    ) -> Self {
        Self::new(
            event_type,
            initializer,
            clock.Now(),
            synthetic_event_type,
            menu_source_type,
            prevent_counting_as_interaction,
            coordinates,
        )
    }

    // Source: pointer_event.h:36-42; pointer_event.cc:19-98
    pub fn new(
        event_type: impl Into<String>,
        initializer: &PointerEventInit,
        platform_time_stamp: TimeTicks,
        synthetic_event_type: SyntheticEventType,
        menu_source_type: MenuSourceType,
        prevent_counting_as_interaction: bool,
        coordinates: MouseCoordinateContext,
    ) -> Self {
        let mut event = Self {
            mouse_event: MouseEvent::new(
                event_type,
                &initializer.mouse_event_init,
                platform_time_stamp,
                synthetic_event_type,
                menu_source_type,
                None,
                coordinates,
            ),
            pointer_id_: 0,
            width_: 0.0,
            height_: 0.0,
            pressure_: 0.0,
            tilt_x_: 0,
            tilt_y_: 0,
            azimuth_angle_: 0.0,
            altitude_angle_: std::f64::consts::FRAC_PI_2,
            tangential_pressure_: 0.0,
            twist_: 0,
            pointer_type_: String::new(),
            is_primary_: false,
            coalesced_events_: Vec::new(),
            predicted_events_: Vec::new(),
            persistent_device_id_: 0,
            prevent_counting_as_interaction_: prevent_counting_as_interaction,
        };
        if let Some(value) = initializer.pointer_id {
            event.pointer_id_ = value;
        }
        if let Some(value) = initializer.width {
            event.width_ = value;
        }
        if let Some(value) = initializer.height {
            event.height_ = value;
        }
        if let Some(value) = initializer.pressure {
            event.pressure_ = value;
        }
        if let Some(value) = initializer.tilt_x {
            event.tilt_x_ = value;
        }
        if let Some(value) = initializer.tilt_y {
            event.tilt_y_ = value;
        }
        if let Some(value) = initializer.tangential_pressure {
            event.tangential_pressure_ = value;
        }
        if let Some(value) = initializer.twist {
            event.twist_ = value;
        }
        if let Some(value) = &initializer.pointer_type {
            event.pointer_type_ = value.clone();
        }
        if let Some(value) = initializer.is_primary {
            event.is_primary_ = value;
        }
        if let Some(coalesced_events) = &initializer.coalesced_events {
            for coalesced_event in coalesced_events {
                event.coalesced_events_.push(coalesced_event.clone());
            }
        }
        if let Some(predicted_events) = &initializer.predicted_events {
            for predicted_event in predicted_events {
                event.predicted_events_.push(predicted_event.clone());
            }
        }
        if let Some(value) = initializer.azimuth_angle {
            event.azimuth_angle_ = value;
        }
        if let Some(value) = initializer.altitude_angle {
            event.altitude_angle_ = value;
        }
        if (initializer.tilt_x.is_some() || initializer.tilt_y.is_some())
            && initializer.azimuth_angle.is_none()
            && initializer.altitude_angle.is_none()
        {
            event.azimuth_angle_ = PointerEventUtil::AzimuthFromTilt(
                PointerEventUtil::TransformToTiltInValidRange(event.tilt_x_) as f64,
                PointerEventUtil::TransformToTiltInValidRange(event.tilt_y_) as f64,
            );
            event.altitude_angle_ = PointerEventUtil::AltitudeFromTilt(
                PointerEventUtil::TransformToTiltInValidRange(event.tilt_x_) as f64,
                PointerEventUtil::TransformToTiltInValidRange(event.tilt_y_) as f64,
            );
        }
        if (initializer.azimuth_angle.is_some() || initializer.altitude_angle.is_some())
            && initializer.tilt_x.is_none()
            && initializer.tilt_y.is_none()
        {
            event.tilt_x_ = PointerEventUtil::TiltXFromSpherical(
                PointerEventUtil::TransformToAzimuthInValidRange(event.azimuth_angle_),
                PointerEventUtil::TransformToAltitudeInValidRange(event.altitude_angle_),
            );
            event.tilt_y_ = PointerEventUtil::TiltYFromSpherical(
                PointerEventUtil::TransformToAzimuthInValidRange(event.azimuth_angle_),
                PointerEventUtil::TransformToAltitudeInValidRange(event.altitude_angle_),
            );
        }
        if let Some(value) = initializer.persistent_device_id {
            event.persistent_device_id_ = value;
        }
        event
    }

    // Source: pointer_event.h:44-56
    pub fn pointerId(&self) -> PointerId {
        self.pointer_id_
    }
    pub fn width(&self) -> f64 {
        self.width_
    }
    pub fn height(&self) -> f64 {
        self.height_
    }
    pub fn pressure(&self) -> f32 {
        self.pressure_
    }
    pub fn tiltX(&self) -> i32 {
        self.tilt_x_
    }
    pub fn tiltY(&self) -> i32 {
        self.tilt_y_
    }
    pub fn azimuthAngle(&self) -> f64 {
        self.azimuth_angle_
    }
    pub fn altitudeAngle(&self) -> f64 {
        self.altitude_angle_
    }
    pub fn tangentialPressure(&self) -> f32 {
        self.tangential_pressure_
    }
    pub fn twist(&self) -> i32 {
        self.twist_
    }
    pub fn pointerType(&self) -> &str {
        &self.pointer_type_
    }
    pub fn isPrimary(&self) -> bool {
        self.is_primary_
    }

    // Source: pointer_event.h:58
    pub fn button(&self) -> i16 {
        self.mouse_event.RawButton()
    }

    // Source: pointer_event.h:59; pointer_event.cc:100-108
    pub fn IsMouseEvent(&self) -> bool {
        matches!(self.r#type(), "click" | "auxclick" | "contextmenu")
    }

    // Source: pointer_event.h:120; pointer_event.cc:110-117
    fn ShouldHaveIntegerCoordinates(&self) -> bool {
        matches!(self.r#type(), "click" | "contextmenu" | "auxclick")
    }

    // Source: pointer_event.h:60; pointer_event.cc:119-121
    pub fn IsPointerEvent(&self) -> bool {
        true
    }

    // Source: pointer_event.h:62-91
    pub fn screenX(&self) -> f64 {
        if self.ShouldHaveIntegerCoordinates() {
            self.mouse_event.screenX()
        } else {
            self.mouse_event.FractionalScreenX()
        }
    }
    pub fn screenY(&self) -> f64 {
        if self.ShouldHaveIntegerCoordinates() {
            self.mouse_event.screenY()
        } else {
            self.mouse_event.FractionalScreenY()
        }
    }
    pub fn clientX(&self) -> f64 {
        if self.ShouldHaveIntegerCoordinates() {
            self.mouse_event.clientX()
        } else {
            self.mouse_event.FractionalClientX()
        }
    }
    pub fn clientY(&self) -> f64 {
        if self.ShouldHaveIntegerCoordinates() {
            self.mouse_event.clientY()
        } else {
            self.mouse_event.FractionalClientY()
        }
    }
    pub fn pageX(&self) -> f64 {
        if self.ShouldHaveIntegerCoordinates() {
            self.mouse_event.pageX()
        } else {
            self.mouse_event.FractionalPageX()
        }
    }
    pub fn pageY(&self) -> f64 {
        if self.ShouldHaveIntegerCoordinates() {
            self.mouse_event.pageY()
        } else {
            self.mouse_event.FractionalPageY()
        }
    }

    // Source: pointer_event.h:93; pointer_event.cc:123-131
    pub fn offsetX(&self) -> f64 {
        if self.ShouldHaveIntegerCoordinates() {
            return self.mouse_event.offsetX();
        }
        if !self.HasPosition() {
            return 0.0;
        }
        self.mouse_event.FractionalOffsetX()
    }
    // Source: pointer_event.h:94; pointer_event.cc:133-141
    pub fn offsetY(&self) -> f64 {
        if self.ShouldHaveIntegerCoordinates() {
            return self.mouse_event.offsetY();
        }
        if !self.HasPosition() {
            return 0.0;
        }
        self.mouse_event.FractionalOffsetY()
    }

    // Source: pointer_event.h:96; pointer_event.cc:143-145
    pub fn ReceivedTarget(&mut self) {
        self.mouse_event.ReceivedTarget();
    }

    // Source: pointer_event.h:98-101; pointer_event.cc:147-153
    pub fn toElement(&self) -> Option<EventTargetRef> {
        None
    }
    pub fn fromElement(&self) -> Option<EventTargetRef> {
        None
    }

    // Source: pointer_event.h:103-104; pointer_event.cc:155-161
    pub fn getCoalescedEvents(&self) -> Vec<PointerEventRef> {
        self.coalesced_events_.clone()
    }
    pub fn getPredictedEvents(&self) -> Vec<PointerEventRef> {
        self.predicted_events_.clone()
    }

    // Source: pointer_event.h:105; pointer_event.cc:163-169
    pub fn OldestPlatformTimeStamp(&self) -> TimeTicks {
        if !self.coalesced_events_.is_empty() {
            return self.coalesced_events_[0].borrow().PlatformTimeStamp();
        }
        self.PlatformTimeStamp()
    }

    // Source: pointer_event.h:117; pointer_event.cc:171-175
    // Rc/Drop trace and release the same coalesced/predicted/base owners.

    // Source: pointer_event.h:107; pointer_event.cc:177-199
    // Capture retained handles before dispatch, then release the event borrow.
    pub fn DispatchHook<'a>(
        &self,
        nested: &'a dyn MouseNestedDispatch,
    ) -> PointerEventDispatchHook<'a> {
        PointerEventDispatchHook {
            coalesced_events: self.coalesced_events_.clone(),
            predicted_events: self.predicted_events_.clone(),
            mouse_hook: MouseEventDispatchHook {
                mouse: self.mouse_event.DispatchValues(),
                nested,
            },
        }
    }

    // Source: pointer_event.h:45; pointer_event.cc:201-205
    pub fn pointerIdForBindings(&self) -> PointerId {
        // TODO: pointer_event.cc:202-203 Document UseCounter(kPointerId).
        self.pointerId()
    }

    // Source: pointer_event.h:109; pointer_event.cc:207-211
    pub fn GetDocument(
        &self,
        query: &dyn PointerEventDocumentQuery,
    ) -> Option<PointerEventDocumentRef> {
        self.view()
            .and_then(|view| query.DocumentForLocalDOMWindow(&view))
    }

    // Source: pointer_event.h:111-115
    pub fn persistentDeviceId(&self) -> i32 {
        self.persistent_device_id_
    }
    pub fn GetPreventCountingAsInteraction(&self) -> bool {
        self.prevent_counting_as_interaction_
    }

    // Source: mouse_event.h:169-170,112-115; mouse_event.cc:340-353
    // Inherited helpers call PointerEvent's virtual projections in C++.
    pub fn x(&self) -> f64 {
        self.clientX()
    }
    pub fn y(&self) -> f64 {
        self.clientY()
    }
    pub fn IsLeftButton(&self) -> bool {
        self.button() == 0
    }
    pub fn IsLinkClickButton(&self) -> bool {
        matches!(self.button(), 0 | 1)
    }
}

// Source: pointer_event.h:107; pointer_event.cc:177-199
pub struct PointerEventDispatchHook<'a> {
    coalesced_events: Vec<PointerEventRef>,
    predicted_events: Vec<PointerEventRef>,
    mouse_hook: MouseEventDispatchHook<'a>,
}
impl EventDispatchHook for PointerEventDispatchHook<'_> {
    fn DispatchEvent(&self, dispatcher: &mut EventDispatcher<'_>) -> DispatchEventResult {
        let event = dispatcher.GetEvent();
        if event.borrow().BaseEvent().r#type().is_empty() {
            return DispatchEventResult::kNotCanceled;
        }
        if event.borrow().BaseEvent().isTrusted() {
            for coalesced_event in &self.coalesced_events {
                coalesced_event
                    .borrow_mut()
                    .SetTarget(Some(dispatcher.GetNode()));
            }
            for predicted_event in &self.predicted_events {
                predicted_event
                    .borrow_mut()
                    .SetTarget(Some(dispatcher.GetNode()));
            }
        }
        if event.borrow().BaseEvent().r#type() == "click" {
            return self.mouse_hook.DispatchEvent(dispatcher);
        }
        {
            let event = event.borrow();
            if let (Some(target), Some(related)) =
                (event.BaseEvent().RawTarget(), event.relatedTarget())
            {
                debug_assert!(!Rc::ptr_eq(&target, &related));
            }
        }
        let related_target = event.borrow().relatedTarget();
        dispatcher
            .GetEventPath()
            .borrow_mut()
            .AdjustForRelatedTarget(&dispatcher.GetNode(), related_target);
        dispatcher.Dispatch()
    }
}

// Source: pointer_event.h:18
impl Deref for PointerEvent {
    type Target = MouseEvent;
    fn deref(&self) -> &Self::Target {
        &self.mouse_event
    }
}
impl DerefMut for PointerEvent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mouse_event
    }
}

impl EventInterface for PointerEvent {
    fn PrepareDispatchHook<'a>(
        &self,
        nested: &'a dyn crate::core::events::mouse_event::MouseNestedDispatch,
    ) -> Box<dyn crate::core::dom::events::event_dispatcher::EventDispatchHook + 'a> {
        Box::new(self.DispatchHook(nested))
    }
    fn AsMouseEvent(&self) -> Option<&MouseEvent> {
        Some(&self.mouse_event)
    }
    fn AsPointerEvent(&self) -> Option<&PointerEvent> {
        Some(self)
    }
    fn AsUIEventWithKeyState(&self) -> Option<&UIEventWithKeyState> {
        Some(self.mouse_event.BaseUIEventWithKeyState())
    }
    fn BaseEvent(&self) -> &Event {
        self.mouse_event.BaseEvent()
    }
    fn BaseEventMut(&mut self) -> &mut Event {
        self.mouse_event.BaseEventMut()
    }
    fn InterfaceName(&self) -> &str {
        self.mouse_event.InterfaceName()
    }
    fn IsUIEvent(&self) -> bool {
        true
    }
    fn IsMouseEvent(&self) -> bool {
        PointerEvent::IsMouseEvent(self)
    }
    // Source: pointer_event.h:145-148; pointer_event.cc:119-121
    fn IsPointerEvent(&self) -> bool {
        true
    }
    fn relatedTarget(&self) -> Option<EventTargetRef> {
        self.mouse_event.relatedTarget()
    }
    fn SetRelatedTarget(&mut self, target: Option<EventTargetRef>) {
        self.mouse_event.SetRelatedTarget(target);
    }
    fn ReceivedTarget(&mut self) {
        PointerEvent::ReceivedTarget(self);
    }
}
impl UIEventInterface for PointerEvent {
    fn BaseUIEvent(&self) -> &UIEvent {
        self.mouse_event.BaseUIEvent()
    }
    fn BaseUIEventMut(&mut self) -> &mut UIEvent {
        self.mouse_event.BaseUIEventMut()
    }
    fn which(&self) -> u32 {
        self.mouse_event.which()
    }
}
impl UIEventWithKeyStateInterface for PointerEvent {
    fn BaseUIEventWithKeyState(&self) -> &UIEventWithKeyState {
        self.mouse_event.BaseUIEventWithKeyState()
    }
    fn BaseUIEventWithKeyStateMut(&mut self) -> &mut UIEventWithKeyState {
        self.mouse_event.BaseUIEventWithKeyStateMut()
    }
}
impl MouseEventInterface for PointerEvent {
    fn BaseMouseEvent(&self) -> &MouseEvent {
        &self.mouse_event
    }
    fn BaseMouseEventMut(&mut self) -> &mut MouseEvent {
        &mut self.mouse_event
    }
    fn button(&self) -> i16 {
        PointerEvent::button(self)
    }
    fn toElement(&self) -> Option<EventTargetRef> {
        None
    }
    fn fromElement(&self) -> Option<EventTargetRef> {
        None
    }
    fn screenX(&self) -> f64 {
        PointerEvent::screenX(self)
    }
    fn screenY(&self) -> f64 {
        PointerEvent::screenY(self)
    }
    fn clientX(&self) -> f64 {
        PointerEvent::clientX(self)
    }
    fn clientY(&self) -> f64 {
        PointerEvent::clientY(self)
    }
    fn offsetX(&self) -> f64 {
        PointerEvent::offsetX(self)
    }
    fn offsetY(&self) -> f64 {
        PointerEvent::offsetY(self)
    }
    fn pageX(&self) -> f64 {
        PointerEvent::pageX(self)
    }
    fn pageY(&self) -> f64 {
        PointerEvent::pageY(self)
    }
}
