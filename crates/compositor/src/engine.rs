//! Page-local compositor command/effect boundary.
//!
//! The assembly owns PAC, main/impl trees, scheduling and tile resources. The
//! embedding supplies execution placement and actual raster storage/replay;
//! neither capability receives a Page, compositor owner or retained tree.
pub use crate::cc::{
    input::scroll_snap_data::TargetSnapAreaElementIds, trees::scroll_source_type::ScrollSourceType,
};
pub use crate::runtime::host_metrics::CustomTrackerResults;
pub use layerize::commit::main_effects::CompositorScrollState;
pub use layerize::commit::paint_holding::{
    CommitDeferralReason, FrameLifecycleMutation, PaintHoldingReason,
};
pub use crate::runtime::metadata::FrameTimingDetails as CompositorFrameTimingDetails;
use crate::{
    adapter::{
        begin_frame::BeginFrameArgs,
        geometry::{PointF, Size, Vector2dF},
        input_types::ElementId,
        layer_tree_impl::ResourceId,
        resource_types::SharedImageFormat,
        time::{TimeDelta, TimeTicks},
    },
    runtime::{
        paint::TileRasterBufferValue,
        resources::{RasterColorSpace, RasterReturnedResource, RasterSharedImage},
    },
    CompositorFrameSubmission,
};
pub use compositor_animation::compositor_mutation::PropertyAnimationMutation;
pub use compositor_animation::keyframe_model::{
    CompositorAnimationCurve, Direction as AnimationDirection, FillMode as AnimationFillMode,
};
pub use compositor_animation::keyframed_animation_curve::{
    KeyframedFloatAnimationCurve, KeyframedTransformAnimationCurve,
    TimingFunction as AnimationTimingFunction,
};
pub use compositor_tree::target_property::Type as AnimationTargetProperty;
use std::{fmt, io, sync::Arc};

/// Independent sequenced execution capability. The embedding may execute
/// directly or enqueue on its own thread; the compositor creates no OS thread.
/// Tasks capture independent work or typed completions, never a Host/Page.
pub trait CompositorExecutor: Send + Sync {
    fn Now(&self) -> TimeTicks;
    fn Execute(&self, task: Box<dyn FnOnce() + Send>);
    fn ExecuteAfter(&self, delay: TimeDelta, task: Box<dyn FnOnce() + Send>);
    fn RunsTasksInCurrentSequence(&self) -> bool;
}

/// Resource allocation and replay primitives, implemented by the raster
/// package. Selection, dependency graphs and completion accounting stay inside
/// the compositor. Frame resources directly retain this backend's read lease.
pub trait CompositorRasterProvider: Send + Sync {
    fn CreateSharedImage(
        &self,
        // Preferred allocation id supplied by the tile resource owner. The
        // returned image's allocation id is authoritative when this hint is
        // already occupied by another live UI or tile allocation.
        resource: ResourceId,
        size: Size,
        format: &SharedImageFormat,
        color_space: &RasterColorSpace,
    ) -> io::Result<RasterSharedImage>;
    fn AcquireBufferForRaster(
        &self,
        resource: ResourceId,
        size: Size,
        format: &SharedImageFormat,
        resource_content_id: u64,
        previous_content_id: u64,
    ) -> io::Result<Box<TileRasterBufferValue>>;
    fn CanPartialRasterIntoProvidedResource(&self) -> bool;
    fn NotifyWorkSubmitted(&self);
    fn Shutdown(&self) -> io::Result<()>;
}

/// The services the embedding injects into `CompositorEngine::new`: the impl
/// sequence's task runner and clock, the raster worker executor and the
/// raster/UI resource backends (Chromium's LayerTreeHostImpl receives its
/// TaskRunnerProvider, RasterContextProvider and TaskGraphRunner the same
/// way, cc/trees/layer_tree_host_impl.cc Create). Presentation stays with
/// the embedding: it receives the move-only frames of `CompositorOutput`
/// and returns acknowledgements, presentation feedback and resources.
#[derive(Clone)]
pub struct CompositorServices {
    pub compositor: Arc<dyn CompositorExecutor>,
    pub raster: Arc<dyn CompositorExecutor>,
    pub raster_provider: Arc<dyn CompositorRasterProvider>,
    /// UI bitmap/surface drawing is an independent capability even when the
    /// embedding supplies the same RasterEngine for both roles.
    pub ui_raster: Arc<dyn crate::runtime::ui::UiRasterServices>,
}

