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
pub use layerize::commit::proxy::CompositorEmbedderNotification;
use crate::{
    adapter::{
        begin_frame::BeginFrameArgs,
        geometry::{PointF, Vector2dF},
        input_types::ElementId,
        time::{TimeDelta, TimeTicks},
    },
    runtime::resources::RasterReturnedResource,
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
pub use cc_raster::raster_buffer_provider::CompositorRasterProvider;

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

/// The Page compositor configuration and widget viewport values are shared
/// with the main half (compositor_values::compositor_config).
pub use compositor_values::compositor_config::{CompositorConfig, CompositorViewportState};

/// The cc gesture-scroll phase lives with the input-queue values.
pub use compositor_values::input_to_impl::ScrollPhase;
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
/// Public commands intentionally carry no layer, tiling, job or graph owner.
/// Construction remains explicit until the real canonical assembly is ready;
/// no command reports success with a fabricated frame or skipped commit.
/// Immutable source conversion output. Its original records retain their
/// PaintArtifact/range owners; no mutable compositor service escapes.
pub type ImmutablePaintRecord =
    crate::runtime::paint::CpuPaintRecord<crate::runtime::skia::SkRegion>;

/// The main (Layerize) family of this compositor: the shared commit family
/// (layerize::commit::CommitFamily) its impl family names as `Main`.
pub type MainFamily = layerize::commit::MainFamily;
/// ProxyMain -> ProxyImpl task of this family.
pub type MainToImplTask = layerize::commit::proxy::MainToImpl<MainFamily>;
/// ProxyImpl -> ProxyMain task of this family.
pub type ImplToMainTask = layerize::commit::proxy::ImplToMain<MainFamily>;
/// The posted BeginMainFrameAndCommitState of this family.
pub type BeginMainFrameState = layerize::commit::proxy::BeginMainFrameAndCommitState<MainFamily>;
pub type ProxyImplInit = layerize::commit::proxy::ProxyImplInit;

// Compile-time thread-safety of the commit values that cross to, or are read
// by, the impl thread during the commit: the main
// LayerTreeHost's ThreadUnsafeCommitState (read by FinishCommit), the moved
// CommitState and the posted Proxy tasks. The lent main LayerTreeHost itself
// is asserted where the Page constructs it (browser `page::main_compositor`).
#[allow(dead_code)]
const _: () = {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assertions() {
        assert_sync::<layerize::commit::commit_state::ThreadUnsafeCommitState<MainFamily>>();
        assert_send::<layerize::commit::commit_state::CommitState<MainFamily>>();
        assert_send::<MainToImplTask>();
        assert_send::<ImplToMainTask>();
    }
};

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
        swap_promises: Vec<Box<crate::runtime::host_owners::SwapPromise>>,
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

