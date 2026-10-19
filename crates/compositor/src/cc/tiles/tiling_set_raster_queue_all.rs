// Translation status: complete
use crate::adapter::picture_layer_tiling::PictureLayerTilingTypes;
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/tiling_set_raster_queue_all.h:1-28,32-37,43-44,56-58,73-78,171-183,216-225;
//         cc/tiles/tiling_set_raster_queue_all.cc:1-17,509.
use super::{
    prioritized_tile::PrioritizedTile,
    raster_tile_priority_queue::{
        PictureLayerTilingSetId, PriorityRectType as Phase, RasterQueueContext,
    },
    tile_priority::{PriorityBin, TileResolution, WhichTree},
};
use crate::{
    adapter::{
        geometry::Rect,
        property_types::InlinedVector,
        tile::{PictureLayerTilingId, TileId},
    },
    cc::base::tiling_data::{DifferenceIterator, Iterator, SpiralDifferenceIterator},
};
use std::marker::PhantomData;

// Source: cc/tiles/tiling_set_raster_queue_all.h:194-198.
#[derive(Clone, Copy)]
#[repr(usize)]
enum IteratorType {
    HIGH_RES = 0,
    ACTIVE_NON_IDEAL_PENDING_HIGH_RES = 1,
}
const NUM_ITERATORS: usize = 2;
// Source: cc/tiles/tiling_set_raster_queue_all.h:208-212;
//         cc/tiles/tiling_set_raster_queue_all.cc:19-23.
#[derive(Clone, Copy)]
struct IterationStage {
    iterator_type: IteratorType,
    tile_type: PriorityBin,
}
impl IterationStage {
    fn new(iterator_type: IteratorType, tile_type: PriorityBin) -> Self {
        Self {
            iterator_type,
            tile_type,
        }
    }
}
// InlinedVector's spare backing slots need inert values, never observed stages.
impl Default for IterationStage {
    fn default() -> Self {
        Self::new(IteratorType::HIGH_RES, PriorityBin::NOW)
    }
}

