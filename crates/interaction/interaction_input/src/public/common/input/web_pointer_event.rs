// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/common/input/web_pointer_event.h:1-96
// Source: third_party/blink/common/input/web_pointer_event.cc:1-132
// Rust modules and embedded base values replace the C++ includes, namespace,
// export annotation, and multiple inheritance.

use std::any::Any;
use std::ops::{Deref, DerefMut};

use super::web_input_event::{DispatchType, Type, WebInputEvent, WebInputEventPayload};
use super::web_mouse_event::WebMouseEvent;
use super::web_pointer_properties::{Button, PointerType, WebPointerProperties};
use crate::base::time::time::TimeTicks;
pub use crate::public::mojom::input::touch_event::TouchState;

// Source: third_party/blink/public/common/input/web_pointer_event.h:41
// Source: third_party/blink/common/input/web_pointer_event.cc:33-62
// A borrowed projection of the WebTouchEvent values read by the constructor.
// The translated concrete touch event supplies this view when it is admitted.
pub trait WebTouchEventView {
    fn Event(&self) -> &WebInputEvent;
    fn Hovering(&self) -> bool;
    fn DispatchType(&self) -> DispatchType;
    fn MovedBeyondSlopRegion(&self) -> bool;
    fn TouchStartOrFirstTouchMove(&self) -> bool;
    fn UniqueTouchEventId(&self) -> u32;
}

// Source: third_party/blink/public/common/input/web_touch_point.h:42-55
// Source: third_party/blink/common/input/web_pointer_event.cc:33-62
// The base properties are borrowed from the same touch point, preserving the
// source base-subobject copy rather than reconstructing its fields.
pub trait WebTouchPointView {
    fn PointerProperties(&self) -> &WebPointerProperties;
    fn State(&self) -> TouchState;
    fn RadiusX(&self) -> f32;
    fn RadiusY(&self) -> f32;
    fn RotationAngle(&self) -> f32;
}

// Source: third_party/blink/public/common/input/web_pointer_event.h:15-23,52-85
#[derive(Clone, Debug)]
pub struct WebPointerEvent {
    pub event: WebInputEvent,
    pub pointer_properties: WebPointerProperties,

    // Source: third_party/blink/public/common/input/web_pointer_event.h:52-54
    pub rotation_angle: f32,

    // Source: third_party/blink/public/common/input/web_pointer_event.h:56-60
    pub unique_touch_event_id: u32,

    // Source: third_party/blink/public/common/input/web_pointer_event.h:62-64
    pub dispatch_type: DispatchType,

    // Source: third_party/blink/public/common/input/web_pointer_event.h:66-69
    pub moved_beyond_slop_region: bool,

    // Source: third_party/blink/public/common/input/web_pointer_event.h:71-73
    pub touch_start_or_first_touch_move: bool,

    // Source: third_party/blink/public/common/input/web_pointer_event.h:75-80
    pub hovering: bool,

    // Source: third_party/blink/public/common/input/web_pointer_event.h:82-85
    pub width: f32,
    pub height: f32,
}

// Source: third_party/blink/public/common/input/web_pointer_event.h:24-28,52-85
impl Default for WebPointerEvent {
    fn default() -> Self {
        Self {
            event: WebInputEvent::new(
                Type::kUndefined,
                Type::kPointerTypeFirst,
                Type::kPointerTypeLast,
                WebInputEvent::kNoModifiers,
                TimeTicks::new(),
            ),
            pointer_properties: WebPointerProperties::WithId(0),
            rotation_angle: 0.0,
            unique_touch_event_id: 0,
            dispatch_type: DispatchType::kBlocking,
            moved_beyond_slop_region: false,
            touch_start_or_first_touch_move: false,
            hovering: false,
            width: 0.0,
            height: 0.0,
        }
    }
}

impl WebPointerEvent {
    // Source: third_party/blink/public/common/input/web_pointer_event.h:29-40
    pub fn new(
        event_type: Type,
        pointer_properties: WebPointerProperties,
        width: f32,
        height: f32,
    ) -> Self {
        let mut event = Self {
            pointer_properties,
            width,
            height,
            ..Self::default()
        };
        event.SetType(event_type);
        event
    }

