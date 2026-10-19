// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/picture_layer_tiling.h:1-77,80-84,92-96,263-275,444,453,462,468,474,485,493-495;
//         cc/tiles/picture_layer_tiling.cc:1-32,941.
use super::tile_priority::GlobalStateThatImpactsTilePriority;
use super::{
    prioritized_tile::PrioritizedTile,
    tile::{CreateInfo, Tile},
    tile_index::TileIndex,
    tile_priority::{PriorityBin, TileMemoryLimitPolicy, TilePriority, TileResolution, WhichTree},
    tiling_coverage_iterator::TilingCoverageIterator,
};
#[cfg(debug_assertions)]
use crate::adapter::geometry::SizeF;
use crate::{
    adapter::{
        geometry::{AxisTransform2d, Rect, RectF, Size, ToEnclosingRect, cpp_max},
        picture_layer_tiling::{
            PendingInvalidationId, PictureLayerTilingClientId, PictureLayerTilingContext,
            PictureLayerTilingQueries, PictureLayerTilingTypes, TilingRasterSource,
        },
        prioritized_tile::ScrollOffsetMap,
        property_types::PropertyTrace,
        tile::{PictureLayerTilingId, TileId},
    },
    cc::{
        base::{
            math_util::MathUtil,
            region::Region,
            simple_enclosed_region::SimpleEnclosedRegion,
            tiling_data::{DifferenceIterator, Iterator as DataIterator, TilingData},
        },
        trees::occlusion::Occlusion,
    },
};
/// Source: cc/tiles/picture_layer_tiling.h:47-69. The native client pointer
/// itself becomes ClientId; every virtual call borrows this actual live client.
pub trait PictureLayerTilingClient<B: PictureLayerTilingTypes> {
    fn GetPendingInvalidation(
        &self,
        queries: &dyn PictureLayerTilingQueries<B>,
    ) -> Option<PendingInvalidationId>;
    fn GetPendingOrActiveTwinTiling(
        &self,
        tiling: &PictureLayerTilingState<B>,
        queries: &dyn PictureLayerTilingQueries<B>,
        selected: Option<&PictureLayerTilingState<B>>,
    ) -> Option<PictureLayerTilingId>;
    fn HasValidTilePriorities(&self) -> bool;
    fn RequiresHighResToDraw(&self) -> bool;
    fn GetPaintWorkletRecords(&self) -> &B::PaintWorkletRecordMap;
    fn GetDiscardableImagesInRect(&self, rect: Rect) -> Vec<&B::DrawImage>;
    fn GetRasterInducingScrollOffsets(&self) -> ScrollOffsetMap;
    fn global_tile_state(&self) -> &GlobalStateThatImpactsTilePriority;
}

