//! The synchronous internal presenter of a Page without an external
//! compositor runtime (headless `PaintInto` and the browser tests): the
//! embedding of the impl half on the document thread. It holds the impl
//! `compositor::CompositorEngine` directly with the Viz/Renderer it
//! presents into; the main half stays on the Page domain
//! (`main_compositor`). The halves are independent instances connected only
//! by the in-process Proxy queues, which this presenter drains after each
//! call (decision D1; cc's SingleThreadProxy::CompositeImmediately shape).
use super::*;
use compositor::{
    CompositorEngine, CompositorFrameSubmission, CompositorInput, CompositorOutput,
    CompositorViewportState,
};

pub(super) struct SynchronousPresenter {
    compositor: CompositorEngine,
    // ProxyMain -> ProxyImpl and ProxyImpl -> ProxyMain queues of the
    // in-process transport.
    to_impl: main_compositor::MainToImplQueue,
    to_main: main_compositor::ImplToMainQueue,
    draining_proxy_tasks: bool,
    viz: viz::adapter::VizEngine,
    renderer: renderer::Renderer<RendererSkiaBackend>,
    surface: viz::adapter::SurfaceId,
    next_surface: u64,
    local_surface_id: Option<compositor::runtime::types::LocalSurfaceId>,
    surface_parameters: Option<(u32, u32, u32)>,
    rendered_surface: Option<viz::adapter::SurfaceId>,
    // The artifact for the next BeginMainFrame (Page::Render commits
    // outside the Page lifecycle), and the artifact whose frame was
    // presented last (PaintInto's no-change fast path).
    artifact: Option<(Arc<paint::PaintArtifact>, u32, u32, f64)>,
    presented: Option<(std::sync::Weak<paint::PaintArtifact>, u64, (u32, u32, u64))>,
    deferred_effects: Vec<CompositorMainThreadEffect>,
    input_effects: Vec<compositor::CompositorInputEffect>,
    begin_frame_sequence: u64,
    needs_begin_frames: bool,
    work_pending: bool,
    main_commit_pending: bool,
    // The layout zoom of the active tree's artifact (InputRouter's widget
    // scale).
    input_scale: f64,
    // Manual driving of the synchronous presenter: the synthetic input
    // BeginFrame may need one or more owner-sequence Polls to reach
    // ScheduledActionDraw. Keep those Polls runnable, then hold later input
    // work once a frame is sent.
    input_frame_issued_since_physical: bool,
    input_frame_pump_pending: bool,
    // A budget-limited Poll may leave already runnable typed impl tasks.
    owner_turn_pending: bool,
}

impl PageDomainServices {
    /// The internal presenter creates both halves on this thread over the
    /// in-process Proxy queues (decision D1). The external runtime creates
    /// its halves on their own threads (`Page::StartExternalCompositor` and
    /// the embedding's impl runner).
    pub(super) fn EnsureCompositor(
        &self,
        viewport: Option<CompositorViewportState>,
    ) -> io::Result<()> {
        if self.external_presentation.get() {
            return if self.HasMainCompositor() {
                Ok(())
            } else {
                Err(io::Error::other("external Page compositor is not started"))
            };
        }
        if self.synchronous_presenter.borrow().is_some() {
            return Ok(());
        }
        let viewport = viewport.unwrap_or(CompositorViewportState {
            size: CompositorSize::default(),
            device_scale_factor: 1.0,
            page_scale_factor: 1.0,
            min_page_scale_factor: 1.0,
            max_page_scale_factor: 1.0,
            visible: true,
        });
        let to_impl = main_compositor::MainToImplQueue::default();
        let to_main = main_compositor::ImplToMainQueue::default();
        let init = self.CreateMainCompositor(Self::CompositorConfig(viewport), to_impl.clone())?;
        self.StartMainCompositor(None);
        let compositor = CompositorEngine::new(
            Self::CompositorConfig(viewport),
            compositor_input::CompositorServicesFor(
                self.compositor_executor.clone(),
                self.compositor_raster_executor.clone(),
                &self.raster_engine,
            ),
            init,
            to_main.clone(),
        )
        .map_err(|error| io::Error::other(error.to_string()))?;
        *self.synchronous_presenter.borrow_mut() = Some(SynchronousPresenter {
            compositor,
            to_impl,
            to_main,
            draining_proxy_tasks: false,
            viz: viz::adapter::VizEngine::New(),
            renderer: renderer::Renderer::new(RendererSkiaBackend::default()),
            surface: viz::adapter::SurfaceId(1),
            next_surface: 2,
            local_surface_id: None,
            surface_parameters: None,
            rendered_surface: None,
            artifact: None,
            presented: None,
            deferred_effects: Vec::new(),
            input_effects: Vec::new(),
            begin_frame_sequence: 0,
            needs_begin_frames: false,
            work_pending: false,
            main_commit_pending: false,
            input_scale: 1.0,
            input_frame_issued_since_physical: false,
            input_frame_pump_pending: false,
            owner_turn_pending: false,
        });
        Ok(())
    }

