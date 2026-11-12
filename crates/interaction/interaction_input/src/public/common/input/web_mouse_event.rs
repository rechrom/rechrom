// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/common/input/web_mouse_event.h:1-130
// Source: third_party/blink/common/input/web_mouse_event.cc:1-140
// Rust modules replace C++ includes, namespaces and export annotations.

use std::any::Any;
use std::ops::{Deref, DerefMut};

use crate::base::time::time::TimeTicks;
use foundation::gfx_geometry::{PointF, Vector2dF};

use super::pointer_id::PointerId;
use super::web_input_event::{Type, WebInputEvent, WebInputEventPayload};
use super::web_pointer_properties::{Button, PointerType, WebPointerProperties};
pub use crate::ui::base::mojom::menu_source_type::MenuSourceType;

// Source: third_party/blink/public/common/input/web_mouse_event.h:16,53-59
// Source: third_party/blink/common/input/web_mouse_event.cc:27-31
// This borrowed projection is the only gesture state consumed by the source
// constructor. The concrete WebGestureEvent supplies it when translated.
pub trait WebGestureEventView {
    fn PositionInWidget(&self) -> PointF;
    fn PositionInScreen(&self) -> PointF;
    fn FrameScale(&self) -> f32;
    fn FrameTranslate(&self) -> Vector2dF;
    fn GetType(&self) -> Type;
}

// Source: third_party/blink/public/common/input/web_mouse_event.h:20-28
#[derive(Clone, Debug)]
pub struct WebMouseEvent {
    pub event: WebInputEvent,
    pub pointer_properties: WebPointerProperties,
    pub click_count: i32,
    pub menu_source_type: MenuSourceType,
}

impl WebMouseEvent {
    // Source: third_party/blink/public/common/input/web_mouse_event.h:23
    pub const kMousePointerId: PointerId = i32::MAX;

    // Source: third_party/blink/public/common/input/web_mouse_event.h:30-51
    pub fn new(
        event_type: Type,
        position: PointF,
        global_position: PointF,
        button: Button,
        click_count: i32,
        modifiers: i32,
        time_stamp: TimeTicks,
        menu_source_type: MenuSourceType,
        id: PointerId,
    ) -> Self {
        Self {
            event: WebInputEvent::new(
                event_type,
                Type::kMouseTypeFirst,
                Type::kMouseTypeLast,
                modifiers,
                time_stamp,
            ),
            pointer_properties: WebPointerProperties::new(
                id,
                PointerType::kMouse,
                button,
                position,
                global_position,
                0,
                0,
            ),
            click_count,
            menu_source_type,
        }
    }

    // Source: third_party/blink/public/common/input/web_mouse_event.h:53-59
    // Source: third_party/blink/common/input/web_mouse_event.cc:11-32
    pub fn FromGesture(
        event_type: Type,
        gesture_event: &impl WebGestureEventView,
        button: Button,
        click_count: i32,
        modifiers: i32,
        time_stamp: TimeTicks,
        id: PointerId,
    ) -> Self {
        let mut event = Self::WithTypeRange(
            event_type,
            Type::kMouseTypeFirst,
            Type::kMouseTypeLast,
            modifiers,
            time_stamp,
            id,
        );
        event.pointer_properties.button = button;
        event.click_count = click_count;
        event
            .pointer_properties
            .SetPositionInWidget(gesture_event.PositionInWidget());
        event
            .pointer_properties
            .SetPositionInScreen(gesture_event.PositionInScreen());
        event.SetFrameScale(gesture_event.FrameScale());
        event.SetFrameTranslate(gesture_event.FrameTranslate());
        event.SetMenuSourceType(gesture_event.GetType());
        event
    }

    // Source: third_party/blink/public/common/input/web_mouse_event.h:61-72
    pub fn WithTypeRange(
        event_type: Type,
        type_range_min: Type,
        type_range_max: Type,
        modifiers: i32,
        time_stamp: TimeTicks,
        id: PointerId,
    ) -> Self {
        let mut pointer_properties = WebPointerProperties::WithId(id);
        pointer_properties.pointer_type = PointerType::kMouse;
        Self {
            event: WebInputEvent::new(
                event_type,
                type_range_min,
                type_range_max,
                modifiers,
                time_stamp,
            ),
            pointer_properties,
            click_count: 0,
            menu_source_type: MenuSourceType::kNone,
        }
    }

    // Source: third_party/blink/public/common/input/web_mouse_event.h:74-83
    pub fn WithType(
        event_type: Type,
        modifiers: i32,
        time_stamp: TimeTicks,
        id: PointerId,
    ) -> Self {
        Self::WithTypeRange(
            event_type,
            Type::kMouseTypeFirst,
            Type::kMouseTypeLast,
            modifiers,
            time_stamp,
            id,
        )
    }

    // Source: third_party/blink/public/common/input/web_mouse_event.h:91-93
    pub fn FromTouch(&self) -> bool {
        self.GetModifiers() & WebInputEvent::kIsCompatibilityEventForTouch != 0
    }

    // Source: third_party/blink/public/common/input/web_mouse_event.h:95
    pub fn ClickCount(&self) -> i32 {
        self.click_count
    }

    // Source: third_party/blink/public/common/input/web_mouse_event.h:97-99
    pub fn GetMenuSourceType(&self) -> MenuSourceType {
        self.menu_source_type
    }

