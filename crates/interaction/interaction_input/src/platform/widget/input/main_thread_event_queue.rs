// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.h:1-272
// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:1-1103
// Rust modules replace includes/namespaces; Arc/Mutex and relaxed atomics
// preserve the source ref-counted owner and its three synchronization domains.

use std::any::Any;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use super::main_thread_event_queue_task::{FilterResult, MainThreadEventQueueTask};
use super::main_thread_event_queue_task_list::{EnqueueResult, MainThreadEventQueueTaskList};
use crate::base::time::time::TimeTicks;
pub use crate::cc::metrics::begin_main_frame_metrics::BeginMainFrameReason;
pub use crate::public::common::features::AsyncTouchMoveThrottlingPolicy;
use crate::public::common::input::web_coalesced_input_event::WebCoalescedInputEvent;
use crate::public::common::input::web_input_event::{
    DispatchType as EventDispatchType, Type, WebInputEvent, WebInputEventPayload,
};
use crate::public::common::input::web_mouse_event::WebMouseEvent;
use crate::public::common::input::web_mouse_wheel_event::WebMouseWheelEvent;
use crate::public::common::input::web_pointer_event::{TouchState, WebPointerEvent};
use crate::public::common::input::web_touch_event::WebTouchEvent;
pub use crate::public::mojom::input::input_event_result::InputEventResultState;
use crate::ui::events::types::event_type::EventType as UiEventType;

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:357-361,833-835,1081-1093
// The embedding resolves Chromium feature values before constructing the
// queue. These values carry no scheduling decision or clock implementation.
#[derive(Clone, Copy, Debug)]
pub struct MainThreadEventQueueFeatures {
    pub resampling_input_events: bool,
    pub eligible_for_throttle_main_frame_to_60hz: bool,
    pub urgent_main_frame_for_input: bool,
    pub unthrottle_async_touch_moves: bool,
    pub async_touch_move_throttling_policy: AsyncTouchMoveThrottlingPolicy,
}

// Rust ports for the source's injected platform services and opaque owners.
// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.h:14-26,34-38,104-109,242-250
pub trait QueueLatencyInfo: Clone + Send + Sync + 'static {
    // Clone must copy the source latency value, including its component state;
    // it may not turn a copied ui::LatencyInfo into a second mutable alias.
    fn trace_id(&self) -> i64;
    fn set_trace_id(&mut self, id: i64);
    fn set_coalesced(&mut self);
}

pub trait QueueEventMetrics: Send + 'static {
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:91-95
    // Calls EventMetrics::CreateFromExisting with the source's
    // kRendererCompositorFinished dispatch stage.
    fn CreateFromExistingForRawEvent(
        event_type: UiEventType,
        original: Option<&Self>,
    ) -> Option<Self>
    where
        Self: Sized;
    fn IsScrollUpdate(&self) -> bool;
    fn CoalesceWith(&mut self, other: &Self);
}

pub trait QueueInputPrediction<L: QueueLatencyInfo>: Send {
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:360-361,885-888
    fn SetResamplingEnabled(&mut self, enabled: bool);
    fn HandleEvents(&mut self, event: &mut WebCoalescedInputEvent<L>, frame_time: TimeTicks);
}

pub trait QueueOneShotTimer: Send {
    // The injected timer is bound to the same main runner as the queue. Start
    // schedules a single callback; Stop and Drop invalidate that callback.
    fn Start(&mut self, delay_microseconds: i64, callback: Box<dyn FnOnce() + Send>);
    fn Stop(&mut self);
}

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:380-383,468-510,543-554,693-696,897-901,987-1000
// These are borrowed projections of input payloads owned by this input crate.
// They carry the source's fields and constructors while concrete touch and
// gesture translation is admitted. Queue policy remains in this module.
pub trait QueuePayloadAccess: Send + Sync + 'static {
    fn TouchEvent(event: &dyn WebInputEventPayload) -> &WebTouchEvent {
        event
            .AsAny()
            .downcast_ref::<WebTouchEvent>()
            .expect("touch payload")
    }
    fn TouchEventMut(event: &mut dyn WebInputEventPayload) -> &mut WebTouchEvent {
        event
            .AsAnyMut()
            .downcast_mut::<WebTouchEvent>()
            .expect("touch payload")
    }
    fn PointerFromMouse(event: &dyn WebInputEventPayload) -> WebPointerEvent {
        WebPointerEvent::FromMouse(
            Type::kPointerRawUpdate,
            event
                .AsAny()
                .downcast_ref::<WebMouseEvent>()
                .expect("mouse payload"),
        )
    }
    // Preserve embedding-only attribution around the source-generated raw
    // payload. This hook does not select, coalesce, schedule or dispatch input.
    fn WrapPointerRawUpdate(
        _: &dyn WebInputEventPayload,
        pointer: WebPointerEvent,
    ) -> Box<dyn WebInputEventPayload> {
        Box::new(pointer)
    }
    fn WheelDispatchType(event: &dyn WebInputEventPayload) -> EventDispatchType {
        event
            .AsAny()
            .downcast_ref::<WebMouseWheelEvent>()
            .expect("wheel payload")
            .dispatch_type
    }
    fn SetWheelDispatchType(event: &mut dyn WebInputEventPayload, value: EventDispatchType) {
        event
            .AsAnyMut()
            .downcast_mut::<WebMouseWheelEvent>()
            .expect("wheel payload")
            .dispatch_type = value;
    }
    fn GestureScrollBeginCursorControl(event: &dyn WebInputEventPayload) -> bool;
    fn TouchDispatchType(event: &dyn WebInputEventPayload) -> EventDispatchType {
        Self::TouchEvent(event).dispatch_type
    }
    fn SetTouchDispatchType(event: &mut dyn WebInputEventPayload, value: EventDispatchType) {
        Self::TouchEventMut(event).dispatch_type = value;
    }
    fn TouchStartOrFirstTouchMove(event: &dyn WebInputEventPayload) -> bool {
        Self::TouchEvent(event).touch_start_or_first_touch_move
    }
    fn IsTouchSequenceStart(event: &dyn WebInputEventPayload) -> bool {
        Self::TouchEvent(event).IsTouchSequenceStart()
    }
    fn TouchMovedBeyondSlopRegion(event: &dyn WebInputEventPayload) -> bool {
        Self::TouchEvent(event).moved_beyond_slop_region
    }
    fn TouchesLength(event: &dyn WebInputEventPayload) -> usize {
        Self::TouchEvent(event).touches_length
    }
    fn TouchPointState(event: &dyn WebInputEventPayload, index: usize) -> TouchState {
        Self::TouchEvent(event).touches[index].state
    }
    // Calls the source WebPointerEvent(touch_event, touch_point) constructor.
    fn PointerFromTouch(event: &dyn WebInputEventPayload, index: usize) -> WebPointerEvent {
        let event = Self::TouchEvent(event);
        WebPointerEvent::FromTouch(event, &event.touches[index])
    }
}

