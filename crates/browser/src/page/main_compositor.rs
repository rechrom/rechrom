//! The Page's main compositor half on the document thread: Blink's LayerTreeView /
//! WebFrameWidget compositor state. The Page domain (`PageDomainServices`)
//! holds `layerize::LayerizeEngine` (main LayerTreeHost,
//! PaintArtifactCompositor, main AnimationHost) and its `ProxyMain`
//! directly; these are their document-thread entry points. The impl half
//! (`compositor::CompositorEngine`) is an independent instance on the
//! compositor thread, owned by the embedding's impl runner, or, for the
//! synchronous internal presenter, on this thread (`synchronous_presenter`).
//! The two are connected only by the Proxy task queues (decision D1).
//!
//! The LayerizeEngine is kept in a `RefCell`: ProxyMain::BeginMainFrame calls
//! back into the Page lifecycle (LayerTreeHost::BeginMainFrame +
//! RequestMainFrameUpdate, proxy_main.cc:357-367), whose Paint borrows the
//! LayerizeEngine for PaintArtifactCompositor::Update, video ForeignLayers and
//! direct animation updates.
use super::*;
use compositor_values::compositor_config::{CompositorConfig, CompositorViewportState};
use layerize::commit::MainFamily;
use layerize::runtime::proxy_main::{MainFrameClient, ProxyMainClient};
pub use layerize::runtime::proxy_main::MainFrameResult;

/// The main half of the Page compositor: Blink's LayerTreeView role.
pub(crate) type MainLayerize = layerize::LayerizeEngine<MainFamily>;
pub(crate) type MainProxy = layerize::runtime::proxy_main::ProxyMain<MainFamily>;
/// The ProxyMain -> ProxyImpl and ProxyImpl -> ProxyMain tasks and the posted
/// BeginMainFrameAndCommitState of the main family.
pub(crate) type MainToImplTask = layerize::commit::proxy::MainToImpl<MainFamily>;
pub(crate) type ImplToMainTask = layerize::commit::proxy::ImplToMain<MainFamily>;
pub(crate) type BeginMainFrameState = layerize::commit::proxy::BeginMainFrameAndCommitState<MainFamily>;

// The main LayerTreeHost is lent to the impl thread inside the protected
// commit sequence.
#[allow(dead_code)]
const _: () = {
    fn assert_send_ref<T: Send + ?Sized>(_: &T) {}
    fn assertions(engine: &MainLayerize) {
        assert_send_ref(engine.Main());
    }
};

/// LayerTreeHost setters the Page makes on its own main half (WebFrameWidget
/// -> LayerTreeHost::SetPageScaleFactorAndLimits / StartPageScaleAnimation /
/// SetHaveScrollEventHandlers / SetEventListenerProperties, and ProxyMain's
/// deferral entry points). They never cross to the impl thread; the commit
/// carries their effect. Only the page-scale setters have Page producers
/// today; the others keep the entry points the former CompositorMutation
/// routing had.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum MainCompositorMutation {
    SetPageScaleFactorAndLimits {
        scale: f32,
        minimum: f32,
        maximum: f32,
    },
    /// WebFrameWidgetImpl::StartPageScaleAnimation
    /// (core/frame/web_frame_widget_impl.cc:1738-1744) ->
    /// LayerTreeHost::StartPageScaleAnimation.
    StartPageScaleAnimation {
        target: compositor_tree::geometry::Point,
        use_anchor: bool,
        scale: f32,
        duration: scroll_animation::TimeDelta,
    },
    SetHaveScrollEventHandlers(bool),
    SetEventListenerProperties {
        class: compositor_values::event_listener_properties::EventListenerClass,
        properties: compositor_values::event_listener_properties::EventListenerProperties,
    },
    FrameLifecycle(layerize::commit::paint_holding::FrameLifecycleMutation),
}

pub type MainToImplQueue =layerize::commit::proxy::ProxyTaskQueue<MainToImplTask>;
pub type ImplToMainQueue = layerize::commit::proxy::ProxyTaskQueue<ImplToMainTask>;

