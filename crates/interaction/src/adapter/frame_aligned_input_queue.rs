//! Host queue boundary for Blink's translated main-thread event queue.
//!
//! Admission, filtering, the dispatch snapshot and acknowledgements belong to
//! `interaction_input`. Page supplies a synchronous borrowed dispatch port.

use std::any::Any;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::input_event::WebGestureEvent;
use super::input_event::{
    ConvertedWebInputEvent, InputDispatchType, InputEvent, PointerEvent, PointerType, WheelPhase,
};
use interaction_input::base::time::time::TimeTicks;
use interaction_input::platform::widget::input::main_thread_event_queue::{
    AsyncTouchMoveThrottlingPolicy, BeginMainFrameReason, DispatchType as QueueDispatchType,
    HandledEventCallback, InputEventResultState, IsRafAlignedInputEvent, MainThreadEventQueue,
    MainThreadEventQueueClient, MainThreadEventQueueFeatures, MainThreadEventQueueServices,
    QueueEventMetrics, QueueLatencyInfo, QueuePayloadAccess,
};
use interaction_input::public::common::input::web_coalesced_input_event::WebCoalescedInputEvent;
use interaction_input::public::common::input::web_input_event::{
    DispatchType, Type, WebInputEvent, WebInputEventPayload,
};
use interaction_input::public::common::input::web_mouse_event::WebMouseEvent;
use interaction_input::public::common::input::web_mouse_wheel_event::{Phase, WebMouseWheelEvent};
use interaction_input::public::common::input::web_pointer_event::WebPointerEvent;
use interaction_input::public::common::input::web_touch_event::WebTouchEvent;
use interaction_input::ui::events::gesture_detection::filtered_gesture_provider::{
    FilteredGestureProvider, GestureTouchActionFilterHandle, TouchAction,
};
use interaction_input::ui::events::types::event_type::EventType;

#[derive(Clone, Debug)]
pub struct QueuedFrameInput {
    pub input: InputEvent,
    pub queued_at: Instant,
    pub samples: usize,
    pub browser_gesture_dispatch_id: Option<u64>,
}

/// Immutable source input borrowed from the queue's retained event owner.
/// No queue lock or mutable engine borrow crosses the synchronous callback.
#[derive(Clone, Copy)]
pub struct BorrowedInputPacket<'a> {
    event: &'a dyn WebInputEventPayload,
    coalesced: &'a [Box<dyn WebInputEventPayload>],
    predicted: &'a [Box<dyn WebInputEventPayload>],
    latency: &'a FrameLatency,
}
impl<'a> BorrowedInputPacket<'a> {
    fn Raw(event: &'a dyn WebInputEventPayload) -> &'a dyn WebInputEventPayload {
        event
            .AsAny()
            .downcast_ref::<HostPayload>()
            .map_or(event, |host| host.raw.as_ref())
    }
    pub fn Event(self) -> &'a dyn WebInputEventPayload {
        Self::Raw(self.event)
    }
    pub fn CoalescedEvents(self) -> impl ExactSizeIterator<Item = &'a dyn WebInputEventPayload> {
        self.coalesced.iter().map(|event| Self::Raw(event.as_ref()))
    }
    pub fn PredictedEvents(self) -> impl ExactSizeIterator<Item = &'a dyn WebInputEventPayload> {
        self.predicted.iter().map(|event| Self::Raw(event.as_ref()))
    }
    pub fn LatencyOwner(self) -> &'a dyn Any {
        self.latency
    }
}

