//! Host input values -> cc impl input (S1b-7 follow-up). These conversions
//! are shared by every owner of a `compositor::CompositorEngine`: the
//! embedding's impl-thread runner and the Page's synchronous presenter.
//! Chromium performs them in InputRouter (ui::ScaleWebInputEvent) and
//! InputHandlerProxy before cc's InputHandler sees the event.
use super::*;
use compositor::{CompositorConfig, CompositorExecutor, CompositorInput};

/// The move-only frame an impl owner presents (CompositorEngine output).
pub type PageCompositorFrame = compositor::CompositorFrameSubmission;

/// The impl half's injected task runner and clock (LayerTreeHostImpl's
/// TaskRunnerProvider, cc/trees/layer_tree_host_impl.cc Create) over the
/// Page's executor and its TimeTicks domain.
impl CompositorExecutor for PageCompositorExecutor {
    fn Now(&self) -> CompositorTimeTicks {
        PageCompositorExecutor::Now(self)
    }

    fn Execute(&self, task: Box<dyn FnOnce() + Send>) {
        PageCompositorExecutor::Execute(self, task)
    }

    fn ExecuteAfter(&self, delay: CompositorTimeDelta, task: Box<dyn FnOnce() + Send>) {
        let delay = delay.InMicroseconds().max(0) as u64;
        let _ = self
            .executor
            .execute_after(std::time::Duration::from_micros(delay), task);
    }

    fn RunsTasksInCurrentSequence(&self) -> bool {
        std::thread::current().id() == self.sequence
    }
}

/// The InputHandlerProxy call of an input-queue message, made by the impl
/// owner when the compositor task runner runs it (InputHandlerProxy::
/// ContinueScrollBeginAfterMainThreadHitTest, the wheel DOM
/// acknowledgement, the injected scrollbar gesture).
pub fn ImplInput(input: InputToImpl) -> CompositorInput {
    match input {
        InputToImpl::ScrollbarGesture {
            scroll_element_id,
            delta,
            granularity,
            event_timestamp,
            phase,
        } => CompositorInput::ScrollbarGesture {
            scroll_element_id,
            delta,
            granularity,
            event_timestamp,
            phase,
        },
        InputToImpl::WheelDOMAcknowledgement {
            request,
            default_prevented,
        } => CompositorInput::WheelDOMAcknowledgement {
            request,
            default_prevented,
        },
        InputToImpl::ContinueWheelScroll {
            request,
            scroll_element_id,
        } => CompositorInput::ContinueWheelScroll {
            request,
            scroll_element_id,
        },
    }
}

/// Rejects a gesture the impl input handler cannot take, on the sender's
/// side, before it is queued.
pub(crate) fn ValidateGestureCompositorInput(
    gesture: &interaction::input_event::WebGestureEvent,
) -> io::Result<()> {
    GestureCompositorInput(gesture).map(drop)
}

/// The synchronous InputHandlerProxy results of one impl input call, sorted
/// for the input owner, plus the main-thread effects the caller routes to
/// the Page.
#[derive(Default)]
pub struct CompositorWheelResult {
    pub dom_ack: Option<u64>,
    /// Original gesture completions, including deferred hit-test dispatches.
    pub gesture_acks: Vec<(u64, bool)>,
    pub compositor_owned: bool,
    pub submitted: bool,
    pub effects: Vec<CompositorMainThreadEffect>,
}
impl CompositorWheelResult {
    pub fn AddInputEffect(&mut self, effect: compositor::CompositorInputEffect) {
        use compositor::CompositorInputEffect as E;
        match effect {
            E::GestureDisposition {
                dispatch_id,
                handled,
            } => self.gesture_acks.push((dispatch_id, handled)),
            E::WheelDisposition { compositor_owned } => self.compositor_owned |= compositor_owned,
            E::WheelRequiresDOMAcknowledgement { request } => {
                assert!(
                    self.dom_ack.replace(request).is_none(),
                    "one original wheel acknowledgement"
                );
            }
            // WidgetInputHandlerManager's main-thread hit test.
            E::WheelRequiresMainThreadHitTest { request, position } => self
                .effects
                .push(CompositorMainThreadEffect::WheelRequiresMainThreadHitTest {
                    request,
                    position,
                }),
        }
    }
}

/// Chromium's InputRouter scales host CSS/DIP coordinates before the
/// renderer sees a WebInputEvent. The active compositor tree corresponds to
/// the last committed artifact's layout zoom (`scale`).
pub fn InputWidgetScale(scale: f64) -> f32 {
    if scale.is_finite() && scale > 0.0 && scale <= f32::MAX as f64 {
        scale as f32
    } else {
        1.0
    }
}

