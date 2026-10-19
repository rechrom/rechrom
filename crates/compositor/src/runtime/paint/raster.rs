//! Independent CPU raster execution/image capabilities. These wrappers only
//! translate the typed invocation envelope; TileManager remains the sole owner
//! of tile policy, cancellation, resource accounting and completion ordering.
use super::*;
use crate::{
    adapter::{
        geometry::{Rect, Size},
        prioritized_tile::ScrollOffsetMap,
        resource_types::SharedImageFormat,
        tile_manager::{
            RasterImageProvider, RasterPlaybackRequest, TileManagerRasterBuffer, TileManagerTypes,
            WorkletImageProvider,
        },
    },
    runtime::{
        cpu_raster::{CpuRasterBuffer, RasterPlayback},
        types::{CpuCompositorTypes, CpuSkiaBackend},
    },
};
use std::{io, sync::Arc};
pub trait CpuDecodedImageLease: Send {
    fn size(&self) -> Size;
    fn format(&self) -> &SharedImageFormat;
    fn row_bytes(&self) -> usize;
    fn pixels(&self) -> &[u8];
    // Source: cc/paint/decoded_draw_image.h:75-87. These belong to the actual
    // decoded allocation, not guesses from requested size or cache identity.
    fn scale_adjustment(&self) -> crate::adapter::geometry::SizeF;
    fn src_rect_offset(&self) -> crate::adapter::geometry::SizeF;
    fn filter_quality(&self) -> super::CpuFilterQuality;
}
pub enum CpuScopedRasterResult<R: CpuSkiaBackend> {
    Decoded(Box<dyn CpuDecodedImageLease>),
    Record(CpuPaintRecord<R>),
    Unavailable,
}
pub trait CpuImageProvider<R: CpuSkiaBackend> {
    fn GetRasterContent(&mut self, image: &CpuDrawImage) -> CpuScopedRasterResult<R>;
}
pub trait CpuWorkletImageProvider<R: CpuSkiaBackend> {
    fn GetPaintRecordResult(&mut self, input: &CpuWorkletInput) -> CpuScopedRasterResult<R>;
}
pub struct CpuPlaybackImageProvider<R: CpuSkiaBackend>(pub Box<dyn CpuImageProvider<R> + Send>);
pub struct CpuWorkletProvider<R: CpuSkiaBackend>(pub Box<dyn CpuWorkletImageProvider<R> + Send>);
pub struct CpuImageProviderRef<'a, R: CpuSkiaBackend>(pub &'a mut dyn CpuImageProvider<R>);
impl<R: CpuSkiaBackend> CpuImageProvider<R> for CpuImageProviderRef<'_, R> {
    fn GetRasterContent(&mut self, image: &CpuDrawImage) -> CpuScopedRasterResult<R> {
        self.0.GetRasterContent(image)
    }
}
pub trait CpuRasterFailureHandler: Send + Sync {
    fn OnRasterFailure(&self, error: io::Error);
}
pub type CpuTileRasterBufferValue<R> = dyn for<'a> CpuRasterBuffer<
    CpuRasterSource<R>,
    RasterTransform,
    CpuImageProviderRef<'a, R>,
    ScrollOffsetMap,
>;
pub struct CpuTileRasterBuffer<R: CpuSkiaBackend> {
    external: Box<CpuTileRasterBufferValue<R>>,
    failure: Arc<dyn CpuRasterFailureHandler>,
}
impl<R: CpuSkiaBackend> CpuTileRasterBuffer<R> {
    pub(crate) fn FromBoxed(
        external: Box<CpuTileRasterBufferValue<R>>,
        failure: Arc<dyn CpuRasterFailureHandler>,
    ) -> Self {
        Self { external, failure }
    }

    pub fn new<T>(external: T, failure: Arc<dyn CpuRasterFailureHandler>) -> Self
    where
        T: for<'a> CpuRasterBuffer<
                CpuRasterSource<R>,
                RasterTransform,
                CpuImageProviderRef<'a, R>,
                ScrollOffsetMap,
            > + 'static,
    {
        Self {
            external: Box::new(external),
            failure,
        }
    }
}
impl<R: CpuSkiaBackend> RasterImageProvider<CpuCompositorTypes<R>> for CpuPlaybackImageProvider<R> {
    fn GetRasterContent(&mut self, image: &CpuDrawImage) -> CpuScopedRasterResult<R> {
        self.0.GetRasterContent(image)
    }
}
impl<R: CpuSkiaBackend> WorkletImageProvider<CpuCompositorTypes<R>> for CpuWorkletProvider<R> {
    fn GetPaintRecordResult(&mut self, input: &CpuWorkletInput) -> CpuScopedRasterResult<R> {
        self.0.GetPaintRecordResult(input)
    }
}
struct NativeImageProvider<'a, R: CpuSkiaBackend>(
    &'a mut dyn RasterImageProvider<CpuCompositorTypes<R>>,
);
impl<R: CpuSkiaBackend> CpuImageProvider<R> for NativeImageProvider<'_, R> {
    fn GetRasterContent(&mut self, image: &CpuDrawImage) -> CpuScopedRasterResult<R> {
        self.0.GetRasterContent(image)
    }
}
impl<R: CpuSkiaBackend> TileManagerRasterBuffer<CpuCompositorTypes<R>> for CpuTileRasterBuffer<R> {
    fn SupportsBackgroundThreadPriority(&self) -> bool {
        self.external.SupportsBackgroundThreadPriority()
    }
    fn Playback(&mut self, request: RasterPlaybackRequest<'_, CpuCompositorTypes<R>>) {
        let mut native = NativeImageProvider(request.settings.image_provider);
        let mut provider = CpuImageProviderRef(&mut native);
        let result = self.external.Playback(RasterPlayback {
            source: request.source,
            content_rect: request.content_rect,
            invalid_content_rect: request.invalid_content_rect,
            new_content_id: request.new_content_id,
            raster_transform: request.raster_transform,
            use_lcd_text: request.settings.values.use_lcd_text,
            msaa_sample_count: request.settings.values.msaa_sample_count,
            visible: request.settings.values.visible,
            hdr_headroom: request.settings.values.hdr_headroom,
            image_provider: &mut provider,
            raster_inducing_scroll_offsets: request.settings.raster_inducing_scroll_offsets,
        });
        if let Err(error) = result {
            self.failure.OnRasterFailure(error);
        }
    }
}