pub struct OwnedInputPacket {
    event: Box<dyn WebInputEventPayload>,
    coalesced: Vec<Box<dyn WebInputEventPayload>>,
    predicted: Vec<Box<dyn WebInputEventPayload>>,
    latency: FrameLatency,
}
impl OwnedInputPacket {
    /// Takes a generated source event without converting its position, time or
    /// dispatch flags through the host projection again (e.g. fling wheel).
    pub fn FromCanonical(event: ConvertedWebInputEvent) -> Self {
        let event: Box<dyn WebInputEventPayload> = match event {
            ConvertedWebInputEvent::Mouse(event) => Box::new(event),
            ConvertedWebInputEvent::Pointer(event) => Box::new(event),
            ConvertedWebInputEvent::Touch(event) => Box::new(event),
            ConvertedWebInputEvent::Gesture(event) => Box::new(event),
            ConvertedWebInputEvent::Wheel(event) => Box::new(event),
            ConvertedWebInputEvent::Keyboard(event) => Box::new(event),
        };
        Self {
            coalesced: vec![event.Clone()],
            event,
            predicted: Vec::new(),
            latency: Default::default(),
        }
    }
    pub fn FromInput(
        input: &InputEvent,
        time: TimeTicks,
        scale: f32,
    ) -> Result<Self, super::input_event::WebInputConversionError> {
        let event = ConvertResult(input, time, scale)?;
        let mut coalesced = Vec::new();
        let mut predicted = Vec::new();
        match input {
            InputEvent::Pointer(pointer) => {
                for sample in &pointer.coalesced {
                    coalesced.push(Box::new(sample.ToWebEventWithWidgetScale(time, scale)?)
                        as Box<dyn WebInputEventPayload>);
                }
                for sample in &pointer.predicted {
                    predicted.push(Box::new(sample.ToWebEventWithWidgetScale(time, scale)?)
                        as Box<dyn WebInputEventPayload>);
                }
            }
            InputEvent::Touch(touch) => {
                for sample in &touch.coalesced {
                    coalesced.push(Box::new(sample.ToWebEventWithWidgetScale(time, scale)?)
                        as Box<dyn WebInputEventPayload>);
                }
                for sample in &touch.predicted {
                    predicted.push(Box::new(sample.ToWebEventWithWidgetScale(time, scale)?)
                        as Box<dyn WebInputEventPayload>);
                }
            }
            _ => {}
        }
        if coalesced.is_empty() {
            coalesced.push(event.Clone());
        }
        Ok(Self {
            event,
            coalesced,
            predicted,
            latency: FrameLatency::default(),
        })
    }
    pub fn Borrow(&self) -> BorrowedInputPacket<'_> {
        BorrowedInputPacket {
            event: self.event.as_ref(),
            coalesced: &self.coalesced,
            predicted: &self.predicted,
            latency: &self.latency,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DispatchedFrameInput {
    pub input: InputEvent,
    pub queued_at: Instant,
    pub samples: usize,
    pub default_prevented: bool,
    pub dispatch_started: Instant,
    pub dispatch_finished: Instant,
}
#[derive(Clone, Copy, Debug)]
pub struct TouchDisposition {
    pub unique_touch_event_id: u32,
    pub consumed: bool,
}
#[derive(Clone, Debug)]
pub struct WheelDisposition {
    pub document_generation: u64,
    pub requests: Vec<u64>,
    pub result: InputEventResultState,
}
#[derive(Clone, Copy, Debug)]
pub struct BrowserGestureDisposition {
    pub dispatch_id: u64,
    pub consumed: bool,
}

#[derive(Clone)]
pub struct FrameAlignedInputQueue {
    queue: Arc<MainThreadEventQueue<HostPorts>>,
    ports: Arc<HostPorts>,
}

impl Default for FrameAlignedInputQueue {
    fn default() -> Self {
        let ports = Arc::new(HostPorts {
            needs_main_frame: AtomicBool::new(false),
            owner_thread: std::thread::current().id(),
            needs_dispatch_task: AtomicBool::new(false),
            widget_scale: AtomicU32::new(1.0f32.to_bits()),
            clock: Mutex::new(Arc::new(super::host_event_clock::HostEventClock)),
            touch_dispositions: Mutex::new(Vec::new()),
            wheel_dispositions: Mutex::new(Vec::new()),
            recognizer: Mutex::new(FilteredGestureProvider::default()),
            document_generation: AtomicU64::new(1),
            generated_gestures: Mutex::new(Vec::new()),
            touch_admission: Mutex::new((None, false)),
            browser_gesture_dispositions: Mutex::new(Vec::new()),
        });
        // The Page rendering-opportunity owner guarantees delivery, or drains
        // immediately when no BeginFrame source is attached. It does not use a
        // separate timer/task runner or enable input prediction.
        let queue = MainThreadEventQueue::new(
            Some(ports.clone()),
            ports.clone(),
            true,
            None,
            None,
            MainThreadEventQueueFeatures {
                resampling_input_events: false,
                eligible_for_throttle_main_frame_to_60hz: false,
                urgent_main_frame_for_input: false,
                unthrottle_async_touch_moves: false,
                async_touch_move_throttling_policy:
                    AsyncTouchMoveThrottlingPolicy::kUnthrottledAlways,
            },
        );
        Self { queue, ports }
    }
}

impl FrameAlignedInputQueue {
    pub fn SetClock(&self, clock: Arc<dyn super::host_event_clock::HostInputClock>) {
        assert!(self.IsEmpty(), "bind the input clock before admission");
        *self.ports.clock.lock().unwrap() = clock;
    }
    pub fn SetWidgetScale(&self, scale: f32) {
        assert!(scale.is_finite() && scale > 0.0);
        self.ports
            .widget_scale
            .store(scale.to_bits(), Ordering::Relaxed);
    }
    // main_thread_event_queue.cc:957-959,861-880. This policy is retained
    // on the same queue that performs admission and coalescing.
    pub fn SetNeedsLowLatency(&mut self, value: bool) {
        self.queue.SetNeedsLowLatency(value);
    }

    pub fn SetHasPointerRawUpdateEventHandlers(&self, value: bool) {
        self.queue.SetHasPointerRawUpdateEventHandlers(value);
    }
    pub fn TakeTouchDispositions(&self) -> Vec<TouchDisposition> {
        std::mem::take(&mut *self.ports.touch_dispositions.lock().unwrap())
    }
    pub fn TakeWheelDispositions(&self) -> Vec<WheelDisposition> {
        std::mem::take(&mut *self.ports.wheel_dispositions.lock().unwrap())
    }
    pub fn InputDocumentGeneration(&self) -> u64 {
        self.ports.document_generation.load(Ordering::Acquire)
    }
    pub fn TakeBrowserGestureDispositions(&self) -> Vec<BrowserGestureDisposition> {
        std::mem::take(&mut *self.ports.browser_gesture_dispositions.lock().unwrap())
    }
    pub fn SetTouchAction(&self, action: TouchAction) {
        self.ports.recognizer.lock().unwrap().SetTouchAction(action);
    }
    pub fn TouchActionFilterHandle(&self) -> GestureTouchActionFilterHandle {
        self.ports
            .recognizer
            .lock()
            .unwrap()
            .TouchActionFilterHandle()
    }
    pub fn TakeGeneratedGestures(&self) -> Vec<WebGestureEvent> {
        std::mem::take(&mut *self.ports.generated_gestures.lock().unwrap())
    }
    pub fn RunGestureTimeouts(&self) {
        let (generation, events) = {
            let mut recognizer = self.ports.recognizer.lock().unwrap();
            (
                self.ports.document_generation.load(Ordering::Acquire),
                recognizer.OnTimeout(self.ports.Now()),
            )
        };
        self.ports.PublishGeneratedGestures(generation, events);
    }
    pub fn ResetGestureDetection(&self) {
        let (generation, events) = {
            let mut recognizer = self.ports.recognizer.lock().unwrap();
            (
                self.ports.document_generation.load(Ordering::Acquire),
                recognizer.ResetDetection(self.ports.Now()),
            )
        };
        *self.ports.touch_admission.lock().unwrap() = (None, false);
        self.ports.PublishGeneratedGestures(generation, events);
    }
    /// Commit detaches the old document client. Original source callbacks
    /// still complete, but their generations cannot mutate the new detector
    /// or deliver old input/timeout output to its DOM.
    pub fn ResetGestureDetectionForDocumentCommit(&self) -> u64 {
        let mut recognizer = self.ports.recognizer.lock().unwrap();
        self.ports
            .document_generation
            .fetch_add(1, Ordering::AcqRel);
        recognizer.ResetForDocumentCommit();
        *self.ports.touch_admission.lock().unwrap() = (None, false);
        self.ports.generated_gestures.lock().unwrap().clear();
        // Already completed ACK effects retain their original ids. Pending
        // ACK packets were detached above; stale callbacks below only finish
        // the original acknowledgement, never re-enter the new detector.
        self.ports.document_generation.load(Ordering::Acquire)
    }
    pub fn NextGestureDeadlineDelay(&self) -> Option<Duration> {
        let deadline = self.ports.recognizer.lock().unwrap().NextDeadline()?;
        Some(Duration::from_micros(
            deadline
                .ToInternalValue()
                .saturating_sub(self.ports.Now().ToInternalValue())
                .max(0) as u64,
        ))
    }
    pub fn RequestUnbufferedInputEvents(&self) {
        self.queue.RequestUnbufferedInputEvents();
    }
    /// Source main_thread_event_queue.cc clears the touch sequence's low
    /// latency request after the original synchronous pointer-up callback.
    pub fn DidDispatchImmediateInput(&self, input: &InputEvent) {
        use super::input_event::{MouseEventType, PointerEventType, TouchEventType};
        let kind = match input {
            InputEvent::Mouse(event) if event.r#type == MouseEventType::kUp => Type::kMouseUp,
            InputEvent::Pointer(event) if event.r#type == PointerEventType::kUp => Type::kPointerUp,
            InputEvent::Pointer(event) if event.r#type == PointerEventType::kCancel => {
                Type::kPointerCancel
            }
            InputEvent::Touch(event) if event.r#type == TouchEventType::kEnd => Type::kTouchEnd,
            InputEvent::Touch(event) if event.r#type == TouchEventType::kCancel => {
                Type::kTouchCancel
            }
            _ => return,
        };
        self.queue.DidHandleInputEventType(kind);
    }
    pub fn SetNeedsUnbufferedInputForDebugger(&self, value: bool) {
        self.queue.SetNeedsUnbufferedInputForDebugger(value);
    }

