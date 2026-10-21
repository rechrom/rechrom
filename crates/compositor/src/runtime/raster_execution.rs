//! Injected raster execution and resource ownership. This interface contains
//! no tile selection, task dependency/cancellation algorithm or Page state.
//! TaskGraphRunner implementations execute the canonical graph/queue protocol;
//! buffer implementations replay the canonical RasterSource selected by cc.
use crate::adapter::{
    geometry::AxisTransform2d,
    geometry::{Rect, Size},
    layer_tree_impl::ResourceId,
    resource_types::SharedImageFormat,
};
pub use crate::cc::raster::{
    task::{TaskGraph, TaskRef, TaskVector},
    task_graph_runner::{NamespaceToken, TaskGraphRunner as RasterTaskRunner},
};
use std::{any::Any, io};
/// The source worker borrows the immutable raster source, resolved transform,
/// image provider and scroll-offset map only for this playback invocation.
/// Neither this envelope nor the backend may retain a compositor owner/context.
pub struct RasterPlayback<'a, S: ?Sized, T: AxisTransform2d, I: ?Sized, O: ?Sized> {
    pub source: &'a S,
    pub content_rect: &'a Rect,
    pub invalid_content_rect: &'a Rect,
    pub new_content_id: u64,
    pub raster_transform: &'a T,
    pub use_lcd_text: bool,
    pub msaa_sample_count: i32,
    pub visible: bool,
    pub hdr_headroom: f32,
    pub image_provider: &'a mut I,
    pub raster_inducing_scroll_offsets: &'a O,
}
pub trait RasterBuffer<S: ?Sized, T: AxisTransform2d, I: ?Sized, O: ?Sized>: Send {
    fn SupportsBackgroundThreadPriority(&self) -> bool;
    fn Playback(&mut self, request: RasterPlayback<'_, S, T, I, O>) -> io::Result<()>;
}
/// Actual external raster allocation/readiness primitives. Canonical
/// ResourcePool/TileManager retain lease identity, limits, selection and release
/// ordering; this provider owns only storage/replay caches for the requested ID.
pub trait RasterBufferProviderBackend<S: ?Sized, T: AxisTransform2d, I: ?Sized, O: ?Sized> {
    type Buffer: RasterBuffer<S, T, I, O>;
    fn AcquireBufferForRaster(
        &mut self,
        resource: ResourceId,
        size: Size,
        format: &SharedImageFormat,
        resource_content_id: u64,
        previous_content_id: u64,
    ) -> io::Result<Self::Buffer>;
    fn IsResourceReadyToDraw(&self, resource: ResourceId) -> bool;
    fn CanPartialRasterIntoProvidedResource(&self) -> bool;
    fn NotifyWorkSubmitted(&mut self);
    fn Shutdown(&mut self) -> io::Result<()>;
}
/// Opaque read lease to one immutable raster resource. The compositor and Viz
/// retain identity and lifetime only; a concrete renderer asks the raster
/// package to map the backend payload it understands.
pub trait RasterResourceLease: Send + Sync {
    fn resource_id(&self) -> ResourceId;
    fn size(&self) -> Size;
    fn format(&self) -> SharedImageFormat;
    fn is_opaque(&self) -> bool;
    fn backend_resource(&self) -> &(dyn Any + Send + Sync);
}
/// Exact move-only origin-thread task results. Values are produced by native
/// TileTask completion and consumed by its native TileManager owner; the
/// envelope makes no activation/readiness decisions.
pub struct RasterReady<C> {
    pub completions: Vec<C>,
}

/// Native default runner; caller explicitly drives RunUntilIdle.
pub use crate::cc::raster::synchronous_task_graph_runner::SynchronousTaskGraphRunner as SynchronousRasterTaskRunner;