use std::{
    collections::{BTreeMap, HashMap},
    ops::{Deref, DerefMut},
    sync::Arc,
};
// Source: cc/tiles/picture_layer_tiling.h:78.
pub const kBorderTexels: i32 = 1;
// Source: cc/tiles/picture_layer_tiling.h:276-282.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(crate) enum PriorityRectType {
    VISIBLE_RECT = 0,
    PENDING_VISIBLE_RECT = 1,
    SKEWPORT_RECT = 2,
    SOON_BORDER_RECT = 3,
    EVENTUALLY_RECT = 4,
}
// Source: cc/tiles/picture_layer_tiling.h:224,455.
// Native unique_ptr may be moved/null before erase; actual null slots are kept.
pub type TileMap<B> = HashMap<TileIndex, Option<Box<Tile<B>>>>;
// Rust ownership split of the same resident owner, never a copied output.
// Source: cc/tiles/picture_layer_tiling.h:445-484.
pub struct PictureLayerTilingState<B: PictureLayerTilingTypes> {
    id_: PictureLayerTilingId,
    // Source: cc/tiles/picture_layer_tiling.h:445-451.
    raster_transform_: B::RasterTransform,
    client_: PictureLayerTilingClientId,
    tree_: WhichTree,
    raster_source_: Option<Arc<B::RasterSource>>,
    min_preraster_distance_: f32,
    max_preraster_distance_: f32,
    resolution_: TileResolution,
    // Source: cc/tiles/picture_layer_tiling.h:454,456-460.
    tiling_data_: TilingData,
    live_tiles_rect_: Rect,
    can_require_tiles_for_activation_: bool,
    tiling_rect_in_layer_space_: Rect,
    // Source: cc/tiles/picture_layer_tiling.h:463-472.
    current_visible_rect_: Rect,
    current_skewport_rect_: Rect,
    current_soon_border_rect_: Rect,
    current_eventually_rect_: Rect,
    current_visible_rect_in_layer_space_: Rect,
    current_skewport_rect_in_layer_space_: Rect,
    current_soon_border_rect_in_layer_space_: Rect,
    current_eventually_rect_in_layer_space_: Rect,
    // Source: cc/tiles/picture_layer_tiling.h:475-484.
    current_content_to_screen_scale_: f32,
    current_occlusion_in_layer_space_: Occlusion<B::OcclusionTransform>,
    max_skewport_extent_in_screen_space_: f32,
    has_visible_rect_tiles_: bool,
    has_skewport_rect_tiles_: bool,
    has_soon_border_rect_tiles_: bool,
    has_eventually_rect_tiles_: bool,
    // Source tiling_set_raster_queue_all.cc:133-140 marks completion through
    // the native range's tiling pointer. This one sequence-local flag is
    // logically mutable; the owning Tiling/TileMap stays resident and shared.
    all_tiles_done_: std::cell::Cell<bool>,
    can_use_lcd_text_: bool,
}
// Source: cc/tiles/picture_layer_tiling.h:73-76,444-484.
// State and TileMap are disjoint resident fields so native const tiling methods
// can safely mutate the currently borrowed Tile's priority flags.
pub struct PictureLayerTiling<B: PictureLayerTilingTypes> {
    state_: PictureLayerTilingState<B>,
    tiles_: TileMap<B>,
}
impl<B: PictureLayerTilingTypes> Deref for PictureLayerTiling<B> {
    type Target = PictureLayerTilingState<B>;
    fn deref(&self) -> &Self::Target {
        &self.state_
    }
}
impl<B: PictureLayerTilingTypes> DerefMut for PictureLayerTiling<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state_
    }
}
impl<B: PictureLayerTilingTypes> PictureLayerTiling<B> {
    // Source: cc/tiles/picture_layer_tiling.h:85-91;
    //         cc/tiles/picture_layer_tiling.cc:33-75.
    pub fn new(
        id: PictureLayerTilingId,
        client: PictureLayerTilingClientId,
        tree: WhichTree,
        raster_transform: &B::RasterTransform,
        raster_source: Arc<B::RasterSource>,
        min: f32,
        max: f32,
        lcd: bool,
        context: &mut dyn PictureLayerTilingContext<B>,
    ) -> Self {
        let mut result = Self {
            state_: PictureLayerTilingState {
                id_: id,
                raster_transform_: raster_transform.clone(),
                client_: client,
                tree_: tree,
                raster_source_: Some(raster_source),
                min_preraster_distance_: min,
                max_preraster_distance_: max,
                resolution_: TileResolution::NON_IDEAL_RESOLUTION,
                tiling_data_: TilingData::new(&Size::default(), &Rect::default(), kBorderTexels),
                live_tiles_rect_: Rect::default(),
                can_require_tiles_for_activation_: false,
                tiling_rect_in_layer_space_: Rect::default(),
                current_visible_rect_: Rect::default(),
                current_skewport_rect_: Rect::default(),
                current_soon_border_rect_: Rect::default(),
                current_eventually_rect_: Rect::default(),
                current_visible_rect_in_layer_space_: Rect::default(),
                current_skewport_rect_in_layer_space_: Rect::default(),
                current_soon_border_rect_in_layer_space_: Rect::default(),
                current_eventually_rect_in_layer_space_: Rect::default(),
                current_content_to_screen_scale_: 0.,
                current_occlusion_in_layer_space_: Occlusion::default(),
                max_skewport_extent_in_screen_space_: 0.,
                has_visible_rect_tiles_: false,
                has_skewport_rect_tiles_: false,
                has_soon_border_rect_tiles_: false,
                has_eventually_rect_tiles_: false,
                all_tiles_done_: std::cell::Cell::new(true),
                can_use_lcd_text_: lcd,
            },
            tiles_: HashMap::new(),
        };
        debug_assert!(!result.source().IsSolidColor());
        debug_assert!(
            raster_transform.translation().x() >= 0. && raster_transform.translation().x() < 1.
        );
        debug_assert!(
            raster_transform.translation().y() >= 0. && raster_transform.translation().y() < 1.
        );
        #[cfg(debug_assertions)]
        {
            let size = result.source().recorded_bounds().size();
            let scaled = SizeF::new(
                size.width() as f32 * raster_transform.scale().x(),
                size.height() as f32 * raster_transform.scale().y(),
            );
            let width = scaled.width();
            let height = scaled.height();
            let floored_width = width.floor() as i32;
            let floored_height = height.floor() as i32;
            let is_width_empty = floored_width == 0 && !MathUtil::IsWithinEpsilon(width, 1.);
            let is_height_empty = floored_height == 0 && !MathUtil::IsWithinEpsilon(height, 1.);
            debug_assert!(
                !is_width_empty && !is_height_empty,
                "tiling created with scale too small"
            );
        }
        let rect = result.ComputeTilingRect();
        result.state_.SetTilingRect(rect);
        let size = context.CalculateTileSizeForClient(client, rect.size());
        result.state_.tiling_data_.SetMaxTextureSize(&size);
        result
    }
    // Source: cc/tiles/picture_layer_tiling.h:93;
    //         cc/tiles/picture_layer_tiling.cc:77.
    // Owning Set calls this before dropping the Box. Each Tile destructor runs
    // with its actual manager; Drop below never captures/calls an owner.
    pub fn Destroy(&mut self, context: &mut dyn PictureLayerTileDestroyer<B>) {
        self.ClearTiles(context);
    }
    fn ReleaseTile(
        tile: Option<Box<Tile<B>>>,
        tiling: &PictureLayerTilingState<B>,
        context: &mut dyn PictureLayerTileDestroyer<B>,
    ) {
        if let Some(mut tile) = tile {
            let manager = tile.identity().manager;
            context.DestroyTileWithManager(manager, &mut tile, tiling);
        }
    }
    fn ClearTiles(&mut self, context: &mut dyn PictureLayerTileDestroyer<B>) {
        for (_, tile) in self.tiles_.drain() {
            Self::ReleaseTile(tile, &self.state_, context);
        }
    }
    // Source: cc/tiles/picture_layer_tiling.h:159-164.
    pub fn TileAt(&self, i: i32, j: i32) -> Option<&Tile<B>> {
        self.tiles_
            .get(&TileIndex::new(i, j))
            .and_then(|t| t.as_deref())
    }
    pub fn TileAtIndex(&self, index: &TileIndex) -> Option<&Tile<B>> {
        self.TileAt(index.i, index.j)
    }
    fn TileAtMutable(&mut self, index: TileIndex) -> Option<&mut Tile<B>> {
        self.tiles_.get_mut(&index).and_then(|t| t.as_deref_mut())
    }
    pub fn TileById(&self, id: TileId) -> &Tile<B> {
        assert_eq!(id.tiling, self.id_);
        let tile = self
            .TileAtIndex(&id.index)
            .expect("resident tile identity no longer attached");
        assert_eq!(tile.identity(), id);
        assert!(!tile.deleted());
        tile
    }
    pub fn TileByIdMutable(&mut self, id: TileId) -> &mut Tile<B> {
        assert_eq!(id.tiling, self.id_);
        let tile = self
            .TileAtMutable(id.index)
            .expect("resident tile identity no longer attached");
        assert_eq!(tile.identity(), id);
        assert!(!tile.deleted());
        tile
    }
    // Source: cc/tiles/picture_layer_tiling.h:166-171.
    pub fn has_tiles(&self) -> bool {
        !self.tiles_.is_empty()
    }
    // Source: cc/tiles/picture_layer_tiling.h:357;
    //         cc/tiles/picture_layer_tiling.cc:79-95.
    fn CreateTile(
        &mut self,
        info: &CreateInfo<B::RasterTransform>,
        context: &mut dyn PictureLayerTilingContext<B>,
    ) -> Option<TileId> {
        let i = info.tiling_i_index;
        let j = info.tiling_j_index;
        let index = TileIndex::new(i, j);
        debug_assert!(!self.tiles_.contains_key(&index));
        if !self.source().IntersectsRect(info.enclosing_layer_rect) {
            return None;
        }
        self.state_.all_tiles_done_.set(false);
        let tile = context.CreateTileForClient(self.client_, info);
        let tile_ptr = tile.as_ref().map(|t| t.identity());
        let old = self.tiles_.insert(index, tile);
        if let Some(old) = old {
            Self::ReleaseTile(old, &self.state_, context);
        }
        tile_ptr
    }
    // Source: cc/tiles/picture_layer_tiling.h:103;
    //         cc/tiles/picture_layer_tiling.cc:97-136.
    pub fn CreateMissingTilesInLiveTilesRect(
        &mut self,
        context: &mut dyn PictureLayerTilingContext<B>,
    ) {
        let active_twin = if self.tree_ == WhichTree::PENDING_TREE {
            crate::adapter::picture_layer_tiling::WithTilingClient(
                context,
                self.client_,
                |client| client.GetPendingOrActiveTwinTiling(&self.state_, context, None),
            )
        } else {
            None
        };
        // Source cc101-102 captures one Region* before tile creation. Keep that
        // same typed resident identity across the mutable CreateTile callback;
        // resolving it below never reselects the client's current invalidation.
        let invalidation = if active_twin.is_some() {
            crate::adapter::picture_layer_tiling::WithTilingClient(
                context,
                self.client_,
                |client| client.GetPendingInvalidation(context),
            )
        } else {
            None
        };
        let mut iter = DataIterator::new(&self.tiling_data_, &self.live_tiles_rect_, false);
        while iter.IsValid() {
            let index = TileIndex::from(iter.index());
            if self.tiles_.contains_key(&index) {
                iter.Advance();
                continue;
            }
            let info = self.CreateInfoForTile(index.i, index.j);
            if self.ShouldCreateTileAt(&info, context) {
                let tile = self.CreateTile(&info, context);
                if let Some(tile) = tile {
                    if invalidation.is_some()
                        && self.TilingMatchesTileIndices(context.TilingById(active_twin.unwrap()))
                    {
                        if let Some(old_tile) =
                            context.TilingById(active_twin.unwrap()).TileAtIndex(&index)
                        {
                            let old_id = old_tile.id();
                            let tile_rect = *self.TileById(tile).content_rect();
                            let mut invalidated = Rect::default();
                            for rect in context.PendingInvalidationById(invalidation.unwrap()) {
                                let mut invalid_content =
                                    self.EnclosingContentsRectFromLayerRect(rect);
                                invalid_content.Intersect(tile_rect);
                                invalidated.Union(invalid_content);
                            }
                            self.TileByIdMutable(tile)
                                .SetInvalidated(&invalidated, old_id);
                        }
                    }
                }
            }
            iter.Advance();
        }
        self.VerifyTiles();
    }
    // Source: cc/tiles/picture_layer_tiling.h:104-105;
    //         cc/tiles/picture_layer_tiling.cc:138-169.
    pub fn TakeTilesAndPropertiesFrom(
        &mut self,
        pending: &mut Self,
        invalidation: &Region<B::RegionBackend>,
        context: &mut dyn PictureLayerTilingContext<B>,
    ) {
        self.SetRasterSourceAndResize(
            pending.raster_source_.as_ref().unwrap().clone(),
            &mut BorrowedTwinTilingContext {
                external: context,
                twin: pending,
            },
        );
        self.RemoveTilesInRegion(
            invalidation,
            false,
            &mut BorrowedTwinTilingContext {
                external: context,
                twin: pending,
            },
        );
        self.state_.resolution_ = pending.resolution_;
        // Source cc147-152 begin/move/erase uses one resident mutable cursor.
        // retain(false) erases each moved-null slot after the callback, exactly
        // after target replacement/destruction; it does not rescan empty buckets.
        pending.tiles_.retain(|key, slot| {
            let tile = slot.as_mut().expect("source nonnull pending tile");
            let old_identity = tile.identity();
            tile.set_tiling(Some(self.state_.id_));
            context.RebindTileWithManager(old_identity, tile.identity());
            let moved = slot.take();
            if let Some(old) = self.tiles_.insert(*key, moved) {
                Self::ReleaseTile(old, &self.state_, context);
            }
            false
        });
        if self.all_tiles_done_.get() && !pending.all_tiles_done_.get() {
            self.state_.all_tiles_done_.set(false);
        }
        self.ComputeTilePriorityRects(
            &pending.current_visible_rect_in_layer_space_,
            &pending.current_skewport_rect_in_layer_space_,
            &pending.current_soon_border_rect_in_layer_space_,
            &pending.current_eventually_rect_in_layer_space_,
            pending.current_content_to_screen_scale_ * self.contents_scale_key(),
            &pending.current_occlusion_in_layer_space_,
            &mut BorrowedTwinTilingContext {
                external: context,
                twin: pending,
            },
        );
        debug_assert!(pending.tiles_.is_empty());
        pending.state_.all_tiles_done_.set(true);
        self.VerifyTiles();
    }
    // Source: cc/tiles/picture_layer_tiling.h:99-101;
    //         cc/tiles/picture_layer_tiling.cc:171-276.
    pub fn SetRasterSourceAndResize(
        &mut self,
        source: Arc<B::RasterSource>,
        context: &mut dyn PictureLayerTilingContext<B>,
    ) -> bool {
        debug_assert!(!source.IsSolidColor());
        self.state_.raster_source_ = Some(source);
        let rect = self.ComputeTilingRect();
        let size = context.CalculateTileSizeForClient(self.client_, rect.size());
        if size != self.tiling_data_.max_texture_size()
            || rect.origin() != self.tiling_data_.tiling_rect().origin()
        {
            self.state_.SetTilingRect(rect);
            self.state_.tiling_data_.SetMaxTextureSize(&size);
            self.Reset(context);
            return true;
        }
        if rect == *self.tiling_data_.tiling_rect() {
            return false;
        }
        let before_eventually_left = self
            .tiling_data_
            .TileXIndexFromSrcCoord(self.current_eventually_rect_.x);
        let before_eventually_top = self
            .tiling_data_
            .TileYIndexFromSrcCoord(self.current_eventually_rect_.y);
        let before_eventually_right = self
            .tiling_data_
            .TileXIndexFromSrcCoord(self.current_eventually_rect_.right() - 1);
        let before_eventually_bottom = self
            .tiling_data_
            .TileYIndexFromSrcCoord(self.current_eventually_rect_.bottom() - 1);
        let before_live_left = self
            .tiling_data_
            .TileXIndexFromSrcCoord(self.live_tiles_rect_.x);
        let before_live_top = self
            .tiling_data_
            .TileYIndexFromSrcCoord(self.live_tiles_rect_.y);
        let before_live_right = self
            .tiling_data_
            .TileXIndexFromSrcCoord(self.live_tiles_rect_.right() - 1);
        let before_live_bottom = self
            .tiling_data_
            .TileYIndexFromSrcCoord(self.live_tiles_rect_.bottom() - 1);
        self.state_.live_tiles_rect_.Intersect(rect);
        self.state_.SetTilingRect(rect);
        let after_eventually_right = self.tiling_data_.num_tiles_x() - 1;
        let after_eventually_bottom = self.tiling_data_.num_tiles_y() - 1;
        let mut after_live_right = -1;
        let mut after_live_bottom = -1;
        if !self.live_tiles_rect_.IsEmpty() {
            after_live_right = self
                .tiling_data_
                .TileXIndexFromSrcCoord(self.live_tiles_rect_.right() - 1);
            after_live_bottom = self
                .tiling_data_
                .TileYIndexFromSrcCoord(self.live_tiles_rect_.bottom() - 1);
        }
        for i in after_eventually_right + 1..=before_eventually_right {
            for j in before_eventually_top..=before_eventually_bottom {
                let tile = self.TakeTileAt(i, j);
                Self::ReleaseTile(tile, &self.state_, context);
            }
        }
        for i in before_eventually_left..=after_eventually_right {
            for j in after_eventually_bottom + 1..=before_eventually_bottom {
                let tile = self.TakeTileAt(i, j);
                Self::ReleaseTile(tile, &self.state_, context);
            }
        }
        if after_live_right > before_live_right {
            debug_assert_eq!(after_live_right, before_live_right + 1);
            for j in before_live_top..=after_live_bottom {
                let info = self.CreateInfoForTile(after_live_right, j);
                if self.ShouldCreateTileAt(&info, context) {
                    self.CreateTile(&info, context);
                }
            }
        }
        if after_live_bottom > before_live_bottom {
            let boundary_right = after_live_right.min(before_live_right);
            debug_assert_eq!(after_live_bottom, before_live_bottom + 1);
            for i in before_live_left..=boundary_right {
                let info = self.CreateInfoForTile(i, after_live_bottom);
                if self.ShouldCreateTileAt(&info, context) {
                    self.CreateTile(&info, context);
                }
            }
        }
        self.VerifyTiles();
        false
    }
    // Source: cc/tiles/picture_layer_tiling.h:102;
    //         cc/tiles/picture_layer_tiling.cc:278-281.
    pub fn Invalidate(
        &mut self,
        invalidation: &Region<B::RegionBackend>,
        context: &mut dyn PictureLayerTilingContext<B>,
    ) {
        debug_assert!(
            self.tree_ != WhichTree::ACTIVE_TREE
                || crate::adapter::picture_layer_tiling::WithTilingClient(
                    context,
                    self.client_,
                    |client| client.GetPendingOrActiveTwinTiling(&self.state_, context, None)
                )
                .is_none()
        );
        self.RemoveTilesInRegion(invalidation, true, context);
    }
    // Source: cc/tiles/picture_layer_tiling.h:434;
    //         cc/tiles/picture_layer_tiling.cc:283-328.
    pub fn RemoveTilesInRegion(
        &mut self,
        invalidation: &Region<B::RegionBackend>,
        recreate: bool,
        context: &mut dyn PictureLayerTilingContext<B>,
    ) {
        if self.current_eventually_rect_.IsEmpty() {
            return;
        }
        let mut remove_tiles: BTreeMap<TileIndex, Rect> = BTreeMap::new();
        let expanded = self
            .tiling_data_
            .ExpandRectToTileBounds(&self.current_eventually_rect_);
        for layer_rect in invalidation {
            let invalid_content_rect = self.EnclosingContentsRectFromLayerRect(layer_rect);
            let mut coverage = invalid_content_rect;
            coverage.Intersect(expanded);
            if coverage.IsEmpty() {
                continue;
            }
            let mut iter = DataIterator::new(&self.tiling_data_, &coverage, true);
            while iter.IsValid() {
                remove_tiles
                    .entry(TileIndex::from(iter.index()))
                    .or_default()
                    .Union(invalid_content_rect);
                iter.Advance();
            }
        }
        for (index, invalid_content_rect) in remove_tiles {
            let old = self.TakeTileAt(index.i, index.j);
            if recreate && old.is_some() {
                let info = self.CreateInfoForTile(index.i, index.j);
                if let Some(tile) = self.CreateTile(&info, context) {
                    self.TileByIdMutable(tile)
                        .SetInvalidated(&invalid_content_rect, old.as_ref().unwrap().id());
                }
            }
            Self::ReleaseTile(old, &self.state_, context);
        }
    }
    // Source: cc/tiles/picture_layer_tiling.h:368;
    //         cc/tiles/picture_layer_tiling.cc:330-342.
    fn CreateInfoForTile(&self, i: i32, j: i32) -> CreateInfo<B::RasterTransform> {
        let mut rect = self.tiling_data_.TileBoundsWithBorder(i, j);
        let size = self.tiling_data_.max_texture_size();
        rect = Rect::new(rect.x, rect.y, size.width(), size.height());
        let enclosing = self.EnclosingLayerRectFromContentsRect(rect);
        CreateInfo {
            tiling: Some(self.id_),
            tiling_i_index: i,
            tiling_j_index: j,
            enclosing_layer_rect: enclosing,
            content_rect: rect,
            raster_transform: self.raster_transform_.clone(),
            can_use_lcd_text: self.can_use_lcd_text_,
        }
    }
    // Source: cc/tiles/picture_layer_tiling.h:369;
    //         cc/tiles/picture_layer_tiling.cc:344-405.
    fn ShouldCreateTileAt(
        &self,
        info: &CreateInfo<B::RasterTransform>,
        queries: &dyn PictureLayerTilingQueries<B>,
    ) -> bool {
        let i = info.tiling_i_index;
        let j = info.tiling_j_index;
        if self.tree_ == WhichTree::ACTIVE_TREE {
            return true;
        }
        let twin = crate::adapter::picture_layer_tiling::WithTilingClient(
            queries,
            self.client_,
            |client| client.GetPendingOrActiveTwinTiling(&self.state_, queries, None),
        );
        let Some(id) = twin else {
            return true;
        };
        let twin = queries.TilingById(id);
        if !self.TilingMatchesTileIndices(twin) {
            return true;
        }
        if self.can_use_lcd_text() != twin.can_use_lcd_text()
            || self.raster_transform() != twin.raster_transform()
        {
            return true;
        }
        if !twin.source().IntersectsRect(info.enclosing_layer_rect) {
            return true;
        }
        let invalidation = crate::adapter::picture_layer_tiling::WithTilingClient(
            queries,
            self.client_,
            |client| client.GetPendingInvalidation(queries),
        )
        .expect("source nonnull pending invalidation");
        for layer_rect in queries.PendingInvalidationById(invalidation) {
            let rect = self.EnclosingContentsRectFromLayerRect(layer_rect);
            if rect.Intersects(info.content_rect) {
                return true;
            }
        }
        if twin.TileAt(i, j).is_none() && self.current_visible_rect_.Intersects(info.content_rect) {
            return true;
        }
        false
    }
    // Source: cc/tiles/picture_layer_tiling.h:361;
    //         cc/tiles/picture_layer_tiling.cc:407-412.
    fn TilingMatchesTileIndices(&self, twin: &Self) -> bool {
        self.state_.MatchesState(&twin.state_)
    }
    // Source: cc/tiles/picture_layer_tiling.h:358-360;
    //         cc/tiles/picture_layer_tiling.cc:414-421.
    pub fn TakeTileAt(&mut self, i: i32, j: i32) -> Option<Box<Tile<B>>> {
        let index = TileIndex::new(i, j);
        let slot = self.tiles_.get_mut(&index)?;
        let result = slot.take();
        self.tiles_.remove(&index);
        result
    }
    // Source: cc/tiles/picture_layer_tiling.h:218-222;
    //         cc/tiles/picture_layer_tiling.cc:423-449.
    pub fn SetTilePriorityRectsForTesting(
        &mut self,
        visible: Rect,
        skewport: Rect,
        soon: Rect,
        eventually: Rect,
        evicts: bool,
        context: &mut dyn PictureLayerTilingContext<B>,
    ) {
        self.state_.current_occlusion_in_layer_space_ = Occlusion::default();
        self.state_.current_content_to_screen_scale_ = 1.;
        self.SetPriorityRect(
            self.EnclosingLayerRectFromContentsRect(visible),
            PriorityRectType::VISIBLE_RECT,
            false,
            context,
        );
        self.SetPriorityRect(
            self.EnclosingLayerRectFromContentsRect(skewport),
            PriorityRectType::SKEWPORT_RECT,
            false,
            context,
        );
        self.SetPriorityRect(
            self.EnclosingLayerRectFromContentsRect(soon),
            PriorityRectType::SOON_BORDER_RECT,
            false,
            context,
        );
        self.SetPriorityRect(
            self.EnclosingLayerRectFromContentsRect(eventually),
            PriorityRectType::EVENTUALLY_RECT,
            evicts,
            context,
        );
        self.state_.max_skewport_extent_in_screen_space_ = self.state_.SkewportExtent() as f32;
    }
    // Source: cc/tiles/picture_layer_tiling.h:242;
    //         cc/tiles/picture_layer_tiling.cc:451-455.
    pub fn Reset(&mut self, context: &mut dyn PictureLayerTileDestroyer<B>) {
        self.state_.live_tiles_rect_ = Rect::default();
        self.ClearTiles(context);
        self.state_.all_tiles_done_.set(true);
    }
    // Source: cc/tiles/picture_layer_tiling.h:244-250;
    //         cc/tiles/picture_layer_tiling.cc:457-526.
    pub fn ComputeTilePriorityRects(
        &mut self,
        visible: &Rect,
        skewport: &Rect,
        soon: &Rect,
        eventually: &Rect,
        ideal_scale: f32,
        occlusion: &Occlusion<B::OcclusionTransform>,
        context: &mut dyn PictureLayerTilingContext<B>,
    ) {
        if occlusion.HasOcclusion() || self.current_occlusion_in_layer_space_.HasOcclusion() {
            self.state_.all_tiles_done_.set(false);
        }
        let policy = crate::adapter::picture_layer_tiling::WithTilingClient(
            context,
            self.client_,
            |client| client.global_tile_state().memory_limit_policy,
        );
        self.state_.current_occlusion_in_layer_space_ = occlusion.clone();
        self.state_.current_content_to_screen_scale_ = ideal_scale / self.contents_scale_key();
        self.SetPriorityRect(*visible, PriorityRectType::VISIBLE_RECT, false, context);
        self.SetPriorityRect(*skewport, PriorityRectType::SKEWPORT_RECT, false, context);
        self.SetPriorityRect(*soon, PriorityRectType::SOON_BORDER_RECT, false, context);
        self.SetPriorityRect(
            *eventually,
            PriorityRectType::EVENTUALLY_RECT,
            true,
            context,
        );
        self.state_.max_skewport_extent_in_screen_space_ =
            self.current_content_to_screen_scale_ * self.state_.SkewportExtent() as f32;
        let mut live;
        if context.IsCCSlimmingEnabled() {
            live = self.current_visible_rect_;
            let mut draws = self.has_visible_rect_tiles_;
            if policy >= TileMemoryLimitPolicy::ALLOW_PREPAINT_ONLY {
                draws |= self.has_skewport_rect_tiles_;
                if self.has_skewport_rect_tiles_ {
                    live.Union(self.current_skewport_rect_);
                }
                draws |= self.has_soon_border_rect_tiles_;
                if self.has_soon_border_rect_tiles_ {
                    live.Union(self.current_soon_border_rect_);
                }
            }
            if policy >= TileMemoryLimitPolicy::ALLOW_ANYTHING {
                draws |= self.has_eventually_rect_tiles_;
                if self.has_eventually_rect_tiles_ {
                    live.Union(self.current_eventually_rect_);
                }
            }
            if !draws {
                self.state_.all_tiles_done_.set(true);
            }
        } else {
            live = self.current_eventually_rect_;
        }
        live.Intersect(*self.tiling_rect());
        self.SetLiveTilesRect(live, context);
    }
    // Source: cc/tiles/picture_layer_tiling.h:363-365;
    //         cc/tiles/picture_layer_tiling.cc:528-581.
    fn SetPriorityRect(
        &mut self,
        rect: Rect,
        kind: PriorityRectType,
        evicts: bool,
        context: &mut dyn PictureLayerTilingContext<B>,
    ) {
        debug_assert!(!evicts || kind == PriorityRectType::EVENTUALLY_RECT);
        match kind {
            PriorityRectType::VISIBLE_RECT => {
                if self.current_visible_rect_in_layer_space_ != rect {
                    self.state_.current_visible_rect_in_layer_space_ = rect;
                    self.state_.current_visible_rect_ =
                        self.EnclosingContentsRectFromLayerRect(rect);
                    self.state_.has_visible_rect_tiles_ =
                        self.tiling_rect().Intersects(self.current_visible_rect_);
                }
            }
            PriorityRectType::SKEWPORT_RECT => {
                if self.current_skewport_rect_in_layer_space_ != rect {
                    self.state_.current_skewport_rect_in_layer_space_ = rect;
                    self.state_.current_skewport_rect_ =
                        self.EnclosingContentsRectFromLayerRect(rect);
                    self.state_.has_skewport_rect_tiles_ =
                        self.tiling_rect().Intersects(self.current_skewport_rect_);
                }
            }
            PriorityRectType::SOON_BORDER_RECT => {
                if self.current_soon_border_rect_in_layer_space_ != rect {
                    self.state_.current_soon_border_rect_in_layer_space_ = rect;
                    self.state_.current_soon_border_rect_ =
                        self.EnclosingContentsRectFromLayerRect(rect);
                    self.state_.has_soon_border_rect_tiles_ = self
                        .tiling_rect()
                        .Intersects(self.current_soon_border_rect_);
                }
            }
            PriorityRectType::EVENTUALLY_RECT => {
                if self.current_eventually_rect_in_layer_space_ != rect {
                    self.state_.current_eventually_rect_in_layer_space_ = rect;
                    let rect = self.EnclosingContentsRectFromLayerRect(rect);
                    if evicts {
                        let mut iter = DifferenceIterator::new(
                            &self.tiling_data_,
                            &self.current_eventually_rect_,
                            &rect,
                        );
                        while iter.IsValid() {
                            let tile = self.TakeTileAt(iter.index_x(), iter.index_y());
                            Self::ReleaseTile(tile, &self.state_, context);
                            iter.Advance();
                        }
                    }
                    self.state_.current_eventually_rect_ = rect;
                    self.state_.has_eventually_rect_tiles_ =
                        self.tiling_rect().Intersects(self.current_eventually_rect_);
                }
            }
            PriorityRectType::PENDING_VISIBLE_RECT => unreachable!("source NOTREACHED"),
        }
    }
    // Source: cc/tiles/picture_layer_tiling.h:355;
    //         cc/tiles/picture_layer_tiling.cc:583-621.
    fn SetLiveTilesRect(&mut self, new_rect: Rect, context: &mut dyn PictureLayerTilingContext<B>) {
        debug_assert!(new_rect.IsEmpty() || self.tiling_rect().Contains(new_rect));
        if self.live_tiles_rect_ == new_rect {
            return;
        }
        if self.resolution_ == TileResolution::NON_IDEAL_RESOLUTION {
            self.state_.live_tiles_rect_.Intersect(new_rect);
            self.VerifyTiles();
            return;
        }
        let mut iter =
            DifferenceIterator::new(&self.tiling_data_, &new_rect, &self.live_tiles_rect_);
        while iter.IsValid() {
            let info = self.CreateInfoForTile(iter.index_x(), iter.index_y());
            if self.ShouldCreateTileAt(&info, context) {
                if let Some(tile) = self.TileAt(iter.index_x(), iter.index_y()) {
                    if !tile.IsReadyToDraw() {
                        self.state_.all_tiles_done_.set(false);
                    }
                } else {
                    self.CreateTile(&info, context);
                }
            }
            iter.Advance();
        }
        self.state_.live_tiles_rect_ = new_rect;
        if self.tiles_.len() == 0 {
            self.state_.all_tiles_done_.set(true);
        }
        self.VerifyTiles();
    }
    // Source: cc/tiles/picture_layer_tiling.h:356;
    //         cc/tiles/picture_layer_tiling.cc:623-643.
    fn VerifyTiles(&self) {
        #[cfg(debug_assertions)]
        for (index, tile) in &self.tiles_ {
            debug_assert!(tile.is_some());
            debug_assert!(index.i < self.tiling_data_.num_tiles_x());
            debug_assert!(index.j < self.tiling_data_.num_tiles_y());
            debug_assert!(
                self.tiling_data_
                    .TileBounds(index.i, index.j)
                    .Intersects(self.current_eventually_rect_)
            );
        }
    }
    // Source: cc/tiles/picture_layer_tiling.h:107-116,371-398.
    pub fn IsTileRequiredForActivation(
        &self,
        tile: &Tile<B>,
        q: &dyn PictureLayerTilingQueries<B>,
    ) -> bool {
        self.state_.IsRequiredForActivation(
            tile,
            &|t| self.state_.IsTileVisible(t),
            self.state_.IsTileOccluded(tile, q),
            q,
            SelectedTile {
                state: &self.state_,
                tile,
            },
        )
    }
    pub fn IsTileRequiredForDraw(&self, tile: &Tile<B>) -> bool {
        self.state_
            .IsRequiredForDraw(tile, |t| self.state_.IsTileVisible(t))
    }
    pub fn IsTileOccluded(&self, tile: &Tile<B>, q: &dyn PictureLayerTilingQueries<B>) -> bool {
        self.state_.IsTileOccluded(tile, q)
    }
    // Source: cc/tiles/picture_layer_tiling.h:118-120;
    //         cc/tiles/picture_layer_tiling.cc:661-692.
    pub fn ShouldDecodeCheckeredImagesForTile(
        &self,
        tile: &Tile<B>,
        q: &dyn PictureLayerTilingQueries<B>,
    ) -> bool {
        self.state_.ShouldDecodeCheckeredImagesForTile(
            tile,
            q,
            SelectedTile {
                state: &self.state_,
                tile,
            },
        )
    }
    // Source: cc/tiles/picture_layer_tiling.h:257;
    //         cc/tiles/picture_layer_tiling.cc:694-700.
    pub fn UpdateRequiredStatesOnTile(&self, id: TileId, q: &dyn PictureLayerTilingQueries<B>) {
        let state = &self.state_;
        let tile = self
            .tiles_
            .get(&id.index)
            .and_then(|t| t.as_deref())
            .expect("source nonnull tile");
        Self::ValidateEntry(state, id.index, tile, id);
        state.UpdateRequiredStatesOnTile(tile, q);
    }
    // Source: cc/tiles/picture_layer_tiling.h:259-261;
    //         cc/tiles/picture_layer_tiling.cc:702-706.
    pub fn Cover(&self, rect: &Rect, scale: f32) -> PictureLayerTilingCoverageIterator<'_, B> {
        TilingCoverageIterator::new(self, scale, rect)
    }
    // Source: cc/tiles/picture_layer_tiling.h:400-402;
    //         cc/tiles/picture_layer_tiling.cc:708-746.
    pub(crate) fn MakePrioritizedTile(
        &self,
        id: TileId,
        kind: PriorityRectType,
        occluded: bool,
        q: &dyn PictureLayerTilingQueries<B>,
    ) -> PrioritizedTile {
        let state = &self.state_;
        let tile = self
            .tiles_
            .get(&id.index)
            .and_then(|t| t.as_deref())
            .expect("source nonnull tile");
        Self::ValidateEntry(state, id.index, tile, id);
        state.MakePrioritizedTile(tile, kind, occluded, q)
    }
    // Actual map entry provenance replaces the source DCHECK(TileAt(i,j)==tile)
    // while flags are exclusively borrowed. This verifies every identity part
    // against the resident Tile; no key list, secondary map or scan is created.
    fn ValidateEntry(
        state: &PictureLayerTilingState<B>,
        index: TileIndex,
        tile: &Tile<B>,
        expected: TileId,
    ) {
        assert_eq!(tile.identity(), expected);
        assert_eq!(expected.tiling, state.id_);
        assert_eq!(expected.index, index);
        assert!(!tile.deleted());
    }
    // Source: cc/tiles/picture_layer_tiling.h:186-190;
    //         cc/tiles/picture_layer_tiling.cc:748-754.
    pub fn CreateAllTilesForTesting(
        &mut self,
        rect: Option<Rect>,
        context: &mut dyn PictureLayerTilingContext<B>,
    ) {
        let rect = rect.unwrap_or(*self.tiling_rect());
        self.SetTilePriorityRectsForTesting(rect, rect, rect, rect, true, context);
        self.SetLiveTilesRect(rect, context);
    }
    // Source: cc/tiles/picture_layer_tiling.h:191-197.
    pub fn TilingDataForTesting(&self) -> &TilingData {
        &self.tiling_data_
    }
    pub fn AllTilesForTesting(&self) -> Vec<Option<TileId>> {
        let mut result = Vec::new();
        for tile in self.tiles_.values() {
            result.push(tile.as_ref().map(|t| t.identity()));
        }
        result
    }
    // Source: cc/tiles/picture_layer_tiling.h:199-205;
    //         cc/tiles/picture_layer_tiling.cc:756-766.
    pub fn UpdateAllRequiredStateForTesting(&mut self, q: &dyn PictureLayerTilingQueries<B>) {
        let state = &self.state_;
        for (index, tile) in &mut self.tiles_ {
            let tile = tile.as_deref_mut().expect("source nonnull tile");
            Self::ValidateEntry(state, *index, tile, tile.identity());
            state.UpdateRequiredStatesOnTile(tile, q);
        }
    }
    pub fn UpdateAndGetAllPrioritizedTilesForTesting(
        &mut self,
        q: &dyn PictureLayerTilingQueries<B>,
    ) -> BTreeMap<TileId, PrioritizedTile> {
        let mut result = BTreeMap::new();
        let state = &self.state_;
        for (index, tile) in &mut self.tiles_ {
            let tile = tile.as_deref_mut().expect("source nonnull tile");
            Self::ValidateEntry(state, *index, tile, tile.identity());
            let priority = state.ComputePriorityRectTypeForTile(tile, q);
            let occluded = state.IsTileOccluded(tile, q);
            let prioritized = state.MakePrioritizedTile(tile, priority, occluded, q);
            result.insert(tile.identity(), prioritized);
        }
        result
    }
    // Source: cc/tiles/picture_layer_tiling.h:207-217.
    pub fn SetAllTilesOccludedForTesting(&mut self) {
        let viewport = self.EnclosingLayerRectFromContentsRect(self.current_visible_rect_);
        self.state_.current_occlusion_in_layer_space_ = Occlusion::new(
            &B::OcclusionTransform::default(),
            SimpleEnclosedRegion::new(viewport),
            SimpleEnclosedRegion::new(viewport),
        );
    }
    pub fn GetCurrentVisibleRectForTesting(&self) -> &Rect {
        &self.current_visible_rect_
    }
    // Source: cc/tiles/picture_layer_tiling.h:403-406;
    //         cc/tiles/picture_layer_tiling.cc:768-829.
    pub(crate) fn ComputePriorityRectTypeForTile(
        &self,
        tile: &Tile<B>,
        q: &dyn PictureLayerTilingQueries<B>,
    ) -> PriorityRectType {
        debug_assert_eq!(
            self.TileAt(tile.tiling_i_index(), tile.tiling_j_index())
                .map(|t| t.identity()),
            Some(tile.identity())
        );
        self.state_.ComputePriorityRectTypeForTile(tile, q)
    }
    pub(crate) fn ComputePriorityForTile(
        &self,
        tile: &Tile<B>,
        kind: PriorityRectType,
        occluded: bool,
        q: &dyn PictureLayerTilingQueries<B>,
    ) -> TilePriority {
        debug_assert_eq!(
            self.TileAt(tile.tiling_i_index(), tile.tiling_j_index())
                .map(|t| t.identity()),
            Some(tile.identity())
        );
        self.state_.ComputePriorityForTile(tile, kind, occluded, q)
    }
    // Source: cc/tiles/picture_layer_tiling.h:252-253;
    //         cc/tiles/picture_layer_tiling.cc:831-838.
    pub fn GetAllPrioritizedTilesForTracing(
        &mut self,
        output: &mut Vec<PrioritizedTile>,
        q: &dyn PictureLayerTilingQueries<B>,
    ) {
        let state = &self.state_;
        for (index, tile) in &mut self.tiles_ {
            let tile = tile.as_deref_mut().expect("source nonnull tile");
            Self::ValidateEntry(state, *index, tile, tile.identity());
            let kind = state.ComputePriorityRectTypeForTile(tile, q);
            let occluded = state.IsTileOccluded(tile, q);
            output.push(state.MakePrioritizedTile(tile, kind, occluded, q));
        }
    }
    // Source: cc/tiles/picture_layer_tiling.h:254;
    //         cc/tiles/picture_layer_tiling.cc:840-862.
    pub fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        value.SetInteger("num_tiles", self.tiles_.len().min(i32::MAX as usize) as i32);
        self.state_.AsValueInto(value);
    }
    // Source: cc/tiles/picture_layer_tiling.h:255;
    //         cc/tiles/picture_layer_tiling.cc:864-871.
    pub fn GPUMemoryUsageInBytes(&self) -> usize {
        let mut amount = 0;
        for tile in self.tiles_.values() {
            amount += tile
                .as_ref()
                .expect("source nonnull tile")
                .GPUMemoryUsageInBytes();
        }
        amount
    }
    // Source: cc/tiles/picture_layer_tiling.h:177-184.
    pub fn VerifyNoTileNeedsRaster(&self, _q: &dyn PictureLayerTilingQueries<B>) {
        #[cfg(debug_assertions)]
        for tile in self.tiles_.values() {
            let tile = tile.as_ref().expect("source nonnull tile");
            debug_assert!(!tile.draw_info().NeedsRaster() || self.IsTileOccluded(tile, _q));
        }
    }
}
impl<B: PictureLayerTilingTypes> Drop for PictureLayerTiling<B> {
    fn drop(&mut self) {
        debug_assert!(
            self.tiles_.is_empty(),
            "owning Set must call Destroy with live TileManager before dropping tiling"
        );
    }
}
// Source: cc/tiles/picture_layer_tiling.h:487-491.
pub type PictureLayerTilingCoverageIterator<'a, B> =
    TilingCoverageIterator<'a, PictureLayerTiling<B>>;
