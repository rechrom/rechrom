// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/eviction_tile_priority_queue.h:5-21
// Source: cc/tiles/eviction_tile_priority_queue.cc:5-12
use super::{
    picture_layer_tiling::PictureLayerTiling,
    prioritized_tile::PrioritizedTile,
    raster_tile_priority_queue::RasterQueueLayer,
    raster_tile_priority_queue_all::{MakeHeap, PopHeap, PushHeap},
    tile_priority::TileResolution,
    tiling_set_eviction_queue::{EvictionQueueContext, TilingSetEvictionQueue},
    WhichTree,
};
use crate::picture_layer_tiling_types::PictureLayerTilingTypes;
use crate::prioritized_tile_types::PrioritizedTileResidentExt;

// Source: cc/tiles/eviction_tile_priority_queue.cc:13-58
struct EvictionOrderComparator;
impl EvictionOrderComparator {
    fn compare<B: PictureLayerTilingTypes>(
        a_queue: &Box<TilingSetEvictionQueue<B>>,
        b_queue: &Box<TilingSetEvictionQueue<B>>,
    ) -> bool {
        let a_tile = a_queue.Top();
        let b_tile = b_queue.Top();
        let a_priority = a_tile.priority();
        let b_priority = b_tile.priority();
        if b_priority.priority_bin == a_priority.priority_bin
            && b_queue.is_drawing_layer() != a_queue.is_drawing_layer()
        {
            return a_queue.is_drawing_layer();
        }
        if a_priority.priority_bin != b_priority.priority_bin {
            return b_priority.priority_bin > a_priority.priority_bin;
        }
        if b_priority.resolution != a_priority.resolution {
            return b_priority.resolution == TileResolution::NON_IDEAL_RESOLUTION;
        }
        let a_is_occluded = a_tile.is_occluded();
        let b_is_occluded = b_tile.is_occluded();
        if a_is_occluded != b_is_occluded {
            return b_is_occluded;
        }
        b_priority.distance_to_visible > a_priority.distance_to_visible
    }
}

// Source: cc/tiles/eviction_tile_priority_queue.cc:60-77
fn CreateTilingSetEvictionQueues<B, L, R>(
    layers: R,
    queues: &mut Vec<Box<TilingSetEvictionQueue<B>>>,
    context: &dyn EvictionQueueContext<B>,
) where
    B: PictureLayerTilingTypes,
    L: RasterQueueLayer<B>,
    R: IntoIterator<Item = L>,
{
    debug_assert!(queues.is_empty());
    for layer in layers {
        let tiling_set = layer.picture_layer_tiling_set().identity();
        let tiling_set_queue = Box::new(TilingSetEvictionQueue::new(
            tiling_set,
            layer.contributes_to_drawn_render_surface(),
            context,
        ));
        if !tiling_set_queue.IsEmpty() {
            queues.push(tiling_set_queue);
        }
    }
    MakeHeap(queues, &EvictionOrderComparator::compare);
}

