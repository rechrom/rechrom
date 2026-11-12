/*
 * Copyright (C) 2009 Google Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 *     * Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above
 * copyright notice, this list of conditions and the following disclaimer
 * in the documentation and/or other materials provided with the
 * distribution.
 *     * Neither the name of Google Inc. nor the names of its
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/events/web_input_event_conversion.h:1-97
// Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:1-318
// Source .h:31-54,95-97 and .cc:31-56,107,318: Rust modules replace include
// guards, includes, export annotations and namespaces. Builders embed the same
// concrete input values instead of inheriting them.
//
// The narrow borrowed ports below require the actual TouchEvent,
// WebGestureEvent, LocalFrameView and LayoutObject source values/operations;
// they do not synthesize missing host state. Gesture payloads remain in their
// original owner and are cloned whole.
// There is no drag conversion in this Chromium source pair.

use std::ops::{Deref, DerefMut, Sub};

use interaction_dispatch::core::events::keyboard_event::KeyboardEvent;
use interaction_dispatch::core::events::mouse_event::MouseEvent;
use interaction_input::base::time::time::TimeTicks;
use interaction_input::public::common::input::web_input_event::{
    Type, WebInputEvent, WebInputEventPayload,
};
use interaction_input::public::common::input::web_keyboard_event::WebKeyboardEvent;
use interaction_input::public::common::input::web_mouse_event::WebMouseEvent;
use interaction_input::public::common::input::web_mouse_wheel_event::WebMouseWheelEvent;
use interaction_input::public::common::input::web_pointer_event::WebPointerEvent;
use interaction_input::public::common::input::web_pointer_properties::{
    Button, PointF, PointerType,
};

// This is the existing gfx::Vector2dF type, obtained through PointF's public
// subtraction contract; no second geometry representation is introduced.
pub type Vector2dF = <PointF as Sub<PointF>>::Output;

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.h:45-51
// Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:60-76
// LocalFrameRootView projects GetFrame().LocalFrameRoot().View(). Only the root
// view's scale, visible-rect origin and ChromeClient overscroll cross this port.
pub trait LocalFrameView {
    fn LocalFrameRootView(&self) -> Option<&dyn LocalFrameView>;
    fn InputEventsScaleFactor(&self) -> f32;
    fn VisualViewportVisibleRectOrigin(&self) -> PointF;
    fn ElasticOverscroll(&self) -> Vector2dF;
}

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:89-91,265-267
pub trait InputEventLayoutObject {
    fn AbsoluteToLocalPoint(&self, point: PointF) -> PointF;
}

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:225-238,249-257,265-266
// Source-shaped readonly Touch projections for the otherwise unported DOM
// TouchEvent/TouchList surface. None retains or duplicates a document tree.
pub struct Touch {
    pub identifier: i32,
    pub screen_location: PointF,
    pub absolute_location: PointF,
}

pub trait TouchEvent {
    fn touches(&self) -> Option<&[Touch]>;
    fn changedTouches(&self) -> Option<&[Touch]>;
    fn r#type(&self) -> &str;
    fn PlatformTimeStamp(&self) -> TimeTicks;
    fn GetModifiers(&self) -> i32;
}

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:137-142
// The concrete gesture owner supplies the base access and full Clone. No
// gesture union, payload or coalescing algorithm is reconstructed here.
pub trait WebGestureEvent: Clone {
    fn InputEventMut(&mut self) -> &mut WebInputEvent;
}

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.h:56-64
// Unsupported DOM event names retain the base default's kUndefined type.
pub struct WebMouseEventBuilder(pub WebMouseEvent);

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.h:66-73
pub struct WebKeyboardEventBuilder(pub WebKeyboardEvent);

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.h:75-93
// Public transform declarations are implemented in the source .cc order below.

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:57-65
fn FrameScale(frame_view: Option<&dyn LocalFrameView>) -> f32 {
    let mut scale = 1.0;
    if let Some(frame_view) = frame_view {
        if let Some(root_view) = frame_view.LocalFrameRootView() {
            scale = root_view.InputEventsScaleFactor();
        }
    }
    scale
}

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:67-80
fn FrameTranslation(frame_view: Option<&dyn LocalFrameView>) -> Vector2dF {
    let mut visual_viewport = PointF::default();
    let mut overscroll_offset = Vector2dF::default();
    if let Some(frame_view) = frame_view {
        if let Some(root_view) = frame_view.LocalFrameRootView() {
            let origin = root_view.VisualViewportVisibleRectOrigin();
            // gfx::ToFlooredPoint uses ClampFloor<int> before conversion back
            // to the vector's float components (including saturated bounds).
            visual_viewport = PointF::new(
                origin.x().floor() as i32 as f32,
                origin.y().floor() as i32 as f32,
            );
            overscroll_offset = root_view.ElasticOverscroll();
        }
    }
    visual_viewport.OffsetFromOrigin() + overscroll_offset
}

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:82-92
fn UpdateWebMouseEventFromCoreMouseEvent(
    event: &MouseEvent,
    layout_object: &dyn InputEventLayoutObject,
    web_event: &mut WebMouseEvent,
) {
    web_event.SetTimeStamp(event.PlatformTimeStamp());
    web_event.SetModifiers(event.GetModifiers());
    web_event
        .pointer_properties
        .SetPositionInScreenXY(event.screenX() as f32, event.screenY() as f32);
    let local_point = layout_object.AbsoluteToLocalPoint(*event.AbsoluteLocation());
    web_event
        .pointer_properties
        .SetPositionInWidget(local_point);
}

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:94-105
// Rust has no free-function overloading; the private float/vector overload has
// a distinct name while retaining its source arguments and assertion order.
fn TransformWebPointerEventWithTransform(
    frame_scale: f32,
    frame_translate: Vector2dF,
    event: &WebPointerEvent,
) -> WebPointerEvent {
    debug_assert_eq!(1.0, event.FrameScale());
    debug_assert_eq!(0.0, event.FrameTranslate().x());
    debug_assert_eq!(0.0, event.FrameTranslate().y());
    let mut result = event.clone();
    result.SetFrameScale(frame_scale);
    result.SetFrameTranslate(frame_translate);
    result
}

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.h:79-80
// Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:109-126
pub fn TransformWebMouseEvent(
    frame_view: Option<&dyn LocalFrameView>,
    event: &WebMouseEvent,
) -> WebMouseEvent {
    let mut result = event.clone();
    // Production fixes these modifiers upstream; injected events also need
    // the source's repair here before the frame transform is attached.
    if matches!(event.GetType(), Type::kMouseUp | Type::kMouseDown) {
        result.UpdateEventModifiersToMatchButton();
    }
    result.SetFrameScale(FrameScale(frame_view));
    result.SetFrameTranslate(FrameTranslation(frame_view));
    result
}

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.h:82-83
// Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:128-135
pub fn TransformWebMouseWheelEvent(
    frame_view: Option<&dyn LocalFrameView>,
    event: &WebMouseWheelEvent,
) -> WebMouseWheelEvent {
    let mut result = event.clone();
    result.SetFrameScale(FrameScale(frame_view));
    result.SetFrameTranslate(FrameTranslation(frame_view));
    result
}

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.h:75-78
// Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:137-143
pub fn TransformWebGestureEvent<G: WebGestureEvent>(
    frame_view: Option<&dyn LocalFrameView>,
    event: &G,
) -> G {
    let mut result = event.clone();
    result.InputEventMut().SetFrameScale(FrameScale(frame_view));
    result
        .InputEventMut()
        .SetFrameTranslate(FrameTranslation(frame_view));
    result
}

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.h:85-86
// Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:145-149
pub fn TransformWebPointerEvent(
    frame_view: Option<&dyn LocalFrameView>,
    event: &WebPointerEvent,
) -> WebPointerEvent {
    TransformWebPointerEventWithTransform(
        FrameScale(frame_view),
        FrameTranslation(frame_view),
        event,
    )
}

impl WebMouseEventBuilder {
    // Source: third_party/blink/renderer/core/events/web_input_event_conversion.h:58-62
    // Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:151-219
    pub fn new(layout_object: &dyn InputEventLayoutObject, event: &MouseEvent) -> Self {
        let mut result = WebMouseEvent::default();
        let event_type = match event.r#type() {
            "mousemove" => Type::kMouseMove,
            "mouseout" => Type::kMouseLeave,
            "mouseover" => Type::kMouseEnter,
            "mousedown" => Type::kMouseDown,
            "mouseup" => Type::kMouseUp,
            "contextmenu" => Type::kContextMenu,
            _ => return Self(result),
        };
        result.SetType(event_type);
        result.SetTimeStamp(event.PlatformTimeStamp());
        result.SetModifiers(event.GetModifiers());
        UpdateWebMouseEventFromCoreMouseEvent(event, layout_object, &mut result);

        match event.button() {
            value if value == Button::kLeft as i16 => {
                result.pointer_properties.button = Button::kLeft;
            }
            value if value == Button::kMiddle as i16 => {
                result.pointer_properties.button = Button::kMiddle;
            }
            value if value == Button::kRight as i16 => {
                result.pointer_properties.button = Button::kRight;
            }
            value if value == Button::kBack as i16 => {
                result.pointer_properties.button = Button::kBack;
            }
            value if value == Button::kForward as i16 => {
                result.pointer_properties.button = Button::kForward;
            }
            _ => {}
        }
        if event.ButtonDown() {
            match event.button() {
                value if value == Button::kLeft as i16 => {
                    result
                        .event
                        .SetModifiers(result.event.GetModifiers() | WebInputEvent::kLeftButtonDown);
                }
                value if value == Button::kMiddle as i16 => {
                    result.event.SetModifiers(
                        result.event.GetModifiers() | WebInputEvent::kMiddleButtonDown,
                    );
                }
                value if value == Button::kRight as i16 => {
                    result.event.SetModifiers(
                        result.event.GetModifiers() | WebInputEvent::kRightButtonDown,
                    );
                }
                value if value == Button::kBack as i16 => {
                    result
                        .event
                        .SetModifiers(result.event.GetModifiers() | WebInputEvent::kBackButtonDown);
                }
                value if value == Button::kForward as i16 => {
                    result.event.SetModifiers(
                        result.event.GetModifiers() | WebInputEvent::kForwardButtonDown,
                    );
                }
                _ => {}
            }
        } else {
            result.pointer_properties.button = Button::kNoButton;
        }
        result.pointer_properties.movement_x = event.movementX();
        result.pointer_properties.movement_y = event.movementY();
        result.click_count = event.detail();
        result.pointer_properties.pointer_type = PointerType::kMouse;
        Self(result)
    }

    // Source: third_party/blink/renderer/core/events/web_input_event_conversion.h:63
    // Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:221-270
    // Synthetic mouse input for plugin touch emulation, preserving identifier
    // zero, single-touch and changedTouches-on-touchend guards exactly.
    pub fn FromTouch(layout_object: &dyn InputEventLayoutObject, event: &dyn TouchEvent) -> Self {
        let mut result = WebMouseEvent::default();
        let Some(touches) = event.touches() else {
            return Self(result);
        };
        if touches.len() != 1
            && (!touches.is_empty()
                || event.r#type() != "touchend"
                || !event
                    .changedTouches()
                    .is_some_and(|touches| touches.len() == 1))
        {
            return Self(result);
        }
        let touch = if touches.len() == 1 {
            &touches[0]
        } else {
            &event.changedTouches().expect("checked touchend list")[0]
        };
        if touch.identifier != 0 {
            return Self(result);
        }
        let event_type = match event.r#type() {
            "touchstart" => Type::kMouseDown,
            "touchmove" => Type::kMouseMove,
            "touchend" => Type::kMouseUp,
            _ => return Self(result),
        };
        result.SetType(event_type);
        result.SetTimeStamp(event.PlatformTimeStamp());
        result.SetModifiers(event.GetModifiers());
        result.SetFrameScale(1.0);
        result.SetFrameTranslate(Vector2dF::default());
        result
            .pointer_properties
            .SetPositionInScreen(touch.screen_location);
        result.pointer_properties.button = Button::kLeft;
        // Preserve cc:260-261: the left-button modifier remains set on up.
        result
            .event
            .SetModifiers(result.event.GetModifiers() | WebInputEvent::kLeftButtonDown);
        result.click_count = i32::from(matches!(event_type, Type::kMouseDown | Type::kMouseUp));
        let local_point = layout_object.AbsoluteToLocalPoint(touch.absolute_location);
        result.pointer_properties.SetPositionInWidget(local_point);
        result.pointer_properties.pointer_type = PointerType::kTouch;
        Self(result)
    }
}

impl WebKeyboardEventBuilder {
    // Source: third_party/blink/renderer/core/events/web_input_event_conversion.h:70-73
    // Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:272-290
    pub fn new(event: &KeyboardEvent) -> Self {
        if let Some(web_event) = event.KeyEvent() {
            return Self(web_event.clone());
        }
        let mut result = WebKeyboardEvent::default();
        let event_type = match event.r#type() {
            "keydown" => Type::kKeyDown,
            "keyup" => Type::kKeyUp,
            "keypress" => Type::kChar,
            _ => return Self(result),
        };
        result.SetType(event_type);
        result.SetModifiers(event.GetModifiers());
        result.SetTimeStamp(event.PlatformTimeStamp());
        result.windows_key_code = event.keyCode();
        Self(result)
    }
}

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.h:88-90
// Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:292-302
// The same function accepts either GetCoalescedEventsPointers() or
// GetPredictedEventsPointers(): preserve order, timestamps and full payloads;
// no coalescing, prediction or synthetic samples occur during conversion.
pub fn TransformWebMouseEventVector(
    frame_view: Option<&dyn LocalFrameView>,
    coalesced_events: &[Box<dyn WebInputEventPayload>],
) -> Vec<WebMouseEvent> {
    let mut result = Vec::new();
    for event in coalesced_events {
        debug_assert!(WebInputEvent::IsMouseEventType(event.Event().GetType()));
        result.push(TransformWebMouseEvent(
            frame_view,
            event
                .AsAny()
                .downcast_ref::<WebMouseEvent>()
                .expect("source requires a WebMouseEvent payload"),
        ));
    }
    result
}

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.h:91-93
// Source: third_party/blink/renderer/core/events/web_input_event_conversion.cc:304-316
pub fn TransformWebPointerEventVector(
    frame_view: Option<&dyn LocalFrameView>,
    coalesced_events: &[Box<dyn WebInputEventPayload>],
) -> Vec<WebPointerEvent> {
    let scale = FrameScale(frame_view);
    let translation = FrameTranslation(frame_view);
    let mut result = Vec::new();
    for event in coalesced_events {
        debug_assert!(WebInputEvent::IsPointerEventType(event.Event().GetType()));
        result.push(TransformWebPointerEventWithTransform(
            scale,
            translation,
            event
                .AsAny()
                .downcast_ref::<WebPointerEvent>()
                .expect("source requires a WebPointerEvent payload"),
        ));
    }
    result
}

// Source: third_party/blink/renderer/core/events/web_input_event_conversion.h:56,70
// Rust inheritance projection; callers may also move the concrete value out
// through the public tuple field, without copying an event into another model.
impl Deref for WebMouseEventBuilder {
    type Target = WebMouseEvent;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WebMouseEventBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for WebKeyboardEventBuilder {
    type Target = WebKeyboardEvent;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WebKeyboardEventBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
