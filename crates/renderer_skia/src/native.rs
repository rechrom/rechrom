//! CPU target and Paint playback capabilities, confined to the Skia backend.
use compositor_core::adapter::geometry::Rect;
use std::io;

/// The actual immutable display-list owner carried by the producer's frame.
pub type PictureDisplayItemList=<compositor_core::NativeCompositorTypes as compositor_core::runtime::frame::FrameValueTypes>::PictureDisplayItemList;
pub struct PictureRasterRequest<'a> {
    pub display_list: &'a PictureDisplayItemList,
    pub content_rect: Rect,
    pub contents_scale: f32,
    pub nearest_neighbor: bool,
    pub image_animation_map: &'a std::collections::BTreeMap<
        compositor_core::adapter::image_animation::PaintImageId,
        usize,
    >,
    pub raster_inducing_scroll_offsets:
        &'a compositor_core::adapter::prioritized_tile::ScrollOffsetMap,
}
/// External CPU Paint playback only. Implementations execute the source
/// SoftwareRenderer::DrawPictureQuad (cc list Raster) on the active canvas.
/// This capability cannot access a compositor tree, tile or scheduler owner.
pub trait PictureRasterizer: Send {
    fn RasterizeInto(
        &mut self,
        request: PictureRasterRequest<'_>,
        canvas: skia::cpu::canvas::Canvas,
        opacity: f32,
    ) -> io::Result<skia::cpu::canvas::Canvas>;
}

#[cfg(feature = "pure_replay")]
impl PictureRasterizer for raster::native::CpuRasterProvider {
    fn RasterizeInto(
        &mut self,
        request: PictureRasterRequest<'_>,
        canvas: skia::cpu::canvas::Canvas,
        opacity: f32,
    ) -> io::Result<skia::cpu::canvas::Canvas> {
        self.RasterizePictureInto(
            request.display_list,
            request.content_rect,
            request.contents_scale,
            request.nearest_neighbor,
            request.image_animation_map,
            request.raster_inducing_scroll_offsets,
            opacity,
            canvas,
        )
    }
}

/// Borrowed host output; no reference escapes Render.
pub struct RenderTarget<'a> {
    pub width: u32,
    pub height: u32,
    pub row_stride: usize,
    pub format: skia::PixelFormat,
    pub pixels: &'a mut [u32],
    /// The target still contains the pixels from this Renderer instance's
    /// previous completed aggregate at the same size and format.
    pub retained: bool,
}