    // Source: third_party/blink/public/common/input/web_mouse_event.h:105
    // Source: third_party/blink/common/input/web_mouse_event.cc:34-37
    pub fn PositionInRootFrame(&self) -> PointF {
        let mut position = *self.pointer_properties.PositionInWidget();
        position.ScaleUniform(1.0 / self.FrameScale());
        position + self.FrameTranslate()
    }

    // Source: third_party/blink/public/common/input/web_mouse_event.h:107-109
    // Source: third_party/blink/common/input/web_mouse_event.cc:67-71
    pub fn FlattenTransform(&self) -> Self {
        let mut result = std::clone::Clone::clone(self);
        result.FlattenTransformSelf();
        result
    }

    // Source: third_party/blink/public/common/input/web_mouse_event.h:111-119
    // Source: third_party/blink/common/input/web_mouse_event.cc:98-138
    pub fn UpdateEventModifiersToMatchButton(&mut self) {
        let button_modifier_bit = match self.pointer_properties.button {
            Button::kNoButton => WebInputEvent::kNoModifiers,
            Button::kLeft => WebInputEvent::kLeftButtonDown,
            Button::kMiddle => WebInputEvent::kMiddleButtonDown,
            Button::kRight => WebInputEvent::kRightButtonDown,
            Button::kBack => WebInputEvent::kBackButtonDown,
            Button::kForward => WebInputEvent::kForwardButtonDown,
            // The source has no modifier bit for the stylus eraser.
            Button::kEraser => WebInputEvent::kNoModifiers,
        };
        if self.GetType() == Type::kMouseDown {
            self.event
                .SetModifiers(self.GetModifiers() | button_modifier_bit);
        } else if self.GetType() == Type::kMouseUp {
            self.event
                .SetModifiers(self.GetModifiers() & !button_modifier_bit);
        }
    }

    // Source: third_party/blink/public/common/input/web_mouse_event.h:121-122
    // Source: third_party/blink/common/input/web_mouse_event.cc:73-77
    pub(crate) fn FlattenTransformSelf(&mut self) {
        self.pointer_properties
            .SetPositionInWidget(self.PositionInRootFrame());
        self.SetFrameTranslate(Vector2dF::default());
        self.SetFrameScale(1.0);
    }

    // Source: third_party/blink/public/common/input/web_mouse_event.h:124-125
    // Source: third_party/blink/common/input/web_mouse_event.cc:79-96
    fn SetMenuSourceType(&mut self, event_type: Type) {
        self.menu_source_type = match event_type {
            Type::kGestureShortPress
            | Type::kGestureTapDown
            | Type::kGestureTap
            | Type::kGestureDoubleTap => MenuSourceType::kTouch,
            Type::kGestureLongPress => MenuSourceType::kLongPress,
            Type::kGestureLongTap => MenuSourceType::kLongTap,
            _ => MenuSourceType::kNone,
        };
    }
}

// Source: third_party/blink/public/common/input/web_mouse_event.h:85-89
impl Default for WebMouseEvent {
    fn default() -> Self {
        Self::WithType(
            Type::kUndefined,
            WebInputEvent::kNoModifiers,
            TimeTicks::new(),
            Self::kMousePointerId,
        )
    }
}

// Source: third_party/blink/public/common/input/web_mouse_event.h:20-21
// Composition and base dereferencing express the source inheritance.
impl Deref for WebMouseEvent {
    type Target = WebInputEvent;
    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl DerefMut for WebMouseEvent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event
    }
}

impl WebInputEventPayload for WebMouseEvent {
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

    // Source: third_party/blink/public/common/input/web_mouse_event.h:101
    // Source: third_party/blink/common/input/web_mouse_event.cc:39-41
    fn Clone(&self) -> Box<dyn WebInputEventPayload> {
        Box::new(std::clone::Clone::clone(self))
    }

    // Source: third_party/blink/public/common/input/web_mouse_event.h:102
    // Source: third_party/blink/common/input/web_mouse_event.cc:43-54
    fn CanCoalesce(&self, event: &dyn WebInputEventPayload) -> bool {
        if !WebInputEvent::IsMouseEventType(event.Event().GetType()) {
            return false;
        }
        let mouse_event = event
            .AsAny()
            .downcast_ref::<Self>()
            .expect("mouse event type must have a WebMouseEvent payload");
        self.GetType() == Type::kMouseMove
            && self.GetType() == mouse_event.GetType()
            && self.GetModifiers() == mouse_event.GetModifiers()
            && self.pointer_properties.id == mouse_event.pointer_properties.id
            && self.pointer_properties.pointer_type == mouse_event.pointer_properties.pointer_type
    }

    // Source: third_party/blink/public/common/input/web_mouse_event.h:103
    // Source: third_party/blink/common/input/web_mouse_event.cc:56-65
    fn Coalesce(&mut self, event: &dyn WebInputEventPayload) {
        debug_assert!(self.CanCoalesce(event));
        let mouse_event = event
            .AsAny()
            .downcast_ref::<Self>()
            .expect("mouse event type must have a WebMouseEvent payload");
        let x = self.pointer_properties.movement_x;
        let y = self.pointer_properties.movement_y;
        *self = std::clone::Clone::clone(mouse_event);
        self.pointer_properties.movement_x += x;
        self.pointer_properties.movement_y += y;
    }
}
