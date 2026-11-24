//! The Page's main compositor half on the document thread (S1b-7 of
//! crates/compositor/S1B_PROXY_DESIGN.md): Blink's LayerTreeView /
//! WebFrameWidget compositor state over `layerize::LayerizeEngine` (main
//! LayerTreeHost, PaintArtifactCompositor, main AnimationHost) and its
//! `ProxyMain`. The impl half (`compositor::CompositorEngine`) runs on the
//! compositor thread or, for the synchronous internal presenter, on this
//! thread over the in-process Proxy queues (decision D1).
//!
//! The engines are kept in `RefCell`s: ProxyMain::BeginMainFrame calls back
//! into the Page lifecycle (LayerTreeHost::BeginMainFrame +
//! RequestMainFrameUpdate, proxy_main.cc:357-367), whose Paint borrows the
//! LayerizeEngine for PaintArtifactCompositor::Update, video ForeignLayers and
//! direct animation updates.
use compositor::{
    adapter::geometry::{Rect as CompositorRect, Size as CompositorSize},
    CompositorError, CompositorMainThreadEffect, CompositorMutation, CompositorViewportState,
    MainLayerize, MainPropertyTreeFeatures, MainProxy,
};
use layerize::runtime::proxy_main::{MainFrameClient, MainFrameResult, ProxyMainClient};
use std::{
    cell::{Cell, RefCell},
    io,
    sync::Arc,
};

pub type MainToImplQueue = layerize::commit::proxy::ProxyTaskQueue<compositor::MainToImplTask>;
pub type ImplToMainQueue = layerize::commit::proxy::ProxyTaskQueue<compositor::ImplToMainTask>;

pub(crate) struct MainCompositor {
    // Declared before `layerize`: ProxyMain::Stop destroys the impl half
    // (and with it any commit lease) before the main LayerTreeHost drops.
    proxy: RefCell<MainProxy>,
    endpoint: Arc<layerize::runtime::proxy_main::ProxyMainEndpoint<compositor::MainFamily>>,
    // A visibility change made inside ProxyMain::BeginMainFrame (the Page
    // lifecycle), applied when it returns.
    pending_visible: Cell<Option<bool>>,
    layerize: RefCell<MainLayerize>,
    features: MainPropertyTreeFeatures,
    executor: Arc<dyn compositor::CompositorExecutor>,
    // PaintArtifactCompositor update sequence (paint_artifact_compositor.cc
    // g_s_property_tree_sequence_number role).
    pac_sequence: Cell<i32>,
    // Blink's ParentLocalSurfaceIdAllocator: the parent sequence advances
    // whenever the surface parameters change
    // (components/viz/common/surfaces/parent_local_surface_id_allocator.cc:
    // 74-96; cc/trees/unbounded_frame_sink_handler.cc:56-77).
    local_surface_id: Cell<compositor::runtime::types::LocalSurfaceId>,
    surface_parameters: Cell<Option<(CompositorSize, u32)>>,
    // LayerTreeHost page scale limits (WebViewImpl -> LayerTreeHost).
    page_scale: Cell<(f32, f32, f32)>,
    // The last artifact pushed to PAC and its viewport, so an unchanged Page
    // frame does not run PaintArtifactCompositor::Update again.
    pushed: RefCell<Option<(std::sync::Weak<paint::PaintArtifact>, u64, (u32, u32, u64))>>,
    // The layout zoom of the pushed artifact (InputRouter's widget scale).
    input_scale: Cell<f64>,
    // ProxyMain::BeginMainFrame is running (UpdatePaintArtifact is only
    // admitted inside it).
    in_main_frame: Cell<bool>,
    effects: RefCell<Vec<CompositorMainThreadEffect>>,
    shut_down: Cell<bool>,
}

