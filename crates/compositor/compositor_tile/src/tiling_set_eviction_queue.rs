// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/tiling_set_eviction_queue.h:5-65
// Source: cc/tiles/tiling_set_eviction_queue.cc:5-13
use super::{
    picture_layer_tiling::{PictureLayerTiling, PriorityRectType},
    prioritized_tile::PrioritizedTile,
    raster_tile_priority_queue::PictureLayerTilingSetId,
    WhichTree,
};
use crate::{
    picture_layer_tiling_types::PictureLayerTilingTypes,
    prioritized_tile_types::PrioritizedTileResidentExt,
    tile_types::PictureLayerTilingId,
    tiling_data::{DifferenceIterator, Iterator, ReverseSpiralDifferenceIterator},
};

// All pending value/query dependencies are now the same canonical owners.
use super::picture_layer_tiling_set::TilingRangeType;
pub use super::raster_tile_priority_queue::RasterQueueContext as EvictionQueueContext;

// Source: cc/tiles/tiling_set_eviction_queue.h:78-88
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Phase {
    EVENTUALLY_RECT,
    SOON_BORDER_RECT,
    SKEWPORT_RECT,
    PENDING_VISIBLE_RECT,
    PENDING_VISIBLE_RECT_REQUIRED_FOR_ACTIVATION,
    VISIBLE_RECT_OCCLUDED,
    VISIBLE_RECT_UNOCCLUDED,
    VISIBLE_RECT_REQUIRED_FOR_ACTIVATION_OCCLUDED,
    VISIBLE_RECT_REQUIRED_FOR_ACTIVATION_UNOCCLUDED,
}
impl Phase {
    // Source: cc/tiles/tiling_set_eviction_queue.cc:70
    // Exhaustive safe equivalent of the source enum's increment.
    fn next(self) -> Self {
        match self {
            Self::EVENTUALLY_RECT => Self::SOON_BORDER_RECT,
            Self::SOON_BORDER_RECT => Self::SKEWPORT_RECT,
            Self::SKEWPORT_RECT => Self::PENDING_VISIBLE_RECT,
            Self::PENDING_VISIBLE_RECT => Self::PENDING_VISIBLE_RECT_REQUIRED_FOR_ACTIVATION,
            Self::PENDING_VISIBLE_RECT_REQUIRED_FOR_ACTIVATION => Self::VISIBLE_RECT_OCCLUDED,
            Self::VISIBLE_RECT_OCCLUDED => Self::VISIBLE_RECT_UNOCCLUDED,
            Self::VISIBLE_RECT_UNOCCLUDED => Self::VISIBLE_RECT_REQUIRED_FOR_ACTIVATION_OCCLUDED,
            Self::VISIBLE_RECT_REQUIRED_FOR_ACTIVATION_OCCLUDED => {
                Self::VISIBLE_RECT_REQUIRED_FOR_ACTIVATION_UNOCCLUDED
            }
            Self::VISIBLE_RECT_REQUIRED_FOR_ACTIVATION_UNOCCLUDED => {
                unreachable!("source phase increment excludes final phase")
            }
        }
    }
}
// Source: cc/tiles/tiling_set_eviction_queue.h:66-76,199-214
// Source non-owning tiling pointers are identities, never copied tiling state.
// Iterators receive this same resident ordered vector as a call-scoped slice,
// replacing their vector* without making a self-referential Rust object.
pub struct TilingSetEvictionQueue<B: PictureLayerTilingTypes> {
    tree_: WhichTree,
    phase_: Phase,
    current_tile_: PrioritizedTile,
    tilings_: Vec<PictureLayerTilingId>,
    eventually_iterator_: EventuallyTilingIterator<B>,
    soon_iterator_: SoonBorderTilingIterator<B>,
    skewport_iterator_: SkewportTilingIterator<B>,
    pending_visible_iterator_: PendingVisibleTilingIterator<B>,
    visible_iterator_: VisibleTilingIterator<B>,
    is_drawing_layer_: bool,
}
impl<B: PictureLayerTilingTypes> TilingSetEvictionQueue<B> {
    // Source: cc/tiles/tiling_set_eviction_queue.h:68-70
    // Source: cc/tiles/tiling_set_eviction_queue.cc:15-33
    pub fn new(
        tiling_set: PictureLayerTilingSetId,
        is_drawing_layer: bool,
        context: &dyn EvictionQueueContext<B>,
    ) -> Self {
        let mut result = Self {
            tree_: context.TilingSetById(tiling_set).tree(),
            phase_: Phase::EVENTUALLY_RECT,
            current_tile_: PrioritizedTile::default(),
            tilings_: Vec::new(),
            eventually_iterator_: EventuallyTilingIterator::default(),
            soon_iterator_: SoonBorderTilingIterator::default(),
            skewport_iterator_: SkewportTilingIterator::default(),
            pending_visible_iterator_: PendingVisibleTilingIterator::default(),
            visible_iterator_: VisibleTilingIterator::default(),
            is_drawing_layer_: is_drawing_layer,
        };
        if context.TilingSetById(tiling_set).num_tilings() == 0 {
            return result;
        }
        result.GenerateTilingOrder(tiling_set, context);
        result.eventually_iterator_ =
            EventuallyTilingIterator::new(&result.tilings_, result.tree_, context);
        if result.eventually_iterator_.base.done() {
            result.AdvancePhase(context);
            return result;
        }
        result.current_tile_ = result.eventually_iterator_.base.get().clone();
        result
    }
    // Source: cc/tiles/tiling_set_eviction_queue.h:90
    // Source: cc/tiles/tiling_set_eviction_queue.cc:35-64
    fn GenerateTilingOrder(
        &mut self,
        tiling_set: PictureLayerTilingSetId,
        context: &dyn EvictionQueueContext<B>,
    ) {
        self.tilings_
            .reserve(context.TilingSetById(tiling_set).num_tilings());
        let mut range = context
            .TilingSetById(tiling_set)
            .GetTilingRange(TilingRangeType::HIGHER_THAN_HIGH_RES);
        for index in range.start..range.end {
            let tiling = context
                .TilingSetById(tiling_set)
                .tiling_at(index)
                .identity();
            if context.TilingById(tiling).has_tiles() {
                self.tilings_.push(tiling);
            }
        }
        range = context
            .TilingSetById(tiling_set)
            .GetTilingRange(TilingRangeType::LOWER_THAN_HIGH_RES);
        for i in range.start..range.end {
            let index = range.start + (range.end - 1 - i);
            let tiling = context
                .TilingSetById(tiling_set)
                .tiling_at(index)
                .identity();
            if context.TilingById(tiling).has_tiles() {
                self.tilings_.push(tiling);
            }
        }
        range = context
            .TilingSetById(tiling_set)
            .GetTilingRange(TilingRangeType::HIGH_RES);
        for index in range.start..range.end {
            let tiling = context
                .TilingSetById(tiling_set)
                .tiling_at(index)
                .identity();
            if context.TilingById(tiling).has_tiles() {
                self.tilings_.push(tiling);
            }
        }
        debug_assert!(context.TilingSetById(tiling_set).num_tilings() >= self.tilings_.len());
    }
    // Source: cc/tiles/tiling_set_eviction_queue.h:199
    // Source: cc/tiles/tiling_set_eviction_queue.cc:66-126
    fn AdvancePhase(&mut self, context: &dyn EvictionQueueContext<B>) {
        self.current_tile_ = PrioritizedTile::default();
        while self.current_tile_.tile_id().is_none()
            && self.phase_ != Phase::VISIBLE_RECT_REQUIRED_FOR_ACTIVATION_UNOCCLUDED
        {
            self.phase_ = self.phase_.next();
            match self.phase_ {
                Phase::EVENTUALLY_RECT => unreachable!("source NOTREACHED"),
                Phase::SOON_BORDER_RECT => {
                    self.soon_iterator_ =
                        SoonBorderTilingIterator::new(&self.tilings_, self.tree_, context);
                    if !self.soon_iterator_.base.done() {
                        self.current_tile_ = self.soon_iterator_.base.get().clone();
                    }
                }
                Phase::SKEWPORT_RECT => {
                    self.skewport_iterator_ =
                        SkewportTilingIterator::new(&self.tilings_, self.tree_, context);
                    if !self.skewport_iterator_.base.done() {
                        self.current_tile_ = self.skewport_iterator_.base.get().clone();
                    }
                }
                Phase::PENDING_VISIBLE_RECT => {
                    self.pending_visible_iterator_ = PendingVisibleTilingIterator::new(
                        &self.tilings_,
                        self.tree_,
                        false,
                        context,
                    );
                    if !self.pending_visible_iterator_.base.done() {
                        self.current_tile_ = self.pending_visible_iterator_.base.get().clone();
                    }
                }
                Phase::PENDING_VISIBLE_RECT_REQUIRED_FOR_ACTIVATION => {
                    self.pending_visible_iterator_ = PendingVisibleTilingIterator::new(
                        &self.tilings_,
                        self.tree_,
                        true,
                        context,
                    );
                    if !self.pending_visible_iterator_.base.done() {
                        self.current_tile_ = self.pending_visible_iterator_.base.get().clone();
                    }
                }
                Phase::VISIBLE_RECT_OCCLUDED => {
                    self.visible_iterator_ = VisibleTilingIterator::new(
                        &self.tilings_,
                        self.tree_,
                        true,
                        false,
                        context,
                    );
                    if !self.visible_iterator_.base.done() {
                        self.current_tile_ = self.visible_iterator_.base.get().clone();
                    }
                }
                Phase::VISIBLE_RECT_UNOCCLUDED => {
                    self.visible_iterator_ = VisibleTilingIterator::new(
                        &self.tilings_,
                        self.tree_,
                        false,
                        false,
                        context,
                    );
                    if !self.visible_iterator_.base.done() {
                        self.current_tile_ = self.visible_iterator_.base.get().clone();
                    }
                }
                Phase::VISIBLE_RECT_REQUIRED_FOR_ACTIVATION_OCCLUDED => {
                    self.visible_iterator_ =
                        VisibleTilingIterator::new(&self.tilings_, self.tree_, true, true, context);
                    if !self.visible_iterator_.base.done() {
                        self.current_tile_ = self.visible_iterator_.base.get().clone();
                    }
                }
                Phase::VISIBLE_RECT_REQUIRED_FOR_ACTIVATION_UNOCCLUDED => {
                    self.visible_iterator_ = VisibleTilingIterator::new(
                        &self.tilings_,
                        self.tree_,
                        false,
                        true,
                        context,
                    );
                    if !self.visible_iterator_.base.done() {
                        self.current_tile_ = self.visible_iterator_.base.get().clone();
                    }
                }
            }
        }
    }
    // Source: cc/tiles/tiling_set_eviction_queue.h:74
    // Source: cc/tiles/tiling_set_eviction_queue.cc:128-130
    pub fn IsEmpty(&self) -> bool {
        self.current_tile_.tile_id().is_none()
    }
    // Source: cc/tiles/tiling_set_eviction_queue.h:72
    // Source: cc/tiles/tiling_set_eviction_queue.cc:132-135
    pub fn Top(&self) -> &PrioritizedTile {
        debug_assert!(!self.IsEmpty());
        &self.current_tile_
    }
    // Source: cc/tiles/tiling_set_eviction_queue.h:75
    pub fn is_drawing_layer(&self) -> bool {
        self.is_drawing_layer_
    }
    // Source: cc/tiles/tiling_set_eviction_queue.h:73
    // Source: cc/tiles/tiling_set_eviction_queue.cc:137-173
    pub fn Pop(&mut self, context: &dyn EvictionQueueContext<B>) {
        debug_assert!(!self.IsEmpty());
        self.current_tile_ = PrioritizedTile::default();
        match self.phase_ {
            Phase::EVENTUALLY_RECT => {
                self.eventually_iterator_.Advance(&self.tilings_, context);
                if !self.eventually_iterator_.base.done() {
                    self.current_tile_ = self.eventually_iterator_.base.get().clone();
                }
            }
            Phase::SOON_BORDER_RECT => {
                self.soon_iterator_.Advance(&self.tilings_, context);
                if !self.soon_iterator_.base.done() {
                    self.current_tile_ = self.soon_iterator_.base.get().clone();
                }
            }
            Phase::SKEWPORT_RECT => {
                self.skewport_iterator_.Advance(&self.tilings_, context);
                if !self.skewport_iterator_.base.done() {
                    self.current_tile_ = self.skewport_iterator_.base.get().clone();
                }
            }
            Phase::PENDING_VISIBLE_RECT | Phase::PENDING_VISIBLE_RECT_REQUIRED_FOR_ACTIVATION => {
                self.pending_visible_iterator_
                    .Advance(&self.tilings_, context);
                if !self.pending_visible_iterator_.base.done() {
                    self.current_tile_ = self.pending_visible_iterator_.base.get().clone();
                }
            }
            Phase::VISIBLE_RECT_OCCLUDED
            | Phase::VISIBLE_RECT_UNOCCLUDED
            | Phase::VISIBLE_RECT_REQUIRED_FOR_ACTIVATION_OCCLUDED
            | Phase::VISIBLE_RECT_REQUIRED_FOR_ACTIVATION_UNOCCLUDED => {
                self.visible_iterator_.Advance(&self.tilings_, context);
                if !self.visible_iterator_.base.done() {
                    self.current_tile_ = self.visible_iterator_.base.get().clone();
                }
            }
        }
        if self.current_tile_.tile_id().is_none() {
            self.AdvancePhase(context);
        }
    }
}