/// Source-generated wheel retains its original TimeTicks through both the
/// compositor boundary and the document's canonical input packet. Returns
/// the host wheel and its impl input.
pub fn CanonicalFlingWheelInput(
    event: &interaction::input::public::common::input::web_mouse_wheel_event::WebMouseWheelEvent,
) -> (interaction::input_event::WheelEvent, CompositorInput) {
    let wheel = interaction::adapter::generated_fling_input::WheelHostSidecar(event);
    let timestamp = CompositorTimeTicks::FromMicroseconds(event.TimeStamp().ToInternalValue());
    // FlingController's canonical WebMouseWheelEvent already carries the
    // root-widget position produced by InputRouter's one-time scaling.
    let (input, _) = WheelCompositorInput(&wheel, timestamp, 1.0);
    (wheel, input)
}

/// The gesture with its input owner's dispatch identity.
pub fn GestureCompositorInputWithDispatchId(
    gesture: &interaction::input_event::WebGestureEvent,
    dispatch_id: u64,
) -> io::Result<CompositorInput> {
    let mut input = GestureCompositorInput(gesture)?;
    match &mut input {
        CompositorInput::ScrollGesture(gesture) => gesture.dispatch_id = dispatch_id,
        CompositorInput::PinchGesture {
            dispatch_id: id, ..
        } => *id = dispatch_id,
        _ => unreachable!(),
    }
    Ok(input)
}

/// The Page's presentation feedback as cc FrameTimingDetails, or the
/// main-thread effect of a presentation without a timestamp.
pub fn PresentationFeedbackDetails(
    feedback: PageFramePresentationFeedback,
    clock: PageCompositorClockMapping,
) -> Result<(u32, viz_common::frame_timing_details::FrameTimingDetails), CompositorMainThreadEffect> {
    let PageFramePresentationFeedback {
        surface_id,
        viz_submission,
        frame_token,
        frame_id,
        outcome,
    } = feedback;
    let PagePresentationOutcome::Presented {
        timestamp,
        interval,
        flags,
    } = outcome
    else {
        let PagePresentationOutcome::Unavailable(reason) = outcome else {
            unreachable!()
        };
        let reason = match reason {
            PagePresentationUnavailableReason::BackendHasNoTimestamp => {
                PresentationUnavailableReason::BackendHasNoTimestamp
            }
            PagePresentationUnavailableReason::OffscreenTarget => {
                PresentationUnavailableReason::OffscreenTarget
            }
            PagePresentationUnavailableReason::DiscardedBeforePresentation => {
                PresentationUnavailableReason::DiscardedBeforePresentation
            }
        };
        return Err(CompositorMainThreadEffect::PresentationFeedbackUnavailable {
            frame_token,
            surface_id,
            viz_submission,
            reason,
        });
    };
    let mut details = viz_common::frame_timing_details::FrameTimingDetails::default();
    details.frame_id = frame_id;
    details.presentation_feedback.timestamp = clock.TimeTicks(timestamp);
    details.presentation_feedback.interval =
        CompositorTimeDelta::Microseconds(interval.as_micros().min(i64::MAX as u128) as i64);
    details.presentation_feedback.flags = flags;
    Ok((frame_token, details))
}

/// The impl half's injected services (`CompositorEngine::new`) for an
/// embedding: the impl task runner and raster worker executors, and a
/// RasterEngine as raster and UI resource provider. Returns the clock
/// mapping of the impl task runner.
pub fn NewCompositorServices(
    compositor: Arc<dyn executor::Executor>,
    raster: Arc<dyn executor::Executor>,
) -> (compositor::CompositorServices, PageCompositorClockMapping) {
    let executor = Arc::new(PageCompositorExecutor::new(compositor));
    let clock = PageCompositorClockMapping {
        origin: executor.origin,
    };
    (
        CompositorServicesFor(
            executor,
            Arc::new(PageCompositorExecutor::new(raster)),
            &NewPageRasterEngine(),
        ),
        clock,
    )
}

/// The impl half's configuration for an embedding's initial
/// `width`x`height`@`scale` widget.
pub fn EmbedderCompositorConfig(
    width: u32,
    height: u32,
    scale: f64,
    transparent_background: bool,
) -> io::Result<CompositorConfig> {
    let viewport = PageDomainServices::InitialCompositorViewport(width, height, scale)?;
    Ok(CompositorConfig {
        transparent_background,
        ..PageDomainServices::CompositorConfig(viewport)
    })
}

