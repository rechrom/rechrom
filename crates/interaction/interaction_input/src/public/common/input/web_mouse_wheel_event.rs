// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:1-127
// Source: third_party/blink/common/input/web_mouse_wheel_event.cc:1-143
// Rust modules replace C++ includes, include guards and namespaces.

use std::any::Any;
use std::ops::{Deref, DerefMut};

use super::web_input_event::{DispatchType, RailsMode, Type, WebInputEvent, WebInputEventPayload};
use super::web_mouse_event::WebMouseEvent;
use crate::base::time::time::TimeTicks;
pub use crate::ui::events::types::scroll_types::ScrollGranularity;

// Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:17-40
// A transparent integer retains combinations of the source's phase bits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct Phase(pub u32);

impl Phase {
    pub const kPhaseNone: Self = Self(0);
    pub const kPhaseBegan: Self = Self(1 << 0);
    pub const kPhaseStationary: Self = Self(1 << 1);
    pub const kPhaseChanged: Self = Self(1 << 2);
    pub const kPhaseEnded: Self = Self(1 << 3);
    pub const kPhaseCancelled: Self = Self(1 << 4);
    pub const kPhaseMayBegin: Self = Self(1 << 5);
    pub const kPhaseBlocked: Self = Self(1 << 6);
}

// Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:42-53
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EventAction {
    kPageZoom = 0,
    kScroll,
    kScrollHorizontal,
    kScrollVertical,
}

// Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:15,55-85
#[derive(Clone, Debug)]
pub struct WebMouseWheelEvent {
    pub mouse_event: WebMouseEvent,
    // Wheel deltas are positive up/left, opposite to Blink/CC/WebAPI scrolling.
    pub delta_x: f32,
    pub delta_y: f32,
    pub wheel_ticks_x: f32,
    pub wheel_ticks_y: f32,
    pub acceleration_ratio_x: f32,
    pub acceleration_ratio_y: f32,
    pub phase: Phase,
    pub momentum_phase: Phase,
    pub rails_mode: RailsMode,
    pub dispatch_type: DispatchType,
    pub event_action: EventAction,
    pub has_synthetic_phase: bool,
    pub delta_units: ScrollGranularity,
}

impl WebMouseWheelEvent {
    // Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:17-40
    // Preserve the source's class-scoped enumerator spelling.
    pub const kPhaseNone: Phase = Phase::kPhaseNone;
    pub const kPhaseBegan: Phase = Phase::kPhaseBegan;
    pub const kPhaseStationary: Phase = Phase::kPhaseStationary;
    pub const kPhaseChanged: Phase = Phase::kPhaseChanged;
    pub const kPhaseEnded: Phase = Phase::kPhaseEnded;
    pub const kPhaseCancelled: Phase = Phase::kPhaseCancelled;
    pub const kPhaseMayBegin: Phase = Phase::kPhaseMayBegin;
    pub const kPhaseBlocked: Phase = Phase::kPhaseBlocked;

    // Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:55-93
    pub fn new(event_type: Type, modifiers: i32, time_stamp: TimeTicks) -> Self {
        Self {
            mouse_event: WebMouseEvent::WithTypeRange(
                event_type,
                Type::kMouseWheel,
                Type::kMouseWheel,
                modifiers,
                time_stamp,
                WebMouseEvent::kMousePointerId,
            ),
            delta_x: 0.0,
            delta_y: 0.0,
            wheel_ticks_x: 0.0,
            wheel_ticks_y: 0.0,
            acceleration_ratio_x: 1.0,
            acceleration_ratio_y: 1.0,
            phase: Phase::kPhaseNone,
            momentum_phase: Phase::kPhaseNone,
            rails_mode: RailsMode::kRailsModeFree,
            dispatch_type: DispatchType::kBlocking,
            event_action: EventAction::kPageZoom,
            has_synthetic_phase: false,
            delta_units: ScrollGranularity::kScrollByPixel,
        }
    }

    // Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:110
    pub fn IsCancelable(&self) -> bool {
        self.dispatch_type == DispatchType::kBlocking
    }
}

// Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:95-101
impl Default for WebMouseWheelEvent {
    fn default() -> Self {
        Self::new(
            Type::kUndefined,
            WebInputEvent::kNoModifiers,
            TimeTicks::new(),
        )
    }
}

// Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:15
// Composition and dereferencing retain the inherited base state identity.
impl Deref for WebMouseWheelEvent {
    type Target = WebMouseEvent;

    fn deref(&self) -> &Self::Target {
        &self.mouse_event
    }
}

impl DerefMut for WebMouseWheelEvent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mouse_event
    }
}

// Source: third_party/blink/common/input/web_mouse_wheel_event.cc:13-15
fn GetUnacceleratedDelta(accelerated_delta: f32, acceleration_ratio: f32) -> f32 {
    accelerated_delta * acceleration_ratio
}

// Source: third_party/blink/common/input/web_mouse_wheel_event.cc:17-21
fn GetAccelerationRatio(accelerated_delta: f32, unaccelerated_delta: f32) -> f32 {
    if unaccelerated_delta == 0.0 || accelerated_delta == 0.0 {
        return 1.0;
    }
    unaccelerated_delta / accelerated_delta
}

impl WebInputEventPayload for WebMouseWheelEvent {
    // Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:15
    fn Event(&self) -> &WebInputEvent {
        &self.mouse_event.event
    }

    fn EventMut(&mut self) -> &mut WebInputEvent {
        &mut self.mouse_event.event
    }

    fn AsAny(&self) -> &dyn Any {
        self
    }
    fn AsAnyMut(&mut self) -> &mut dyn Any {
        self
    }

    // Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:112
    // Source: third_party/blink/common/input/web_mouse_wheel_event.cc:25-27
    fn Clone(&self) -> Box<dyn WebInputEventPayload> {
        Box::new(std::clone::Clone::clone(self))
    }

    // Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:113
    // Source: third_party/blink/common/input/web_mouse_wheel_event.cc:29-38
    fn CanCoalesce(&self, event: &dyn WebInputEventPayload) -> bool {
        if event.Event().GetType() != Type::kMouseWheel {
            return false;
        }
        let mouse_wheel_event = event
            .AsAny()
            .downcast_ref::<Self>()
            .expect("wheel event type must have a WebMouseWheelEvent payload");
        self.GetModifiers() == mouse_wheel_event.GetModifiers()
            && self.delta_units == mouse_wheel_event.delta_units
            && self.HaveConsistentPhase(mouse_wheel_event)
    }

    // Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:114
    // Source: third_party/blink/common/input/web_mouse_wheel_event.cc:58-97
    fn Coalesce(&mut self, event: &dyn WebInputEventPayload) {
        debug_assert!(self.CanCoalesce(event));
        let mouse_wheel_event = event
            .AsAny()
            .downcast_ref::<Self>()
            .expect("wheel event type must have a WebMouseWheelEvent payload");
        let unaccelerated_x = GetUnacceleratedDelta(self.delta_x, self.acceleration_ratio_x)
            + GetUnacceleratedDelta(
                mouse_wheel_event.delta_x,
                mouse_wheel_event.acceleration_ratio_x,
            );
        let unaccelerated_y = GetUnacceleratedDelta(self.delta_y, self.acceleration_ratio_y)
            + GetUnacceleratedDelta(
                mouse_wheel_event.delta_y,
                mouse_wheel_event.acceleration_ratio_y,
            );
        let old_delta_x = self.delta_x;
        let old_delta_y = self.delta_y;
        let old_wheel_ticks_x = self.wheel_ticks_x;
        let old_wheel_ticks_y = self.wheel_ticks_y;
        // The source retains movement in float temporaries before int += float.
        let old_movement_x = self.mouse_event.pointer_properties.movement_x as f32;
        let old_movement_y = self.mouse_event.pointer_properties.movement_y as f32;
        let old_phase = self.phase;
        let old_dispatch_type = self.dispatch_type;
        *self = std::clone::Clone::clone(mouse_wheel_event);
        self.delta_x += old_delta_x;
        self.delta_y += old_delta_y;
        self.wheel_ticks_x += old_wheel_ticks_x;
        self.wheel_ticks_y += old_wheel_ticks_y;
        self.mouse_event.pointer_properties.movement_x =
            (self.mouse_event.pointer_properties.movement_x as f32 + old_movement_x) as i32;
        self.mouse_event.pointer_properties.movement_y =
            (self.mouse_event.pointer_properties.movement_y as f32 + old_movement_y) as i32;
        self.acceleration_ratio_x = GetAccelerationRatio(self.delta_x, unaccelerated_x);
        self.acceleration_ratio_y = GetAccelerationRatio(self.delta_y, unaccelerated_y);
        self.dispatch_type =
            WebInputEvent::MergeDispatchTypes(old_dispatch_type, mouse_wheel_event.dispatch_type);
        if mouse_wheel_event.has_synthetic_phase && mouse_wheel_event.phase != old_phase {
            debug_assert_eq!(Phase::kPhaseChanged, mouse_wheel_event.phase);
            debug_assert_eq!(Phase::kPhaseBegan, old_phase);
            self.phase = Phase::kPhaseBegan;
        }
    }
}