    pub fn IsQueueInput(input: &InputEvent) -> bool {
        if matches!(input, InputEvent::Gesture(event) if event.IsScrollEvent()) {
            // Chromium routes gesture scrolling on impl. Only a source
            // cursor-control override may admit it to the main-thread queue.
            return false;
        }
        Self::IsFrameAligned(input)
            || matches!(
                input,
                InputEvent::Touch(_) | InputEvent::Pointer(_) | InputEvent::Gesture(_)
            ) && ConvertedEventType(input).is_some()
    }

    pub fn IsFrameAligned(input: &InputEvent) -> bool {
        ConvertedEventType(input)
            .is_some_and(|event_type| IsRafAlignedInputEvent(event_type, true, false, false, false))
    }

    pub fn ShouldAlign(&self, input: &InputEvent) -> bool {
        ConvertedEventType(input)
            .is_some_and(|event_type| self.queue.ShouldAlignEventType(event_type))
    }

    pub fn IsEmpty(&self) -> bool {
        self.queue.IsEmpty()
    }

    pub fn HasWheel(&self) -> bool {
        self.queue.HasPendingEventType(Type::kMouseWheel)
    }

    pub fn NeedsBeginFrame(&self) -> bool {
        self.ports.needs_main_frame.load(Ordering::Relaxed)
    }
    pub fn NeedsDispatchTask(&self) -> bool {
        self.ports.needs_dispatch_task.load(Ordering::Relaxed)
    }

    pub fn Dispatch(&self, mut dispatch: impl FnMut(QueuedFrameInput) -> bool) {
        let result = self.TryDispatch(|entry| Ok::<_, std::convert::Infallible>(dispatch(entry)));
        let Ok(()) = result;
    }

    pub fn TryDispatch<E>(
        &self,
        mut dispatch: impl FnMut(QueuedFrameInput) -> Result<bool, E>,
    ) -> Result<(), E> {
        self.TryDispatchPacket(true, |entry, _| dispatch(entry))
    }