    /// Runs `f` on the internal presenter, creating both halves first.
    pub(super) fn WithSynchronousPresenter<T>(
        &self,
        f: impl FnOnce(&mut SynchronousPresenter, &PageDomainServices) -> io::Result<T>,
    ) -> io::Result<T> {
        self.EnsureCompositor(None)?;
        let mut presenter = self.synchronous_presenter.borrow_mut();
        let presenter = presenter
            .as_mut()
            .ok_or_else(|| io::Error::other("Page compositor is externally presented"))?;
        f(presenter, self)
    }

    /// Runs `f` on the internal presenter's resident compositor; impl input
    /// never creates one.
    pub(super) fn WithResidentSynchronousPresenter<T>(
        &self,
        f: impl FnOnce(&mut SynchronousPresenter, &PageDomainServices) -> io::Result<T>,
    ) -> io::Result<T> {
        let mut presenter = self.synchronous_presenter.borrow_mut();
        let presenter = presenter.as_mut().ok_or_else(|| {
            io::Error::other("wheel input requires a committed resident compositor")
        })?;
        f(presenter, self)
    }

    pub(super) fn SynchronousPresenterNeedsBeginFrames(&self) -> bool {
        self.synchronous_presenter
            .borrow()
            .as_ref()
            .is_some_and(|presenter| presenter.needs_begin_frames)
    }
    pub(super) fn SynchronousPresenterNeedsPump(&self) -> bool {
        self.synchronous_presenter
            .borrow()
            .as_ref()
            .is_some_and(|presenter| presenter.NeedsPump())
    }
    /// The InputRouter widget scale of the internal presenter's active tree
    /// (1 when the impl half runs elsewhere: the embedding scales there).
    pub(super) fn SynchronousPresenterInputScale(&self) -> f64 {
        self.synchronous_presenter
            .borrow()
            .as_ref()
            .map_or(1.0, |presenter| presenter.input_scale)
    }
    pub(super) fn InvalidatePreparedArtifact(&self) {
        if let Some(presenter) = self.synchronous_presenter.borrow_mut().as_mut() {
            presenter.presented = None;
        }
    }

    /// A LayerTreeHost setter of the internal presenter's Page, applied to
    /// the main half, then the Proxy tasks it posted run in process.
    pub(super) fn ContinueMainCompositorMutation(
        &self,
        mutation: main_compositor::MainCompositorMutation,
    ) -> io::Result<CompositorWheelResult> {
        self.EnsureCompositor(None)?;
        self.MutateMainCompositor(mutation)?;
        Ok(CompositorWheelResult {
            effects: self.DidMutateMainCompositor()?,
            ..Default::default()
        })
    }

    /// The internal presenter runs an input-queue message where the
    /// compositor task runner would: the InputHandlerProxy call on the impl
    /// half.
    pub(super) fn ContinueImplInput(&self, input: InputToImpl) -> io::Result<CompositorWheelResult> {
        self.ContinueImplCompositorInput(compositor_input::ImplInput(input))
    }

