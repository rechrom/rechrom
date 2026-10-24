//! Page-local compositor command/effect boundary.
//!
//! The assembly owns PAC, main/impl trees, scheduling and tile resources. The
//! embedding supplies execution placement and actual raster storage/replay;
//! neither capability receives a Page, compositor owner or retained tree.
pub use crate::cc::{
    input::scroll_snap_data::TargetSnapAreaElementIds, trees::scroll_source_type::ScrollSourceType,
};
pub use crate::runtime::host_metrics::CustomTrackerResults;
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
        resources::{RasterReturnedResource, RasterSharedImage},
    },
    NativeCompositorFrameSubmission,
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
    pub ui_raster: Arc<dyn crate::runtime::native_ui::NativeUiRasterServices>,
}

#[derive(Clone, Debug)]
pub struct CompositorConfig {
    pub thread: bool,
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
#[derive(Clone, Copy, Debug)]
pub enum CompositorInput {
    Wheel {
        position: PointF,
        delta: Vector2dF,
        unconstrained_delta: Vector2dF,
        granularity: crate::adapter::input_types::ScrollGranularity,
        is_inertial: bool,
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

#[derive(Clone, Debug)]
pub enum CompositorMainThreadEffect {
    LayerTreeFrameSinkLost,
    PageScaleAnimationCompleted,
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
    pub frame: Option<NativeCompositorFrameSubmission>,
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
            E::UnsupportedCanvasChild { .. } => CompositorPaintInputKind::UnsupportedCanvasChild,
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
impl From<crate::runtime::native_pac_scroll::NativePacScrollEffect> for CompositorMainThreadEffect {
    fn from(effect: crate::runtime::native_pac_scroll::NativePacScrollEffect) -> Self {
        use crate::runtime::native_pac_scroll::NativePacScrollEffect as E;
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
            E::DidChangeScrollbarsHidden { element, hidden } => {
                Self::ScrollbarsHiddenChanged { element, hidden }
            }
        }
    }
}

/// Public commands intentionally carry no layer, tiling, job or graph owner.
/// Construction remains explicit until the real canonical assembly is ready;
/// no command reports success with a fabricated frame or skipped commit.
pub struct CompositorEngine {
    assembly: crate::runtime::engine_assembly::NativeEngineAssembly,
}
impl CompositorEngine {
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
    /// Mirrors LayerTreeHost/ProxyMain commit demand before the main-frame
    /// lifecycle has produced its PaintArtifact.
    pub fn RequestMainFrame(&mut self) -> Result<CompositorOutput, CompositorError> {
        self.assembly.RequestMainFrame()
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
