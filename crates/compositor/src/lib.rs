#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    unused_assignments,
    unused_mut,
    unused_parens
)]

//! Source-aligned Chromium impl-compositor core.
//!
//! The public `CompositorEngine` adapter will preserve Rechrom's Page-domain
//! command/effect boundary. Source translations live under source-relative `cc/`, `third_party/`, and `ui/`
//! and contain no window, graphics-context, platform-surface, OS-thread, or
//! presentation implementation.

pub mod adapter;
pub mod cc;
pub mod ui;

pub mod performance;
pub mod runtime;

/// The default software compositor family used by the Page assembly. These
/// aliases expose only the immutable native frame/resource output; translated
/// layer, tiling, scheduler and host owners remain private to CompositorEngine.
pub type CompositorTypes = runtime::types::CpuCompositorTypes<runtime::skia::SkRegion>;
/// The frame values are components/viz/common (`viz_common`); the family's
/// passes and metadata are exactly viz_common's software frame instance.
pub type CompositorFrame = viz_common::CompositorFrame;
pub type CompositorRenderPass = viz_common::CompositorRenderPass;
pub type CompositorFrameMetadata = viz_common::CompositorFrameMetadata;
pub type TransferableResource = viz_common::TransferableResource;
pub type RasterResourceLease = dyn runtime::raster_execution::RasterResourceLease;
/// The value submitted through LayerTreeFrameSink. Chromium carries the
/// LocalSurfaceId beside each CompositorFrame; keeping that pair intact lets
/// Viz reject an older resize generation instead of guessing from dimensions.
pub type CompositorFrameSubmission =
    compositor_frame::CompositorFrameSubmission<runtime::types::LocalSurfaceId, CompositorFrame>;

pub mod engine;
pub use engine::{
    CommitDeferralReason, CompositorConfig, CompositorEngine, CompositorError, CompositorExecutor,

    CompositorServices, CompositorInput, CompositorInputEffect, CompositorEmbedderNotification, CompositorMainThreadEffect, CompositorMutation,
    CompositorOutput, CompositorPaintInputKind, CompositorRasterProvider, CompositorScrollState,
    CompositorViewportState, FrameLifecycleMutation, PaintHoldingReason,
    PresentationUnavailableReason, BeginMainFrameState, ImplToMainTask, MainFamily,
    MainAnimationHost, MainHostParams, MainPropertyTreeFeatures, MainToImplTask, ProxyImplInit,
    CreateMainHostParams,
    ScrollPhase, ScrollSourceType, TargetSnapAreaElementIds,
};
