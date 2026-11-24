// Translation status: complete
// Copyright 2011 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/tile_manager_settings.h:1-21.
#[derive(Clone, Copy, Debug)]
pub struct TileManagerSettings {
    pub use_partial_raster: bool,
    pub enable_checker_imaging: bool,
    pub min_image_bytes_to_checker: usize,
    pub needs_notify_ready_to_draw: bool,
}
// Source: cc/tiles/tile_manager_settings.h:12-16.
impl Default for TileManagerSettings {
    fn default() -> Self {
        Self {
            use_partial_raster: false,
            enable_checker_imaging: false,
            min_image_bytes_to_checker: 1024 * 1024,
            needs_notify_ready_to_draw: true,
        }
    }
}
