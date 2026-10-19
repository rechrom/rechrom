// Translation status: complete
// Copyright 2019 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/tile_size_calculator.h:1-54
// Source: cc/layers/tile_size_calculator.cc:1-269

use crate::adapter::geometry::Size;
use crate::cc::base::math_util::MathUtil;

// Source: cc/layers/tile_size_calculator.cc:17-29
const kTileRoundUp: i32 = 64;
const kGpuDefaultTileRoundUp: i32 = 32;
const kTileMinimalAlignment: i32 = 4;
const kBorderTexels: i32 = 1;

// Typed live view of the PictureLayerImpl/LayerTreeImpl values read by the
// source. It avoids storing a child-to-owner pointer while preserving the same
// per-call reads and cache key.
pub trait TileSizeCalculatorContext {
    fn IsBackdropFilterMask(&self) -> bool;
    fn MaxTextureSize(&self) -> i32;
    fn UseGpuRasterization(&self) -> bool;
    fn MaxGpuRasterTileSize(&self) -> Size;
    fn MinHeightForGpuRasterTile(&self) -> i32;
    fn GpuRasterMaxTextureSize(&self) -> Size;
    fn DeviceScaleFactor(&self) -> f32;
    fn MaxUntiledLayerSize(&self) -> Size;
    fn DefaultTileSize(&self) -> Size;
}

// Source: cc/layers/tile_size_calculator.cc:36-43
fn ScaleToCeiledSize(size: Size, scale: f32) -> Size {
    Size::new(
        (size.width() as f32 * scale).ceil() as i32,
        (size.height() as f32 * scale).ceil() as i32,
    )
}

fn ApplyDsfAdjustment(device_pixels_size: Size, dsf: f32) -> Size {
    let content_size_in_dips = ScaleToCeiledSize(device_pixels_size, 1.0 / dsf);
    ScaleToCeiledSize(content_size_in_dips, dsf)
}

// Source: cc/layers/tile_size_calculator.cc:45-70
fn AdjustGpuTileSize(
    mut tile_width: i32,
    mut tile_height: i32,
    max_tile_size: Size,
    min_height_for_gpu_raster_tile: i32,
) -> Size {
    tile_width += 2 * kBorderTexels;
    tile_height += 2 * kBorderTexels;
    tile_width = MathUtil::UncheckedRoundUp(tile_width, kGpuDefaultTileRoundUp);
    tile_height = MathUtil::UncheckedRoundUp(tile_height, kGpuDefaultTileRoundUp);
    tile_height = tile_height.max(min_height_for_gpu_raster_tile);
    if max_tile_size.width() != 0 && max_tile_size.height() != 0 {
        tile_width = tile_width.min(max_tile_size.width());
        tile_height = tile_height.min(max_tile_size.height());
    }
    Size::new(tile_width, tile_height)
}

// Source: cc/layers/tile_size_calculator.cc:72-94
fn CalculateGpuTileSize(
    base_tile_size: Size,
    content_bounds: Size,
    max_tile_size: Size,
    min_height_for_gpu_raster_tile: i32,
) -> Size {
    let tile_width = base_tile_size.width();
    let mut divisor = 4;
    if content_bounds.width() <= base_tile_size.width() / 2 {
        divisor = 2;
    }
    if content_bounds.width() <= base_tile_size.width() / 4 {
        divisor = 1;
    }
    let tile_height = MathUtil::UncheckedRoundUp(base_tile_size.height(), divisor) / divisor;
    AdjustGpuTileSize(
        tile_width,
        tile_height,
        max_tile_size,
        min_height_for_gpu_raster_tile,
    )
}

// Source: cc/layers/tile_size_calculator.cc:96-131
fn CalculateGpuRawDrawTileSize(
    base_tile_size: Size,
    content_bounds: Size,
    max_tile_size: Size,
    min_height_for_gpu_raster_tile: i32,
    raw_draw_tile_size_factor: f64,
) -> Size {
    const kMinBaseTileSize: i32 = 2280;
    let mut tile_size = base_tile_size
        .width()
        .max(base_tile_size.height())
        .max(kMinBaseTileSize);
    tile_size = (tile_size as f64 * raw_draw_tile_size_factor).ceil() as i32;
    let content_area = content_bounds.width() as u64 * content_bounds.height() as u64;
    if content_area <= tile_size as u64 * tile_size as u64 {
        return AdjustGpuTileSize(
            content_bounds.width(),
            content_bounds.height(),
            max_tile_size,
            min_height_for_gpu_raster_tile,
        );
    }
    AdjustGpuTileSize(
        tile_size.min(content_bounds.width()),
        tile_size.min(content_bounds.height()),
        max_tile_size,
        min_height_for_gpu_raster_tile,
    )
}

// Source: cc/layers/tile_size_calculator.h:24-35
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct AffectingParams {
    max_texture_size: i32,
    use_gpu_rasterization: bool,
    device_scale_factor: f32,
    max_tile_size: Size,
    min_height_for_gpu_raster_tile: i32,
    gpu_raster_max_texture_size: Size,
    max_untiled_layer_size: Size,
    default_tile_size: Size,
    content_bounds: Size,
}

// Source: cc/layers/tile_size_calculator.h:15-51
pub struct TileSizeCalculator {
    is_using_raw_draw_: bool,
    raw_draw_tile_size_factor_: f64,
    affecting_params_: AffectingParams,
    tile_size_: Size,
}

