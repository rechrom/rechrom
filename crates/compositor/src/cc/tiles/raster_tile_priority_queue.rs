// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/raster_tile_priority_queue.h:1-19,42-44;
//         cc/tiles/raster_tile_priority_queue.cc:1-16,42.
// Rust modules replace includes, forward declarations and namespaces.
use super::{
    picture_layer_tiling::PictureLayerTiling, picture_layer_tiling_set::PictureLayerTilingSet,
};
use super::{prioritized_tile::PrioritizedTile, tile_priority::TreePriority};
use crate::adapter::picture_layer_tiling::{PictureLayerTilingQueries, PictureLayerTilingTypes};
use crate::adapter::{
    prioritized_tile::PrioritizedTileContext,
    tile::{PictureLayerTilingId, TileId},
};

// Source: cc/tiles/raster_tile_priority_queue.h:21.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Type {
    ALL = 0,
    REQUIRED_FOR_ACTIVATION = 1,
    REQUIRED_FOR_DRAW = 2,
}

// Source: cc/tiles/raster_tile_priority_queue.h:29-39.
// Dynamic Box drop is the virtual destructor. Top borrows the resident entry.
// Pop receives only a call-scoped resolver for the source non-owning pointers.
pub trait RasterTilePriorityQueue<B: PictureLayerTilingTypes> {
    fn IsEmpty(&self) -> bool;
    fn Top(&self) -> &PrioritizedTile;
    fn Pop(&mut self, context: &dyn RasterQueueContext<B>);
}

// Source non-owning Set identity. Set/Tiling themselves are canonical owners.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PictureLayerTilingSetId(pub u64);
pub(crate) use super::picture_layer_tiling::PriorityRectType;
pub(crate) trait RasterQueueContext<B: PictureLayerTilingTypes>:
    PictureLayerTilingQueries<B> + PrioritizedTileContext<PictureLayerTiling<B>>
{
    fn TilingSetById(&self, id: PictureLayerTilingSetId) -> &PictureLayerTilingSet<B>;
    fn IsCCSlimmingEnabled(&self) -> bool;
}
pub(crate) fn MakePrioritizedTile<B: PictureLayerTilingTypes>(
    context: &dyn RasterQueueContext<B>,
    tiling: PictureLayerTilingId,
    tile: TileId,
    kind: PriorityRectType,
    occluded: bool,
) -> PrioritizedTile {
    context
        .TilingById(tiling)
        .MakePrioritizedTile(tile, kind, occluded, context)
}
pub(crate) trait RasterQueueLayer<B: PictureLayerTilingTypes> {
    fn HasValidTilePriorities(&self) -> bool;
    fn contributes_to_drawn_render_surface(&self) -> bool;
    fn picture_layer_tiling_set(&self) -> &PictureLayerTilingSet<B>;
}
impl<B: PictureLayerTilingTypes + 'static> dyn RasterTilePriorityQueue<B> {
    // Source: cc/tiles/raster_tile_priority_queue.h:23-27;
    //         cc/tiles/raster_tile_priority_queue.cc:17-40.
    pub(crate) fn Create<L: RasterQueueLayer<B>, R: IntoIterator<Item = L>>(
        active_layers: R,
        pending_layers: R,
        tree_priority: TreePriority,
        r#type: Type,
        context: &dyn RasterQueueContext<B>,
    ) -> Box<dyn RasterTilePriorityQueue<B>> {
        match r#type {
            Type::ALL => {
                let mut queue = Box::new(
                    super::raster_tile_priority_queue_all::RasterTilePriorityQueueAll::<B>::new(),
                );
                queue.Build(active_layers, pending_layers, tree_priority, context);
                queue
            }
            Type::REQUIRED_FOR_ACTIVATION | Type::REQUIRED_FOR_DRAW => {
                let mut queue =
                    Box::new(
                        super::raster_tile_priority_queue_required::RasterTilePriorityQueueRequired::<
                            B,
                        >::new(),
                    );
                queue.Build(active_layers, pending_layers, r#type, context);
                queue
            }
        }
        // Exhaustive source Type makes the final NOTREACHED impossible.
    }
}