impl MainCompositor {
    /// Creates the main half and returns the ProxyImpl construction values.
    pub(crate) fn new(
        config: compositor::CompositorConfig,
        executor: Arc<dyn compositor::CompositorExecutor>,
        to_impl: MainToImplQueue,
        main_tasks: layerize::commit::main_callbacks::TaskRunner,
    ) -> io::Result<(Self, Box<compositor::ProxyImplInit>)> {
        let (layerize, proxy, features, init) =
            compositor::CreateMainHalf(config, to_impl, main_tasks)
                .map_err(|error| io::Error::other(error.to_string()))?;
        Ok((
            Self {
                endpoint: proxy.Endpoint().clone(),
                pending_visible: Cell::new(None),
                proxy: RefCell::new(proxy),
                layerize: RefCell::new(layerize),
                features,
                executor,
                pac_sequence: Cell::new(0),
                local_surface_id: Cell::new(compositor::engine::NewParentLocalSurfaceId()),
                surface_parameters: Cell::new(None),
                page_scale: Cell::new((1.0, 1.0, 1.0)),
                pushed: RefCell::new(None),
                input_scale: Cell::new(1.0),
                in_main_frame: Cell::new(false),
                effects: RefCell::new(Vec::new()),
                shut_down: Cell::new(false),
            },
            init,
        ))
    }

    /// ProxyMain::Start (proxy_main.cc:898-914). `init` is posted with
    /// Initialize and waited for; an in-process embedding passes `None`.
    pub(crate) fn Start(&self, init: Option<Box<compositor::ProxyImplInit>>) {
        self.proxy.borrow_mut().Start(init);
    }

    /// ProxyMain::Stop, then PAC destruction and root detach.
    pub(crate) fn Shutdown(&self) {
        if self.shut_down.replace(true) {
            return;
        }
        self.proxy.borrow_mut().Stop();
        self.layerize.borrow_mut().Shutdown();
    }

    pub(crate) fn Now(&self) -> compositor::adapter::time::TimeTicks {
        self.executor.Now()
    }

    pub(crate) fn HasResidentPaintArtifact(&self) -> bool {
        self.layerize.borrow().HasResidentPaintArtifact()
    }
    pub(crate) fn InputScale(&self) -> f64 {
        self.input_scale.get()
    }
    pub(crate) fn TakeEffects(&self) -> Vec<CompositorMainThreadEffect> {
        let mut effects = std::mem::take(&mut *self.effects.borrow_mut());
        for effect in self.endpoint.TakeEmbedderEffects() {
            use layerize::commit::main_effects::MainHostEffect as E;
            match effect {
                E::SaveActiveEventMetrics => {
                    effects.push(CompositorMainThreadEffect::SaveActiveEventMetrics)
                }
                E::UpdateDeferMainFrameUpdate => {
                    effects.push(CompositorMainThreadEffect::UpdateDeferMainFrameUpdate)
                }
                _ => {}
            }
        }
        effects
    }

    /// ProxyMain::SetNeedsAnimate (Blink ScheduleAnimation).
    pub(crate) fn SetNeedsAnimate(&self) {
        self.endpoint.SetNeedsAnimate(false);
    }
    /// ProxyMain::SetNeedsCommit.
    pub(crate) fn SetNeedsCommit(&self) {
        self.endpoint.SetNeedsCommit(false);
    }
    /// The Send + Sync request side of ProxyMain, for frame sources.
    pub(crate) fn Requests(&self) -> Arc<layerize::runtime::proxy_main::ProxyMainEndpoint<compositor::MainFamily>> {
        self.endpoint.clone()
    }