impl WebMouseWheelEvent {
    // Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:121-122
    // Source: third_party/blink/common/input/web_mouse_wheel_event.cc:40-56
    fn HaveConsistentPhase(&self, event: &Self) -> bool {
        if self.has_synthetic_phase != event.has_synthetic_phase {
            return false;
        }
        if self.phase == event.phase && self.momentum_phase == event.momentum_phase {
            return true;
        }
        if self.has_synthetic_phase {
            return self.phase == Phase::kPhaseBegan && event.phase == Phase::kPhaseChanged;
        }
        false
    }

    // Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:103
    // Source: third_party/blink/common/input/web_mouse_wheel_event.cc:99-101
    pub fn DeltaXInRootFrame(&self) -> f32 {
        self.delta_x / self.FrameScale()
    }

    // Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:104
    // Source: third_party/blink/common/input/web_mouse_wheel_event.cc:103-105
    pub fn DeltaYInRootFrame(&self) -> f32 {
        self.delta_y / self.FrameScale()
    }

    // Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:106-108
    // Source: third_party/blink/common/input/web_mouse_wheel_event.cc:107-113
    pub fn FlattenTransform(&self) -> Self {
        let mut result = std::clone::Clone::clone(self);
        result.delta_x /= result.FrameScale();
        result.delta_y /= result.FrameScale();
        result.mouse_event.FlattenTransformSelf();
        result
    }

    // Source: third_party/blink/public/common/input/web_mouse_wheel_event.h:116-119
    // Source: third_party/blink/common/input/web_mouse_wheel_event.cc:115-141
    pub fn GetPlatformSpecificDefaultEventAction(event: &Self) -> EventAction {
        // Source: build/config/ui.gni:27 and build/config/BUILD.gn:58-60
        // Chromium enables USE_AURA on Windows/Linux/ChromeOS/Fuchsia. ChromeOS
        // is covered by Rust's Linux target; Android has its own source branch.
        #[cfg(any(
            target_os = "windows",
            target_os = "linux",
            target_os = "fuchsia",
            target_os = "android"
        ))]
        {
            if event.delta_units != ScrollGranularity::kScrollByPrecisePixel
                && event.GetModifiers() & WebInputEvent::kControlKey != 0
            {
                return EventAction::kPageZoom;
            }
            if event.delta_x == 0.0 && event.GetModifiers() & WebInputEvent::kShiftKey != 0 {
                return EventAction::kScrollHorizontal;
            }
        }
        if event.rails_mode == RailsMode::kRailsModeHorizontal
            || (event.delta_x != 0.0 && event.delta_y == 0.0)
        {
            return EventAction::kScrollHorizontal;
        }
        if event.rails_mode == RailsMode::kRailsModeVertical
            || (event.delta_x == 0.0 && event.delta_y != 0.0)
        {
            return EventAction::kScrollVertical;
        }
        EventAction::kScroll
    }
}