// Source: cc/tiles/tiling_set_eviction_queue.h:107-110
// Rust equivalent of the two native iterator templates. These implementations
// forward only to canonical TilingData cursors; they contain no region policy.
trait TileCursor {
    fn advance(&mut self);
    fn valid(&self) -> bool;
    fn x(&self) -> i32;
    fn y(&self) -> i32;
}
macro_rules! cursor_impl {
    ($ty:ty) => {
        impl TileCursor for $ty {
            fn advance(&mut self) {
                self.Advance();
            }
            fn valid(&self) -> bool {
                self.IsValid()
            }
            fn x(&self) -> i32 {
                self.index_x()
            }
            fn y(&self) -> i32 {
                self.index_y()
            }
        }
    };
}
cursor_impl!(Iterator);
cursor_impl!(DifferenceIterator);
cursor_impl!(ReverseSpiralDifferenceIterator);
// Source: cc/tiles/tiling_set_eviction_queue.h:92-122
struct EvictionRectIterator<B: PictureLayerTilingTypes> {
    prioritized_tile_: PrioritizedTile,
    tree_: WhichTree,
    priority_rect_type_: Option<PriorityRectType>,
    tiling_index_: usize,
    source_type_: std::marker::PhantomData<fn() -> B>,
}
impl<B: PictureLayerTilingTypes> Default for EvictionRectIterator<B> {
    // Source: cc/tiles/tiling_set_eviction_queue.cc:175-178
    fn default() -> Self {
        Self {
            prioritized_tile_: PrioritizedTile::default(),
            tree_: WhichTree::ACTIVE_TREE,
            priority_rect_type_: None,
            tiling_index_: 0,
            source_type_: std::marker::PhantomData,
        }
    }
}
impl<B: PictureLayerTilingTypes> EvictionRectIterator<B> {
    // Source: cc/tiles/tiling_set_eviction_queue.cc:180-187
    fn new(tree: WhichTree, priority_rect_type: PriorityRectType) -> Self {
        Self {
            prioritized_tile_: PrioritizedTile::default(),
            tree_: tree,
            priority_rect_type_: Some(priority_rect_type),
            tiling_index_: 0,
            source_type_: std::marker::PhantomData,
        }
    }
    // Source: cc/tiles/tiling_set_eviction_queue.h:101-102
    fn done(&self) -> bool {
        self.prioritized_tile_.tile_id().is_none()
    }
    fn get(&self) -> &PrioritizedTile {
        &self.prioritized_tile_
    }
    // Source: cc/tiles/tiling_set_eviction_queue.cc:189-202
    fn AdvanceToNextTile(
        &mut self,
        iterator: &mut impl TileCursor,
        tilings: &[PictureLayerTilingId],
        context: &dyn EvictionQueueContext<B>,
    ) -> bool {
        let mut found_tile = false;
        while !found_tile {
            iterator.advance();
            if !iterator.valid() {
                self.prioritized_tile_ = PrioritizedTile::default();
                break;
            }
            found_tile = self.GetFirstTileAndCheckIfValid(iterator, tilings, context);
        }
        found_tile
    }
    // Source: cc/tiles/tiling_set_eviction_queue.cc:204-226
    fn GetFirstTileAndCheckIfValid(
        &mut self,
        iterator: &impl TileCursor,
        tilings: &[PictureLayerTilingId],
        context: &dyn EvictionQueueContext<B>,
    ) -> bool {
        let tiling = tilings[self.tiling_index_];
        let tile = context
            .TilingById(tiling)
            .TileAt(iterator.x(), iterator.y())
            .map(|tile| tile.identity());
        self.prioritized_tile_ = PrioritizedTile::default();
        if tile.is_none() || !context.TileById(tile.unwrap()).draw_info().has_resource() {
            return false;
        }
        let tile = tile.unwrap();
        let priority_rect_type = self
            .priority_rect_type_
            .expect("initialized iterator priority rect");
        if priority_rect_type > PriorityRectType::PENDING_VISIBLE_RECT {
            let tile_rect = context.TilingById(tiling).tiling_data().TileBounds(
                context.TileById(tile).tiling_i_index(),
                context.TileById(tile).tiling_j_index(),
            );
            if context
                .TilingById(tiling)
                .pending_visible_rect(context)
                .Intersects(tile_rect)
            {
                return false;
            }
        }
        let is_occluded = context
            .TilingById(tiling)
            .IsTileOccluded(context.TileById(tile), context);
        self.prioritized_tile_ = context.TilingById(tiling).MakePrioritizedTile(
            tile,
            priority_rect_type,
            is_occluded,
            context,
        );
        true
    }
}

