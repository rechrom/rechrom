// Copyright 2014 The Chromium Authors. BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: ui/events/gesture_detection/gesture_event_data_packet.h;
// gesture_event_data_packet.cc:17-152.
use crate::public::common::input::web_gesture_event::{GestureDevice, WebGestureEvent};
use crate::public::common::input::web_input_event::Type;
use crate::public::common::input::web_touch_event::WebTouchEvent;
use crate::ui::events::event_latency_metadata::EventLatencyMetadata;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum GestureSource {
    Start,
    PointerDown,
    Move,
    PointerUp,
    End,
    Cancel,
    Timeout,
}
pub(super) struct GesturePacket {
    pub source: GestureSource,
    pub event: WebGestureEvent,
    pub gestures: Vec<WebGestureEvent>,
    pub(super) ack: Option<(bool, bool)>,
    pub(super) latency: Option<EventLatencyMetadata>,
}
impl GesturePacket {
    pub fn new(
        source: GestureSource,
        event: WebGestureEvent,
        gestures: Vec<WebGestureEvent>,
    ) -> Self {
        Self {
            source,
            event,
            gestures,
            ack: None,
            latency: None,
        }
    }
}
impl GesturePacket {
    // FromTouch retains the original source event metadata and gesture order.
    pub fn FromTouch(
        e: &WebTouchEvent,
        primary_touch_id: u32,
        gestures: Vec<WebGestureEvent>,
    ) -> Self {
        let source = match e.GetType() {
            Type::kTouchStart if e.IsTouchSequenceStart() => GestureSource::Start,
            Type::kTouchStart => GestureSource::PointerDown,
            Type::kTouchMove => GestureSource::Move,
            Type::kTouchEnd if e.IsTouchSequenceEnd() => GestureSource::End,
            Type::kTouchEnd => GestureSource::PointerUp,
            _ => GestureSource::Cancel,
        };
        let mut event = WebGestureEvent::new(
            Type::kGestureTapCancel,
            e.GetModifiers(),
            e.TimeStamp(),
            GestureDevice::kTouchscreen,
        );
        event.unique_touch_event_id = e.unique_touch_event_id;
        event.primary_unique_touch_event_id = primary_touch_id;
        event.SetQueuedTimeStamp(e.QueuedTimeStamp());
        if e.touches_length != 0 {
            event.SetPositionInWidget(*e.touches[0].pointer_properties.PositionInWidget());
            event.SetPositionInScreen(*e.touches[0].pointer_properties.PositionInScreen());
            event.primary_pointer_type = e.touches[0].pointer_properties.pointer_type;
        }
        *event.GetModifiableEventLatencyMetadata() = *e.GetEventLatencyMetadata();
        Self::new(source, event, gestures)
    }
    pub fn FromTouchTimeout(event: WebGestureEvent) -> Self {
        Self::new(GestureSource::Timeout, event.clone(), vec![event])
    }
    pub fn Ack(&mut self, consumed: bool, blocking: bool, latency: Option<EventLatencyMetadata>) {
        self.ack = Some((consumed, blocking));
        self.latency = latency;
    }
    // Apply the packet facts at the existing per-gesture publication point.
    // Taking copied scalar facts preserves partial-move ownership of gestures.
    pub fn PrepareGesture(
        gesture: &mut WebGestureEvent,
        source: GestureSource,
        ack: Option<(bool, bool)>,
        latency: Option<EventLatencyMetadata>,
    ) {
        if let Some((_, blocking)) = ack {
            gesture.is_source_touch_event_set_blocking = blocking;
        }
        if source == GestureSource::Move
            && matches!(
                gesture.GetType(),
                Type::kGestureScrollBegin | Type::kGestureScrollUpdate
            )
        {
            if let Some(latency) = latency {
                gesture
                    .GetModifiableEventLatencyMetadata()
                    .scrolls_blocking_touch_dispatched_to_renderer = latency.dispatched_to_renderer;
            }
        }
    }
}
