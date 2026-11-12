// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/common/input/web_touch_event.h:17-89
// Source: third_party/blink/common/input/web_touch_event.cc:1-158

use std::any::Any;
use std::ops::{Deref, DerefMut};

use super::web_input_event::{DispatchType, Type, WebInputEvent, WebInputEventPayload};
use super::web_pointer_event::WebTouchEventView;
use super::web_touch_point::{TouchState, WebTouchPoint};
use crate::base::time::time::TimeTicks;

// Source: web_touch_event.h:17-49. The fixed array preserves the source cap;
// touches_length remains independent, including its validation in CanCoalesce.
#[derive(Clone, Debug)]
pub struct WebTouchEvent {
    pub event: WebInputEvent,
    pub touches_length: usize,
    pub touches: [WebTouchPoint; Self::kTouchesLengthCap],
    pub dispatch_type: DispatchType,
    pub moved_beyond_slop_region: bool,
    pub hovering: bool,
    pub touch_start_or_first_touch_move: bool,
    pub unique_touch_event_id: u32,
}

impl Default for WebTouchEvent {
    fn default() -> Self {
        Self::new(Type::kUndefined, 0, TimeTicks::new())
    }
}

impl WebTouchEvent {
    pub const kTouchesLengthCap: usize = 16;

    // Source: web_touch_event.h:51-62.
    pub fn new(event_type: Type, modifiers: i32, time_stamp: TimeTicks) -> Self {
        Self {
            event: WebInputEvent::new(
                event_type,
                Type::kTouchTypeFirst,
                Type::kTouchTypeLast,
                modifiers,
                time_stamp,
            ),
            touches_length: 0,
            touches: std::array::from_fn(|_| WebTouchPoint::default()),
            dispatch_type: DispatchType::kBlocking,
            moved_beyond_slop_region: false,
            hovering: false,
            touch_start_or_first_touch_move: false,
            unique_touch_event_id: 0,
        }
    }

    // Source: web_touch_event.cc:15-21.
    fn IndexOfTouchID(&self, id: i32) -> Option<usize> {
        self.touches[..self.touches_length]
            .iter()
            .position(|touch| touch.pointer_properties.id == id)
    }

    // Source: web_touch_event.cc:95-106.
    pub fn FlattenTransform(&self) -> Self {
        let mut event = self.clone();
        for index in 0..self.touches_length {
            event.touches[index] = self.TouchPointInRootFrame(index);
        }
        event.SetFrameTranslate(Default::default());
        event.SetFrameScale(1.0);
        event
    }

    // Source: web_touch_event.cc:108-122.
    pub fn TouchPointInRootFrame(&self, index: usize) -> WebTouchPoint {
        debug_assert!(index < self.touches_length);
        if index >= self.touches_length {
            return WebTouchPoint::default();
        }
        let mut point = self.touches[index].clone();
        point.radius_x /= self.FrameScale();
        point.radius_y /= self.FrameScale();
        let mut position = *point.pointer_properties.PositionInWidget();
        position.ScaleUniform(1.0 / self.FrameScale());
        point
            .pointer_properties
            .SetPositionInWidget(position + self.FrameTranslate());
        point
    }

    // Source: web_touch_event.h:76.
    pub fn IsCancelable(&self) -> bool {
        self.dispatch_type == DispatchType::kBlocking
    }

    // Source: web_touch_event.cc:124-137.
    pub fn IsTouchSequenceStart(&self) -> bool {
        debug_assert!(self.touches_length != 0 || self.GetType() == Type::kTouchScrollStarted);
        self.GetType() == Type::kTouchStart
            && self.touches[..self.touches_length]
                .iter()
                .all(|point| point.state == TouchState::kStatePressed)
    }

    // Source: web_touch_event.cc:139-155.
    pub fn IsTouchSequenceEnd(&self) -> bool {
        matches!(self.GetType(), Type::kTouchEnd | Type::kTouchCancel)
            && self.touches[..self.touches_length].iter().all(|point| {
                matches!(
                    point.state,
                    TouchState::kStateReleased | TouchState::kStateCancelled
                )
            })
    }
}

impl Deref for WebTouchEvent {
    type Target = WebInputEvent;
    fn deref(&self) -> &WebInputEvent {
        &self.event
    }
}
impl DerefMut for WebTouchEvent {
    fn deref_mut(&mut self) -> &mut WebInputEvent {
        &mut self.event
    }
}

impl WebTouchEventView for WebTouchEvent {
    fn Event(&self) -> &WebInputEvent {
        &self.event
    }
    fn Hovering(&self) -> bool {
        self.hovering
    }
    fn DispatchType(&self) -> DispatchType {
        self.dispatch_type
    }
    fn MovedBeyondSlopRegion(&self) -> bool {
        self.moved_beyond_slop_region
    }
    fn TouchStartOrFirstTouchMove(&self) -> bool {
        self.touch_start_or_first_touch_move
    }
    fn UniqueTouchEventId(&self) -> u32 {
        self.unique_touch_event_id
    }
}

impl WebInputEventPayload for WebTouchEvent {
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

    // Source: web_touch_event.cc:25-27.
    fn Clone(&self) -> Box<dyn WebInputEventPayload> {
        Box::new(self.clone())
    }

    // Source: web_touch_event.cc:29-60. ID matching is bijective, independent
    // of touch-array order, and compares each point's original pointer type.
    fn CanCoalesce(&self, other: &dyn WebInputEventPayload) -> bool {
        if !WebInputEvent::IsTouchEventType(other.Event().GetType()) {
            return false;
        }
        let other = other.AsAny().downcast_ref::<Self>().expect("touch payload");
        if self.GetType() != other.GetType()
            || self.GetType() != Type::kTouchMove
            || self.GetModifiers() != other.GetModifiers()
            || self.touches_length != other.touches_length
            || self.touches_length > Self::kTouchesLengthCap
        {
            return false;
        }
        let mut unmatched = (1u32 << self.touches_length) - 1;
        for point in &other.touches[..other.touches_length] {
            let Some(index) = self.IndexOfTouchID(point.pointer_properties.id) else {
                return false;
            };
            if unmatched & (1 << index) == 0
                || self.touches[index].pointer_properties.pointer_type
                    != point.pointer_properties.pointer_type
            {
                return false;
            }
            unmatched &= !(1 << index);
        }
        unmatched == 0
    }

    // Source: web_touch_event.cc:62-93. Preserve moved states, accumulated
    // movement, sequence identity, slop and first-move cancellation information.
    fn Coalesce(&mut self, other: &dyn WebInputEventPayload) {
        debug_assert!(self.CanCoalesce(other));
        let other = other.AsAny().downcast_ref::<Self>().expect("touch payload");
        let old = std::mem::replace(self, other.clone());
        for point in &mut self.touches[..self.touches_length] {
            let index = old
                .IndexOfTouchID(point.pointer_properties.id)
                .expect("coalescible touch ID");
            if old.touches[index].state == TouchState::kStateMoved {
                point.state = TouchState::kStateMoved;
                point.pointer_properties.movement_x +=
                    old.touches[index].pointer_properties.movement_x;
                point.pointer_properties.movement_y +=
                    old.touches[index].pointer_properties.movement_y;
            }
        }
        self.moved_beyond_slop_region |= old.moved_beyond_slop_region;
        self.dispatch_type =
            WebInputEvent::MergeDispatchTypes(old.dispatch_type, other.dispatch_type);
        self.unique_touch_event_id = old.unique_touch_event_id;
        self.touch_start_or_first_touch_move |= old.touch_start_or_first_touch_move;
    }
}
