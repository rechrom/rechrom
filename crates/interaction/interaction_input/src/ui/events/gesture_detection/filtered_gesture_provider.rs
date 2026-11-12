// Copyright 2014 The Chromium Authors. BSD-style license.
// Chromium: ui/events/gesture_detection/filtered_gesture_provider.h:25-126;
// filtered_gesture_provider.cc:43-82,132-151.
use super::gesture_provider::GestureProvider;
pub use super::gesture_provider::GestureProviderConfig;
pub use super::touch_action_filter::TouchAction;
use super::touch_action_filter::TouchActionFilter;
use super::touch_disposition_gesture_filter::{GesturePacket, TouchDispositionGestureFilter};
use crate::base::time::time::TimeTicks;
#[cfg(test)]
use crate::public::common::input::web_gesture_event::GestureDevice;
use crate::public::common::input::web_gesture_event::WebGestureEvent;
use crate::public::common::input::web_input_event::Type;
use crate::public::common::input::web_touch_event::WebTouchEvent;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, Default)]
pub struct TouchHandlingResult {
    pub succeeded: bool,
    pub moved_beyond_slop_region: bool,
}
/// Single per-widget recognizer and source disposition owner. Call admission
/// once per original touch before coalescing, ACK each original ID, and route
/// returned gestures only after releasing the provider borrow/queue lock.
#[derive(Clone, Default)]
pub struct GestureTouchActionFilterHandle(Arc<Mutex<TouchActionFilter>>);
impl GestureTouchActionFilterHandle {
    pub fn FilterGeneratedGesture(&self, event: &mut WebGestureEvent) -> bool {
        self.0.lock().unwrap().Filter(event)
    }
}
pub struct FilteredGestureProvider {
    provider: GestureProvider,
    filter: TouchDispositionGestureFilter,
    action: GestureTouchActionFilterHandle,
    moved_beyond_slop_region: bool,
}
impl Default for FilteredGestureProvider {
    fn default() -> Self {
        Self::new(GestureProviderConfig::default())
    }
}
impl FilteredGestureProvider {
    pub fn new(config: GestureProviderConfig) -> Self {
        Self {
            provider: GestureProvider::new(config),
            filter: TouchDispositionGestureFilter::default(),
            action: GestureTouchActionFilterHandle::default(),
            moved_beyond_slop_region: false,
        }
    }
    /// Called with the target chain's canonical, intersected touch-action before
    /// acknowledging that touch; pending gesture packets remain source ordered.
    pub fn SetTouchAction(&mut self, action: TouchAction) {
        self.action.0.lock().unwrap().SetTouchAction(action);
    }
    pub fn TouchActionFilterHandle(&self) -> GestureTouchActionFilterHandle {
        self.action.clone()
    }
    pub fn OnTouchEvent(&mut self, e: &WebTouchEvent) -> TouchHandlingResult {
        let Some(gestures) = self.provider.OnTouchEvent(e) else {
            return TouchHandlingResult::default();
        };
        if e.IsTouchSequenceStart() {
            self.moved_beyond_slop_region = false;
        }
        self.moved_beyond_slop_region |= gestures
            .iter()
            .any(|g| g.GetType() == Type::kGestureScrollBegin);
        let packet = GesturePacket::FromTouch(e, self.provider.PrimaryTouchId(), gestures);
        self.filter.OnGesturePacket(packet);
        TouchHandlingResult {
            succeeded: true,
            moved_beyond_slop_region: self.moved_beyond_slop_region,
        }
    }
    pub fn OnTouchEventAck(
        &mut self,
        id: u32,
        consumed: bool,
        blocking: bool,
    ) -> Vec<WebGestureEvent> {
        self.OnTouchEventAckWithLatency(id, consumed, blocking, None)
    }
    pub fn OnTouchEventAckWithLatency(
        &mut self,
        id: u32,
        consumed: bool,
        blocking: bool,
        latency: Option<crate::ui::events::event_latency_metadata::EventLatencyMetadata>,
    ) -> Vec<WebGestureEvent> {
        let events = self.filter.OnTouchEventAck(id, consumed, blocking, latency);
        self.ApplyTouchAction(events)
    }
    /// Chromium places TouchActionFilter after the browser FlingController;
    /// generated fling scroll updates reuse this same retained filter owner.
    pub fn FilterGeneratedGesture(&mut self, event: &mut WebGestureEvent) -> bool {
        self.action.FilterGeneratedGesture(event)
    }
    pub fn OnTimeout(&mut self, now: TimeTicks) -> Vec<WebGestureEvent> {
        let mut result = Vec::new();
        for event in self.provider.OnTimeout(now) {
            let packet = GesturePacket::FromTouchTimeout(event);
            result.extend(self.filter.OnGesturePacket(packet));
        }
        self.ApplyTouchAction(result)
    }
    // Source filtered_gesture_provider.cc:78-87,122-132. Reset-generated
    // gestures occur outside touch admission and use timeout packet ordering.
    pub fn ResetDetection(&mut self, now: TimeTicks) -> Vec<WebGestureEvent> {
        let mut result = Vec::new();
        for event in self.provider.ResetDetection(now) {
            result.extend(
                self.filter
                    .OnGesturePacket(GesturePacket::FromTouchTimeout(event)),
            );
        }
        self.ApplyTouchAction(result)
    }
    pub fn ResetGestureHandlingState(&mut self) {
        self.filter.ResetGestureHandlingState();
    }
    /// Document-client detach is the lifetime boundary of the detector and
    /// its outstanding disposition packets. Unlike ResetDetection (which
    /// sends CANCEL to the still-attached client), no old-client gestures may
    /// be delivered here. Keep the injected touch-action handle's identity.
    pub fn ResetForDocumentCommit(&mut self) {
        self.provider = GestureProvider::new(self.provider.config.clone());
        self.filter = TouchDispositionGestureFilter::default();
        *self.action.0.lock().unwrap() = TouchActionFilter::default();
        self.moved_beyond_slop_region = false;
    }
    pub fn NextDeadline(&self) -> Option<TimeTicks> {
        self.provider.NextDeadline()
    }
    fn ApplyTouchAction(&mut self, events: Vec<WebGestureEvent>) -> Vec<WebGestureEvent> {
        events
            .into_iter()
            .filter_map(|mut event| {
                (!self.action.FilterGeneratedGesture(&mut event)).then_some(event)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::public::common::input::web_touch_point::TouchState;
    fn touch(kind: Type, id: u32, time: i64, x: f32) -> WebTouchEvent {
        let mut e = WebTouchEvent::new(kind, 0, TimeTicks::FromInternalValue(time));
        e.unique_touch_event_id = id;
        e.touches_length = 1;
        let p = &mut e.touches[0];
        p.state = match kind {
            Type::kTouchStart => TouchState::kStatePressed,
            Type::kTouchEnd => TouchState::kStateReleased,
            _ => TouchState::kStateMoved,
        };
        p.pointer_properties.id = 7;
        p.pointer_properties.SetPositionInWidgetXY(x, 20.);
        p.radius_x = 5.;
        p.radius_y = 5.;
        e
    }
    #[test]
    fn admission_ack_and_timeout_keep_source_order_and_identity() {
        let mut p = FilteredGestureProvider::default();
        let down = touch(Type::kTouchStart, 11, 1_000_000, 10.);
        let moved = touch(Type::kTouchMove, 12, 1_020_000, 40.);
        assert!(p.OnTouchEvent(&down).succeeded);
        assert!(p.OnTouchEvent(&moved).moved_beyond_slop_region);
        assert!(p.OnTouchEventAck(12, false, true).is_empty());
        let events = p.OnTouchEventAck(11, false, true);
        assert_eq!(
            events.iter().map(|e| e.GetType()).collect::<Vec<_>>(),
            vec![
                Type::kGestureTapDown,
                Type::kGestureTapCancel,
                Type::kGestureScrollBegin,
                Type::kGestureScrollUpdate
            ]
        );
        let update = events.last().unwrap();
        assert_eq!(
            (
                update.primary_unique_touch_event_id,
                update.unique_touch_event_id
            ),
            (11, 12)
        );
        assert_eq!(update.TimeStamp(), moved.TimeStamp());
        assert_eq!(update.data.scroll_update.delta_x, 15.);
        assert!(p.NextDeadline().is_none());

        let mut p = FilteredGestureProvider::default();
        p.OnTouchEvent(&down);
        assert!(p
            .OnTimeout(TimeTicks::FromInternalValue(1_650_000))
            .is_empty());
        let events = p.OnTouchEventAck(11, false, true);
        assert_eq!(
            events.iter().map(|e| e.GetType()).collect::<Vec<_>>(),
            vec![
                Type::kGestureTapDown,
                Type::kGestureShowPress,
                Type::kGestureShortPress,
                Type::kGestureLongPress
            ]
        );
        assert!(events.iter().all(|e| e.unique_touch_event_id == 11));

        // Navigation must cancel an armed press without replacing the action
        // owner already injected into the independent impl fling sequence.
        let mut p = FilteredGestureProvider::default();
        p.OnTouchEvent(&down);
        p.OnTouchEventAck(11, false, false);
        p.SetTouchAction(TouchAction::kNone);
        let action = p.TouchActionFilterHandle();
        p.ResetDetection(TimeTicks::FromInternalValue(1_050_000));
        assert!(p.NextDeadline().is_none());
        assert!(p
            .OnTimeout(TimeTicks::FromInternalValue(2_000_000))
            .is_empty());
        let mut begin = WebGestureEvent::new(
            Type::kGestureScrollBegin,
            0,
            down.TimeStamp(),
            GestureDevice::kTouchscreen,
        );
        begin.data.scroll_begin.pointer_count = 1;
        begin.data.scroll_begin.delta_y_hint = 1.;
        assert!(action.FilterGeneratedGesture(&mut begin));
    }
    #[test]
    fn consumed_start_and_touch_action_suppress_only_the_source_gestures() {
        let down = touch(Type::kTouchStart, 11, 1_000_000, 10.);
        let moved = touch(Type::kTouchMove, 12, 1_020_000, 40.);
        let up = touch(Type::kTouchEnd, 13, 1_030_000, 40.);
        let mut p = FilteredGestureProvider::default();
        for e in [&down, &moved, &up] {
            assert!(p.OnTouchEvent(e).succeeded);
        }
        assert!(p.OnTouchEventAck(11, true, true).is_empty());
        assert!(p.OnTouchEventAck(12, false, true).is_empty());
        assert!(p.OnTouchEventAck(13, false, true).is_empty());
        let mut p = FilteredGestureProvider::default();
        p.SetTouchAction(TouchAction::kPanY);
        p.OnTouchEvent(&down);
        p.OnTouchEventAck(11, false, true);
        p.OnTouchEvent(&moved);
        let events = p.OnTouchEventAck(12, false, true);
        assert_eq!(
            events.iter().map(|e| e.GetType()).collect::<Vec<_>>(),
            vec![Type::kGestureTapCancel]
        );
    }
    #[test]
    fn document_detach_drops_pending_ack_and_timeout_but_retains_action_owner() {
        let mut provider = FilteredGestureProvider::default();
        let action = provider.TouchActionFilterHandle();
        provider.SetTouchAction(TouchAction::kNone);
        assert!(
            provider
                .OnTouchEvent(&touch(Type::kTouchStart, 11, 1_000_000, 10.))
                .succeeded
        );
        assert!(provider
            .OnTimeout(TimeTicks::FromInternalValue(1_600_000))
            .is_empty());
        provider.ResetForDocumentCommit();
        assert!(Arc::ptr_eq(
            &action.0,
            &provider.TouchActionFilterHandle().0
        ));
        assert!(provider.NextDeadline().is_none());
        assert!(provider.OnTouchEventAck(11, false, true).is_empty());
        assert!(provider
            .OnTimeout(TimeTicks::FromInternalValue(2_000_000))
            .is_empty());
        provider.OnTouchEvent(&touch(Type::kTouchStart, 22, 2_000_000, 20.));
        let events = provider.OnTouchEventAck(22, false, true);
        assert_eq!(events.len(), 1);
        assert_eq!(
            (events[0].GetType(), events[0].unique_touch_event_id),
            (Type::kGestureTapDown, 22)
        );
        let mut begin = WebGestureEvent::new(
            Type::kGestureScrollBegin,
            0,
            TimeTicks::FromInternalValue(2_020_000),
            GestureDevice::kTouchscreen,
        );
        begin.data.scroll_begin.pointer_count = 1;
        begin.data.scroll_begin.delta_x_hint = 1.;
        assert!(!action.FilterGeneratedGesture(&mut begin));
    }
}
