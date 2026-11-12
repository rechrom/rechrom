// Copyright 2014 The Chromium Authors. BSD-style license.
// Chromium: ui/events/gesture_detection/touch_disposition_gesture_filter.h;
// touch_disposition_gesture_filter.cc:70-132,201-626.
// Packets are recognizer results awaiting the original touch ACK, not another
// host-input queue. Each packet preserves its source event and sequence owner.
use crate::public::common::input::web_gesture_event::WebGestureEvent;
use crate::public::common::input::web_input_event::Type;
use crate::ui::events::event_latency_metadata::EventLatencyMetadata;
use std::collections::{BTreeSet, VecDeque};

// Preserve the original filter-facing packet type path.
pub(super) use super::gesture_event_data_packet::{GesturePacket, GestureSource};

#[derive(Default)]
struct GestureHandlingState {
    start_consumed: bool,
    current_consumed: bool,
    dropped: BTreeSet<Type>,
}
impl GestureHandlingState {
    fn Filter(&mut self, kind: Type) -> bool {
        use Type::*;
        let (start, current, antecedent) = match kind {
            kGestureTapDown | kGestureTapCancel | kGestureShowPress | kGestureLongPress
            | kGestureShortPress => (true, false, None),
            kGestureLongTap | kGestureTapUnconfirmed => (true, true, None),
            kGestureTap | kGestureDoubleTap => (true, true, Some(kGestureTapUnconfirmed)),
            kGestureScrollBegin => (true, false, None),
            kGestureScrollUpdate => (false, true, Some(kGestureScrollBegin)),
            kGestureScrollEnd => (false, false, Some(kGestureScrollBegin)),
            kGestureFlingStart => (false, false, Some(kGestureScrollUpdate)),
            kGestureFlingCancel => (false, false, Some(kGestureFlingStart)),
            kGesturePinchBegin => (true, false, Some(kGestureScrollBegin)),
            kGesturePinchUpdate => (false, true, Some(kGesturePinchBegin)),
            kGesturePinchEnd => (false, false, Some(kGesturePinchBegin)),
            _ => (true, false, None),
        };
        let drop = start && self.start_consumed
            || current && self.current_consumed
            || antecedent.is_some_and(|a| self.dropped.contains(&a));
        if drop {
            self.dropped.insert(kind);
        } else {
            self.dropped.remove(&kind);
        }
        drop
    }
}
#[derive(Default)]
pub(super) struct TouchDispositionGestureFilter {
    sequences: VecDeque<VecDeque<GesturePacket>>,
    state: GestureHandlingState,
    needs_tap_end: bool,
    needs_show_press: bool,
    needs_scroll_end: bool,
    needs_fling_end: bool,
}
impl TouchDispositionGestureFilter {
    // Source touch_disposition_gesture_filter.cc:328-330. Pending packets and
    // synthesized-end obligations retain their original sequence ownership.
    pub fn ResetGestureHandlingState(&mut self) {
        self.state = GestureHandlingState::default();
    }
    pub fn OnGesturePacket(&mut self, packet: GesturePacket) -> Vec<WebGestureEvent> {
        if packet.source == GestureSource::Start {
            self.sequences.push_back(VecDeque::new());
        }
        let Some(tail) = self.sequences.back_mut() else {
            return Vec::new();
        };
        let immediate =
            packet.source == GestureSource::Timeout && tail.is_empty() && self.sequences.len() == 1;
        if immediate {
            return self.FilterAndSendPacket(packet);
        }
        self.sequences.back_mut().unwrap().push_back(packet);
        Vec::new()
    }
    pub fn OnTouchEventAck(
        &mut self,
        id: u32,
        consumed: bool,
        blocking: bool,
        latency: Option<EventLatencyMetadata>,
    ) -> Vec<WebGestureEvent> {
        // Chromium permits the newest packet's nonblocking ACK to arrive before
        // an older blocking ACK. Admission order still controls publication.
        for sequence in &mut self.sequences {
            if let Some(packet) = sequence
                .iter_mut()
                .find(|p| p.source != GestureSource::Timeout && p.event.unique_touch_event_id == id)
            {
                packet.Ack(consumed, blocking, latency);
                break;
            }
        }
        let mut result = Vec::new();
        loop {
            if self.sequences.len() > 1 && self.sequences.front().is_some_and(|q| q.is_empty()) {
                self.sequences.pop_front();
                self.state = GestureHandlingState::default();
            }
            let Some(front) = self.sequences.front_mut() else {
                break;
            };
            let Some(packet) = front.front() else {
                break;
            };
            if packet.source != GestureSource::Timeout && packet.ack.is_none() {
                break;
            }
            let packet = front.pop_front().unwrap();
            if let Some((consumed, _)) = packet.ack {
                self.state.current_consumed = consumed;
                if consumed
                    && matches!(
                        packet.source,
                        GestureSource::Start | GestureSource::PointerDown
                    )
                {
                    self.state.start_consumed = true;
                }
            }
            result.extend(self.FilterAndSendPacket(packet));
        }
        result
    }
    fn Synthetic(
        &mut self,
        kind: Type,
        source: &WebGestureEvent,
        result: &mut Vec<WebGestureEvent>,
    ) {
        let mut g = source.clone();
        g.SetType(kind);
        self.SendGesture(g, result);
    }
    fn CancelTap(&mut self, source: &WebGestureEvent, result: &mut Vec<WebGestureEvent>) {
        if self.needs_tap_end {
            self.Synthetic(Type::kGestureTapCancel, source, result);
        }
    }
    fn EndScroll(&mut self, source: &WebGestureEvent, result: &mut Vec<WebGestureEvent>) {
        if self.needs_scroll_end {
            self.Synthetic(Type::kGestureScrollEnd, source, result);
        }
    }
    fn CancelFling(&mut self, source: &WebGestureEvent, result: &mut Vec<WebGestureEvent>) {
        if self.needs_fling_end {
            self.Synthetic(Type::kGestureFlingCancel, source, result);
        }
    }
    fn FilterAndSendPacket(&mut self, packet: GesturePacket) -> Vec<WebGestureEvent> {
        let mut result = Vec::new();
        if packet.source == GestureSource::Start {
            self.CancelTap(&packet.event, &mut result);
            self.EndScroll(&packet.event, &mut result);
            self.CancelFling(&packet.event, &mut result);
        } else if packet.source == GestureSource::PointerDown {
            self.CancelTap(&packet.event, &mut result);
        }
        for mut gesture in packet.gestures {
            GesturePacket::PrepareGesture(&mut gesture, packet.source, packet.ack, packet.latency);
            if self.state.Filter(gesture.GetType()) {
                self.CancelTap(&packet.event, &mut result);
                continue;
            }
            if gesture.GetType() == Type::kGestureTapCancel {
                self.CancelTap(&packet.event, &mut result);
                continue;
            }
            self.SendGesture(gesture, &mut result);
        }
        if packet.source == GestureSource::Cancel {
            self.EndScroll(&packet.event, &mut result);
            self.CancelTap(&packet.event, &mut result);
        }
        if packet.source == GestureSource::End {
            self.EndScroll(&packet.event, &mut result);
        }
        result
    }
    fn SendGesture(&mut self, event: WebGestureEvent, result: &mut Vec<WebGestureEvent>) {
        use Type::*;
        match event.GetType() {
            kGestureLongTap => {
                if !self.needs_tap_end {
                    return;
                }
                self.CancelTap(&event, result);
                self.CancelFling(&event, result);
            }
            kGestureTapDown => {
                self.needs_show_press = true;
                self.needs_tap_end = true;
            }
            kGestureShowPress => {
                if !self.needs_show_press {
                    return;
                }
                self.needs_show_press = false;
            }
            kGestureDoubleTap => {
                self.CancelTap(&event, result);
                self.needs_show_press = false;
            }
            kGestureTap => {
                if self.needs_show_press {
                    self.Synthetic(kGestureShowPress, &event, result);
                }
                self.needs_tap_end = false;
            }
            kGestureTapCancel => {
                self.needs_show_press = false;
                self.needs_tap_end = false;
            }
            kGestureScrollBegin => {
                self.CancelTap(&event, result);
                self.CancelFling(&event, result);
                self.EndScroll(&event, result);
                self.needs_scroll_end = true;
            }
            kGestureScrollEnd => self.needs_scroll_end = false,
            kGestureFlingStart => {
                self.CancelFling(&event, result);
                self.needs_fling_end = true;
                self.needs_scroll_end = false;
            }
            kGestureFlingCancel => self.needs_fling_end = false,
            _ => {}
        }
        result.push(event);
    }
}
