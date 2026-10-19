// Translation status: complete
// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/tiling_internal.h:1-16,39-41.
// Rust modules and trait bounds replace includes, namespaces and C++ concepts.
use super::tile_index::TileIndex;
use crate::adapter::geometry::{AxisTransform2d, Size};
use crate::cc::base::tiling_data::TilingData;

// Source: cc/tiles/tiling_internal.h:17-22.
// The source's const-expression query borrows the actual tile.
pub trait TileLike {
    fn IsReadyToDraw(&self) -> bool;
}

// Source: cc/tiles/tiling_internal.h:24-37.
// The source nullable Tile* is a borrowed optional resident tile. The owner
// retains the allocation; no tile, TilingData or transform is copied here.
pub trait Tiling {
    type Tile: TileLike;
    type RasterTransform: AxisTransform2d;
    fn TileAt(&self, index: &TileIndex) -> Option<&Self::Tile>;
    fn contents_scale_key(&self) -> f32;
    fn tiling_data(&self) -> &TilingData;
    fn raster_size(&self) -> Size;
    fn raster_transform(&self) -> &Self::RasterTransform;
}