    /// A browser-dispatched scroll or pinch gesture on the internal impl half.
    pub(super) fn ContinueGestureScroll(
        &self,
        gesture: &interaction::input_event::WebGestureEvent,
    ) -> io::Result<CompositorWheelResult> {
        self.ContinueImplCompositorInput(compositor_input::GestureCompositorInput(gesture)?)
    }

    fn ContinueImplCompositorInput(
        &self,
        input: CompositorInput,
    ) -> io::Result<CompositorWheelResult> {
        self.EnsureCompositor(None)?;
        let mutation = compositor::CompositorMutation::Input(input);
        self.WithSynchronousPresenter(|presenter, main| presenter.WheelInput(main, mutation, false))
    }

    /// After synchronous main-half calls (LayerTreeHost, PAC direct
    /// updates, the main AnimationHost) of the internal presenter: takes the
    /// main-half effects, then runs the in-process Proxy tasks they posted
    /// (SetNeedsCommitOnImpl, ...) like SingleThreadProxy.
    pub(super) fn DidMutateMainCompositor(&self) -> io::Result<Vec<CompositorMainThreadEffect>> {
        let effects = self.TakeMainCompositorEffects();
        if let Some(presenter) = self.synchronous_presenter.borrow_mut().as_mut() {
            presenter.RunInProcessProxyTasks(self)?;
        }
        Ok(effects)
    }

    /// The internal presenter's synchronous composite
    /// (SingleThreadProxy::CompositeImmediately shape over ProxyMain /
    /// ProxyImpl, decision D1): requests a commit, then drives manual
    /// BeginFrames until the BeginMainFrame that pushes `artifact` has
    /// committed and its frame was drawn or cc reports a stable frame.
    pub(super) fn CommitArtifact(
        &self,
        artifact: &Arc<paint::PaintArtifact>,
        width: u32,
        height: u32,
        scale: f64,
    ) -> io::Result<()> {
        if self.external_presentation.get() {
            return Err(io::Error::other(
                "external Page artifacts are pushed by ProxyMain::BeginMainFrame",
            ));
        }
        let key = (width, height, scale.to_bits());
        if self
            .synchronous_presenter
            .borrow()
            .as_ref()
            .and_then(|presenter| presenter.presented.as_ref())
            .is_some_and(|(presented, revision, viewport)| {
                *viewport == key
                    && *revision == artifact.recording_revision
                    && presented
                        .upgrade()
                        .is_some_and(|presented| Arc::ptr_eq(&presented, artifact))
            })
        {
            return Ok(());
        }
        let main_viewport = {
            self.EnsureCompositor(None)?;
            self.MainCompositorViewportFor(artifact, width, height, scale)?.0
        };
        self.EnsureCompositor(Some(main_viewport))?;
        self.WithSynchronousPresenter(|presenter, main| {
            presenter.artifact = Some((artifact.clone(), width, height, scale));
            main.SetNeedsMainCompositorCommit();
            presenter.RunInProcessProxyTasks(main)?;
            for _ in 0..16 {
                let output = presenter.BeginFrame(main)?;
                let stable_frame = output.frame.is_none() && !output.needs_begin_frames;
                let (_, submitted_latest_commit, _, main_commit_pending) =
                    presenter.AcceptOutput(main, output)?;
                if presenter.artifact.is_none()
                    && !main_commit_pending
                    && (submitted_latest_commit || stable_frame)
                {
                    presenter.presented =
                        Some((Arc::downgrade(artifact), artifact.recording_revision, key));
                    return Ok(());
                }
            }
            Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "compositor is still preparing the committed PaintArtifact",
            ))
        })
    }

    pub(super) fn Render(
        &self,
        artifact: &Arc<paint::PaintArtifact>,
        width: u32,
        height: u32,
        scale: f64,
        target: &mut [u32],
        format: PixelFormat,
        stride: usize,
    ) -> io::Result<RenderUpdate> {
        let profile_started = std::env::var_os("BROWSER_PROFILE_INPUT")
            .is_some()
            .then(std::time::Instant::now);
        self.CommitArtifact(artifact, width, height, scale)?;
        let committed_at = profile_started.map(|_| std::time::Instant::now());
        self.WithSynchronousPresenter(|presenter, _| {
            presenter.RenderToTarget(
                width,
                height,
                target,
                format,
                stride,
                false,
                profile_started.zip(committed_at),
            )
        })
    }
}