pub trait MainThreadEventQueueServices: Send + Sync + Sized + 'static {
    type Latency: QueueLatencyInfo;
    type Attribution: Clone + Default + Send + Sync + 'static;
    type Metrics: QueueEventMetrics;
    type Overscroll: Send + 'static;
    type TouchAction: Copy + Send + 'static;
    type PayloadAccess: QueuePayloadAccess;

    fn Now(&self) -> TimeTicks;
    fn MainThreadBelongsToCurrentThread(&self) -> bool;
    fn CompositorThreadBelongsToCurrentThread(&self) -> bool;
    // Preserve SingleThreadTaskRunner::PostTask: enqueue, never invoke inline.
    fn PostTaskToMainThread(&self, task: Box<dyn FnOnce() + Send>);
    fn WillPostInputEventToMainThread(&self, event_type: Type, attribution: &Self::Attribution);
    fn WillHandleInputEventOnMainThread(&self, event_type: Type, attribution: &Self::Attribution);
    fn DidHandleInputEventOnMainThread(
        &self,
        event: &WebInputEvent,
        handled_application: bool,
        requested_main_frame_pending: bool,
    );
}

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.h:34-38
pub type HandledEventCallback<B> = Box<
    dyn FnOnce(
            InputEventResultState,
            &<B as MainThreadEventQueueServices>::Latency,
            Option<<B as MainThreadEventQueueServices>::Overscroll>,
            Option<<B as MainThreadEventQueueServices>::TouchAction>,
        ) + Send,
>;

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.h:40-64
pub trait MainThreadEventQueueClient<B: MainThreadEventQueueServices>: Send + Sync {
    fn HandleInputEvent(
        &self,
        event: &WebCoalescedInputEvent<B::Latency>,
        metrics: Option<B::Metrics>,
        handled_callback: HandledEventCallback<B>,
    ) -> bool;
    fn InputEventsDispatched(&self, raf_aligned: bool);
    fn SetNeedsMainFrame(&self, reason: BeginMainFrameReason, urgent: bool);
    fn RequestedMainFramePending(&self) -> bool;
}

type QueueOwner<B> = Arc<MainThreadEventQueue<B>>;
type QueueTask<B> = dyn MainThreadEventQueueTask<QueueOwner<B>>;

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:30
const kMaxRafDelay: i64 = 5_000_000;

// Source: main_thread_event_queue.cc:861-880. Shared value-only admission rule.
pub fn IsRafAlignedInputEvent(
    event_type: Type,
    allow_raf_aligned_input: bool,
    needs_low_latency: bool,
    needs_low_latency_until_pointer_up: bool,
    needs_unbuffered_input_for_debugger: bool,
) -> bool {
    matches!(
        event_type,
        Type::kMouseMove | Type::kMouseWheel | Type::kTouchMove
    ) && allow_raf_aligned_input
        && !needs_low_latency
        && !needs_low_latency_until_pointer_up
        && !needs_unbuffered_input_for_debugger
}

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:32-49
struct QueuedClosure(Option<Box<dyn FnOnce() + Send>>);

impl<B: MainThreadEventQueueServices> MainThreadEventQueueTask<QueueOwner<B>> for QueuedClosure {
    fn FilterNewEvent(&mut self, other: &mut QueueTask<B>) -> FilterResult {
        if other.IsWebInputEvent() {
            FilterResult::KeepIterating
        } else {
            FilterResult::StopIterating
        }
    }
    fn IsWebInputEvent(&self) -> bool {
        false
    }
    fn Dispatch(&mut self, _queue: &mut QueueOwner<B>) {
        self.0.take().expect("closure dispatch is single-shot")();
    }
    fn AsAny(&self) -> &dyn Any {
        self
    }
    fn AsAnyMut(&mut self) -> &mut dyn Any {
        self
    }
}

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:51-53
const kAsyncTouchMoveInterval: i64 = 200_000;

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:55-64
fn IsGestureScroll(event_type: Type) -> bool {
    matches!(
        event_type,
        Type::kGestureScrollBegin | Type::kGestureScrollUpdate | Type::kGestureScrollEnd
    )
}

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:228-231
struct CallbackInfo<B: MainThreadEventQueueServices> {
    callback: HandledEventCallback<B>,
    latency_info: B::Latency,
}

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:324-327,336
struct CallbackState<B: MainThreadEventQueueServices> {
    callback: Option<HandledEventCallback<B>>,
    blocking_coalesced_callbacks: VecDeque<(HandledEventCallback<B>, i64)>,
}

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:68-83,322-343
struct QueuedWebInputEvent<B: MainThreadEventQueueServices> {
    // Before dispatch this Arc has one owner. The handled callback retains the
    // same frozen event owner, expressing source Unretained(this)+RetainedRef
    // without a pointer into a task that Rust would otherwise drop.
    event_: Arc<WebCoalescedInputEvent<B::Latency>>,
    originally_cancelable_: bool,
    callbacks_: Arc<Mutex<CallbackState<B>>>,
    known_by_scheduler_count_: usize,
    attribution_: B::Attribution,
    metrics_: Option<B::Metrics>,
}