#[derive(Clone, Debug)]
pub struct CompositorConfig {
    pub thread: bool,
    /// Whether the root render pass preserves alpha instead of filling the
    /// viewport with the default opaque background.
    pub transparent_background: bool,
    /// Initial widget viewport used by Blink's compositor settings policy.
    /// The compositor still receives every later viewport through Commit.
    pub initial_viewport_size: Size,
    pub initial_device_scale_factor: f32,
    pub raster_task_limit: usize,
    /// Maximum number of independent task-graph drains offered to the
    /// embedding's raster executor. The executor remains authoritative for
    /// the actual worker count and sequencing policy.
    pub raster_worker_limit: usize,
}
impl Default for CompositorConfig {
    fn default() -> Self {
        Self {
            thread: false,
            transparent_background: false,
            initial_viewport_size: Size::default(),
            initial_device_scale_factor: 1.0,
            raster_task_limit: 32,
            raster_worker_limit: 8,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CompositorViewportState {
    pub size: Size,
    pub device_scale_factor: f32,
    pub page_scale_factor: f32,
    pub min_page_scale_factor: f32,
    pub max_page_scale_factor: f32,
    pub visible: bool,
}
impl CompositorViewportState {
    pub(crate) fn Validate(&self) -> Result<(), CompositorError> {
        if self.size.width() < 0
            || self.size.height() < 0
            || !self.device_scale_factor.is_finite()
            || self.device_scale_factor <= 0.0
            || !self.page_scale_factor.is_finite()
            || self.page_scale_factor <= 0.0
            || !self.min_page_scale_factor.is_finite()
            || self.min_page_scale_factor <= 0.0
            || !self.max_page_scale_factor.is_finite()
            || self.max_page_scale_factor < self.min_page_scale_factor
            || self.page_scale_factor < self.min_page_scale_factor
            || self.page_scale_factor > self.max_page_scale_factor
        {
            return Err(CompositorError::InvalidViewport);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollPhase {
    Begin,
    Update,
    End,
}
/// Platform-neutral InputHandlerProxy gesture values. Deltas retain Blink's
/// gesture signs until conversion by the impl input owner.
#[derive(Clone, Copy, Debug)]
pub struct CompositorScrollGesture {
    /// Opaque input-owner dispatch identity, preserved through hit-test continuation.
    pub dispatch_id: u64,
    pub position: PointF,
    pub delta: Vector2dF,
    pub unconstrained_delta: Vector2dF,
    pub granularity: crate::adapter::input_types::ScrollGranularity,
    pub input_type: crate::adapter::input_handler::ScrollInputType,
    pub is_inertial: bool,
    pub synthetic: bool,
    pub target_viewport: bool,
    pub scroll_element_id: ElementId,
    pub main_thread_hit_tested_reasons: crate::adapter::input_types::MainThreadHitTestReasons,
    pub event_timestamp: TimeTicks,
    pub phase: ScrollPhase,
}
#[derive(Clone, Copy, Debug)]
pub enum CompositorInput {
    SetPageScaleFactorAndLimits {
        scale: f32,
        minimum: f32,
        maximum: f32,
    },
    StartPageScaleAnimation {
        target: crate::adapter::geometry::Point,
        use_anchor: bool,
        scale: f32,
        duration: crate::adapter::time::TimeDelta,
    },
    ScrollGesture(CompositorScrollGesture),
    PinchGesture {
        dispatch_id: u64,
        anchor: PointF,
        scale: f32,
        input_type: crate::adapter::input_handler::ScrollInputType,
        phase: ScrollPhase,
    },
    /// ChromeClient -> WebFrameWidget -> LayerTreeHost pending commit state.
    SetHaveScrollEventHandlers(bool),
    SetEventListenerProperties {
        class: crate::cc::input::event_listener_properties::EventListenerClass,
        properties: crate::cc::input::event_listener_properties::EventListenerProperties,
    },
    /// Original wheel remains with its impl owner until canonical DOM dispatch
    /// acknowledges it; this value contains no DOM/renderer state.
    WheelDOMAcknowledgement {
        request: u64,
        default_prevented: bool,
    },
    /// InputHandlerProxy::ContinueScrollBeginAfterMainThreadHitTest. The id is
    /// returned by the source main-frame hit test, never selected by Page.
    ContinueWheelScroll {
        request: u64,
        scroll_element_id: Option<ElementId>,
    },
    Wheel {
        position: PointF,
        delta: Vector2dF,
        unconstrained_delta: Vector2dF,
        granularity: crate::adapter::input_types::ScrollGranularity,
        is_inertial: bool,
        has_synthetic_phase: bool,
        event_timestamp: crate::adapter::time::TimeTicks,
        phase: ScrollPhase,
    },
    /// WebFrameWidgetImpl's generated kScrollbar gesture. Values carry the
    /// original gesture signs; InputHandlerProxy performs cc state conversion.
    ScrollbarGesture {
        scroll_element_id: ElementId,
        delta: Vector2dF,
        granularity: crate::adapter::input_types::ScrollGranularity,
        event_timestamp: crate::adapter::time::TimeTicks,
        phase: ScrollPhase,
    },
    PinchBegin {
        anchor: PointF,
    },
    PinchUpdate {
        anchor: PointF,
        scale: f32,
    },
    PinchEnd {
        anchor: PointF,
    },
}

/// Ordered Page-to-compositor commands; raw input keeps its original Copy
/// event semantics. PAC direct property and scrollbar-layer updates and the
/// main AnimationHost's KeyframeModels are synchronous calls on the Page's
/// own LayerizeEngine (S1b-8), not mutations.
#[derive(Clone, Debug)]
pub enum CompositorMutation {
    FrameLifecycle(FrameLifecycleMutation),
    Input(CompositorInput),
}
/// A backend can acknowledge a buffer without knowing a display time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationUnavailableReason {
    BackendHasNoTimestamp,
    OffscreenTarget,
    DiscardedBeforePresentation,
}
#[derive(Clone, Debug)]
pub enum CompositorMainThreadEffect {
    /// ProxyImpl::ScheduledActionCommit records this immediately after
    /// FinishCommit, before the separate PostCommit and draw actions.
    MainFrameCommitCompleted {
        commit_finish_time: TimeTicks,
    },
    PresentationFeedbackUnavailable {
        frame_token: u32,
        surface_id: u64,
        viz_submission: u64,
        reason: PresentationUnavailableReason,
    },
    PresentationPromiseSwapped {
        request: u64,
        frame_token: u32,
        swap_time: TimeTicks,
    },
    PresentationPromiseDidNotSwap {
        request: u64,
        failure_time: TimeTicks,
    },
    FramePresented {
        frame_token: u32,
        details: CompositorFrameTimingDetails,
    },
    MainFrameUpdatesDeferralChanged {
        deferred: bool,
    },
    CommitDeferralChanged {
        deferred: bool,
        reason: PaintHoldingReason,
    },
    MainFrameCommitDeferred {
        reason: CommitDeferralReason,
    },
    ScrollAnimationCompleted(scroll_animation::compositor_mutation::ScrollAnimationCompletion),
    /// InputHandlerProxy's main-thread hit-test request
    /// (`CompositorInputEffect::WheelRequiresMainThreadHitTest`), which the
    /// main thread answers with `CompositorInput::ContinueWheelScroll`.
    WheelRequiresMainThreadHitTest {
        request: u64,
        position: PointF,
    },
    CanvasChildPaintRecords(Arc<canvas::CanvasChildRecordPublication>),
    UpdateCompositorScrollState(CompositorScrollState),
    LayerTreeFrameSinkLost,
    PageScaleAnimationCompleted,
    ViewportScaleChanged {
        scale: f32,
        delta: f32,
        pinch_active: bool,
        inner_scroll_delta: Vector2dF,
    },
    VideoBeginFrame {
        controller: u64,
        args: BeginFrameArgs,
    },
    VideoNeedsBeginFrames {
        needs: bool,
    },
    CompositorMetrics {
        results: CustomTrackerResults,
    },
    FrameNotProduced {
        ack: crate::adapter::scheduler_host::BeginFrameAck,
        reason: crate::cc::scheduler::scheduler::FrameSkippedReason,
    },
    /// A single-sequence embedding ran ProxyMain::BeginMainFrame; the Page
    /// schedules its lifecycle (the threaded embedding runs it inside).
    BeginMainFrame,
    SaveActiveEventMetrics,
    UpdateDeferMainFrameUpdate,
    ImageDecodeFinished {
        request_id: i32,
        speculative: bool,
        succeeded: bool,
    },
    BeginFrameSourcePaused {
        paused: bool,
    },
    ScrollOffsetChanged {
        sequence: u64,
        element: ElementId,
        offset: PointF,
        source: ScrollSourceType,
        snap: Option<TargetSnapAreaElementIds>,
    },
    ScrollbarsHiddenChanged {
        element: ElementId,
        hidden: bool,
    },
}

/// The InputHandlerProxy results of one impl input call, returned to the
/// input owner that made it (content/renderer/input/input_handler_proxy.cc:
/// the event disposition callback, DidHandleInputEventAndOverscroll, and
/// the main-thread hit-test / blocking-wheel continuations). They are
/// produced synchronously by the input call and are not ProxyMain tasks.
#[derive(Clone, Copy, Debug)]
pub enum CompositorInputEffect {
    /// A gesture's InputHandlerProxy disposition (consumed or not).
    GestureDisposition { dispatch_id: u64, handled: bool },
    /// The raw wheel's disposition: the impl InputHandler owns its scroll.
    WheelDisposition { compositor_owned: bool },
    /// A blocking wheel waits for the canonical DOM dispatch's
    /// acknowledgement (`CompositorInput::WheelDOMAcknowledgement`).
    WheelRequiresDOMAcknowledgement { request: u64 },
    /// ScrollBegin needs the main thread's hit test
    /// (InputHandlerProxy::ContinueScrollBeginAfterMainThreadHitTest).
    WheelRequiresMainThreadHitTest { request: u64, position: PointF },
}

/// The Rechrom impl -> main notifications without a ProxyMain method
/// (`layerize::commit::proxy::ProxyTypes::EmbedderNotification`).
#[derive(Clone, Debug)]
pub enum CompositorEmbedderNotification {
    /// ReportTimeSwapPromise::DidSwap (cc/trees/swap_promise.h).
    PresentationPromiseSwapped {
        request: u64,
        frame_token: u32,
        swap_time: TimeTicks,
    },
    /// ReportTimeSwapPromise::DidNotSwap.
    PresentationPromiseDidNotSwap { request: u64, failure_time: TimeTicks },
    /// LayerTreeFrameSink::DidNotProduceFrame.
    FrameNotProduced {
        ack: crate::adapter::scheduler_host::BeginFrameAck,
        reason: crate::cc::scheduler::scheduler::FrameSkippedReason,
    },
    /// The video frame provider's BeginFrame (VideoFrameSubmitter).
    VideoBeginFrame { controller: u64, args: BeginFrameArgs },
    VideoNeedsBeginFrames { needs: bool },
    /// LayerTreeHostClient custom tracker metrics.
    CompositorMetrics { results: CustomTrackerResults },
}

/// Move-only frame output: no separate raster bundle or resource snapshot.
/// Impl -> main notifications are not part of it: they are ProxyImpl ->
/// ProxyMain tasks on the `to_main` queue, except the synchronous results
/// of an input call (`input_effects`).
pub struct CompositorOutput {
    pub frame: Option<CompositorFrameSubmission>,
    /// The frame was drawn from the source frame consumed by the most recent
    /// Scheduler COMMIT. This is false for an old active-tree frame submitted
    /// just before a newly committed pending tree activates in the same turn.
    pub frame_matches_latest_commit: bool,
    pub input_effects: Vec<CompositorInputEffect>,
    pub needs_begin_frames: bool,
    /// Typed impl tasks remain queued and require another owner-sequence turn.
    pub work_pending: bool,
    /// ProxyImpl holds a ready commit (DataForCommit) that has not yet
    /// reached Scheduler's COMMIT action.
    pub main_commit_pending: bool,
}

/// Producer input diagnostics contain only values. In particular an error
/// cannot expose/retain PAC, layer/property-tree or tile owners through Page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositorPaintInputKind {
    MissingImageResource,
    ImageMetadata,
    ImagePlayback,
    MissingViewportMetadata,
    InvalidViewportMetadata,
    MissingStickyConstraint,
    UnsupportedCanvasChild,
    InvalidRecordRange,
    InvalidDomNodeId,
    MissingScrollCompositorData,
    MissingChunkCompositorData,
    UnsupportedDisplayItem,
    EffectFilter,
    ClipPath,
    InvalidChunkRange,
}

#[derive(Debug)]
pub enum CompositorError {
    InvalidViewport,
    UnsupportedExecutionConfiguration {
        reason: &'static str,
    },
    PaintInput {
        kind: CompositorPaintInputKind,
        detail: String,
    },
    NotReady {
        stage: &'static str,
    },
    Backend(io::Error),
}
impl fmt::Display for CompositorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidViewport => f.write_str("invalid compositor viewport"),
            Self::UnsupportedExecutionConfiguration { reason } => f.write_str(reason),
            Self::PaintInput { detail, .. } => f.write_str(detail),
            Self::NotReady { stage } => write!(f, "compositor assembly is not ready: {stage}"),
            Self::Backend(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for CompositorError {}
impl From<io::Error> for CompositorError {
    fn from(error: io::Error) -> Self {
        Self::Backend(error)
    }
}

impl From<layerize::commit::layerize_error::LayerizeError> for CompositorError {
    fn from(error: layerize::commit::layerize_error::LayerizeError) -> Self {
        use layerize::commit::layerize_error::LayerizeErrorKind as E;
        let kind = match error.kind {
            E::MissingImageResource => CompositorPaintInputKind::MissingImageResource,
            E::ImageMetadata => CompositorPaintInputKind::ImageMetadata,
            E::ImagePlayback => CompositorPaintInputKind::ImagePlayback,
            E::MissingViewportMetadata => CompositorPaintInputKind::MissingViewportMetadata,
            E::InvalidViewportMetadata => CompositorPaintInputKind::InvalidViewportMetadata,
            E::MissingStickyConstraint => CompositorPaintInputKind::MissingStickyConstraint,
            E::InvalidCanvasChild => CompositorPaintInputKind::UnsupportedCanvasChild,
            E::InvalidRecordRange => CompositorPaintInputKind::InvalidRecordRange,
            E::MissingScrollCompositorData => CompositorPaintInputKind::MissingScrollCompositorData,
            E::MissingChunkCompositorData => CompositorPaintInputKind::MissingChunkCompositorData,
            E::UnsupportedDisplayItem => CompositorPaintInputKind::UnsupportedDisplayItem,
            E::EffectFilter => CompositorPaintInputKind::EffectFilter,
            E::ClipPath => CompositorPaintInputKind::ClipPath,
            E::InvalidChunkRange => CompositorPaintInputKind::InvalidChunkRange,
            E::InvalidDomNodeId => CompositorPaintInputKind::InvalidDomNodeId,
            E::MissingCanvasDocument => CompositorPaintInputKind::UnsupportedCanvasChild,
        };
        Self::PaintInput {
            kind,
            detail: error.detail,
        }
    }
}
impl From<layerize::commit::main_effects::PacScrollEffect> for CompositorMainThreadEffect {
    fn from(effect: layerize::commit::main_effects::PacScrollEffect) -> Self {
        use layerize::commit::main_effects::PacScrollEffect as E;
        match effect {
            E::DidCompositorScroll {
                sequence,
                element,
                offset,
                source,
                snap,
            } => Self::ScrollOffsetChanged {
                sequence,
                element,
                offset,
                source,
                snap,
            },
            E::DidUpdateCompositorScrollState(state) => Self::UpdateCompositorScrollState(state),
            E::DidChangeScrollbarsHidden { element, hidden } => {
                Self::ScrollbarsHiddenChanged { element, hidden }
            }
        }
    }
}

/// Public commands intentionally carry no layer, tiling, job or graph owner.
/// Construction remains explicit until the real canonical assembly is ready;
/// no command reports success with a fabricated frame or skipped commit.
/// Immutable source conversion output. Its original records retain their
/// PaintArtifact/range owners; no mutable compositor service escapes.
pub type ImmutablePaintRecord =
    crate::runtime::paint::CpuPaintRecord<crate::runtime::skia::SkRegion>;

/// The single Layerize family of this compositor (S1B_PROXY_DESIGN.md 1).
pub type MainFamily = crate::runtime::types::CpuCompositorTypes<crate::runtime::skia::SkRegion>;
/// ProxyMain -> ProxyImpl task of this family.
pub type MainToImplTask = layerize::commit::proxy::MainToImpl<MainFamily>;
/// ProxyImpl -> ProxyMain task of this family.
pub type ImplToMainTask = layerize::commit::proxy::ImplToMain<MainFamily>;
/// ProxyMain::BeginMainFrame's answer for this family.
pub type MainFrameResult = layerize::runtime::proxy_main::MainFrameResult;
/// The posted BeginMainFrameAndCommitState of this family.
pub type BeginMainFrameState = layerize::commit::proxy::BeginMainFrameAndCommitState<MainFamily>;
/// The main half of a Page compositor: Blink's LayerTreeView role.
pub type MainLayerize = layerize::LayerizeEngine<MainFamily>;
pub type MainProxy = layerize::runtime::proxy_main::ProxyMain<MainFamily>;
pub type ProxyImplInit = crate::runtime::engine_factory::ProxyImplInit<crate::runtime::skia::SkRegion>;
/// The main half's PropertyTreeFeatures (draw_property_utils /
/// LayerTreeHost::UpdateLayers inputs), built from the same settings as the
/// impl half's.
pub type MainPropertyTreeFeatures = crate::runtime::render::RenderServices;

// Compile-time thread-safety of the main-half values that cross to, or are
// read by, the impl thread during the commit (S1B_PROXY_DESIGN.md B4): the
// main LayerTreeHost (lent inside the protected sequence), its
// ThreadUnsafeCommitState (read by FinishCommit), the moved CommitState and
// the posted Proxy tasks.
#[allow(dead_code)]
const _: () = {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_send_ref<T: Send + ?Sized>(_: &T) {}
    fn assertions(engine: &layerize::LayerizeEngine<MainFamily>) {
        assert_send_ref(engine.Main());
        assert_sync::<layerize::commit::commit_state::ThreadUnsafeCommitState<MainFamily>>();
        assert_send::<layerize::commit::commit_state::CommitState<MainFamily>>();
        assert_send::<MainToImplTask>();
        assert_send::<ImplToMainTask>();
    }
};

/// Creates the main half of one Page compositor on the calling (document)
/// thread: the main LayerTreeHost, PaintArtifactCompositor and main
/// AnimationHost (`layerize::LayerizeEngine`, Blink's LayerTreeView) and its
/// ProxyMain over `to_impl`. Returns the ProxyImpl construction values that
/// `ProxyMain::Start` posts with Initialize (proxy_main.cc:80-89,898-914),
/// from which the impl thread builds its `CompositorEngine`. `main_tasks`
/// runs the main LayerTreeHost's posted main-thread callbacks.
pub fn CreateMainHalf(
    config: CompositorConfig,
    to_impl: layerize::commit::proxy::ProxyTaskQueue<MainToImplTask>,
    main_tasks: layerize::commit::main_callbacks::TaskRunner,
) -> Result<(MainLayerize, MainProxy, MainPropertyTreeFeatures, Box<ProxyImplInit>), CompositorError>
{
    let parts = crate::runtime::engine_factory::MainHalfParts::new(&config)?;
    let in_process = to_impl.IsInProcess();
    let endpoint = layerize::runtime::proxy_main::ProxyMainEndpoint::new(
        to_impl,
        main_tasks,
        parts.CommitRequestTrace(),
    );
    let layerize = layerize::LayerizeEngine::<MainFamily>::new(
        parts.paint_features,
        parts.paint_diagnostics,
        layerize::commit::main_effects::MainHostInit {
            id: parts.owner,
            owner: crate::adapter::property_tree::PropertyTreesOwnerId::FromOwnerValue(parts.owner),
            mutator: parts.main_animations.clone(),
            settings: parts.settings.clone(),
            threaded: false,
            // Chromium's WebLayerTreeView forwards the embedder background
            // color to LayerTreeHost. An alpha-zero root keeps popup Pages
            // compositable over the document.
            background_color: config.transparent_background.then_some(
                crate::adapter::layer_tree_impl::Color4f {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 0.0,
                },
            ),
            // A single-sequence embedding drains both queues itself; its
            // protected-sequence waits return immediately
            // (layer_tree_host.cc:521-522).
            impl_on_owner_sequence: in_process,
            sink: endpoint.clone() as Arc<dyn layerize::commit::main_effects::MainHostEffectSink>,
        },
    )
    .map_err(|error| CompositorError::Backend(std::io::Error::other(error.to_string())))?;
    let proxy = layerize::runtime::proxy_main::ProxyMain::new(endpoint, parts.owner + 1);
    let init = Box::new(parts.ImplInit(layerize.PaintValues()));
    let features = parts.features;
    Ok((layerize, proxy, features, init))
}

/// The impl half of one Page compositor (cc LayerTreeHostImpl, Scheduler,
/// ProxyImpl) on the compositor thread. It receives the main thread's
/// commits only as `CommitState` values with their protected lease; its
/// outputs carry the ProxyImpl -> ProxyMain tasks.
pub struct CompositorEngine {
    assembly: crate::runtime::engine_assembly::EngineAssembly,
}
impl CompositorEngine {
    /// ProxyMain::InitializeOnImplThread -> ProxyImpl construction
    /// (proxy_main.cc:80-89) from the values ProxyMain::Start posted.
    pub fn new(
        config: CompositorConfig,
        services: CompositorServices,
        init: Box<ProxyImplInit>,
        to_main: layerize::commit::proxy::ProxyTaskQueue<ImplToMainTask>,
    ) -> Result<Self, CompositorError> {
        crate::runtime::engine_factory::CreateImpl(config, services, init, to_main)
            .map(|assembly| Self { assembly })
    }
    /// ProxyMain::Stop -> DestroyProxyImplOnImplThread.
    pub fn Shutdown(&mut self) {
        self.assembly.Shutdown()
    }
    /// ProxyImpl::NotifyReadyToCommitOnImpl (proxy_impl.cc:425-493).
    pub fn ReadyToCommit(
        &mut self,
        commit_state: Box<layerize::commit::commit_state::CommitState<MainFamily>>,
        lease: layerize::commit::proxy::ProtectedCommitLease<MainFamily>,
        main_thread_start_time: TimeTicks,
        scroll_and_viewport_changes_synced: bool,
        commit_timeout: bool,
    ) -> Result<CompositorOutput, CompositorError> {
        self.assembly.ReadyToCommit(
            commit_state,
            lease,
            main_thread_start_time,
            scroll_and_viewport_changes_synced,
            commit_timeout,
        )
    }
    /// ProxyImpl::BeginMainFrameAbortedOnImpl (proxy_impl.cc:306-323).
    pub fn BeginMainFrameAborted(
        &mut self,
        reason: crate::cc::scheduler::commit_earlyout_reason::CommitEarlyOutReason,
        main_thread_start_time: TimeTicks,
        swap_promises: Vec<Box<crate::runtime::host_owners::SwapPromise<crate::runtime::skia::SkRegion>>>,
        scroll_and_viewport_changes_synced: bool,
    ) -> Result<CompositorOutput, CompositorError> {
        self.assembly.BeginMainFrameAborted(
            reason,
            main_thread_start_time,
            swap_promises,
            scroll_and_viewport_changes_synced,
        )
    }
    /// The other ProxyMain -> ProxyImpl tasks (SetNeedsCommitOnImpl,
    /// SetDeferBeginMainFrameFromMain, SetVisibleOnImpl, ...). Initialize and
    /// Stop belong to the embedder that owns this engine's lifetime.
    pub fn RunMainThreadTask(
        &mut self,
        task: MainToImplTask,
    ) -> Result<CompositorOutput, CompositorError> {
        self.assembly.RunMainThreadTask(task)
    }
    /// Requests a compositor frame from the resident active tree without
    /// a main-tree commit.
    pub fn RequestRedraw(&mut self) -> Result<CompositorOutput, CompositorError> {
        self.assembly.RequestRedraw()
    }
    pub fn BeginFrame(&mut self, args: BeginFrameArgs) -> Result<CompositorOutput, CompositorError> {
        self.assembly.BeginFrame(args)
    }
    pub fn Input(&mut self, input: CompositorInput) -> Result<CompositorOutput, CompositorError> {
        self.assembly.Input(input)
    }
    /// Impl-side mutations only (input routing); main-side mutations
    /// (`CompositorMutation::IsMainThreadMutation`) belong to the main half.
    pub fn Mutate(
        &mut self,
        mutation: CompositorMutation,
    ) -> Result<CompositorOutput, CompositorError> {
        match mutation {
            CompositorMutation::Input(input) if !mutation_is_main_input(&input) => {
                self.assembly.Input(input)
            }
            _ => Err(CompositorError::NotReady {
                stage: "main-thread compositor mutation reached the impl half",
            }),
        }
    }
    /// Signals that Viz accepted a submitted frame.
    pub fn DidReceiveFrameAck(&mut self) -> Result<CompositorOutput, CompositorError> {
        self.assembly.DidReceiveFrameAck()
    }
    /// Actual Viz presentation feedback.
    pub fn DidPresentFrame(
        &mut self,
        frame_token: u32,
        details: CompositorFrameTimingDetails,
    ) -> Result<CompositorOutput, CompositorError> {
        self.assembly.DidPresentFrame(frame_token, details)
    }
    pub fn ReclaimResources(
        &mut self,
        resources: Vec<RasterReturnedResource>,
    ) -> Result<CompositorOutput, CompositorError> {
        self.assembly.ReclaimResources(resources)
    }
    pub fn Poll(&mut self) -> Result<CompositorOutput, CompositorError> {
        self.assembly.Poll()
    }
}

fn mutation_is_main_input(input: &CompositorInput) -> bool {
    matches!(
        input,
        CompositorInput::SetPageScaleFactorAndLimits { .. }
            | CompositorInput::SetHaveScrollEventHandlers(_)
            | CompositorInput::SetEventListenerProperties { .. }
    )
}
impl CompositorMutation {
    /// The mutation targets the main LayerTreeHost or ProxyMain (main
    /// thread), not InputHandlerProxy (impl).
    pub fn IsMainThreadMutation(&self) -> bool {
        match self {
            Self::Input(input) => mutation_is_main_input(input),
            Self::FrameLifecycle(_) => true,
        }
    }
}


/// The first parent LocalSurfaceId of a Page widget (Blink's
/// ParentLocalSurfaceIdAllocator::GenerateId with a new embed token,
/// components/viz/common/surfaces/parent_local_surface_id_allocator.cc).
pub fn NewParentLocalSurfaceId() -> crate::runtime::types::LocalSurfaceId {
    crate::runtime::types::LocalSurfaceId {
        parent_sequence_number: 1,
        child_sequence_number: 1,
        embed_token: crate::runtime::host_core::SurfaceTokenSource::NextEmbedToken(
            &crate::runtime::host_core::OsSurfaceTokenSource,
        ),
    }
}