    /// LayerTreeHost::SetViewportRectAndScale with the parent LocalSurfaceId,
    /// page scale limits and visibility (Blink applies VisualProperties on
    /// the main thread; layer_tree_host.cc:1477-1530).
    pub(crate) fn SetViewport(&self, viewport: CompositorViewportState) -> io::Result<()> {
        let parameters = (viewport.size, viewport.device_scale_factor.to_bits());
        if self
            .surface_parameters
            .get()
            .is_some_and(|previous| previous != parameters)
        {
            let mut id = self.local_surface_id.get();
            id.parent_sequence_number = id
                .parent_sequence_number
                .checked_add(1)
                .ok_or_else(|| io::Error::other("parent LocalSurfaceId sequence exhausted"))?;
            self.local_surface_id.set(id);
        }
        self.surface_parameters.set(Some(parameters));
        let mut layerize = self.layerize.borrow_mut();
        let main = layerize.MainMut();
        main.SetViewportRectAndScale(
            CompositorRect::new(0, 0, viewport.size.width(), viewport.size.height()),
            viewport.device_scale_factor,
            self.local_surface_id.get(),
        );
        main.SetPageScaleFactorAndLimits(
            viewport.page_scale_factor,
            viewport.min_page_scale_factor,
            viewport.max_page_scale_factor,
        );
        drop(layerize);
        match self.proxy.try_borrow_mut() {
            Ok(mut proxy) => proxy.SetVisible(viewport.visible),
            Err(_) => self.pending_visible.set(Some(viewport.visible)),
        }
        Ok(())
    }