// Source: cc/tiles/tiling_set_eviction_queue.h:186-197
struct EventuallyTilingIterator<B: PictureLayerTilingTypes> {
    base: EvictionRectIterator<B>,
    iterator_: ReverseSpiralDifferenceIterator,
}
impl<B: PictureLayerTilingTypes> Default for EventuallyTilingIterator<B> {
    fn default() -> Self {
        Self {
            base: EvictionRectIterator::default(),
            iterator_: ReverseSpiralDifferenceIterator::default(),
        }
    }
}
impl<B: PictureLayerTilingTypes> EventuallyTilingIterator<B> {
    // Source: cc/tiles/tiling_set_eviction_queue.cc:228-254
    fn new(
        tilings: &[PictureLayerTilingId],
        tree: WhichTree,
        context: &dyn EvictionQueueContext<B>,
    ) -> Self {
        let mut result = Self {
            base: EvictionRectIterator::new(tree, PriorityRectType::EVENTUALLY_RECT),
            iterator_: ReverseSpiralDifferenceIterator::default(),
        };
        while result.base.tiling_index_ < tilings.len() {
            if !context
                .TilingById(tilings[result.base.tiling_index_])
                .has_eventually_rect_tiles()
            {
                result.base.tiling_index_ += 1;
                continue;
            }
            let tiling = context.TilingById(tilings[result.base.tiling_index_]);
            result.iterator_ = ReverseSpiralDifferenceIterator::new(
                tiling.tiling_data(),
                tiling.current_eventually_rect(),
                tiling.current_skewport_rect(),
                tiling.current_soon_border_rect(),
            );
            if !result.iterator_.IsValid() {
                result.base.tiling_index_ += 1;
                continue;
            }
            break;
        }
        if result.base.tiling_index_ >= tilings.len() {
            return result;
        }
        if !result
            .base
            .GetFirstTileAndCheckIfValid(&result.iterator_, tilings, context)
        {
            result.Advance(tilings, context);
        }
        result
    }
    // Source: cc/tiles/tiling_set_eviction_queue.cc:256-276
    fn Advance(
        &mut self,
        tilings: &[PictureLayerTilingId],
        context: &dyn EvictionQueueContext<B>,
    ) -> &mut Self {
        let mut found_tile = self
            .base
            .AdvanceToNextTile(&mut self.iterator_, tilings, context);
        while !found_tile && self.base.tiling_index_ + 1 < tilings.len() {
            self.base.tiling_index_ += 1;
            if !context
                .TilingById(tilings[self.base.tiling_index_])
                .has_eventually_rect_tiles()
            {
                continue;
            }
            let tiling = context.TilingById(tilings[self.base.tiling_index_]);
            self.iterator_ = ReverseSpiralDifferenceIterator::new(
                tiling.tiling_data(),
                tiling.current_eventually_rect(),
                tiling.current_skewport_rect(),
                tiling.current_soon_border_rect(),
            );
            if !self.iterator_.IsValid() {
                continue;
            }
            found_tile = self
                .base
                .GetFirstTileAndCheckIfValid(&self.iterator_, tilings, context);
            if !found_tile {
                found_tile = self
                    .base
                    .AdvanceToNextTile(&mut self.iterator_, tilings, context);
            }
        }
        self
    }
}