/// A Page's RasterEngine (one command-buffer identity per engine).
pub(super) fn NewPageRasterEngine() -> RasterEngine {
    static NEXT_COMMAND_BUFFER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let command_buffer = NEXT_COMMAND_BUFFER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    RasterEngine::new(command_buffer, 64 * 1024 * 1024)
}

pub(super) fn CompositorServicesFor(
    executor: Arc<PageCompositorExecutor>,
    raster_executor: Arc<PageCompositorExecutor>,
    raster_engine: &RasterEngine,
) -> compositor::CompositorServices {
    let raster_provider = Arc::new(raster_engine.clone());
    let ui_raster = raster_provider.clone();
    compositor::CompositorServices {
        compositor: executor,
        raster: raster_executor,
        raster_provider,
        ui_raster,
    }
}

/// The impl wheel input of `wheel` and whether it carries a scroll delta.
pub fn WheelCompositorInput(
    wheel: &interaction::input_event::WheelEvent,
    event_timestamp: CompositorTimeTicks,
    position_scale: f32,
) -> (CompositorInput, bool) {
    use interaction::input_event::{ScrollGranularity, WheelPhase};
    let phase = match wheel.phase {
        WheelPhase::kBegan => ScrollPhase::Begin,
        WheelPhase::kEnded | WheelPhase::kCancelled => ScrollPhase::End,
        WheelPhase::kNone | WheelPhase::kChanged => ScrollPhase::Update,
    };
    let (mut delta, mut unconstrained) = wheel.GestureScrollDelta();
    let granularity = match wheel.delta_units {
        ScrollGranularity::kScrollByPrecisePixel => {
            compositor::adapter::input_types::ScrollGranularity::kScrollByPrecisePixel
        }
        ScrollGranularity::kScrollByPixel => {
            compositor::adapter::input_types::ScrollGranularity::kScrollByPixel
        }
        ScrollGranularity::kScrollByLine => {
            compositor::adapter::input_types::ScrollGranularity::kScrollByLine
        }
        ScrollGranularity::kScrollByPage => {
            compositor::adapter::input_types::ScrollGranularity::kScrollByPage
        }
        ScrollGranularity::kScrollByDocument => {
            compositor::adapter::input_types::ScrollGranularity::kScrollByDocument
        }
    };
    // InputRouter::ScaleEvent delegates to ui::ScaleWebInputEvent, which
    // scales both the widget position and pixel/precise-pixel wheel
    // deltas before the renderer and cc see the event.  The resident cc
    // scroll tree is expressed in the same LayoutZoomFactor coordinates
    // as Paint's scroll property nodes, so scaling only the position made
    // an impl scroll advance in CSS pixels and then jump when the main
    // thread committed the corresponding Layout offset.
    //
    // Page/document granularity is deliberately scale independent, and
    // Chromium also leaves line/document deltas unchanged here.
    if matches!(
        wheel.delta_units,
        ScrollGranularity::kScrollByPrecisePixel | ScrollGranularity::kScrollByPixel
    ) {
        let scale = f64::from(position_scale);
        delta.x *= scale;
        delta.y *= scale;
        unconstrained.x *= scale;
        unconstrained.y *= scale;
    }
    let input = CompositorInput::Wheel {
        position: CompositorPointF::new(
            wheel.position.x as f32 * position_scale,
            wheel.position.y as f32 * position_scale,
        ),
        delta: Vector2dF::new(delta.x as f32, delta.y as f32),
        unconstrained_delta: Vector2dF::new(unconstrained.x as f32, unconstrained.y as f32),
        granularity,
        is_inertial: wheel
            .native
            .is_some_and(|native| native.momentum_phase != 0),
        has_synthetic_phase: wheel.has_synthetic_phase,
        event_timestamp,
        phase,
    };
    (input, delta.x != 0. || delta.y != 0.)
}

