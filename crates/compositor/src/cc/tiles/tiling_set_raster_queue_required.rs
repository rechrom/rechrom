// Translation status: complete
use crate::adapter::picture_layer_tiling::PictureLayerTilingTypes;
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/tiling_set_raster_queue_required.h:1-22,33-38,49-55,68-72;
//         cc/tiles/tiling_set_raster_queue_required.cc:1-17,163.
// Modules replace includes, guards, export and namespace. Source non-owning
// tiling/tile pointers use typed identities with per-call resident resolution.
use super::{
    prioritized_tile::PrioritizedTile,
    raster_tile_priority_queue::{PictureLayerTilingSetId, RasterQueueContext, Type},
    tile_priority::{TileResolution, WhichTree},
};
use crate::{
    adapter::tile::{PictureLayerTilingId, TileId},
    cc::base::tiling_data::Iterator,
};
use std::marker::PhantomData;

// Source: cc/tiles/tiling_set_raster_queue_required.h:24-31,60-67.
// The default C++ ctor leaves type_ uninitialized. None represents that
// inaccessible value on the empty-queue path, rather than inventing a Type.
pub(crate) struct TilingSetRasterQueueRequired<B: PictureLayerTilingTypes> {
    iterator_: TilingIterator<B>,
    type_: Option<Type>,
}
impl<B: PictureLayerTilingTypes> TilingSetRasterQueueRequired<B> {
    // Source: cc/tiles/tiling_set_raster_queue_required.h:24-26;
    //         cc/tiles/tiling_set_raster_queue_required.cc:18-51.
    pub(crate) fn Create(
        set: PictureLayerTilingSetId,
        r#type: Type,
        context: &dyn RasterQueueContext<B>,
    ) -> Option<Box<Self>> {
        debug_assert_ne!(r#type as i32, Type::ALL as i32);
        let mut tiling = None;
        if r#type == Type::REQUIRED_FOR_ACTIVATION
            && context.TilingSetById(set).tree() == WhichTree::ACTIVE_TREE
        {
            let mut i = 0;
            while i < context.TilingSetById(set).num_tilings() {
                let active_tiling = context.TilingSetById(set).tiling_at(i).identity();
                let pending_twin = crate::adapter::picture_layer_tiling::WithTilingClient(context, context.TilingSetById(set).client(), |client| client.GetPendingOrActiveTwinTiling(context.TilingById(active_tiling), context, None));
                if pending_twin.is_some_and(|id| {
                    context.TilingById(id).resolution() == TileResolution::HIGH_RESOLUTION
                }) {
                    tiling = Some(active_tiling);
                    break;
                }
                i += 1;
            }
        } else {
            tiling = context
                .TilingSetById(set)
                .FindTilingWithResolution(TileResolution::HIGH_RESOLUTION);
        }
        if tiling.is_none() || context.TilingById(tiling.unwrap()).all_tiles_done() {
            if context.IsCCSlimmingEnabled() {
                return None;
            }
            return Some(Box::new(Self::new()));
        }
        Some(Box::new(Self::with_tiling(
            tiling.unwrap(),
            r#type,
            context,
        )))
    }
    // Source: cc/tiles/tiling_set_raster_queue_required.h:60;
    //         cc/tiles/tiling_set_raster_queue_required.cc:53.
    fn new() -> Self {
        Self {
            iterator_: TilingIterator::default(),
            type_: None,
        }
    }
    // Source: cc/tiles/tiling_set_raster_queue_required.h:61-62;
    //         cc/tiles/tiling_set_raster_queue_required.cc:55-72.
    fn with_tiling(
        tiling: PictureLayerTilingId,
        r#type: Type,
        context: &dyn RasterQueueContext<B>,
    ) -> Self {
        debug_assert!(!context.TilingById(tiling).all_tiles_done());
        let iterator = if r#type == Type::REQUIRED_FOR_ACTIVATION {
            TilingIterator::new(tiling, true, context)
        } else {
            TilingIterator::new(tiling, false, context)
        };
        let mut result = Self {
            iterator_: iterator,
            type_: Some(r#type),
        };
        while !result.iterator_.done() && !result.IsTileRequired(result.iterator_.get(), context) {
            result.iterator_.Advance(context);
        }
        result
    }
    // Source: cc/tiles/tiling_set_raster_queue_required.h:27;
    //         cc/tiles/tiling_set_raster_queue_required.cc:74.
    // Default destructor: only source-owned values are dropped; IDs do not own.
    // Source: cc/tiles/tiling_set_raster_queue_required.h:31;
    //         cc/tiles/tiling_set_raster_queue_required.cc:76-78.
    pub(crate) fn IsEmpty(&self) -> bool {
        self.iterator_.done()
    }
    // Source: cc/tiles/tiling_set_raster_queue_required.h:30;
    //         cc/tiles/tiling_set_raster_queue_required.cc:80-85.
    pub(crate) fn Pop(&mut self, context: &dyn RasterQueueContext<B>) {
        debug_assert!(!self.IsEmpty());
        self.iterator_.Advance(context);
        while !self.iterator_.done() && !self.IsTileRequired(self.iterator_.get(), context) {
            self.iterator_.Advance(context);
        }
    }
    // Source: cc/tiles/tiling_set_raster_queue_required.h:29;
    //         cc/tiles/tiling_set_raster_queue_required.cc:87-90.
    pub(crate) fn Top(&self) -> &PrioritizedTile {
        debug_assert!(!self.IsEmpty());
        self.iterator_.get()
    }
    // Source: cc/tiles/tiling_set_raster_queue_required.h:64;
    //         cc/tiles/tiling_set_raster_queue_required.cc:92-98.
    fn IsTileRequired(&self, tile: &PrioritizedTile, context: &dyn RasterQueueContext<B>) -> bool {
        (self.type_ == Some(Type::REQUIRED_FOR_ACTIVATION)
            && tile
                .tile::<super::picture_layer_tiling::PictureLayerTiling<B>>(context)
                .expect("source nonnull tile")
                .required_for_activation())
            || (self.type_ == Some(Type::REQUIRED_FOR_DRAW)
                && tile
                    .tile::<super::picture_layer_tiling::PictureLayerTiling<B>>(context)
                    .expect("source nonnull tile")
                    .required_for_draw())
    }
}

// Source: cc/tiles/tiling_set_raster_queue_required.h:37-58.
// tiling_data_ aliases the selected tiling's own data; retaining the tiling ID
// resolves the same object. The native Iterator owns only its cursor/IndexRect.
struct TilingIterator<B: PictureLayerTilingTypes> {
    tiling_: Option<PictureLayerTilingId>,
    current_tile_: PrioritizedTile,
    visible_iterator_: Iterator,
    source_type_: PhantomData<fn() -> B>,
}
impl<B: PictureLayerTilingTypes> Default for TilingIterator<B> {
    // Source: cc/tiles/tiling_set_raster_queue_required.h:39;
    //         cc/tiles/tiling_set_raster_queue_required.cc:100-102.
    fn default() -> Self {
        Self {
            tiling_: None,
            current_tile_: PrioritizedTile::default(),
            visible_iterator_: Iterator::default(),
            source_type_: PhantomData,
        }
    }
}
impl<B: PictureLayerTilingTypes> TilingIterator<B> {
    // Source: cc/tiles/tiling_set_raster_queue_required.h:40-42;
    //         cc/tiles/tiling_set_raster_queue_required.cc:104-125.
    fn new(
        tiling: PictureLayerTilingId,
        pending: bool,
        context: &dyn RasterQueueContext<B>,
    ) -> Self {
        let mut result = Self {
            tiling_: Some(tiling),
            current_tile_: PrioritizedTile::default(),
            visible_iterator_: {
                let selected = context.TilingById(tiling);
                if pending {
                    Iterator::new(
                        selected.tiling_data(),
                        &selected.pending_visible_rect(context),
                        false,
                    )
                } else {
                    Iterator::new(
                        selected.tiling_data(),
                        selected.current_visible_rect(),
                        false,
                    )
                }
            },
            source_type_: PhantomData,
        };
        if !result.visible_iterator_.IsValid() {
            return result;
        }
        let tile = context
            .TilingById(tiling)
            .TileAt(
                result.visible_iterator_.index_x(),
                result.visible_iterator_.index_y(),
            )
            .map(|tile| tile.identity());
        if let Some(tile) = tile {
            if context.TileById(tile).draw_info().NeedsRaster()
                && !context
                    .TilingById(tiling)
                    .IsTileOccluded(context.TileById(tile), context)
            {
                result.current_tile_ = Self::MakePrioritizedTile(tiling, tile, context);
                return result;
            }
        }
        result.Advance(context);
        result
    }
    // Source: cc/tiles/tiling_set_raster_queue_required.h:43;
    //         cc/tiles/tiling_set_raster_queue_required.cc:127.
    // Default destructor drops only cursor/priority values, never the tiling.
    // Source: cc/tiles/tiling_set_raster_queue_required.h:45.
    fn done(&self) -> bool {
        self.current_tile_.tile_id().is_none()
    }
    // Source: cc/tiles/tiling_set_raster_queue_required.h:46.
    fn get(&self) -> &PrioritizedTile {
        &self.current_tile_
    }
    // Source: cc/tiles/tiling_set_raster_queue_required.h:47;
    //         cc/tiles/tiling_set_raster_queue_required.cc:129-161.
    fn Advance(&mut self, context: &dyn RasterQueueContext<B>) -> &mut Self {
        let tiling = self.tiling_.expect("source nonnull tiling");
        let tile;
        loop {
            self.visible_iterator_.Advance();
            if !self.visible_iterator_.IsValid() {
                self.current_tile_ = PrioritizedTile::default();
                return self;
            }
            let next_index = self.visible_iterator_.index();
            let candidate = context
                .TilingById(tiling)
                .TileAt(next_index.0, next_index.1)
                .map(|tile| tile.identity());
            if candidate.is_none()
                || !context
                    .TileById(candidate.unwrap())
                    .draw_info()
                    .NeedsRaster()
            {
                continue;
            }
            let candidate = candidate.unwrap();
            if context
                .TilingById(tiling)
                .IsTileOccluded(context.TileById(candidate), context)
            {
                continue;
            }
            tile = candidate;
            break;
        }
        self.current_tile_ = Self::MakePrioritizedTile(tiling, tile, context);
        self
    }
    // Borrow-only call expression shared by source cc120-121 and158-159.
    // Priority rect is computed before the canonical same-package tiling call.
    fn MakePrioritizedTile(
        tiling: PictureLayerTilingId,
        tile: TileId,
        context: &dyn RasterQueueContext<B>,
    ) -> PrioritizedTile {
        let rect_type = context
            .TilingById(tiling)
            .ComputePriorityRectTypeForTile(context.TileById(tile), context);
        super::raster_tile_priority_queue::MakePrioritizedTile(
            context, tiling, tile, rect_type, false,
        )
    }
}