impl<B: MainThreadEventQueueServices> QueuedWebInputEvent<B> {
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:70-83
    fn new(
        event: WebCoalescedInputEvent<B::Latency>,
        originally_cancelable: bool,
        callback: Option<HandledEventCallback<B>>,
        known_by_scheduler: bool,
        attribution: B::Attribution,
        metrics: Option<B::Metrics>,
    ) -> Self {
        Self {
            event_: Arc::new(event),
            originally_cancelable_: originally_cancelable,
            callbacks_: Arc::new(Mutex::new(CallbackState {
                callback,
                blocking_coalesced_callbacks: VecDeque::new(),
            })),
            known_by_scheduler_count_: usize::from(known_by_scheduler),
            attribution_: attribution,
            metrics_: metrics,
        }
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:85-99
    fn CreateForRawEvent(
        event: WebCoalescedInputEvent<B::Latency>,
        attribution: B::Attribution,
        original_metrics: Option<&B::Metrics>,
    ) -> Self {
        debug_assert_eq!(event.Event().Event().GetType(), Type::kPointerRawUpdate);
        let metrics = B::Metrics::CreateFromExistingForRawEvent(
            event.Event().Event().GetTypeAsUiEventType(),
            original_metrics,
        );
        Self::new(event, false, None, false, attribution, metrics)
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:101-120
    fn AreCoalescablePointerRawUpdateEvents(&self, other: &Self) -> bool {
        debug_assert!(
            self.Event().GetType() != Type::kPointerMove
                && other.Event().GetType() != Type::kPointerMove
        );
        self.Event().GetType() == Type::kPointerRawUpdate
            && other.Event().GetType() == Type::kPointerRawUpdate
            && (self.Event().GetModifiers() & WebInputEvent::kRelativeMotionEvent)
                == (other.Event().GetModifiers() & WebInputEvent::kRelativeMotionEvent)
    }

    // Source: main_thread_event_queue.cc:122-169.
    fn FilterNewQueuedEvent(&mut self, other: &mut Self) -> FilterResult {
        if other.Event().GetType() == Type::kTouchScrollStarted {
            return self.HandleTouchScrollStartQueued();
        }
        if !self.Event().IsSameEventClass(other.Event()) {
            return FilterResult::KeepIterating;
        }
        if !self.event_.CanCoalesceWith(&other.event_) {
            return if self.AreCoalescablePointerRawUpdateEvents(other) {
                FilterResult::KeepIterating
            } else {
                FilterResult::StopIterating
            };
        }
        if let Some(callback) = other.callbacks_.lock().unwrap().callback.take() {
            self.callbacks_
                .lock()
                .unwrap()
                .blocking_coalesced_callbacks
                .push_back((callback, other.event_.latency_info().trace_id()));
        }
        self.known_by_scheduler_count_ += other.known_by_scheduler_count_;
        self.mutable_coalesced_event().CoalesceWith(&other.event_);
        if let (Some(metrics), Some(other_metrics)) = (&mut self.metrics_, &other.metrics_) {
            if metrics.IsScrollUpdate() && other_metrics.IsScrollUpdate() {
                metrics.CoalesceWith(other_metrics);
            }
        }
        self.originally_cancelable_ |= other.originally_cancelable_;
        FilterResult::CoalescedEvent
    }
}

impl<B: MainThreadEventQueueServices> MainThreadEventQueueTask<QueueOwner<B>>
    for QueuedWebInputEvent<B>
{
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:122-169
    fn FilterNewEvent(&mut self, other_task: &mut QueueTask<B>) -> FilterResult {
        if !other_task.IsWebInputEvent() {
            return FilterResult::StopIterating;
        }
        let other = other_task
            .AsAnyMut()
            .downcast_mut::<Self>()
            .expect("input task payload");
        self.FilterNewQueuedEvent(other)
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:171
    fn IsWebInputEvent(&self) -> bool {
        true
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:173-204
    fn Dispatch(&mut self, queue: &mut QueueOwner<B>) {
        let owner = Arc::clone(queue);
        let result = self.DispatchWith(queue, &mut |event, metrics, callback| {
            Ok::<_, std::convert::Infallible>(
                owner
                    .Client()
                    .is_some_and(|client| client.HandleInputEvent(event, metrics, callback)),
            )
        });
        let Ok(()) = result;
    }
    fn AsAny(&self) -> &dyn Any {
        self
    }
    fn AsAnyMut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<B: MainThreadEventQueueServices> QueuedWebInputEvent<B> {
    // The embedding borrows its document executor for this synchronous client
    // call. Queue ownership and acknowledgement order remain unchanged.
    fn DispatchWith<E>(
        &mut self,
        queue: &QueueOwner<B>,
        dispatch: &mut impl FnMut(
            &WebCoalescedInputEvent<B::Latency>,
            Option<B::Metrics>,
            HandledEventCallback<B>,
        ) -> Result<bool, E>,
    ) -> Result<(), E> {
        if self.originally_cancelable_
            && matches!(self.Event().GetType(), Type::kTouchMove | Type::kTouchEnd)
        {
            if queue.GetMainThreadOnly().should_unblock_touch_sequence {
                B::PayloadAccess::SetTouchDispatchType(
                    self.mutable_coalesced_event().EventPointer(),
                    EventDispatchType::kEventNonBlocking,
                );
            }
            if B::PayloadAccess::TouchDispatchType(self.event_.Event())
                == EventDispatchType::kEventNonBlocking
            {
                Self::RunCallbacks(
                    &self.callbacks_,
                    InputEventResultState::kNotConsumed,
                    self.event_.latency_info(),
                    None,
                    None,
                );
            }
        }
        let queue_owner = Arc::clone(queue);
        let event = Arc::clone(&self.event_);
        let callbacks = Arc::clone(&self.callbacks_);
        let count = self.known_by_scheduler_count_;
        let callback = Box::new(move |ack, latency: &B::Latency, overscroll, touch_action| {
            Self::HandledEvent(
                &queue_owner,
                &event,
                &callbacks,
                count,
                ack,
                latency,
                overscroll,
                touch_action,
            );
        });
        let metrics = self.metrics_.take();
        if !queue.HandleEventOnMainThreadWith(
            &self.event_,
            self.attribution(),
            metrics,
            callback,
            dispatch,
        )? {
            Self::HandledEvent(
                queue,
                &self.event_,
                &self.callbacks_,
                self.known_by_scheduler_count_,
                InputEventResultState::kNotConsumed,
                self.event_.latency_info(),
                None,
                None,
            );
        }
        Ok(())
    }
}

impl<B: MainThreadEventQueueServices> QueuedWebInputEvent<B> {
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:206-226
    fn HandledEvent(
        queue: &QueueOwner<B>,
        event: &WebCoalescedInputEvent<B::Latency>,
        callbacks: &Mutex<CallbackState<B>>,
        known_by_scheduler_count: usize,
        ack_result: InputEventResultState,
        latency: &B::Latency,
        overscroll: Option<B::Overscroll>,
        touch_action: Option<B::TouchAction>,
    ) {
        Self::RunCallbacks(callbacks, ack_result, latency, overscroll, touch_action);
        for _ in 0..known_by_scheduler_count {
            let pending = queue
                .Client()
                .is_some_and(|client| client.RequestedMainFramePending());
            queue.services_.DidHandleInputEventOnMainThread(
                event.Event().Event(),
                ack_result == InputEventResultState::kConsumed,
                pending,
            );
        }
        queue.UnblockQueuedBlockingTouchEventsIfNeeded(event.Event(), ack_result);
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:232-247
    fn TakeCallbacksInto(&mut self, callbacks: &mut Vec<CallbackInfo<B>>) {
        let mut state = self.callbacks_.lock().unwrap();
        if let Some(callback) = state.callback.take() {
            callbacks.push(CallbackInfo {
                callback,
                latency_info: self.event_.latency_info().clone(),
            });
        }
        if !state.blocking_coalesced_callbacks.is_empty() {
            let mut latency = self.event_.latency_info().clone();
            latency.set_coalesced();
            for (callback, trace_id) in state.blocking_coalesced_callbacks.drain(..) {
                latency.set_trace_id(trace_id);
                callbacks.push(CallbackInfo {
                    callback,
                    latency_info: latency.clone(),
                });
            }
        }
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:248-259
    fn Unblock(&mut self) -> Option<CallbackInfo<B>> {
        debug_assert!(matches!(
            self.Event().GetType(),
            Type::kTouchMove | Type::kTouchEnd
        ));
        let callback = {
            let mut state = self.callbacks_.lock().unwrap();
            if state.callback.is_none() {
                return None;
            }
            debug_assert!(state.blocking_coalesced_callbacks.is_empty());
            state.callback.take().unwrap()
        };
        B::PayloadAccess::SetTouchDispatchType(
            self.mutable_coalesced_event().EventPointer(),
            EventDispatchType::kEventNonBlocking,
        );
        Some(CallbackInfo {
            callback,
            latency_info: self.event_.latency_info().clone(),
        })
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:261
    fn originally_cancelable(&self) -> bool {
        self.originally_cancelable_
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:263
    fn attribution(&self) -> &B::Attribution {
        &self.attribution_
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:265
    fn Event(&self) -> &WebInputEvent {
        self.event_.Event().Event()
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:267
    fn mutable_coalesced_event(&mut self) -> &mut WebCoalescedInputEvent<B::Latency> {
        Arc::get_mut(&mut self.event_)
            .expect("queued event remains uniquely mutable until dispatch")
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:269-271
    fn SetQueuedTimeStamp(&mut self, queued_time: TimeTicks) {
        self.mutable_coalesced_event()
            .EventPointer()
            .EventMut()
            .SetQueuedTimeStamp(queued_time);
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:274-298
    fn RunCallbacks(
        callbacks: &Mutex<CallbackState<B>>,
        ack_result: InputEventResultState,
        latency: &B::Latency,
        overscroll: Option<B::Overscroll>,
        touch_action: Option<B::TouchAction>,
    ) {
        let (callback, coalesced) = {
            let mut state = callbacks.lock().unwrap();
            (
                state.callback.take(),
                std::mem::take(&mut state.blocking_coalesced_callbacks),
            )
        };
        if let Some(callback) = callback {
            callback(ack_result, latency, overscroll, touch_action);
        }
        if !coalesced.is_empty() {
            let mut coalesced_latency = latency.clone();
            coalesced_latency.set_coalesced();
            for (callback, trace_id) in coalesced {
                coalesced_latency.set_trace_id(trace_id);
                callback(ack_result, &coalesced_latency, None, None);
            }
        }
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:300-320
    fn HandleTouchScrollStartQueued(&mut self) -> FilterResult {
        match self.Event().GetType() {
            Type::kTouchMove => {
                if B::PayloadAccess::TouchDispatchType(self.event_.Event())
                    == EventDispatchType::kBlocking
                {
                    B::PayloadAccess::SetTouchDispatchType(
                        self.mutable_coalesced_event().EventPointer(),
                        EventDispatchType::kEventNonBlocking,
                    );
                }
                FilterResult::KeepIterating
            }
            Type::kTouchStart | Type::kTouchEnd => FilterResult::StopIterating,
            _ => FilterResult::KeepIterating,
        }
    }
}

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.h:114-115
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DispatchType {
    kBlocking,
    kNonBlocking,
}

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.h:192-204
#[derive(Default)]
struct MainThreadOnly {
    blocking_touch_start_not_consumed: bool,
    should_unblock_touch_sequence: bool,
    touch_sequence_start_dequeued_count: i64,
}
#[derive(Default)]
struct CompositorThreadOnly {
    last_touch_start_forced_nonblocking_due_to_fling: bool,
    touch_sequence_start_enqueued_count: i64,
}

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.h:221-237
struct SharedState<B: MainThreadEventQueueServices> {
    events_: MainThreadEventQueueTaskList<QueueOwner<B>>,
    sent_main_frame_request_: bool,
    sent_post_task_: bool,
    any_gsu_acked_as_consumed_: Option<bool>,
    last_gsu_acked_as_consumed_: Option<bool>,
    last_async_touch_move_timestamp_: TimeTicks,
    unblock_touch_sequence_start_count_: i64,
}
impl<B: MainThreadEventQueueServices> Default for SharedState<B> {
    fn default() -> Self {
        Self {
            events_: MainThreadEventQueueTaskList::new(),
            sent_main_frame_request_: false,
            sent_post_task_: false,
            any_gsu_acked_as_consumed_: None,
            last_gsu_acked_as_consumed_: None,
            last_async_touch_move_timestamp_: TimeTicks::new(),
            unblock_touch_sequence_start_count_: -1,
        }
    }
}

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.h:66-111,189-267
pub struct MainThreadEventQueue<B: MainThreadEventQueueServices> {
    client_: Mutex<Option<Arc<dyn MainThreadEventQueueClient<B>>>>,
    allow_raf_aligned_input_: bool,
    main_thread_only_: Mutex<MainThreadOnly>,
    compositor_thread_only_: Mutex<CompositorThreadOnly>,
    has_pointerrawupdate_handlers_: AtomicBool,
    needs_low_latency_: AtomicBool,
    needs_unbuffered_input_for_debugger_: AtomicBool,
    needs_low_latency_until_pointer_up_: AtomicBool,
    shared_state_: Mutex<SharedState<B>>,
    services_: Arc<B>,
    raf_fallback_timer_: Mutex<Option<Box<dyn QueueOneShotTimer>>>,
    event_predictor_: Mutex<Option<Box<dyn QueueInputPrediction<B::Latency>>>>,
    features_: MainThreadEventQueueFeatures,
    cursor_control_in_progress_: Mutex<bool>,
}

impl<B: MainThreadEventQueueServices> MainThreadEventQueue<B> {
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:345-367
    // Services retain the injected task runners and scheduler. Timer and
    // predictor are queue-owned, as in Chromium; neither is a global service.
    pub fn new(
        client: Option<Arc<dyn MainThreadEventQueueClient<B>>>,
        services: Arc<B>,
        allow_raf_aligned_input: bool,
        timer: Option<Box<dyn QueueOneShotTimer>>,
        mut predictor: Option<Box<dyn QueueInputPrediction<B::Latency>>>,
        features: MainThreadEventQueueFeatures,
    ) -> QueueOwner<B> {
        if let Some(predictor) = predictor.as_mut() {
            predictor.SetResamplingEnabled(features.resampling_input_events);
        }
        Arc::new(Self {
            client_: Mutex::new(client),
            allow_raf_aligned_input_: allow_raf_aligned_input,
            main_thread_only_: Mutex::new(MainThreadOnly::default()),
            compositor_thread_only_: Mutex::new(CompositorThreadOnly::default()),
            has_pointerrawupdate_handlers_: AtomicBool::new(false),
            needs_low_latency_: AtomicBool::new(false),
            needs_unbuffered_input_for_debugger_: AtomicBool::new(false),
            needs_low_latency_until_pointer_up_: AtomicBool::new(false),
            shared_state_: Mutex::new(SharedState::default()),
            services_: services,
            raf_fallback_timer_: Mutex::new(timer),
            event_predictor_: Mutex::new(predictor),
            features_: features,
            cursor_control_in_progress_: Mutex::new(false),
        })
    }

    // Rust borrow of the source nullable client pointer. No queue lock is held
    // across a client invocation, preserving synchronous listener reentry.
    fn Client(&self) -> Option<Arc<dyn MainThreadEventQueueClient<B>>> {
        self.client_.lock().unwrap().clone()
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:369-397
    fn Allowed(&self, event: &dyn WebInputEventPayload, force_allow: bool) -> bool {
        if force_allow {
            return true;
        }
        let event_type = event.Event().GetType();
        if !IsGestureScroll(event_type) {
            return true;
        }
        let mut cursor_control = self.cursor_control_in_progress_.lock().unwrap();
        if event_type == Type::kGestureScrollBegin
            && B::PayloadAccess::GestureScrollBeginCursorControl(event)
        {
            *cursor_control = true;
        }
        let allowed = *cursor_control;
        if event_type == Type::kGestureScrollEnd && *cursor_control {
            *cursor_control = false;
        }
        allowed
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:399-415
    pub fn OnGestureScrollEventAck(&self, event_type: Type, ack: InputEventResultState) {
        match event_type {
            Type::kGestureScrollBegin => self.OnGestureScrollStartAck(ack),
            Type::kGestureScrollUpdate => self.OnGestureScrollUpdateAck(ack),
            Type::kGestureScrollEnd => self.OnGestureScrollEndAck(ack),
            _ => unreachable!("gesture scroll ACK requires a scroll event"),
        }
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:417-422
    fn OnGestureScrollStartAck(&self, _ack: InputEventResultState) {
        let mut state = self.shared_state_.lock().unwrap();
        state.any_gsu_acked_as_consumed_ = Some(false);
        state.last_gsu_acked_as_consumed_ = Some(false);
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:424-433
    fn OnGestureScrollUpdateAck(&self, ack: InputEventResultState) {
        let mut state = self.shared_state_.lock().unwrap();
        let Some(any) = state.any_gsu_acked_as_consumed_.as_mut() else {
            return;
        };
        let consumed = ack == InputEventResultState::kConsumed;
        *any |= consumed;
        state.last_gsu_acked_as_consumed_ = Some(consumed);
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:435-440
    fn OnGestureScrollEndAck(&self, _ack: InputEventResultState) {
        let mut state = self.shared_state_.lock().unwrap();
        state.any_gsu_acked_as_consumed_ = None;
        state.last_gsu_acked_as_consumed_ = None;
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:442-577
    pub fn HandleEvent(
        self: &QueueOwner<B>,
        mut event: WebCoalescedInputEvent<B::Latency>,
        original_dispatch_type: DispatchType,
        ack_result: InputEventResultState,
        attribution: &B::Attribution,
        metrics: Option<B::Metrics>,
        mut callback: Option<HandledEventCallback<B>>,
        allow_main_gesture_scroll: bool,
    ) {
        // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:450-451
        // TODO: connect the input trace duration and dispatch/event type args.
        debug_assert!(matches!(
            ack_result,
            InputEventResultState::kSetNonBlocking
                | InputEventResultState::kSetNonBlockingDueToFling
                | InputEventResultState::kNotConsumed
        ));
        debug_assert!(self.Allowed(event.Event(), allow_main_gesture_scroll));
        let mut is_blocking = original_dispatch_type == DispatchType::kBlocking
            && ack_result != InputEventResultState::kSetNonBlocking;
        let is_wheel = event.Event().Event().GetType() == Type::kMouseWheel;
        let is_touch = WebInputEvent::IsTouchEventType(event.Event().Event().GetType());
        let mut originally_cancelable = false;
        if is_touch {
            originally_cancelable =
                B::PayloadAccess::TouchDispatchType(event.Event()) == EventDispatchType::kBlocking;
            if !is_blocking {
                B::PayloadAccess::SetTouchDispatchType(
                    event.EventPointer(),
                    EventDispatchType::kListenersNonBlockingPassive,
                );
            }
            let mut compositor = self.GetCompositorThreadOnly();
            if event.Event().Event().GetType() == Type::kTouchStart {
                compositor.last_touch_start_forced_nonblocking_due_to_fling = false;
            }
            if B::PayloadAccess::TouchStartOrFirstTouchMove(event.Event())
                && B::PayloadAccess::TouchDispatchType(event.Event())
                    == EventDispatchType::kBlocking
            {
                if ack_result == InputEventResultState::kSetNonBlockingDueToFling
                    || compositor.last_touch_start_forced_nonblocking_due_to_fling
                {
                    B::PayloadAccess::SetTouchDispatchType(
                        event.EventPointer(),
                        EventDispatchType::kListenersForcedNonBlockingDueToFling,
                    );
                    is_blocking = false;
                    compositor.last_touch_start_forced_nonblocking_due_to_fling = true;
                }
            }
            if B::PayloadAccess::IsTouchSequenceStart(event.Event()) {
                compositor.touch_sequence_start_enqueued_count += 1;
            }
            if is_blocking
                && B::PayloadAccess::TouchDispatchType(event.Event())
                    != EventDispatchType::kBlocking
            {
                is_blocking = false;
            }
        }
        if is_wheel {
            originally_cancelable =
                B::PayloadAccess::WheelDispatchType(event.Event()) == EventDispatchType::kBlocking;
            if !is_blocking {
                B::PayloadAccess::SetWheelDispatchType(
                    event.EventPointer(),
                    EventDispatchType::kListenersNonBlockingPassive,
                );
            }
        }
        // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:529
        // TODO: connect the Blocking input instant trace.
        let event_callback = if is_blocking { callback.take() } else { None };
        if self.has_pointerrawupdate_handlers_.load(Ordering::Relaxed) {
            if event.Event().Event().GetType() == Type::kMouseMove {
                let raw = WebCoalescedInputEvent::FromOwnedEvent(
                    B::PayloadAccess::WrapPointerRawUpdate(
                        event.Event(),
                        B::PayloadAccess::PointerFromMouse(event.Event()),
                    ),
                    event.latency_info(),
                );
                self.QueueEvent(Box::new(QueuedWebInputEvent::<B>::CreateForRawEvent(
                    raw,
                    attribution.clone(),
                    metrics.as_ref(),
                )));
            } else if event.Event().Event().GetType() == Type::kTouchMove {
                for index in 0..B::PayloadAccess::TouchesLength(event.Event()) {
                    if B::PayloadAccess::TouchPointState(event.Event(), index)
                        == TouchState::kStateMoved
                    {
                        let mut pointer = B::PayloadAccess::PointerFromTouch(event.Event(), index);
                        pointer.SetType(Type::kPointerRawUpdate);
                        let raw = WebCoalescedInputEvent::FromOwnedEvent(
                            B::PayloadAccess::WrapPointerRawUpdate(event.Event(), pointer),
                            event.latency_info(),
                        );
                        self.QueueEvent(Box::new(QueuedWebInputEvent::<B>::CreateForRawEvent(
                            raw,
                            attribution.clone(),
                            metrics.as_ref(),
                        )));
                    }
                }
            }
        }
        let cloned_latency_info = callback.as_ref().map(|_| event.latency_info().clone());
        let queued = QueuedWebInputEvent::<B>::new(
            event,
            originally_cancelable,
            event_callback,
            Self::IsForwardedAndSchedulerKnown(ack_result),
            attribution.clone(),
            metrics,
        );
        self.QueueEvent(Box::new(queued));
        if let Some(callback) = callback {
            callback(
                ack_result,
                cloned_latency_info.as_ref().unwrap(),
                None,
                None,
            );
        }
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:579-591
    pub fn QueueClosure(self: &QueueOwner<B>, closure: Box<dyn FnOnce() + Send>) {
        let needs_post_task = {
            let mut state = self.shared_state_.lock().unwrap();
            state
                .events_
                .Enqueue(Box::new(QueuedClosure(Some(closure))));
            let needed = !state.sent_post_task_;
            state.sent_post_task_ = true;
            needed
        };
        if needs_post_task {
            self.PostTaskToMainThread();
        }
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:593-607
    fn PossiblyScheduleMainFrame(self: &QueueOwner<B>) {
        let needs_main_frame = {
            let mut state = self.shared_state_.lock().unwrap();
            if !state.sent_main_frame_request_
                && !state.events_.empty()
                && self.IsRafAlignedEvent(state.events_.front())
            {
                state.sent_main_frame_request_ = true;
                true
            } else {
                false
            }
        };
        if needs_main_frame {
            self.SetNeedsMainFrame(BeginMainFrameReason::kInput, false);
        }
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:609-683
    pub fn DispatchEvents(self: &QueueOwner<B>) {
        let result = self.DispatchEventsWith(&mut |event, metrics, callback| {
            Ok::<_, std::convert::Infallible>(
                self.Client()
                    .is_some_and(|client| client.HandleInputEvent(event, metrics, callback)),
            )
        });
        let Ok(()) = result;
    }

    pub fn DispatchEventsWith<E>(
        self: &QueueOwner<B>,
        dispatch: &mut impl FnMut(
            &WebCoalescedInputEvent<B::Latency>,
            Option<B::Metrics>,
            HandledEventCallback<B>,
        ) -> Result<bool, E>,
    ) -> Result<(), E> {
        let mut events_to_process = {
            let mut state = self.shared_state_.lock().unwrap();
            state.sent_post_task_ = false;
            let mut count = state.events_.size();
            while count > 0 && !self.ShouldFlushQueue(state.events_.at(count - 1)) {
                count -= 1;
            }
            count
        };
        while events_to_process > 0 {
            events_to_process -= 1;
            let task = self.shared_state_.lock().unwrap().events_.Pop();
            let Some(mut task) = task else {
                return Ok(());
            };
            self.HandleEventResampling(task.as_mut(), self.services_.Now());
            self.DispatchTaskWith(task.as_mut(), dispatch)?;
        }
        let mut queue_size = self.shared_state_.lock().unwrap().events_.size();
        let mut current_task_index = 0;
        while current_task_index < queue_size {
            let task = {
                let mut state = self.shared_state_.lock().unwrap();
                while current_task_index < queue_size && current_task_index < state.events_.size() {
                    if !self.IsRafAlignedEvent(state.events_.at(current_task_index)) {
                        break;
                    }
                    current_task_index += 1;
                }
                if current_task_index >= queue_size || current_task_index >= state.events_.size() {
                    break;
                }
                if self.IsRawUpdateEvent(state.events_.at(current_task_index)) {
                    queue_size -= 1;
                    state.events_.remove(current_task_index)
                } else if !self.IsRafAlignedEvent(state.events_.at(current_task_index)) {
                    break;
                } else {
                    current_task_index += 1;
                    None
                }
            };
            if let Some(mut task) = task {
                self.DispatchTaskWith(task.as_mut(), dispatch)?;
            }
            // A removed raw event leaves the next index unchanged, equivalent
            // to the source's --current_task_index followed by the for ++.
        }
        self.PossiblyScheduleMainFrame();
        if let Some(client) = self.Client() {
            client.InputEventsDispatched(false);
        }
        Ok(())
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:685-697
    fn IsAsyncTouchMove(item: &QueueTask<B>) -> bool {
        if !item.IsWebInputEvent() {
            return false;
        }
        let event = item
            .AsAny()
            .downcast_ref::<QueuedWebInputEvent<B>>()
            .expect("input task");
        event.Event().GetType() == Type::kTouchMove
            && B::PayloadAccess::TouchMovedBeyondSlopRegion(event.event_.Event())
            && !event.originally_cancelable()
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:699-704
    pub fn RafFallbackTimerFired(self: &QueueOwner<B>) {
        self.DispatchRafAlignedInput(self.services_.Now());
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:706-708
    pub fn ClearRafFallbackTimerForTesting(&self) {
        self.raf_fallback_timer_.lock().unwrap().take();
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:710-713
    pub fn IsEmptyForTesting(&self) -> bool {
        self.IsEmpty()
    }
    pub fn IsEmpty(&self) -> bool {
        self.shared_state_.lock().unwrap().events_.empty()
    }

    pub fn HasPendingEventType(&self, event_type: Type) -> bool {
        let state = self.shared_state_.lock().unwrap();
        (0..state.events_.size()).any(|index| {
            state
                .events_
                .at(index)
                .AsAny()
                .downcast_ref::<QueuedWebInputEvent<B>>()
                .is_some_and(|event| event.Event().GetType() == event_type)
        })
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:715-759
    pub fn DispatchRafAlignedInput(self: &QueueOwner<B>, frame_time: TimeTicks) {
        let result =
            self.DispatchRafAlignedInputWith(frame_time, &mut |event, metrics, callback| {
                Ok::<_, std::convert::Infallible>(
                    self.Client()
                        .is_some_and(|client| client.HandleInputEvent(event, metrics, callback)),
                )
            });
        let Ok(()) = result;
    }

    /// Run the source snapshot and dispatch path with a borrowed synchronous
    /// document executor. No input is buffered past its client callback.
    pub fn DispatchRafAlignedInputWith<E>(
        self: &QueueOwner<B>,
        frame_time: TimeTicks,
        dispatch: &mut impl FnMut(
            &WebCoalescedInputEvent<B::Latency>,
            Option<B::Metrics>,
            HandledEventCallback<B>,
        ) -> Result<bool, E>,
    ) -> Result<(), E> {
        if let Some(timer) = self.raf_fallback_timer_.lock().unwrap().as_mut() {
            timer.Stop();
        }
        let mut count = {
            let mut state = self.shared_state_.lock().unwrap();
            state.sent_main_frame_request_ = false;
            state.events_.size()
        };
        while count > 0 {
            count -= 1;
            let task = {
                let mut state = self.shared_state_.lock().unwrap();
                if state.events_.empty() {
                    return Ok(());
                }
                if self.IsRafAlignedEvent(state.events_.front())
                    && Self::IsAsyncTouchMove(state.events_.front())
                    && self.ShouldThrottleAsyncTouchMoves(&state)
                {
                    if state.events_.size() == 1
                        && frame_time.ToInternalValue()
                            < state
                                .last_async_touch_move_timestamp_
                                .ToInternalValue()
                                .saturating_add(kAsyncTouchMoveInterval)
                    {
                        break;
                    }
                    state.last_async_touch_move_timestamp_ = frame_time;
                }
                state.events_.Pop()
            };
            let mut task = task.expect("nonempty queue");
            self.HandleEventResampling(task.as_mut(), frame_time);
            self.DispatchTaskWith(task.as_mut(), dispatch)?;
        }
        self.PossiblyScheduleMainFrame();
        if let Some(client) = self.Client() {
            client.InputEventsDispatched(true);
        }
        Ok(())
    }

    fn DispatchTaskWith<E>(
        self: &QueueOwner<B>,
        task: &mut QueueTask<B>,
        dispatch: &mut impl FnMut(
            &WebCoalescedInputEvent<B::Latency>,
            Option<B::Metrics>,
            HandledEventCallback<B>,
        ) -> Result<bool, E>,
    ) -> Result<(), E> {
        if task.IsWebInputEvent() {
            task.AsAnyMut()
                .downcast_mut::<QueuedWebInputEvent<B>>()
                .expect("input task")
                .DispatchWith(self, dispatch)
        } else {
            task.Dispatch(&mut Arc::clone(self));
            Ok(())
        }
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:761-764
    fn PostTaskToMainThread(self: &QueueOwner<B>) {
        let queue = Arc::clone(self);
        self.services_
            .PostTaskToMainThread(Box::new(move || queue.DispatchEvents()));
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:766-844
    fn QueueEvent(self: &QueueOwner<B>, mut event: Box<QueueTask<B>>) {
        let is_raf_aligned = self.IsRafAlignedEvent(event.as_ref());
        let mut needs_main_frame = false;
        let mut needs_post_task = false;
        let mut unblocked_callback = None;
        let is_input_event = event.IsWebInputEvent();
        let mut input_event_type = Type::kUndefined;
        let mut attribution = B::Attribution::default();
        if is_input_event {
            let queued = event
                .AsAnyMut()
                .downcast_mut::<QueuedWebInputEvent<B>>()
                .expect("input task");
            input_event_type = queued.Event().GetType();
            attribution = queued.attribution().clone();
            queued.SetQueuedTimeStamp(self.services_.Now());
        }
        {
            let mut state = self.shared_state_.lock().unwrap();
            let enqueued_count = self
                .GetCompositorThreadOnly()
                .touch_sequence_start_enqueued_count;
            assert!(state.unblock_touch_sequence_start_count_ <= enqueued_count);
            if is_input_event
                && matches!(input_event_type, Type::kTouchMove | Type::kTouchEnd)
                && enqueued_count == state.unblock_touch_sequence_start_count_
            {
                unblocked_callback = event
                    .AsAnyMut()
                    .downcast_mut::<QueuedWebInputEvent<B>>()
                    .expect("input task")
                    .Unblock();
            }
            if state.events_.Enqueue(event) == EnqueueResult::kEnqueued {
                if !is_raf_aligned {
                    needs_post_task = !state.sent_post_task_;
                    state.sent_post_task_ = true;
                } else {
                    needs_main_frame = !state.sent_main_frame_request_;
                    state.sent_main_frame_request_ = true;
                }
                if is_input_event {
                    self.services_
                        .WillPostInputEventToMainThread(input_event_type, &attribution);
                }
            }
        }
        if needs_post_task {
            self.PostTaskToMainThread();
        }
        if needs_main_frame {
            let urgent = self.features_.eligible_for_throttle_main_frame_to_60hz
                && self.features_.urgent_main_frame_for_input;
            self.SetNeedsMainFrame(BeginMainFrameReason::kInput, urgent);
        }
        if let Some(info) = unblocked_callback {
            (info.callback)(
                InputEventResultState::kSetNonBlocking,
                &info.latency_info,
                None,
                None,
            );
        }
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:846-852
    fn IsRawUpdateEvent(&self, item: &QueueTask<B>) -> bool {
        item.IsWebInputEvent()
            && item
                .AsAny()
                .downcast_ref::<QueuedWebInputEvent<B>>()
                .expect("input task")
                .Event()
                .GetType()
                == Type::kPointerRawUpdate
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:854-859
    fn ShouldFlushQueue(&self, item: &QueueTask<B>) -> bool {
        if self.IsRawUpdateEvent(item) {
            return false;
        }
        !self.IsRafAlignedEvent(item)
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:861-880
    fn IsRafAlignedEvent(&self, item: &QueueTask<B>) -> bool {
        if !item.IsWebInputEvent() {
            return false;
        }
        let event = item
            .AsAny()
            .downcast_ref::<QueuedWebInputEvent<B>>()
            .expect("input task");
        self.ShouldAlignEventType(event.Event().GetType())
    }

    pub fn ShouldAlignEventType(&self, event_type: Type) -> bool {
        IsRafAlignedInputEvent(
            event_type,
            self.allow_raf_aligned_input_,
            self.needs_low_latency_.load(Ordering::Relaxed),
            self.needs_low_latency_until_pointer_up_
                .load(Ordering::Relaxed),
            self.needs_unbuffered_input_for_debugger_
                .load(Ordering::Relaxed),
        )
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:882-890
    pub fn HandleEventResampling(&self, item: &mut QueueTask<B>, frame_time: TimeTicks) {
        if item.IsWebInputEvent() && self.allow_raf_aligned_input_ {
            if let Some(predictor) = self.event_predictor_.lock().unwrap().as_mut() {
                let event = item
                    .AsAnyMut()
                    .downcast_mut::<QueuedWebInputEvent<B>>()
                    .expect("input task");
                predictor.HandleEvents(event.mutable_coalesced_event(), frame_time);
            }
        }
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:892-930
    fn HandleEventOnMainThreadWith<E>(
        &self,
        event: &WebCoalescedInputEvent<B::Latency>,
        attribution: &B::Attribution,
        metrics: Option<B::Metrics>,
        callback: HandledEventCallback<B>,
        dispatch: &mut impl FnMut(
            &WebCoalescedInputEvent<B::Latency>,
            Option<B::Metrics>,
            HandledEventCallback<B>,
        ) -> Result<bool, E>,
    ) -> Result<bool, E> {
        if WebInputEvent::IsTouchEventType(event.Event().Event().GetType())
            && B::PayloadAccess::IsTouchSequenceStart(event.Event())
        {
            self.GetMainThreadOnly().touch_sequence_start_dequeued_count += 1;
        }
        self.services_
            .WillHandleInputEventOnMainThread(event.Event().Event().GetType(), attribution);
        let handled = dispatch(event, metrics, callback)?;
        self.DidHandleInputEventType(event.Event().Event().GetType());
        Ok(handled)
    }

    // Source main_thread_event_queue.cc:914-928. Result-returning host
    // ingress uses this same source owner after synchronous dispatch, while
    // queued packets reach it above. Neither path recreates queue policy.
    pub fn DidHandleInputEventType(&self, event_type: Type) {
        if self
            .needs_low_latency_until_pointer_up_
            .load(Ordering::Relaxed)
        {
            if matches!(
                event_type,
                Type::kMouseUp
                    | Type::kTouchCancel
                    | Type::kTouchEnd
                    | Type::kPointerCancel
                    | Type::kPointerUp
            ) {
                self.needs_low_latency_until_pointer_up_
                    .store(false, Ordering::Relaxed);
            }
        }
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:932-949
    fn SetNeedsMainFrame(self: &QueueOwner<B>, reason: BeginMainFrameReason, urgent: bool) {
        if self.services_.MainThreadBelongsToCurrentThread() {
            if let Some(timer) = self.raf_fallback_timer_.lock().unwrap().as_mut() {
                // A queue-owned timer may not keep its owner alive forever.
                // The source's this pointer is invalidated with that timer.
                let queue = Arc::downgrade(self);
                timer.Start(
                    kMaxRafDelay,
                    Box::new(move || {
                        if let Some(queue) = queue.upgrade() {
                            queue.RafFallbackTimerFired();
                        }
                    }),
                );
            }
            if let Some(client) = self.Client() {
                client.SetNeedsMainFrame(reason, urgent);
            }
            return;
        }
        let queue = Arc::clone(self);
        self.services_
            .PostTaskToMainThread(Box::new(move || queue.SetNeedsMainFrame(reason, urgent)));
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:951-955
    pub fn ClearClient(&self) {
        debug_assert!(self.services_.MainThreadBelongsToCurrentThread());
        self.client_.lock().unwrap().take();
        self.raf_fallback_timer_.lock().unwrap().take();
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:957-959
    pub fn SetNeedsLowLatency(&self, value: bool) {
        self.needs_low_latency_.store(value, Ordering::Relaxed);
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:961-964
    pub fn SetNeedsUnbufferedInputForDebugger(&self, value: bool) {
        self.needs_unbuffered_input_for_debugger_
            .store(value, Ordering::Relaxed);
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:966-969
    pub fn SetHasPointerRawUpdateEventHandlers(&self, value: bool) {
        self.has_pointerrawupdate_handlers_
            .store(value, Ordering::Relaxed);
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:971-973
    pub fn RequestUnbufferedInputEvents(&self) {
        self.needs_low_latency_until_pointer_up_
            .store(true, Ordering::Relaxed);
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.h:146-151
    pub fn IsForwardedAndSchedulerKnown(ack: InputEventResultState) -> bool {
        matches!(
            ack,
            InputEventResultState::kNotConsumed | InputEventResultState::kSetNonBlockingDueToFling
        )
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:975-1063
    fn UnblockQueuedBlockingTouchEventsIfNeeded(
        &self,
        dispatched_event: &dyn WebInputEventPayload,
        ack_result: InputEventResultState,
    ) {
        if !WebInputEvent::IsTouchEventType(dispatched_event.Event().GetType()) {
            return;
        }
        {
            let mut main = self.GetMainThreadOnly();
            if B::PayloadAccess::TouchStartOrFirstTouchMove(dispatched_event) {
                let not_consumed_blocking = B::PayloadAccess::TouchDispatchType(dispatched_event)
                    == EventDispatchType::kBlocking
                    && ack_result == InputEventResultState::kNotConsumed;
                if dispatched_event.Event().GetType() == Type::kTouchStart {
                    main.blocking_touch_start_not_consumed = not_consumed_blocking;
                    main.should_unblock_touch_sequence = false;
                } else {
                    assert_eq!(dispatched_event.Event().GetType(), Type::kTouchMove);
                    main.should_unblock_touch_sequence =
                        main.blocking_touch_start_not_consumed && not_consumed_blocking;
                }
            }
            if !main.should_unblock_touch_sequence {
                return;
            }
        }
        let mut callbacks = Vec::new();
        {
            let mut state = self.shared_state_.lock().unwrap();
            state.unblock_touch_sequence_start_count_ =
                self.GetMainThreadOnly().touch_sequence_start_dequeued_count;
            for index in 0..state.events_.size() {
                let task = state.events_.at_mut(index);
                if !task.IsWebInputEvent() {
                    continue;
                }
                let queued = task
                    .AsAnyMut()
                    .downcast_mut::<QueuedWebInputEvent<B>>()
                    .expect("input task");
                let event_type = queued.Event().GetType();
                if event_type == Type::kTouchStart {
                    break;
                }
                if event_type == Type::kTouchMove
                    && !B::PayloadAccess::TouchStartOrFirstTouchMove(queued.event_.Event())
                    && B::PayloadAccess::TouchDispatchType(queued.event_.Event())
                        == EventDispatchType::kBlocking
                {
                    B::PayloadAccess::SetTouchDispatchType(
                        queued.mutable_coalesced_event().EventPointer(),
                        EventDispatchType::kEventNonBlocking,
                    );
                    queued.TakeCallbacksInto(&mut callbacks);
                }
                if event_type == Type::kTouchEnd {
                    if B::PayloadAccess::TouchDispatchType(queued.event_.Event())
                        == EventDispatchType::kBlocking
                    {
                        B::PayloadAccess::SetTouchDispatchType(
                            queued.mutable_coalesced_event().EventPointer(),
                            EventDispatchType::kEventNonBlocking,
                        );
                        queued.TakeCallbacksInto(&mut callbacks);
                    }
                    break;
                }
            }
        }
        for info in callbacks {
            (info.callback)(
                InputEventResultState::kNotConsumed,
                &info.latency_info,
                None,
                None,
            );
        }
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:1065-1069
    fn GetMainThreadOnly(&self) -> std::sync::MutexGuard<'_, MainThreadOnly> {
        debug_assert!(self.services_.MainThreadBelongsToCurrentThread());
        self.main_thread_only_.lock().unwrap()
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:1071-1077
    fn GetCompositorThreadOnly(&self) -> std::sync::MutexGuard<'_, CompositorThreadOnly> {
        debug_assert!(self.services_.CompositorThreadBelongsToCurrentThread());
        self.compositor_thread_only_.lock().unwrap()
    }
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue.cc:1079-1101
    fn ShouldThrottleAsyncTouchMoves(&self, state: &SharedState<B>) -> bool {
        if self.features_.unthrottle_async_touch_moves {
            if self.features_.async_touch_move_throttling_policy
                == AsyncTouchMoveThrottlingPolicy::kUnthrottledAlways
            {
                return false;
            }
            if self.features_.async_touch_move_throttling_policy
                == AsyncTouchMoveThrottlingPolicy::kUnthrottledWhenGsuUnconsumed
            {
                if let Some(last) = state.last_gsu_acked_as_consumed_ {
                    return last;
                }
            }
            return true;
        }
        state.any_gsu_acked_as_consumed_.unwrap_or(true)
    }
}
