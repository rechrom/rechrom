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
// Copyright 2020 The Chromium Authors
// Use of the translated .cc code is governed by a BSD-style license that can
// be found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/common/input/web_input_event.h:1-455
// Source: third_party/blink/common/input/web_input_event.cc:1-112
// Source .h:31-52,453-455 and .cc:5-8,112: Rust modules replace C++ includes,
// include guards, export annotations, and namespaces.

use std::any::Any;

use crate::base::time::time::TimeTicks;
use crate::ui::events::event_latency_metadata::EventLatencyMetadata;
use crate::ui::events::types::event_type::EventType as UiEventType;
use foundation::gfx_geometry::Vector2dF;

// Source: third_party/blink/public/common/input/web_input_event.h:54-78
// KeyDown combines RawKeyDown and Char; the browser supplies input-method
// events asynchronously, preserving the shared Windows/Linux/macOS event model.
pub use crate::public::mojom::input::input_event::EventType as Type;

// Source: third_party/blink/public/common/input/web_input_event.h:80-168
// These numeric values are also the Pepper modifier values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Modifiers {
    kShiftKey = 1 << 0,
    kControlKey = 1 << 1,
    kAltKey = 1 << 2,
    kMetaKey = 1 << 3,
    kIsKeyPad = 1 << 4,
    kIsAutoRepeat = 1 << 5,
    kLeftButtonDown = 1 << 6,
    kMiddleButtonDown = 1 << 7,
    kRightButtonDown = 1 << 8,
    kCapsLockOn = 1 << 9,
    kNumLockOn = 1 << 10,
    kIsLeft = 1 << 11,
    kIsRight = 1 << 12,
    kIsTouchAccessibility = 1 << 13,
    kIsComposing = 1 << 14,
    kAltGrKey = 1 << 15,
    kFnKey = 1 << 16,
    kSymbolKey = 1 << 17,
    kScrollLockOn = 1 << 18,
    kIsCompatibilityEventForTouch = 1 << 19,
    kBackButtonDown = 1 << 20,
    kForwardButtonDown = 1 << 21,
    kRelativeMotionEvent = 1 << 22,
    kFromDebugger = 1 << 23,
    kTargetFrameMovedRecently = 1 << 24,
    kTargetFrameMovedRecentlyForIOv2 = 1 << 25,
    kScrollbarManipulationHandledOnCompositorThread = 1 << 26,
    kKeyModifiers = (1 << 17) | (1 << 16) | (1 << 15) | (1 << 3) | (1 << 2) | (1 << 1) | (1 << 0),
    kNoModifiers = 0,
}

// Source: third_party/blink/public/common/input/web_input_event.h:170-172
pub use crate::public::mojom::input::input_event::{DispatchType, RailsMode};

// Source: third_party/blink/public/common/input/web_input_event.h:422-450
// The source abstract base's common fields are embedded by concrete Rust
// event payloads. The virtual interface is WebInputEventPayload below.
#[derive(Clone, Debug)]
pub struct WebInputEvent {
    frame_scale_: f32,
    frame_translate_: Vector2dF,
    time_stamp_: TimeTicks,
    queued_time_stamp_: TimeTicks,
    type_: Type,
    type_range_min_: Type,
    type_range_max_: Type,
    modifiers_: i32,
    event_latency_metadata_: EventLatencyMetadata,
    prevent_counting_as_interaction_: bool,
}