// Source: cc/tiles/eviction_tile_priority_queue.h:22-45
// Unique queue ownership stays Box. Reverse member declaration order preserves
// native destruction of pending before active. No Clone implements the deleted
// copy constructor/assignment. Only resident identities and canonical queue
// state persist; no layer/tiling/raster-source snapshot crosses this boundary.
pub struct EvictionTilePriorityQueue<B: PictureLayerTilingTypes> {
    pending_queues_: Vec<Box<TilingSetEvictionQueue<B>>>,
    active_queues_: Vec<Box<TilingSetEvictionQueue<B>>>,
}
impl<B: PictureLayerTilingTypes> EvictionTilePriorityQueue<B> {
    // Source: cc/tiles/eviction_tile_priority_queue.h:24-29
    // Source: cc/tiles/eviction_tile_priority_queue.cc:79-81
    pub fn new() -> Self {
        Self {
            pending_queues_: Vec::new(),
            active_queues_: Vec::new(),
        }
    }
    // Source: cc/tiles/eviction_tile_priority_queue.h:31-32
    // Source: cc/tiles/eviction_tile_priority_queue.cc:83-87
    pub fn Build<L, R>(
        &mut self,
        active_layers: R,
        pending_layers: R,
        context: &dyn EvictionQueueContext<B>,
    ) where
        L: RasterQueueLayer<B>,
        R: IntoIterator<Item = L>,
    {
        CreateTilingSetEvictionQueues(active_layers, &mut self.active_queues_, context);
        CreateTilingSetEvictionQueues(pending_layers, &mut self.pending_queues_, context);
    }
    // Source: cc/tiles/eviction_tile_priority_queue.h:34
    // Source: cc/tiles/eviction_tile_priority_queue.cc:89-91
    pub fn IsEmpty(&self) -> bool {
        self.active_queues_.is_empty() && self.pending_queues_.is_empty()
    }
    // Source: cc/tiles/eviction_tile_priority_queue.h:35
    // Source: cc/tiles/eviction_tile_priority_queue.cc:93-97
    pub fn Top(&self, context: &dyn EvictionQueueContext<B>) -> &PrioritizedTile {
        debug_assert!(!self.IsEmpty());
        let next_queues = self.GetNextQueues(context);
        next_queues.first().unwrap().Top()
    }
    // Source: cc/tiles/eviction_tile_priority_queue.h:36
    // Source: cc/tiles/eviction_tile_priority_queue.cc:99-115
    pub fn Pop(&mut self, context: &dyn EvictionQueueContext<B>) {
        debug_assert!(!self.IsEmpty());
        let next_queues = self.GetNextQueuesMut(context);
        PopHeap(next_queues, &EvictionOrderComparator::compare);
        let queue = next_queues.last_mut().unwrap();
        queue.Pop(context);
        if queue.IsEmpty() {
            next_queues.pop();
        } else {
            PushHeap(next_queues, &EvictionOrderComparator::compare);
        }
    }
    // Source: cc/tiles/eviction_tile_priority_queue.h:39
    // Source: cc/tiles/eviction_tile_priority_queue.cc:117-124
    // The native const_cast maps to one shared selection body, followed by a
    // mutable borrow of the selected resident member vector, never a copy.
    fn GetNextQueuesMut(
        &mut self,
        context: &dyn EvictionQueueContext<B>,
    ) -> &mut Vec<Box<TilingSetEvictionQueue<B>>> {
        match self.NextQueuesTree(context) {
            WhichTree::ACTIVE_TREE => &mut self.active_queues_,
            WhichTree::PENDING_TREE => &mut self.pending_queues_,
        }
    }
    // Source: cc/tiles/eviction_tile_priority_queue.h:40-41
    // Source: cc/tiles/eviction_tile_priority_queue.cc:126-155
    fn GetNextQueues(
        &self,
        context: &dyn EvictionQueueContext<B>,
    ) -> &Vec<Box<TilingSetEvictionQueue<B>>> {
        match self.NextQueuesTree(context) {
            WhichTree::ACTIVE_TREE => &self.active_queues_,
            WhichTree::PENDING_TREE => &self.pending_queues_,
        }
    }
    // Source: cc/tiles/eviction_tile_priority_queue.cc:126-155
    // Rust borrow split: the native const body runs once for both projections.
    // required_for_activation is read from the real Tile on every selection.
    fn NextQueuesTree(&self, context: &dyn EvictionQueueContext<B>) -> WhichTree {
        debug_assert!(!self.IsEmpty());
        if self.active_queues_.is_empty() {
            return WhichTree::PENDING_TREE;
        }
        if self.pending_queues_.is_empty() {
            return WhichTree::ACTIVE_TREE;
        }
        let active_tile = self.active_queues_.first().unwrap().Top();
        let pending_tile = self.pending_queues_.first().unwrap().Top();
        let active_priority = active_tile.priority();
        let pending_priority = pending_tile.priority();
        if active_priority.priority_bin == pending_priority.priority_bin
            && active_tile
                .tile::<PictureLayerTiling<B>>(context)
                .expect("source nonnull active tile")
                .required_for_activation()
                != pending_tile
                    .tile::<PictureLayerTiling<B>>(context)
                    .expect("source nonnull pending tile")
                    .required_for_activation()
        {
            return if active_tile
                .tile::<PictureLayerTiling<B>>(context)
                .expect("source nonnull active tile")
                .required_for_activation()
            {
                WhichTree::PENDING_TREE
            } else {
                WhichTree::ACTIVE_TREE
            };
        }
        if pending_priority.IsHigherPriorityThan(active_priority) {
            return WhichTree::ACTIVE_TREE;
        }
        WhichTree::PENDING_TREE
    }
}
// Source: cc/tiles/eviction_tile_priority_queue.h:47-49
// Source: cc/tiles/eviction_tile_priority_queue.cc:157
// Namespace/include guards/default vector+Box destructors map directly to Rust.