    pub fn TryDispatchPacket<E>(
        &self,
        raf_aligned: bool,
        mut dispatch: impl FnMut(QueuedFrameInput, BorrowedInputPacket<'_>) -> Result<bool, E>,
    ) -> Result<(), E> {
        if !raf_aligned {
            self.ports
                .needs_dispatch_task
                .store(false, Ordering::Relaxed);
            return self.DispatchWith(false, dispatch);
        }
        if self
            .ports
            .needs_dispatch_task
            .swap(false, Ordering::Relaxed)
        {
            self.DispatchWith(false, &mut dispatch)?;
        }
        self.ports.needs_main_frame.store(false, Ordering::Relaxed);
        self.DispatchWith(true, dispatch)
    }

    pub fn TryDispatchImmediate<E>(
        &self,
        mut dispatch: impl FnMut(QueuedFrameInput) -> Result<bool, E>,
    ) -> Result<(), E> {
        self.TryDispatchPacket(false, |entry, _| dispatch(entry))
    }

    fn DispatchWith<E>(
        &self,
        raf_aligned: bool,
        mut dispatch: impl FnMut(QueuedFrameInput, BorrowedInputPacket<'_>) -> Result<bool, E>,
    ) -> Result<(), E> {
        let mut handle = |event: &WebCoalescedInputEvent<FrameLatency>,
                          _,
                          callback: HandledEventCallback<HostPorts>| {
            let payload = event
                .Event()
                .AsAny()
                .downcast_ref::<HostPayload>()
                .expect("host input payload");
            if payload.document_generation != self.ports.document_generation.load(Ordering::Acquire)
            {
                callback(
                    InputEventResultState::kNoConsumerExists,
                    event.latency_info(),
                    None,
                    None,
                );
                return Ok(true);
            }
            let queued = payload.Project();
            let packet = BorrowedInputPacket {
                event: event.Event(),
                coalesced: event.GetCoalescedEventsPointers(),
                predicted: event.GetPredictedEventsPointers(),
                latency: event.latency_info(),
            };
            let consumed = dispatch(queued, packet)?;
            callback(
                if consumed {
                    InputEventResultState::kConsumed
                } else {
                    InputEventResultState::kNotConsumed
                },
                event.latency_info(),
                None,
                None,
            );
            Ok(true)
        };
        if raf_aligned {
            self.queue
                .DispatchRafAlignedInputWith(self.ports.Now(), &mut handle)
        } else {
            self.queue.DispatchEventsWith(&mut handle)
        }
    }

    pub fn Push(&mut self, input: InputEvent, queued_at: Instant, samples: usize) {
        assert!(
            Self::IsQueueInput(&input),
            "host-only inputs use their explicit ingress"
        );
        let widget_scale = f32::from_bits(self.ports.widget_scale.load(Ordering::Relaxed));
        let observed = match &input {
            InputEvent::Touch(touch) => touch.timestamp.unwrap_or(queued_at),
            _ => queued_at,
        };
        let packet =
            OwnedInputPacket::FromInput(&input, self.ports.TimeForInstant(observed), widget_scale)
                .expect("representable host event");
        self.PushPacket(input, packet, queued_at, samples);
    }
    pub fn PushPacket(
        &mut self,
        input: InputEvent,
        packet: OwnedInputPacket,
        queued_at: Instant,
        samples: usize,
    ) {
        self.PushPacketWithGestureDispatch(input, packet, queued_at, samples, None);
    }
    pub fn PushBrowserGesturePacket(
        &mut self,
        input: InputEvent,
        packet: OwnedInputPacket,
        queued_at: Instant,
        samples: usize,
        dispatch_id: u64,
    ) {
        assert!(matches!(&input,InputEvent::Gesture(event) if !event.IsScrollEvent()));
        self.PushPacketWithGestureDispatch(input, packet, queued_at, samples, Some(dispatch_id));
    }
    fn PushPacketWithGestureDispatch(
        &mut self,
        mut input: InputEvent,
        mut packet: OwnedInputPacket,
        queued_at: Instant,
        samples: usize,
        browser_gesture_dispatch_id: Option<u64>,
    ) {
        let mut document_generation = self.ports.document_generation.load(Ordering::Acquire);
        let widget_scale = f32::from_bits(self.ports.widget_scale.load(Ordering::Relaxed));
        // Chromium recognizes each original MotionEvent in DIP, before queue
        // coalescing; renderer input retains its original native coordinates.
        if let Some(raw) = packet.event.AsAnyMut().downcast_mut::<WebTouchEvent>() {
            if raw.GetType() != Type::kTouchScrollStarted {
                let mut dip = raw.FlattenTransform();
                let touches_length = dip.touches_length;
                for point in &mut dip.touches[..touches_length] {
                    let mut position = *point.pointer_properties.PositionInWidget();
                    position.ScaleUniform(1.0 / widget_scale);
                    point.pointer_properties.SetPositionInWidget(position);
                    point.radius_x /= widget_scale;
                    point.radius_y /= widget_scale;
                }
                let result = {
                    let mut recognizer = self.ports.recognizer.lock().unwrap();
                    document_generation = self.ports.document_generation.load(Ordering::Acquire);
                    recognizer.OnTouchEvent(&dip)
                };
                raw.moved_beyond_slop_region = result.moved_beyond_slop_region;
                // passthrough_touch_event_queue.cc:337-352. The first move
                // region ends only after a move crosses recognizer slop.
                let mut admission = self.ports.touch_admission.lock().unwrap();
                raw.touch_start_or_first_touch_move = false;
                if raw.GetType() == Type::kTouchStart {
                    raw.touch_start_or_first_touch_move = true;
                    admission.1 = false;
                } else if raw.GetType() == Type::kTouchMove
                    && (admission.0 == Some(Type::kTouchStart)
                        || (admission.0 == Some(Type::kTouchMove) && !admission.1))
                {
                    raw.touch_start_or_first_touch_move = true;
                    admission.1 |= raw.moved_beyond_slop_region;
                }
                admission.0 = Some(raw.GetType());
                if let InputEvent::Touch(touch) = &mut input {
                    touch.moved_beyond_slop_region = raw.moved_beyond_slop_region;
                    touch.touch_start_or_first_touch_move = raw.touch_start_or_first_touch_move;
                }
            }
        }
        let queued = QueuedFrameInput {
            input,
            queued_at,
            samples,
            browser_gesture_dispatch_id,
        };
        let (dispatch_type, ack) = match &queued.input {
            InputEvent::Touch(touch) => (
                if touch.dispatch_type == InputDispatchType::kBlocking {
                    QueueDispatchType::kBlocking
                } else {
                    QueueDispatchType::kNonBlocking
                },
                if touch.compositor_nonblocking_due_to_fling {
                    InputEventResultState::kSetNonBlockingDueToFling
                } else if touch.compositor_nonblocking {
                    InputEventResultState::kSetNonBlocking
                } else {
                    InputEventResultState::kNotConsumed
                },
            ),
            InputEvent::Wheel(wheel) if wheel.compositor_nonblocking => (
                QueueDispatchType::kBlocking,
                InputEventResultState::kSetNonBlocking,
            ),
            _ => (
                QueueDispatchType::kBlocking,
                InputEventResultState::kNotConsumed,
            ),
        };
        // Keep each original acknowledgement on MainThreadEventQueue's own
        // callback list, including events merged into a later touch move.
        let callback: Option<HandledEventCallback<HostPorts>> = match &queued.input {
            InputEvent::Wheel(wheel) => {
                let requests = wheel.compositor_ack.clone();
                let ports = self.ports.clone();
                Some(Box::new(move |result, _, _, _| {
                    ports
                        .wheel_dispositions
                        .lock()
                        .unwrap()
                        .push(WheelDisposition {
                            document_generation,
                            requests,
                            result,
                        });
                }))
            }
            InputEvent::Touch(touch)
                if packet.event.Event().GetType() != Type::kTouchScrollStarted =>
            {
                let id = touch.unique_touch_event_id;
                let disposition = touch.touch_start_or_first_touch_move;
                let metadata = (dispatch_type == QueueDispatchType::kBlocking
                    && ack == InputEventResultState::kNotConsumed)
                    .then(|| *packet.event.Event().GetEventLatencyMetadata());
                let ports = self.ports.clone();
                Some(Box::new(move |result, _, _, _| {
                    let consumed = result == InputEventResultState::kConsumed;
                    let events = {
                        let mut recognizer = ports.recognizer.lock().unwrap();
                        if document_generation == ports.document_generation.load(Ordering::Acquire)
                        {
                            recognizer.OnTouchEventAckWithLatency(
                                id, consumed,
                                // events_helper.cc:11-28: set-blocking reflects the
                                // consumed ACK, independently of renderer dispatch mode.
                                consumed, metadata,
                            )
                        } else {
                            Vec::new()
                        }
                    };
                    ports.PublishGeneratedGestures(document_generation, events);
                    if disposition {
                        ports
                            .touch_dispositions
                            .lock()
                            .unwrap()
                            .push(TouchDisposition {
                                unique_touch_event_id: id,
                                consumed,
                            });
                    }
                }))
            }
            InputEvent::Gesture(_) if browser_gesture_dispatch_id.is_some() => {
                let dispatch_id = browser_gesture_dispatch_id.unwrap();
                let ports = self.ports.clone();
                Some(Box::new(move |result, _, _, _| {
                    ports.browser_gesture_dispositions.lock().unwrap().push(
                        BrowserGestureDisposition {
                            dispatch_id,
                            consumed: result == InputEventResultState::kConsumed,
                        },
                    );
                }))
            }
            _ => None,
        };
        self.queue.HandleEvent(
            WebCoalescedInputEvent::WithEvents(
                Box::new(HostPayload {
                    document_generation,
                    raw: packet.event,
                    queued,
                    widget_scale,
                }),
                packet.coalesced,
                packet.predicted,
                &packet.latency,
            ),
            dispatch_type,
            ack,
            &(),
            None,
            callback,
            false,
        );
    }
}

// No clock conversion is needed by filtering: source timestamps are opaque
// ordering values, and the original host Instant remains on every queued input.
// A queue adapter must not guess Chromium's TimeTicks epoch from Instant.
fn ConvertedEventType(input: &InputEvent) -> Option<Type> {
    Some(match input.ToWebEvent(TimeTicks::new()).ok()? {
        ConvertedWebInputEvent::Mouse(event) => event.GetType(),
        ConvertedWebInputEvent::Pointer(event) => event.GetType(),
        ConvertedWebInputEvent::Touch(event) => event.GetType(),
        ConvertedWebInputEvent::Gesture(event) => event.GetType(),
        ConvertedWebInputEvent::Wheel(event) => event.GetType(),
        ConvertedWebInputEvent::Keyboard(event) => event.GetType(),
    })
}

fn ConvertResult(
    input: &InputEvent,
    time: TimeTicks,
    widget_scale: f32,
) -> Result<Box<dyn WebInputEventPayload>, super::input_event::WebInputConversionError> {
    Ok(match input.ToWebEventWithWidgetScale(time, widget_scale)? {
        ConvertedWebInputEvent::Mouse(event) => Box::new(event),
        ConvertedWebInputEvent::Pointer(event) => Box::new(event),
        ConvertedWebInputEvent::Touch(event) => Box::new(event),
        ConvertedWebInputEvent::Gesture(event) => Box::new(event),
        ConvertedWebInputEvent::Wheel(event) => Box::new(event),
        ConvertedWebInputEvent::Keyboard(event) => Box::new(event),
    })
}

struct HostPayload {
    document_generation: u64,
    raw: Box<dyn WebInputEventPayload>,
    queued: QueuedFrameInput,
    widget_scale: f32,
}

impl HostPayload {
    fn Project(&self) -> QueuedFrameInput {
        let mut queued = self.queued.clone();
        match &mut queued.input {
            InputEvent::Pointer(pointer) => pointer.ProjectWebEvent(
                self.raw
                    .AsAny()
                    .downcast_ref::<WebPointerEvent>()
                    .expect("pointer payload"),
            ),
            InputEvent::Touch(touch) => touch.ProjectWebEvent(
                self.raw
                    .AsAny()
                    .downcast_ref::<WebTouchEvent>()
                    .expect("touch payload"),
            ),
            InputEvent::Gesture(gesture) => {
                *gesture = self
                    .raw
                    .AsAny()
                    .downcast_ref::<super::input_event::WebGestureEvent>()
                    .expect("gesture payload")
                    .clone();
            }
            _ => {}
        }
        match &mut queued.input {
            InputEvent::Pointer(pointer) => {
                pointer.position.x /= f64::from(self.widget_scale);
                pointer.position.y /= f64::from(self.widget_scale);
                pointer.width /= self.widget_scale;
                pointer.height /= self.widget_scale;
            }
            InputEvent::Touch(touch) => {
                for point in &mut touch.points {
                    point.position.x /= f64::from(self.widget_scale);
                    point.position.y /= f64::from(self.widget_scale);
                    point.radius_x /= self.widget_scale;
                    point.radius_y /= self.widget_scale;
                }
            }
            _ => {}
        }
        queued
    }
}

impl WebInputEventPayload for HostPayload {
    fn Event(&self) -> &WebInputEvent {
        self.raw.Event()
    }

    fn EventMut(&mut self) -> &mut WebInputEvent {
        self.raw.EventMut()
    }

    fn AsAny(&self) -> &dyn Any {
        self
    }

    fn AsAnyMut(&mut self) -> &mut dyn Any {
        self
    }

    fn Clone(&self) -> Box<dyn WebInputEventPayload> {
        Box::new(Self {
            document_generation: self.document_generation,
            raw: self.raw.Clone(),
            queued: self.queued.clone(),
            widget_scale: self.widget_scale,
        })
    }

    fn CanCoalesce(&self, other: &dyn WebInputEventPayload) -> bool {
        let Some(other) = other.AsAny().downcast_ref::<Self>() else {
            return false;
        };
        if self.widget_scale != other.widget_scale
            || self.document_generation != other.document_generation
        {
            return false;
        }
        // These routing facts have no WebInputEvent representation. Everything
        // represented by Web events is compared by the source payload itself.
        let same_route = match (&self.queued.input, &other.queued.input) {
            (InputEvent::Mouse(a), InputEvent::Mouse(b)) => a.target_node_id == b.target_node_id,
            (InputEvent::Pointer(a), InputEvent::Pointer(b)) => {
                a.target_node_id == b.target_node_id
            }
            (InputEvent::Touch(_), InputEvent::Touch(_)) => true,
            (InputEvent::Wheel(a), InputEvent::Wheel(b)) => {
                a.target_node_id == b.target_node_id
                    && a.compositor_owned == b.compositor_owned
                    && a.compositor_nonblocking == b.compositor_nonblocking
                    && a.compositor_ack.is_empty() == b.compositor_ack.is_empty()
            }
            _ => false,
        };
        same_route && self.raw.CanCoalesce(other.raw.as_ref())
    }

    fn Coalesce(&mut self, other: &dyn WebInputEventPayload) {
        let other = other
            .AsAny()
            .downcast_ref::<Self>()
            .expect("host input payload");
        let acknowledgements = match (&self.queued.input, &other.queued.input) {
            (InputEvent::Wheel(a), InputEvent::Wheel(b)) => {
                let mut ids = a.compositor_ack.clone();
                ids.extend_from_slice(&b.compositor_ack);
                ids
            }
            _ => Vec::new(),
        };
        self.raw.Coalesce(other.raw.as_ref());
        let queued_at = self.queued.queued_at.min(other.queued.queued_at);
        let samples = self.queued.samples.saturating_add(other.queued.samples);
        self.queued = QueuedFrameInput {
            input: other.queued.input.clone(),
            queued_at,
            samples,
            browser_gesture_dispatch_id: other.queued.browser_gesture_dispatch_id,
        };
        if let InputEvent::Wheel(wheel) = &mut self.queued.input {
            wheel.compositor_ack = acknowledgements;
            let raw = self
                .raw
                .AsAny()
                .downcast_ref::<WebMouseWheelEvent>()
                .expect("wheel input payload");
            // Source wheel coalescing owns delta arithmetic and synthetic
            // Began+Changed phase retention. Project its result into host units.
            wheel.delta.x = -f64::from(raw.delta_x);
            wheel.delta.y = -f64::from(raw.delta_y);
            if raw.has_synthetic_phase {
                wheel.phase = match raw.phase {
                    Phase::kPhaseBegan => WheelPhase::kBegan,
                    Phase::kPhaseChanged => WheelPhase::kChanged,
                    Phase::kPhaseEnded => WheelPhase::kEnded,
                    Phase::kPhaseCancelled => WheelPhase::kCancelled,
                    _ => WheelPhase::kNone,
                };
            }
        }
    }
}

// Host tracing keeps the original Instant and sample count on HostPayload;
// these source-associated values preserve acknowledgement and metrics types.
#[derive(Clone, Default)]
struct FrameLatency {
    trace_id: i64,
    coalesced: bool,
}

impl QueueLatencyInfo for FrameLatency {
    fn trace_id(&self) -> i64 {
        self.trace_id
    }
    fn set_trace_id(&mut self, id: i64) {
        self.trace_id = id;
    }
    fn set_coalesced(&mut self) {
        self.coalesced = true;
    }
}

struct FrameMetrics;
impl QueueEventMetrics for FrameMetrics {
    fn CreateFromExistingForRawEvent(_: EventType, _: Option<&Self>) -> Option<Self> {
        None
    }
    fn IsScrollUpdate(&self) -> bool {
        false
    }
    fn CoalesceWith(&mut self, _: &Self) {}
}

struct HostPorts {
    document_generation: AtomicU64,
    needs_main_frame: AtomicBool,
    owner_thread: std::thread::ThreadId,
    needs_dispatch_task: AtomicBool,
    widget_scale: AtomicU32,
    clock: Mutex<Arc<dyn super::host_event_clock::HostInputClock>>,
    touch_dispositions: Mutex<Vec<TouchDisposition>>,
    wheel_dispositions: Mutex<Vec<WheelDisposition>>,
    recognizer: Mutex<FilteredGestureProvider>,
    generated_gestures: Mutex<Vec<WebGestureEvent>>,
    touch_admission: Mutex<(Option<Type>, bool)>,
    browser_gesture_dispositions: Mutex<Vec<BrowserGestureDisposition>>,
}
impl HostPorts {
    fn PublishGeneratedGestures(&self, generation: u64, events: Vec<WebGestureEvent>) {
        let scale = f32::from_bits(self.widget_scale.load(Ordering::Relaxed));
        let mut generated = self.generated_gestures.lock().unwrap();
        if generation != self.document_generation.load(Ordering::Acquire) {
            return;
        }
        generated.extend(events.into_iter().map(|mut event| {
            event.SetFrameScale(1.0 / scale);
            event.SetFrameTranslate(Default::default());
            event
        }));
    }
    fn TimeForInstant(&self, instant: Instant) -> TimeTicks {
        self.clock.lock().unwrap().TimeForInstant(instant)
    }
}

impl MainThreadEventQueueClient<HostPorts> for HostPorts {
    fn HandleInputEvent(
        &self,
        _: &WebCoalescedInputEvent<FrameLatency>,
        _: Option<FrameMetrics>,
        _: HandledEventCallback<HostPorts>,
    ) -> bool {
        unreachable!("Page supplies the synchronous borrowed dispatch port")
    }
    fn InputEventsDispatched(&self, _: bool) {}
    fn SetNeedsMainFrame(&self, _: BeginMainFrameReason, _: bool) {
        self.needs_main_frame.store(true, Ordering::Relaxed);
    }
    fn RequestedMainFramePending(&self) -> bool {
        self.needs_main_frame.load(Ordering::Relaxed)
    }
}
impl MainThreadEventQueueServices for HostPorts {
    type Latency = FrameLatency;
    type Attribution = ();
    type Metrics = FrameMetrics;
    type Overscroll = ();
    type TouchAction = ();
    type PayloadAccess = Self;

    fn Now(&self) -> TimeTicks {
        self.TimeForInstant(Instant::now())
    }
    fn MainThreadBelongsToCurrentThread(&self) -> bool {
        self.owner_thread == std::thread::current().id()
    }
    fn CompositorThreadBelongsToCurrentThread(&self) -> bool {
        self.MainThreadBelongsToCurrentThread()
    }
    fn PostTaskToMainThread(&self, _: Box<dyn FnOnce() + Send>) {
        // Page executes this queue's task through DispatchEventsWith so it can
        // borrow the document executor synchronously. Retain the task request,
        // rather than an Arc callback cycle back to this queue owner.
        debug_assert!(self.MainThreadBelongsToCurrentThread());
        self.needs_dispatch_task.store(true, Ordering::Relaxed);
    }
    fn WillPostInputEventToMainThread(&self, _: Type, _: &()) {}
    fn WillHandleInputEventOnMainThread(&self, _: Type, _: &()) {}
    fn DidHandleInputEventOnMainThread(&self, _: &WebInputEvent, _: bool, _: bool) {}
}

impl QueuePayloadAccess for HostPorts {
    fn TouchEvent(event: &dyn WebInputEventPayload) -> &WebTouchEvent {
        event
            .AsAny()
            .downcast_ref::<HostPayload>()
            .expect("host input payload")
            .raw
            .AsAny()
            .downcast_ref::<WebTouchEvent>()
            .expect("touch payload")
    }
    fn TouchEventMut(event: &mut dyn WebInputEventPayload) -> &mut WebTouchEvent {
        event
            .AsAnyMut()
            .downcast_mut::<HostPayload>()
            .expect("host input payload")
            .raw
            .AsAnyMut()
            .downcast_mut::<WebTouchEvent>()
            .expect("touch payload")
    }
    fn PointerFromMouse(event: &dyn WebInputEventPayload) -> WebPointerEvent {
        let mouse = event
            .AsAny()
            .downcast_ref::<HostPayload>()
            .expect("host input payload")
            .raw
            .AsAny()
            .downcast_ref::<WebMouseEvent>()
            .expect("mouse payload");
        WebPointerEvent::FromMouse(Type::kPointerRawUpdate, mouse)
    }
    fn WrapPointerRawUpdate(
        event: &dyn WebInputEventPayload,
        pointer: WebPointerEvent,
    ) -> Box<dyn WebInputEventPayload> {
        let host = event
            .AsAny()
            .downcast_ref::<HostPayload>()
            .expect("host input payload");
        let pointer_id = u32::try_from(pointer.pointer_properties.id).expect("host pointer ID");
        let (pointer_type, modifiers, target_node_id) = match &host.queued.input {
            InputEvent::Mouse(mouse) => {
                (PointerType::kMouse, mouse.modifiers, mouse.target_node_id)
            }
            InputEvent::Touch(touch) => {
                let point = touch
                    .points
                    .iter()
                    .find(|point| point.pointer_id == pointer_id)
                    .expect("source touch ID");
                (point.pointer_type, touch.modifiers, None)
            }
            _ => unreachable!("source generates raw update from mouse or touch input"),
        };
        let mut input = PointerEvent {
            pointer_id,
            pointer_type,
            modifiers,
            target_node_id,
            ..Default::default()
        };
        input.ProjectWebEvent(&pointer);
        Box::new(HostPayload {
            document_generation: host.document_generation,
            raw: Box::new(pointer),
            widget_scale: host.widget_scale,
            queued: QueuedFrameInput {
                input: InputEvent::Pointer(input),
                queued_at: host.queued.queued_at,
                samples: host.queued.samples,
                browser_gesture_dispatch_id: None,
            },
        })
    }
    fn WheelDispatchType(event: &dyn WebInputEventPayload) -> DispatchType {
        event
            .AsAny()
            .downcast_ref::<HostPayload>()
            .expect("host input payload")
            .raw
            .AsAny()
            .downcast_ref::<WebMouseWheelEvent>()
            .expect("wheel payload")
            .dispatch_type
    }
    fn SetWheelDispatchType(event: &mut dyn WebInputEventPayload, value: DispatchType) {
        event
            .AsAnyMut()
            .downcast_mut::<HostPayload>()
            .expect("host input payload")
            .raw
            .AsAnyMut()
            .downcast_mut::<WebMouseWheelEvent>()
            .expect("wheel payload")
            .dispatch_type = value;
    }
    fn GestureScrollBeginCursorControl(event: &dyn WebInputEventPayload) -> bool {
        event
            .AsAny()
            .downcast_ref::<HostPayload>()
            .expect("host input payload")
            .raw
            .AsAny()
            .downcast_ref::<super::input_event::WebGestureEvent>()
            .expect("gesture payload")
            .data
            .scroll_begin
            .cursor_control
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::input_event::{MouseEvent, WheelEvent};
    use std::time::Duration;

    #[test]
    fn source_queue_preserves_synthetic_phase_samples_and_dispatch_snapshot() {
        let mut queue = FrameAlignedInputQueue::default();
        let first_time = Instant::now();
        let later_time = first_time + Duration::from_millis(1);
        let wheel = |phase, delta| {
            InputEvent::Wheel(WheelEvent {
                phase,
                has_synthetic_phase: true,
                delta: super::super::input_event::Offset { x: 0.0, y: delta },
                ..Default::default()
            })
        };

        // Blink allows synthetic Began + Changed, retains Began, and searches
        // through another continuous event class. The former host algorithm
        // rejected this pair before reaching WebMouseWheelEvent::Coalesce.
        queue.Push(wheel(WheelPhase::kBegan, 10.0), first_time, 2);
        queue.Push(InputEvent::Mouse(MouseEvent::default()), later_time, 1);
        queue.Push(wheel(WheelPhase::kChanged, 20.0), later_time, 3);
        assert!(queue.HasWheel());
        let mut dispatched = Vec::new();
        queue.Dispatch(|entry| {
            dispatched.push(entry);
            false
        });
        let mut dispatched = dispatched.into_iter();
        let merged = dispatched.next().unwrap();
        assert_eq!(merged.queued_at, first_time);
        assert_eq!(merged.samples, 5);
        assert!(matches!(merged.input, InputEvent::Wheel(event)
            if event.phase == WheelPhase::kBegan && event.delta.y == 30.0));
        assert!(matches!(
            dispatched.next().unwrap().input,
            InputEvent::Mouse(_)
        ));
        assert!(queue.IsEmpty());

        queue.Push(wheel(WheelPhase::kChanged, 1.0), first_time, 1);
        let reentrant = queue.clone();
        queue.Dispatch(|entry| {
            assert_eq!(entry.samples, 1);
            // A newly admitted event belongs to the following source snapshot.
            let mut reentrant = reentrant.clone();
            reentrant.Push(wheel(WheelPhase::kChanged, 2.0), later_time, 1);
            false
        });
        assert!(!queue.IsEmpty());
        queue.Dispatch(|entry| {
            assert_eq!(entry.samples, 1);
            false
        });
        assert!(queue.IsEmpty());
    }
}