impl WebInputEvent {
    // Source: third_party/blink/public/common/input/web_input_event.h:87-168
    // Associated constants preserve C++'s WebInputEvent::kModifier spelling.
    pub const kShiftKey: i32 = Modifiers::kShiftKey as i32;
    pub const kControlKey: i32 = Modifiers::kControlKey as i32;
    pub const kAltKey: i32 = Modifiers::kAltKey as i32;
    pub const kMetaKey: i32 = Modifiers::kMetaKey as i32;
    pub const kIsKeyPad: i32 = Modifiers::kIsKeyPad as i32;
    pub const kIsAutoRepeat: i32 = Modifiers::kIsAutoRepeat as i32;
    pub const kLeftButtonDown: i32 = Modifiers::kLeftButtonDown as i32;
    pub const kMiddleButtonDown: i32 = Modifiers::kMiddleButtonDown as i32;
    pub const kRightButtonDown: i32 = Modifiers::kRightButtonDown as i32;
    pub const kCapsLockOn: i32 = Modifiers::kCapsLockOn as i32;
    pub const kNumLockOn: i32 = Modifiers::kNumLockOn as i32;
    pub const kIsLeft: i32 = Modifiers::kIsLeft as i32;
    pub const kIsRight: i32 = Modifiers::kIsRight as i32;
    pub const kIsTouchAccessibility: i32 = Modifiers::kIsTouchAccessibility as i32;
    pub const kIsComposing: i32 = Modifiers::kIsComposing as i32;
    pub const kAltGrKey: i32 = Modifiers::kAltGrKey as i32;
    pub const kFnKey: i32 = Modifiers::kFnKey as i32;
    pub const kSymbolKey: i32 = Modifiers::kSymbolKey as i32;
    pub const kScrollLockOn: i32 = Modifiers::kScrollLockOn as i32;
    pub const kIsCompatibilityEventForTouch: i32 = Modifiers::kIsCompatibilityEventForTouch as i32;
    pub const kBackButtonDown: i32 = Modifiers::kBackButtonDown as i32;
    pub const kForwardButtonDown: i32 = Modifiers::kForwardButtonDown as i32;
    pub const kRelativeMotionEvent: i32 = Modifiers::kRelativeMotionEvent as i32;
    pub const kFromDebugger: i32 = Modifiers::kFromDebugger as i32;
    pub const kTargetFrameMovedRecently: i32 = Modifiers::kTargetFrameMovedRecently as i32;
    pub const kTargetFrameMovedRecentlyForIOv2: i32 =
        Modifiers::kTargetFrameMovedRecentlyForIOv2 as i32;
    pub const kScrollbarManipulationHandledOnCompositorThread: i32 =
        Modifiers::kScrollbarManipulationHandledOnCompositorThread as i32;
    pub const kKeyModifiers: i32 = Modifiers::kKeyModifiers as i32;
    pub const kNoModifiers: i32 = Modifiers::kNoModifiers as i32;

    // Source: third_party/blink/public/common/input/web_input_event.h:174-175
    pub const kInputModifiers: i32 =
        Self::kShiftKey | Self::kControlKey | Self::kAltKey | Self::kMetaKey;