impl<B: PictureLayerTilingTypes> super::tiling_internal::Tiling for PictureLayerTiling<B> {
    type Tile = Tile<B>;
    type RasterTransform = B::RasterTransform;
    fn TileAt(&self, index: &TileIndex) -> Option<&Tile<B>> {
        self.TileAtIndex(index)
    }
    fn contents_scale_key(&self) -> f32 {
        self.state_.contents_scale_key()
    }
    fn tiling_data(&self) -> &TilingData {
        &self.tiling_data_
    }
    fn raster_size(&self) -> Size {
        self.state_.raster_size()
    }
    fn raster_transform(&self) -> &B::RasterTransform {
        &self.raster_transform_
    }
}

// Call-scoped selected entry, carrying actual state/Tile borrows only. The
// source's twin query can revisit this same tile while its priority flags are
// being computed; it never needs a second mutable map/owner borrow.
struct SelectedTile<'a, B: PictureLayerTilingTypes> {
    state: &'a PictureLayerTilingState<B>,
    tile: &'a Tile<B>,
}
impl<B: PictureLayerTilingTypes> Copy for SelectedTile<'_, B> {}
impl<B: PictureLayerTilingTypes> Clone for SelectedTile<'_, B> {
    fn clone(&self) -> Self {
        *self
    }
}
enum TwinRead<'a, B: PictureLayerTilingTypes> {
    Resident(&'a PictureLayerTiling<B>),
    Selected(SelectedTile<'a, B>),
}
impl<B: PictureLayerTilingTypes> TwinRead<'_, B> {
    fn state(&self) -> &PictureLayerTilingState<B> {
        match self {
            Self::Resident(t) => &t.state_,
            Self::Selected(t) => t.state,
        }
    }
    fn HasTileMatching(&self, tile: &Tile<B>) -> bool {
        match self {
            Self::Resident(t) => t
                .TileAt(tile.tiling_i_index(), tile.tiling_j_index())
                .is_some(),
            Self::Selected(t) => {
                assert_eq!(t.tile.tiling_i_index(), tile.tiling_i_index());
                assert_eq!(t.tile.tiling_j_index(), tile.tiling_j_index());
                true
            }
        }
    }
}
impl<B: PictureLayerTilingTypes> PictureLayerTilingState<B> {
    pub fn identity(&self) -> PictureLayerTilingId {
        self.id_
    }
    // Source: cc/tiles/picture_layer_tiling.h:97.
    pub fn client(&self) -> PictureLayerTilingClientId {
        self.client_
    }
    // Source: cc/tiles/picture_layer_tiling.h:122-131.
    pub fn set_resolution(&mut self, resolution: TileResolution) {
        self.resolution_ = resolution;
    }
    pub fn resolution(&self) -> TileResolution {
        self.resolution_
    }
    pub fn set_can_require_tiles_for_activation(&mut self, value: bool) {
        self.can_require_tiles_for_activation_ = value;
    }
    pub fn can_require_tiles_for_activation(&self) -> bool {
        self.can_require_tiles_for_activation_
    }
    // Source: cc/tiles/picture_layer_tiling.h:133-145.
    pub fn raster_size(&self) -> Size {
        self.source().size()
    }
    pub fn raster_source(&self) -> &Option<Arc<B::RasterSource>> {
        &self.raster_source_
    }
    fn source(&self) -> &B::RasterSource {
        self.raster_source_
            .as_deref()
            .expect("source nonnull RasterSource")
    }
    pub fn GetPaintWorkletRecords<'a>(
        &self,
        q: &'a dyn PictureLayerTilingQueries<B>,
    ) -> &'a B::PaintWorkletRecordMap {
        q.PaintWorkletRecordsForClient(self.client_)
    }
    pub fn GetRasterInducingScrollOffsets(
        &self,
        q: &dyn PictureLayerTilingQueries<B>,
    ) -> ScrollOffsetMap {
        crate::adapter::picture_layer_tiling::WithTilingClient(q, self.client_, |client| {
            client.GetRasterInducingScrollOffsets()
        })
    }
    pub fn tiling_rect(&self) -> &Rect {
        self.tiling_data_.tiling_rect()
    }
    pub fn live_tiles_rect(&self) -> &Rect {
        &self.live_tiles_rect_
    }
    pub fn tile_size(&self) -> Size {
        self.tiling_data_.max_texture_size()
    }
    // Source: cc/tiles/picture_layer_tiling.h:146-157.
    pub fn contents_scale_key(&self) -> f32 {
        let scale = self.raster_transform_.scale();
        cpp_max(scale.x(), scale.y())
    }
    pub fn raster_transform(&self) -> &B::RasterTransform {
        &self.raster_transform_
    }
    pub fn tiling_data(&self) -> &TilingData {
        &self.tiling_data_
    }
    // Source: cc/tiles/picture_layer_tiling.h:167-175.
    pub fn all_tiles_done(&self) -> bool {
        self.all_tiles_done_.get()
    }
    pub fn set_all_tiles_done(&self, value: bool) {
        self.all_tiles_done_.set(value);
    }
    pub fn can_use_lcd_text(&self) -> bool {
        self.can_use_lcd_text_
    }
    pub fn tree(&self) -> WhichTree {
        self.tree_
    }
    // Source: cc/tiles/picture_layer_tiling.h:284-288.
    fn IsTileVisible(&self, tile: &Tile<B>) -> bool {
        let bounds = self
            .tiling_data_
            .TileBounds(tile.tiling_i_index(), tile.tiling_j_index());
        bounds.Intersects(self.current_visible_rect_)
    }
    // This resolver only selects native live twin state. Selected tile/state
    // are actual source borrows, allowing the source recursive pending→active
    // query to revisit the original entry while its map is exclusively borrowed.
    fn Twin<'a>(
        &'a self,
        q: &'a dyn PictureLayerTilingQueries<B>,
        selected: SelectedTile<'a, B>,
    ) -> Option<TwinRead<'a, B>> {
        let id =
            crate::adapter::picture_layer_tiling::WithTilingClient(q, self.client_, |client| {
                client.GetPendingOrActiveTwinTiling(self, q, Some(selected.state))
            })?;
        Some(if id == selected.state.id_ {
            TwinRead::Selected(selected)
        } else {
            TwinRead::Resident(q.TilingById(id))
        })
    }
    // Source: cc/tiles/picture_layer_tiling.h:290-346.
    fn IsRequiredForActivation(
        &self,
        tile: &Tile<B>,
        is_visible: &dyn Fn(&Tile<B>) -> bool,
        is_tile_occluded: bool,
        q: &dyn PictureLayerTilingQueries<B>,
        selected: SelectedTile<'_, B>,
    ) -> bool {
        if self.tree_ == WhichTree::PENDING_TREE {
            if !self.can_require_tiles_for_activation_
                || self.resolution_ != TileResolution::HIGH_RESOLUTION
                || is_tile_occluded
            {
                return false;
            }
            if tile.tiling_i_index() >= self.tiling_data_.num_tiles_x()
                || tile.tiling_j_index() >= self.tiling_data_.num_tiles_y()
            {
                return false;
            }
            if !is_visible(tile) {
                return false;
            }
            if crate::adapter::picture_layer_tiling::WithTilingClient(q, self.client_, |client| {
                client.RequiresHighResToDraw()
            }) {
                return true;
            }
            let twin = self.Twin(q, selected);
            let Some(twin) = twin else {
                return true;
            };
            if !self.MatchesState(twin.state()) {
                return true;
            }
            if twin.state().source().size() != self.source().size()
                || twin.state().source().recorded_bounds() != self.source().recorded_bounds()
            {
                return true;
            }
            if twin.state().current_visible_rect_ != self.current_visible_rect_ {
                return true;
            }
            if !twin.HasTileMatching(tile) {
                return false;
            }
            return true;
        }
        debug_assert_eq!(self.tree_, WhichTree::ACTIVE_TREE);
        let twin = self.Twin(q, selected);
        let Some(twin) = twin else {
            return false;
        };
        if !self.MatchesState(twin.state()) || twin.HasTileMatching(tile) {
            return false;
        }
        let pending = twin.state();
        pending.IsRequiredForActivation(
            tile,
            &|t| pending.IsTileVisible(t),
            pending.IsTileOccludedWithSelected(tile, q, selected),
            q,
            selected,
        )
    }
    // Source: cc/tiles/picture_layer_tiling.h:348-353.
    fn IsRequiredForDraw(&self, tile: &Tile<B>, is_visible: impl Fn(&Tile<B>) -> bool) -> bool {
        self.tree_ == WhichTree::ACTIVE_TREE
            && self.resolution_ == TileResolution::HIGH_RESOLUTION
            && is_visible(tile)
            && !self.IsTileOccludedOnCurrentTree(tile)
    }
    // Source: cc/tiles/picture_layer_tiling.cc:407-412.
    fn MatchesState(&self, twin: &Self) -> bool {
        self.tiling_data_.max_texture_size() == twin.tiling_data_.max_texture_size()
            && self.tiling_rect().origin() == twin.tiling_rect().origin()
    }
    // Source: cc/tiles/picture_layer_tiling.h:367;
    //         cc/tiles/picture_layer_tiling.cc:645-659.
    fn IsTileOccludedOnCurrentTree(&self, tile: &Tile<B>) -> bool {
        if !self.current_occlusion_in_layer_space_.HasOcclusion() {
            return false;
        }
        let bounds = self
            .tiling_data_
            .TileBounds(tile.tiling_i_index(), tile.tiling_j_index());
        let mut query = bounds;
        query.Intersect(self.current_visible_rect_);
        if query.IsEmpty() {
            return false;
        }
        query = self.EnclosingLayerRectFromContentsRect(query);
        self.current_occlusion_in_layer_space_.IsOccluded(query)
    }
    // Source: cc/tiles/picture_layer_tiling.h:371-398.
    pub(crate) fn IsTileOccluded(
        &self,
        tile: &Tile<B>,
        q: &dyn PictureLayerTilingQueries<B>,
    ) -> bool {
        self.IsTileOccludedWithSelected(tile, q, SelectedTile { state: self, tile })
    }
    fn IsTileOccludedWithSelected(
        &self,
        tile: &Tile<B>,
        q: &dyn PictureLayerTilingQueries<B>,
        selected: SelectedTile<'_, B>,
    ) -> bool {
        self.IsTileOccludedWithTwin(tile, || self.Twin(q, selected))
    }
    // Source: cc/tiles/picture_layer_tiling.h:371-398. The source client/twin
    // query stays lazy, after current-tree occlusion and the pending-tree test.
    // A resident traversal can lend the actual opposite-tree tiling without
    // rebuilding a readonly client over its mutably borrowed current Picture.
    fn IsTileOccludedWithTwin<'a>(
        &self,
        tile: &Tile<B>,
        resolve_twin: impl FnOnce() -> Option<TwinRead<'a, B>>,
    ) -> bool
    where
        B: 'a,
    {
        if !self.IsTileOccludedOnCurrentTree(tile) {
            return false;
        }
        if self.tree_ == WhichTree::PENDING_TREE {
            return true;
        }
        if let Some(twin) = resolve_twin() {
            if !self.MatchesState(twin.state()) || twin.HasTileMatching(tile) {
                return true;
            }
            return twin.state().IsTileOccludedOnCurrentTree(tile);
        }
        true
    }
    pub(crate) fn IsTileOccludedWithResolvedTwin<'a>(
        &self,
        tile: &Tile<B>,
        resolve_twin: impl FnOnce() -> Option<&'a PictureLayerTiling<B>>,
    ) -> bool
    where
        B: 'a,
    {
        self.IsTileOccludedWithTwin(tile, || resolve_twin().map(TwinRead::Resident))
    }

    // Source: cc/tiles/picture_layer_tiling.cc:661-692.
    fn ShouldDecodeCheckeredImagesForTile(
        &self,
        tile: &Tile<B>,
        q: &dyn PictureLayerTilingQueries<B>,
        selected: SelectedTile<'_, B>,
    ) -> bool {
        if self.tree_ == WhichTree::PENDING_TREE {
            return !self.IsTileOccludedOnCurrentTree(tile);
        }
        debug_assert_eq!(self.tree_, WhichTree::ACTIVE_TREE);
        let twin = self.Twin(q, selected);
        let Some(twin) = twin else {
            return !self.IsTileOccludedOnCurrentTree(tile);
        };
        if !self.MatchesState(twin.state()) || twin.HasTileMatching(tile) {
            return false;
        }
        !twin.state().IsTileOccludedOnCurrentTree(tile)
    }
    // Source: cc/tiles/picture_layer_tiling.cc:694-700.
    fn UpdateRequiredStatesOnTile(&self, tile: &Tile<B>, q: &dyn PictureLayerTilingQueries<B>) {
        let activation = self.IsRequiredForActivation(
            tile,
            &|t| self.IsTileVisible(t),
            self.IsTileOccluded(tile, q),
            q,
            SelectedTile { state: self, tile },
        );
        tile.set_required_for_activation(activation);
        let draw = self.IsRequiredForDraw(tile, |t| self.IsTileVisible(t));
        tile.set_required_for_draw(draw);
    }
    // Source: cc/tiles/picture_layer_tiling.cc:708-746.
    fn MakePrioritizedTile(
        &self,
        tile: &Tile<B>,
        kind: PriorityRectType,
        is_tile_occluded: bool,
        q: &dyn PictureLayerTilingQueries<B>,
    ) -> PrioritizedTile {
        debug_assert!(self.source().IntersectsRect(*tile.enclosing_layer_rect()));
        let activation = self.IsRequiredForActivation(
            tile,
            &|_| kind == PriorityRectType::VISIBLE_RECT,
            is_tile_occluded,
            q,
            SelectedTile { state: self, tile },
        );
        tile.set_required_for_activation(activation);
        let draw = self.IsRequiredForDraw(tile, |_| kind == PriorityRectType::VISIBLE_RECT);
        tile.set_required_for_draw(draw);
        let priority = self.ComputePriorityForTile(tile, kind, is_tile_occluded, q);
        debug_assert!(
            (!tile.required_for_activation() && !tile.required_for_draw())
                || priority.priority_bin == PriorityBin::NOW
                || !crate::adapter::picture_layer_tiling::WithTilingClient(
                    q,
                    self.client_,
                    |client| client.HasValidTilePriorities()
                )
        );
        let images_only = priority.distance_to_visible > self.min_preraster_distance_
            && (priority.distance_to_visible > self.max_preraster_distance_
                || priority.distance_to_visible > 0.5 * self.max_skewport_extent_in_screen_space_);
        PrioritizedTile::new(
            Some(tile.identity()),
            Some(self.id_),
            &priority,
            is_tile_occluded,
            images_only,
            self.ShouldDecodeCheckeredImagesForTile(tile, q, SelectedTile { state: self, tile }),
        )
    }
    // Source: cc/tiles/picture_layer_tiling.cc:768-807.
    fn ComputePriorityForTile(
        &self,
        tile: &Tile<B>,
        kind: PriorityRectType,
        occluded: bool,
        q: &dyn PictureLayerTilingQueries<B>,
    ) -> TilePriority {
        debug_assert_eq!(self.ComputePriorityRectTypeForTile(tile, q), kind);
        // TileAt equality is checked by the caller's actual map-entry proof.
        let mut bin =
            if crate::adapter::picture_layer_tiling::WithTilingClient(q, self.client_, |client| {
                client.HasValidTilePriorities()
            }) {
                if occluded {
                    PriorityBin::SOON
                } else {
                    PriorityBin::NOW
                }
            } else {
                PriorityBin::EVENTUALLY
            };
        match kind {
            PriorityRectType::VISIBLE_RECT | PriorityRectType::PENDING_VISIBLE_RECT => {
                return TilePriority::new(self.resolution_, bin, 0.);
            }
            PriorityRectType::SKEWPORT_RECT | PriorityRectType::SOON_BORDER_RECT => {
                if bin < PriorityBin::SOON {
                    bin = PriorityBin::SOON;
                }
            }
            PriorityRectType::EVENTUALLY_RECT => bin = PriorityBin::EVENTUALLY,
        }
        let bounds = self
            .tiling_data_
            .TileBounds(tile.tiling_i_index(), tile.tiling_j_index());
        debug_assert!(self.current_content_to_screen_scale_ > 0.);
        let distance = self.current_content_to_screen_scale_
            * self.current_visible_rect_.ManhattanInternalDistance(bounds) as f32;
        TilePriority::new(self.resolution_, bin, distance)
    }
    // Source: cc/tiles/picture_layer_tiling.cc:809-829.
    fn ComputePriorityRectTypeForTile(
        &self,
        tile: &Tile<B>,
        q: &dyn PictureLayerTilingQueries<B>,
    ) -> PriorityRectType {
        let bounds = self
            .tiling_data_
            .TileBounds(tile.tiling_i_index(), tile.tiling_j_index());
        if self.current_visible_rect_.Intersects(bounds) {
            return PriorityRectType::VISIBLE_RECT;
        }
        if self.pending_visible_rect(q).Intersects(bounds) {
            return PriorityRectType::PENDING_VISIBLE_RECT;
        }
        if self.current_skewport_rect_.Intersects(bounds) {
            return PriorityRectType::SKEWPORT_RECT;
        }
        if self.current_soon_border_rect_.Intersects(bounds) {
            return PriorityRectType::SOON_BORDER_RECT;
        }
        debug_assert!(self.current_eventually_rect_.Intersects(bounds));
        PriorityRectType::EVENTUALLY_RECT
    }
    // Source: cc/tiles/picture_layer_tiling.h:407-433.
    pub fn has_visible_rect_tiles(&self) -> bool {
        self.has_visible_rect_tiles_
    }
    pub fn has_skewport_rect_tiles(&self) -> bool {
        self.has_skewport_rect_tiles_
    }
    pub fn has_soon_border_rect_tiles(&self) -> bool {
        self.has_soon_border_rect_tiles_
    }
    pub fn has_eventually_rect_tiles(&self) -> bool {
        self.has_eventually_rect_tiles_
    }
    pub fn current_visible_rect(&self) -> &Rect {
        &self.current_visible_rect_
    }
    pub fn pending_visible_rect(&self, q: &dyn PictureLayerTilingQueries<B>) -> Rect {
        if self.tree_ == WhichTree::ACTIVE_TREE {
            if let Some(id) =
                crate::adapter::picture_layer_tiling::WithTilingClient(q, self.client_, |client| {
                    client.GetPendingOrActiveTwinTiling(self, q, None)
                })
            {
                return *q.TilingById(id).current_visible_rect();
            }
            return Rect::default();
        }
        self.current_visible_rect_
    }
    pub fn current_skewport_rect(&self) -> &Rect {
        &self.current_skewport_rect_
    }
    pub fn current_soon_border_rect(&self) -> &Rect {
        &self.current_soon_border_rect_
    }
    pub fn current_eventually_rect(&self) -> &Rect {
        &self.current_eventually_rect_
    }
    // Source: cc/tiles/picture_layer_tiling.cc:840-862.
    fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        // num_tiles is emitted by the owner, which owns the actual TileMap.
        value.SetDouble("content_scale", self.contents_scale_key() as f64);
        value.BeginDictionary("raster_transform");
        value.BeginArray("scale");
        value.AppendDouble(self.raster_transform_.scale().x() as f64);
        value.AppendDouble(self.raster_transform_.scale().y() as f64);
        value.EndArray();
        value.BeginArray("translation");
        value.AppendDouble(self.raster_transform_.translation().x() as f64);
        value.AppendDouble(self.raster_transform_.translation().y() as f64);
        value.EndArray();
        value.EndDictionary();
        MathUtil::AddToTracedValue("visible_rect", &self.current_visible_rect_, value);
        MathUtil::AddToTracedValue("skewport_rect", &self.current_skewport_rect_, value);
        MathUtil::AddToTracedValue("soon_rect", &self.current_soon_border_rect_, value);
        MathUtil::AddToTracedValue("eventually_rect", &self.current_eventually_rect_, value);
        MathUtil::AddToTracedValue("tiling_rect", self.tiling_rect(), value);
    }
    // Source: cc/tiles/picture_layer_tiling.h:436-439;
    //         cc/tiles/picture_layer_tiling.cc:873-882.
    pub fn EnclosingContentsRectFromLayerRect(&self, rect: Rect) -> Rect {
        ToEnclosingRect(self.raster_transform_.MapRect(RectF::from(rect)))
    }
    pub fn EnclosingLayerRectFromContentsRect(&self, rect: Rect) -> Rect {
        ToEnclosingRect(self.raster_transform_.InverseMapRect(RectF::from(rect)))
    }
    // Source: cc/tiles/picture_layer_tiling.h:441;
    //         cc/tiles/picture_layer_tiling.cc:884-907.
    pub fn ComputeTilingRect(&self) -> Rect {
        let recorded = self.source().recorded_bounds();
        let mut tiling = self.EnclosingContentsRectFromLayerRect(recorded);
        let size = self.source().size();
        let layer = Rect::new(0, 0, size.width(), size.height());
        if recorded != layer {
            let contents = self.EnclosingContentsRectFromLayerRect(layer);
            let snap = 128;
            tiling = Rect::FromBounds(
                MathUtil::UncheckedRoundDown(tiling.x, snap),
                MathUtil::UncheckedRoundDown(tiling.y, snap),
                if tiling.right() + snap > contents.right() {
                    contents.right()
                } else {
                    tiling.right()
                },
                if tiling.bottom() + snap > contents.bottom() {
                    contents.bottom()
                } else {
                    tiling.bottom()
                },
            );
            debug_assert!(contents.Contains(tiling));
        }
        tiling
    }
    // Source: cc/tiles/picture_layer_tiling.h:442;
    //         cc/tiles/picture_layer_tiling.cc:909-921.
    fn SetTilingRect(&mut self, rect: Rect) {
        if *self.tiling_data_.tiling_rect() == rect {
            return;
        }
        self.has_visible_rect_tiles_ = rect.Intersects(self.current_visible_rect_);
        self.has_skewport_rect_tiles_ = rect.Intersects(self.current_skewport_rect_);
        self.has_soon_border_rect_tiles_ = rect.Intersects(self.current_soon_border_rect_);
        self.has_eventually_rect_tiles_ = rect.Intersects(self.current_eventually_rect_);
        self.tiling_data_.SetTilingRect(&rect);
        self.tiling_rect_in_layer_space_ = self.EnclosingLayerRectFromContentsRect(rect);
    }
    // Source: cc/tiles/picture_layer_tiling.cc:444-448,487-493.
    fn SkewportExtent(&self) -> i32 {
        [
            self.current_visible_rect_.x - self.current_skewport_rect_.x,
            self.current_skewport_rect_.right() - self.current_visible_rect_.right(),
            self.current_visible_rect_.y - self.current_skewport_rect_.y,
            self.current_skewport_rect_.bottom() - self.current_visible_rect_.bottom(),
        ]
        .into_iter()
        .max()
        .unwrap()
    }
}
// Source: cc/tiles/picture_layer_tiling.h:226-240;
//         cc/tiles/picture_layer_tiling.cc:923-939.
// The native cursor directly borrows actual map entries, never key snapshots.
pub struct TileIterator<'a, B: PictureLayerTilingTypes> {
    iter_: std::collections::hash_map::IterMut<'a, TileIndex, Option<Box<Tile<B>>>>,
    current_: Option<&'a mut Option<Box<Tile<B>>>>,
}
impl<'a, B: PictureLayerTilingTypes> TileIterator<'a, B> {
    pub fn new(tiling: &'a mut PictureLayerTiling<B>) -> Self {
        Self::FromMap(&mut tiling.tiles_)
    }
    // Source: cc/tiles/picture_layer_tiling.cc:923-924.
    // Shared constructor body permits a borrow of the same owner's state/map.
    fn FromMap(tiles: &'a mut TileMap<B>) -> Self {
        let mut iter = tiles.iter_mut();
        let current = iter.next().map(|(_, tile)| tile);
        Self {
            iter_: iter,
            current_: current,
        }
    }
    pub fn GetCurrent(&mut self) -> Option<&mut Tile<B>> {
        self.current_.as_mut().and_then(|t| t.as_deref_mut())
    }
    pub fn Next(&mut self) {
        if !self.AtEnd() {
            self.current_ = self.iter_.next().map(|(_, tile)| tile);
        }
    }
    pub fn AtEnd(&self) -> bool {
        self.current_.is_none()
    }
}

