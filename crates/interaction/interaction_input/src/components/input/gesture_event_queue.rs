// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Source: components/input/gesture_event_queue.{h,cc}.
// The embedding supplies monotonic deadlines and transports the returned
// dispatches/ACKs. Callback values move with deferred events; no second raw
// input queue or gesture recognizer is owned here.
use super::fling_controller::{Config as FlingConfig, FlingController, FlingControllerOutput};
use crate::{
    base::time::time::TimeTicks,
    public::{
        common::input::{web_gesture_event::WebGestureEvent, web_input_event::Type},
        mojom::input::input_event_result::{InputEventResultSource, InputEventResultState},
    },
};
use std::collections::VecDeque;

#[derive(Clone, Copy, Default)]
pub struct Config {
    pub fling_config: FlingConfig,
    /// Zero disables debounce, as in GestureEventQueue::Config.
    pub debounce_interval_us: i64,
}

pub struct GestureEventWithCallback<Callback> {
    pub event: WebGestureEvent,
    pub dispatch_callback: Callback,
}
pub struct ForwardedGesture<Callback> {
    /// Platform-neutral identity retained across renderer/main-thread deferral.
    pub dispatch_id: u64,
    pub event: WebGestureEvent,
    pub dispatch_callback: Callback,
}
#[derive(Clone, Debug)]
pub struct GestureEventAck {
    pub dispatch_id: u64,
    pub event: WebGestureEvent,
    pub source: InputEventResultSource,
    pub result: InputEventResultState,
}
struct OutstandingGesture {
    dispatch_id: u64,
    event: WebGestureEvent,
    ack: Option<(InputEventResultSource, InputEventResultState)>,
}
/// Typed client calls preserve their source order on debounce expiration.
pub enum DebounceEffect<Callback> {
    Fling(FlingControllerOutput),
    Forward(ForwardedGesture<Callback>),
    NotDispatched(GestureEventWithCallback<Callback>),
}