// Source: cc/tiles/tiling_set_raster_queue_all.h:200-220.
// Each ID maps a native non-owning pointer; native current entries remain values.
pub(crate) struct TilingSetRasterQueueAll<B: PictureLayerTilingTypes> {
    current_stage_: usize,
    stages_: InlinedVector<IterationStage, 5>,
    iterators_: [Option<TilingIterator<B>>; NUM_ITERATORS],
    is_drawing_layer_: bool,
}
impl<B: PictureLayerTilingTypes> TilingSetRasterQueueAll<B> {
    // Source: cc/tiles/tiling_set_raster_queue_all.h:29-31;
    //         cc/tiles/tiling_set_raster_queue_all.cc:25-82.
    pub(crate) fn Create(
        set: PictureLayerTilingSetId,
        is_drawing_layer: bool,
        context: &dyn RasterQueueContext<B>,
    ) -> Option<Box<Self>> {
        if context.IsCCSlimmingEnabled() {
            if context.TilingSetById(set).all_tiles_done() {
                return None;
            }
        } else {
            if context.TilingSetById(set).num_tilings() == 0 {
                return Some(Box::new(Self::new(None, None, is_drawing_layer, context)));
            }
        }
        let tree = context.TilingSetById(set).tree();
        let mut high_res_tiling = None;
        let mut active_non_ideal_pending_high_res_tiling = None;
        let mut i = 0;
        while i < context.TilingSetById(set).num_tilings() {
            let tiling = context.TilingSetById(set).tiling_at(i).identity();
            if context.TilingById(tiling).resolution() == TileResolution::HIGH_RESOLUTION {
                high_res_tiling = Some(tiling);
            }
            if tree == WhichTree::ACTIVE_TREE
                && context.TilingById(tiling).resolution() != TileResolution::HIGH_RESOLUTION
            {
                let twin = crate::adapter::picture_layer_tiling::WithTilingClient(
                    context,
                    context.TilingSetById(set).client(),
                    |client| {
                        client.GetPendingOrActiveTwinTiling(
                            context.TilingById(tiling),
                            context,
                            None,
                        )
                    },
                );
                if twin.is_some_and(|t| {
                    context.TilingById(t).resolution() == TileResolution::HIGH_RESOLUTION
                }) {
                    active_non_ideal_pending_high_res_tiling = Some(tiling);
                }
            }
            i += 1;
        }
        let use_high_res_tiling = high_res_tiling.is_some_and(|t| {
            context.TilingById(t).has_tiles() && !context.TilingById(t).all_tiles_done()
        });
        let use_active_non_ideal_pending_high_res_tiling = active_non_ideal_pending_high_res_tiling
            .is_some_and(|t| {
                context.TilingById(t).has_tiles() && !context.TilingById(t).all_tiles_done()
            });
        if !use_high_res_tiling
            && !use_active_non_ideal_pending_high_res_tiling
            && context.IsCCSlimmingEnabled()
        {
            return None;
        }
        Some(Box::new(Self::new(
            if use_high_res_tiling {
                high_res_tiling
            } else {
                None
            },
            if use_active_non_ideal_pending_high_res_tiling {
                active_non_ideal_pending_high_res_tiling
            } else {
                None
            },
            is_drawing_layer,
            context,
        )))
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.h:200-203;
    //         cc/tiles/tiling_set_raster_queue_all.cc:84-132.
    fn new(
        high_res: Option<PictureLayerTilingId>,
        non_ideal: Option<PictureLayerTilingId>,
        drawing: bool,
        context: &dyn RasterQueueContext<B>,
    ) -> Self {
        let mut result = Self {
            current_stage_: 0,
            stages_: InlinedVector::default(),
            iterators_: std::array::from_fn(|_| None),
            is_drawing_layer_: drawing,
        };
        if high_res.is_none() && non_ideal.is_none() {
            debug_assert!(!context.IsCCSlimmingEnabled());
            return result;
        }
        if let Some(tiling) = high_res {
            result.MakeTilingIterator(IteratorType::HIGH_RES, tiling, context);
        } else if !context.IsCCSlimmingEnabled() {
            result.iterators_[IteratorType::HIGH_RES as usize] = Some(TilingIterator::default());
        }
        if let Some(tiling) = non_ideal {
            result.MakeTilingIterator(
                IteratorType::ACTIVE_NON_IDEAL_PENDING_HIGH_RES,
                tiling,
                context,
            );
        } else if !context.IsCCSlimmingEnabled() {
            result.iterators_[IteratorType::ACTIVE_NON_IDEAL_PENDING_HIGH_RES as usize] =
                Some(TilingIterator::default());
        }
        if high_res.is_some() {
            result.stages_.push(IterationStage::new(
                IteratorType::HIGH_RES,
                PriorityBin::NOW,
            ));
        }
        if non_ideal.is_some() {
            result.stages_.push(IterationStage::new(
                IteratorType::ACTIVE_NON_IDEAL_PENDING_HIGH_RES,
                PriorityBin::NOW,
            ));
            result.stages_.push(IterationStage::new(
                IteratorType::ACTIVE_NON_IDEAL_PENDING_HIGH_RES,
                PriorityBin::SOON,
            ));
        }
        if high_res.is_some() {
            result.stages_.push(IterationStage::new(
                IteratorType::HIGH_RES,
                PriorityBin::SOON,
            ));
            result.stages_.push(IterationStage::new(
                IteratorType::HIGH_RES,
                PriorityBin::EVENTUALLY,
            ));
        }
        debug_assert!(!result.stages_.is_empty());
        let index = result.stages_[result.current_stage_].iterator_type as usize;
        let tile_type = result.stages_[result.current_stage_].tile_type;
        if result.iterators_[index]
            .as_ref()
            .is_none_or(|i| i.done() || i.r#type() != tile_type)
        {
            result.AdvanceToNextStage();
        }
        result
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.h:34;
    //         cc/tiles/tiling_set_raster_queue_all.cc:134.
    // Default destructor drops no resident tiling/tile; no Clone maps deleted copy.
    // Source: cc/tiles/tiling_set_raster_queue_all.h:205;
    //         cc/tiles/tiling_set_raster_queue_all.cc:136-144.
    fn MakeTilingIterator(
        &mut self,
        r#type: IteratorType,
        tiling: PictureLayerTilingId,
        context: &dyn RasterQueueContext<B>,
    ) {
        self.iterators_[r#type as usize] = Some(TilingIterator::new(tiling, context));
        if self.iterators_[r#type as usize].as_ref().unwrap().done() {
            context.TilingById(tiling).set_all_tiles_done(true);
            context.TilingById(tiling).VerifyNoTileNeedsRaster(context);
        }
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.h:40;
    //         cc/tiles/tiling_set_raster_queue_all.cc:146-148.
    pub(crate) fn IsEmpty(&self) -> bool {
        self.current_stage_ >= self.stages_.len()
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.h:39;
    //         cc/tiles/tiling_set_raster_queue_all.cc:150-164.
    pub(crate) fn Pop(&mut self, context: &dyn RasterQueueContext<B>) {
        let index = self.stages_[self.current_stage_].iterator_type as usize;
        let tile_type = self.stages_[self.current_stage_].tile_type;
        debug_assert!(self.iterators_[index].is_some());
        debug_assert!(!self.iterators_[index].as_ref().unwrap().done());
        debug_assert!(self.iterators_[index].as_ref().unwrap().r#type() == tile_type);
        self.iterators_[index].as_mut().unwrap().Advance(context);
        if self.iterators_[index]
            .as_ref()
            .is_none_or(|i| i.done() || i.r#type() != tile_type)
        {
            self.AdvanceToNextStage();
        }
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.h:38;
    //         cc/tiles/tiling_set_raster_queue_all.cc:166-175.
    pub(crate) fn Top(&self) -> &PrioritizedTile {
        debug_assert!(!self.IsEmpty());
        let index = self.stages_[self.current_stage_].iterator_type as usize;
        debug_assert!(self.iterators_[index].is_some());
        debug_assert!(!self.iterators_[index].as_ref().unwrap().done());
        debug_assert!(
            self.iterators_[index].as_ref().unwrap().r#type()
                == self.stages_[self.current_stage_].tile_type
        );
        self.iterators_[index].as_ref().unwrap().get()
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.h:41.
    pub(crate) fn is_drawing_layer(&self) -> bool {
        self.is_drawing_layer_
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.h:206;
    //         cc/tiles/tiling_set_raster_queue_all.cc:177-190.
    fn AdvanceToNextStage(&mut self) {
        debug_assert!(self.current_stage_ < self.stages_.len());
        self.current_stage_ += 1;
        while self.current_stage_ < self.stages_.len() {
            let index = self.stages_[self.current_stage_].iterator_type as usize;
            let tile_type = self.stages_[self.current_stage_].tile_type;
            if self.iterators_[index]
                .as_ref()
                .is_some_and(|i| !i.done() && i.r#type() == tile_type)
            {
                break;
            }
            self.current_stage_ += 1;
        }
    }
}

// Source: cc/tiles/tiling_set_raster_queue_all.h:59-62.
// Native iterator concept: these bridges call the already canonical cursor.
trait TilingIteratorType {
    fn Advance(&mut self);
    fn cursor_valid(&self) -> bool;
    fn cursor_x(&self) -> i32;
    fn cursor_y(&self) -> i32;
}
macro_rules! native_iterator {
    ($t:ty) => {
        impl TilingIteratorType for $t {
            fn Advance(&mut self) {
                <$t>::Advance(self);
            }
            fn cursor_valid(&self) -> bool {
                self.IsValid()
            }
            fn cursor_x(&self) -> i32 {
                self.index_x()
            }
            fn cursor_y(&self) -> i32 {
                self.index_y()
            }
        }
    };
}
native_iterator!(Iterator);
native_iterator!(DifferenceIterator);
native_iterator!(SpiralDifferenceIterator);

// Source: cc/tiles/tiling_set_raster_queue_all.h:64-68.
#[derive(Clone, Copy, PartialEq, Eq)]
enum IsTileValidResult {
    kTileNotValid,
    kTileNeedsRaster,
    kTileNeedsCheckerImageReraster,
}
use IsTileValidResult::*;
// Source: cc/tiles/tiling_set_raster_queue_all.h:45-81.
struct OnePriorityRectIterator<B: PictureLayerTilingTypes> {
    current_tile_: PrioritizedTile,
    tiling_: Option<PictureLayerTilingId>,
    priority_rect_type_: Option<Phase>,
    pending_visible_rect_: Rect,
    source_type_: PhantomData<fn() -> B>,
}
impl<B: PictureLayerTilingTypes> Default for OnePriorityRectIterator<B> {
    // Source: cc/tiles/tiling_set_raster_queue_all.h:47;
    //         cc/tiles/tiling_set_raster_queue_all.cc:192-195.
    fn default() -> Self {
        Self {
            current_tile_: PrioritizedTile::default(),
            tiling_: None,
            priority_rect_type_: None,
            pending_visible_rect_: Rect::default(),
            source_type_: PhantomData,
        }
    }
}
impl<B: PictureLayerTilingTypes> OnePriorityRectIterator<B> {
    // Source: cc/tiles/tiling_set_raster_queue_all.h:48-51;
    //         cc/tiles/tiling_set_raster_queue_all.cc:197-205.
    fn new(
        tiling: PictureLayerTilingId,
        priority_rect_type: Phase,
        context: &dyn RasterQueueContext<B>,
    ) -> Self {
        Self {
            current_tile_: PrioritizedTile::default(),
            tiling_: Some(tiling),
            priority_rect_type_: Some(priority_rect_type),
            pending_visible_rect_: context.TilingById(tiling).pending_visible_rect(context),
            source_type_: PhantomData,
        }
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.h:53-54.
    fn done(&self) -> bool {
        self.current_tile_.tile_id().is_none()
    }
    fn get(&self) -> &PrioritizedTile {
        &self.current_tile_
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.h:59-60;
    //         cc/tiles/tiling_set_raster_queue_all.cc:207-227.
    fn AdvanceToNextTile<I: TilingIteratorType>(
        &mut self,
        iterator: &mut I,
        context: &dyn RasterQueueContext<B>,
    ) {
        loop {
            iterator.Advance();
            if !iterator.cursor_valid() {
                self.current_tile_ = PrioritizedTile::default();
                break;
            }
            let tiling = self.tiling_.expect("source nonnull tiling");
            let tile = context
                .TilingById(tiling)
                .TileAt(iterator.cursor_x(), iterator.cursor_y())
                .map(|tile| tile.identity());
            let result = self.IsTileValid(tile, context);
            if result == kTileNotValid {
                continue;
            }
            let tile = tile.expect("source valid tile");
            let is_tile_occluded = result != kTileNeedsRaster
                && context
                    .TilingById(tiling)
                    .IsTileOccluded(context.TileById(tile), context);
            self.current_tile_ = super::raster_tile_priority_queue::MakePrioritizedTile(
                context,
                tiling,
                tile,
                self.priority_rect_type_.unwrap(),
                is_tile_occluded,
            );
            break;
        }
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.h:61-62;
    //         cc/tiles/tiling_set_raster_queue_all.cc:229-244.
    fn GetFirstTileAndCheckIfValid<I: TilingIteratorType>(
        &mut self,
        iterator: &mut I,
        context: &dyn RasterQueueContext<B>,
    ) -> bool {
        let tiling = self.tiling_.expect("source nonnull tiling");
        let tile = context
            .TilingById(tiling)
            .TileAt(iterator.cursor_x(), iterator.cursor_y())
            .map(|tile| tile.identity());
        let result = self.IsTileValid(tile, context);
        if result == kTileNotValid {
            self.current_tile_ = PrioritizedTile::default();
            return false;
        }
        let tile = tile.expect("source valid tile");
        let is_tile_occluded = result != kTileNeedsRaster
            && context
                .TilingById(tiling)
                .IsTileOccluded(context.TileById(tile), context);
        self.current_tile_ = super::raster_tile_priority_queue::MakePrioritizedTile(
            context,
            tiling,
            tile,
            self.priority_rect_type_.unwrap(),
            is_tile_occluded,
        );
        true
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.h:69;
    //         cc/tiles/tiling_set_raster_queue_all.cc:246-279.
    fn IsTileValid(
        &self,
        tile: Option<TileId>,
        context: &dyn RasterQueueContext<B>,
    ) -> IsTileValidResult {
        let Some(tile) = tile else {
            return kTileNotValid;
        };
        let tiling = self.tiling_.expect("source nonnull tiling");
        let tile_is_valid_for_raster = context.TileById(tile).draw_info().NeedsRaster()
            && !context
                .TilingById(tiling)
                .IsTileOccluded(context.TileById(tile), context);
        let mut result = kTileNeedsRaster;
        if !tile_is_valid_for_raster {
            let tile_is_valid_for_checker_images =
                context.TileById(tile).draw_info().is_checker_imaged()
                    && context
                        .TilingById(tiling)
                        .ShouldDecodeCheckeredImagesForTile(context.TileById(tile), context);
            if !tile_is_valid_for_checker_images {
                return kTileNotValid;
            }
            result = kTileNeedsCheckerImageReraster;
        }
        if self.priority_rect_type_.unwrap() > Phase::PENDING_VISIBLE_RECT {
            let tile_bounds = context.TilingById(tiling).tiling_data().TileBounds(
                context.TileById(tile).tiling_i_index(),
                context.TileById(tile).tiling_j_index(),
            );
            if self.pending_visible_rect_.Intersects(tile_bounds) {
                return kTileNotValid;
            }
        }
        result
    }
}

// Source: cc/tiles/tiling_set_raster_queue_all.h:83-93.
struct VisibleTilingIterator<B: PictureLayerTilingTypes> {
    base: OnePriorityRectIterator<B>,
    iterator_: Iterator,
}
impl<B: PictureLayerTilingTypes> Default for VisibleTilingIterator<B> {
    fn default() -> Self {
        Self {
            base: OnePriorityRectIterator::default(),
            iterator_: Iterator::default(),
        }
    }
}
impl<B: PictureLayerTilingTypes> std::ops::Deref for VisibleTilingIterator<B> {
    type Target = OnePriorityRectIterator<B>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl<B: PictureLayerTilingTypes> VisibleTilingIterator<B> {
    // Source: cc/tiles/tiling_set_raster_queue_all.cc:281-297.
    fn new(id: PictureLayerTilingId, context: &dyn RasterQueueContext<B>) -> Self {
        let mut result = Self {
            base: OnePriorityRectIterator::new(id, Phase::VISIBLE_RECT, context),
            iterator_: Iterator::default(),
        };
        if !context.TilingById(id).has_visible_rect_tiles() {
            return result;
        }
        let tiling = context.TilingById(id);
        result.iterator_ =
            Iterator::new(tiling.tiling_data(), tiling.current_visible_rect(), false);
        if !result.iterator_.IsValid() {
            return result;
        }
        if !result
            .base
            .GetFirstTileAndCheckIfValid(&mut result.iterator_, context)
        {
            result.Advance(context);
        }
        result
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.cc:299-304.
    fn Advance(&mut self, context: &dyn RasterQueueContext<B>) -> &mut Self {
        self.base.AdvanceToNextTile(&mut self.iterator_, context);
        self
    }
}

// Source: cc/tiles/tiling_set_raster_queue_all.h:95-105.
struct PendingVisibleTilingIterator<B: PictureLayerTilingTypes> {
    base: OnePriorityRectIterator<B>,
    iterator_: DifferenceIterator,
}
impl<B: PictureLayerTilingTypes> Default for PendingVisibleTilingIterator<B> {
    fn default() -> Self {
        Self {
            base: OnePriorityRectIterator::default(),
            iterator_: DifferenceIterator::default(),
        }
    }
}
impl<B: PictureLayerTilingTypes> std::ops::Deref for PendingVisibleTilingIterator<B> {
    type Target = OnePriorityRectIterator<B>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl<B: PictureLayerTilingTypes> PendingVisibleTilingIterator<B> {
    // Source: cc/tiles/tiling_set_raster_queue_all.cc:306-319.
    fn new(id: PictureLayerTilingId, context: &dyn RasterQueueContext<B>) -> Self {
        let mut result = Self {
            base: OnePriorityRectIterator::new(id, Phase::PENDING_VISIBLE_RECT, context),
            iterator_: DifferenceIterator::default(),
        };
        let tiling = context.TilingById(id);
        result.iterator_ = DifferenceIterator::new(
            tiling.tiling_data(),
            &result.base.pending_visible_rect_,
            tiling.current_visible_rect(),
        );
        if !result.iterator_.IsValid() {
            return result;
        }
        if !result
            .base
            .GetFirstTileAndCheckIfValid(&mut result.iterator_, context)
        {
            result.Advance(context);
        }
        result
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.cc:321-326.
    fn Advance(&mut self, context: &dyn RasterQueueContext<B>) -> &mut Self {
        self.base.AdvanceToNextTile(&mut self.iterator_, context);
        self
    }
}

// Source: cc/tiles/tiling_set_raster_queue_all.h:107-117.
struct SkewportTilingIterator<B: PictureLayerTilingTypes> {
    base: OnePriorityRectIterator<B>,
    iterator_: SpiralDifferenceIterator,
}
impl<B: PictureLayerTilingTypes> Default for SkewportTilingIterator<B> {
    fn default() -> Self {
        Self {
            base: OnePriorityRectIterator::default(),
            iterator_: SpiralDifferenceIterator::default(),
        }
    }
}
impl<B: PictureLayerTilingTypes> std::ops::Deref for SkewportTilingIterator<B> {
    type Target = OnePriorityRectIterator<B>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl<B: PictureLayerTilingTypes> SkewportTilingIterator<B> {
    // Source: cc/tiles/tiling_set_raster_queue_all.cc:328-346.
    fn new(id: PictureLayerTilingId, context: &dyn RasterQueueContext<B>) -> Self {
        let mut result = Self {
            base: OnePriorityRectIterator::new(id, Phase::SKEWPORT_RECT, context),
            iterator_: SpiralDifferenceIterator::default(),
        };
        if !context.TilingById(id).has_skewport_rect_tiles() {
            return result;
        }
        let tiling = context.TilingById(id);
        result.iterator_ = SpiralDifferenceIterator::new(
            tiling.tiling_data(),
            tiling.current_skewport_rect(),
            tiling.current_visible_rect(),
            tiling.current_visible_rect(),
        );
        if !result.iterator_.IsValid() {
            return result;
        }
        if !result
            .base
            .GetFirstTileAndCheckIfValid(&mut result.iterator_, context)
        {
            result.Advance(context);
            return result;
        }
        result
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.cc:348-353.
    fn Advance(&mut self, context: &dyn RasterQueueContext<B>) -> &mut Self {
        self.base.AdvanceToNextTile(&mut self.iterator_, context);
        self
    }
}

// Source: cc/tiles/tiling_set_raster_queue_all.h:119-130.
struct SoonBorderTilingIterator<B: PictureLayerTilingTypes> {
    base: OnePriorityRectIterator<B>,
    iterator_: SpiralDifferenceIterator,
}
impl<B: PictureLayerTilingTypes> Default for SoonBorderTilingIterator<B> {
    fn default() -> Self {
        Self {
            base: OnePriorityRectIterator::default(),
            iterator_: SpiralDifferenceIterator::default(),
        }
    }
}
impl<B: PictureLayerTilingTypes> std::ops::Deref for SoonBorderTilingIterator<B> {
    type Target = OnePriorityRectIterator<B>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl<B: PictureLayerTilingTypes> SoonBorderTilingIterator<B> {
    // Source: cc/tiles/tiling_set_raster_queue_all.cc:355-373.
    fn new(id: PictureLayerTilingId, context: &dyn RasterQueueContext<B>) -> Self {
        let mut result = Self {
            base: OnePriorityRectIterator::new(id, Phase::SOON_BORDER_RECT, context),
            iterator_: SpiralDifferenceIterator::default(),
        };
        if !context.TilingById(id).has_soon_border_rect_tiles() {
            return result;
        }
        let tiling = context.TilingById(id);
        result.iterator_ = SpiralDifferenceIterator::new(
            tiling.tiling_data(),
            tiling.current_soon_border_rect(),
            tiling.current_skewport_rect(),
            tiling.current_visible_rect(),
        );
        if !result.iterator_.IsValid() {
            return result;
        }
        if !result
            .base
            .GetFirstTileAndCheckIfValid(&mut result.iterator_, context)
        {
            result.Advance(context);
            return result;
        }
        result
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.cc:375-380.
    fn Advance(&mut self, context: &dyn RasterQueueContext<B>) -> &mut Self {
        self.base.AdvanceToNextTile(&mut self.iterator_, context);
        self
    }
}

// Source: cc/tiles/tiling_set_raster_queue_all.h:132-143.
struct EventuallyTilingIterator<B: PictureLayerTilingTypes> {
    base: OnePriorityRectIterator<B>,
    iterator_: SpiralDifferenceIterator,
}
impl<B: PictureLayerTilingTypes> Default for EventuallyTilingIterator<B> {
    fn default() -> Self {
        Self {
            base: OnePriorityRectIterator::default(),
            iterator_: SpiralDifferenceIterator::default(),
        }
    }
}
impl<B: PictureLayerTilingTypes> std::ops::Deref for EventuallyTilingIterator<B> {
    type Target = OnePriorityRectIterator<B>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl<B: PictureLayerTilingTypes> EventuallyTilingIterator<B> {
    // Source: cc/tiles/tiling_set_raster_queue_all.cc:382-400.
    fn new(id: PictureLayerTilingId, context: &dyn RasterQueueContext<B>) -> Self {
        let mut result = Self {
            base: OnePriorityRectIterator::new(id, Phase::EVENTUALLY_RECT, context),
            iterator_: SpiralDifferenceIterator::default(),
        };
        if !context.TilingById(id).has_eventually_rect_tiles() {
            return result;
        }
        let tiling = context.TilingById(id);
        result.iterator_ = SpiralDifferenceIterator::new(
            tiling.tiling_data(),
            tiling.current_eventually_rect(),
            tiling.current_skewport_rect(),
            tiling.current_soon_border_rect(),
        );
        if !result.iterator_.IsValid() {
            return result;
        }
        if !result
            .base
            .GetFirstTileAndCheckIfValid(&mut result.iterator_, context)
        {
            result.Advance(context);
            return result;
        }
        result
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.cc:402-407.
    fn Advance(&mut self, context: &dyn RasterQueueContext<B>) -> &mut Self {
        self.base.AdvanceToNextTile(&mut self.iterator_, context);
        self
    }
}

// Source: cc/tiles/tiling_set_raster_queue_all.h:145-192.
// tiling_data_ is the tiling's own resident data, resolved via the same ID.
struct TilingIterator<B: PictureLayerTilingTypes> {
    tiling_: Option<PictureLayerTilingId>,
    phase_: Option<Phase>,
    current_tile_: PrioritizedTile,
    visible_iterator_: VisibleTilingIterator<B>,
    pending_visible_iterator_: PendingVisibleTilingIterator<B>,
    skewport_iterator_: SkewportTilingIterator<B>,
    soon_border_iterator_: SoonBorderTilingIterator<B>,
    eventually_iterator_: EventuallyTilingIterator<B>,
}
impl<B: PictureLayerTilingTypes> Default for TilingIterator<B> {
    // Source: cc/tiles/tiling_set_raster_queue_all.h:149;
    //         cc/tiles/tiling_set_raster_queue_all.cc:409-411.
    fn default() -> Self {
        Self {
            tiling_: None,
            phase_: None,
            current_tile_: PrioritizedTile::default(),
            visible_iterator_: VisibleTilingIterator::default(),
            pending_visible_iterator_: PendingVisibleTilingIterator::default(),
            skewport_iterator_: SkewportTilingIterator::default(),
            soon_border_iterator_: SoonBorderTilingIterator::default(),
            eventually_iterator_: EventuallyTilingIterator::default(),
        }
    }
}
impl<B: PictureLayerTilingTypes> TilingIterator<B> {
    // Source: cc/tiles/tiling_set_raster_queue_all.h:150;
    //         cc/tiles/tiling_set_raster_queue_all.cc:413-423.
    fn new(tiling: PictureLayerTilingId, context: &dyn RasterQueueContext<B>) -> Self {
        let mut result = Self {
            tiling_: Some(tiling),
            phase_: Some(Phase::VISIBLE_RECT),
            ..Self::default()
        };
        result.visible_iterator_ = VisibleTilingIterator::new(tiling, context);
        if result.visible_iterator_.done() {
            result.AdvancePhase(context);
            return result;
        }
        result.current_tile_ = result.visible_iterator_.get().clone();
        result
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.h:151;
    //         cc/tiles/tiling_set_raster_queue_all.cc:425.
    // Native default destructor: all fields own values, no resident owners.
    // Source: cc/tiles/tiling_set_raster_queue_all.h:153-154.
    fn done(&self) -> bool {
        self.current_tile_.tile_id().is_none()
    }
    fn get(&self) -> &PrioritizedTile {
        &self.current_tile_
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.h:155-167.
    fn r#type(&self) -> PriorityBin {
        match self.phase_.expect("source initialized phase") {
            Phase::VISIBLE_RECT => PriorityBin::NOW,
            Phase::PENDING_VISIBLE_RECT | Phase::SKEWPORT_RECT | Phase::SOON_BORDER_RECT => {
                PriorityBin::SOON
            }
            Phase::EVENTUALLY_RECT => PriorityBin::EVENTUALLY,
        }
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.h:174;
    //         cc/tiles/tiling_set_raster_queue_all.cc:427-459.
    fn AdvancePhase(&mut self, context: &dyn RasterQueueContext<B>) {
        debug_assert!(self.phase_.unwrap() < Phase::EVENTUALLY_RECT);
        self.current_tile_ = PrioritizedTile::default();
        while self.current_tile_.tile_id().is_none()
            && self.phase_.unwrap() < Phase::EVENTUALLY_RECT
        {
            self.phase_ = Some(match self.phase_.unwrap() {
                Phase::VISIBLE_RECT => Phase::PENDING_VISIBLE_RECT,
                Phase::PENDING_VISIBLE_RECT => Phase::SKEWPORT_RECT,
                Phase::SKEWPORT_RECT => Phase::SOON_BORDER_RECT,
                Phase::SOON_BORDER_RECT => Phase::EVENTUALLY_RECT,
                Phase::EVENTUALLY_RECT => unreachable!("source phase increment"),
            });
            let tiling = self.tiling_.expect("source nonnull tiling");
            match self.phase_.unwrap() {
                Phase::VISIBLE_RECT => unreachable!("source NOTREACHED"),
                Phase::PENDING_VISIBLE_RECT => {
                    self.pending_visible_iterator_ =
                        PendingVisibleTilingIterator::new(tiling, context);
                    if !self.pending_visible_iterator_.done() {
                        self.current_tile_ = self.pending_visible_iterator_.get().clone();
                    }
                }
                Phase::SKEWPORT_RECT => {
                    self.skewport_iterator_ = SkewportTilingIterator::new(tiling, context);
                    if !self.skewport_iterator_.done() {
                        self.current_tile_ = self.skewport_iterator_.get().clone();
                    }
                }
                Phase::SOON_BORDER_RECT => {
                    self.soon_border_iterator_ = SoonBorderTilingIterator::new(tiling, context);
                    if !self.soon_border_iterator_.done() {
                        self.current_tile_ = self.soon_border_iterator_.get().clone();
                    }
                }
                Phase::EVENTUALLY_RECT => {
                    self.eventually_iterator_ = EventuallyTilingIterator::new(tiling, context);
                    if !self.eventually_iterator_.done() {
                        self.current_tile_ = self.eventually_iterator_.get().clone();
                    }
                }
            }
        }
    }
    // Source: cc/tiles/tiling_set_raster_queue_all.h:169;
    //         cc/tiles/tiling_set_raster_queue_all.cc:461-507.
    fn Advance(&mut self, context: &dyn RasterQueueContext<B>) -> &mut Self {
        match self.phase_.expect("source initialized phase") {
            Phase::VISIBLE_RECT => {
                self.visible_iterator_.Advance(context);
                if self.visible_iterator_.done() {
                    self.AdvancePhase(context);
                    return self;
                }
                self.current_tile_ = self.visible_iterator_.get().clone();
            }
            Phase::PENDING_VISIBLE_RECT => {
                self.pending_visible_iterator_.Advance(context);
                if self.pending_visible_iterator_.done() {
                    self.AdvancePhase(context);
                    return self;
                }
                self.current_tile_ = self.pending_visible_iterator_.get().clone();
            }
            Phase::SKEWPORT_RECT => {
                self.skewport_iterator_.Advance(context);
                if self.skewport_iterator_.done() {
                    self.AdvancePhase(context);
                    return self;
                }
                self.current_tile_ = self.skewport_iterator_.get().clone();
            }
            Phase::SOON_BORDER_RECT => {
                self.soon_border_iterator_.Advance(context);
                if self.soon_border_iterator_.done() {
                    self.AdvancePhase(context);
                    return self;
                }
                self.current_tile_ = self.soon_border_iterator_.get().clone();
            }
            Phase::EVENTUALLY_RECT => {
                self.eventually_iterator_.Advance(context);
                if self.eventually_iterator_.done() {
                    self.current_tile_ = PrioritizedTile::default();
                    return self;
                }
                self.current_tile_ = self.eventually_iterator_.get().clone();
            }
        }
        self
    }
}
