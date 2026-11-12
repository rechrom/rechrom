// Chromium components/input/fling_controller.h:29-215; .cc:19-481.
// Sequenced input owner. Client calls are returned as typed, ordered effects;
// the embedding supplies the clock/viewport and schedules animation ticks.
use super::tap_suppression_controller::{Config as TapConfig, TapSuppressionController};
use crate::{
    base::time::time::TimeTicks,
    public::{
        common::input::{
            web_gesture_event::{GestureDevice, InertialPhaseState, WebGestureEvent},
            web_input_event::{DispatchType, Type},
            web_mouse_wheel_event::{Phase, ScrollGranularity, WebMouseWheelEvent},
            web_pointer_properties::PointerType,
        },
        mojom::input::input_event_result::InputEventResultState,
    },
    ui::events::{
        blink::fling_booster::FlingBooster, gestures::web_gesture_curve_impl::WebGestureCurveImpl,
    },
};
use foundation::gfx_geometry::{PointF, Vector2dF};

#[derive(Clone, Copy, Default)]
pub struct Config {
    pub touchpad_tap_suppression: TapConfig,
    pub touchscreen_tap_suppression: TapConfig,
    pub progress_fling_on_fling_start: bool,
    pub resample_scroll_events_for_fling: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FlingSequence {
    pub unique_touch_event_id: u32,
    pub primary_unique_touch_event_id: u32,
}
#[derive(Clone, Debug)]
pub enum GeneratedFlingEvent {
    Wheel(WebMouseWheelEvent),
    Gesture(WebGestureEvent),
}
#[derive(Default)]
pub struct FlingControllerOutput {
    pub consumed: bool,
    /// InputRouterImpl.cc:170-180: dispatch completion precedes this browser
    /// consumed ACK; the original source event retains sequence identity.
    pub acknowledged: Option<WebGestureEvent>,
    pub generated: Vec<GeneratedFlingEvent>,
    pub sequence: FlingSequence,
    pub needs_animation_frame: bool,
    pub stopped_flinging: bool,
}
struct ActiveFlingParameters {
    velocity: Vector2dF,
    point: PointF,
    global_point: PointF,
    modifiers: i32,
    source_device: GestureDevice,
    start_time: TimeTicks,
    fling_start_event_time: TimeTicks,
    frame_scale: f32,
    frame_translate: Vector2dF,
}
impl Default for ActiveFlingParameters {
    fn default() -> Self {
        Self {
            velocity: Vector2dF::default(),
            point: PointF::default(),
            global_point: PointF::default(),
            modifiers: 0,
            source_device: GestureDevice::kUninitialized,
            start_time: TimeTicks::new(),
            fling_start_event_time: TimeTicks::new(),
            frame_scale: 1.,
            frame_translate: Vector2dF::default(),
        }
    }
}
pub struct FlingController {
    config: Config,
    fling_booster: FlingBooster,
    touchpad_tap_suppression: TapSuppressionController,
    touchscreen_tap_suppression: TapSuppressionController,
    fling_curve: Option<WebGestureCurveImpl>,
    current_fling_parameters: ActiveFlingParameters,
    last_progress_time: TimeTicks,
    last_seen_scroll_update: TimeTicks,
    last_wheel_event_consumed: bool,
    sequence: FlingSequence,
}
impl Default for FlingController {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
impl FlingController {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            fling_booster: FlingBooster::default(),
            touchpad_tap_suppression: TapSuppressionController::new(
                config.touchpad_tap_suppression,
            ),
            touchscreen_tap_suppression: TapSuppressionController::new(
                config.touchscreen_tap_suppression,
            ),
            fling_curve: None,
            current_fling_parameters: ActiveFlingParameters::default(),
            last_progress_time: TimeTicks::new(),
            last_seen_scroll_update: TimeTicks::new(),
            last_wheel_event_consumed: false,
            sequence: FlingSequence::default(),
        }
    }
    pub fn fling_in_progress(&self) -> bool {
        self.fling_curve.is_some()
    }
    pub fn CurrentFlingVelocity(&self) -> Vector2dF {
        self.current_fling_parameters.velocity
    }
    pub fn OnWheelEventAck(&mut self, result: InputEventResultState) {
        self.last_wheel_event_consumed = result == InputEventResultState::kConsumed;
    }
    pub fn ShouldSuppressMouseDown(&mut self, now: TimeTicks) -> bool {
        self.touchpad_tap_suppression.ShouldSuppressMouseDown(now)
    }
    pub fn ShouldSuppressMouseUp(&self) -> bool {
        self.touchpad_tap_suppression.ShouldSuppressMouseUp()
    }

