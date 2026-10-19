// Translation status: complete
// Copyright 2023 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/raster_capabilities.h:1-39
// Source: cc/trees/raster_capabilities.cc:1-19
use crate::adapter::resource_types::{SharedImageFormat, SinglePlaneFormat};

// Source: cc/trees/raster_capabilities.h:13-35
// Source: cc/trees/raster_capabilities.cc:11-17
// Rust Clone/Copy and the implicit destructor implement the defaulted special
// members without changing ownership of the format values.
#[derive(Clone, Copy, Debug)]
pub struct RasterCapabilities {
    // Source: cc/trees/raster_capabilities.h:19
    pub use_gpu_rasterization: bool,
    // Source: cc/trees/raster_capabilities.h:21
    pub can_use_msaa: bool,
    // Source: cc/trees/raster_capabilities.h:23-25
    pub max_texture_size: i32,
    // Source: cc/trees/raster_capabilities.h:27-28
    pub tile_format: SharedImageFormat,
    // Source: cc/trees/raster_capabilities.h:30-31
    pub tile_overlay_candidate: bool,
    // Source: cc/trees/raster_capabilities.h:33-34
    pub ui_rgba_format: SharedImageFormat,
}

impl Default for RasterCapabilities {
    // Source: cc/trees/raster_capabilities.h:19-34
    // Source: cc/trees/raster_capabilities.cc:9
    fn default() -> Self {
        Self {
            use_gpu_rasterization: false,
            can_use_msaa: false,
            max_texture_size: 0,
            tile_format: SinglePlaneFormat::kRGBA_8888,
            tile_overlay_candidate: false,
            ui_rgba_format: SinglePlaneFormat::kRGBA_8888,
        }
    }
}