// Source: cc/tiles/tiling_set_eviction_queue.h:173-184
struct SoonBorderTilingIterator<B: PictureLayerTilingTypes> {
    base: EvictionRectIterator<B>,
    iterator_: ReverseSpiralDifferenceIterator,
}
impl<B: PictureLayerTilingTypes> Default for SoonBorderTilingIterator<B> {
    fn default() -> Self {
        Self {
            base: EvictionRectIterator::default(),
            iterator_: ReverseSpiralDifferenceIterator::default(),
        }
    }
}
impl<B: PictureLayerTilingTypes> SoonBorderTilingIterator<B> {
    // Source: cc/tiles/tiling_set_eviction_queue.cc:278-306
    fn new(
        tilings: &[PictureLayerTilingId],
        tree: WhichTree,
        context: &dyn EvictionQueueContext<B>,
    ) -> Self {
        let mut result = Self {
            base: EvictionRectIterator::new(tree, PriorityRectType::SOON_BORDER_RECT),
            iterator_: ReverseSpiralDifferenceIterator::default(),
        };
        while result.base.tiling_index_ < tilings.len() {
            if !context
                .TilingById(tilings[result.base.tiling_index_])
                .has_soon_border_rect_tiles()
            {
                result.base.tiling_index_ += 1;
                continue;
            }
            let tiling = context.TilingById(tilings[result.base.tiling_index_]);
            result.iterator_ = ReverseSpiralDifferenceIterator::new(
                tiling.tiling_data(),
                tiling.current_soon_border_rect(),
                tiling.current_skewport_rect(),
                tiling.current_visible_rect(),
            );
            if !result.iterator_.IsValid() {
                result.base.tiling_index_ += 1;
                continue;
            }
            break;
        }
        if result.base.tiling_index_ >= tilings.len() {
            return result;
        }
        if !result
            .base
            .GetFirstTileAndCheckIfValid(&result.iterator_, tilings, context)
        {
            result.Advance(tilings, context);
        }
        result
    }
    // Source: cc/tiles/tiling_set_eviction_queue.cc:308-328
    fn Advance(
        &mut self,
        tilings: &[PictureLayerTilingId],
        context: &dyn EvictionQueueContext<B>,
    ) -> &mut Self {
        let mut found_tile = self
            .base
            .AdvanceToNextTile(&mut self.iterator_, tilings, context);
        while !found_tile && self.base.tiling_index_ + 1 < tilings.len() {
            self.base.tiling_index_ += 1;
            if !context
                .TilingById(tilings[self.base.tiling_index_])
                .has_soon_border_rect_tiles()
            {
                continue;
            }
            let tiling = context.TilingById(tilings[self.base.tiling_index_]);
            self.iterator_ = ReverseSpiralDifferenceIterator::new(
                tiling.tiling_data(),
                tiling.current_soon_border_rect(),
                tiling.current_skewport_rect(),
                tiling.current_visible_rect(),
            );
            if !self.iterator_.IsValid() {
                continue;
            }
            found_tile = self
                .base
                .GetFirstTileAndCheckIfValid(&self.iterator_, tilings, context);
            if !found_tile {
                found_tile = self
                    .base
                    .AdvanceToNextTile(&mut self.iterator_, tilings, context);
            }
        }
        self
    }
}

