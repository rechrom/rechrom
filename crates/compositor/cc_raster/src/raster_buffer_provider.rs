//! cc::RasterBuffer and cc::RasterBufferProvider (cc/raster/raster_buffer.h,
//! cc/raster/raster_buffer_provider.h): the injected raster execution and
//! resource interface. It contains no tile selection, task dependency or
//! cancellation algorithm and no Page state. TaskGraphRunner implementations
//! execute the canonical graph/queue protocol; buffer implementations replay
//! the canonical RasterSource selected by cc.
use crate::raster_source::RasterSourceStorage;
use cc_paint::skia_region::SkRegion;
use cc_paint::RasterImageProviderRef;
use compositor_tree::geometry::{AxisTransform2d, Rect, Size};
use compositor_values::axis_transform2d::RasterTransform;
use compositor_values::color_space::RasterColorSpace;
use compositor_values::prioritized_tile::ScrollOffsetMap;
use compositor_values::shared_image_format::SharedImageFormat;
use std::io;
use viz_common::shared_image::RasterSharedImage;
use viz_common::ResourceId;

/// The source worker borrows the immutable raster source, resolved transform,
/// image provider and scroll-offset map only for this playback invocation.
/// Neither this envelope nor the backend may retain a compositor owner/context.
// Source: cc/raster/raster_buffer.h RasterBuffer::Playback arguments.
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

/// The tile raster buffer of one Skia value family.
pub type CpuTileRasterBufferValueFor<R> = dyn for<'a> RasterBuffer<
    RasterSourceStorage<R>,
    RasterTransform,
    RasterImageProviderRef<'a, R>,
    ScrollOffsetMap,
>;
/// The production compositor has one feature-selected Skia value family. Keep
/// that selection behind the injected pixel boundary; generic cc machinery
/// continues to use `CpuTileRasterBufferValueFor<R>` internally.
pub type TileRasterBufferValue = CpuTileRasterBufferValueFor<SkRegion>;

/// The compositor's injected pixel capability (the production
/// cc::RasterBufferProvider together with the shared-image allocation the
/// ResourcePool asks for).
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
