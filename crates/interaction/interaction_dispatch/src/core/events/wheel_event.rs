/*
 * Copyright (C) 2001 Peter Kelly (pmk@post.com)
 * Copyright (C) 2001 Tobias Anton (anton@stud.fbi.fh-darmstadt.de)
 * Copyright (C) 2006 Samuel Weinig (sam.weinig@gmail.com)
 * Copyright (C) 2003, 2004, 2005, 2006, 2008, 2010 Apple Inc. All rights
 * reserved.
 * Copyright (C) 2013 Samsung Electronics. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB. If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/wheel_event.h:1-110
// Source: third_party/blink/renderer/core/events/wheel_event.cc:1-187
// Rust modules and an embedded MouseEvent replace includes, namespace,
// wrapper annotations, inheritance and GC tracing.

use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use interaction_input::base::time::time::TimeTicks;
use interaction_input::public::common::input::web_mouse_wheel_event::{
    ScrollGranularity, WebMouseWheelEvent,
};

use crate::core::dom::events::event::{
    Event, EventClock, EventDispatcher as EventDispatcherInterface, EventInterface, EventTargetRef,
    PassiveMode,
};
use crate::core::dom::events::event_dispatch_result::DispatchEventResult;
use crate::core::dom::events::event_dispatcher::{EventDispatchHook, EventDispatcher};

use super::mouse_event::{
    MenuSourceType, MouseCoordinateContext, MouseEvent, MouseEventInterface, SyntheticEventType,
};
use super::mouse_event_init::MouseEventInit;
use super::ui_event::{AbstractViewRef, UIEvent, UIEventInterface};
use super::ui_event_with_key_state::{UIEventWithKeyState, UIEventWithKeyStateInterface};
pub use super::wheel_event_init::WheelEventInit;

// Source: wheel_event.h:45
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum DeltaMode {
    kDomDeltaPixel = 0,
    kDomDeltaLine,
    kDomDeltaPage,
}

// Source: wheel_event.cc:51-65,103-130
// The window retains its identity. These inputs carry only the device scale
// and existing coordinate queries read by the source constructor.
pub struct WheelEventWindow {
    pub view: AbstractViewRef,
    pub device_pixel_ratio: f64,
    pub coordinates: MouseCoordinateContext,
    pub intervention: Option<Rc<dyn WheelInterventionReport>>,
}

// Source: wheel_event.cc:153-164
// The embedding supplies the frame-owned report sink. Dispatch does not gain
// access to the frame or its lifecycle through this effect interface.
pub trait WheelInterventionReport {
    fn HasLocalFrame(&self, view: &AbstractViewRef) -> bool;
    fn GenerateReport(
        &self,
        view: &AbstractViewRef,
        id: &str,
        message: &str,
        location: crate::core::frame::reporting_context::ReportLocation,
    );
}

// Source: wheel_event.h:39-40,93-100
pub struct WheelEvent {
    mouse_event: MouseEvent,
    // gfx::Point stores two signed integers; the tuple keeps that value only.
    wheel_delta_: (i32, i32),
    delta_x_: f64,
    delta_y_: f64,
    delta_z_: f64,
    delta_mode_: u32,
    native_event_: WebMouseWheelEvent,
    is_momentum_: bool,
    intervention_: Option<Rc<dyn WheelInterventionReport>>,
}

// Source: wheel_event.cc:41-49
fn ConvertDeltaMode(event: &WebMouseWheelEvent) -> u32 {
    debug_assert!(matches!(
        event.delta_units,
        ScrollGranularity::kScrollByPage
            | ScrollGranularity::kScrollByPixel
            | ScrollGranularity::kScrollByPrecisePixel
    ));
    if event.delta_units == ScrollGranularity::kScrollByPage {
        DeltaMode::kDomDeltaPage as u32
    } else {
        DeltaMode::kDomDeltaPixel as u32
    }
}

// Source: wheel_event.cc:51-72
fn GetMouseEventInitForWheel(
    event: &WebMouseWheelEvent,
    window: &WheelEventWindow,
) -> MouseEventInit {
    let mut initializer = MouseEventInit::default();
    initializer
        .event_modifier_init
        .ui_event_init
        .event_init
        .bubbles = true;
    initializer
        .event_modifier_init
        .ui_event_init
        .event_init
        .cancelable = event.IsCancelable();
    let flattened = event.FlattenTransform();
    MouseEvent::SetCoordinatesFromWebPointerProperties(
        &flattened.mouse_event.pointer_properties,
        Some(&window.view),
        window.coordinates.views.as_deref(),
        &mut initializer,
    );
    initializer.button = event.mouse_event.pointer_properties.button as i16;
    initializer.buttons = MouseEvent::WebInputEventModifiersToButtons(event.GetModifiers() as u32);
    initializer.event_modifier_init.ui_event_init.view = Some(window.view.clone());
    initializer
        .event_modifier_init
        .ui_event_init
        .event_init
        .composed = true;
    initializer.event_modifier_init.ui_event_init.detail = event.mouse_event.click_count;
    UIEventWithKeyState::SetFromWebInputEventModifiers(
        &mut initializer.event_modifier_init,
        event.GetModifiers(),
    );
    // Source cc:67-69 retains the upstream canvas-hit-region TODO.
    initializer
}

impl WheelEvent {
    // Source: wheel_event.h:43
    pub const kTickMultiplier: i32 = 120;

    // Source: wheel_event.h:45
    pub const kDomDeltaPixel: u32 = DeltaMode::kDomDeltaPixel as u32;
    pub const kDomDeltaLine: u32 = DeltaMode::kDomDeltaLine as u32;
    pub const kDomDeltaPage: u32 = DeltaMode::kDomDeltaPage as u32;

    // Source: wheel_event.h:47,57; wheel_event.cc:81-82
    pub fn Create(clock: &dyn EventClock) -> Self {
        Self::Uninitialized(clock.Now())
    }

    pub fn Uninitialized(platform_time_stamp: TimeTicks) -> Self {
        Self {
            mouse_event: MouseEvent::Uninitialized(platform_time_stamp),
            wheel_delta_: (0, 0),
            delta_x_: 0.0,
            delta_y_: 0.0,
            delta_z_: 0.0,
            delta_mode_: Self::kDomDeltaPixel,
            native_event_: WebMouseWheelEvent::default(),
            is_momentum_: false,
            intervention_: None,
        }
    }

    // Source: wheel_event.h:49-50; wheel_event.cc:76-79
    pub fn CreateFromNative(event: &WebMouseWheelEvent, window: &WheelEventWindow) -> Self {
        Self::FromNative(event, window)
    }

    // Source: wheel_event.h:52-55
    pub fn CreateWithInitializer(
        event_type: impl Into<String>,
        initializer: &WheelEventInit,
        clock: &dyn EventClock,
        coordinates: MouseCoordinateContext,
        intervention: Option<Rc<dyn WheelInterventionReport>>,
    ) -> Self {
        Self::new(
            event_type,
            initializer,
            clock.Now(),
            coordinates,
            intervention,
        )
    }

    // Source: wheel_event.h:58; wheel_event.cc:84-101
    pub fn new(
        event_type: impl Into<String>,
        initializer: &WheelEventInit,
        platform_time_stamp: TimeTicks,
        coordinates: MouseCoordinateContext,
        intervention: Option<Rc<dyn WheelInterventionReport>>,
    ) -> Self {
        Self {
            mouse_event: MouseEvent::new(
                event_type,
                &initializer.mouse_event_init,
                platform_time_stamp,
                SyntheticEventType::kRealOrIndistinguishable,
                MenuSourceType::kNone,
                None,
                coordinates,
            ),
            // Rust's float-to-int conversion performs the source ClampTo<i32>
            // saturation and truncation for the finite IDL double inputs.
            wheel_delta_: (
                if initializer.wheel_delta_x != 0 {
                    initializer.wheel_delta_x
                } else {
                    initializer.delta_x as i32
                },
                if initializer.wheel_delta_y != 0 {
                    initializer.wheel_delta_y
                } else {
                    initializer.delta_y as i32
                },
            ),
            delta_x_: if initializer.delta_x != 0.0 {
                initializer.delta_x
            } else {
                (-(initializer.wheel_delta_x as f64) as i32) as f64
            },
            delta_y_: if initializer.delta_y != 0.0 {
                initializer.delta_y
            } else {
                (-(initializer.wheel_delta_y as f64) as i32) as f64
            },
            delta_z_: initializer.delta_z,
            delta_mode_: initializer.delta_mode,
            native_event_: WebMouseWheelEvent::default(),
            is_momentum_: initializer.momentum,
            intervention_: intervention,
        }
    }

    // Source: wheel_event.h:59; wheel_event.cc:103-115
    pub fn FromNative(event: &WebMouseWheelEvent, window: &WheelEventWindow) -> Self {
        let initializer = GetMouseEventInitForWheel(event, window);
        Self {
            mouse_event: MouseEvent::new(
                "wheel",
                &initializer,
                event.TimeStamp(),
                SyntheticEventType::kRealOrIndistinguishable,
                MenuSourceType::kNone,
                None,
                window.coordinates.clone(),
            ),
            wheel_delta_: (
                ((event.wheel_ticks_x * Self::kTickMultiplier as f32) as f64
                    / window.device_pixel_ratio) as i32,
                ((event.wheel_ticks_y * Self::kTickMultiplier as f32) as f64
                    / window.device_pixel_ratio) as i32,
            ),
            delta_x_: -event.DeltaXInRootFrame() as f64 / window.device_pixel_ratio,
            delta_y_: -event.DeltaYInRootFrame() as f64 / window.device_pixel_ratio,
            delta_z_: 0.0,
            delta_mode_: ConvertDeltaMode(event),
            native_event_: event.clone(),
            is_momentum_: event.momentum_phase != WebMouseWheelEvent::kPhaseNone,
            intervention_: window.intervention.clone(),
        }
    }

    // Source: wheel_event.h:60-62; wheel_event.cc:117-130
    // The tuple is the existing gfx::Vector2dF's two values, already resolved
    // into pixels by the owning input stage.
    pub fn FromNativeWithPixelDelta(
        event: &WebMouseWheelEvent,
        delta_in_pixels: (f32, f32),
        window: &WheelEventWindow,
    ) -> Self {
        let initializer = GetMouseEventInitForWheel(event, window);
        Self {
            mouse_event: MouseEvent::new(
                "wheel",
                &initializer,
                event.TimeStamp(),
                SyntheticEventType::kRealOrIndistinguishable,
                MenuSourceType::kNone,
                None,
                window.coordinates.clone(),
            ),
            wheel_delta_: (
                (event.wheel_ticks_x * Self::kTickMultiplier as f32) as i32,
                (event.wheel_ticks_y * Self::kTickMultiplier as f32) as i32,
            ),
            delta_x_: delta_in_pixels.0 as f64,
            delta_y_: delta_in_pixels.1 as f64,
            delta_z_: 0.0,
            delta_mode_: Self::kDomDeltaPixel,
            native_event_: event.clone(),
            is_momentum_: event.momentum_phase != WebMouseWheelEvent::kPhaseNone,
            intervention_: window.intervention.clone(),
        }
    }

    // Source: wheel_event.h:64-77
    pub fn deltaX(&self) -> f64 {
        self.delta_x_
    }
    pub fn deltaY(&self) -> f64 {
        self.delta_y_
    }
    pub fn deltaZ(&self) -> f64 {
        self.delta_z_
    }
    pub fn wheelDelta(&self) -> i32 {
        if self.wheelDeltaY() != 0 {
            self.wheelDeltaY()
        } else {
            self.wheelDeltaX()
        }
    }
    pub fn wheelDeltaX(&self) -> i32 {
        self.wheel_delta_.0
    }
    pub fn wheelDeltaY(&self) -> i32 {
        self.wheel_delta_.1
    }
    pub fn deltaMode(&self) -> u32 {
        self.delta_mode_
    }
    pub fn momentum(&self) -> bool {
        self.is_momentum_
    }

    // Source: wheel_event.h:79-81; wheel_event.cc:132-142
    pub fn InterfaceName(&self) -> &str {
        "WheelEvent"
    }
    pub fn IsMouseEvent(&self) -> bool {
        false
    }
    pub fn IsWheelEvent(&self) -> bool {
        true
    }

    // Source: wheel_event.h:83; wheel_event.cc:144-177
    pub fn preventDefault(&mut self) {
        self.preventDefaultWithLocation(
            crate::core::frame::reporting_context::ReportLocation::default(),
        );
    }
    pub fn preventDefaultWithLocation(
        &mut self,
        location: crate::core::frame::reporting_context::ReportLocation,
    ) {
        self.mouse_event.BaseEventMut().preventDefault();
        if !self.IsFullyTrusted() {
            return;
        }
        let passive_mode = self.HandlingPassive();
        if passive_mode == PassiveMode::kPassiveForcedDocumentLevel {
            let id = "PreventDefaultPassive";
            let message = "Unable to preventDefault inside passive event listener due to target being treated as passive. See https://www.chromestatus.com/feature/6662647093133312";
            if let (Some(view), Some(report)) = (self.view(), &self.intervention_) {
                if report.HasLocalFrame(&view) {
                    report.GenerateReport(&view, id, message, location);
                }
            }
        }
        // TODO: wheel_event.cc:166-176 document-level passive-wheel UseCounter.
    }

    // Source: wheel_event.h:85
    pub fn NativeEvent(&self) -> &WebMouseWheelEvent {
        &self.native_event_
    }

    // Source: wheel_event.h:87-89; wheel_event.cc:179-181
    pub fn DispatchHook() -> WheelEventDispatchHook {
        WheelEventDispatchHook
    }

    // Source: wheel_event.h:91; wheel_event.cc:183-185
    // Rust field destruction traces/releases the same MouseEvent owners.
}

impl Deref for WheelEvent {
    type Target = MouseEvent;
    fn deref(&self) -> &Self::Target {
        &self.mouse_event
    }
}
impl DerefMut for WheelEvent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mouse_event
    }
}

// Source: wheel_event.h:79-89,103-106
// The EventInterface type flag replaces source DowncastTraits.
impl EventInterface for WheelEvent {
    fn PrepareDispatchHook<'a>(
        &self,
        _nested: &'a dyn crate::core::events::mouse_event::MouseNestedDispatch,
    ) -> Box<dyn crate::core::dom::events::event_dispatcher::EventDispatchHook + 'a> {
        Box::new(Self::DispatchHook())
    }
    fn AsWheelEvent(&self) -> Option<&WheelEvent> {
        Some(self)
    }
    fn AsMouseEvent(&self) -> Option<&MouseEvent> {
        Some(&self.mouse_event)
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
        WheelEvent::InterfaceName(self)
    }
    fn IsUIEvent(&self) -> bool {
        true
    }
    fn IsMouseEvent(&self) -> bool {
        false
    }
    fn IsWheelEvent(&self) -> bool {
        true
    }
    fn relatedTarget(&self) -> Option<EventTargetRef> {
        self.mouse_event.relatedTarget()
    }
    fn SetRelatedTarget(&mut self, target: Option<EventTargetRef>) {
        self.mouse_event.SetRelatedTarget(target);
    }
    fn ReceivedTarget(&mut self) {
        self.mouse_event.ReceivedTarget();
    }
    fn preventDefault(&mut self) {
        WheelEvent::preventDefault(self);
    }
    fn preventDefaultWithLocation(
        &mut self,
        location: crate::core::frame::reporting_context::ReportLocation,
    ) {
        WheelEvent::preventDefaultWithLocation(self, location);
    }

    fn DispatchEvent(
        &mut self,
        dispatcher: &mut dyn EventDispatcherInterface,
    ) -> DispatchEventResult {
        dispatcher.Dispatch()
    }
}
impl UIEventInterface for WheelEvent {
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
impl UIEventWithKeyStateInterface for WheelEvent {
    fn BaseUIEventWithKeyState(&self) -> &UIEventWithKeyState {
        self.mouse_event.BaseUIEventWithKeyState()
    }
    fn BaseUIEventWithKeyStateMut(&mut self) -> &mut UIEventWithKeyState {
        self.mouse_event.BaseUIEventWithKeyStateMut()
    }
}
impl MouseEventInterface for WheelEvent {
    fn BaseMouseEvent(&self) -> &MouseEvent {
        &self.mouse_event
    }
    fn BaseMouseEventMut(&mut self) -> &mut MouseEvent {
        &mut self.mouse_event
    }
}

// Source: wheel_event.cc:179-181
// Dispatching through this shared-owner hook holds no EventRef borrow across
// listener invocation, and deliberately bypasses MouseEvent's path adjustment.
pub struct WheelEventDispatchHook;
impl EventDispatchHook for WheelEventDispatchHook {
    fn DispatchEvent(&self, dispatcher: &mut EventDispatcher<'_>) -> DispatchEventResult {
        dispatcher.Dispatch()
    }
}