impl SynchronousPresenter {
    /// DestroyProxyImplOnImplThread (proxy_main.cc:916-946), before the
    /// main half stops.
    pub(super) fn Shutdown(&mut self) {
        self.compositor.Shutdown();
    }

    /// The in-process Proxy transport: runs the posted tasks of both
    /// directions until neither queue has one. A BeginMainFrame is answered
    /// with the artifact Page committed for it (`CommitArtifact`), or
    /// without one.
    pub(super) fn RunInProcessProxyTasks(&mut self, main: &PageDomainServices) -> io::Result<()> {
        if std::mem::replace(&mut self.draining_proxy_tasks, true) {
            return Ok(());
        }
        let result = (|| -> io::Result<()> {
            loop {
                if let Some(task) = self.to_main.TakeTask() {
                    match task {
                        layerize::commit::proxy::ImplToMain::BeginMainFrame(state) => {
                            let artifact = self.artifact.take();
                            let mut error = None;
                            let deferred = &mut self.deferred_effects;
                            main.BeginMainCompositorFrame(state, &mut |effects| {
                                deferred.extend(effects);
                                // The Page schedules its lifecycle.
                                deferred.push(CompositorMainThreadEffect::BeginMainFrame);
                                if let Some((artifact, width, height, scale)) = &artifact {
                                    if let Err(failure) = main.UpdateMainCompositorPaintArtifact(
                                        artifact, *width, *height, *scale,
                                    ) {
                                        error = Some(failure);
                                    }
                                }
                            });
                            if let Some(error) = error {
                                return Err(error);
                            }
                            self.input_scale = main.MainCompositorInputScale();
                        }
                        task => main.RunMainCompositorImplTask(task),
                    }
                    let effects = main.TakeMainCompositorEffects();
                    self.deferred_effects.extend(effects);
                } else if let Some(task) = self.to_impl.TakeTask() {
                    let output = self
                        .compositor
                        .RunMainThreadTask(task)
                        .map_err(|error| io::Error::other(error.to_string()))?;
                    self.AcceptOutput(main, output)?;
                } else {
                    return Ok(());
                }
            }
        })();
        self.draining_proxy_tasks = false;
        result
    }

    /// SingleThreadProxy delivers the impl half's LayerTreeHostClient
    /// notifications synchronously with the call that produced them; the
    /// commit-pipeline tasks keep their queue order for
    /// `RunInProcessProxyTasks`.
    fn DeliverInProcessNotifications(&mut self) {
        let mut retained = Vec::new();
        while let Some(task) = self.to_main.TakeTask() {
            match task {
                layerize::commit::proxy::ImplToMain::BeginMainFrame(_)
                | layerize::commit::proxy::ImplToMain::DidCompleteCommit { .. }
                | layerize::commit::proxy::ImplToMain::NotifyTransitionRequestFinished {
                    ..
                } => retained.push(task),
                task => self
                    .deferred_effects
                    .extend(main_compositor::ImplNotificationEffect(task)),
            }
        }
        for task in retained {
            let _ = self.to_main.PostTask(task);
        }
    }

    fn OutputResult(
        &mut self,
        main: &PageDomainServices,
        output: CompositorOutput,
    ) -> io::Result<CompositorWheelResult> {
        let (submitted, _, _, _) = self.AcceptOutput(main, output)?;
        let mut result = CompositorWheelResult {
            submitted,
            ..Default::default()
        };
        for effect in self.input_effects.drain(..) {
            result.AddInputEffect(effect);
        }
        result.effects.append(&mut self.deferred_effects);
        Ok(result)
    }

