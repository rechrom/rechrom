//! Injected raster execution and resource ownership. This interface contains
//! no tile selection, task dependency/cancellation algorithm or Page state.
//! TaskGraphRunner implementations execute the canonical graph/queue protocol;
//! buffer implementations replay the canonical RasterSource selected by cc.
pub use crate::cc::raster::{
    task::{TaskGraph, TaskRef, TaskVector},
    task_graph_runner::{NamespaceToken, TaskGraphRunner as RasterTaskRunner},
};
// cc::RasterBuffer / RasterBufferProvider are the cc/raster interfaces the
// injected raster service implements (cc_raster).
pub use cc_raster::raster_buffer_provider::{
    RasterBuffer, RasterBufferProviderBackend, RasterPlayback,
};
// The display read lease of an exported resource is a viz value
// (viz_common::raster_resource_lease).
pub use viz_common::raster_resource_lease::RasterResourceLease;
/// Exact move-only origin-thread task results. Values are produced by native
/// TileTask completion and consumed by its native TileManager owner; the
/// envelope makes no activation/readiness decisions.
pub struct RasterReady<C> {
    pub completions: Vec<C>,
}

/// Native default runner; caller explicitly drives RunUntilIdle.
pub use crate::cc::raster::synchronous_task_graph_runner::SynchronousTaskGraphRunner as SynchronousRasterTaskRunner;
