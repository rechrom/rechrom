//! Private canonical Scheduler/Host command loop. No layer/tile/job/frame plan
//! crosses the Engine command/effect boundary.
mod gesture;
use super::{
    engine_core::EngineCore,
    execution::{EngineTask, EngineTaskQueue},
    host_lifecycle::{HostLifecycleEffect, HostLifecycleEffects},
    main_endpoint::MainHostEffect,
    render::RenderServices,
    resource_events::{ResourceEffect, ResourceEffects},
    resources::RasterReturnedResource,
    scheduler_services::{EngineSchedulerHost, SchedulerEffect, SchedulerEffects, TimingHistory},
};
use crate::{
    adapter::{begin_frame::BeginFrameArgs, layer_tree_host_impl::HostImplContext},
    cc::{
        scheduler::{scheduler::Scheduler, scheduler_settings::SchedulerSettings},
        trees::layer_tree_host_impl::WithCpuTileManagerContext,
    },
    engine::{
        CompositorError, CompositorInput, CompositorMainThreadEffect, CompositorOutput,
        CompositorViewportState,
    },
    performance::{CompositorPackage, Span},
};
use paint::PaintArtifact;
use std::sync::Arc;

// InputHandlerProxy owns the exact scroll begin while a source DOM ack or
// main-thread hit-test callback crosses the sequenced package boundary.
struct PendingWheelInput {
    input: CompositorInput,
    stage: PendingWheelStage,
}
enum PendingWheelStage {
    DOMAcknowledgement,
    MainThreadHitTest(crate::adapter::input_types::MainThreadHitTestReasons),
}
pub(crate) struct EngineAssembly {
    core: EngineCore,
    scroll_animation_commit: Option<crate::engine::ScrollAnimationCommitToken>,
    next_scroll_animation_commit: u64,
    pending_wheel_inputs: std::collections::BTreeMap<u64, PendingWheelInput>,
    next_wheel_request: u64,
    waiting_wheel_hit_test: Option<u64>,
    deferred_wheel_inputs: std::collections::VecDeque<(CompositorInput, bool)>,
    wheel_gesture_in_progress: bool,
    active_scroll_gesture: Option<crate::adapter::input_handler::ScrollInputType>,
    gesture_pinch_in_progress: bool,
    scrollbar_gesture_active: bool,
    // MouseWheelEventQueue::send_wheel_events_async_, source cc:246-247.
    wheel_events_async: bool,
    scheduler: Scheduler,
    commands: EngineTaskQueue,
    lifecycle: HostLifecycleEffects,
    scheduler_effects: SchedulerEffects,
    main_features: RenderServices,
    needs_begin_frames: bool,
    deferred: Option<HostLifecycleEffect>,
    resource_effects: ResourceEffects,
    latency_monitors: super::latency_info_swap_promise_monitor::LatencyMonitorRegistry,
    input_client: Arc<super::input_client::QueuedInputHandlerClient>,
    smoothness_priority_expiration_notifier:
        Arc<crate::cc::base::delayed_unique_notifier::DelayedUniqueNotifier>,
}
impl EngineAssembly {
    pub(crate) fn ConvertPaintArtifactSubsetToRecord(
        &self,
        artifact: Arc<PaintArtifact>,
        state: &paint::paint_property_tree::PropertyTreeState,
    ) -> crate::engine::ImmutablePaintRecord {
        self.core
            .ConvertPaintArtifactSubsetToRecord(artifact, state)
    }

    pub(crate) fn CreateVideoSurfaceForeignLayer(
        &mut self,
        range: crate::runtime::types::SurfaceRange,
    ) -> Arc<dyn paint::ForeignLayer> {
        self.core.CreateVideoSurfaceForeignLayer(range)
    }

    pub(crate) fn new(
        mut core: EngineCore,
        settings: SchedulerSettings,
        scheduler_host: Arc<EngineSchedulerHost>,
        timing: Arc<TimingHistory>,
        commands: EngineTaskQueue,
        lifecycle: HostLifecycleEffects,
        scheduler_effects: SchedulerEffects,
        main_features: RenderServices,
        resource_effects: ResourceEffects,
        latency_monitors: super::latency_info_swap_promise_monitor::LatencyMonitorRegistry,
        input_client: Arc<super::input_client::QueuedInputHandlerClient>,
    ) -> Result<Self, CompositorError> {
        // Host and Scheduler share the exact same reporting allocation. Event
        // metrics submission is the canonical move-only SubmitInfo, not an ID.
        let reporting = core
            .host
            .as_ref()
            .unwrap()
            .Base()
            .compositor_frame_reporting_controller()
            .expect("source reporting owner")
            .0
            .clone();
        let id = core.host.as_ref().unwrap().Base().id();
        let scheduler = super::scheduler::CreateScheduler(
            &mut core,
            &settings,
            id,
            scheduler_host,
            timing,
            reporting,
        );
        // Source: cc/trees/proxy_impl.cc:61-63,131-135. This notifier owns
        // only its runner and command payload, never the Engine/Host receiver.
        let runner = core
            .host
            .as_ref()
            .unwrap()
            .Base()
            .GetTaskRunner(&HostImplContext {
                services: &mut core.services,
            });
        let task_runner =
            crate::adapter::layer_tree_host_impl::HostImplServices::ResolveSequencedTaskRunner(
                &core.services,
                runner,
            );
        let delivery = commands.clone();
        let smoothness_priority_expiration_notifier =
            crate::cc::base::delayed_unique_notifier::DelayedUniqueNotifier::new(
                task_runner,
                Arc::new(move || delivery.Push(EngineTask::RenewTreePriority)),
                crate::adapter::time::TimeDelta::Milliseconds(250),
            );
        Ok(Self {
            pending_wheel_inputs: Default::default(),
            next_wheel_request: 0,
            waiting_wheel_hit_test: None,
            deferred_wheel_inputs: Default::default(),
            wheel_gesture_in_progress: false,
            active_scroll_gesture: None,
            gesture_pinch_in_progress: false,
            scrollbar_gesture_active: false,
            wheel_events_async: false,
            core,
            scroll_animation_commit: None,
            next_scroll_animation_commit: 0,
            scheduler,
            commands,
            lifecycle,
            scheduler_effects,
            main_features,
            needs_begin_frames: false,
            deferred: None,
            resource_effects,
            latency_monitors,
            input_client,
            smoothness_priority_expiration_notifier,
        })
    }
    pub(crate) fn Commit(
        &mut self,
        artifact: Arc<PaintArtifact>,
        viewport: CompositorViewportState,
    ) -> Result<CompositorOutput, CompositorError> {
        self.CommitWithScrollAnimations(artifact, viewport, Vec::new())
    }
    pub(crate) fn CommitWithScrollAnimations(
        &mut self,
        artifact: Arc<PaintArtifact>,
        viewport: CompositorViewportState,
        areas: Vec<crate::adapter::input_types::ElementId>,
    ) -> Result<CompositorOutput, CompositorError> {
        self.core.ExpireCommitDeferral();
        if let Some(reason) = self.core.main_frame_lifecycle.DeferralReason() {
            // ProxyMain completes Blink's lifecycle but aborts before PAC /
            // UpdateLayers. The producer's immutable artifact remains with Page;
            // no cc ready-commit or presentable marker is manufactured here.
            self.core
                .AbortDeferredMainFrame(&mut self.scheduler, reason);
            return self.Poll();
        }
        if !self.core.awaiting_main {
            // The embedding can finish Page's artifact before Scheduler has
            // dispatched the BeginMainFrame which will own it.  Synchronize
            // impl-side deltas before snapshotting the main tree in that
            // case.  Otherwise PrepareCommit captures the pre-scroll base;
            // the later BMF reflects the already-applied delta into a second
            // slot and a following commit can restore that stale base.
            //
            // This preserves Chromium's ordering even though this resident
            // single-sequence embedding receives the produced artifact at
            // the Commit boundary: ApplyCompositorDeltas precedes the main
            // tree commit it belongs to.
            self.core.ApplyCompositorDeltas();
        }
        {
            let _query = self
                .core
                .scheduler_queries
                .Enter(self.scheduler.IsReadyToActivate());
            self.core.PreparePaintCommit(artifact, viewport)?;
        }
        self.scheduler.SetVisible(viewport.visible, &mut self.core);
        if !areas.is_empty() {
            self.next_scroll_animation_commit = self
                .next_scroll_animation_commit
                .checked_add(1)
                .expect("native animation commit identity exhausted");
            let token =
                crate::engine::ScrollAnimationCommitToken(self.next_scroll_animation_commit);
            assert!(self.scroll_animation_commit.replace(token).is_none());
            let areas = self.core.ClassifyScrollAnimationAreas(&areas);
            self.core
                .effects
                .push(CompositorMainThreadEffect::ScrollAnimationCommitPrepared { token, areas });
            if !self.core.awaiting_main {
                self.scheduler
                    .SetNeedsBeginMainFrame(false, false, &mut self.core);
            }
            return self.Poll();
        }
        self.core.FinalizePreparedCommit(&self.main_features)?;
        self.ScheduleFinalizedMainCommit();
        self.Poll()
    }
    pub(crate) fn FinalizeScrollAnimationCommit(
        &mut self,
        token: crate::engine::ScrollAnimationCommitToken,
        commands: Vec<scroll_animation::compositor_mutation::ScrollAnimationMutation>,
    ) -> Result<CompositorOutput, CompositorError> {
        if self.scroll_animation_commit != Some(token) {
            return Err(CompositorError::NotReady {
                stage: "scroll animation reply does not own this pending main commit",
            });
        }
        for command in commands {
            self.core.ApplySourceScrollAnimation(command);
        }
        self.core.FinalizePreparedCommit(&self.main_features)?;
        self.scroll_animation_commit = None;
        self.core
            .effects
            .push(CompositorMainThreadEffect::ScrollAnimationCommitFinalized { token });
        self.ScheduleFinalizedMainCommit();
        self.Poll()
    }
    fn ScheduleFinalizedMainCommit(&mut self) {
        if self.core.awaiting_main
            && self.core.HasReadyCommit()
            && !self.core.ReadyCommitWasNotified()
        {
            self.core
                .NotifyReadyToCommit(&mut self.scheduler, true, true);
        } else if self.core.HasReadyCommit()
            && !self.core.awaiting_main
            && !self.core.ReadyCommitWasNotified()
        {
            self.scheduler
                .SetNeedsBeginMainFrame(false, false, &mut self.core);
        }
    }