// Source: cc/tiles/picture_layer_tiling.h:133-142. These are resident source
// values; client getters resolve the same native call-scoped client.
impl<B: PictureLayerTilingTypes> crate::adapter::prioritized_tile::PrioritizedTileTiling
    for PictureLayerTiling<B>
{
    type Types = B;
    type RasterSource = B::RasterSource;
    type PaintWorkletRecordMap = B::PaintWorkletRecordMap;
    fn raster_source(&self) -> &Option<Arc<B::RasterSource>> {
        self.state_.raster_source()
    }
}
impl<B: PictureLayerTilingTypes, C: PictureLayerTilingQueries<B> + ?Sized>
    crate::adapter::prioritized_tile::PrioritizedTileContext<PictureLayerTiling<B>> for C
{
    fn TileById(&self, id: TileId) -> &Tile<B> {
        PictureLayerTilingQueries::TilingById(self, id.tiling).TileById(id)
    }
    fn PrioritizedTilingById(&self, id: PictureLayerTilingId) -> &PictureLayerTiling<B> {
        PictureLayerTilingQueries::TilingById(self, id)
    }
    fn GetPaintWorkletRecords<'a>(
        &'a self,
        id: PictureLayerTilingId,
    ) -> &'a B::PaintWorkletRecordMap
    where
        PictureLayerTiling<B>: 'a,
        B: 'a,
    {
        let tiling = PictureLayerTilingQueries::TilingById(self, id);
        self.PaintWorkletRecordsForClient(tiling.client())
    }
    fn GetRasterInducingScrollOffsets(&self, id: PictureLayerTilingId) -> ScrollOffsetMap {
        let tiling = PictureLayerTilingQueries::TilingById(self, id);
        crate::adapter::picture_layer_tiling::WithTilingClient(self, tiling.client(), |client| {
            client.GetRasterInducingScrollOffsets()
        })
    }
}