    // Source: third_party/blink/public/common/input/web_pointer_event.h:41-42
    // FromTouch and FromMouse are defined in the .cc translation below.

    // Source: third_party/blink/public/common/input/web_pointer_event.h:44-46
    // Clone, CanCoalesce, and Coalesce are defined in the payload trait below.

    // Source: third_party/blink/public/common/input/web_pointer_event.h:48-50
    // CreatePointerCausesUaActionEvent is defined in the .cc translation below.

    // Source: third_party/blink/public/common/input/web_pointer_event.h:87
    pub fn IsCancelable(&self) -> bool {
        self.dispatch_type == DispatchType::kBlocking
    }

    // Source: third_party/blink/public/common/input/web_pointer_event.h:88
    pub fn HasWidth(&self) -> bool {
        !self.width.is_nan()
    }

    // Source: third_party/blink/public/common/input/web_pointer_event.h:89
    pub fn HasHeight(&self) -> bool {
        !self.height.is_nan()
    }

    // Source: third_party/blink/public/common/input/web_pointer_event.h:91
    // WebPointerEventInRootFrame is defined in the .cc translation below.
}

// Source: third_party/blink/public/common/input/web_pointer_event.h:21-22
impl Deref for WebPointerEvent {
    type Target = WebInputEvent;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl DerefMut for WebPointerEvent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event
    }
}

// Source: third_party/blink/common/input/web_pointer_event.cc:12-31
fn PointerEventTypeForTouchPointState(state: TouchState) -> Type {
    match state {
        TouchState::kStateReleased => Type::kPointerUp,
        TouchState::kStateCancelled => Type::kPointerCancel,
        TouchState::kStatePressed => Type::kPointerDown,
        TouchState::kStateMoved => Type::kPointerMove,
        TouchState::kStateStationary | TouchState::kStateUndefined => {
            unreachable!("a stationary or undefined touch point has no pointer event type")
        }
    }
}

impl WebPointerEvent {
    // Source: third_party/blink/common/input/web_pointer_event.cc:33-62
    pub fn FromTouch(
        touch_event: &impl WebTouchEventView,
        touch_point: &impl WebTouchPointView,
    ) -> Self {
        let source_event = touch_event.Event();
        let mut event = Self {
            event: WebInputEvent::new(
                PointerEventTypeForTouchPointState(touch_point.State()),
                Type::kPointerTypeFirst,
                Type::kPointerTypeLast,
                source_event.GetModifiers(),
                source_event.TimeStamp(),
            ),
            pointer_properties: touch_point.PointerProperties().clone(),
            hovering: touch_event.Hovering(),
            width: touch_point.RadiusX() * 2.0,
            height: touch_point.RadiusY() * 2.0,
            rotation_angle: 0.0,
            unique_touch_event_id: 0,
            dispatch_type: DispatchType::kBlocking,
            moved_beyond_slop_region: false,
            touch_start_or_first_touch_move: false,
        };
        event.SetFrameScale(source_event.FrameScale());
        event.SetFrameTranslate(source_event.FrameTranslate());
        event.dispatch_type = touch_event.DispatchType();
        event.moved_beyond_slop_region = touch_event.MovedBeyondSlopRegion();
        event.touch_start_or_first_touch_move = touch_event.TouchStartOrFirstTouchMove();
        event.unique_touch_event_id = touch_event.UniqueTouchEventId();
        event.rotation_angle = touch_point.RotationAngle();
        // The source touch point does not carry its button at this point yet.
        event.pointer_properties.button =
            if event.GetType() == Type::kPointerDown || event.GetType() == Type::kPointerUp {
                Button::kLeft
            } else {
                Button::kNoButton
            };
        if source_event.GetPreventCountingAsInteraction() {
            event.SetPreventCountingAsInteractionTrue();
        }
        event
    }

