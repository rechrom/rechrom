// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/common/input/web_gesture_event.h:1-388
// Source: third_party/blink/common/input/web_gesture_event.cc:1-375
// The C++ union is represented by independent zero-initialized payload structs;
// callers select the active member using the original event type.
use super::web_input_event::{Type, WebInputEvent, WebInputEventPayload};
use super::web_mouse_event::WebGestureEventView;
use super::web_pointer_properties::PointerType;
use crate::base::time::time::TimeTicks;
use crate::ui::events::types::scroll_types::ScrollGranularity;
use foundation::gfx_geometry::{PointF, SizeF, Vector2dF};
use foundation_transform::gfx::Transform;
use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

// Source: third_party/blink/common/input/web_gesture_event.cc:16-24.
fn IsContinuousGestureEvent(event_type: Type) -> bool {
    match event_type {
        Type::kGestureScrollUpdate | Type::kGesturePinchUpdate => true,
        _ => false,
    }
}

// Source: third_party/blink/common/input/web_gesture_event.cc:26-45.
fn GetTransformForEvent(gesture_event: &WebGestureEvent) -> Transform {
    let mut gesture_transform = Transform::default();
    if gesture_event.GetType() == Type::kGestureScrollUpdate {
        gesture_transform.Translate(
            gesture_event.data.scroll_update.delta_x,
            gesture_event.data.scroll_update.delta_y,
        );
    } else if gesture_event.GetType() == Type::kGesturePinchUpdate {
        let scale = gesture_event.data.pinch_update.scale;
        gesture_transform.Translate(
            -gesture_event.PositionInWidget().x(),
            -gesture_event.PositionInWidget().y(),
        );
        gesture_transform.Scale(scale, scale);
        gesture_transform.Translate(
            gesture_event.PositionInWidget().x(),
            gesture_event.PositionInWidget().y(),
        );
    } else {
        unreachable!(
            "Invalid event type for transform retrieval: {}",
            WebInputEvent::GetName(gesture_event.GetType())
        );
    }
    gesture_transform
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum GestureDevice {
    #[default]
    kUninitialized,
    kTouchpad,
    kTouchscreen,
    kSyntheticAutoscroll,
    kScrollbar,
}
pub type WebGestureDevice = GestureDevice;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum InertialPhaseState {
    #[default]
    kUnknownMomentum,
    kNonMomentum,
    kMomentum,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ScrollInputType {
    kTouchscreen,
    kWheel,
    kAutoscroll,
    kScrollbar,
}
#[derive(Clone, Copy, Debug)]
pub struct Tap {
    pub tap_count: i32,
    pub width: f32,
    pub height: f32,
    pub needs_wheel_event: bool,
}
impl Default for Tap {
    fn default() -> Self {
        Self {
            tap_count: Default::default(),
            width: Default::default(),
            height: Default::default(),
            needs_wheel_event: Default::default(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct TapDown {
    pub tap_down_count: i32,
    pub width: f32,
    pub height: f32,
}
impl Default for TapDown {
    fn default() -> Self {
        Self {
            tap_down_count: Default::default(),
            width: Default::default(),
            height: Default::default(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ContactArea {
    pub width: f32,
    pub height: f32,
}
impl Default for ContactArea {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct TwoFingerTap {
    pub first_finger_width: f32,
    pub first_finger_height: f32,
}
impl Default for TwoFingerTap {
    fn default() -> Self {
        Self {
            first_finger_width: Default::default(),
            first_finger_height: Default::default(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ScrollBegin {
    pub scrollable_area_element_id: u64,
    pub delta_x_hint: f32,
    pub delta_y_hint: f32,
    pub pointer_count: i32,
    pub delta_hint_units: ScrollGranularity,
    pub inertial_phase: InertialPhaseState,
    pub target_viewport: bool,
    pub synthetic: bool,
    pub cursor_control: bool,
    pub main_thread_hit_tested_reasons: u64,
}
impl Default for ScrollBegin {
    fn default() -> Self {
        Self {
            scrollable_area_element_id: Default::default(),
            delta_x_hint: Default::default(),
            delta_y_hint: Default::default(),
            pointer_count: Default::default(),
            delta_hint_units: ScrollGranularity::kScrollByPrecisePixel,
            inertial_phase: Default::default(),
            target_viewport: Default::default(),
            synthetic: Default::default(),
            cursor_control: Default::default(),
            main_thread_hit_tested_reasons: Default::default(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ScrollUpdate {
    pub delta_x: f32,
    pub delta_y: f32,
    pub delta_x_unconstrained: f32,
    pub delta_y_unconstrained: f32,
    pub inertial_phase: InertialPhaseState,
    pub delta_units: ScrollGranularity,
}
impl Default for ScrollUpdate {
    fn default() -> Self {
        Self {
            delta_x: Default::default(),
            delta_y: Default::default(),
            delta_x_unconstrained: Default::default(),
            delta_y_unconstrained: Default::default(),
            inertial_phase: Default::default(),
            delta_units: ScrollGranularity::kScrollByPrecisePixel,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ScrollEnd {
    pub delta_x_compensated: f32,
    pub delta_y_compensated: f32,
    pub delta_units: ScrollGranularity,
    pub inertial_phase: InertialPhaseState,
    pub synthetic: bool,
    pub generated_by_fling_controller: bool,
}
impl Default for ScrollEnd {
    fn default() -> Self {
        Self {
            delta_x_compensated: Default::default(),
            delta_y_compensated: Default::default(),
            delta_units: ScrollGranularity::kScrollByPrecisePixel,
            inertial_phase: Default::default(),
            synthetic: Default::default(),
            generated_by_fling_controller: Default::default(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct FlingStart {
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub target_viewport: bool,
}
impl Default for FlingStart {
    fn default() -> Self {
        Self {
            velocity_x: Default::default(),
            velocity_y: Default::default(),
            target_viewport: Default::default(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct FlingCancel {
    pub target_viewport: bool,
    pub prevent_boosting: bool,
}
impl Default for FlingCancel {
    fn default() -> Self {
        Self {
            target_viewport: Default::default(),
            prevent_boosting: Default::default(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct PinchUpdate {
    pub needs_wheel_event: bool,
    pub zoom_disabled: bool,
    pub scale: f32,
}
impl Default for PinchUpdate {
    fn default() -> Self {
        Self {
            needs_wheel_event: Default::default(),
            zoom_disabled: Default::default(),
            scale: Default::default(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct PinchBoundary {
    pub needs_wheel_event: bool,
}
impl Default for PinchBoundary {
    fn default() -> Self {
        Self {
            needs_wheel_event: Default::default(),
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct GestureData {
    pub tap: Tap,
    pub tap_down: TapDown,
    pub show_press: ContactArea,
    pub long_press: ContactArea,
    pub two_finger_tap: TwoFingerTap,
    pub scroll_begin: ScrollBegin,
    pub scroll_update: ScrollUpdate,
    pub scroll_end: ScrollEnd,
    pub fling_start: FlingStart,
    pub fling_cancel: FlingCancel,
    pub pinch_update: PinchUpdate,
    pub pinch_begin: PinchBoundary,
    pub pinch_end: PinchBoundary,
}
#[derive(Clone, Debug)]
pub struct WebGestureEvent {
    pub event: WebInputEvent,
    pub is_source_touch_event_set_blocking: bool,
    pub primary_pointer_type: PointerType,
    pub primary_unique_touch_event_id: u32,
    pub unique_touch_event_id: u32,
    pub data: GestureData,
    position_in_widget_: PointF,
    position_in_screen_: PointF,
    source_device_: GestureDevice,
}
impl Deref for WebGestureEvent {
    type Target = WebInputEvent;
    fn deref(&self) -> &Self::Target {
        &self.event
    }
}
impl DerefMut for WebGestureEvent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event
    }
}
impl Default for WebGestureEvent {
    fn default() -> Self {
        Self::new(
            Type::kUndefined,
            0,
            TimeTicks::default(),
            GestureDevice::kUninitialized,
        )
    }
}
impl WebGestureEvent {
    pub fn new(
        event_type: Type,
        modifiers: i32,
        time_stamp: TimeTicks,
        source_device: GestureDevice,
    ) -> Self {
        Self {
            event: WebInputEvent::new(
                event_type,
                Type::kGestureTypeFirst,
                Type::kGestureTypeLast,
                modifiers,
                time_stamp,
            ),
            is_source_touch_event_set_blocking: false,
            primary_pointer_type: PointerType::kUnknown,
            primary_unique_touch_event_id: 0,
            unique_touch_event_id: 0,
            data: GestureData::default(),
            position_in_widget_: PointF::default(),
            position_in_screen_: PointF::default(),
            source_device_: source_device,
        }
    }
    pub fn PositionInWidget(&self) -> PointF {
        self.position_in_widget_
    }
    pub fn PositionInScreen(&self) -> PointF {
        self.position_in_screen_
    }
    pub fn SetPositionInWidget(&mut self, p: PointF) {
        self.position_in_widget_ = p;
    }
    pub fn SetPositionInScreen(&mut self, p: PointF) {
        self.position_in_screen_ = p;
    }
    pub fn SourceDevice(&self) -> GestureDevice {
        self.source_device_
    }
    pub fn SetSourceDevice(&mut self, d: GestureDevice) {
        self.source_device_ = d;
    }
    pub fn GetScrollInputType(&self) -> ScrollInputType {
        match self.SourceDevice() {
            GestureDevice::kTouchpad => {
                debug_assert!(
                    self.IsGestureScroll()
                        || WebInputEvent::IsPinchGestureEventType(self.GetType())
                );
                ScrollInputType::kWheel
            }
            GestureDevice::kTouchscreen => {
                debug_assert!(
                    self.IsGestureScroll()
                        || WebInputEvent::IsPinchGestureEventType(self.GetType())
                );
                ScrollInputType::kTouchscreen
            }
            GestureDevice::kSyntheticAutoscroll => {
                debug_assert!(self.IsGestureScroll());
                ScrollInputType::kAutoscroll
            }
            GestureDevice::kScrollbar => {
                debug_assert!(self.IsGestureScroll());
                ScrollInputType::kScrollbar
            }
            GestureDevice::kUninitialized => panic!("uninitialized gesture device"),
        }
    }
    pub fn DeltaXInRootFrame(&self) -> f32 {
        (if self.GetType() == Type::kGestureScrollBegin {
            self.data.scroll_begin.delta_x_hint
        } else {
            self.data.scroll_update.delta_x
        }) / self.FrameScale()
    }
    pub fn DeltaYInRootFrame(&self) -> f32 {
        (if self.GetType() == Type::kGestureScrollBegin {
            self.data.scroll_begin.delta_y_hint
        } else {
            self.data.scroll_update.delta_y
        }) / self.FrameScale()
    }
    pub fn DeltaUnits(&self) -> ScrollGranularity {
        match self.GetType() {
            Type::kGestureScrollBegin => self.data.scroll_begin.delta_hint_units,
            Type::kGestureScrollUpdate => self.data.scroll_update.delta_units,
            _ => {
                debug_assert_eq!(self.GetType(), Type::kGestureScrollEnd);
                self.data.scroll_end.delta_units
            }
        }
    }
    pub fn InertialPhase(&self) -> InertialPhaseState {
        match self.GetType() {
            Type::kGestureScrollBegin => self.data.scroll_begin.inertial_phase,
            Type::kGestureScrollUpdate => self.data.scroll_update.inertial_phase,
            _ => {
                debug_assert_eq!(self.GetType(), Type::kGestureScrollEnd);
                self.data.scroll_end.inertial_phase
            }
        }
    }
    pub fn Synthetic(&self) -> bool {
        if self.GetType() == Type::kGestureScrollBegin {
            self.data.scroll_begin.synthetic
        } else {
            debug_assert_eq!(self.GetType(), Type::kGestureScrollEnd);
            self.data.scroll_end.synthetic
        }
    }
    // Source: third_party/blink/common/input/web_gesture_event.cc:172-196.
    pub fn TapAreaInRootFrame(&self) -> SizeF {
        if self.GetType() == Type::kGestureTwoFingerTap {
            return SizeF::new(
                self.data.two_finger_tap.first_finger_width / self.FrameScale(),
                self.data.two_finger_tap.first_finger_height / self.FrameScale(),
            );
        } else if matches!(
            self.GetType(),
            Type::kGestureShortPress | Type::kGestureLongPress | Type::kGestureLongTap
        ) {
            return SizeF::new(
                self.data.long_press.width / self.FrameScale(),
                self.data.long_press.height / self.FrameScale(),
            );
        } else if matches!(
            self.GetType(),
            Type::kGestureTap | Type::kGestureTapUnconfirmed | Type::kGestureDoubleTap
        ) {
            return SizeF::new(
                self.data.tap.width / self.FrameScale(),
                self.data.tap.height / self.FrameScale(),
            );
        } else if self.GetType() == Type::kGestureTapDown {
            return SizeF::new(
                self.data.tap_down.width / self.FrameScale(),
                self.data.tap_down.height / self.FrameScale(),
            );
        } else if self.GetType() == Type::kGestureShowPress {
            return SizeF::new(
                self.data.show_press.width / self.FrameScale(),
                self.data.show_press.height / self.FrameScale(),
            );
        }
        SizeF::default()
    }
    // Source: third_party/blink/common/input/web_gesture_event.cc:198-201.
    pub fn PositionInRootFrame(&self) -> PointF {
        let mut position = self.position_in_widget_;
        position.ScaleUniform(1.0 / self.FrameScale());
        position + self.FrameTranslate()
    }
    pub fn TapCount(&self) -> i32 {
        debug_assert_eq!(self.GetType(), Type::kGestureTap);
        self.data.tap.tap_count
    }
    pub fn TapDownCount(&self) -> i32 {
        debug_assert_eq!(self.GetType(), Type::kGestureTapDown);
        self.data.tap_down.tap_down_count
    }
    // Source: third_party/blink/common/input/web_gesture_event.cc:213-222.
    pub fn ApplyTouchAdjustment(&mut self, root_frame_coords: PointF) {
        let mut position = self.position_in_widget_;
        position.ScaleUniform(1.0 / self.FrameScale());
        self.SetFrameTranslate(root_frame_coords - position);
    }
    pub fn FlattenTransform(&mut self) {
        let scale = self.FrameScale();
        if scale != 1. {
            match self.GetType() {
                Type::kGestureScrollBegin => {
                    self.data.scroll_begin.delta_x_hint /= scale;
                    self.data.scroll_begin.delta_y_hint /= scale;
                }
                Type::kGestureScrollUpdate => {
                    self.data.scroll_update.delta_x /= scale;
                    self.data.scroll_update.delta_y /= scale;
                }
                Type::kGestureTwoFingerTap => {
                    self.data.two_finger_tap.first_finger_width /= scale;
                    self.data.two_finger_tap.first_finger_height /= scale;
                }
                Type::kGestureShortPress | Type::kGestureLongPress | Type::kGestureLongTap => {
                    self.data.long_press.width /= scale;
                    self.data.long_press.height /= scale;
                }
                Type::kGestureTap | Type::kGestureTapUnconfirmed | Type::kGestureDoubleTap => {
                    self.data.tap.width /= scale;
                    self.data.tap.height /= scale;
                }
                Type::kGestureTapDown => {
                    self.data.tap_down.width /= scale;
                    self.data.tap_down.height /= scale;
                }
                Type::kGestureShowPress => {
                    self.data.show_press.width /= scale;
                    self.data.show_press.height /= scale;
                }
                _ => {}
            }
        }
        self.SetPositionInWidget(self.PositionInRootFrame());
        self.SetFrameTranslate(Vector2dF::default());
        self.SetFrameScale(1.);
    }
    pub fn IsScrollEvent(&self) -> bool {
        match self.GetType() {
            Type::kGestureScrollBegin
            | Type::kGestureScrollEnd
            | Type::kGestureScrollUpdate
            | Type::kGestureFlingStart
            | Type::kGestureFlingCancel
            | Type::kGesturePinchBegin
            | Type::kGesturePinchEnd
            | Type::kGesturePinchUpdate => true,
            Type::kGestureTap
            | Type::kGestureTapUnconfirmed
            | Type::kGestureTapDown
            | Type::kGestureShowPress
            | Type::kGestureTapCancel
            | Type::kGestureTwoFingerTap
            | Type::kGestureShortPress
            | Type::kGestureLongPress
            | Type::kGestureLongTap
            | Type::kGestureDoubleTap => false,
            _ => panic!("invalid gesture type"),
        }
    }
    pub fn IsTargetViewport(&self) -> bool {
        match self.GetType() {
            Type::kGestureScrollBegin => self.data.scroll_begin.target_viewport,
            Type::kGestureFlingStart => self.data.fling_start.target_viewport,
            Type::kGestureFlingCancel => self.data.fling_cancel.target_viewport,
            _ => false,
        }
    }
    pub fn IsTouchpadZoomEvent(&self) -> bool {
        self.SourceDevice() == GestureDevice::kTouchpad
            && (WebInputEvent::IsPinchGestureEventType(self.GetType())
                || self.GetType() == Type::kGestureDoubleTap)
    }
    pub fn NeedsWheelEvent(&self) -> bool {
        debug_assert!(self.IsTouchpadZoomEvent());
        match self.GetType() {
            Type::kGesturePinchBegin => self.data.pinch_begin.needs_wheel_event,
            Type::kGesturePinchUpdate => self.data.pinch_update.needs_wheel_event,
            Type::kGesturePinchEnd => self.data.pinch_end.needs_wheel_event,
            Type::kGestureDoubleTap => self.data.tap.needs_wheel_event,
            _ => panic!("invalid zoom gesture"),
        }
    }
    pub fn SetNeedsWheelEvent(&mut self, b: bool) {
        debug_assert!(!b || self.IsTouchpadZoomEvent());
        match self.GetType() {
            Type::kGesturePinchBegin => self.data.pinch_begin.needs_wheel_event = b,
            Type::kGesturePinchUpdate => self.data.pinch_update.needs_wheel_event = b,
            Type::kGesturePinchEnd => self.data.pinch_end.needs_wheel_event = b,
            Type::kGestureDoubleTap => self.data.tap.needs_wheel_event = b,
            _ => {}
        }
    }
    // Source: third_party/blink/common/input/web_gesture_event.cc:58-80.
    // The polymorphic entry checks the event class before projecting Self.
    pub fn CanCoalesceGesture(&self, other: &Self) -> bool {
        if self.GetType() != other.GetType()
            || self.SourceDevice() != other.SourceDevice()
            || self.GetModifiers() != other.GetModifiers()
        {
            return false;
        }
        if self.GetType() == Type::kGestureScrollUpdate {
            return true;
        }
        const kAnchorTolerance: f32 = 1.0;
        if self.GetType() == Type::kGesturePinchUpdate
            && (self.PositionInWidget().x() - other.PositionInWidget().x()).abs() < kAnchorTolerance
            && (self.PositionInWidget().y() - other.PositionInWidget().y()).abs() < kAnchorTolerance
        {
            return true;
        }
        false
    }
    // Source: third_party/blink/common/input/web_gesture_event.cc:82-85.
    // Typed callers and the virtual entry share the same mutation below, each
    // checking its original input before the payload is consumed.
    pub fn CoalesceGesture(&mut self, other: &Self) {
        debug_assert!(self.CanCoalesceGesture(other));
        self.CoalesceGesturePayload(other);
    }
    // Source: third_party/blink/common/input/web_gesture_event.cc:86-102.
    fn CoalesceGesturePayload(&mut self, other: &Self) {
        if self.GetType() == Type::kGestureScrollUpdate {
            self.data.scroll_update.delta_x += other.data.scroll_update.delta_x;
            self.data.scroll_update.delta_y += other.data.scroll_update.delta_y;
            self.data.scroll_update.delta_x_unconstrained +=
                other.data.scroll_update.delta_x_unconstrained;
            self.data.scroll_update.delta_y_unconstrained +=
                other.data.scroll_update.delta_y_unconstrained;
        } else if self.GetType() == Type::kGesturePinchUpdate {
            self.data.pinch_update.scale *= other.data.pinch_update.scale;
            if self.data.pinch_update.scale < f32::MIN_POSITIVE {
                self.data.pinch_update.scale = f32::MIN_POSITIVE;
            } else if self.data.pinch_update.scale > f32::MAX {
                self.data.pinch_update.scale = f32::MAX;
            }
        }
    }
    pub fn IsCompatibleScrollorPinch(new_event: &Self, event_in_queue: &Self) -> bool {
        debug_assert!(matches!(
            new_event.GetType(),
            Type::kGestureScrollUpdate | Type::kGesturePinchUpdate
        ));
        matches!(
            event_in_queue.GetType(),
            Type::kGestureScrollUpdate | Type::kGesturePinchUpdate
        ) && event_in_queue.GetModifiers() == new_event.GetModifiers()
            && event_in_queue.SourceDevice() == GestureDevice::kTouchscreen
            && new_event.SourceDevice() == GestureDevice::kTouchscreen
    }
    // Source: third_party/blink/common/input/web_gesture_event.cc:291-335.
    pub fn CoalesceScrollAndPinch(
        second_last_event: Option<&Self>,
        last_event: &Self,
        new_event: &Self,
    ) -> (Self, Self) {
        debug_assert!(!last_event.CanCoalesceGesture(new_event));
        debug_assert!(IsContinuousGestureEvent(new_event.GetType()));
        debug_assert!(Self::IsCompatibleScrollorPinch(new_event, last_event));
        debug_assert!(
            second_last_event.is_none_or(|e| Self::IsCompatibleScrollorPinch(new_event, e))
        );
        let mut scroll = Self::new(
            Type::kGestureScrollUpdate,
            new_event.GetModifiers(),
            new_event.TimeStamp(),
            new_event.SourceDevice(),
        );
        scroll.primary_pointer_type = new_event.primary_pointer_type;
        scroll.primary_unique_touch_event_id = new_event.primary_unique_touch_event_id;
        let mut pinch = scroll.clone();
        pinch.SetType(Type::kGesturePinchUpdate);
        pinch.SetPositionInWidget(if new_event.GetType() == Type::kGesturePinchUpdate {
            new_event.PositionInWidget()
        } else {
            last_event.PositionInWidget()
        });
        let mut combined_scroll_pinch = GetTransformForEvent(last_event);
        if let Some(second_last_event) = second_last_event {
            combined_scroll_pinch.PreConcat(&GetTransformForEvent(second_last_event));
        }
        combined_scroll_pinch.PostConcat(&GetTransformForEvent(new_event));

        let combined_scale = combined_scroll_pinch.To2dScale().x();
        let combined_translation = combined_scroll_pinch.To2dTranslation();
        scroll.data.scroll_update.delta_x =
            (combined_translation.x() + pinch.PositionInWidget().x()) / combined_scale
                - pinch.PositionInWidget().x();
        scroll.data.scroll_update.delta_y =
            (combined_translation.y() + pinch.PositionInWidget().y()) / combined_scale
                - pinch.PositionInWidget().y();
        pinch.data.pinch_update.scale = combined_scale;
        (scroll, pinch)
    }
    pub fn GenerateInjectedScrollbarGestureScroll(
        event_type: Type,
        timestamp: TimeTicks,
        position: PointF,
        delta: Vector2dF,
        granularity: ScrollGranularity,
    ) -> Self {
        let mut e = Self::new(event_type, 0, timestamp, GestureDevice::kScrollbar);
        debug_assert!(e.IsGestureScroll());
        if event_type == Type::kGestureScrollBegin {
            e.data.scroll_begin.delta_x_hint = -delta.x();
            e.data.scroll_begin.delta_y_hint = -delta.y();
            e.data.scroll_begin.inertial_phase = InertialPhaseState::kNonMomentum;
            e.data.scroll_begin.delta_hint_units = granularity;
        } else if event_type == Type::kGestureScrollUpdate {
            e.data.scroll_update.delta_x = -delta.x();
            e.data.scroll_update.delta_y = -delta.y();
            e.data.scroll_update.delta_x_unconstrained = -delta.x();
            e.data.scroll_update.delta_y_unconstrained = -delta.y();
            e.data.scroll_update.inertial_phase = InertialPhaseState::kNonMomentum;
            e.data.scroll_update.delta_units = granularity;
        }
        e.SetPositionInWidget(position);
        e
    }
}
impl WebGestureEventView for WebGestureEvent {
    fn PositionInWidget(&self) -> PointF {
        self.PositionInWidget()
    }
    fn PositionInScreen(&self) -> PointF {
        self.PositionInScreen()
    }
    fn FrameScale(&self) -> f32 {
        self.event.FrameScale()
    }
    fn FrameTranslate(&self) -> Vector2dF {
        self.event.FrameTranslate()
    }
    fn GetType(&self) -> Type {
        self.event.GetType()
    }
}
impl WebInputEventPayload for WebGestureEvent {
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
    fn Clone(&self) -> Box<dyn WebInputEventPayload> {
        Box::new(self.clone())
    }
    // Source: third_party/blink/common/input/web_gesture_event.cc:53-80.
    fn CanCoalesce(&self, event: &dyn WebInputEventPayload) -> bool {
        if !WebInputEvent::IsGestureEventType(event.Event().GetType()) {
            return false;
        }
        let gesture_event = event
            .AsAny()
            .downcast_ref::<Self>()
            .expect("gesture payload");
        self.CanCoalesceGesture(gesture_event)
    }
    // Source: third_party/blink/common/input/web_gesture_event.cc:82-102.
    fn Coalesce(&mut self, event: &dyn WebInputEventPayload) {
        debug_assert!(self.CanCoalesce(event));
        let gesture_event = event
            .AsAny()
            .downcast_ref::<Self>()
            .expect("gesture payload");
        self.CoalesceGesturePayload(gesture_event);
    }
}