    /// The viewport of an artifact rendered at `width`x`height` device
    /// pixels with `scale` device pixels per CSS pixel.
    pub(crate) fn ViewportFor(
        &self,
        artifact: &paint::PaintArtifact,
        width: u32,
        height: u32,
        scale: f64,
    ) -> io::Result<(CompositorViewportState, f64)> {
        if !scale.is_finite() || scale <= 0.0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "raster scale must be positive"));
        }
        if width > i32::MAX as u32 || height > i32::MAX as u32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "render target exceeds compositor geometry range",
            ));
        }
        let layout_zoom = artifact
            .resources
            .as_ref()
            .map_or(1.0, |resources| resources.layout_zoom_factor)
            .max(f64::MIN_POSITIVE);
        let raster_scale = scale / layout_zoom;
        if !raster_scale.is_finite() || raster_scale <= 0.0 || raster_scale > f32::MAX as f64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "artifact-to-device scale is outside compositor range",
            ));
        }
        let (page_scale, min, max) = self.page_scale.get();
        Ok((
            CompositorViewportState {
                size: CompositorSize::new(width as i32, height as i32),
                device_scale_factor: raster_scale as f32,
                page_scale_factor: page_scale,
                min_page_scale_factor: min,
                max_page_scale_factor: max,
                visible: true,
            },
            layout_zoom,
        ))
    }

    /// LocalFrameView::PushPaintArtifactToCompositor (local_frame_view.cc:
    /// 2965-2975,3348): PaintArtifactCompositor::Update with the viewport,
    /// inside ProxyMain::BeginMainFrame only. An artifact already pushed
    /// with the same viewport is not updated again.
    pub(crate) fn UpdatePaintArtifact(
        &self,
        artifact: &Arc<paint::PaintArtifact>,
        width: u32,
        height: u32,
        scale: f64,
    ) -> io::Result<()> {
        assert!(
            self.in_main_frame.get(),
            "PaintArtifactCompositor::Update runs inside ProxyMain::BeginMainFrame"
        );
        let key = (width, height, scale.to_bits());
        if self.pushed.borrow().as_ref().is_some_and(|(pushed, revision, viewport)| {
            *viewport == key
                && *revision == artifact.recording_revision
                && pushed.upgrade().is_some_and(|pushed| Arc::ptr_eq(&pushed, artifact))
        }) {
            return Ok(());
        }
        let (viewport, layout_zoom) = self.ViewportFor(artifact, width, height, scale)?;
        self.SetViewport(viewport)?;
        let mut trace = browser_tracing::span("compositor_layerize", "Layerize.PaintArtifact");
        trace.set("display_items", artifact.display_items.len() as f64);
        trace.set("chunks", artifact.chunks.len() as f64);
        let mut layerize = self.layerize.borrow_mut();
        let input = layerize
            .PrepareInput(artifact.clone())
            .map_err(|error| io::Error::other(CompositorError::from(error).to_string()))?;
        let mut sequence = self.pac_sequence.get();
        let publication = layerize
            .UpdatePaintArtifact(artifact, input, &mut sequence)
            .map_err(|error| io::Error::other(CompositorError::from(error).to_string()))?;
        self.pac_sequence.set(sequence);
        drop(layerize);
        if let Some(publication) = publication {
            self.effects
                .borrow_mut()
                .push(CompositorMainThreadEffect::CanvasChildPaintRecords(publication));
        }
        self.input_scale.set(layout_zoom);
        *self.pushed.borrow_mut() = Some((Arc::downgrade(artifact), artifact.recording_revision, key));
        Ok(())
    }

    /// The main-thread compositor mutations
    /// (`CompositorMutation::IsMainThreadMutation`): LayerTreeHost setters,
    /// PaintArtifactCompositor direct updates and the main AnimationHost.
    pub(crate) fn Mutate(&self, mutation: CompositorMutation) -> io::Result<()> {
        use compositor::{CompositorInput, FrameLifecycleMutation};
        match mutation {
            CompositorMutation::Input(CompositorInput::SetPageScaleFactorAndLimits {
                scale,
                minimum,
                maximum,
            }) => {
                if !scale.is_finite()
                    || !minimum.is_finite()
                    || !maximum.is_finite()
                    || minimum <= 0.
                    || scale < minimum
                    || scale > maximum
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "invalid page scale limits",
                    ));
                }
                self.page_scale.set((scale, minimum, maximum));
                self.layerize
                    .borrow_mut()
                    .MainMut()
                    .SetPageScaleFactorAndLimits(scale, minimum, maximum);
            }
            // WebFrameWidgetImpl.cc:2220-2232 -> LayerTreeHost.
            CompositorMutation::Input(CompositorInput::SetHaveScrollEventHandlers(value)) => self
                .layerize
                .borrow_mut()
                .MainMut()
                .SetHaveScrollEventHandlers(value),
            CompositorMutation::Input(CompositorInput::SetEventListenerProperties {
                class,
                properties,
            }) => self
                .layerize
                .borrow_mut()
                .MainMut()
                .SetEventListenerProperties(class, properties),
            CompositorMutation::Input(_) => {
                return Err(io::Error::other("impl input reached the main compositor half"))
            }
            CompositorMutation::FrameLifecycle(mutation) => {
                let mut layerize = self.layerize.borrow_mut();
                match mutation {
                    // Source: cc/trees/proxy_main.cc:792-815.
                    FrameLifecycleMutation::SetDeferMainFrameUpdates(deferred) => {
                        if layerize
                            .MainFrameLifecycleMut()
                            .SetDeferMainFrameUpdates(deferred)
                        {
                            self.endpoint.Post(
                                layerize::commit::proxy::MainToImpl::SetDeferBeginMainFrame(
                                    deferred,
                                ),
                            );
                            self.effects.borrow_mut().push(
                                CompositorMainThreadEffect::MainFrameUpdatesDeferralChanged {
                                    deferred,
                                },
                            );
                        }
                    }
                    // Source: cc/trees/proxy_main.cc:848-879.
                    FrameLifecycleMutation::StartDeferringCommits { timeout, reason } => {
                        if layerize.MainFrameLifecycleMut().StartDeferringCommits(
                            self.executor.Now(),
                            timeout,
                            reason,
                        ) {
                            self.effects.borrow_mut().push(
                                CompositorMainThreadEffect::CommitDeferralChanged {
                                    deferred: true,
                                    reason,
                                },
                            );
                        }
                    }
                    FrameLifecycleMutation::StopDeferringCommits => {
                        if let Some(reason) = layerize.MainFrameLifecycleMut().StopDeferringCommits()
                        {
                            self.effects.borrow_mut().push(
                                CompositorMainThreadEffect::CommitDeferralChanged {
                                    deferred: false,
                                    reason,
                                },
                            );
                        }
                    }
                }
            }
            // PaintArtifactCompositor::DirectlyUpdate{Transform,Compositing}
            // (paint_artifact_compositor.cc), then LayerTreeHost::
            // SetNeedsCommit; the commit follows in the next BeginMainFrame.
            CompositorMutation::CompositedAnimationProperties(update) => {
                if !self.HasResidentPaintArtifact() {
                    return Err(io::Error::new(
                        io::ErrorKind::WouldBlock,
                        "composited animation requires a resident paint artifact",
                    ));
                }
                if !self
                    .layerize
                    .borrow_mut()
                    .ApplyCompositedAnimationProperties(&update)
                {
                    return Err(io::Error::other(
                        "resident compositor property mapping is unavailable",
                    ));
                }
                self.SetNeedsCommit();
            }
            CompositorMutation::ScrollbarPaint(mutation) => {
                let applied = self.layerize.borrow_mut().ApplyScrollbarPaintMutation(
                    mutation.element,
                    mutation.opacity,
                    mutation.thumb_width,
                    mutation.needs_repaint,
                );
                if !applied {
                    self.effects
                        .borrow_mut()
                        .push(CompositorMainThreadEffect::ScrollbarPaintFallback {
                            element: mutation.element,
                        });
                }
            }
            CompositorMutation::PropertyAnimation(mutation) => self
                .layerize
                .borrow_mut()
                .ApplySourcePropertyAnimation(mutation),
            CompositorMutation::ScrollAnimation(mutation) => self
                .layerize
                .borrow_mut()
                .ApplySourceScrollAnimation(mutation),
        }
        Ok(())
    }

    pub(crate) fn ConvertPaintArtifactSubsetToRecord(
        &self,
        artifact: Arc<paint::PaintArtifact>,
        state: &paint::paint_property_tree::PropertyTreeState,
    ) -> compositor::engine::ImmutablePaintRecord {
        self.layerize
            .borrow()
            .ConvertPaintArtifactSubsetToRecord(artifact, state)
    }
    pub(crate) fn CreateVideoSurfaceForeignLayer(
        &self,
        range: compositor::runtime::types::SurfaceRange,
    ) -> Arc<dyn paint::ForeignLayer> {
        self.layerize
            .borrow_mut()
            .CreateVideoSurfaceForeignLayer(range)
    }

    /// Runs a ProxyImpl -> ProxyMain task other than BeginMainFrame. The
    /// LayerTreeHostClient notifications are returned as effects.
    pub(crate) fn RunImplTask(&self, task: compositor::ImplToMainTask) {
        let mut client = EffectClient {
            effects: Vec::new(),
        };
        self.proxy
            .borrow_mut()
            .RunImplTask(task, &mut self.layerize.borrow_mut(), &mut client);
        self.effects.borrow_mut().extend(client.effects);
    }

    /// ProxyMain::BeginMainFrame (proxy_main.cc:174-580) with the embedder
    /// lifecycle `update_rendering`, which runs after the compositor deltas
    /// and mutator events were applied and their effects handed to it.
    pub(crate) fn BeginMainFrame(
        &self,
        state: Box<compositor::BeginMainFrameState>,
        update_rendering: &mut dyn FnMut(Vec<CompositorMainThreadEffect>),
    ) -> MainFrameResult {
        let mut client = LifecycleClient {
            effects: EffectClient {
                effects: Vec::new(),
            },
            now: self.executor.clone(),
            update_rendering,
        };
        self.in_main_frame.set(true);
        let result = self.proxy.borrow_mut().BeginMainFrame(
            state,
            &self.layerize,
            &self.features,
            &mut client,
        );
        self.in_main_frame.set(false);
        self.effects.borrow_mut().extend(client.effects.effects);
        if let Some(visible) = self.pending_visible.take() {
            self.proxy.borrow_mut().SetVisible(visible);
        }
        result
    }
}