/// Widget input gestures are already recognized; only values cross into cc.
pub fn GestureCompositorInput(
    gesture: &interaction::input_event::WebGestureEvent,
) -> io::Result<CompositorInput> {
    use compositor::adapter::input_handler::ScrollInputType;
    use interaction::input::public::common::input::{
        web_gesture_event::{GestureDevice, InertialPhaseState},
        web_input_event::Type,
    };
    let input_type = match gesture.SourceDevice() {
        GestureDevice::kTouchpad => ScrollInputType::kWheel,
        GestureDevice::kTouchscreen => ScrollInputType::kTouchscreen,
        GestureDevice::kSyntheticAutoscroll => ScrollInputType::kAutoscroll,
        GestureDevice::kScrollbar => ScrollInputType::kScrollbar,
        GestureDevice::kUninitialized => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "gesture source device is uninitialized",
            ));
        }
    };
    let phase = match gesture.GetType() {
        Type::kGestureScrollBegin | Type::kGesturePinchBegin => ScrollPhase::Begin,
        Type::kGestureScrollUpdate | Type::kGesturePinchUpdate => ScrollPhase::Update,
        Type::kGestureScrollEnd | Type::kGesturePinchEnd => ScrollPhase::End,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "gesture requires its source main-thread or browser fling owner",
            ));
        }
    };
    let anchor = gesture.PositionInWidget();
    let position = CompositorPointF::new(anchor.x(), anchor.y());
    if matches!(
        gesture.GetType(),
        Type::kGesturePinchBegin | Type::kGesturePinchUpdate | Type::kGesturePinchEnd
    ) {
        return Ok(CompositorInput::PinchGesture {
            dispatch_id: 0,
            anchor: position,
            scale: if phase == ScrollPhase::Update {
                gesture.data.pinch_update.scale
            } else {
                1.0
            },
            input_type,
            phase,
        });
    }
    let delta = match phase {
        ScrollPhase::Begin => Vector2dF::new(
            gesture.data.scroll_begin.delta_x_hint,
            gesture.data.scroll_begin.delta_y_hint,
        ),
        ScrollPhase::Update => Vector2dF::new(
            gesture.data.scroll_update.delta_x,
            gesture.data.scroll_update.delta_y,
        ),
        ScrollPhase::End => Vector2dF::new(
            gesture.data.scroll_end.delta_x_compensated,
            gesture.data.scroll_end.delta_y_compensated,
        ),
    };
    let granularity = match gesture.DeltaUnits() {
        interaction::input::ui::events::types::scroll_types::ScrollGranularity::kScrollByPrecisePixel => {
            compositor::adapter::input_types::ScrollGranularity::kScrollByPrecisePixel
        }
        interaction::input::ui::events::types::scroll_types::ScrollGranularity::kScrollByPixel => {
            compositor::adapter::input_types::ScrollGranularity::kScrollByPixel
        }
        interaction::input::ui::events::types::scroll_types::ScrollGranularity::kScrollByLine => {
            compositor::adapter::input_types::ScrollGranularity::kScrollByLine
        }
        interaction::input::ui::events::types::scroll_types::ScrollGranularity::kScrollByPage => {
            compositor::adapter::input_types::ScrollGranularity::kScrollByPage
        }
        interaction::input::ui::events::types::scroll_types::ScrollGranularity::kScrollByDocument => {
            compositor::adapter::input_types::ScrollGranularity::kScrollByDocument
        }
    };
    let mut reasons = compositor::adapter::input_types::MainThreadHitTestReasons::default();
    if phase == ScrollPhase::Begin {
        use compositor::adapter::input_types::MainThreadHitTestReason;
        for reason in [
            MainThreadHitTestReason::kScrollbarScrolling,
            MainThreadHitTestReason::kMainThreadScrollHitTestRegion,
            MainThreadHitTestReason::kFailedHitTest,
        ] {
            if gesture.data.scroll_begin.main_thread_hit_tested_reasons & (1 << reason as u32)
                != 0
            {
                reasons.Put(reason);
            }
        }
    }
    Ok(CompositorInput::ScrollGesture(
        compositor::engine::CompositorScrollGesture {
            dispatch_id: 0,
            position,
            delta,
            granularity,
            input_type,
            unconstrained_delta: if phase == ScrollPhase::Update {
                Vector2dF::new(
                    gesture.data.scroll_update.delta_x_unconstrained,
                    gesture.data.scroll_update.delta_y_unconstrained,
                )
            } else {
                delta
            },
            is_inertial: gesture.InertialPhase() == InertialPhaseState::kMomentum,
            synthetic: phase != ScrollPhase::Update && gesture.Synthetic(),
            target_viewport: phase == ScrollPhase::Begin
                && gesture.data.scroll_begin.target_viewport,
            scroll_element_id: compositor::adapter::input_types::ElementId::new(
                if phase == ScrollPhase::Begin {
                    gesture.data.scroll_begin.scrollable_area_element_id
                } else {
                    0
                },
            ),
            main_thread_hit_tested_reasons: reasons,
            event_timestamp: CompositorTimeTicks::FromMicroseconds(
                gesture.TimeStamp().ToInternalValue(),
            ),
            phase,
        },
    ))
}