pub struct GestureEventQueue<Callback = ()> {
    fling_controller: FlingController,
    fling_config: FlingConfig,
    debounce_interval_us: i64,
    scrolling_in_progress: bool,
    scroll_end_filtered_by_debouncing_deferral_queue: bool,
    debounce_deadline: Option<TimeTicks>,
    debouncing_deferral_queue: VecDeque<GestureEventWithCallback<Callback>>,
    deferred_gesture_queue: VecDeque<GestureEventWithCallback<Callback>>,
    sent_events_awaiting_ack: VecDeque<OutstandingGesture>,
    next_dispatch_id: u64,
}
impl<Callback> Default for GestureEventQueue<Callback> {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
impl<Callback> GestureEventQueue<Callback> {
    pub fn new(config: Config) -> Self {
        Self {
            fling_controller: FlingController::new(config.fling_config),
            fling_config: config.fling_config,
            debounce_interval_us: config.debounce_interval_us,
            scrolling_in_progress: false,
            scroll_end_filtered_by_debouncing_deferral_queue: false,
            debounce_deadline: None,
            debouncing_deferral_queue: VecDeque::new(),
            deferred_gesture_queue: VecDeque::new(),
            sent_events_awaiting_ack: VecDeque::new(),
            next_dispatch_id: 0,
        }
    }
    /// Render-widget document replacement retires every old target. Drop the
    /// curve directly: StopFling generates a synthetic WheelEnd, which must
    /// never be dispatched into the replacement document. Keep dispatch ids
    /// monotonic so a late renderer response cannot acknowledge a new event.
    pub fn DocumentDetached(&mut self) -> Vec<GestureEventAck> {
        self.fling_controller = FlingController::new(self.fling_config);
        self.debounce_deadline = None;
        self.scrolling_in_progress = false;
        self.scroll_end_filtered_by_debouncing_deferral_queue = false;
        self.debouncing_deferral_queue.clear();
        self.deferred_gesture_queue.clear();
        self.sent_events_awaiting_ack
            .drain(..)
            .map(|event| {
                let (source, result) = event.ack.unwrap_or((
                    InputEventResultSource::kBrowser,
                    InputEventResultState::kNoConsumerExists,
                ));
                GestureEventAck {
                    dispatch_id: event.dispatch_id,
                    event: event.event,
                    source,
                    result,
                }
            })
            .collect()
    }
    pub fn empty(&self) -> bool {
        self.sent_events_awaiting_ack.is_empty() && self.debouncing_deferral_queue.is_empty()
    }
    pub fn outstanding_count(&self) -> usize {
        self.sent_events_awaiting_ack.len()
    }
    pub fn NextDebounceDeadline(&self) -> Option<TimeTicks> {
        self.debounce_deadline
    }
    pub fn fling_in_progress(&self) -> bool {
        self.fling_controller.fling_in_progress()
    }
    pub fn CurrentFlingVelocity(&self) -> foundation::gfx_geometry::Vector2dF {
        self.fling_controller.CurrentFlingVelocity()
    }
    pub fn OnWheelEventAck(&mut self, result: InputEventResultState) {
        self.fling_controller.OnWheelEventAck(result);
    }
    pub fn ShouldSuppressMouseDown(&mut self, now: TimeTicks) -> bool {
        self.fling_controller.ShouldSuppressMouseDown(now)
    }
    pub fn ShouldSuppressMouseUp(&self) -> bool {
        self.fling_controller.ShouldSuppressMouseUp()
    }
    pub fn StopFling(&mut self, now: TimeTicks) -> FlingControllerOutput {
        self.fling_controller.StopFling(now)
    }
    pub fn ProgressFling(
        &mut self,
        now: TimeTicks,
        first_frame: Option<TimeTicks>,
    ) -> FlingControllerOutput {
        self.fling_controller.ProgressFling(now, first_frame)
    }
    pub fn PassToFlingController(
        &mut self,
        event: &WebGestureEvent,
        now: TimeTicks,
        viewport_empty: bool,
    ) -> FlingControllerOutput {
        self.fling_controller
            .ObserveAndMaybeConsumeGestureEvent(event, now, viewport_empty)
    }
    pub fn QueueDeferredEvents(&mut self, event: WebGestureEvent, dispatch_callback: Callback) {
        self.deferred_gesture_queue
            .push_back(GestureEventWithCallback {
                event,
                dispatch_callback,
            });
    }
    pub fn TakeDeferredEvents(&mut self) -> VecDeque<GestureEventWithCallback<Callback>> {
        std::mem::take(&mut self.deferred_gesture_queue)
    }
    pub fn DebounceOrForwardEvent(
        &mut self,
        event: WebGestureEvent,
        dispatch_callback: Callback,
        now: TimeTicks,
    ) -> Option<ForwardedGesture<Callback>> {
        assert!(
            !matches!(
                event.GetType(),
                Type::kGestureFlingStart | Type::kGestureFlingCancel
            ),
            "fling events must first pass the fling controller"
        );
        if self.ShouldForwardForBounceReduction(&event, now) {
            Some(self.ForwardGestureEvent(event, dispatch_callback))
        } else {
            self.debouncing_deferral_queue
                .push_back(GestureEventWithCallback {
                    event,
                    dispatch_callback,
                });
            None
        }
    }
    fn ShouldForwardForBounceReduction(&mut self, event: &WebGestureEvent, now: TimeTicks) -> bool {
        if self.debounce_interval_us <= 0 {
            return true;
        }
        if event.GetType() == Type::kGestureScrollEnd {
            self.scroll_end_filtered_by_debouncing_deferral_queue = false;
        }
        if self.fling_controller.fling_in_progress() {
            return true;
        }
        match event.GetType() {
            Type::kGestureScrollUpdate => {
                self.debounce_deadline = Some(TimeTicks::FromInternalValue(
                    now.ToInternalValue()
                        .saturating_add(self.debounce_interval_us),
                ));
                self.scrolling_in_progress = true;
                // Chromium clears these once-callback owners on a scroll bounce;
                // they are destroyed, rather than dispatched or renderer-ACK'd.
                self.debouncing_deferral_queue.clear();
                true
            }
            Type::kGesturePinchBegin | Type::kGesturePinchUpdate | Type::kGesturePinchEnd => true,
            Type::kGestureScrollBegin => !self.scroll_end_filtered_by_debouncing_deferral_queue,
            Type::kGestureScrollEnd => {
                if event.data.scroll_end.generated_by_fling_controller {
                    self.scrolling_in_progress = false;
                    true
                } else if self.scrolling_in_progress {
                    self.scroll_end_filtered_by_debouncing_deferral_queue = true;
                    false
                } else {
                    true
                }
            }
            _ => !self.scrolling_in_progress,
        }
    }
    pub fn ForwardGestureEvent(
        &mut self,
        event: WebGestureEvent,
        dispatch_callback: Callback,
    ) -> ForwardedGesture<Callback> {
        assert!(!matches!(
            event.GetType(),
            Type::kGestureFlingStart | Type::kGestureFlingCancel
        ));
        self.next_dispatch_id = self
            .next_dispatch_id
            .checked_add(1)
            .expect("gesture dispatch identity exhausted");
        let dispatch_id = self.next_dispatch_id;
        self.sent_events_awaiting_ack.push_back(OutstandingGesture {
            dispatch_id,
            event: event.clone(),
            ack: None,
        });
        ForwardedGesture {
            dispatch_id,
            event,
            dispatch_callback,
        }
    }
    /// Chromium matches the first unacknowledged event of this type. The
    /// transport also supports an exact id so same-type delayed responses cannot
    /// accidentally complete another original event.
    pub fn ProcessGestureAckByType(
        &mut self,
        source: InputEventResultSource,
        result: InputEventResultState,
        event_type: Type,
    ) -> Vec<GestureEventAck> {
        let id = self
            .sent_events_awaiting_ack
            .iter()
            .find(|event| event.ack.is_none() && event.event.GetType() == event_type)
            .map(|event| event.dispatch_id);
        id.map_or_else(Vec::new, |id| self.ProcessGestureAck(id, source, result))
    }
    pub fn ProcessGestureAck(
        &mut self,
        dispatch_id: u64,
        source: InputEventResultSource,
        result: InputEventResultState,
    ) -> Vec<GestureEventAck> {
        if result == InputEventResultState::kUnknown {
            return Vec::new();
        }
        if let Some(event) = self
            .sent_events_awaiting_ack
            .iter_mut()
            .find(|event| event.dispatch_id == dispatch_id && event.ack.is_none())
        {
            event.ack = Some((source, result));
        }
        let mut completed = Vec::new();
        // Drain before returning client calls. No callback can reenter this
        // owner while its outstanding FIFO is being mutated.
        while self
            .sent_events_awaiting_ack
            .front()
            .is_some_and(|event| event.ack.is_some())
        {
            let event = self.sent_events_awaiting_ack.pop_front().unwrap();
            let (source, result) = event.ack.unwrap();
            completed.push(GestureEventAck {
                dispatch_id: event.dispatch_id,
                event: event.event,
                source,
                result,
            });
        }
        completed
    }
    pub fn ExpireDebounce(
        &mut self,
        now: TimeTicks,
        viewport_empty: bool,
    ) -> Vec<DebounceEffect<Callback>> {
        if !self
            .debounce_deadline
            .is_some_and(|deadline| now >= deadline)
        {
            return Vec::new();
        }
        self.debounce_deadline = None;
        self.scrolling_in_progress = false;
        let mut output = Vec::new();
        for deferred in std::mem::take(&mut self.debouncing_deferral_queue) {
            let fling = self.PassToFlingController(&deferred.event, now, viewport_empty);
            let consumed = fling.consumed;
            output.push(DebounceEffect::Fling(fling));
            if consumed {
                output.push(DebounceEffect::NotDispatched(deferred));
            } else {
                if deferred.event.GetType() == Type::kGestureScrollEnd {
                    self.scroll_end_filtered_by_debouncing_deferral_queue = false;
                }
                output.push(DebounceEffect::Forward(
                    self.ForwardGestureEvent(deferred.event, deferred.dispatch_callback),
                ));
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::public::common::input::web_gesture_event::GestureDevice;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    fn gesture(kind: Type) -> WebGestureEvent {
        WebGestureEvent::new(
            kind,
            0,
            TimeTicks::FromInternalValue(1),
            GestureDevice::kTouchscreen,
        )
    }
    #[test]
    fn renderer_acks_preserve_dispatch_order_and_exact_same_type_identity() {
        let mut queue = GestureEventQueue::default();
        let begin = queue.ForwardGestureEvent(gesture(Type::kGestureScrollBegin), ());
        let first = queue.ForwardGestureEvent(gesture(Type::kGestureScrollUpdate), ());
        let second = queue.ForwardGestureEvent(gesture(Type::kGestureScrollUpdate), ());
        assert!(queue
            .ProcessGestureAck(
                second.dispatch_id,
                InputEventResultSource::kCompositorThread,
                InputEventResultState::kConsumed
            )
            .is_empty());
        assert!(queue
            .ProcessGestureAck(
                first.dispatch_id,
                InputEventResultSource::kMainThread,
                InputEventResultState::kNotConsumed
            )
            .is_empty());
        let completed = queue.ProcessGestureAck(
            begin.dispatch_id,
            InputEventResultSource::kCompositorThread,
            InputEventResultState::kConsumed,
        );
        assert_eq!(
            completed
                .iter()
                .map(|event| event.dispatch_id)
                .collect::<Vec<_>>(),
            vec![begin.dispatch_id, first.dispatch_id, second.dispatch_id]
        );
        assert_eq!(completed[1].source, InputEventResultSource::kMainThread);
        assert_eq!(completed[1].result, InputEventResultState::kNotConsumed);
        assert!(queue.empty());
        assert!(queue
            .ProcessGestureAck(
                first.dispatch_id,
                InputEventResultSource::kMainThread,
                InputEventResultState::kConsumed
            )
            .is_empty());
    }
    struct Callback(Arc<AtomicUsize>);
    impl Drop for Callback {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    #[test]
    fn scroll_bounces_drop_deferred_callbacks_and_timer_releases_in_order() {
        let mut queue = GestureEventQueue::new(Config {
            debounce_interval_us: 100,
            ..Config::default()
        });
        let dropped = Arc::new(AtomicUsize::new(0));
        let callback = || Callback(dropped.clone());
        drop(queue.DebounceOrForwardEvent(
            gesture(Type::kGestureScrollUpdate),
            callback(),
            TimeTicks::FromInternalValue(1),
        ));
        assert!(queue
            .DebounceOrForwardEvent(
                gesture(Type::kGestureScrollEnd),
                callback(),
                TimeTicks::FromInternalValue(2)
            )
            .is_none());
        assert!(queue
            .DebounceOrForwardEvent(
                gesture(Type::kGestureScrollBegin),
                callback(),
                TimeTicks::FromInternalValue(3)
            )
            .is_none());
        assert_eq!(dropped.load(Ordering::SeqCst), 1);
        drop(queue.DebounceOrForwardEvent(
            gesture(Type::kGestureScrollUpdate),
            callback(),
            TimeTicks::FromInternalValue(4),
        ));
        assert_eq!(dropped.load(Ordering::SeqCst), 4);
        assert!(queue
            .DebounceOrForwardEvent(
                gesture(Type::kGestureScrollEnd),
                callback(),
                TimeTicks::FromInternalValue(5)
            )
            .is_none());
        assert!(queue
            .DebounceOrForwardEvent(
                gesture(Type::kGestureScrollBegin),
                callback(),
                TimeTicks::FromInternalValue(6)
            )
            .is_none());
        assert!(queue
            .ExpireDebounce(TimeTicks::FromInternalValue(103), false)
            .is_empty());
        let effects = queue.ExpireDebounce(TimeTicks::FromInternalValue(104), false);
        let types = effects
            .iter()
            .filter_map(|effect| match effect {
                DebounceEffect::Forward(event) => Some(event.event.GetType()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            types,
            vec![Type::kGestureScrollEnd, Type::kGestureScrollBegin]
        );
        assert_eq!(dropped.load(Ordering::SeqCst), 4);
        drop(effects);
        assert_eq!(dropped.load(Ordering::SeqCst), 6);
        assert!(queue.NextDebounceDeadline().is_none());
    }
}