impl Drop for MainCompositor {
    fn drop(&mut self) {
        self.Shutdown();
    }
}

/// LayerTreeHostClient notifications as Page compositor effects.
struct EffectClient {
    effects: Vec<CompositorMainThreadEffect>,
}
impl ProxyMainClient<compositor::MainFamily> for EffectClient {
    fn CommitDeferralChanged(
        &mut self,
        deferred: bool,
        reason: compositor::PaintHoldingReason,
    ) {
        self.effects
            .push(CompositorMainThreadEffect::CommitDeferralChanged { deferred, reason });
    }
    fn ApplyViewportChanges(
        &mut self,
        scale: f32,
        data: &layerize::commit::compositor_commit_data::CompositorCommitData,
    ) {
        self.effects
            .push(CompositorMainThreadEffect::ViewportScaleChanged {
                scale,
                delta: data.page_scale_delta,
                pinch_active: data.is_pinch_gesture_active,
                inner_scroll_delta: data.inner_viewport_scroll.scroll_delta,
            });
    }
    fn ScrollAnimationsFinished(
        &mut self,
        completions: Vec<scroll_animation::compositor_mutation::ScrollAnimationCompletion>,
    ) {
        self.effects.extend(
            completions
                .into_iter()
                .map(CompositorMainThreadEffect::ScrollAnimationCompleted),
        );
    }
    // The finish timestamp belongs to FinishCommit (proxy_impl.cc
    // ScheduledActionCommit); it is in the impl half's clock.
    fn DidCommit(
        &mut self,
        _source_frame_number: i32,
        timestamps: layerize::commit::commit_state::CommitTimestamps,
    ) {
        self.effects
            .push(CompositorMainThreadEffect::MainFrameCommitCompleted {
                commit_finish_time: timestamps.finish,
            });
    }
    fn ImplNotification(&mut self, _task: compositor::ImplToMainTask) {}
    fn DidCompositorScroll(&mut self, effect: layerize::commit::main_effects::PacScrollEffect) {
        self.effects.push(effect.into());
    }
}