impl PageDomainServices {
    /// The compositor configuration of the Page's main half (and of the
    /// synchronous presenter's impl half).
    pub(super) fn CompositorConfig(viewport: CompositorViewportState) -> CompositorConfig {
        CompositorConfig {
            transparent_background: false,
            initial_viewport_size: viewport.size,
            initial_device_scale_factor: viewport.device_scale_factor,
            ..Default::default()
        }
    }

    /// The widget state of a `width`x`height`@`scale` viewport before the
    /// first artifact (Blink's initial VisualProperties).
    pub(super) fn InitialCompositorViewport(
        width: u32,
        height: u32,
        scale: f64,
    ) -> io::Result<CompositorViewportState> {
        if !scale.is_finite() || scale <= 0.0 || scale > f32::MAX as f64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "viewport scale must be positive",
            ));
        }
        Ok(CompositorViewportState {
            size: CompositorSize::new(
                i32::try_from(width).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "viewport width is too large")
                })?,
                i32::try_from(height).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "viewport height is too large")
                })?,
            ),
            device_scale_factor: scale as f32,
            page_scale_factor: 1.0,
            min_page_scale_factor: 1.0,
            max_page_scale_factor: 1.0,
            visible: true,
        })
    }

    /// Creates the main half on this thread with ProxyMain posting to
    /// `to_impl` (LayerTreeHost::CreateThreaded, cc/trees/layer_tree_host.cc:
    /// the main LayerTreeHost, then its ProxyMain; Blink's
    /// LayerTreeView::Initialize supplies the settings and the main
    /// AnimationHost, here `layerize::runtime::main_host_params`), and
    /// returns the ProxyImpl construction values that ProxyMain::Start posts
    /// with Initialize (proxy_main.cc:80-89,898-914).
    pub(super) fn CreateMainCompositor(
        &self,
        config: CompositorConfig,
        to_impl: MainToImplQueue,
    ) -> io::Result<Box<layerize::commit::proxy::ProxyImplInit>> {
        assert!(self.layerize.get().is_none(), "one main compositor half per Page");
        let main_tasks: layerize::commit::main_callbacks::TaskRunner =
            Arc::new(PageMainTaskRunner(self.compositor_executor.clone()));
        let params = layerize::runtime::main_host_params::CreateMainHostParams(&config)?;
        let in_process = to_impl.IsInProcess();
        let endpoint = layerize::runtime::proxy_main::ProxyMainEndpoint::new(
            to_impl,
            main_tasks,
            params.commit_request_trace,
        );
        let layerize = MainLayerize::new(
            params.paint_features,
            params.paint_diagnostics,
            layerize::commit::main_effects::MainHostInit {
                id: params.id,
                owner: layerize::commit::property_tree_services::PropertyTreesOwnerId::FromOwnerValue(
                    params.id,
                ),
                mutator: params.animation_host.clone(),
                settings: params.settings,
                threaded: false,
                background_color: params.background_color,
                // A single-sequence embedding drains both queues itself; its
                // protected-sequence waits return immediately
                // (layer_tree_host.cc:521-522).
                impl_on_owner_sequence: in_process,
                sink: endpoint.clone()
                    as Arc<dyn layerize::commit::main_effects::MainHostEffectSink>,
            },
        )
        .map_err(|error| io::Error::other(error.to_string()))?;
        let proxy = MainProxy::new(endpoint, params.first_property_owner);
        let init = Box::new(layerize::commit::proxy::ProxyImplInit::new(
            params.id,
            &params.animation_host,
            layerize.PaintValues(),
        ));
        let features = params.property_tree_features;
        let _ = self.proxy_main_endpoint.set(proxy.Endpoint().clone());
        let _ = self.proxy_main.set(RefCell::new(proxy));
        let _ = self.layerize.set(RefCell::new(layerize));
        let _ = self.main_property_tree_features.set(features);
        self.compositor_local_surface_id
            .set(Some(layerize::commit::main_effects::NewParentLocalSurfaceId()));
        Ok(init)
    }

    /// The Page compositor clock (the injected executor's TimeTicks domain).
    pub(super) fn CompositorClock(&self) -> PageCompositorClockMapping {
        PageCompositorClockMapping {
            origin: self.compositor_executor.origin,
        }
    }

    pub(super) fn HasMainCompositor(&self) -> bool {
        self.layerize.get().is_some()
    }

    /// ProxyMain::Start (proxy_main.cc:898-914). `init` is posted with
    /// Initialize and waited for; an in-process embedding passes `None`.
    pub(super) fn StartMainCompositor(&self, init: Option<Box<layerize::commit::proxy::ProxyImplInit>>) {
        self.proxy_main
            .get()
            .expect("created main compositor half")
            .borrow_mut()
            .Start(init);
    }

    /// ProxyMain::Stop, then PAC destruction and root detach.
    pub(super) fn ShutdownMainCompositor(&self) {
        let (Some(proxy), Some(layerize)) = (self.proxy_main.get(), self.layerize.get()) else {
            return;
        };
        if self.main_compositor_shut_down.replace(true) {
            return;
        }
        proxy.borrow_mut().Stop();
        layerize.borrow_mut().Shutdown();
    }

    fn MainLayerize(&self) -> &RefCell<MainLayerize> {
        self.layerize.get().expect("created main compositor half")
    }
    fn MainEndpoint(
        &self,
    ) -> &Arc<layerize::runtime::proxy_main::ProxyMainEndpoint<MainFamily>> {
        self.proxy_main_endpoint
            .get()
            .expect("created main compositor half")
    }

    /// A direct PaintArtifactCompositor property update is admissible only
    /// after the main half has installed at least one PaintArtifact.
    pub(super) fn HasResidentPaintArtifact(&self) -> bool {
        self.layerize
            .get()
            .is_some_and(|layerize| layerize.borrow().HasResidentPaintArtifact())
    }
    /// PaintArtifactCompositor::DirectlyUpdateTransform /
    /// DirectlyUpdateCompositedOpacityValue on the main half, synchronously
    /// in the lifecycle task (paint_property_tree_builder.cc; PAC
    /// paint_artifact_compositor.cc:1458-1469,1479-1495). Each accepted node
    /// calls LayerTreeHost::SetNeedsCommit (property_tree_manager.cc:
    /// 120-136,179-206), which inside ProxyMain::BeginMainFrame only raises
    /// the final pipeline stage to COMMIT (proxy_main.cc:741-756), so this
    /// main frame's own commit carries the values. `false` when a node was
    /// rejected: the caller runs the full Paint, as Blink does.
    pub(super) fn DirectlyUpdateCompositedAnimationProperties(
        &self,
        update: &paint::CompositedAnimationPropertyUpdate,
    ) -> bool {
        self.HasResidentPaintArtifact()
            && self
                .MainLayerize()
                .borrow_mut()
                .ApplyCompositedAnimationProperties(update)
    }
    /// cc::Animation::AddKeyframeModel / RemoveKeyframeModel on the main
    /// AnimationHost (CompositorAnimations::StartAnimationOnCompositor and
    /// CancelAnimationOnCompositor), then MutatorHostClient::
    /// SetMutatorsNeedCommit -> LayerTreeHost::SetNeedsCommit; the
    /// KeyframeModels reach the impl instance in the same commit as the
    /// property-tree element ids (animation_host.cc:485-518). `false` when
    /// the external runtime has not started the main half yet: as in Blink
    /// without a compositor, the animation stays on the document thread.
    pub(super) fn ApplySourcePropertyAnimation(
        &self,
        mutation: layerize::commit::animation::PropertyAnimationMutation,
    ) -> io::Result<bool> {
        if self.external_presentation.get() {
            if !self.HasMainCompositor() {
                return Ok(false);
            }
        } else {
            self.EnsureCompositor(None)?;
        }
        self.MainLayerize()
            .borrow_mut()
            .ApplySourcePropertyAnimation(mutation);
        Ok(true)
    }
    /// The layout zoom of the artifact last pushed to the main half (the
    /// impl side's InputRouter widget scale).
    pub(super) fn MainCompositorInputScale(&self) -> f64 {
        self.compositor_input_scale.get()
    }
    pub(super) fn TakeMainCompositorEffects(&self) -> Vec<CompositorMainThreadEffect> {
        let mut effects = std::mem::take(&mut *self.main_compositor_effects.borrow_mut());
        let Some(endpoint) = self.proxy_main_endpoint.get() else {
            return effects;
        };
        for effect in endpoint.TakeEmbedderEffects() {
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
    pub(super) fn SetNeedsMainCompositorAnimate(&self) {
        if let Some(endpoint) = self.proxy_main_endpoint.get() {
            endpoint.SetNeedsAnimate(false);
        }
    }
    /// ProxyMain::SetNeedsCommit.
    pub(super) fn SetNeedsMainCompositorCommit(&self) {
        self.MainEndpoint().SetNeedsCommit(false);
    }
    /// The Send + Sync request side of ProxyMain, for frame sources.
    pub(super) fn MainCompositorRequests(
        &self,
    ) -> Option<Arc<layerize::runtime::proxy_main::ProxyMainEndpoint<MainFamily>>>
    {
        self.proxy_main_endpoint.get().cloned()
    }

    /// LayerTreeHost::SetViewportRectAndScale with the parent LocalSurfaceId,
    /// page scale limits and visibility (Blink applies VisualProperties on
    /// the main thread; layer_tree_host.cc:1477-1530).
    pub(super) fn SetMainCompositorViewport(
        &self,
        viewport: CompositorViewportState,
    ) -> io::Result<()> {
        let parameters = (viewport.size, viewport.device_scale_factor.to_bits());
        if self
            .compositor_surface_parameters
            .get()
            .is_some_and(|previous| previous != parameters)
        {
            let mut id = self
                .compositor_local_surface_id
                .get()
                .expect("created main compositor half");
            id.parent_sequence_number = id
                .parent_sequence_number
                .checked_add(1)
                .ok_or_else(|| io::Error::other("parent LocalSurfaceId sequence exhausted"))?;
            self.compositor_local_surface_id.set(Some(id));
        }
        self.compositor_surface_parameters.set(Some(parameters));
        let mut layerize = self.MainLayerize().borrow_mut();
        let main = layerize.MainMut();
        main.SetViewportRectAndScale(
            CompositorRect::new(0, 0, viewport.size.width(), viewport.size.height()),
            viewport.device_scale_factor,
            self.compositor_local_surface_id
                .get()
                .expect("created main compositor half"),
        );
        main.SetPageScaleFactorAndLimits(
            viewport.page_scale_factor,
            viewport.min_page_scale_factor,
            viewport.max_page_scale_factor,
        );
        drop(layerize);
        match self
            .proxy_main
            .get()
            .expect("created main compositor half")
            .try_borrow_mut()
        {
            Ok(mut proxy) => proxy.SetVisible(viewport.visible),
            Err(_) => self.pending_main_compositor_visible.set(Some(viewport.visible)),
        }
        Ok(())
    }

    /// The viewport of an artifact rendered at `width`x`height` device
    /// pixels with `scale` device pixels per CSS pixel.
    pub(super) fn MainCompositorViewportFor(
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
        let (page_scale, min, max) = self.compositor_page_scale.get();
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
    pub(super) fn UpdateMainCompositorPaintArtifact(
        &self,
        artifact: &Arc<paint::PaintArtifact>,
        width: u32,
        height: u32,
        scale: f64,
    ) -> io::Result<()> {
        assert!(
            self.in_main_compositor_frame.get(),
            "PaintArtifactCompositor::Update runs inside ProxyMain::BeginMainFrame"
        );
        let key = (width, height, scale.to_bits());
        if self
            .pushed_paint_artifact
            .borrow()
            .as_ref()
            .is_some_and(|(pushed, revision, viewport)| {
                *viewport == key
                    && *revision == artifact.recording_revision
                    && pushed
                        .upgrade()
                        .is_some_and(|pushed| Arc::ptr_eq(&pushed, artifact))
            })
        {
            return Ok(());
        }
        let (viewport, layout_zoom) = self.MainCompositorViewportFor(artifact, width, height, scale)?;
        self.SetMainCompositorViewport(viewport)?;
        let mut trace = browser_tracing::span("compositor_layerize", "Layerize.PaintArtifact");
        trace.set("display_items", artifact.display_items.get(&paint::artifact_access()).len() as f64);
        trace.set("chunks", artifact.chunks.get(&paint::artifact_access()).len() as f64);
        let mut layerize = self.MainLayerize().borrow_mut();
        let input = layerize
            .PrepareInput(artifact.clone())
            .map_err(|error| io::Error::other(error.to_string()))?;
        let mut sequence = self.compositor_pac_sequence.get();
        let publication = layerize
            .UpdatePaintArtifact(artifact, input, &mut sequence)
            .map_err(|error| io::Error::other(error.to_string()))?;
        self.compositor_pac_sequence.set(sequence);
        drop(layerize);
        if let Some(publication) = publication {
            self.main_compositor_effects
                .borrow_mut()
                .push(CompositorMainThreadEffect::CanvasChildPaintRecords(publication));
        }
        self.compositor_input_scale.set(layout_zoom);
        *self.pushed_paint_artifact.borrow_mut() =
            Some((Arc::downgrade(artifact), artifact.recording_revision, key));
        Ok(())
    }

    /// The main-thread compositor mutations: LayerTreeHost setters and
    /// frame-lifecycle deferrals queued by input routing.
    pub(super) fn MutateMainCompositor(&self, mutation: MainCompositorMutation) -> io::Result<()> {
        use layerize::commit::paint_holding::FrameLifecycleMutation;
        match mutation {
            MainCompositorMutation::SetPageScaleFactorAndLimits {
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
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "invalid page scale limits",
                    ));
                }
                self.compositor_page_scale.set((scale, minimum, maximum));
                self.MainLayerize()
                    .borrow_mut()
                    .MainMut()
                    .SetPageScaleFactorAndLimits(scale, minimum, maximum);
            }
            MainCompositorMutation::StartPageScaleAnimation {
                target,
                use_anchor,
                scale,
                duration,
            } => {
                if !scale.is_finite() || scale <= 0. || duration.InMicroseconds() < 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "invalid page scale animation",
                    ));
                }
                self.MainLayerize()
                    .borrow_mut()
                    .MainMut()
                    .StartPageScaleAnimation(target, use_anchor, scale, duration);
            }
            // WebFrameWidgetImpl.cc:2220-2232 -> LayerTreeHost.
            MainCompositorMutation::SetHaveScrollEventHandlers(value) => self
                .MainLayerize()
                .borrow_mut()
                .MainMut()
                .SetHaveScrollEventHandlers(value),
            MainCompositorMutation::SetEventListenerProperties {
                class,
                properties,
            } => self
                .MainLayerize()
                .borrow_mut()
                .MainMut()
                .SetEventListenerProperties(class, properties),
            MainCompositorMutation::FrameLifecycle(mutation) => {
                let mut layerize = self.MainLayerize().borrow_mut();
                match mutation {
                    // Source: cc/trees/proxy_main.cc:792-815.
                    FrameLifecycleMutation::SetDeferMainFrameUpdates(deferred) => {
                        if layerize
                            .MainFrameLifecycleMut()
                            .SetDeferMainFrameUpdates(deferred)
                        {
                            self.MainEndpoint().Post(
                                layerize::commit::proxy::MainToImpl::SetDeferBeginMainFrame(
                                    deferred,
                                ),
                            );
                            self.main_compositor_effects.borrow_mut().push(
                                CompositorMainThreadEffect::MainFrameUpdatesDeferralChanged {
                                    deferred,
                                },
                            );
                        }
                    }
                    // Source: cc/trees/proxy_main.cc:848-879.
                    FrameLifecycleMutation::StartDeferringCommits { timeout, reason } => {
                        if layerize.MainFrameLifecycleMut().StartDeferringCommits(
                            self.compositor_executor.Now(),
                            timeout,
                            reason,
                        ) {
                            self.main_compositor_effects.borrow_mut().push(
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
                            self.main_compositor_effects.borrow_mut().push(
                                CompositorMainThreadEffect::CommitDeferralChanged {
                                    deferred: false,
                                    reason,
                                },
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Blink's live WebScrollbar values on the resident composited
    /// PaintedScrollbarLayer (PaintArtifactCompositor's scrollbar layer for
    /// `element`; scrollbar_display_item.h:93-106,
    /// painted_scrollbar_layer.cc:185-283), synchronously on the main half;
    /// the layer asks LayerTreeHost for a commit. `false` when the scrollbar
    /// has no resident composited layer: the caller falls back to Blink's
    /// ordinary targeted scrollbar paint invalidation.
    pub(super) fn ApplyScrollbarPaint(
        &self,
        element: compositor_values::input_types::ElementId,
        opacity: f32,
        thumb_width: i32,
        needs_repaint: bool,
    ) -> bool {
        let applied = self.layerize.get().is_some_and(|layerize| {
            layerize.borrow_mut().ApplyScrollbarPaintMutation(
                element,
                opacity,
                thumb_width,
                needs_repaint,
            )
        });
        if applied {
            self.main_compositor_mutated_outside_lifecycle.set(true);
        }
        applied
    }

    pub(super) fn ConvertPaintArtifactSubsetToRecord(
        &self,
        artifact: Arc<paint::PaintArtifact>,
        state: &paint::paint_property_tree::PropertyTreeState,
    ) -> Option<cc_paint::CpuPaintRecord<cc_paint::skia_region::SkRegion>> {
        Some(
            self.layerize
                .get()?
                .borrow()
                .ConvertPaintArtifactSubsetToRecord(artifact, state),
        )
    }
    pub(super) fn CreateVideoSurfaceForeignLayer(
        &self,
        range: compositor_values::surface_id::SurfaceRange,
    ) -> Arc<dyn paint::ForeignLayer> {
        self.MainLayerize()
            .borrow_mut()
            .CreateVideoSurfaceForeignLayer(range)
    }

    /// Runs a ProxyImpl -> ProxyMain task other than BeginMainFrame. The
    /// LayerTreeHostClient notifications are queued as effects.
    pub(super) fn RunMainCompositorImplTask(&self, task: ImplToMainTask) {
        let mut client = EffectClient {
            effects: Vec::new(),
        };
        self.proxy_main
            .get()
            .expect("created main compositor half")
            .borrow_mut()
            .RunImplTask(task, &mut self.MainLayerize().borrow_mut(), &mut client);
        self.main_compositor_effects
            .borrow_mut()
            .extend(client.effects);
    }

    /// ProxyMain::BeginMainFrame (proxy_main.cc:174-580) with the embedder
    /// lifecycle `update_rendering`, which runs after the compositor deltas
    /// and mutator events were applied and their effects handed to it.
    pub(super) fn BeginMainCompositorFrame(
        &self,
        state: Box<BeginMainFrameState>,
        update_rendering: &mut dyn FnMut(Vec<CompositorMainThreadEffect>),
    ) -> MainFrameResult {
        let mut client = LifecycleClient {
            effects: EffectClient {
                effects: Vec::new(),
            },
            now: self.compositor_executor.clone(),
            update_rendering,
        };
        self.in_main_compositor_frame.set(true);
        let result = self
            .proxy_main
            .get()
            .expect("created main compositor half")
            .borrow_mut()
            .BeginMainFrame(
                state,
                self.MainLayerize(),
                self.main_property_tree_features
                    .get()
                    .expect("created main compositor half"),
                &mut client,
            );
        self.in_main_compositor_frame.set(false);
        self.main_compositor_effects
            .borrow_mut()
            .extend(client.effects.effects);
        if let Some(visible) = self.pending_main_compositor_visible.take() {
            self.proxy_main
                .get()
                .expect("created main compositor half")
                .borrow_mut()
                .SetVisible(visible);
        }
        result
    }
}

/// The ProxyMain -> LayerTreeHostClient notifications of a ProxyImpl task
/// (proxy_main.cc DidPresentCompositorFrame, NotifyImageDecodeRequestFinished,
/// DidCompletePageScaleAnimation, DidLoseLayerTreeFrameSink, ...) as Page
/// compositor effects. A single-sequence presenter delivers the same values
/// synchronously (SingleThreadProxy calls LayerTreeHost directly).
pub(super) fn ImplNotificationEffect(
    task: ImplToMainTask,
) -> Option<CompositorMainThreadEffect> {
    use layerize::commit::proxy::CompositorEmbedderNotification as N;
    use layerize::commit::proxy::ImplToMain;
    Some(match task {
        ImplToMain::DidPresentCompositorFrame {
            frame_token,
            feedback,
        } => CompositorMainThreadEffect::FramePresented {
            frame_token,
            details: feedback,
        },
        ImplToMain::NotifyImageDecodeRequestFinished {
            request_id,
            speculative,
            decode_succeeded,
        } => CompositorMainThreadEffect::ImageDecodeFinished {
            request_id,
            speculative,
            succeeded: decode_succeeded,
        },
        ImplToMain::DidCompletePageScaleAnimation => {
            CompositorMainThreadEffect::PageScaleAnimationCompleted
        }
        ImplToMain::DidChangeBeginFrameSourcePaused(paused) => {
            CompositorMainThreadEffect::BeginFrameSourcePaused { paused }
        }
        ImplToMain::DidLoseLayerTreeFrameSink => CompositorMainThreadEffect::LayerTreeFrameSinkLost,
        ImplToMain::EmbedderNotification(notification) => match notification {
            N::PresentationPromiseSwapped {
                request,
                frame_token,
                swap_time,
            } => CompositorMainThreadEffect::PresentationPromiseSwapped {
                request,
                frame_token,
                swap_time,
            },
            N::PresentationPromiseDidNotSwap {
                request,
                failure_time,
            } => CompositorMainThreadEffect::PresentationPromiseDidNotSwap {
                request,
                failure_time,
            },
            N::FrameNotProduced { ack, reason } => {
                CompositorMainThreadEffect::FrameNotProduced { ack, reason }
            }
            N::VideoBeginFrame { controller, args } => {
                CompositorMainThreadEffect::VideoBeginFrame { controller, args }
            }
            N::VideoNeedsBeginFrames { needs } => {
                CompositorMainThreadEffect::VideoNeedsBeginFrames { needs }
            }
            N::CompositorMetrics { results } => {
                CompositorMainThreadEffect::CompositorMetrics { results }
            }
        },
        _ => return None,
    })
}

/// LayerTreeHostClient notifications as Page compositor effects.
struct EffectClient {
    effects: Vec<CompositorMainThreadEffect>,
}
impl ProxyMainClient<MainFamily> for EffectClient {
    fn CommitDeferralChanged(
        &mut self,
        deferred: bool,
        reason: layerize::commit::paint_holding::PaintHoldingReason,
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
    fn ImplNotification(&mut self, task: ImplToMainTask) {
        self.effects.extend(ImplNotificationEffect(task));
    }
    fn DidCompositorScroll(&mut self, effect: layerize::commit::main_effects::PacScrollEffect) {
        self.effects.push(effect.into());
    }
}

struct LifecycleClient<'a> {
    effects: EffectClient,
    now: Arc<PageCompositorExecutor>,
    update_rendering: &'a mut dyn FnMut(Vec<CompositorMainThreadEffect>),
}
impl ProxyMainClient<MainFamily> for LifecycleClient<'_> {
    fn CommitDeferralChanged(&mut self, deferred: bool, reason: layerize::commit::paint_holding::PaintHoldingReason) {
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
    fn ImplNotification(&mut self, task: ImplToMainTask) {
        self.effects.ImplNotification(task)
    }
    fn DidCompositorScroll(&mut self, effect: layerize::commit::main_effects::PacScrollEffect) {
        self.effects.DidCompositorScroll(effect)
    }
}
impl MainFrameClient<MainFamily> for LifecycleClient<'_> {
    fn Now(&self) -> scroll_animation::TimeTicks {
        self.now.Now()
    }
    fn UpdateRendering(&mut self, _args: &compositor_values::begin_frame::BeginFrameArgs) {
        // The PAC scroll callbacks of ApplyCompositorChanges run before the
        // lifecycle (cc::ScrollCallbacks are synchronous in Chromium).
        let effects = std::mem::take(&mut self.effects.effects);
        (self.update_rendering)(effects);
    }
}

// VideoFrameSubmitter on the document sequence (media/blink/
// video_frame_submitter.cc, webmediaplayer_impl.cc): the video
// cc::SurfaceLayer is created through the main LayerTreeHost; its child
// Surface frames go to the presenter's Viz (internal) or are queued for the
// embedding's window-owned Viz (external).
impl PageDomainServices {
    fn BuildVideoChildFrame(
        &self,
        video: &image_resource::VideoFrameResource,
    ) -> io::Result<viz_common::CompositorFrame> {
        let resource = self
            .raster_engine
            .CreateImmutableVideoTransferableResource(video)?;
        let width = i32::try_from(video.width)
            .map_err(|_| io::Error::other("video frame width overflow"))?;
        let height = i32::try_from(video.height)
            .map_err(|_| io::Error::other("video frame height overflow"))?;
        let rect = CompositorRect::new(0, 0, width, height);
        let mut pass = viz_common::CompositorRenderPass::new(1);
        pass.id = viz_common::draw_quad::RenderPassId(1);
        pass.output_rect = rect;
        pass.damage_rect = rect;
        pass.has_damage_from_contributing_content = true;
        pass.has_transparent_background = false;
        let shared = pass.AppendSharedQuadState();
        {
            let state = pass.SharedQuadStateMutable(shared);
            state.quad_layer_rect = rect;
            state.visible_quad_layer_rect = rect;
            state.are_contents_opaque = true;
            state.opacity = 1.0;
        }
        pass.AppendTextureQuad(
            shared,
            rect,
            rect,
            false,
            ResourceId(resource.id.0),
            CompositorRectF::new(0.0, 0.0, 1.0, 1.0),
            true,
            CompositorColor4f::default(),
            false,
            false,
            true,
            true,
        );
        let mut frame = viz_common::CompositorFrame::default();
        let token = self.video_frame_token.get().wrapping_add(1).max(1);
        self.video_frame_token.set(token);
        frame.metadata.device_scale_factor = 1.0;
        frame.metadata.page_scale_factor = 1.0;
        frame.metadata.may_contain_video = true;
        frame.metadata.frame_token = token;
        frame.metadata.visible_viewport_size = CompositorSize::new(width, height);
        frame.metadata.root_background_color = CompositorColor4f {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
        frame.render_pass_list.push(Box::new(pass));
        frame.resource_list.push(resource);
        Ok(frame)
    }

    fn SubmitVideoChildFrame(
        &self,
        surface_id: viz::adapter::SurfaceId,
        video: &image_resource::VideoFrameResource,
    ) -> io::Result<()> {
        let frame = self.BuildVideoChildFrame(video)?;
        if self.external_presentation.get() {
            let mut outbound = self.outbound_child_frames.borrow_mut();
            if let Some(pending) = outbound
                .iter_mut()
                .find(|pending| pending.surface_id == surface_id)
            {
                pending.frame = frame;
            } else {
                outbound.push_back(PageChildCompositorFrame { surface_id, frame });
            }
            Ok(())
        } else {
            self.WithSynchronousPresenter(|presenter, _| {
                presenter.SubmitChildFrame(surface_id, frame)
            })
        }
    }

    pub(super) fn PublishVideoFrame(
        &self,
        node_id: u64,
        video: &image_resource::VideoFrameResource,
    ) -> io::Result<Arc<dyn paint::paint_engine::ForeignLayer>> {
        const VIDEO_SURFACE_NAMESPACE: u64 = 1_u64 << 32;
        static NEXT_VIDEO_SURFACE: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(VIDEO_SURFACE_NAMESPACE);

        self.EnsureCompositor(None)?;
        let frame_size = (video.width, video.height);
        let replace_surface = self
            .video_surfaces
            .borrow()
            .get(&node_id)
            .is_none_or(|current| current.frame_size != frame_size);
        if replace_surface {
            let raw = NEXT_VIDEO_SURFACE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if raw < VIDEO_SURFACE_NAMESPACE || raw >= (1_u64 << 63) {
                return Err(io::Error::other("video SurfaceId namespace exhausted"));
            }
            let surface_id = viz::adapter::SurfaceId(raw);
            let range = viz::adapter::VizEngine::RuntimeSurfaceRange(surface_id);
            let layer = self.CreateVideoSurfaceForeignLayer(range);
            self.video_surfaces.borrow_mut().insert(
                node_id,
                VideoSurfaceEntry {
                    surface_id,
                    frame_size,
                    layer,
                },
            );
        }
        let (surface_id, layer) = {
            let surfaces = self.video_surfaces.borrow();
            let entry = surfaces
                .get(&node_id)
                .expect("video surface entry was created");
            (entry.surface_id, Arc::clone(&entry.layer))
        };
        self.SubmitVideoChildFrame(surface_id, video)?;
        Ok(layer)
    }

    /// The delegated media frames queued for the external runtime.
    pub(super) fn TakeChildFrames(&self) -> Vec<PageChildCompositorFrame> {
        self.outbound_child_frames.borrow_mut().drain(..).collect()
    }
}
