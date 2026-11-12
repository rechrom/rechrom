// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/common/input/web_coalesced_input_event.h:1-66
// Source: third_party/blink/common/input/web_coalesced_input_event.cc:1-127
// Rust modules replace the includes, include guards, namespaces, and export
// annotations. Box owns the polymorphic event just as the source unique_ptr.

use super::web_input_event::WebInputEventPayload;

// Source: third_party/blink/public/common/input/web_coalesced_input_event.h:18-21,57-61
// L is the embedding's complete ui::LatencyInfo value. The source copies it
// without inspecting or merging its fields, so Clone preserves that boundary.
pub struct WebCoalescedInputEvent<L: Clone> {
    event_: Box<dyn WebInputEventPayload>,
    coalesced_events_: Vec<Box<dyn WebInputEventPayload>>,
    predicted_events_: Vec<Box<dyn WebInputEventPayload>>,
    latency_: L,
}

impl<L: Clone> WebCoalescedInputEvent<L> {
    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:34
    // Source: third_party/blink/common/input/web_coalesced_input_event.cc:17-19
    pub fn EventPointer(&mut self) -> &mut dyn WebInputEventPayload {
        self.event_.as_mut()
    }

    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:35
    // Source: third_party/blink/common/input/web_coalesced_input_event.cc:21-24
    pub fn AddCoalescedEvent(&mut self, event: &dyn WebInputEventPayload) {
        self.coalesced_events_.push(event.Clone());
    }

    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:36
    // Source: third_party/blink/common/input/web_coalesced_input_event.cc:26-28
    pub fn Event(&self) -> &dyn WebInputEventPayload {
        self.event_.as_ref()
    }

    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:37
    // Source: third_party/blink/common/input/web_coalesced_input_event.cc:30-32
    pub fn CoalescedEventSize(&self) -> usize {
        self.coalesced_events_.len()
    }

    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:38
    // Source: third_party/blink/common/input/web_coalesced_input_event.cc:34-37
    pub fn CoalescedEvent(&self, index: usize) -> &dyn WebInputEventPayload {
        self.coalesced_events_[index].as_ref()
    }

    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:39-40
    // Source: third_party/blink/common/input/web_coalesced_input_event.cc:39-42
    pub fn GetCoalescedEventsPointers(&self) -> &[Box<dyn WebInputEventPayload>] {
        &self.coalesced_events_
    }

    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:42
    // Source: third_party/blink/common/input/web_coalesced_input_event.cc:44-47
    pub fn AddPredictedEvent(&mut self, event: &dyn WebInputEventPayload) {
        self.predicted_events_.push(event.Clone());
    }

    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:43
    // Source: third_party/blink/common/input/web_coalesced_input_event.cc:49-51
    pub fn PredictedEventSize(&self) -> usize {
        self.predicted_events_.len()
    }

    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:44
    // Source: third_party/blink/common/input/web_coalesced_input_event.cc:53-56
    pub fn PredictedEvent(&self, index: usize) -> &dyn WebInputEventPayload {
        self.predicted_events_[index].as_ref()
    }

    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:45-46
    // Source: third_party/blink/common/input/web_coalesced_input_event.cc:58-61
    pub fn GetPredictedEventsPointers(&self) -> &[Box<dyn WebInputEventPayload>] {
        &self.predicted_events_
    }

    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:48
    pub fn latency_info(&self) -> &L {
        &self.latency_
    }

    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:49
    pub fn latency_info_mut(&mut self) -> &mut L {
        &mut self.latency_
    }
}

impl<L: Clone> WebCoalescedInputEvent<L> {
    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:23
    // Source: third_party/blink/common/input/web_coalesced_input_event.cc:63-65
    pub fn new(event: &dyn WebInputEventPayload, latency: &L) -> Self {
        Self::FromOwnedEvent(event.Clone(), latency)
    }

    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:24-25
    // Source: third_party/blink/common/input/web_coalesced_input_event.cc:67-73
    pub fn FromOwnedEvent(event: Box<dyn WebInputEventPayload>, latency: &L) -> Self {
        // A non-null Box enforces the source DCHECK(event_) at construction.
        let mut result = Self {
            event_: event,
            coalesced_events_: Vec::new(),
            predicted_events_: Vec::new(),
            latency_: latency.clone(),
        };
        result.coalesced_events_.push(result.event_.Clone());
        result
    }

    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:26-29
    // Source: third_party/blink/common/input/web_coalesced_input_event.cc:75-83
    pub fn WithEvents(
        event: Box<dyn WebInputEventPayload>,
        coalesced_events: Vec<Box<dyn WebInputEventPayload>>,
        predicted_events: Vec<Box<dyn WebInputEventPayload>>,
        latency: &L,
    ) -> Self {
        Self {
            event_: event,
            coalesced_events_: coalesced_events,
            predicted_events_: predicted_events,
            latency_: latency.clone(),
        }
    }
}

// Source: third_party/blink/public/common/input/web_coalesced_input_event.h:30-31
// Source: third_party/blink/common/input/web_coalesced_input_event.cc:85-93
impl<L: Clone> Clone for WebCoalescedInputEvent<L> {
    fn clone(&self) -> Self {
        let event = self.event_.Clone();
        let latency = self.latency_.clone();
        let mut coalesced_events = Vec::new();
        for coalesced_event in &self.coalesced_events_ {
            coalesced_events.push(coalesced_event.Clone());
        }
        let mut predicted_events = Vec::new();
        for predicted_event in &self.predicted_events_ {
            predicted_events.push(predicted_event.Clone());
        }
        Self {
            event_: event,
            coalesced_events_: coalesced_events,
            predicted_events_: predicted_events,
            latency_: latency,
        }
    }
}

// Source: third_party/blink/public/common/input/web_coalesced_input_event.h:32
// Source: third_party/blink/common/input/web_coalesced_input_event.cc:95
// Rust's field destruction supplies the source's default destructor.

impl<L: Clone> WebCoalescedInputEvent<L> {
    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:51
    // Source: third_party/blink/common/input/web_coalesced_input_event.cc:97-100
    #[must_use]
    pub fn CanCoalesceWith(&self, other: &Self) -> bool {
        self.event_.CanCoalesce(other.event_.as_ref())
    }

    // Source: third_party/blink/public/common/input/web_coalesced_input_event.h:53-55
    // Source: third_party/blink/common/input/web_coalesced_input_event.cc:102-125
    pub fn CoalesceWith(&mut self, newer_event: &Self) {
        // Source: third_party/blink/common/input/web_coalesced_input_event.cc:104-106
        // TODO: hook input trace duration and traceId/coalescedTraceId.

        // Source: third_party/blink/common/input/web_coalesced_input_event.cc:108-113
        // Coalesce newer input into the existing payload, retaining the newer
        // timestamp even when a concrete event's Coalesce has other behavior.
        let time_stamp = newer_event.event_.Event().TimeStamp();
        self.event_.Coalesce(newer_event.event_.as_ref());
        self.event_.EventMut().SetTimeStamp(time_stamp);
        self.AddCoalescedEvent(newer_event.event_.as_ref());

        // Source: third_party/blink/common/input/web_coalesced_input_event.cc:115-124
        // TODO: hook input trace scroll_deltas trace_id and coalesced_to_trace_id,
        // emitted after the event has been appended.
    }
}