// Source: cc/tiles/tiling_set_eviction_queue.h:160-171
struct SkewportTilingIterator<B: PictureLayerTilingTypes> {
    base: EvictionRectIterator<B>,
    iterator_: ReverseSpiralDifferenceIterator,
}
impl<B: PictureLayerTilingTypes> Default for SkewportTilingIterator<B> {
    fn default() -> Self {
        Self {
            base: EvictionRectIterator::default(),
            iterator_: ReverseSpiralDifferenceIterator::default(),
        }
    }
}
impl<B: PictureLayerTilingTypes> SkewportTilingIterator<B> {
    // Source: cc/tiles/tiling_set_eviction_queue.cc:330-356
    fn new(
        tilings: &[PictureLayerTilingId],
        tree: WhichTree,
        context: &dyn EvictionQueueContext<B>,
    ) -> Self {
        let mut result = Self {
            base: EvictionRectIterator::new(tree, PriorityRectType::SKEWPORT_RECT),
            iterator_: ReverseSpiralDifferenceIterator::default(),
        };
        while result.base.tiling_index_ < tilings.len() {
            if !context
                .TilingById(tilings[result.base.tiling_index_])
                .has_skewport_rect_tiles()
            {
                result.base.tiling_index_ += 1;
                continue;
            }
            let tiling = context.TilingById(tilings[result.base.tiling_index_]);
            result.iterator_ = ReverseSpiralDifferenceIterator::new(
                tiling.tiling_data(),
                tiling.current_skewport_rect(),
                tiling.current_visible_rect(),
                tiling.current_visible_rect(),
            );
            if !result.iterator_.IsValid() {
                result.base.tiling_index_ += 1;
                continue;
            }
            break;
        }
        if result.base.tiling_index_ >= tilings.len() {
            return result;
        }
        if !result
            .base
            .GetFirstTileAndCheckIfValid(&result.iterator_, tilings, context)
        {
            result.Advance(tilings, context);
        }
        result
    }
    // Source: cc/tiles/tiling_set_eviction_queue.cc:358-378
    fn Advance(
        &mut self,
        tilings: &[PictureLayerTilingId],
        context: &dyn EvictionQueueContext<B>,
    ) -> &mut Self {
        let mut found_tile = self
            .base
            .AdvanceToNextTile(&mut self.iterator_, tilings, context);
        while !found_tile && self.base.tiling_index_ + 1 < tilings.len() {
            self.base.tiling_index_ += 1;
            if !context
                .TilingById(tilings[self.base.tiling_index_])
                .has_skewport_rect_tiles()
            {
                continue;
            }
            let tiling = context.TilingById(tilings[self.base.tiling_index_]);
            self.iterator_ = ReverseSpiralDifferenceIterator::new(
                tiling.tiling_data(),
                tiling.current_skewport_rect(),
                tiling.current_visible_rect(),
                tiling.current_visible_rect(),
            );
            if !self.iterator_.IsValid() {
                continue;
            }
            found_tile = self
                .base
                .GetFirstTileAndCheckIfValid(&self.iterator_, tilings, context);
            if !found_tile {
                found_tile = self
                    .base
                    .AdvanceToNextTile(&mut self.iterator_, tilings, context);
            }
        }
        self
    }
}