    // Source: third_party/blink/common/input/web_pointer_event.cc:64-80
    pub fn FromMouse(event_type: Type, mouse_event: &WebMouseEvent) -> Self {
        let mut event = Self {
            event: WebInputEvent::new(
                event_type,
                Type::kPointerTypeFirst,
                Type::kPointerTypeLast,
                mouse_event.GetModifiers(),
                mouse_event.TimeStamp(),
            ),
            pointer_properties: mouse_event.pointer_properties.clone(),
            hovering: true,
            width: f32::NAN,
            height: f32::NAN,
            rotation_angle: 0.0,
            unique_touch_event_id: 0,
            dispatch_type: DispatchType::kBlocking,
            moved_beyond_slop_region: false,
            touch_start_or_first_touch_move: false,
        };
        event.SetFrameScale(mouse_event.FrameScale());
        event.SetFrameTranslate(mouse_event.FrameTranslate());
        if mouse_event.GetPreventCountingAsInteraction() {
            event.SetPreventCountingAsInteractionTrue();
        }
        event
    }
}

impl WebInputEventPayload for WebPointerEvent {
    // Source: third_party/blink/public/common/input/web_pointer_event.h:21-22
    fn Event(&self) -> &WebInputEvent {
        &self.event
    }

    fn EventMut(&mut self) -> &mut WebInputEvent {
        &mut self.event
    }

    fn AsAny(&self) -> &dyn Any {
        self
    }

    fn AsAnyMut(&mut self) -> &mut dyn Any {
        self
    }

    // Source: third_party/blink/common/input/web_pointer_event.cc:82-84
    fn Clone(&self) -> Box<dyn WebInputEventPayload> {
        Box::new(std::clone::Clone::clone(self))
    }

    // Source: third_party/blink/common/input/web_pointer_event.cc:86-96
    fn CanCoalesce(&self, event: &dyn WebInputEventPayload) -> bool {
        if !WebInputEvent::IsPointerEventType(event.Event().GetType()) {
            return false;
        }
        let pointer_event = event
            .AsAny()
            .downcast_ref::<Self>()
            .expect("pointer event type must have a WebPointerEvent payload");
        (self.GetType() == Type::kPointerMove || self.GetType() == Type::kPointerRawUpdate)
            && self.GetType() == event.Event().GetType()
            && self.GetModifiers() == event.Event().GetModifiers()
            && self.pointer_properties.id == pointer_event.pointer_properties.id
            && self.pointer_properties.pointer_type == pointer_event.pointer_properties.pointer_type
    }

    // Source: third_party/blink/common/input/web_pointer_event.cc:98-108
    fn Coalesce(&mut self, event: &dyn WebInputEventPayload) {
        debug_assert!(self.CanCoalesce(event));
        let pointer_event = event
            .AsAny()
            .downcast_ref::<Self>()
            .expect("pointer event type must have a WebPointerEvent payload");
        let x = self.pointer_properties.movement_x;
        let y = self.pointer_properties.movement_y;
        *self = std::clone::Clone::clone(pointer_event);
        self.pointer_properties.movement_x += x;
        self.pointer_properties.movement_y += y;
    }
}

impl WebPointerEvent {
    // Source: third_party/blink/common/input/web_pointer_event.cc:110-118
    pub fn CreatePointerCausesUaActionEvent(
        pointer_type: PointerType,
        time_stamp: TimeTicks,
    ) -> Self {
        let mut event = Self::default();
        event.pointer_properties.pointer_type = pointer_type;
        event.SetTimeStamp(time_stamp);
        event.SetType(Type::kPointerCausedUaAction);
        event
    }

    // Source: third_party/blink/common/input/web_pointer_event.cc:120-130
    pub fn WebPointerEventInRootFrame(&self) -> Self {
        let mut transformed_event = std::clone::Clone::clone(self);
        if self.HasWidth() {
            transformed_event.width /= self.FrameScale();
        }
        if self.HasHeight() {
            transformed_event.height /= self.FrameScale();
        }
        let mut position = *transformed_event.pointer_properties.PositionInWidget();
        position.ScaleUniform(1.0 / self.FrameScale());
        transformed_event
            .pointer_properties
            .SetPositionInWidget(position + self.FrameTranslate());
        transformed_event
    }
}
