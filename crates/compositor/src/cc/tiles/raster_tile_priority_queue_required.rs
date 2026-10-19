// Translation status: complete
use crate::adapter::picture_layer_tiling::PictureLayerTilingTypes;
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/raster_tile_priority_queue_required.h:1-20,46-48;
//         cc/tiles/raster_tile_priority_queue_required.cc:1-16,39-41,101.
use super::tiling_set_raster_queue_required::TilingSetRasterQueueRequired;
use super::{
    prioritized_tile::PrioritizedTile,
    raster_tile_priority_queue::{
        RasterQueueContext, RasterQueueLayer, RasterTilePriorityQueue, Type,
    },
};
use std::marker::PhantomData;

// Source: cc/tiles/raster_tile_priority_queue_required.cc:17-37.
fn AppendTilingSetRequiredQueues<
    B: PictureLayerTilingTypes,
    L: RasterQueueLayer<B>,
    R: IntoIterator<Item = L>,
>(
    layers: R,
    queues: &mut Vec<Box<TilingSetRasterQueueRequired<B>>>,
    context: &dyn RasterQueueContext<B>,
) {
    let cc_slimming_enabled = context.IsCCSlimmingEnabled();
    for layer in layers {
        if !layer.HasValidTilePriorities() {
            continue;
        }
        let tiling_set = layer.picture_layer_tiling_set();
        if cc_slimming_enabled && tiling_set.all_tiles_done() {
            continue;
        }
        let tiling_set_queue = TilingSetRasterQueueRequired::Create(
            tiling_set.identity(),
            Type::REQUIRED_FOR_ACTIVATION,
            context,
        );
        if let Some(tiling_set_queue) = tiling_set_queue {
            if !tiling_set_queue.IsEmpty() {
                queues.push(tiling_set_queue);
            }
        }
    }
}

// Source: cc/tiles/raster_tile_priority_queue_required.h:20-44.
// Native unique_ptr ownership remains Box; each inner queue is now the same
// package canonical concrete type. Only source non-owning identities persist.
pub(crate) struct RasterTilePriorityQueueRequired<B: PictureLayerTilingTypes> {
    tiling_set_queues_: Vec<Box<TilingSetRasterQueueRequired<B>>>,
    source_type_: PhantomData<fn() -> B>,
}
impl<B: PictureLayerTilingTypes> RasterTilePriorityQueueRequired<B> {
    // Source: cc/tiles/raster_tile_priority_queue_required.h:22-28;
    //         cc/tiles/raster_tile_priority_queue_required.cc:41-43.
    // Empty constructor/default destructor; deleted copy maps to no Clone.
    pub(crate) fn new() -> Self {
        Self {
            tiling_set_queues_: Vec::new(),
            source_type_: PhantomData,
        }
    }
    // Source: cc/tiles/raster_tile_priority_queue_required.h:35-41;
    //         cc/tiles/raster_tile_priority_queue_required.cc:45-56.
    pub(crate) fn Build<L: RasterQueueLayer<B>, R: IntoIterator<Item = L>>(
        &mut self,
        active_layers: R,
        pending_layers: R,
        r#type: Type,
        context: &dyn RasterQueueContext<B>,
    ) {
        debug_assert_ne!(r#type as i32, Type::ALL as i32);
        if r#type == Type::REQUIRED_FOR_DRAW {
            self.BuildRequiredForDraw(active_layers, context);
        } else {
            self.BuildRequiredForActivation(active_layers, pending_layers, context);
        }
    }
    // Source: cc/tiles/raster_tile_priority_queue_required.cc:58-76.
    fn BuildRequiredForDraw<L: RasterQueueLayer<B>, R: IntoIterator<Item = L>>(
        &mut self,
        active_layers: R,
        context: &dyn RasterQueueContext<B>,
    ) {
        let cc_slimming_enabled = context.IsCCSlimmingEnabled();
        for layer in active_layers {
            if !layer.HasValidTilePriorities() {
                continue;
            }
            let tiling_set = layer.picture_layer_tiling_set();
            if cc_slimming_enabled && tiling_set.all_tiles_done() {
                continue;
            }
            let tiling_set_queue = TilingSetRasterQueueRequired::Create(
                tiling_set.identity(),
                Type::REQUIRED_FOR_DRAW,
                context,
            );
            if let Some(tiling_set_queue) = tiling_set_queue {
                if !tiling_set_queue.IsEmpty() {
                    self.tiling_set_queues_.push(tiling_set_queue);
                }
            }
        }
    }
    // Source: cc/tiles/raster_tile_priority_queue_required.cc:78-83.
    fn BuildRequiredForActivation<L: RasterQueueLayer<B>, R: IntoIterator<Item = L>>(
        &mut self,
        active_layers: R,
        pending_layers: R,
        context: &dyn RasterQueueContext<B>,
    ) {
        AppendTilingSetRequiredQueues(active_layers, &mut self.tiling_set_queues_, context);
        AppendTilingSetRequiredQueues(pending_layers, &mut self.tiling_set_queues_, context);
    }
}
impl<B: PictureLayerTilingTypes> RasterTilePriorityQueue<B> for RasterTilePriorityQueueRequired<B> {
    // Source: cc/tiles/raster_tile_priority_queue_required.h:29-31;
    //         cc/tiles/raster_tile_priority_queue_required.cc:85-87.
    fn IsEmpty(&self) -> bool {
        self.tiling_set_queues_.is_empty()
    }
    // Source: cc/tiles/raster_tile_priority_queue_required.cc:89-92.
    fn Top(&self) -> &PrioritizedTile {
        debug_assert!(!self.IsEmpty());
        self.tiling_set_queues_
            .last()
            .expect("source nonempty required queue")
            .Top()
    }
    // Source: cc/tiles/raster_tile_priority_queue_required.cc:94-99.
    fn Pop(&mut self, context: &dyn RasterQueueContext<B>) {
        debug_assert!(!self.IsEmpty());
        self.tiling_set_queues_
            .last_mut()
            .expect("source nonempty required queue")
            .Pop(context);
        if self.tiling_set_queues_.last().unwrap().IsEmpty() {
            self.tiling_set_queues_.pop();
        }
    }
}
