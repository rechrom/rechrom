//! Private canonical Scheduler/Host command loop. No layer/tile/job/frame plan
//! crosses the Engine command/effect boundary.
use super::{
    engine_core::NativeEngineCore,
    execution::{EngineTask, EngineTaskQueue},
    main_endpoint::MainHostEffect,
    native_host_lifecycle::{HostLifecycleEffect, HostLifecycleEffects},
    native_render::NativeRenderServices,
    native_resource_events::{NativeResourceEffect, NativeResourceEffects},
    native_scheduler::{
        NativeSchedulerEffect, NativeSchedulerEffects, NativeSchedulerHost, NativeTimingHistory,
    },
    resources::CpuReturnedResource,
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
};
use paint::PaintArtifact;
use std::sync::Arc;

pub(crate) struct NativeEngineAssembly {
    core: NativeEngineCore,
    scheduler: Scheduler,
    commands: EngineTaskQueue,
    lifecycle: HostLifecycleEffects,
    scheduler_effects: NativeSchedulerEffects,
    main_features: NativeRenderServices,
    needs_begin_frames: bool,
    deferred: Option<HostLifecycleEffect>,
    resource_effects: NativeResourceEffects,
    latency_monitors: super::native_latency::NativeLatencyMonitorRegistry,
    smoothness_priority_expiration_notifier:
        Arc<crate::cc::base::delayed_unique_notifier::DelayedUniqueNotifier>,
}
impl NativeEngineAssembly {
    pub(crate) fn new(
        mut core: NativeEngineCore,
        settings: SchedulerSettings,
        scheduler_host: Arc<NativeSchedulerHost>,
        timing: Arc<NativeTimingHistory>,
        commands: EngineTaskQueue,
        lifecycle: HostLifecycleEffects,
        scheduler_effects: NativeSchedulerEffects,
        main_features: NativeRenderServices,
        resource_effects: NativeResourceEffects,
        latency_monitors: super::native_latency::NativeLatencyMonitorRegistry,
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
            core,
            scheduler,
            commands,
            lifecycle,
            scheduler_effects,
            main_features,
            needs_begin_frames: false,
            deferred: None,
            resource_effects,
            latency_monitors,
            smoothness_priority_expiration_notifier,
        })
    }
    pub(crate) fn Commit(
        &mut self,
        artifact: Arc<PaintArtifact>,
        viewport: CompositorViewportState,
    ) -> Result<CompositorOutput, CompositorError> {
        {
            let _query = self
                .core
                .scheduler_queries
                .Enter(self.scheduler.IsReadyToActivate());
            self.core
                .PrepareCommit(artifact, viewport, &self.main_features)?;
        }
        self.scheduler.SetVisible(viewport.visible, &mut self.core);
        if self.core.awaiting_main {
            self.core
                .NotifyReadyToCommit(&mut self.scheduler, true, true);
        } else {
            self.scheduler
                .SetNeedsBeginMainFrame(false, false, &mut self.core);
        }
        self.Poll()
    }
    pub(crate) fn BeginFrame(
        &mut self,
        args: BeginFrameArgs,
    ) -> Result<CompositorOutput, CompositorError> {
        self.scheduler.OnBeginFrame(&args, &mut self.core);
        self.Poll()
    }

    /// Value conversion only: source InputHandler owns hit testing, latching,
    /// scroll chaining, animation and mutation of the current canonical tree.
    pub(crate) fn Input(
        &mut self,
        input: CompositorInput,
    ) -> Result<CompositorOutput, CompositorError> {
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
        let handled = match input {
            CompositorInput::Wheel {
                position,
                delta,
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
                    let mut handled = true;
                    if phase == ScrollPhase::Begin {
                        let mut data = ScrollStateData::default();
                        data.position_x = position.x();
                        data.position_y = position.y();
                        data.delta_x_hint = f64::from(delta.x());
                        data.delta_y_hint = f64::from(delta.y());
                        data.is_beginning = true;
                        data.from_user_input = true;
                        let mut state = ScrollState::new(data);
                        let status =
                            input.ScrollBegin(runtime, &mut state, ScrollInputType::kWheel);
                        handled = status.thread == ScrollThread::kScrollOnImplThread
                            && status.main_thread_hit_test_reasons.empty();
                    }
                    if phase == ScrollPhase::End {
                        handled = input.IsCurrentlyScrolling(runtime);
                        input.ScrollEnd(runtime, true, None);
                    } else if handled
                        && (phase == ScrollPhase::Update || delta.x() != 0. || delta.y() != 0.)
                    {
                        let mut data = ScrollStateData::default();
                        data.position_x = position.x();
                        data.position_y = position.y();
                        data.delta_x = f64::from(delta.x());
                        data.delta_y = f64::from(delta.y());
                        data.delta_x_unconstrained = f64::from(delta.x());
                        data.delta_y_unconstrained = f64::from(delta.y());
                        data.from_user_input = true;
                        let result = input.ScrollUpdate(
                            runtime,
                            ScrollState::new(data),
                            TimeDelta::default(),
                        );
                        handled = result.did_scroll;
                    }
                    handled
                }))
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
        if let Some(handled) = handled {
            self.core
                .effects
                .push(CompositorMainThreadEffect::WheelDisposition { handled });
        }
        drop(query);
        self.Poll()
    }
    /// The actual Viz acceptance event is distinct from resource returns.
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
        resources: Vec<CpuReturnedResource>,
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
            let avoid = (context.services.NewContentForCheckerboardedScrollsEnabled()
                && host.Base().PrioritizeNewContentDueToCheckerboarding())
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
                self.scheduler.SetDeferBeginMainFrame(defer, &mut self.core)
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
    fn ResourceEffect(&mut self, effect: NativeResourceEffect) {
        let delegate = self
            .core
            .host
            .as_ref()
            .unwrap()
            .Base()
            .delegate_for_testing();
        match effect {
            NativeResourceEffect::DidLoseLayerTreeFrameSink { delegate: target } => {
                assert_eq!(target, delegate);
                self.scheduler.DidLoseLayerTreeFrameSink(&mut self.core);
                self.core
                    .effects
                    .push(CompositorMainThreadEffect::LayerTreeFrameSinkLost);
            }
            NativeResourceEffect::CanDrawStateChanged {
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
            NativeResourceEffect::PageScaleAnimationCompleted { delegate: target } => {
                assert_eq!(target, delegate);
                self.core
                    .effects
                    .push(CompositorMainThreadEffect::PageScaleAnimationCompleted);
            }
            NativeResourceEffect::VideoBeginFrame { controller, args } => {
                self.core
                    .effects
                    .push(CompositorMainThreadEffect::VideoBeginFrame {
                        controller: controller.0,
                        args,
                    });
            }
            NativeResourceEffect::VideoNeedsBeginFrames {
                delegate: target,
                needs,
            } => {
                assert_eq!(target, delegate);
                self.core
                    .effects
                    .push(CompositorMainThreadEffect::VideoNeedsBeginFrames { needs });
            }
            NativeResourceEffect::Presented {
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
            }
            NativeResourceEffect::TransitionFinished {
                delegate: target,
                sequence,
                rects,
            } => {
                assert_eq!(target, delegate);
                self.core
                    .main
                    .NotifyTransitionRequestsFinished(sequence, &rects);
            }
            NativeResourceEffect::CustomMetrics {
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
        if let Some(effect) = self.deferred.take() {
            self.Lifecycle(effect)?;
        }
        loop {
            let mut progressed = false;
            while let Some(task) = self.commands.Pop() {
                self.Task(task)?;
                progressed = true;
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
                let NativeSchedulerEffect::NeedsBeginFrameChanged { needed, .. } = effect;
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
        Ok(CompositorOutput {
            frame: self.core.TakeFrame(),
            main_thread_effects: effects,
            needs_begin_frames: self.needs_begin_frames,
            main_commit_pending: self.core.HasReadyCommit(),
        })
    }
}
impl Drop for NativeEngineAssembly {
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