// Source: cc/tiles/tiling_set_eviction_queue.h:124-139
struct PendingVisibleTilingIterator<B: PictureLayerTilingTypes> {
    base: EvictionRectIterator<B>,
    iterator_: DifferenceIterator,
    return_required_for_activation_tiles_: Option<bool>,
}
impl<B: PictureLayerTilingTypes> Default for PendingVisibleTilingIterator<B> {
    // Source: cc/tiles/tiling_set_eviction_queue.h:126
    // The default native bool is uninitialized and never read before the
    // parameterized constructor; None preserves that inaccessible state.
    fn default() -> Self {
        Self {
            base: EvictionRectIterator::default(),
            iterator_: DifferenceIterator::default(),
            return_required_for_activation_tiles_: None,
        }
    }
}
impl<B: PictureLayerTilingTypes> PendingVisibleTilingIterator<B> {
    // Source: cc/tiles/tiling_set_eviction_queue.cc:380-413
    fn new(
        tilings: &[PictureLayerTilingId],
        tree: WhichTree,
        return_required_for_activation_tiles: bool,
        context: &dyn EvictionQueueContext<B>,
    ) -> Self {
        let mut result = Self {
            base: EvictionRectIterator::new(tree, PriorityRectType::PENDING_VISIBLE_RECT),
            iterator_: DifferenceIterator::default(),
            return_required_for_activation_tiles_: Some(return_required_for_activation_tiles),
        };
        while result.base.tiling_index_ < tilings.len() {
            let tiling = context.TilingById(tilings[result.base.tiling_index_]);
            result.iterator_ = DifferenceIterator::new(
                tiling.tiling_data(),
                &tiling.pending_visible_rect(context),
                tiling.current_visible_rect(),
            );
            if !result.iterator_.IsValid() {
                result.base.tiling_index_ += 1;
                continue;
            }
            break;
        }
        if result.base.tiling_index_ >= tilings.len() {
            return result;
        }
        if !result
            .base
            .GetFirstTileAndCheckIfValid(&result.iterator_, tilings, context)
        {
            result.Advance(tilings, context);
            return result;
        }
        if !result.TileMatchesRequiredFlags(result.base.get(), context) {
            result.Advance(tilings, context);
            return result;
        }
        result
    }
    // Source: cc/tiles/tiling_set_eviction_queue.cc:415-437
    fn Advance(
        &mut self,
        tilings: &[PictureLayerTilingId],
        context: &dyn EvictionQueueContext<B>,
    ) -> &mut Self {
        let mut found_tile = self
            .base
            .AdvanceToNextTile(&mut self.iterator_, tilings, context);
        while found_tile && !self.TileMatchesRequiredFlags(self.base.get(), context) {
            found_tile = self
                .base
                .AdvanceToNextTile(&mut self.iterator_, tilings, context);
        }
        while !found_tile && self.base.tiling_index_ + 1 < tilings.len() {
            self.base.tiling_index_ += 1;
            let tiling = context.TilingById(tilings[self.base.tiling_index_]);
            self.iterator_ = DifferenceIterator::new(
                tiling.tiling_data(),
                &tiling.pending_visible_rect(context),
                tiling.current_visible_rect(),
            );
            if !self.iterator_.IsValid() {
                continue;
            }
            found_tile = self
                .base
                .GetFirstTileAndCheckIfValid(&self.iterator_, tilings, context);
            if !found_tile {
                found_tile = self
                    .base
                    .AdvanceToNextTile(&mut self.iterator_, tilings, context);
            }
            while found_tile && !self.TileMatchesRequiredFlags(self.base.get(), context) {
                found_tile = self
                    .base
                    .AdvanceToNextTile(&mut self.iterator_, tilings, context);
            }
        }
        self
    }
    // Source: cc/tiles/tiling_set_eviction_queue.cc:439-444
    fn TileMatchesRequiredFlags(
        &self,
        tile: &PrioritizedTile,
        context: &dyn EvictionQueueContext<B>,
    ) -> bool {
        let activation_flag_matches = tile
            .tile::<PictureLayerTiling<B>>(context)
            .expect("source nonnull tile")
            .required_for_activation()
            == self
                .return_required_for_activation_tiles_
                .expect("initialized activation flag");
        activation_flag_matches
    }
}