    // Source: third_party/blink/public/common/input/web_input_event.h:177-187
    pub const fn GetStaticTimeStampForTests() -> TimeTicks {
        // Keep the timestamp far enough from the null origin for delta gates.
        TimeTicks::FromInternalValue(123_000_000)
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:189-192
    pub fn IsMouseEventType(event_type: Type) -> bool {
        Type::kMouseTypeFirst <= event_type && event_type <= Type::kMouseTypeLast
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:194-197
    pub fn IsKeyboardEventType(event_type: Type) -> bool {
        Type::kKeyboardTypeFirst <= event_type && event_type <= Type::kKeyboardTypeLast
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:199-202
    pub fn IsTouchEventType(event_type: Type) -> bool {
        Type::kTouchTypeFirst <= event_type && event_type <= Type::kTouchTypeLast
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:204-207
    pub fn IsGestureEventType(event_type: Type) -> bool {
        Type::kGestureTypeFirst <= event_type && event_type <= Type::kGestureTypeLast
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:209-212
    pub fn IsPointerEventType(event_type: Type) -> bool {
        Type::kPointerTypeFirst <= event_type && event_type <= Type::kPointerTypeLast
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:214-241
    pub fn IsWebInteractionEvent(event_type: Type) -> bool {
        matches!(
            event_type,
            Type::kMouseDown
                | Type::kMouseUp
                | Type::kKeyDown
                | Type::kRawKeyDown
                | Type::kKeyUp
                | Type::kChar
                | Type::kGestureTapDown
                | Type::kGestureTap
                | Type::kPointerDown
                | Type::kPointerUp
                | Type::kTouchStart
                | Type::kTouchEnd
        )
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:243-255
    pub fn IsSameEventClass(&self, other: &Self) -> bool {
        if Self::IsMouseEventType(self.type_) {
            return Self::IsMouseEventType(other.type_);
        }
        if Self::IsGestureEventType(self.type_) {
            return Self::IsGestureEventType(other.type_);
        }
        if Self::IsTouchEventType(self.type_) {
            return Self::IsTouchEventType(other.type_);
        }
        if Self::IsKeyboardEventType(self.type_) {
            return Self::IsKeyboardEventType(other.type_);
        }
        if Self::IsPointerEventType(self.type_) {
            return Self::IsPointerEventType(other.type_);
        }
        self.type_ == other.type_
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:257-266
    pub fn IsGestureScroll(&self) -> bool {
        matches!(
            self.type_,
            Type::kGestureScrollBegin | Type::kGestureScrollUpdate | Type::kGestureScrollEnd
        )
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:268-272
    pub fn IsPinchGestureEventType(event_type: Type) -> bool {
        Type::kGesturePinchTypeFirst <= event_type && event_type <= Type::kGesturePinchTypeLast
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:274-278
    pub fn IsFlingGestureEventType(event_type: Type) -> bool {
        Type::kGestureFlingStart <= event_type && event_type <= Type::kGestureFlingCancel
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:280-331
    pub fn GetName(event_type: Type) -> &'static str {
        match event_type {
            Type::kUndefined => "Undefined",
            Type::kMouseDown => "MouseDown",
            Type::kMouseUp => "MouseUp",
            Type::kMouseMove => "MouseMove",
            Type::kMouseEnter => "MouseEnter",
            Type::kMouseLeave => "MouseLeave",
            Type::kContextMenu => "ContextMenu",
            Type::kMouseWheel => "MouseWheel",
            Type::kRawKeyDown => "RawKeyDown",
            Type::kKeyDown => "KeyDown",
            Type::kKeyUp => "KeyUp",
            Type::kChar => "Char",
            Type::kGestureScrollBegin => "GestureScrollBegin",
            Type::kGestureScrollEnd => "GestureScrollEnd",
            Type::kGestureScrollUpdate => "GestureScrollUpdate",
            Type::kGestureFlingStart => "GestureFlingStart",
            Type::kGestureFlingCancel => "GestureFlingCancel",
            Type::kGestureShowPress => "GestureShowPress",
            Type::kGestureTap => "GestureTap",
            Type::kGestureTapUnconfirmed => "GestureTapUnconfirmed",
            Type::kGestureTapDown => "GestureTapDown",
            Type::kGestureTapCancel => "GestureTapCancel",
            Type::kGestureDoubleTap => "GestureDoubleTap",
            Type::kGestureTwoFingerTap => "GestureTwoFingerTap",
            Type::kGestureShortPress => "GestureShortPress",
            Type::kGestureLongPress => "GestureLongPress",
            Type::kGestureLongTap => "GestureLongTap",
            Type::kGestureBegin => "GestureBegin",
            Type::kGestureEnd => "GestureEnd",
            Type::kGesturePinchBegin => "GesturePinchBegin",
            Type::kGesturePinchEnd => "GesturePinchEnd",
            Type::kGesturePinchUpdate => "GesturePinchUpdate",
            Type::kTouchStart => "TouchStart",
            Type::kTouchMove => "TouchMove",
            Type::kTouchEnd => "TouchEnd",
            Type::kTouchCancel => "TouchCancel",
            Type::kTouchScrollStarted => "TouchScrollStarted",
            Type::kPointerDown => "PointerDown",
            Type::kPointerUp => "PointerUp",
            Type::kPointerMove => "PointerMove",
            Type::kPointerRawUpdate => "PointerRawUpdate",
            Type::kPointerCancel => "PointerCancel",
            Type::kPointerCausedUaAction => "PointerCausedUaAction",
        }
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:333-334
    pub fn FrameScale(&self) -> f32 {
        self.frame_scale_
    }
    pub fn SetFrameScale(&mut self, scale: f32) {
        self.frame_scale_ = scale;
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:336-339
    pub fn FrameTranslate(&self) -> Vector2dF {
        self.frame_translate_
    }
    pub fn SetFrameTranslate(&mut self, translate: Vector2dF) {
        self.frame_translate_ = translate;
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:341-345
    pub fn GetType(&self) -> Type {
        self.type_
    }
    pub fn SetType(&mut self, event_type: Type) {
        self.type_ = event_type;
        self.CheckEventType();
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:347-348
    pub fn GetModifiers(&self) -> i32 {
        self.modifiers_
    }
    pub fn SetModifiers(&mut self, modifiers: i32) {
        self.modifiers_ = modifiers;
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:350-357
    pub fn TimeStamp(&self) -> TimeTicks {
        self.time_stamp_
    }
    pub fn SetTimeStamp(&mut self, time_stamp: TimeTicks) {
        self.time_stamp_ = time_stamp;
    }
    pub fn QueuedTimeStamp(&self) -> TimeTicks {
        self.queued_time_stamp_
    }
    pub fn SetQueuedTimeStamp(&mut self, time_stamp: TimeTicks) {
        self.queued_time_stamp_ = time_stamp;
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:359-364
    pub fn GetEventLatencyMetadata(&self) -> &EventLatencyMetadata {
        &self.event_latency_metadata_
    }
    pub fn GetModifiableEventLatencyMetadata(&mut self) -> &mut EventLatencyMetadata {
        &mut self.event_latency_metadata_
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:366-368
    pub fn SetTargetFrameMovedRecently(&mut self) {
        self.modifiers_ |= Self::kTargetFrameMovedRecently;
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:370-372
    pub fn SetTargetFrameMovedRecentlyForIOv2(&mut self) {
        self.modifiers_ |= Self::kTargetFrameMovedRecentlyForIOv2;
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:374-376
    pub fn SetScrollbarManipulationHandledOnCompositorThread(&mut self) {
        self.modifiers_ |= Self::kScrollbarManipulationHandledOnCompositorThread;
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:378-387
    // Rust Drop supplies the default virtual destruction. The payload trait
    // below carries Clone/CanCoalesce/Coalesce for concrete event types.

    // Source: third_party/blink/public/common/input/web_input_event.h:389-392
    // GetTypeAsUiEventType is defined in the .cc translation below.

    // Source: third_party/blink/public/common/input/web_input_event.h:394-402
    pub fn GetPreventCountingAsInteraction(&self) -> bool {
        self.prevent_counting_as_interaction_
    }
    pub fn SetPreventCountingAsInteractionTrue(&mut self) {
        self.prevent_counting_as_interaction_ = true;
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:404-406
    // MergeDispatchTypes is defined in the .cc translation below.

    // Source: third_party/blink/public/common/input/web_input_event.h:408-420
    pub(crate) fn new(
        event_type: Type,
        type_range_min: Type,
        type_range_max: Type,
        modifiers: i32,
        time_stamp: TimeTicks,
    ) -> Self {
        assert!(type_range_min <= type_range_max);
        let event = Self {
            frame_scale_: 1.0,
            frame_translate_: Vector2dF::default(),
            time_stamp_: time_stamp,
            queued_time_stamp_: TimeTicks::new(),
            type_: event_type,
            type_range_min_: type_range_min,
            type_range_max_: type_range_max,
            modifiers_: modifiers,
            event_latency_metadata_: EventLatencyMetadata::default(),
            prevent_counting_as_interaction_: false,
        };
        event.CheckEventType();
        event
    }

    // Source: third_party/blink/public/common/input/web_input_event.h:440-446
    fn CheckEventType(&self) {
        debug_assert!(
            self.type_ == Type::kUndefined
                || (self.type_range_min_ <= self.type_ && self.type_ <= self.type_range_max_),
            "{}",
            Self::GetName(self.type_)
        );
    }
}

// Source: third_party/blink/public/common/input/web_input_event.h:378-387
// Rust representation of the virtual base interface. Event/EventMut/AsAny
// express base access and checked downcasts for translated derived classes.
pub trait WebInputEventPayload: Any + Send + Sync {
    fn Event(&self) -> &WebInputEvent;
    fn EventMut(&mut self) -> &mut WebInputEvent;
    fn AsAny(&self) -> &dyn Any;
    fn AsAnyMut(&mut self) -> &mut dyn Any;
    fn Clone(&self) -> Box<dyn WebInputEventPayload>;
    fn CanCoalesce(&self, event: &dyn WebInputEventPayload) -> bool;
    fn Coalesce(&mut self, event: &dyn WebInputEventPayload);
}

// Source: third_party/blink/common/input/web_input_event.cc:9-13
const kButtonModifiers: i32 = WebInputEvent::kLeftButtonDown
    | WebInputEvent::kMiddleButtonDown
    | WebInputEvent::kRightButtonDown
    | WebInputEvent::kBackButtonDown
    | WebInputEvent::kForwardButtonDown;

impl WebInputEvent {
    // Source: third_party/blink/common/input/web_input_event.cc:15-27
    pub(crate) fn MergeDispatchTypes(type_1: DispatchType, type_2: DispatchType) -> DispatchType {
        const _: () =
            assert!((DispatchType::kBlocking as i32) < DispatchType::kEventNonBlocking as i32);
        const _: () = assert!(
            (DispatchType::kEventNonBlocking as i32)
                < DispatchType::kListenersNonBlockingPassive as i32
        );
        const _: () = assert!(
            (DispatchType::kListenersNonBlockingPassive as i32)
                < DispatchType::kListenersForcedNonBlockingDueToFling as i32
        );
        std::cmp::min(type_1, type_2)
    }

    // Source: third_party/blink/common/input/web_input_event.cc:29-110
    pub fn GetTypeAsUiEventType(&self) -> UiEventType {
        match self.type_ {
            Type::kMouseDown => UiEventType::kMousePressed,
            Type::kMouseUp => UiEventType::kMouseReleased,
            Type::kMouseMove => {
                if self.modifiers_ & kButtonModifiers != 0 {
                    UiEventType::kMouseDragged
                } else {
                    UiEventType::kMouseMoved
                }
            }
            Type::kMouseEnter => UiEventType::kMouseEntered,
            Type::kMouseLeave => UiEventType::kMouseExited,
            Type::kContextMenu => UiEventType::kUnknown,
            Type::kMouseWheel => UiEventType::kMousewheel,
            Type::kRawKeyDown => UiEventType::kKeyPressed,
            Type::kKeyDown => UiEventType::kKeyPressed,
            Type::kKeyUp => UiEventType::kKeyReleased,
            Type::kChar => UiEventType::kKeyPressed,
            Type::kGestureScrollBegin => UiEventType::kGestureScrollBegin,
            Type::kGestureScrollEnd => UiEventType::kGestureScrollEnd,
            Type::kGestureScrollUpdate => UiEventType::kGestureScrollUpdate,
            Type::kGestureFlingStart => UiEventType::kScrollFlingStart,
            Type::kGestureFlingCancel => UiEventType::kScrollFlingCancel,
            Type::kGesturePinchBegin => UiEventType::kGesturePinchBegin,
            Type::kGesturePinchEnd => UiEventType::kGesturePinchEnd,
            Type::kGesturePinchUpdate => UiEventType::kGesturePinchUpdate,
            Type::kGestureTapDown => UiEventType::kGestureTapDown,
            Type::kGestureShowPress => UiEventType::kGestureShowPress,
            Type::kGestureTap => UiEventType::kGestureTap,
            Type::kGestureTapCancel => UiEventType::kGestureTapCancel,
            Type::kGestureShortPress => UiEventType::kGestureShortPress,
            Type::kGestureLongPress => UiEventType::kGestureLongPress,
            Type::kGestureLongTap => UiEventType::kGestureLongTap,
            Type::kGestureTwoFingerTap => UiEventType::kGestureTwoFingerTap,
            Type::kGestureTapUnconfirmed => UiEventType::kGestureTapUnconfirmed,
            Type::kGestureDoubleTap => UiEventType::kGestureDoubleTap,
            Type::kTouchStart => UiEventType::kTouchPressed,
            Type::kTouchMove => UiEventType::kTouchMoved,
            Type::kTouchEnd => UiEventType::kTouchReleased,
            Type::kTouchCancel => UiEventType::kTouchCancelled,
            Type::kTouchScrollStarted => UiEventType::kTouchPressed,
            Type::kPointerDown => UiEventType::kTouchPressed,
            Type::kPointerUp => UiEventType::kTouchReleased,
            Type::kPointerMove => UiEventType::kTouchMoved,
            Type::kPointerCancel => UiEventType::kTouchCancelled,
            _ => UiEventType::kUnknown,
        }
    }
}