    /// Establishes the widget state before the first main-frame request.
    pub(crate) fn SetViewport(
        &mut self,
        viewport: CompositorViewportState,
    ) -> Result<CompositorOutput, CompositorError> {
        self.core.SetViewportState(viewport)?;
        self.scheduler.SetVisible(viewport.visible, &mut self.core);
        self.Poll()
    }

    /// Main-thread commit demand, before Blink produces DataForCommit.
    ///
    /// Source: cc/trees/proxy_main.cc:741-756;
    ///         cc/trees/proxy_impl.cc:567-573.
    pub(crate) fn RequestMainFrame(&mut self) -> Result<CompositorOutput, CompositorError> {
        self.scheduler
            .SetNeedsBeginMainFrame(false, false, &mut self.core);
        self.Poll()
    }

    pub(crate) fn FinishMainFrameWithoutUpdates(
        &mut self,
    ) -> Result<CompositorOutput, CompositorError> {
        if !self.core.awaiting_main {
            return Err(CompositorError::NotReady {
                stage: "no BeginMainFrame awaits a no-update response",
            });
        }
        self.core.FinishMainFrameWithoutUpdates(&mut self.scheduler);
        self.Poll()
    }
    pub(crate) fn BeginFrame(
        &mut self,
        args: BeginFrameArgs,
    ) -> Result<CompositorOutput, CompositorError> {
        let mut trace = Span(
            CompositorPackage::Scheduler,
            "Compositor.Scheduler.BeginFrame",
        );
        trace.set("sequence", args.frame_id.sequence_number as f64);
        self.scheduler.OnBeginFrame(&args, &mut self.core);
        self.Poll()
    }

    pub(crate) fn ApplySourceScrollAnimation(
        &mut self,
        mutation: scroll_animation::compositor_mutation::ScrollAnimationMutation,
    ) -> Result<CompositorOutput, CompositorError> {
        self.core.ApplySourceScrollAnimation(mutation);
        self.Poll()
    }
    pub(crate) fn ApplyFrameLifecycle(
        &mut self,
        mutation: crate::engine::FrameLifecycleMutation,
    ) -> Result<CompositorOutput, CompositorError> {
        use crate::engine::{CompositorMainThreadEffect, FrameLifecycleMutation};
        match mutation {
            FrameLifecycleMutation::SetDeferMainFrameUpdates(deferred) => {
                if self
                    .core
                    .main_frame_lifecycle
                    .SetDeferMainFrameUpdates(deferred)
                {
                    self.core.effects.push(
                        CompositorMainThreadEffect::MainFrameUpdatesDeferralChanged { deferred },
                    );
                    self.scheduler.SetDeferBeginMainFrame(
                        self.core.main_frame_lifecycle.DeferBeginMainFrame(),
                        &mut self.core,
                    );
                }
            }
            FrameLifecycleMutation::StartDeferringCommits { timeout, reason } => {
                if self.core.main_frame_lifecycle.StartDeferringCommits(
                    self.core.services.core.MonotonicTimeNow(),
                    timeout,
                    reason,
                ) {
                    self.core
                        .effects
                        .push(CompositorMainThreadEffect::CommitDeferralChanged {
                            deferred: true,
                            reason,
                        });
                }
            }
            FrameLifecycleMutation::StopDeferringCommits => {
                if let Some(reason) = self.core.main_frame_lifecycle.StopDeferringCommits() {
                    self.core
                        .effects
                        .push(CompositorMainThreadEffect::CommitDeferralChanged {
                            deferred: false,
                            reason,
                        });
                }
            }
        }
        self.Poll()
    }