    pub(super) fn AcceptOutput(
        &mut self,
        main: &PageDomainServices,
        first: CompositorOutput,
    ) -> io::Result<(bool, bool, bool, bool)> {
        let mut pending = std::collections::VecDeque::from([first]);
        let mut submitted = false;
        let mut submitted_latest_commit = false;
        let mut needs_begin_frames = false;
        let mut main_commit_pending = false;
        let mut turns = 0usize;
        while let Some(mut output) = pending.pop_front() {
            turns += 1;
            if turns > 64 {
                return Err(io::Error::other(
                    "compositor output did not reach a stable frame boundary",
                ));
            }
            // Each output reports the engine's current subscription state.
            // Ack/reclaim outputs later in this queue supersede earlier values.
            needs_begin_frames = output.needs_begin_frames;
            main_commit_pending = output.main_commit_pending;
            self.needs_begin_frames = needs_begin_frames;
            self.work_pending = output.work_pending;
            self.main_commit_pending = main_commit_pending;
            for effect in output.input_effects.drain(..) {
                // The main-thread hit-test request keeps its place among the
                // Page effects; the other input results are sorted fields.
                if let compositor::CompositorInputEffect::WheelRequiresMainThreadHitTest {
                    request,
                    position,
                } = effect
                {
                    self.deferred_effects.push(
                        CompositorMainThreadEffect::WheelRequiresMainThreadHitTest {
                            request,
                            position,
                        },
                    );
                } else {
                    self.input_effects.push(effect);
                }
            }
            self.DeliverInProcessNotifications();
            let frame_matches_latest_commit = output.frame_matches_latest_commit;
            let Some(frame) = output.frame else {
                continue;
            };
            let (accepted, returned_resources) = self.SubmitFrame(frame)?;
            submitted |= accepted;
            submitted_latest_commit |= accepted && frame_matches_latest_commit;
            pending.push_back(
                self.compositor
                    .DidReceiveFrameAck()
                    .map_err(|error| io::Error::other(error.to_string()))?,
            );
            if !returned_resources.is_empty() {
                pending.push_back(
                    self.compositor
                        .ReclaimResources(returned_resources)
                        .map_err(|error| io::Error::other(error.to_string()))?,
                );
            }
        }
        // CompositorOutput::work_pending is the Engine owner-turn contract
        // (see NeedsPump).
        self.owner_turn_pending |= self.work_pending;
        self.RunInProcessProxyTasks(main)?;
        Ok((
            submitted,
            submitted_latest_commit,
            needs_begin_frames,
            main_commit_pending,
        ))
    }

    fn SubmitFrame(
        &mut self,
        submission: CompositorFrameSubmission,
    ) -> io::Result<(
        bool,
        Vec<compositor::runtime::resources::RasterReturnedResource>,
    )> {
        let local_surface_id = submission.local_surface_id;
        let frame = submission.frame;
        let frame_token = frame.metadata.frame_token;
        let root = frame
            .render_pass_list
            .last()
            .ok_or_else(|| io::Error::other("compositor frame has no root render pass"))?;
        let parameters = (
            root.output_rect.width.max(0) as u32,
            root.output_rect.height.max(0) as u32,
            frame.metadata.device_scale_factor.to_bits(),
        );
        let mut returned = Vec::new();
        if let Some(current) = self.local_surface_id {
            let same_embed = current.embed_token == local_surface_id.embed_token;
            if same_embed && !local_surface_id.IsSameOrNewerThan(&current) {
                returned.extend(frame.resource_list.iter().map(
                    compositor::runtime::resources::RasterTransferableResource::ToReturnedResource,
                ));
                return Ok((false, returned));
            }
        }
        let generation_changed = self
            .local_surface_id
            .is_some_and(|current| current != local_surface_id);
        if !generation_changed
            && self
                .surface_parameters
                .is_some_and(|current| current != parameters)
        {
            return Err(io::Error::other(
                "CompositorFrame dimensions changed without a new LocalSurfaceId",
            ));
        }
        if generation_changed {
            if let Some(resources) = self.viz.DestroySurface(self.surface) {
                returned.extend(resources);
            }
            self.surface = viz::adapter::SurfaceId(self.next_surface);
            self.next_surface = self.next_surface.wrapping_add(1).max(1);
            self.rendered_surface = None;
        }
        self.local_surface_id = Some(local_surface_id);
        self.surface_parameters = Some(parameters);
        let accepted = self
            .viz
            .SubmitFrame(self.surface, frame)
            .map_err(|error| io::Error::other(error.to_string()))?;
        returned.extend(accepted.returned_resources);
        // The in-process CPU target has no native presentation clock. This
        // reports the actual accepted Viz identity without manufacturing time.
        self.deferred_effects.push(
            CompositorMainThreadEffect::PresentationFeedbackUnavailable {
                frame_token,
                surface_id: self.surface.0,
                viz_submission: accepted.submission,
                reason: PresentationUnavailableReason::OffscreenTarget,
            },
        );
        Ok((true, returned))
    }

