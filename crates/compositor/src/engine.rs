//! Page-local compositor command/effect boundary.
//!
//! The assembly owns PAC, main/impl trees, scheduling and tile resources. The
//! embedding supplies execution placement and actual raster storage/replay;
//! neither capability receives a Page, compositor owner or retained tree.
pub use crate::cc::{
    input::scroll_snap_data::TargetSnapAreaElementIds, trees::scroll_source_type::ScrollSourceType,
};
pub use crate::runtime::host_metrics::CustomTrackerResults;
pub use crate::runtime::main_frame_lifecycle::{
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
use paint::PaintArtifact;
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

pub struct CompositorExecutors {
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

/// Only the compositor can mint the pending main commit's identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollAnimationCommitToken(pub(crate) u64);
#[derive(Clone, Copy, Debug)]
pub struct ScrollAnimationAreaClassification {
    pub element: ElementId,
    pub uses_composited_scrolling: bool,
    pub reasons: crate::cc::input::main_thread_scrolling_reason::MainThreadRepaintReasons,
}
/// Ordered Page-to-compositor commands; curves retain the canonical shared
/// value type while raw input keeps its original Copy event semantics.
#[derive(Clone, Debug)]
pub enum CompositorMutation {
    RequestPresentationTime {
        request: u64,
    },
    FrameLifecycle(FrameLifecycleMutation),
    Input(CompositorInput),
    ScrollAnimation(scroll_animation::compositor_mutation::ScrollAnimationMutation),
    FinalizeScrollAnimations {
        token: ScrollAnimationCommitToken,
        commands: Vec<scroll_animation::compositor_mutation::ScrollAnimationMutation>,
    },
}
/// LayerTreeHostClient::UpdateCompositorScrollState, from the same commit
/// as the preceding scroll deltas. Element/source identities remain native.
#[derive(Clone, Debug)]
pub struct CompositorScrollState {
    pub is_scroll_active: bool,
    pub scroll_latched_element_id: ElementId,
    pub source_type: ScrollSourceType,
    pub done_containers: Vec<ElementId>,
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
    ScrollAnimationCommitPrepared {
        token: ScrollAnimationCommitToken,
        areas: Vec<ScrollAnimationAreaClassification>,
    },
    ScrollAnimationCommitFinalized {
        token: ScrollAnimationCommitToken,
    },
    ScrollAnimationCompleted(scroll_animation::compositor_mutation::ScrollAnimationCompletion),
    GestureDisposition {
        dispatch_id: u64,
        handled: bool,
    },
    WheelRequiresDOMAcknowledgement {
        request: u64,
    },
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
    /// Scheduler has entered BeginMainFrameState::SENT and the main thread
    /// must answer this transaction with a commit or an explicit early-out.
    BeginMainFrame,
    /// MainHost asks Page to schedule lifecycle work. Unlike BeginMainFrame,
    /// this does not prove that Scheduler has dispatched a transaction.
    RequestMainFrame,
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
    WheelDisposition {
        compositor_owned: bool,
    },
}

/// Move-only frame output: no separate raster bundle or resource snapshot.
pub struct CompositorOutput {
    pub frame: Option<CompositorFrameSubmission>,
    /// The frame was drawn from the source frame consumed by the most recent
    /// Scheduler COMMIT. This is false for an old active-tree frame submitted
    /// just before a newly committed pending tree activates in the same turn.
    pub frame_matches_latest_commit: bool,
    pub main_thread_effects: Vec<CompositorMainThreadEffect>,
    pub needs_begin_frames: bool,
    /// Typed impl tasks remain queued and require another owner-sequence turn.
    pub work_pending: bool,
    /// The accepted main commit has not yet reached Scheduler's COMMIT action.
    /// Embedders may retain a newer PaintArtifact, but must not submit it yet.
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

impl From<crate::runtime::paint::PaintInputError> for CompositorError {
    fn from(error: crate::runtime::paint::PaintInputError) -> Self {
        use crate::runtime::paint::PaintInputError as E;
        let kind = match &error {
            E::MissingImageResource { .. } => CompositorPaintInputKind::MissingImageResource,
            E::ImageMetadata { .. } => CompositorPaintInputKind::ImageMetadata,
            E::ImagePlayback { .. } => CompositorPaintInputKind::ImagePlayback,
            E::MissingViewportMetadata => CompositorPaintInputKind::MissingViewportMetadata,
            E::InvalidViewportMetadata { .. } => CompositorPaintInputKind::InvalidViewportMetadata,
            E::MissingStickyConstraint { .. } => CompositorPaintInputKind::MissingStickyConstraint,
            E::InvalidCanvasChild { .. } => CompositorPaintInputKind::UnsupportedCanvasChild,
            E::InvalidRecordRange { .. } => CompositorPaintInputKind::InvalidRecordRange,
            E::MissingScrollCompositorData { .. } => {
                CompositorPaintInputKind::MissingScrollCompositorData
            }
            E::MissingChunkCompositorData { .. } => {
                CompositorPaintInputKind::MissingChunkCompositorData
            }
            E::UnsupportedDisplayItem { .. } => CompositorPaintInputKind::UnsupportedDisplayItem,
            E::EffectFilter { .. } => CompositorPaintInputKind::EffectFilter,
            E::ClipPath { .. } => CompositorPaintInputKind::ClipPath,
            E::InvalidChunkRange { .. } => CompositorPaintInputKind::InvalidChunkRange,
        };
        Self::PaintInput {
            kind,
            detail: error.to_string(),
        }
    }
}
impl From<crate::runtime::pac_scroll::PacScrollEffect> for CompositorMainThreadEffect {
    fn from(effect: crate::runtime::pac_scroll::PacScrollEffect) -> Self {
        use crate::runtime::pac_scroll::PacScrollEffect as E;
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

pub struct CompositorEngine {
    assembly: crate::runtime::engine_assembly::EngineAssembly,
}
impl CompositorEngine {
    /// PaintRecordBuilder::EndRecording, source paint_record_builder.cc:36-44.
    /// This is a value-only conversion using the same Page compositor's source
    /// Paint services. It neither commits nor mutates the resident PAC/tree.
    pub fn ConvertPaintArtifactSubsetToRecord(
        &self,
        artifact: Arc<PaintArtifact>,
        state: &paint::paint_property_tree::PropertyTreeState,
    ) -> ImmutablePaintRecord {
        self.assembly
            .ConvertPaintArtifactSubsetToRecord(artifact, state)
    }

    pub fn new(
        config: CompositorConfig,
        executors: CompositorExecutors,
    ) -> Result<Self, CompositorError> {
        crate::runtime::engine_factory::CreateDefault(config, executors)
            .map(|assembly| Self { assembly })
    }
    pub fn Commit(
        &mut self,
        artifact: Arc<PaintArtifact>,
        viewport: CompositorViewportState,
    ) -> Result<CompositorOutput, CompositorError> {
        self.assembly.Commit(artifact, viewport)
    }
    pub fn CommitWithScrollAnimations(
        &mut self,
        artifact: Arc<PaintArtifact>,
        viewport: CompositorViewportState,
        areas: Vec<ElementId>,
    ) -> Result<CompositorOutput, CompositorError> {
        self.assembly
            .CommitWithScrollAnimations(artifact, viewport, areas)
    }
    /// Creates the resident cc::SurfaceLayer used by VideoPainter's
    /// kForeignLayerVideo item. The child frame is submitted separately to Viz
    /// and identified by `range`; Browser never receives compositor services.
    pub fn CreateVideoSurfaceForeignLayer(
        &mut self,
        range: crate::runtime::types::SurfaceRange,
    ) -> Arc<dyn paint::ForeignLayer> {
        self.assembly.CreateVideoSurfaceForeignLayer(range)
    }
    pub fn SetViewport(
        &mut self,
        viewport: CompositorViewportState,
    ) -> Result<CompositorOutput, CompositorError> {
        self.assembly.SetViewport(viewport)
    }
    /// Mirrors LayerTreeHost/ProxyMain commit demand before the main-frame
    /// lifecycle has produced its PaintArtifact.
    pub fn RequestMainFrame(&mut self) -> Result<CompositorOutput, CompositorError> {
        self.assembly.RequestMainFrame()
    }
    /// Requests a compositor frame from the resident active tree without
    /// manufacturing a new PaintArtifact or main-tree commit.
    pub fn RequestRedraw(&mut self) -> Result<CompositorOutput, CompositorError> {
        self.assembly.RequestRedraw()
    }
    pub fn FinishMainFrameWithoutUpdates(&mut self) -> Result<CompositorOutput, CompositorError> {
        self.assembly.FinishMainFrameWithoutUpdates()
    }
    pub fn BeginFrame(
        &mut self,
        args: BeginFrameArgs,
    ) -> Result<CompositorOutput, CompositorError> {
        self.assembly.BeginFrame(args)
    }
    pub fn Input(&mut self, input: CompositorInput) -> Result<CompositorOutput, CompositorError> {
        self.assembly.Input(input)
    }
    pub fn Mutate(
        &mut self,
        mutation: CompositorMutation,
    ) -> Result<CompositorOutput, CompositorError> {
        match mutation {
            CompositorMutation::RequestPresentationTime { request } => {
                self.assembly.RequestPresentationTime(request)
            }
            CompositorMutation::FrameLifecycle(mutation) => {
                self.assembly.ApplyFrameLifecycle(mutation)
            }
            CompositorMutation::Input(input) => self.assembly.Input(input),
            CompositorMutation::ScrollAnimation(mutation) => {
                self.assembly.ApplySourceScrollAnimation(mutation)
            }
            CompositorMutation::FinalizeScrollAnimations { token, commands } => {
                self.assembly.FinalizeScrollAnimationCommit(token, commands)
            }
        }
    }
    /// Signals that Viz accepted a submitted frame. This does not reclaim
    /// resources; their actual lease returns arrive through ReclaimResources.
    pub fn DidReceiveFrameAck(&mut self) -> Result<CompositorOutput, CompositorError> {
        self.assembly.DidReceiveFrameAck()
    }
    /// Actual Viz presentation feedback; neither submit nor ack synthesizes it.
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

#[cfg(test)]
mod public_boundary_tests {
    use super::*;

    #[test]
    fn compositor_engine_command_effect_boundary_signatures_stay_stable() {
        let _: fn(
            CompositorConfig,
            CompositorExecutors,
        ) -> Result<CompositorEngine, CompositorError> = CompositorEngine::new;
        let _: fn(
            &mut CompositorEngine,
            Arc<PaintArtifact>,
            CompositorViewportState,
        ) -> Result<CompositorOutput, CompositorError> = CompositorEngine::Commit;
        let _: fn(
            &mut CompositorEngine,
            crate::runtime::types::SurfaceRange,
        ) -> Arc<dyn paint::ForeignLayer> = CompositorEngine::CreateVideoSurfaceForeignLayer;
        let _: fn(&mut CompositorEngine) -> Result<CompositorOutput, CompositorError> =
            CompositorEngine::RequestMainFrame;
        let _: fn(&mut CompositorEngine) -> Result<CompositorOutput, CompositorError> =
            CompositorEngine::RequestRedraw;
        let _: fn(&mut CompositorEngine) -> Result<CompositorOutput, CompositorError> =
            CompositorEngine::FinishMainFrameWithoutUpdates;
        let _: fn(
            &mut CompositorEngine,
            BeginFrameArgs,
        ) -> Result<CompositorOutput, CompositorError> = CompositorEngine::BeginFrame;
        let _: fn(
            &mut CompositorEngine,
            CompositorInput,
        ) -> Result<CompositorOutput, CompositorError> = CompositorEngine::Input;
        let _: fn(&mut CompositorEngine) -> Result<CompositorOutput, CompositorError> =
            CompositorEngine::DidReceiveFrameAck;
        let _: fn(
            &mut CompositorEngine,
            u32,
            CompositorFrameTimingDetails,
        ) -> Result<CompositorOutput, CompositorError> = CompositorEngine::DidPresentFrame;
        let _: fn(
            &mut CompositorEngine,
            Vec<RasterReturnedResource>,
        ) -> Result<CompositorOutput, CompositorError> = CompositorEngine::ReclaimResources;
        let _: fn(&mut CompositorEngine) -> Result<CompositorOutput, CompositorError> =
            CompositorEngine::Poll;

        fn frame_boundary(output: CompositorOutput) -> Option<CompositorFrameSubmission> {
            output.frame
        }
        let _: fn(CompositorOutput) -> Option<CompositorFrameSubmission> = frame_boundary;
    }
}
