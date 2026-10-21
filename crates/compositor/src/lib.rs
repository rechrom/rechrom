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
pub mod third_party;
pub mod ui;

pub mod runtime;

/// The default software compositor family used by the Page assembly. These
/// aliases expose only the immutable native frame/resource output; translated
/// layer, tiling, scheduler and host owners remain private to CompositorEngine.
pub type NativeCompositorTypes = runtime::types::CpuCompositorTypes<runtime::skia::SkRegion>;
pub type NativeCompositorFrame = runtime::frame::CompositorFrame<
    runtime::metadata::CompositorFrameMetadata<NativeCompositorTypes>,
    runtime::frame::CompositorRenderPass<NativeCompositorTypes>,
    runtime::resources::RasterTransferableResource,
>;
pub type NativeCompositorRenderPass = runtime::frame::CompositorRenderPass<NativeCompositorTypes>;
pub type NativeCompositorFrameMetadata =
    runtime::metadata::CompositorFrameMetadata<NativeCompositorTypes>;
pub type NativeTransferableResource = runtime::resources::RasterTransferableResource;
pub type NativeRasterResourceLease = dyn runtime::raster_execution::RasterResourceLease;
/// The value submitted through LayerTreeFrameSink. Chromium carries the
/// LocalSurfaceId beside each CompositorFrame; keeping that pair intact lets
/// Viz reject an older resize generation instead of guessing from dimensions.
pub struct NativeCompositorFrameSubmission {
    pub local_surface_id: runtime::types::LocalSurfaceId,
    pub frame: NativeCompositorFrame,
}

pub mod engine;
pub use engine::{
    CompositorConfig, CompositorEngine, CompositorError, CompositorExecutor, CompositorExecutors,
    CompositorInput, CompositorMainThreadEffect, CompositorOutput, CompositorPaintInputKind,
    CompositorRasterProvider, CompositorViewportState, ScrollPhase, ScrollSourceType,
    TargetSnapAreaElementIds,
};