    /// A delegated video child Surface frame, submitted into this
    /// presenter's Viz.
    pub(super) fn SubmitChildFrame(
        &mut self,
        surface_id: viz::adapter::SurfaceId,
        frame: compositor::CompositorFrame,
    ) -> io::Result<()> {
        self.viz
            .SubmitFrame(surface_id, frame)
            .map_err(|error| io::Error::other(error.to_string()))?;
        Ok(())
    }

    fn BeginFrame(&mut self, main: &PageDomainServices) -> io::Result<CompositorOutput> {
        self.begin_frame_sequence = self.begin_frame_sequence.wrapping_add(1).max(1);
        let now = main.compositor_executor.Now();
        let interval = BeginFrameArgs::DefaultInterval();
        let mut trace = browser_tracing::span("compositor", "PageCompositor.ManualBeginFrame");
        trace.set("external_presentation", 0.0);
        trace.set("sequence", self.begin_frame_sequence as f64);
        self.compositor
            .BeginFrame(BeginFrameArgs {
                frame_time: now,
                deadline: now + interval,
                interval,
                unthrottled_interval: interval,
                frame_id: BeginFrameId {
                    source_id: BeginFrameArgs::kManualSourceId,
                    sequence_number: self.begin_frame_sequence,
                },
                trace_id: self.begin_frame_sequence as i64,
                dispatch_time: now,
                client_arrival_time: now,
                type_: BeginFrameArgsType::NORMAL,
                ..BeginFrameArgs::default()
            })
            .map_err(|error| io::Error::other(error.to_string()))
    }

    pub(super) fn NeedsPump(&self) -> bool {
        // Keep posting owner turns only when an actual Poll exhausted its
        // budget with runnable typed tasks left.
        self.input_frame_pump_pending || self.owner_turn_pending
    }

    /// Runs work posted to cc's injected task runner (the native embedding's
    /// impl-sequence Pump).
    pub(super) fn Pump(&mut self, main: &PageDomainServices) -> io::Result<CompositorWheelResult> {
        let input_frame_pump = std::mem::take(&mut self.input_frame_pump_pending);
        let output = self
            .compositor
            .Poll()
            .map_err(|error| io::Error::other(error.to_string()))?;
        let result = self.OutputResult(main, output)?;
        self.input_frame_issued_since_physical |= result.submitted;
        self.input_frame_pump_pending = input_frame_pump && !result.submitted && self.work_pending;
        self.owner_turn_pending = self.work_pending;
        Ok(result)
    }

    pub(super) fn Wheel(
        &mut self,
        main: &PageDomainServices,
        wheel: &::interaction::input_event::WheelEvent,
    ) -> io::Result<CompositorWheelResult> {
        let timestamp = main
            .compositor_executor
            .TimeTicks(wheel.timestamp.unwrap_or_else(std::time::Instant::now));
        let (input, has_delta) = compositor_input::WheelCompositorInput(
            wheel,
            timestamp,
            compositor_input::InputWidgetScale(self.input_scale),
        );
        self.WheelInput(main, compositor::CompositorMutation::Input(input), has_delta)
    }