// Source: cc/tiles/picture_layer_tiling.h:75-76,125,259-261,487-491.
impl<B: PictureLayerTilingTypes> super::tiling_set_coverage_iterator::TilingWithCover
    for PictureLayerTiling<B>
{
    fn resolution(&self) -> TileResolution {
        self.state_.resolution()
    }
}

impl<B: PictureLayerTilingTypes> PictureLayerTiling<B> {
    // Source: cc/tiles/picture_layer_tiling.h:226-240;
    //         cc/tiles/picture_layer_tiling.cc:923-924.
    // Rust borrow split: both references name fields of this same resident.
    pub(crate) fn StateAndTileIterator(
        &mut self,
    ) -> (&PictureLayerTilingState<B>, TileIterator<'_, B>) {
        (&self.state_, TileIterator::FromMap(&mut self.tiles_))
    }
}

use crate::adapter::picture_layer_tiling::PictureLayerTileDestroyer;

// Same-package scoped native pending pointer. No owner/map/state copy is made;
// the immutable borrow ends before the next source mutation of pending tiles.
struct BorrowedTwinTilingContext<'a, B: PictureLayerTilingTypes> {
    external: &'a mut dyn PictureLayerTilingContext<B>,
    twin: &'a PictureLayerTiling<B>,
}
impl<B: PictureLayerTilingTypes> PictureLayerTilingQueries<B> for BorrowedTwinTilingContext<'_, B> {
    fn WithClient(
        &self,
        id: PictureLayerTilingClientId,
        callback: &mut dyn FnMut(&dyn PictureLayerTilingClient<B>),
    ) {
        self.external.WithClient(id, callback)
    }
    fn PaintWorkletRecordsForClient(
        &self,
        id: PictureLayerTilingClientId,
    ) -> &B::PaintWorkletRecordMap {
        self.external.PaintWorkletRecordsForClient(id)
    }
    fn TilingById(&self, id: PictureLayerTilingId) -> &PictureLayerTiling<B> {
        if id == self.twin.identity() {
            self.twin
        } else {
            self.external.TilingById(id)
        }
    }
    fn IsClientOnActiveOrPendingTree(&self, id: PictureLayerTilingClientId) -> bool {
        self.external.IsClientOnActiveOrPendingTree(id)
    }
    fn TilingStateForClientAndScale(
        &self,
        id: PictureLayerTilingClientId,
        scale: f32,
    ) -> Option<&PictureLayerTilingState<B>> {
        if id == self.twin.client() && scale == self.twin.contents_scale_key() {
            Some(&self.twin.state_)
        } else {
            self.external.TilingStateForClientAndScale(id, scale)
        }
    }
    fn PendingInvalidationById(&self, id: PendingInvalidationId) -> &Region<B::RegionBackend> {
        self.external.PendingInvalidationById(id)
    }
}
impl<B: PictureLayerTilingTypes> crate::adapter::picture_layer_tiling::PictureLayerTileDestroyer<B>
    for BorrowedTwinTilingContext<'_, B>
{
    fn RebindTileWithManager(
        &mut self,
        old: crate::adapter::tile::TileId,
        new: crate::adapter::tile::TileId,
    ) {
        self.external.RebindTileWithManager(old, new)
    }
    fn DestroyTileWithManager(
        &mut self,
        id: crate::adapter::tile::TileManagerId,
        tile: &mut Tile<B>,
        tiling: &crate::cc::tiles::picture_layer_tiling::PictureLayerTilingState<B>,
    ) {
        self.external.DestroyTileWithManager(id, tile, tiling)
    }
}
impl<B: PictureLayerTilingTypes> PictureLayerTilingContext<B> for BorrowedTwinTilingContext<'_, B> {
    fn CalculateTileSizeForClient(&mut self, id: PictureLayerTilingClientId, bounds: Size) -> Size {
        self.external.CalculateTileSizeForClient(id, bounds)
    }
    fn CreateTileForClient(
        &mut self,
        id: PictureLayerTilingClientId,
        info: &CreateInfo<B::RasterTransform>,
    ) -> Option<Box<Tile<B>>> {
        self.external.CreateTileForClient(id, info)
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.external.IsCCSlimmingEnabled()
    }
}