    /// Requests a new compositor submission from the resident active tree.
    /// This is ProxyImpl::SetNeedsRedrawOnImplThread: no main-tree state is
    /// copied or rebuilt, and the normal Scheduler/BeginFrame path still owns
    /// the draw opportunity.
    pub(crate) fn RequestRedraw(&mut self) -> Result<CompositorOutput, CompositorError> {
        // SetNeedsRedraw schedules a draw, while viewport damage makes that
        // draw produce a CompositorFrame even when every layer pixel is
        // retained. Chromium pairs these in embedder-driven full redraws.
        self.core.SetFullViewportDamage();
        self.scheduler.SetNeedsRedraw(&mut self.core);
        self.Poll()
    }
    /// Value conversion only: source InputHandler owns hit testing, latching,
    /// scroll chaining, animation and mutation of the current canonical tree.
    pub(crate) fn Input(
        &mut self,
        input: CompositorInput,
    ) -> Result<CompositorOutput, CompositorError> {
        let mut output = self.InputOnce(input, false)?;
        while self.waiting_wheel_hit_test.is_none() {
            let Some((input, acknowledged)) = self.deferred_wheel_inputs.pop_front() else {
                break;
            };
            let mut next = self.InputOnce(input, acknowledged)?;
            debug_assert!(
                output.frame.is_none() && next.frame.is_none(),
                "source input never draws"
            );
            output
                .main_thread_effects
                .append(&mut next.main_thread_effects);
            output.needs_begin_frames |= next.needs_begin_frames;
            output.work_pending |= next.work_pending;
            output.main_commit_pending |= next.main_commit_pending;
        }
        Ok(output)
    }
    fn InputOnce(
        &mut self,
        input: CompositorInput,
        acknowledged: bool,
    ) -> Result<CompositorOutput, CompositorError> {
        // WebFrameWidgetImpl.cc:2220-2232. Listener metadata is applied to
        // the same main host that produces the next pending commit, never to
        // an impl tree or a parallel registry snapshot.
        match input {
            CompositorInput::SetPageScaleFactorAndLimits {
                scale,
                minimum,
                maximum,
            } => {
                if !scale.is_finite()
                    || !minimum.is_finite()
                    || !maximum.is_finite()
                    || minimum <= 0.
                    || scale < minimum
                    || scale > maximum
                {
                    return Err(Self::WheelContinuationError("invalid page scale limits"));
                }
                self.core
                    .main
                    .SetPageScaleFactorAndLimits(scale, minimum, maximum);
                return self.Poll();
            }
            CompositorInput::StartPageScaleAnimation {
                target,
                use_anchor,
                scale,
                duration,
            } => {
                if !scale.is_finite() || scale <= 0. || duration.InMicroseconds() < 0 {
                    return Err(Self::WheelContinuationError("invalid page scale animation"));
                }
                self.core
                    .host
                    .as_mut()
                    .expect("resident Host")
                    .BaseMutable()
                    .StartPageScaleAnimation(
                        target,
                        use_anchor,
                        scale,
                        duration,
                        &mut HostImplContext {
                            services: &mut self.core.services,
                        },
                    );
                return self.Poll();
            }
            CompositorInput::SetHaveScrollEventHandlers(value) => {
                self.core.main.SetHaveScrollEventHandlers(value);
                return self.Poll();
            }
            CompositorInput::SetEventListenerProperties { class, properties } => {
                self.core.main.SetEventListenerProperties(class, properties);
                return self.Poll();
            }
            _ => {}
        }
        // InputHandlerProxy.cc:497-548 keeps the original event, then either
        // reinjects its source hit id/reasons or drops an invalid hit result.
        let (input, dom_acknowledged, main_thread_hit_test) = match input {
            CompositorInput::WheelDOMAcknowledgement {
                request,
                default_prevented,
            } => {
                let pending = self
                    .pending_wheel_inputs
                    .remove(&request)
                    .ok_or_else(|| Self::WheelContinuationError("unknown wheel acknowledgement"))?;
                if !matches!(pending.stage, PendingWheelStage::DOMAcknowledgement) {
                    return Err(Self::WheelContinuationError(
                        "wheel acknowledgement stage mismatch",
                    ));
                }
                if default_prevented {
                    return Ok(self.WaitingWheelOutput(
                        CompositorMainThreadEffect::WheelDisposition {
                            compositor_owned: false,
                        },
                    ));
                }
                (pending.input, true, None)
            }
            CompositorInput::ContinueWheelScroll {
                request,
                scroll_element_id,
            } => {
                let pending = self.pending_wheel_inputs.remove(&request).ok_or_else(|| {
                    Self::WheelContinuationError("unknown wheel hit-test continuation")
                })?;
                let PendingWheelStage::MainThreadHitTest(reasons) = pending.stage else {
                    return Err(Self::WheelContinuationError(
                        "wheel hit-test stage mismatch",
                    ));
                };
                if self.waiting_wheel_hit_test != Some(request) {
                    return Err(Self::WheelContinuationError(
                        "wheel hit-test identity mismatch",
                    ));
                }
                self.waiting_wheel_hit_test = None;
                let Some(id) = scroll_element_id else {
                    let effect = match pending.input {
                        CompositorInput::ScrollGesture(gesture) => {
                            CompositorMainThreadEffect::GestureDisposition {
                                dispatch_id: gesture.dispatch_id,
                                handled: false,
                            }
                        }
                        _ => CompositorMainThreadEffect::WheelDisposition {
                            compositor_owned: false,
                        },
                    };
                    return Ok(self.WaitingWheelOutput(effect));
                };
                (pending.input, true, Some((id, reasons)))
            }
            input => (input, acknowledged, None),
        };
        if matches!(
            input,
            CompositorInput::ScrollGesture(_) | CompositorInput::PinchGesture { .. }
        ) {
            return self.GestureInput(input, main_thread_hit_test);
        }
        let mut input_trace = Span(CompositorPackage::Input, "Compositor.Input.Dispatch");
        input_trace.set(
            "kind",
            match &input {
                CompositorInput::ScrollGesture(_) | CompositorInput::PinchGesture { .. } => {
                    unreachable!()
                }
                CompositorInput::Wheel { .. } => 0.0,
                CompositorInput::ScrollbarGesture { .. } => 4.0,
                CompositorInput::PinchBegin { .. } => 1.0,
                CompositorInput::PinchUpdate { .. } => 2.0,
                CompositorInput::PinchEnd { .. } => 3.0,
                CompositorInput::SetPageScaleFactorAndLimits { .. }
                | CompositorInput::StartPageScaleAnimation { .. }
                | CompositorInput::WheelDOMAcknowledgement { .. }
                | CompositorInput::ContinueWheelScroll { .. }
                | CompositorInput::SetHaveScrollEventHandlers(_)
                | CompositorInput::SetEventListenerProperties { .. } => {
                    unreachable!("continuation unwrapped above")
                }
            },
        );
        use crate::{
            adapter::{geometry::Point, input_handler::ScrollInputType, time::TimeDelta},
            cc::input::{
                input_handler::ScrollThread, scroll_state::ScrollState,
                scroll_state_data::ScrollStateData,
            },
            engine::ScrollPhase,
        };
        let point = |p: crate::adapter::geometry::PointF| -> Result<Point, CompositorError> {
            if !p.x().is_finite() || !p.y().is_finite() {
                return Err(CompositorError::Backend(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "nonfinite input position",
                )));
            }
            Ok(Point::new(p.x().round() as i32, p.y().round() as i32))
        };
        let query = self
            .core
            .scheduler_queries
            .Enter(self.scheduler.IsReadyToActivate());
        let host = self
            .core
            .host
            .as_mut()
            .expect("resident Host")
            .BaseMutable();
        let mut context = HostImplContext {
            services: &mut self.core.services,
        };
        // InputHandlerProxy::HandleMouseWheel, cc:1132-1154: only the
        // canonical DOM acknowledgement admits the default gesture through a
        // blocking wheel region. Native hit-test/scroll policy remains in cc.
        if !dom_acknowledged && !self.wheel_events_async {
            if let CompositorInput::Wheel { position, .. } = input {
                let point = point(position)?;
                let blocked = host.WithInputHandler(&mut context, |input, runtime| {
                    input
                        .expect("source bound InputHandler")
                        .HasBlockingWheelEventHandlerAt(runtime, point)
                });
                if blocked {
                    drop(query);
                    let request =
                        self.RetainWheelInput(input, PendingWheelStage::DOMAcknowledgement);
                    return Ok(self.WaitingWheelOutput(
                        CompositorMainThreadEffect::WheelRequiresDOMAcknowledgement { request },
                    ));
                }
            }
        }
        if self.waiting_wheel_hit_test.is_some() && matches!(input, CompositorInput::Wheel { .. }) {
            self.deferred_wheel_inputs
                .push_back((input, dom_acknowledged));
            drop(query);
            return Ok(
                self.WaitingWheelOutput(CompositorMainThreadEffect::WheelDisposition {
                    compositor_owned: false,
                }),
            );
        }
        // MouseWheelEventQueue.cc:237-267 synthesizes GSB for the first
        // nonzero acknowledged wheel, including phase-less and cancelled-Began
        // streams. Native cc still selects the scrolling element.
        let begin_gesture = main_thread_hit_test.is_some()
            || matches!(input,
            CompositorInput::Wheel { unconstrained_delta, .. }
            if !self.wheel_gesture_in_progress && (unconstrained_delta.x() != 0. || unconstrained_delta.y() != 0.));
        if begin_gesture {
            self.scrollbar_gesture_active = false;
        }
        if matches!(input, CompositorInput::Wheel { phase: ScrollPhase::Begin, unconstrained_delta, .. }
            if unconstrained_delta.x() != 0. || unconstrained_delta.y() != 0.)
        {
            self.wheel_events_async = true;
        }
        let mut hit_test_reasons = None;
        let mut gesture_in_progress = self.wheel_gesture_in_progress;
        // A new wheel-generated GSB shares the proxy's single scroll latch
        // with host touchscreen/autoscroll gestures.
        let previous_scroll_gesture = if begin_gesture {
            self.active_scroll_gesture.take()
        } else {
            None
        };
        // A gesture phase boundary cannot overtake scroll updates that were
        // queued for the previous BeginImplFrame. InputHandlerProxy preserves
        // this ordering in its compositor event queue before dispatching the
        // next GestureScrollBegin/End. Flushing only at End lets a rapid new
        // finger gesture relatch first, so the previous gesture's queued
        // momentum is then applied to the new latch and visibly snaps the
        // page.
        if begin_gesture
            || matches!(
                input,
                CompositorInput::Wheel {
                    phase: ScrollPhase::Begin | ScrollPhase::End,
                    ..
                }
            )
        {
            let pending = self.input_client.TakeQueued();
            if !pending.is_empty() {
                host.WithInputHandler(&mut context, |input, runtime| {
                    let input = input.expect("source bound InputHandler");
                    for update in pending {
                        input.ScrollUpdateWithElasticOverscroll(
                            runtime,
                            ScrollState::new(update.data),
                            TimeDelta::default(),
                            update.event_timestamp,
                        );
                    }
                });
            }
        }
        if begin_gesture {
            self.input_client.BeginGesture();
        }
        if let CompositorInput::Wheel {
            position,
            delta,
            unconstrained_delta,
            granularity,
            is_inertial,
            event_timestamp,
            phase: ScrollPhase::Update,
            ..
        } = input
        {
            let compositor_owned = host.WithInputHandler(&mut context, |input, runtime| {
                input.is_some_and(|input| input.IsCurrentlyScrolling(runtime))
            });
            if compositor_owned {
                let position = point(position)?;
                if !delta.x().is_finite() || !delta.y().is_finite() {
                    return Err(CompositorError::Backend(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "nonfinite wheel delta",
                    )));
                }
                let mut data = ScrollStateData::default();
                data.position_x = position.x();
                data.position_y = position.y();
                data.delta_x = f64::from(delta.x());
                data.delta_y = f64::from(delta.y());
                data.delta_x_unconstrained = f64::from(unconstrained_delta.x());
                data.delta_y_unconstrained = f64::from(unconstrained_delta.y());
                data.is_in_inertial_phase = is_inertial;
                data.delta_granularity = granularity;
                data.from_user_input = true;
                let route = self.input_client.RouteUpdate(data, event_timestamp);
                if route != super::input_client::ScrollUpdateRoute::Dispatch {
                    // Source: blink InputHandlerProxy::RouteToTypeSpecificHandler.
                    // The transition from an empty compositor event queue to a
                    // non-empty queue requests one BeginImplFrame through cc's
                    // resident InputHandler/LayerTreeHostImpl/Scheduler chain.
                    // The output bit below mirrors the current subscription to
                    // the embedder; it is not a substitute for this cc state.
                    if route == super::input_client::ScrollUpdateRoute::EnqueueAndRequestAnimation {
                        host.WithInputHandler(&mut context, |input, runtime| {
                            input
                                .expect("source bound InputHandler")
                                .SetNeedsAnimateInput(runtime)
                        });
                    }
                    drop(query);
                    // Source: cc/trees/proxy_impl.cc:469-471. The native
                    // InputHandler calls SetNeedsOneBeginImplFrame through
                    // LayerTreeHostImpl::SetNeedsAnimateInput, and ProxyImpl
                    // updates Scheduler synchronously on the impl sequence.
                    // Our delegate stores that callback while Host is
                    // borrowed, so deliver it before returning to the
                    // embedder. Otherwise the current physical BeginFrame can
                    // overtake the request and input advances every other
                    // display interval.
                    while let Some(effect) = self.lifecycle.Pop() {
                        self.Lifecycle(effect)?;
                    }
                    if let Some(error) = self.core.failure.take() {
                        return Err(error);
                    }
                    input_trace.set("compositor_owned", compositor_owned as u8 as f64);
                    input_trace.set("queued_for_begin_frame", 1.0);
                    self.core
                        .effects
                        .push(CompositorMainThreadEffect::WheelDisposition { compositor_owned });
                    return Ok(CompositorOutput {
                        frame: None,
                        frame_matches_latest_commit: false,
                        main_thread_effects: std::mem::take(&mut self.core.effects),
                        needs_begin_frames: true,
                        work_pending: true,
                        main_commit_pending: self.core.HasReadyCommit()
                            || self.scroll_animation_commit.is_some(),
                    });
                }
            }
        }
        let compositor_owned = match input {
            CompositorInput::ScrollGesture(_) | CompositorInput::PinchGesture { .. } => {
                unreachable!()
            }
            CompositorInput::Wheel {
                position,
                delta,
                unconstrained_delta,
                granularity,
                is_inertial,
                has_synthetic_phase,
                event_timestamp,
                phase,
            } => {
                let position = point(position)?;
                if !delta.x().is_finite() || !delta.y().is_finite() {
                    return Err(CompositorError::Backend(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "nonfinite wheel delta",
                    )));
                }
                Some(host.WithInputHandler(&mut context, |input, runtime| {
                    let input = input.expect("source bound InputHandler");
                    // Wheel disposition records which thread owns the
                    // gesture, not whether this particular update changed an
                    // offset. An impl-thread gesture remains compositor-owned
                    // when it reaches a scroll boundary; InputHandler reports
                    // that separately through did_scroll/unused_scroll_delta.
                    // Falling back to Blink for a zero-consumption update
                    // would apply the same wheel delta twice and make the two
                    // scroll ranges fight at the boundary.
                    // Source: content/renderer/input/input_handler_proxy.cc;
                    //         cc/input/input_handler.h:InputHandlerScrollResult.
                    let mut compositor_owned = input.IsCurrentlyScrolling(runtime);
                    if begin_gesture {
                        if let Some(previous) = previous_scroll_gesture {
                            input.RecordScrollEnd(runtime, previous);
                            input.ScrollEnd(runtime, true, None);
                        }
                        gesture_in_progress = true;
                        let mut data = ScrollStateData::default();
                        data.position_x = position.x();
                        data.position_y = position.y();
                        data.delta_x_hint = f64::from(delta.x());
                        data.delta_y_hint = f64::from(delta.y());
                        data.is_beginning = true;
                        data.is_in_inertial_phase = is_inertial;
                        data.delta_granularity = granularity;
                        data.from_user_input = true;
                        if let Some((id, reasons)) = main_thread_hit_test {
                            data.set_current_native_scrolling_element(id);
                            data.main_thread_hit_tested_reasons = reasons;
                        }
                        let mut state = ScrollState::new(data);
                        let status = input.ScrollBeginWithElasticOverscroll(
                            runtime,
                            &mut state,
                            ScrollInputType::kWheel,
                            has_synthetic_phase,
                        );
                        compositor_owned = status.thread == ScrollThread::kScrollOnImplThread
                            && status.main_thread_hit_test_reasons.empty();
                        if !status.main_thread_hit_test_reasons.empty() {
                            assert!(
                                main_thread_hit_test.is_none(),
                                "source reinjected hit test cannot request another hit test"
                            );
                            hit_test_reasons = Some(status.main_thread_hit_test_reasons);
                        }
                    }
                    if compositor_owned
                        && (phase == ScrollPhase::Update || delta.x() != 0. || delta.y() != 0.)
                    {
                        let mut data = ScrollStateData::default();
                        data.position_x = position.x();
                        data.position_y = position.y();
                        data.delta_x = f64::from(delta.x());
                        data.delta_y = f64::from(delta.y());
                        data.delta_x_unconstrained = f64::from(unconstrained_delta.x());
                        data.delta_y_unconstrained = f64::from(unconstrained_delta.y());
                        data.is_in_inertial_phase = is_inertial;
                        data.delta_granularity = granularity;
                        data.from_user_input = true;
                        let _result = input.ScrollUpdateWithElasticOverscroll(
                            runtime,
                            ScrollState::new(data),
                            TimeDelta::default(),
                            event_timestamp,
                        );
                    }
                    if phase == ScrollPhase::End && gesture_in_progress {
                        gesture_in_progress = false;
                        compositor_owned = input.IsCurrentlyScrolling(runtime);
                        input.ScrollEndWithElasticOverscroll(
                            runtime,
                            true,
                            None,
                            event_timestamp,
                            has_synthetic_phase,
                        );
                    }
                    compositor_owned
                }))
            }
            CompositorInput::SetPageScaleFactorAndLimits { .. }
            | CompositorInput::StartPageScaleAnimation { .. }
            | CompositorInput::WheelDOMAcknowledgement { .. }
            | CompositorInput::ContinueWheelScroll { .. }
            | CompositorInput::SetHaveScrollEventHandlers(_)
            | CompositorInput::SetEventListenerProperties { .. } => {
                unreachable!("continuation unwrapped above")
            }
            CompositorInput::ScrollbarGesture {
                scroll_element_id,
                delta,
                granularity,
                event_timestamp,
                phase,
            } => {
                if !delta.x().is_finite() || !delta.y().is_finite() {
                    return Err(CompositorError::Backend(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "nonfinite scrollbar gesture delta",
                    )));
                }
                let mut active = self.scrollbar_gesture_active;
                host.WithInputHandler(&mut context, |input, runtime| {
                    use crate::cc::input::input_handler::ScrollBeginThreadState as BeginState;
                    let input = input.expect("source bound InputHandler");
                    // InputHandlerProxy.cc:61-122,1176-1254. An injected
                    // scrollbar begin already has the main-thread hit target.
                    if phase == ScrollPhase::Begin {
                        if let Some(previous) = self.active_scroll_gesture.take() {
                            input.RecordScrollEnd(runtime, previous);
                            input.ScrollEnd(runtime, true, None);
                        }
                        if active || gesture_in_progress {
                            input.RecordScrollEnd(runtime, if active { ScrollInputType::kScrollbar } else { ScrollInputType::kWheel });
                            input.ScrollEnd(runtime, true, None);
                        }
                        let mut data = ScrollStateData::default();
                        data.is_scrollbar_interaction = true;
                        data.is_beginning = true;
                        data.delta_x_hint = -f64::from(delta.x());
                        data.delta_y_hint = -f64::from(delta.y());
                        data.delta_granularity = granularity;
                        data.set_current_native_scrolling_element(scroll_element_id);
                        data.main_thread_hit_tested_reasons.Put(crate::adapter::input_types::MainThreadHitTestReason::kScrollbarScrolling);
                        let status = input.ScrollBeginWithElasticOverscroll(runtime, &mut ScrollState::new(data), ScrollInputType::kScrollbar, false);
                        assert!(status.main_thread_hit_test_reasons.empty(), "injected scrollbar has its source hit identity");
                        // Proxy.cc:1042-1082. kScrollbarScrolling means this
                        // source begin was hit-tested on main in all cases.
                        if status.thread != ScrollThread::kScrollIgnored {
                            let state = if status.main_thread_repaint_reasons.empty() {
                                BeginState::kScrollingOnCompositorBlockedOnMain
                            } else if status.raster_inducing {
                                BeginState::kRasterInducingScrollBlockedOnMain
                            } else { BeginState::kScrollingOnMain };
                            input.RecordScrollBegin(runtime, ScrollInputType::kScrollbar, state);
                        }
                        active = status.thread == ScrollThread::kScrollOnImplThread;
                        gesture_in_progress = false;
                    } else if active && phase == ScrollPhase::Update {
                        // Proxy.cc:1257-1346: cc owns granularity, target,
                        // snapping constraints and unused delta/overscroll.
                        let mut data = ScrollStateData::default();
                        data.is_scrollbar_interaction = true;
                        data.delta_x = -f64::from(delta.x());
                        data.delta_y = -f64::from(delta.y());
                        data.delta_x_unconstrained = data.delta_x;
                        data.delta_y_unconstrained = data.delta_y;
                        data.delta_granularity = granularity;
                        let result = input.ScrollUpdateWithElasticOverscroll(runtime, ScrollState::new(data), TimeDelta::default(), event_timestamp);
                        if result.hit_snap_constraint { input.ScrollEnd(runtime, true, None); active = false; }
                    } else if phase == ScrollPhase::End {
                        if active {
                            input.RecordScrollEnd(runtime, ScrollInputType::kScrollbar);
                            input.ScrollEndWithElasticOverscroll(runtime, true, None, event_timestamp, false);
                        }
                        active = false;
                    }
                });
                self.scrollbar_gesture_active = active;
                None
            }
            CompositorInput::PinchBegin { anchor } => {
                let anchor = point(anchor)?;
                host.WithInputHandler(&mut context, |input, runtime| {
                    input.expect("source bound InputHandler").PinchGestureBegin(
                        runtime,
                        anchor,
                        ScrollInputType::kTouchscreen,
                    )
                });
                None
            }
            CompositorInput::PinchUpdate { anchor, scale } => {
                let anchor = point(anchor)?;
                if !scale.is_finite() || scale <= 0. {
                    return Err(CompositorError::Backend(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "invalid pinch scale",
                    )));
                }
                host.WithInputHandler(&mut context, |input, runtime| {
                    input
                        .expect("source bound InputHandler")
                        .PinchGestureUpdate(runtime, scale, anchor)
                });
                None
            }
            CompositorInput::PinchEnd { anchor } => {
                let anchor = point(anchor)?;
                host.WithInputHandler(&mut context, |input, runtime| {
                    input
                        .expect("source bound InputHandler")
                        .PinchGestureEnd(runtime, anchor)
                });
                None
            }
        };
        if matches!(
            input,
            CompositorInput::Wheel {
                phase: ScrollPhase::End,
                ..
            }
        ) {
            self.wheel_events_async = false;
        }
        self.wheel_gesture_in_progress = gesture_in_progress;
        if let Some(compositor_owned) = compositor_owned {
            input_trace.set("compositor_owned", compositor_owned as u8 as f64);
            self.core
                .effects
                .push(CompositorMainThreadEffect::WheelDisposition { compositor_owned });
        }
        drop(query);
        if let Some(reasons) = hit_test_reasons {
            let CompositorInput::Wheel { position, .. } = input else {
                unreachable!()
            };
            let request =
                self.RetainWheelInput(input, PendingWheelStage::MainThreadHitTest(reasons));
            self.waiting_wheel_hit_test = Some(request);
            self.core
                .effects
                .push(CompositorMainThreadEffect::WheelRequiresMainThreadHitTest {
                    request,
                    position,
                });
        }
        // LayerTreeHostImpl delegate calls stay on Chromium's impl sequence.
        // In particular, ProxyImpl::SetNeedsRedrawOnImplThread immediately
        // updates Scheduler before the next BeginFrame.  Our delegate endpoint
        // records those calls while Host is borrowed, so release that borrow
        // and deliver the recorded lifecycle calls now.  Leaving them for
        // Poll() lets a physical BeginFrame overtake SetNeedsRedraw and adds a
        // full display interval to the first scroll frame.
        //
        // Do not Poll here: scheduler actions, raster work, and draw remain
        // owned by the normal impl turn / physical BeginFrame.
        while let Some(effect) = self.lifecycle.Pop() {
            self.Lifecycle(effect)?;
        }
        if let Some(error) = self.core.failure.take() {
            return Err(error);
        }
        let mut effects = std::mem::take(&mut self.core.effects);
        self.core.DrainScrollEffects(&mut effects);
        // InputHandlerProxy returns after mutating the active tree and posts
        // the resulting scheduler work back to the impl task runner. Running
        // Poll here would drain tile, raster and scheduler commands inside the
        // input callback, so 120 Hz wheel delivery could occupy the sequence
        // past a 60 Hz BeginFrame deadline. The queued lifecycle effects are
        // consumed by the already requested Pump/BeginFrame turn.
        //
        // Source: content/renderer/input/input_handler_proxy.cc;
        //         cc/trees/proxy_impl.cc SetNeedsRedrawOnImplThread.
        Ok(CompositorOutput {
            frame: None,
            frame_matches_latest_commit: false,
            main_thread_effects: effects,
            needs_begin_frames: true,
            work_pending: true,
            main_commit_pending: self.core.HasReadyCommit()
                || self.scroll_animation_commit.is_some(),
        })
    }
    /// The actual Viz acceptance event is distinct from resource returns.
    pub(crate) fn RequestPresentationTime(
        &mut self,
        request: u64,
    ) -> Result<CompositorOutput, CompositorError> {
        let promise = super::report_time_swap_promise::Create(
            request,
            self.resource_effects.clone(),
            self.core.endpoint.Clock(),
        );
        // LayerTreeHost::QueueSwapPromise does not invent a commit or paint.
        self.core
            .main
            .pending_commit_state_mut()
            .swap_promises
            .push(promise);
        self.Poll()
    }
    pub(crate) fn DidReceiveFrameAck(&mut self) -> Result<CompositorOutput, CompositorError> {
        {
            let _query = self
                .core
                .scheduler_queries
                .Enter(self.scheduler.IsReadyToActivate());
            self.core
                .host
                .as_mut()
                .unwrap()
                .BaseMutable()
                .DidReceiveCompositorFrameAck(&mut HostImplContext {
                    services: &mut self.core.services,
                });
        }
        self.Poll()
    }
    pub(crate) fn DidPresentFrame(
        &mut self,
        token: u32,
        details: super::metadata::FrameTimingDetails,
    ) -> Result<CompositorOutput, CompositorError> {
        {
            let _query = self
                .core
                .scheduler_queries
                .Enter(self.scheduler.IsReadyToActivate());
            self.core
                .host
                .as_mut()
                .unwrap()
                .BaseMutable()
                .DidPresentCompositorFrame(
                    token,
                    &details,
                    &mut HostImplContext {
                        services: &mut self.core.services,
                    },
                );
        }
        self.Poll()
    }
    pub(crate) fn ReclaimResources(
        &mut self,
        resources: Vec<RasterReturnedResource>,
    ) -> Result<CompositorOutput, CompositorError> {
        {
            let _query = self
                .core
                .scheduler_queries
                .Enter(self.scheduler.IsReadyToActivate());
            self.core.ReclaimResources(resources);
        }
        self.Poll()
    }
    fn Task(&mut self, task: EngineTask) -> Result<(), CompositorError> {
        if matches!(&task, EngineTask::RenewTreePriority) {
            return self.RenewTreePriority();
        }

        let query = self
            .core
            .scheduler_queries
            .Enter(self.scheduler.IsReadyToActivate());
        let host = self.core.host.as_mut().unwrap().BaseMutable();
        let mut context = HostImplContext {
            services: &mut self.core.services,
        };
        match task {
            EngineTask::PaintWorkletState { delegate, state } => {
                assert_eq!(delegate, host.delegate_for_testing());
                drop(query);
                self.scheduler
                    .NotifyPaintWorkletStateChange(state, &mut self.core);
            }

            EngineTask::AnimationWorkletState {
                delegate,
                state,
                list,
            } => {
                assert_eq!(delegate, host.delegate_for_testing());
                let state = if state
                    == crate::adapter::layer_tree_host_impl::AnimationWorkletMutationState::STARTED
                {
                    crate::cc::scheduler::scheduler_state_machine::AnimationWorkletState::PROCESSING
                } else {
                    crate::cc::scheduler::scheduler_state_machine::AnimationWorkletState::IDLE
                };
                let tree = if list == crate::adapter::layer_tree_impl::ElementListType::ACTIVE {
                    crate::cc::scheduler::scheduler_state_machine::TreeType::ACTIVE
                } else {
                    crate::cc::scheduler::scheduler_state_machine::TreeType::PENDING
                };
                drop(query);
                self.scheduler
                    .NotifyAnimationWorkletStateChange(state, tree, &mut self.core);
            }

            EngineTask::RemoveLatencyMonitor { owner, id } => {
                if owner.IsValid() {
                    let current = host.AsWeakPtr();
                    assert_eq!(owner.owner, current.owner);
                    assert!(owner.alive.ptr_eq(&current.alive));
                    host.RemoveLatencyInfoSwapPromiseMonitor(id);
                }
                self.latency_monitors.Remove(id);
            }

            EngineTask::DidNotProduceFrame { ack, reason } => self
                .core
                .effects
                .push(CompositorMainThreadEffect::FrameNotProduced { ack, reason }),
            EngineTask::BeginFrameSourcePaused { paused } => self
                .core
                .effects
                .push(CompositorMainThreadEffect::BeginFrameSourcePaused { paused }),

            EngineTask::CheckerNeedsInvalidation { delegate: _ } => {
                drop(query);
                self.scheduler
                    .SetNeedsImplSideInvalidation(false, &mut self.core);
            }
            EngineTask::MainHost(effect) => {
                match effect {
                    MainHostEffect::NeedsUpdateLayers | MainHostEffect::NeedsCommit { .. } => {
                        // Source endpoint requests fulfilled by the current main
                        // commit have already cleared their actual request bit.
                        if super::main_host::CpuMainHostEndpoint::CommitRequested(
                            self.core.endpoint.as_ref(),
                        ) {
                            self.core
                                .effects
                                .push(CompositorMainThreadEffect::RequestMainFrame);
                        }
                    }
                    MainHostEffect::SetTargetLocalSurfaceId(id) => {
                        host.SetTargetLocalSurfaceId(&id, &mut context)
                    }
                    MainHostEffect::NewLocalSurfaceIdExpectedWhilePaused => {
                        host.NotifyNewLocalSurfaceIdExpectedWhilePaused(&mut context)
                    }
                    MainHostEffect::NotifyLatencyInfoSwapPromiseMonitors => {
                        host.NotifyLatencyInfoSwapPromiseMonitors(&mut context)
                    }
                    MainHostEffect::SaveActiveEventMetrics => self
                        .core
                        .effects
                        .push(CompositorMainThreadEffect::SaveActiveEventMetrics),
                    MainHostEffect::UpdateDeferMainFrameUpdate => self
                        .core
                        .effects
                        .push(CompositorMainThreadEffect::UpdateDeferMainFrameUpdate),
                }
            }
            EngineTask::Autoscroll { owner, task } => {
                host.RunAutoscrollTask(owner, task, &mut context)
            }
            EngineTask::ScrollbarAnimation { delegate, task } => {
                host.RunScrollbarAnimationTask(delegate, task, &mut context)
            }
            EngineTask::ImageAnimation(task) => host.RunImageAnimationTask(task, &mut context),
            EngineTask::ImageDecoded {
                callback,
                succeeded,
            } => host.RunImageDecodeCallback(callback, succeeded, &mut context),
            EngineTask::Host(task) => host.RunTask(task, &mut context),
            EngineTask::PostedHost(task) => host.RunPostedCompositorTask(task, &mut context),
            EngineTask::Metrics { target, results } => {
                if target.IsValid() {
                    let resident = host.AsWeakPtr();
                    assert_eq!(target.owner, resident.owner);
                    assert!(target.alive.ptr_eq(&resident.alive));
                    host.NotifyCompositorMetricsTrackerResults(&results, &mut context);
                }
            }
            EngineTask::Scheduler(task) => {
                drop(query);
                self.scheduler.RunTask(task, &mut self.core);
            }
            EngineTask::PoolEviction(owner) => {
                if let Some(pool) = host.resource_pool() {
                    pool.RunEvictionTask(owner);
                }
            }
            EngineTask::Tile {
                owner,
                alive,
                command,
            } => {
                WithCpuTileManagerContext(host, &mut context, |manager, runtime| {
                    manager.OnCommand(owner, alive, command, runtime)
                });
            }
            EngineTask::TileTasksFinished {
                owner,
                alive,
                start,
                all_tiles_done: has_pending_queries,
            } => {
                WithCpuTileManagerContext(host, &mut context, |manager, runtime| {
                    manager.OnAllTileTasksFinished(
                        owner,
                        alive,
                        start,
                        has_pending_queries,
                        runtime,
                    )
                });
            }
            EngineTask::PostedClosure(closure) => {
                // End the entire source-call borrow phase before a native once
                // closure can post more typed completion/deadline commands.
                drop(context);
                drop(query);
                closure();
            }
            // The worker already delivered original task-set callbacks. This
            // entry is only a wake signal, never a second completion algorithm.
            EngineTask::RasterWorkCompleted => {}
            EngineTask::RasterFailure(error) => return Err(error.into()),
            EngineTask::RenewTreePriority => unreachable!("handled before the live Host borrow"),
        }
        Ok(())
    }
    // Source: cc/trees/proxy_impl.cc:595-650. All queries below borrow the
    // current canonical Host; no scrolling/viewport state is cached here.
    fn RenewTreePriority(&mut self) -> Result<(), CompositorError> {
        use crate::cc::{
            input::actively_scrolling_type::ActivelyScrollingType,
            tiles::tile_priority::TreePriority,
        };
        let is_current_scroll_main_painted;
        let tree_priority;
        {
            let _query = self
                .core
                .scheduler_queries
                .Enter(self.scheduler.IsReadyToActivate());
            let host = self.core.host.as_mut().expect("resident Host");
            let mut context = HostImplContext {
                services: &mut self.core.services,
            };
            let precise = host.BaseMutable().GetActivelyScrollingType(&mut context)
                == ActivelyScrollingType::kPrecise;
            let checkerboarded_scroll =
                context.services.NewContentForCheckerboardedScrollsEnabled()
                    && host.Base().PrioritizeNewContentDueToCheckerboarding();
            let avoid = checkerboarded_scroll
                || (precise
                    && host
                        .BaseMutable()
                        .IsCurrentScrollMainRepainted(&mut context));
            let non_scroll =
                host.Base().IsPinchGestureActive() || host.Base().page_scale_animation_active();
            is_current_scroll_main_painted = host
                .BaseMutable()
                .IsCurrentScrollMainRepainted(&mut context);
            if (non_scroll || precise) && !avoid {
                self.smoothness_priority_expiration_notifier.Schedule();
            }
            let mut priority = TreePriority::SAME_PRIORITY_FOR_BOTH_TREES;
            if self
                .smoothness_priority_expiration_notifier
                .HasPendingNotification()
            {
                priority = TreePriority::SMOOTHNESS_TAKES_PRIORITY;
            }
            let viewport = host
                .Base()
                .active_tree()
                .expect("source active tree")
                .GetDeviceViewportWithExternal(*host.Base().external_viewport());
            if viewport.IsEmpty()
                || host.Base().EvictedUIResourcesExist()
                || is_current_scroll_main_painted
            {
                host.BaseMutable().SetRequiresHighResToDraw();
                priority = TreePriority::NEW_CONTENT_TAKES_PRIORITY;
            }
            drop(context);
            host.SetTreePriority(
                priority,
                &mut crate::adapter::client_layer_tree_host_impl::ClientHostContext {
                    services: &mut self.core.services,
                },
            );
            tree_priority = priority;
        }
        self.scheduler.SetTreePrioritiesAndScrollState(
            tree_priority,
            is_current_scroll_main_painted,
            &mut self.core,
        );
        Ok(())
    }
    fn Lifecycle(&mut self, effect: HostLifecycleEffect) -> Result<(), CompositorError> {
        use HostLifecycleEffect as E;
        match effect {
            E::NotifyReadyToActivate { .. } => self.scheduler.NotifyReadyToActivate(&mut self.core),
            E::NotifyReadyToDraw { .. } => self.scheduler.NotifyReadyToDraw(&mut self.core),
            E::DidPrepareTiles { .. } => self.scheduler.DidPrepareTiles(),
            E::SetNeedsPrepareTiles { .. } => self.scheduler.SetNeedsPrepareTiles(&mut self.core),
            // Source: cc/trees/proxy_impl.cc:763-782. An impl-side commit
            // request enters Scheduler; the Scheduler's SEND_BEGIN_MAIN_FRAME
            // action owns delta collection and the embedder request ordering.
            E::SetNeedsCommit { .. } => {
                self.scheduler
                    .SetNeedsBeginMainFrame(false, false, &mut self.core)
            }
            E::SetNeedsImplSideInvalidation {
                needs_first_draw_on_activation,
                ..
            } => self
                .scheduler
                .SetNeedsImplSideInvalidation(needs_first_draw_on_activation, &mut self.core),
            E::SetHasActiveThreadedScroll { active, .. } => self.scheduler.SetIsScrolling(active),
            E::SetWaitingForScrollEvent { waiting, .. } => {
                self.scheduler.SetWaitingForScrollEvent(waiting)
            }
            E::SetDeferBeginMainFrameFromImpl { defer, .. } => {
                self.core.main_frame_lifecycle.SetDeferFromImpl(defer);
                self.scheduler.SetDeferBeginMainFrame(
                    self.core.main_frame_lifecycle.DeferBeginMainFrame(),
                    &mut self.core,
                )
            }
            E::SetNeedsOneBeginImplFrame { .. } => {
                self.scheduler.SetNeedsOneBeginImplFrame(&mut self.core)
            }
            E::SetNeedsRedraw { .. } => self.scheduler.SetNeedsRedraw(&mut self.core),
            E::RenewTreePriority { .. } => self.RenewTreePriority()?,
            E::DidReceiveCompositorFrameAck { .. } => {
                self.scheduler.DidReceiveCompositorFrameAck(&mut self.core)
            }
            E::OnDrawForLayerTreeFrameSink {
                resourceless, skip, ..
            } => self
                .scheduler
                .OnDrawForLayerTreeFrameSink(resourceless, skip, &mut self.core),
            E::SetBeginFrameSource { source, .. } => {
                self.scheduler.SetBeginFrameSource(source, &mut self.core)
            }
            E::ClearHistory { .. } => self.scheduler.ClearHistory(&mut self.core),
            E::ReturnResource { resource, .. } => {
                let _query = self
                    .core
                    .scheduler_queries
                    .Enter(self.scheduler.IsReadyToActivate());
                self.core.ReclaimResources(vec![resource]);
            }
            E::ImageDecodeRequestFinished {
                request_id,
                speculative,
                succeeded,
                ..
            } => self
                .core
                .effects
                .push(CompositorMainThreadEffect::ImageDecodeFinished {
                    request_id,
                    speculative,
                    succeeded,
                }),
            // Preserve the original typed effect until its actual destination
            // is wired; never silently acknowledge an unimplemented callback.
            pending => {
                self.deferred = Some(pending);
                return Err(CompositorError::NotReady {
                    stage: "native scheduler auxiliary effect destination",
                });
            }
        }
        Ok(())
    }
    // Source: cc/trees/proxy_impl.cc:711-737,759-766.
    // Only native values/once callbacks cross this same-sequence mailbox. The
    // original compositor callbacks have already run synchronously at the C
    // delegate call; Scheduler feedback precedes the posted main callbacks.
    fn ResourceEffect(&mut self, effect: ResourceEffect) {
        let delegate = self
            .core
            .host
            .as_ref()
            .unwrap()
            .Base()
            .delegate_for_testing();
        match effect {
            ResourceEffect::ReportTimeSwapped {
                request,
                frame_token,
                swap_time,
            } => self
                .core
                .effects
                .push(CompositorMainThreadEffect::PresentationPromiseSwapped {
                    request,
                    frame_token,
                    swap_time,
                }),
            ResourceEffect::ReportTimeDidNotSwap {
                request,
                failure_time,
            } => {
                self.core
                    .effects
                    .push(CompositorMainThreadEffect::PresentationPromiseDidNotSwap {
                        request,
                        failure_time,
                    })
            }
            ResourceEffect::DidLoseLayerTreeFrameSink { delegate: target } => {
                assert_eq!(target, delegate);
                self.scheduler.DidLoseLayerTreeFrameSink(&mut self.core);
                self.core
                    .effects
                    .push(CompositorMainThreadEffect::LayerTreeFrameSinkLost);
            }
            ResourceEffect::CanDrawStateChanged {
                delegate: target,
                can_draw: _,
            } => {
                assert_eq!(target, delegate);
                // Source delivers this delegate callback synchronously. Our typed
                // mailbox may consume it after activation, so the captured value
                // can describe an older tree. Query the same resident Host at the
                // delivery point and preserve source ordering semantics.
                let can_draw = self
                    .core
                    .host
                    .as_ref()
                    .unwrap()
                    .Base()
                    .CanDraw(&HostImplContext {
                        services: &mut self.core.services,
                    });
                self.scheduler.SetCanDraw(can_draw, &mut self.core);
            }
            ResourceEffect::PageScaleAnimationCompleted { delegate: target } => {
                assert_eq!(target, delegate);
                self.core
                    .effects
                    .push(CompositorMainThreadEffect::PageScaleAnimationCompleted);
            }
            ResourceEffect::VideoBeginFrame { controller, args } => {
                self.core
                    .effects
                    .push(CompositorMainThreadEffect::VideoBeginFrame {
                        controller: controller.0,
                        args,
                    });
            }
            ResourceEffect::VideoNeedsBeginFrames {
                delegate: target,
                needs,
            } => {
                assert_eq!(target, delegate);
                self.core
                    .effects
                    .push(CompositorMainThreadEffect::VideoNeedsBeginFrames { needs });
            }
            ResourceEffect::Presented {
                delegate: target,
                token,
                mut callbacks,
                details,
            } => {
                assert_eq!(target, delegate);
                self.scheduler.DidPresentCompositorFrame(token, &details);
                for callback in callbacks.main_callbacks.drain(..) {
                    callback(&details.presentation_feedback);
                }
                for callback in callbacks.main_successful_callbacks.drain(..) {
                    callback(&details);
                }
                // LayerTreeHost.cc:998-1003: client notification follows
                // the source main-thread presentation callback lists.
                self.core
                    .effects
                    .push(CompositorMainThreadEffect::FramePresented {
                        frame_token: token,
                        details: details.clone(),
                    });
            }
            ResourceEffect::TransitionFinished {
                delegate: target,
                sequence,
                rects,
            } => {
                assert_eq!(target, delegate);
                self.core
                    .main
                    .NotifyTransitionRequestsFinished(sequence, &rects);
            }
            ResourceEffect::CustomMetrics {
                delegate: target,
                results,
            } => {
                assert_eq!(target, delegate);
                self.core
                    .effects
                    .push(CompositorMainThreadEffect::CompositorMetrics { results });
            }
        }
    }
    pub(crate) fn Poll(&mut self) -> Result<CompositorOutput, CompositorError> {
        let mut poll_trace = Span(
            CompositorPackage::Scheduler,
            "Compositor.Scheduler.OwnerTurn",
        );
        const OWNER_TURN_BUDGET: std::time::Duration = std::time::Duration::from_micros(1_000);
        let turn_started = std::time::Instant::now();
        let mut budget_exhausted = false;
        let mut tasks_processed = 0usize;
        let mut turns = 0usize;
        if let Some(effect) = self.deferred.take() {
            self.Lifecycle(effect)?;
        }
        loop {
            turns += 1;
            let mut progressed = false;
            while let Some(task) = self.commands.Pop() {
                let (task_package, task_name) = match &task {
                    EngineTask::RenewTreePriority => (CompositorPackage::Tile, "Compositor.Tile.RenewPriority"),
                    EngineTask::PaintWorkletState { .. } => (CompositorPackage::Layerize, "Compositor.Layerize.PaintWorkletState"),
                    EngineTask::AnimationWorkletState { .. } => (CompositorPackage::Animation, "Compositor.Animation.WorkletState"),
                    EngineTask::RemoveLatencyMonitor { .. } => (CompositorPackage::Input, "Compositor.Input.RemoveLatencyMonitor"),
                    EngineTask::ScrollbarAnimation { .. } => (CompositorPackage::Animation, "Compositor.Animation.Scrollbar"),
                    EngineTask::DidNotProduceFrame { .. } => (CompositorPackage::Scheduler, "Compositor.Scheduler.DidNotProduceFrame"),
                    EngineTask::BeginFrameSourcePaused { .. } => (CompositorPackage::Scheduler, "Compositor.Scheduler.SourcePaused"),
                    EngineTask::CheckerNeedsInvalidation { .. } => (CompositorPackage::Tile, "Compositor.Tile.CheckerInvalidation"),
                    EngineTask::Autoscroll { .. } => (CompositorPackage::Input, "Compositor.Input.Autoscroll"),
                    EngineTask::ImageAnimation(_) => (CompositorPackage::Animation, "Compositor.Animation.Image"),
                    EngineTask::MainHost(_) => (CompositorPackage::Tree, "Compositor.Tree.MainHostTask"),
                    EngineTask::ImageDecoded { .. } => (CompositorPackage::Resource, "Compositor.Resource.ImageDecoded"),
                    EngineTask::Host(_) => (CompositorPackage::Tree, "Compositor.Tree.HostTask"),
                    EngineTask::Metrics { .. } => (CompositorPackage::Scheduler, "Compositor.Scheduler.Metrics"),
                    EngineTask::PostedHost(_) => (CompositorPackage::Tree, "Compositor.Tree.PostedHostTask"),
                    EngineTask::Scheduler(task) => match task.kind {
                        crate::adapter::scheduler_host::SchedulerTaskKind::BeginImplFrameDeadline =>
                            (CompositorPackage::Scheduler, "Compositor.Scheduler.BeginImplFrameDeadline"),
                        crate::adapter::scheduler_host::SchedulerTaskKind::PendingBeginFrame =>
                            (CompositorPackage::Scheduler, "Compositor.Scheduler.PendingBeginFrame"),
                    },
                    EngineTask::PoolEviction(_) => (CompositorPackage::Resource, "Compositor.Resource.PoolEviction"),
                    EngineTask::Tile { command, .. } => match command {
                        crate::cc::tiles::tile_manager::TileManagerCommand::CheckIfMoreTilesNeedToBePrepared =>
                            (CompositorPackage::Tile, "Compositor.Tile.CheckIfMoreNeedPreparation"),
                        crate::cc::tiles::tile_manager::TileManagerCommand::CheckForCompletedTasksAndIssueSignals =>
                            (CompositorPackage::Tile, "Compositor.Tile.CheckCompletedTasks"),
                        crate::cc::tiles::tile_manager::TileManagerCommand::ReduceTileMemoryWhenIdle =>
                            (CompositorPackage::Resource, "Compositor.Resource.ReduceTileMemory"),
                        crate::cc::tiles::tile_manager::TileManagerCommand::TrimPrepaintTiles =>
                            (CompositorPackage::Tile, "Compositor.Tile.TrimPrepaint"),
                        crate::cc::tiles::tile_manager::TileManagerCommand::CheckRasterFinishedQueries =>
                            (CompositorPackage::Raster, "Compositor.Raster.CheckFinishedQueries"),
                        crate::cc::tiles::tile_manager::TileManagerCommand::CheckPendingGpuWorkAndIssueSignals =>
                            (CompositorPackage::Raster, "Compositor.Raster.CheckPendingWork"),
                        crate::cc::tiles::tile_manager::TileManagerCommand::DidFinishRunningTileTasksRequiredForActivation =>
                            (CompositorPackage::Raster, "Compositor.Raster.ActivationTasksFinished"),
                        crate::cc::tiles::tile_manager::TileManagerCommand::DidFinishRunningTileTasksRequiredForDraw =>
                            (CompositorPackage::Raster, "Compositor.Raster.DrawTasksFinished"),
                    },
                    EngineTask::TileTasksFinished { .. } => (CompositorPackage::Raster, "Compositor.Raster.TileTasksFinished"),
                    EngineTask::PostedClosure(_) => (CompositorPackage::Scheduler, "Compositor.Scheduler.PostedClosure"),
                    EngineTask::RasterWorkCompleted => (CompositorPackage::Raster, "Compositor.Raster.WorkCompleted"),
                    EngineTask::RasterFailure(_) => (CompositorPackage::Raster, "Compositor.Raster.Failure"),
                };
                let mut task_trace = Span(task_package, task_name);
                if let EngineTask::DidNotProduceFrame { reason, .. } = &task {
                    task_trace.set("reason", *reason as u8 as f64);
                }
                self.Task(task)?;
                tasks_processed += 1;
                progressed = true;
                if turn_started.elapsed() >= OWNER_TURN_BUDGET {
                    budget_exhausted = true;
                    break;
                }
            }
            while let Some(effect) = self.lifecycle.Pop() {
                self.Lifecycle(effect)?;
                progressed = true;
            }
            while let Some(effect) = self.resource_effects.Pop() {
                self.ResourceEffect(effect);
                progressed = true;
            }
            while let Some(effect) = self.scheduler_effects.Pop() {
                let SchedulerEffect::NeedsBeginFrameChanged { needed, .. } = effect;
                self.needs_begin_frames = needed;
            }
            let can_draw = {
                let _query = self
                    .core
                    .scheduler_queries
                    .Enter(self.scheduler.IsReadyToActivate());
                self.core
                    .host
                    .as_ref()
                    .unwrap()
                    .Base()
                    .CanDraw(&HostImplContext {
                        services: &mut self.core.services,
                    })
            };
            self.scheduler.SetCanDraw(can_draw, &mut self.core);

            // ScheduleTasks deliberately mutates only Chromium's resident task
            // graph while TileManager/Host borrows are live.  Once this source
            // turn has ended, hand the graph to the injected raster executor.
            // The worker posts the original task-set callbacks back through
            // `commands`; it does not call Host or choose compositor policy.
            let raster_tasks = {
                self.core
                    .services
                    .tile_execution
                    .lock()
                    .expect("independent native tile capabilities")
                    .task_graph
                    .clone()
            };
            raster_tasks.Kick();
            if budget_exhausted {
                break;
            }
            if !progressed
                && self.commands.IsEmpty()
                && self.lifecycle.IsEmpty()
                && self.resource_effects.IsEmpty()
                && self.scheduler_effects.IsEmpty()
            {
                break;
            }
        }
        if let Some(error) = self.core.failure.take() {
            return Err(error);
        }
        let mut effects = std::mem::take(&mut self.core.effects);
        self.core.DrainScrollEffects(&mut effects);
        let work_pending = self.deferred.is_some()
            || !self.commands.IsEmpty()
            || !self.lifecycle.IsEmpty()
            || !self.resource_effects.IsEmpty()
            || !self.scheduler_effects.IsEmpty();
        poll_trace.set("budget_exhausted", budget_exhausted as u8 as f64);
        poll_trace.set("work_pending", work_pending as u8 as f64);
        poll_trace.set("tasks", tasks_processed as f64);
        poll_trace.set("turns", turns as f64);
        let (frame, frame_matches_latest_commit) = self.core.TakeFrame();
        Ok(CompositorOutput {
            frame,
            frame_matches_latest_commit,
            main_thread_effects: effects,
            needs_begin_frames: self.needs_begin_frames,
            work_pending,
            main_commit_pending: self.core.HasReadyCommit()
                || self.scroll_animation_commit.is_some(),
        })
    }
}
impl Drop for EngineAssembly {
    fn drop(&mut self) {
        self.smoothness_priority_expiration_notifier.Shutdown();
        self.scheduler.Stop();
        // Teardown may query the same scheduler through source Picture hooks.
        // Keep that query valid only for this synchronous destruction call.
        let queries = self.core.scheduler_queries.clone();
        let _query = queries.Enter(self.scheduler.IsReadyToActivate());
        self.core.Shutdown();
    }
}

impl EngineAssembly {
    fn WheelContinuationError(message: &str) -> CompositorError {
        CompositorError::Backend(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            message,
        ))
    }
    fn RetainWheelInput(&mut self, input: CompositorInput, stage: PendingWheelStage) -> u64 {
        self.next_wheel_request = self
            .next_wheel_request
            .checked_add(1)
            .expect("source wheel request identity exhausted");
        self.pending_wheel_inputs
            .insert(self.next_wheel_request, PendingWheelInput { input, stage });
        self.next_wheel_request
    }
    fn WaitingWheelOutput(&mut self, effect: CompositorMainThreadEffect) -> CompositorOutput {
        self.core.effects.push(effect);
        CompositorOutput {
            frame: None,
            frame_matches_latest_commit: false,
            main_thread_effects: std::mem::take(&mut self.core.effects),
            needs_begin_frames: self.needs_begin_frames,
            work_pending: false,
            main_commit_pending: self.core.HasReadyCommit()
                || self.scroll_animation_commit.is_some(),
        }
    }
}