impl TileSizeCalculator {
    // Source: cc/layers/tile_size_calculator.h:19
    // Source: cc/layers/tile_size_calculator.cc:140-143
    pub fn new(is_using_raw_draw: bool, raw_draw_tile_size_factor: f64) -> Self {
        Self {
            is_using_raw_draw_: is_using_raw_draw,
            raw_draw_tile_size_factor_: raw_draw_tile_size_factor,
            affecting_params_: AffectingParams::default(),
            tile_size_: Size::default(),
        }
    }

    // Source: cc/layers/tile_size_calculator.cc:145-153
    fn UpdateAffectingParams(
        &mut self,
        content_bounds: Size,
        context: &impl TileSizeCalculatorContext,
    ) -> bool {
        let new_params = Self::GetAffectingParams(content_bounds, context);
        if self.affecting_params_ == new_params {
            return false;
        }
        self.affecting_params_ = new_params;
        true
    }

    // Source: cc/layers/tile_size_calculator.cc:155-174
    fn GetAffectingParams(
        content_bounds: Size,
        context: &impl TileSizeCalculatorContext,
    ) -> AffectingParams {
        AffectingParams {
            max_texture_size: context.MaxTextureSize(),
            use_gpu_rasterization: context.UseGpuRasterization(),
            max_tile_size: context.MaxGpuRasterTileSize(),
            min_height_for_gpu_raster_tile: context.MinHeightForGpuRasterTile(),
            gpu_raster_max_texture_size: context.GpuRasterMaxTextureSize(),
            device_scale_factor: context.DeviceScaleFactor(),
            max_untiled_layer_size: context.MaxUntiledLayerSize(),
            default_tile_size: context.DefaultTileSize(),
            content_bounds,
        }
    }

    // Source: cc/layers/tile_size_calculator.h:21
    // Source: cc/layers/tile_size_calculator.cc:176-267
    pub fn CalculateTileSize(
        &mut self,
        content_bounds: Size,
        context: &impl TileSizeCalculatorContext,
    ) -> Size {
        if context.IsBackdropFilterMask() {
            debug_assert!(content_bounds.width() <= context.MaxTextureSize());
            debug_assert!(content_bounds.height() <= context.MaxTextureSize());
            return content_bounds;
        }
        if !self.UpdateAffectingParams(content_bounds, context) {
            return self.tile_size_;
        }
        let mut default_tile_width;
        let mut default_tile_height;
        if self.affecting_params_.use_gpu_rasterization {
            let max_tile_size = self.affecting_params_.max_tile_size;
            let mut base_tile_size = ApplyDsfAdjustment(
                self.affecting_params_.gpu_raster_max_texture_size,
                self.affecting_params_.device_scale_factor,
            );
            let mut default_tile_size;
            if self.is_using_raw_draw_ {
                default_tile_size = CalculateGpuRawDrawTileSize(
                    base_tile_size,
                    content_bounds,
                    max_tile_size,
                    self.affecting_params_.min_height_for_gpu_raster_tile,
                    self.raw_draw_tile_size_factor_,
                );
            } else {
                default_tile_size = CalculateGpuTileSize(
                    base_tile_size,
                    content_bounds,
                    max_tile_size,
                    self.affecting_params_.min_height_for_gpu_raster_tile,
                );
                if content_bounds.width() > default_tile_size.width() {
                    base_tile_size =
                        Size::new((base_tile_size.width() + 1) / 2, base_tile_size.height());
                    default_tile_size = CalculateGpuTileSize(
                        base_tile_size,
                        content_bounds,
                        max_tile_size,
                        self.affecting_params_.min_height_for_gpu_raster_tile,
                    );
                }
            }
            default_tile_width = default_tile_size.width();
            default_tile_height = default_tile_size.height();
        } else {
            let max_untiled_content_width = self.affecting_params_.max_untiled_layer_size.width();
            let max_untiled_content_height = self.affecting_params_.max_untiled_layer_size.height();
            default_tile_width = self.affecting_params_.default_tile_size.width();
            default_tile_height = self.affecting_params_.default_tile_size.height();
            if content_bounds.width() < default_tile_width {
                default_tile_height = max_untiled_content_height;
            }
            if content_bounds.height() < default_tile_height {
                default_tile_width = max_untiled_content_width;
            }
            if content_bounds.width() < max_untiled_content_width
                && content_bounds.height() < max_untiled_content_height
            {
                default_tile_height = max_untiled_content_height;
                default_tile_width = max_untiled_content_width;
            }
        }
        let mut tile_width = default_tile_width;
        let mut tile_height = default_tile_height;
        if content_bounds.width() < default_tile_width {
            tile_width = MathUtil::UncheckedRoundUp(content_bounds.width(), kTileRoundUp);
            tile_width = tile_width.min(default_tile_width);
        }
        if content_bounds.height() < default_tile_height {
            tile_height = MathUtil::UncheckedRoundUp(content_bounds.height(), kTileRoundUp);
            tile_height = tile_height.min(default_tile_height);
        }
        tile_width = MathUtil::UncheckedRoundUp(tile_width, kTileMinimalAlignment);
        tile_height = MathUtil::UncheckedRoundUp(tile_height, kTileMinimalAlignment);
        tile_width = tile_width.min(self.affecting_params_.max_texture_size);
        tile_height = tile_height.min(self.affecting_params_.max_texture_size);
        self.tile_size_ = Size::new(tile_width, tile_height);
        self.tile_size_
    }
}

// Source: cc/layers/tile_size_calculator.h:5-14,22-23,37-45,53-54;
// Source: cc/layers/tile_size_calculator.cc:5-16,133-138,269.
// Includes, namespaces, export decoration, raw-pointer storage, and feature
// lookup map to Rust modules and the explicit live context/constructor values.