struct LifecycleClient<'a> {
    effects: EffectClient,
    now: Arc<dyn compositor::CompositorExecutor>,
    update_rendering: &'a mut dyn FnMut(Vec<CompositorMainThreadEffect>),
}
impl ProxyMainClient<compositor::MainFamily> for LifecycleClient<'_> {
    fn CommitDeferralChanged(&mut self, deferred: bool, reason: compositor::PaintHoldingReason) {
        self.effects.CommitDeferralChanged(deferred, reason)
    }
    fn ApplyViewportChanges(
        &mut self,
        scale: f32,
        data: &layerize::commit::compositor_commit_data::CompositorCommitData,
    ) {
        self.effects.ApplyViewportChanges(scale, data)
    }
    fn ScrollAnimationsFinished(
        &mut self,
        completions: Vec<scroll_animation::compositor_mutation::ScrollAnimationCompletion>,
    ) {
        self.effects.ScrollAnimationsFinished(completions)
    }
    fn DidCommit(
        &mut self,
        source_frame_number: i32,
        timestamps: layerize::commit::commit_state::CommitTimestamps,
    ) {
        self.effects.DidCommit(source_frame_number, timestamps)
    }
    fn ImplNotification(&mut self, task: compositor::ImplToMainTask) {
        self.effects.ImplNotification(task)
    }
    fn DidCompositorScroll(&mut self, effect: layerize::commit::main_effects::PacScrollEffect) {
        self.effects.DidCompositorScroll(effect)
    }
}
impl MainFrameClient<compositor::MainFamily> for LifecycleClient<'_> {
    fn Now(&self) -> compositor::adapter::time::TimeTicks {
        self.now.Now()
    }
    fn UpdateRendering(&mut self, _args: &compositor::adapter::begin_frame::BeginFrameArgs) {
        // The PAC scroll callbacks of ApplyCompositorChanges run before the
        // lifecycle (cc::ScrollCallbacks are synchronous in Chromium).
        let effects = std::mem::take(&mut self.effects.effects);
        (self.update_rendering)(effects);
    }
}
