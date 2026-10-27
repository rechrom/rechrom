// Translation status: complete
// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/tile_manager_client.h:1-102.

use crate::TreePriority;
use compositor_tree::WhichTree;

/// The platform-neutral client contract used by `TileManager`.
///
/// Chromium names concrete paint and Viz types here. Rechrom keeps those
/// owners outside this package and preserves the same calls with associated
/// types, so the translated tile manager never acquires a platform dependency.
// Source: cc/tiles/tile_manager_client.h:19-27.
pub trait TileManagerClient {
    type Tile;
    type RasterQueue: ?Sized;
    type RasterQueueType;
    type EvictionQueue: ?Sized;
    type TilesWithResourceIterator: ?Sized;
    type TargetColorParams;
    type ContentColorUsage;
    type SharedImageFormat;
    type PaintImage;
    type DisplayItemList;

    // Source: cc/tiles/tile_manager_client.h:28-37.
    fn NotifyReadyToActivate(&mut self);
    fn NotifyReadyToDraw(&mut self);
    fn NotifyAllTileTasksCompleted(&mut self);

    // Source: cc/tiles/tile_manager_client.h:39-50.
    fn NotifyTileStateChanged(
        &mut self,
        tile: &Self::Tile,
        update_damage: bool,
        set_needs_redraw: bool,
    );
    fn SetNeedsRedraw(&mut self, animation_only: bool, skip_if_inside_draw: bool);

    // Source: cc/tiles/tile_manager_client.h:52-66.
    fn BuildRasterQueue(
        &mut self,
        tree_priority: TreePriority,
        queue_type: Self::RasterQueueType,
    ) -> Box<Self::RasterQueue>;
    fn BuildEvictionQueue(&mut self) -> Box<Self::EvictionQueue>;
    fn CreateTilesWithResourceIterator(&mut self) -> Box<Self::TilesWithResourceIterator>;

    // Source: cc/tiles/tile_manager_client.h:68-78.
    fn SetIsLikelyToRequireADraw(&mut self, is_likely_to_require_a_draw: bool);
    fn GetTargetColorParams(
        &self,
        content_color_usage: Self::ContentColorUsage,
    ) -> Self::TargetColorParams;
    fn GetTileFormat(&self) -> Self::SharedImageFormat;

    // Source: cc/tiles/tile_manager_client.h:80-94.
    fn RequestImplSideInvalidationForCheckerImagedTiles(&mut self);
    fn GetFrameIndexForImage(&self, paint_image: &Self::PaintImage, tree: WhichTree) -> usize;
    fn GetMSAASampleCountForRaster(&self, display_list: &Self::DisplayItemList) -> i32;
    fn HasPendingTree(&mut self) -> bool;
}

// Source: cc/tiles/tile_manager_client.h:96-102. Rust trait-object drop and
// module visibility replace the protected virtual destructor and namespace
// closure.
