//! Window-facing CPU layer/tile renderer. Supported input is rasterized into
//! persistent tile resources and composed directly into the presentation target.
//! Unsupported input is reported explicitly; there is no legacy execution path.
use std::io;

use layer_tile::FramePlan;
use skia::src::core::SkCanvas::{RasterImageCache, RasterImageCacheStats};

use crate::layer_raster::LayerRaster;
pub use crate::layer_raster::LayerTileRasterStats;

#[derive(Clone, Copy, Debug)]
pub struct RasterUpdate {
    pub mode: &'static str,
    pub reason: &'static str,
    pub damage_pixels: usize,
    pub copied_bytes: usize,
}

#[derive(Default)]
pub struct LayerTileRenderer {
    layers: LayerRaster,
    image_cache: RasterImageCache,
    clip_cache: skia::RasterClipProductCache,
}

impl LayerTileRenderer {
    pub fn layer_tile_stats(&self) -> LayerTileRasterStats {
        self.layers.stats()
    }

    pub fn image_cache_stats(&self) -> RasterImageCacheStats {
        self.image_cache.stats()
    }

    pub fn image_cache_resident_pixel_bytes(&self) -> usize {
        self.image_cache.resident_pixel_bytes()
    }

    pub fn clip_product_cache_stats(&self) -> skia::RasterClipProductCacheStats {
        self.clip_cache.stats()
    }

    pub fn clip_product_cache_accounting(&self) -> skia::RasterClipProductCacheAccounting {
        self.clip_cache.accounting()
    }

    pub fn with_clip_product_cache_byte_limit(limit: usize) -> Self {
        Self {
            clip_cache: skia::RasterClipProductCache::with_byte_limit(limit),
            ..Self::default()
        }
    }

    pub fn invalidate(&mut self) {
        self.layers.invalidate();
    }

    pub fn paint(
        &mut self,
        plan: &FramePlan,
        width: u32,
        height: u32,
        buffer: &mut [u32],
        format: skia::PixelFormat,
    ) -> io::Result<RasterUpdate> {
        self.paint_with_stride(plan, width, height, buffer, format, width as usize)
    }

    pub fn paint_with_stride(
        &mut self,
        plan: &FramePlan,
        width: u32,
        height: u32,
        buffer: &mut [u32],
        format: skia::PixelFormat,
        row_stride: usize,
    ) -> io::Result<RasterUpdate> {
        self.layers.paint(
            plan,
            width,
            height,
            buffer,
            format,
            row_stride,
            &mut self.image_cache,
            &mut self.clip_cache,
        )
    }

    /// Rasterize dirty tile resources without touching a presentation target.
    /// Layer/tile planning remains owned by `layer_tile`; this renderer keeps
    /// all resident pixel resources private across the later compose call.
    pub fn prepare(&mut self, plan: &FramePlan, width: u32, height: u32) -> io::Result<()> {
        self.layers.prepare(
            plan,
            width,
            height,
            &mut self.image_cache,
            &mut self.clip_cache,
        )
    }

    /// Composite an already prepared plan into the borrowed output target.
    pub fn compose_with_stride(
        &mut self,
        plan: &FramePlan,
        width: u32,
        height: u32,
        buffer: &mut [u32],
        format: skia::PixelFormat,
        row_stride: usize,
    ) -> io::Result<RasterUpdate> {
        self.layers.compose(
            plan,
            width,
            height,
            buffer,
            format,
            row_stride,
            &mut self.image_cache,
            &mut self.clip_cache,
        )
    }
}