    pub(super) fn WheelInput(
        &mut self,
        main: &PageDomainServices,
        input: compositor::CompositorMutation,
        allow_input_frame: bool,
    ) -> io::Result<CompositorWheelResult> {
        let output = self
            .compositor
            .Mutate(input)
            .map_err(|error| io::Error::other(error.to_string()))?;
        let mut result = self.OutputResult(main, output)?;
        // A compositor scroll requests redraw on the existing scheduler. The
        // synchronous presenter has no external frame source; it explicitly
        // drives a frame after the input mutation.
        let input_frame = result.compositor_owned
            && allow_input_frame
            && !result.submitted
            && !self.input_frame_issued_since_physical;
        if input_frame {
            self.input_frame_issued_since_physical = true;
            let output = self.BeginFrame(main)?;
            let immediate = self.OutputResult(main, output)?;
            self.input_frame_pump_pending = !immediate.submitted && self.work_pending;
            result.submitted |= immediate.submitted;
            result.compositor_owned |= immediate.compositor_owned;
            result.gesture_acks.extend(immediate.gesture_acks);
            result.effects.extend(immediate.effects);
        }
        Ok(result)
    }

    fn RenderToTarget(
        &mut self,
        width: u32,
        height: u32,
        target: &mut [u32],
        format: PixelFormat,
        stride: usize,
        retained: bool,
        profile: Option<(std::time::Instant, std::time::Instant)>,
    ) -> io::Result<RenderUpdate> {
        let surface = self.surface;
        let output = viz::adapter::DeviceRect::new(0, 0, width as i32, height as i32);
        let placement = viz::adapter::SurfacePlacement {
            surface_id: surface,
            destination: output,
        };
        let (update, viz_output) = {
            let mut viz_output = self
                .viz
                .Aggregate(output, &[placement])
                .map_err(|error| io::Error::other(error.to_string()))?;
            let aggregated_at = profile.map(|_| std::time::Instant::now());
            let aggregate_damage_pixels = viz_output.DamagePixels();
            let owned_target = target.to_vec();
            let (update, rendered_target) = self.renderer.Render(
                &mut viz_output,
                renderer_skia::RenderTarget {
                    width,
                    height,
                    pixels: renderer_skia::PixelStorage::owned(owned_target),
                    format,
                    row_stride: stride,
                    retained: retained && self.rendered_surface == Some(surface),
                },
            )?;
            target.copy_from_slice(&rendered_target.pixels);
            if let (Some((started, committed)), Some(aggregated)) = (profile, aggregated_at) {
                eprintln!(
                    "page-render-profile commit_ms={:.3} aggregate_ms={:.3} raster_ms={:.3} total_ms={:.3} passes={} quads={} frame_damage_pixels={} replay_damage_pixels={}",
                    (committed - started).as_secs_f64() * 1000.0,
                    (aggregated - committed).as_secs_f64() * 1000.0,
                    aggregated.elapsed().as_secs_f64() * 1000.0,
                    started.elapsed().as_secs_f64() * 1000.0,
                    update.passes,
                    update.quads,
                    aggregate_damage_pixels,
                    update.damage_pixels,
                );
            }
            (update, viz_output)
        };
        self.viz.DidDraw(viz_output);
        self.rendered_surface = Some(surface);
        Ok(update)
    }
}

#[cfg(test)]
#[test]
fn compositor_work_pending_posts_an_owner_turn_until_pump_consumes_it() {
    let page = PageState::new(
        crate::CreateBrowserConstraints(320, 200),
        Rc::new(RefCell::new(NullPageClient::default())),
    );
    page.WithSynchronousPresenter(|presenter, main| {
        presenter.AcceptOutput(
            main,
            CompositorOutput {
                frame: None,
                frame_matches_latest_commit: false,
                input_effects: Vec::new(),
                needs_begin_frames: false,
                work_pending: true,
                main_commit_pending: false,
            },
        )?;
        assert!(presenter.NeedsPump());

        // A later stable output cannot erase the already queued continuation.
        presenter.AcceptOutput(
            main,
            CompositorOutput {
                frame: None,
                frame_matches_latest_commit: false,
                input_effects: Vec::new(),
                needs_begin_frames: false,
                work_pending: false,
                main_commit_pending: false,
            },
        )?;
        assert!(presenter.NeedsPump());
        Ok(())
    })
    .unwrap();
}