// Source: cc/tiles/tiling_set_eviction_queue.h:141-158
struct VisibleTilingIterator<B: PictureLayerTilingTypes> {
    base: EvictionRectIterator<B>,
    iterator_: Iterator,
    return_occluded_tiles_: Option<bool>,
    return_required_for_activation_tiles_: Option<bool>,
}
impl<B: PictureLayerTilingTypes> Default for VisibleTilingIterator<B> {
    // Source: cc/tiles/tiling_set_eviction_queue.h:143
    fn default() -> Self {
        Self {
            base: EvictionRectIterator::default(),
            iterator_: Iterator::default(),
            return_occluded_tiles_: None,
            return_required_for_activation_tiles_: None,
        }
    }
}
impl<B: PictureLayerTilingTypes> VisibleTilingIterator<B> {
    // Source: cc/tiles/tiling_set_eviction_queue.cc:446-481
    fn new(
        tilings: &[PictureLayerTilingId],
        tree: WhichTree,
        return_occluded_tiles: bool,
        return_required_for_activation_tiles: bool,
        context: &dyn EvictionQueueContext<B>,
    ) -> Self {
        let mut result = Self {
            base: EvictionRectIterator::new(tree, PriorityRectType::VISIBLE_RECT),
            iterator_: Iterator::default(),
            return_occluded_tiles_: Some(return_occluded_tiles),
            return_required_for_activation_tiles_: Some(return_required_for_activation_tiles),
        };
        while result.base.tiling_index_ < tilings.len() {
            if !context
                .TilingById(tilings[result.base.tiling_index_])
                .has_visible_rect_tiles()
            {
                result.base.tiling_index_ += 1;
                continue;
            }
            let tiling = context.TilingById(tilings[result.base.tiling_index_]);
            result.iterator_ =
                Iterator::new(tiling.tiling_data(), tiling.current_visible_rect(), false);
            if !result.iterator_.IsValid() {
                result.base.tiling_index_ += 1;
                continue;
            }
            break;
        }
        if result.base.tiling_index_ >= tilings.len() {
            return result;
        }
        if !result
            .base
            .GetFirstTileAndCheckIfValid(&result.iterator_, tilings, context)
        {
            result.Advance(tilings, context);
            return result;
        }
        if !result.TileMatchesRequiredFlags(result.base.get(), context) {
            result.Advance(tilings, context);
            return result;
        }
        result
    }
    // Source: cc/tiles/tiling_set_eviction_queue.cc:483-506
    fn Advance(
        &mut self,
        tilings: &[PictureLayerTilingId],
        context: &dyn EvictionQueueContext<B>,
    ) -> &mut Self {
        let mut found_tile = self
            .base
            .AdvanceToNextTile(&mut self.iterator_, tilings, context);
        while found_tile && !self.TileMatchesRequiredFlags(self.base.get(), context) {
            found_tile = self
                .base
                .AdvanceToNextTile(&mut self.iterator_, tilings, context);
        }
        while !found_tile && self.base.tiling_index_ + 1 < tilings.len() {
            self.base.tiling_index_ += 1;
            if !context
                .TilingById(tilings[self.base.tiling_index_])
                .has_visible_rect_tiles()
            {
                continue;
            }
            let tiling = context.TilingById(tilings[self.base.tiling_index_]);
            self.iterator_ =
                Iterator::new(tiling.tiling_data(), tiling.current_visible_rect(), false);
            if !self.iterator_.IsValid() {
                continue;
            }
            found_tile = self
                .base
                .GetFirstTileAndCheckIfValid(&self.iterator_, tilings, context);
            if !found_tile {
                found_tile = self
                    .base
                    .AdvanceToNextTile(&mut self.iterator_, tilings, context);
            }
            while found_tile && !self.TileMatchesRequiredFlags(self.base.get(), context) {
                found_tile = self
                    .base
                    .AdvanceToNextTile(&mut self.iterator_, tilings, context);
            }
        }
        self
    }
    // Source: cc/tiles/tiling_set_eviction_queue.cc:508-514
    fn TileMatchesRequiredFlags(
        &self,
        tile: &PrioritizedTile,
        context: &dyn EvictionQueueContext<B>,
    ) -> bool {
        let activation_flag_matches = tile
            .tile::<PictureLayerTiling<B>>(context)
            .expect("source nonnull tile")
            .required_for_activation()
            == self
                .return_required_for_activation_tiles_
                .expect("initialized activation flag");
        let occluded_flag_matches = tile.is_occluded()
            == self
                .return_occluded_tiles_
                .expect("initialized occlusion flag");
        activation_flag_matches && occluded_flag_matches
    }
}
// Source: cc/tiles/tiling_set_eviction_queue.h:216-218
// Source: cc/tiles/tiling_set_eviction_queue.cc:516
// Include guards/namespace/default destruction map to Rust modules and values.