    // Source .cc:62-154. Every incoming gesture passes this owner, including
    // non-scroll taps, so cancellation suppression and boosting stay ordered.
    pub fn ObserveAndMaybeConsumeGestureEvent(
        &mut self,
        event: &WebGestureEvent,
        now: TimeTicks,
        viewport_empty: bool,
    ) -> FlingControllerOutput {
        let mut output = FlingControllerOutput::default();
        if event.GetType() == Type::kGestureFlingCancel && !self.fling_in_progress() {
            output.consumed = true;
        } else {
            let filtered = match event.GetType() {
                Type::kGestureFlingCancel => {
                    if event.SourceDevice() == GestureDevice::kTouchscreen {
                        self.touchscreen_tap_suppression
                            .GestureFlingCancelStoppedFling(now);
                    } else if event.SourceDevice() == GestureDevice::kTouchpad {
                        self.touchpad_tap_suppression
                            .GestureFlingCancelStoppedFling(now);
                    }
                    false
                }
                Type::kGestureTapDown
                | Type::kGestureShowPress
                | Type::kGestureTapUnconfirmed
                | Type::kGestureTapCancel
                | Type::kGestureTap
                | Type::kGestureDoubleTap
                | Type::kGestureLongPress
                | Type::kGestureLongTap
                | Type::kGestureTwoFingerTap => {
                    event.SourceDevice() == GestureDevice::kTouchscreen
                        && self
                            .touchscreen_tap_suppression
                            .FilterTouchscreenTapEvent(event.GetType(), now)
                }
                _ => false,
            };
            if filtered {
                output.consumed = true;
            } else {
                match event.GetType() {
                    Type::kGestureScrollUpdate => self.last_seen_scroll_update = event.TimeStamp(),
                    Type::kGestureScrollBegin | Type::kGestureScrollEnd => {
                        self.last_seen_scroll_update = TimeTicks::new()
                    }
                    _ => {}
                }
                self.fling_booster.ObserveGestureEvent(event);
                if event.GetType() == Type::kGestureFlingStart {
                    self.ProcessGestureFlingStart(event, now, viewport_empty, &mut output);
                    output.consumed = true;
                } else if event.GetType() == Type::kGestureFlingCancel {
                    self.EndCurrentFling(event.TimeStamp(), &mut output);
                    output.consumed = true;
                }
            }
        }
        if output.consumed {
            output.acknowledged = Some(event.clone());
        }
        output.sequence = self.sequence;
        output
    }
    // Source .cc:156-207,414-465. The active curve is replaced at the same
    // point as the source; GFC deliberately preserves the booster.
    fn ProcessGestureFlingStart(
        &mut self,
        event: &WebGestureEvent,
        now: TimeTicks,
        viewport_empty: bool,
        output: &mut FlingControllerOutput,
    ) {
        if event.SourceDevice() == GestureDevice::kTouchpad && self.last_wheel_event_consumed {
            return;
        }
        debug_assert!(matches!(
            event.SourceDevice(),
            GestureDevice::kTouchscreen
                | GestureDevice::kTouchpad
                | GestureDevice::kSyntheticAutoscroll
        ));
        let velocity = self.fling_booster.GetVelocityForFlingStart(event);
        self.sequence = FlingSequence {
            unique_touch_event_id: event.unique_touch_event_id,
            primary_unique_touch_event_id: event.primary_unique_touch_event_id,
        };
        self.current_fling_parameters = ActiveFlingParameters {
            velocity,
            point: event.PositionInWidget(),
            global_point: event.PositionInScreen(),
            modifiers: event.GetModifiers(),
            source_device: event.SourceDevice(),
            frame_scale: event.FrameScale(),
            frame_translate: event.FrameTranslate(),
            start_time: if event.SourceDevice() == GestureDevice::kSyntheticAutoscroll
                || self.last_seen_scroll_update.is_null()
            {
                event.TimeStamp()
            } else {
                self.last_seen_scroll_update
            },
            fling_start_event_time: event.TimeStamp(),
        };
        if ((velocity.x() == 0. && velocity.y() == 0.)
            && event.SourceDevice() != GestureDevice::kSyntheticAutoscroll)
            || viewport_empty
        {
            self.fling_booster.Reset();
            self.EndCurrentFling(
                if viewport_empty {
                    self.last_seen_scroll_update
                } else {
                    event.TimeStamp()
                },
                output,
            );
            return;
        }
        self.fling_curve = Some(WebGestureCurveImpl::CreateFromDefaultPlatformCurve(
            event.SourceDevice(),
            velocity,
        ));
        self.last_progress_time = TimeTicks::new();
        if self.config.progress_fling_on_fling_start {
            self.ProgressFlingInto(now, None, output);
        } else {
            output.needs_animation_frame = true;
        }
    }
    pub fn ProgressFling(
        &mut self,
        now: TimeTicks,
        first_coalesced_frame_begin_time: Option<TimeTicks>,
    ) -> FlingControllerOutput {
        let mut output = FlingControllerOutput::default();
        self.ProgressFlingInto(now, first_coalesced_frame_begin_time, &mut output);
        output.sequence = self.sequence;
        output
    }
    // Source .cc:209-271. Signed source ticks are used without clock rebasing.
    fn ProgressFlingInto(
        &mut self,
        now: TimeTicks,
        first: Option<TimeTicks>,
        output: &mut FlingControllerOutput,
    ) {
        if self.fling_curve.is_none() {
            return;
        }
        if self.last_progress_time.is_null() {
            if self.current_fling_parameters.start_time.is_null() {
                self.current_fling_parameters.start_time = now;
                output.needs_animation_frame = true;
                return;
            }
            if now.ToInternalValue()
                >= self.current_fling_parameters.start_time.ToInternalValue() + 33_333
            {
                self.current_fling_parameters.start_time =
                    TimeTicks::FromInternalValue(now.ToInternalValue() - 16_666);
            }
        }
        if now < self.last_progress_time || now <= self.current_fling_parameters.start_time {
            output.needs_animation_frame = true;
            return;
        }
        let mut delta = Vector2dF::default();
        let elapsed = (now.ToInternalValue()
            - self.current_fling_parameters.start_time.ToInternalValue())
            as f64
            / 1_000_000.;
        let active = self.fling_curve.as_mut().unwrap().Advance(
            elapsed,
            &mut self.current_fling_parameters.velocity,
            &mut delta,
        );
        if !active
            && self.current_fling_parameters.source_device != GestureDevice::kSyntheticAutoscroll
        {
            self.fling_booster.Reset();
            self.EndCurrentFling(now, output);
            return;
        }
        if delta.x().abs() > 0.1 || delta.y().abs() > 0.1 {
            let generation = if self.last_progress_time.is_null()
                && self.config.resample_scroll_events_for_fling
            {
                self.current_fling_parameters.fling_start_event_time
            } else {
                first.unwrap_or(now)
            };
            self.GenerateAndSendFlingProgressEvents(generation, delta, output);
            self.last_progress_time = now;
        }
        output.needs_animation_frame = self.fling_curve.is_some();
    }
    pub fn StopFling(&mut self, now: TimeTicks) -> FlingControllerOutput {
        self.fling_booster.Reset();
        let mut output = FlingControllerOutput::default();
        if self.fling_in_progress() {
            self.EndCurrentFling(now, &mut output);
        }
        output.sequence = self.sequence;
        output
    }
    // Source .cc:280-334. Generated values are canonical; adapters must not
    // round-trip them through a host projection and ToWebEvent.
    fn GenerateWheel(&self, now: TimeTicks, delta: Vector2dF, phase: Phase) -> GeneratedFlingEvent {
        let p = &self.current_fling_parameters;
        let mut event = WebMouseWheelEvent::new(Type::kMouseWheel, p.modifiers, now);
        event.delta_units = ScrollGranularity::kScrollByPrecisePixel;
        event.delta_x = delta.x();
        event.delta_y = delta.y();
        event.momentum_phase = phase;
        event.has_synthetic_phase = true;
        event
            .mouse_event
            .pointer_properties
            .SetPositionInWidget(p.point);
        event
            .mouse_event
            .pointer_properties
            .SetPositionInScreen(p.global_point);
        event.dispatch_type = DispatchType::kEventNonBlocking;
        event.SetFrameScale(p.frame_scale);
        event.SetFrameTranslate(p.frame_translate);
        GeneratedFlingEvent::Wheel(event)
    }
    fn GenerateGesture(
        &self,
        now: TimeTicks,
        event_type: Type,
        delta: Vector2dF,
    ) -> GeneratedFlingEvent {
        let p = &self.current_fling_parameters;
        let mut event = WebGestureEvent::new(event_type, p.modifiers, now, p.source_device);
        event.SetPositionInWidget(p.point);
        event.SetPositionInScreen(p.global_point);
        event.primary_pointer_type = PointerType::kTouch;
        event.SetFrameScale(p.frame_scale);
        event.SetFrameTranslate(p.frame_translate);
        if event_type == Type::kGestureScrollUpdate {
            event.data.scroll_update.delta_x = delta.x();
            event.data.scroll_update.delta_y = delta.y();
            event.data.scroll_update.delta_x_unconstrained = delta.x();
            event.data.scroll_update.delta_y_unconstrained = delta.y();
            event.data.scroll_update.inertial_phase = InertialPhaseState::kMomentum;
        } else {
            debug_assert_eq!(event_type, Type::kGestureScrollEnd);
            event.data.scroll_end.inertial_phase = InertialPhaseState::kMomentum;
            event.data.scroll_end.generated_by_fling_controller = true;
        }
        GeneratedFlingEvent::Gesture(event)
    }
    // Source .cc:336-413. Touchpad produces Wheel; touchscreen/autoscroll GSU.
    fn GenerateAndSendFlingProgressEvents(
        &mut self,
        now: TimeTicks,
        delta: Vector2dF,
        output: &mut FlingControllerOutput,
    ) {
        output
            .generated
            .push(match self.current_fling_parameters.source_device {
                GestureDevice::kTouchpad => self.GenerateWheel(
                    now,
                    delta,
                    if self.last_progress_time.is_null() {
                        Phase::kPhaseBegan
                    } else {
                        Phase::kPhaseChanged
                    },
                ),
                GestureDevice::kTouchscreen | GestureDevice::kSyntheticAutoscroll => {
                    self.GenerateGesture(now, Type::kGestureScrollUpdate, delta)
                }
                _ => panic!("unsupported fling source device"),
            });
        self.fling_booster
            .ObserveProgressFling(self.current_fling_parameters.velocity);
    }
    fn EndCurrentFling(&mut self, now: TimeTicks, output: &mut FlingControllerOutput) {
        self.last_progress_time = TimeTicks::new();
        output
            .generated
            .push(match self.current_fling_parameters.source_device {
                GestureDevice::kTouchpad => {
                    self.GenerateWheel(now, Vector2dF::default(), Phase::kPhaseEnded)
                }
                GestureDevice::kTouchscreen | GestureDevice::kSyntheticAutoscroll => {
                    self.GenerateGesture(now, Type::kGestureScrollEnd, Vector2dF::default())
                }
                _ => panic!("unsupported fling source device"),
            });
        self.current_fling_parameters = ActiveFlingParameters::default();
        output.stopped_flinging = self.fling_curve.is_some();
        self.fling_curve = None;
        output.needs_animation_frame = false;
    }
}
