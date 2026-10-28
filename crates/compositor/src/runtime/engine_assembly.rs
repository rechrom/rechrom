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
    resources::RasterReturnedResource,
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
    input_client: Arc<super::native_input_client::NativeInputHandlerClient>,
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
        input_client: Arc<super::native_input_client::NativeInputHandlerClient>,
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
            input_client,
            smoothness_priority_expiration_notifier,
        })
    }
    pub(crate) fn Commit(
        &mut self,
        artifact: Arc<PaintArtifact>,
        viewport: CompositorViewportState,
    ) -> Result<CompositorOutput, CompositorError> {
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
            self.core
                .PrepareCommit(artifact, viewport, &self.main_features)?;
        }
        self.scheduler.SetVisible(viewport.visible, &mut self.core);
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

    /// Value conversion only: source InputHandler owns hit testing, latching,
    /// scroll chaining, animation and mutation of the current canonical tree.
    pub(crate) fn Input(
        &mut self,
        input: CompositorInput,
    ) -> Result<CompositorOutput, CompositorError> {
        let mut input_trace = Span(CompositorPackage::Input, "Compositor.Input.Dispatch");
        input_trace.set(
            "kind",
            match &input {
                CompositorInput::Wheel { .. } => 0.0,
                CompositorInput::PinchBegin { .. } => 1.0,
                CompositorInput::PinchUpdate { .. } => 2.0,
                CompositorInput::PinchEnd { .. } => 3.0,
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
        if matches!(
            input,
            CompositorInput::Wheel {
                phase: ScrollPhase::Begin,
                ..
            }
        ) {
            self.input_client.BeginGesture();
        }
        if matches!(
            input,
            CompositorInput::Wheel {
                phase: ScrollPhase::End,
                ..
            }
        ) {
            let pending = self.input_client.TakeQueued();
            if !pending.is_empty() {
                host.WithInputHandler(&mut context, |input, runtime| {
                    let input = input.expect("source bound InputHandler");
                    for data in pending {
                        input.ScrollUpdate(runtime, ScrollState::new(data), TimeDelta::default());
                    }
                });
            }
        }
        if let CompositorInput::Wheel {
            position,
            delta,
            unconstrained_delta,
            granularity,
            is_inertial,
            phase: ScrollPhase::Update,
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
                let route = self.input_client.RouteUpdate(data, self.scheduler.Now());
                if route != super::native_input_client::ScrollUpdateRoute::Dispatch {
                    // Source: blink InputHandlerProxy::RouteToTypeSpecificHandler.
                    // The transition from an empty compositor event queue to a
                    // non-empty queue requests one BeginImplFrame through cc's
                    // resident InputHandler/LayerTreeHostImpl/Scheduler chain.
                    // The output bit below mirrors the current subscription to
                    // the embedder; it is not a substitute for this cc state.
                    if route
                        == super::native_input_client::ScrollUpdateRoute::EnqueueAndRequestAnimation
                    {
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
                        main_thread_effects: std::mem::take(&mut self.core.effects),
                        needs_begin_frames: true,
                        work_pending: true,
                        main_commit_pending: self.core.HasReadyCommit(),
                    });
                }
            }
        }
        let compositor_owned = match input {
            CompositorInput::Wheel {
                position,
                delta,
                unconstrained_delta,
                granularity,
                is_inertial,
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
                    if phase == ScrollPhase::Begin {
                        let mut data = ScrollStateData::default();
                        data.position_x = position.x();
                        data.position_y = position.y();
                        data.delta_x_hint = f64::from(delta.x());
                        data.delta_y_hint = f64::from(delta.y());
                        data.is_beginning = true;
                        data.is_in_inertial_phase = is_inertial;
                        data.delta_granularity = granularity;
                        data.from_user_input = true;
                        let mut state = ScrollState::new(data);
                        let status =
                            input.ScrollBegin(runtime, &mut state, ScrollInputType::kWheel);
                        compositor_owned = status.thread == ScrollThread::kScrollOnImplThread
                            && status.main_thread_hit_test_reasons.empty();
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
                        let _result = input.ScrollUpdate(
                            runtime,
                            ScrollState::new(data),
                            TimeDelta::default(),
                        );
                    }
                    if phase == ScrollPhase::End {
                        compositor_owned = input.IsCurrentlyScrolling(runtime);
                        input.ScrollEnd(runtime, true, None);
                    }
                    compositor_owned
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
        if let Some(compositor_owned) = compositor_owned {
            input_trace.set("compositor_owned", compositor_owned as u8 as f64);
            self.core
                .effects
                .push(CompositorMainThreadEffect::WheelDisposition { compositor_owned });
        }
        drop(query);
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
            main_thread_effects: effects,
            needs_begin_frames: true,
            work_pending: true,
            main_commit_pending: self.core.HasReadyCommit(),
        })
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
        if std::env::var_os("RECHROM_DIAG_CC_LIFECYCLE").is_some() {
            if matches!(effect, E::NotifyReadyToActivate { .. }) {
                eprintln!("cc-lifecycle NotifyReadyToActivate");
            } else if matches!(effect, E::NotifyReadyToDraw { .. }) {
                eprintln!("cc-lifecycle NotifyReadyToDraw");
            } else if matches!(effect, E::DidPrepareTiles { .. }) {
                eprintln!("cc-lifecycle DidPrepareTiles");
            } else if matches!(effect, E::SetNeedsPrepareTiles { .. }) {
                eprintln!("cc-lifecycle SetNeedsPrepareTiles");
            } else if matches!(effect, E::SetNeedsRedraw { .. }) {
                eprintln!("cc-lifecycle SetNeedsRedraw");
            }
        }
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
                let _task_trace = Span(task_package, task_name);
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
        if std::env::var_os("RECHROM_DIAG_CC_LIFECYCLE").is_some()
            && (self.core.HasReadyCommit()
                || self.scheduler.CommitPending()
                || self.scheduler.RedrawPending()
                || self.scheduler.PrepareTilesPending())
        {
            let host = self.core.host.as_ref().unwrap().Base();
            eprintln!(
                "cc-state pending_tree={} ready_activate={} commit_pending={} ready_commit={} redraw={} prepare={} can_draw={} needs_bf={} work_pending={}",
                host.pending_tree().is_some(),
                self.scheduler.pending_tree_is_ready_for_activation(),
                self.scheduler.CommitPending(),
                self.core.HasReadyCommit(),
                self.scheduler.RedrawPending(),
                self.scheduler.PrepareTilesPending(),
                host.CanDraw(&HostImplContext { services: &mut self.core.services }),
                self.needs_begin_frames,
                work_pending,
            );
        }
        Ok(CompositorOutput {
            frame: self.core.TakeFrame(),
            main_thread_effects: effects,
            needs_begin_frames: self.needs_begin_frames,
            work_pending,
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
