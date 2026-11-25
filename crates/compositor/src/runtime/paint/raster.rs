//! Independent CPU raster execution/image capabilities. These wrappers only
//! translate the typed invocation envelope; TileManager remains the sole owner
//! of tile policy, cancellation, resource accounting and completion ordering.
use super::*;
use crate::{
    adapter::{
        prioritized_tile::ScrollOffsetMap,
        tile_manager::{
            RasterImageProvider, RasterPlaybackRequest, TileManagerRasterBuffer,
            WorkletImageProvider,
        },
    },
    runtime::{
        raster_execution::{RasterBuffer, RasterPlayback},
        types::{CpuCompositorTypes, CpuSkiaBackend},
    },
};
use std::{io, sync::Arc};
pub trait RasterFailureHandler: Send + Sync {
    fn OnRasterFailure(&self, error: io::Error);
}
// The tile raster buffer values of the cc/raster interface (cc_raster).
pub use cc_raster::raster_buffer_provider::{CpuTileRasterBufferValueFor, TileRasterBufferValue};
pub struct TileRasterBuffer<R: CpuSkiaBackend> {
    external: Box<CpuTileRasterBufferValueFor<R>>,
    failure: Arc<dyn RasterFailureHandler>,
}
impl<R: CpuSkiaBackend> TileRasterBuffer<R> {
    pub(crate) fn FromBoxed(
        external: Box<CpuTileRasterBufferValueFor<R>>,
        failure: Arc<dyn RasterFailureHandler>,
    ) -> Self {
        Self { external, failure }
    }

    pub fn new<T>(external: T, failure: Arc<dyn RasterFailureHandler>) -> Self
    where
        T: for<'a> RasterBuffer<
                RasterSourceStorage<R>,
                RasterTransform,
                RasterImageProviderRef<'a, R>,
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
struct ImageProvider<'a, R: CpuSkiaBackend>(&'a mut dyn RasterImageProvider<CpuCompositorTypes<R>>);
impl<R: CpuSkiaBackend> CpuImageProvider<R> for ImageProvider<'_, R> {
    fn GetRasterContent(&mut self, image: &CpuDrawImage) -> CpuScopedRasterResult<R> {
        self.0.GetRasterContent(image)
    }
}
impl<R: CpuSkiaBackend> TileManagerRasterBuffer<CpuCompositorTypes<R>> for TileRasterBuffer<R> {
    fn SupportsBackgroundThreadPriority(&self) -> bool {
        self.external.SupportsBackgroundThreadPriority()
    }
    fn Playback(&mut self, request: RasterPlaybackRequest<'_, CpuCompositorTypes<R>>) {
        let mut image_provider = ImageProvider(request.settings.image_provider);
        let mut provider = RasterImageProviderRef(&mut image_provider);
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
