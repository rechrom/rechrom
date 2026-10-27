// Translation status: complete
use crate::picture_layer_tiling_types::PictureLayerTilingTypes;
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/raster_tile_priority_queue_all.h:1-23,25-30,35-36,49-53;
//         cc/tiles/raster_tile_priority_queue_all.cc:1-17,75,172.
use super::{
    prioritized_tile::PrioritizedTile,
    raster_tile_priority_queue::{RasterQueueContext, RasterQueueLayer, RasterTilePriorityQueue},
    tile_priority::{PriorityBin, TileResolution},
    tiling_set_raster_queue_all::TilingSetRasterQueueAll,
    TreePriority, WhichTree,
};

// Source: cc/tiles/raster_tile_priority_queue_all.cc:18-48.
struct RasterOrderComparator;
impl RasterOrderComparator {
    fn compare<B: PictureLayerTilingTypes>(
        a_queue: &TilingSetRasterQueueAll<B>,
        b_queue: &TilingSetRasterQueueAll<B>,
    ) -> bool {
        let a_priority = a_queue.Top().priority();
        let b_priority = b_queue.Top().priority();
        if b_priority.priority_bin == a_priority.priority_bin
            && b_queue.is_drawing_layer() != a_queue.is_drawing_layer()
        {
            return b_queue.is_drawing_layer();
        }
        if b_priority.priority_bin == a_priority.priority_bin
            && b_priority.resolution != a_priority.resolution
        {
            return a_priority.resolution == TileResolution::NON_IDEAL_RESOLUTION;
        }
        b_priority.IsHigherPriorityThan(a_priority)
    }
}
// Source: cc/tiles/raster_tile_priority_queue_all.cc:50-73.
fn CreateTilingSetRasterQueues<
    B: PictureLayerTilingTypes,
    L: RasterQueueLayer<B>,
    R: IntoIterator<Item = L>,
>(
    layers: R,
    queues: &mut Vec<Box<TilingSetRasterQueueAll<B>>>,
    context: &dyn RasterQueueContext<B>,
) {
    debug_assert!(queues.is_empty());
    let cc_slimming_enabled = context.IsCCSlimmingEnabled();
    for layer in layers {
        if !layer.HasValidTilePriorities() {
            continue;
        }
        let tiling_set = layer.picture_layer_tiling_set();
        if cc_slimming_enabled && tiling_set.all_tiles_done() {
            continue;
        }
        let tiling_set = tiling_set.identity();
        let tiling_set_queue = TilingSetRasterQueueAll::Create(
            tiling_set,
            layer.contributes_to_drawn_render_surface(),
            context,
        );
        if let Some(queue) = tiling_set_queue {
            if !queue.IsEmpty() {
                queues.push(queue);
            }
        }
    }
    MakeHeap(queues, &|a, b| RasterOrderComparator::compare(a, b));
}

// Source: cc/tiles/raster_tile_priority_queue_all.h:38-48.
// unique_ptr ownership stays Box; no layer/tiling snapshots persist here.
pub struct RasterTilePriorityQueueAll<B: PictureLayerTilingTypes> {
    active_queues_: Vec<Box<TilingSetRasterQueueAll<B>>>,
    pending_queues_: Vec<Box<TilingSetRasterQueueAll<B>>>,
    tree_priority_: Option<TreePriority>,
}
impl<B: PictureLayerTilingTypes> RasterTilePriorityQueueAll<B> {
    // Source: cc/tiles/raster_tile_priority_queue_all.h:24,26;
    //         cc/tiles/raster_tile_priority_queue_all.cc:77-78.
    // None represents the source default ctor's unread uninitialized enum.
    // Default Box/vector drop is the virtual destructor; no Clone maps deleted copy.
    pub fn new() -> Self {
        Self {
            active_queues_: Vec::new(),
            pending_queues_: Vec::new(),
            tree_priority_: None,
        }
    }
    // Source: cc/tiles/raster_tile_priority_queue_all.h:38-40;
    //         cc/tiles/raster_tile_priority_queue_all.cc:80-87.
    pub fn Build<L: RasterQueueLayer<B>, R: IntoIterator<Item = L>>(
        &mut self,
        active_layers: R,
        pending_layers: R,
        tree_priority: TreePriority,
        context: &dyn RasterQueueContext<B>,
    ) {
        self.tree_priority_ = Some(tree_priority);
        CreateTilingSetRasterQueues(active_layers, &mut self.active_queues_, context);
        CreateTilingSetRasterQueues(pending_layers, &mut self.pending_queues_, context);
    }
    // Source: cc/tiles/raster_tile_priority_queue_all.h:42-44;
    //         cc/tiles/raster_tile_priority_queue_all.cc:117-123.
    // The C++ const_cast becomes selection by the same native body below;
    // only the final borrow changes mutability. No queue aggregate is copied.
    fn GetNextQueuesMut(&mut self) -> &mut Vec<Box<TilingSetRasterQueueAll<B>>> {
        match self.NextQueuesTree() {
            WhichTree::ACTIVE_TREE => &mut self.active_queues_,
            WhichTree::PENDING_TREE => &mut self.pending_queues_,
        }
    }
    // Source: cc/tiles/raster_tile_priority_queue_all.h:43-44;
    //         cc/tiles/raster_tile_priority_queue_all.cc:125-170.
    fn GetNextQueues(&self) -> &Vec<Box<TilingSetRasterQueueAll<B>>> {
        match self.NextQueuesTree() {
            WhichTree::ACTIVE_TREE => &self.active_queues_,
            WhichTree::PENDING_TREE => &self.pending_queues_,
        }
    }
    // Borrow-only selection from the single source const GetNextQueues body.
    // Source: cc/tiles/raster_tile_priority_queue_all.cc:125-170.
    fn NextQueuesTree(&self) -> WhichTree {
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
        if pending_priority.priority_bin == PriorityBin::NOW
            && active_priority.priority_bin == PriorityBin::NOW
        {
            if self.tree_priority_ == Some(TreePriority::SMOOTHNESS_TAKES_PRIORITY) {
                return WhichTree::ACTIVE_TREE;
            }
            if self.tree_priority_ == Some(TreePriority::NEW_CONTENT_TAKES_PRIORITY) {
                return WhichTree::PENDING_TREE;
            }
        }
        if active_priority.IsHigherPriorityThan(pending_priority) {
            return WhichTree::ACTIVE_TREE;
        }
        WhichTree::PENDING_TREE
    }
}
impl<B: PictureLayerTilingTypes> RasterTilePriorityQueue<B> for RasterTilePriorityQueueAll<B> {
    // Source: cc/tiles/raster_tile_priority_queue_all.h:31;
    //         cc/tiles/raster_tile_priority_queue_all.cc:89-91.
    fn IsEmpty(&self) -> bool {
        self.active_queues_.is_empty() && self.pending_queues_.is_empty()
    }
    // Source: cc/tiles/raster_tile_priority_queue_all.h:32;
    //         cc/tiles/raster_tile_priority_queue_all.cc:93-97.
    fn Top(&self) -> &PrioritizedTile {
        debug_assert!(!self.IsEmpty());
        let next_queues = self.GetNextQueues();
        next_queues.first().unwrap().Top()
    }
    // Source: cc/tiles/raster_tile_priority_queue_all.h:33;
    //         cc/tiles/raster_tile_priority_queue_all.cc:99-115.
    fn Pop(&mut self, context: &dyn RasterQueueContext<B>) {
        debug_assert!(!self.IsEmpty());
        let next_queues = self.GetNextQueuesMut();
        PopHeap(next_queues, &|a, b| RasterOrderComparator::compare(a, b));
        let queue = next_queues.last_mut().unwrap();
        queue.Pop(context);
        if queue.IsEmpty() {
            next_queues.pop();
        } else {
            PushHeap(next_queues, &|a, b| RasterOrderComparator::compare(a, b));
        }
    }
}

// Native std::heap primitives, parameterized by values/comparator. These are
// the fixed Chromium libc++ non-arithmetic path; swaps implement its hole
// moves without unsafe or cloning any queue. Strict comparator/ties and every
// comparison point match the source library invoked by cc72/103/112 above.
// Source: third_party/libc++/src/include/__algorithm/sift_down.h:27-81.
// LLVM implementation portions: Apache-2.0 WITH LLVM-exception.
fn SiftDown<M, C: Fn(&M, &M) -> bool + ?Sized>(queues: &mut [M], mut start: usize, compare: &C) {
    let len = queues.len();
    let mut child = start;
    if len < 2 || (len - 2) / 2 < child {
        return;
    }
    child = 2 * child + 1;
    if child + 1 < len && compare(&queues[child], &queues[child + 1]) {
        child += 1;
    }
    if compare(&queues[child], &queues[start]) {
        return;
    }
    loop {
        queues.swap(start, child);
        start = child;
        if (len - 2) / 2 < child {
            break;
        }
        child = 2 * child + 1;
        if child + 1 < len && compare(&queues[child], &queues[child + 1]) {
            child += 1;
        }
        if compare(&queues[child], &queues[start]) {
            break;
        }
    }
}
// Source: third_party/libc++/src/include/__algorithm/make_heap.h:31-54.
// Box is non-arithmetic: __assume_both_children is false.
pub fn MakeHeap<M, C: Fn(&M, &M) -> bool + ?Sized>(queues: &mut [M], compare: &C) {
    let n = queues.len();
    if n > 1 {
        for start in (0..=(n - 2) / 2).rev() {
            SiftDown(queues, start, compare);
        }
    }
}
// Source: third_party/libc++/src/include/__algorithm/push_heap.h:30-63.
pub fn PushHeap<M, C: Fn(&M, &M) -> bool + ?Sized>(queues: &mut [M], compare: &C) {
    let len = queues.len();
    if len > 1 {
        let mut parent = (len - 2) / 2;
        let mut last = len - 1;
        if compare(&queues[parent], &queues[last]) {
            loop {
                queues.swap(last, parent);
                last = parent;
                if parent == 0 {
                    break;
                }
                parent = (parent - 1) / 2;
                if !compare(&queues[parent], &queues[last]) {
                    break;
                }
            }
        }
    }
}
// Source: third_party/libc++/src/include/__algorithm/pop_heap.h:33-59;
//         third_party/libc++/src/include/__algorithm/sift_down.h:83-113.
pub fn PopHeap<M, C: Fn(&M, &M) -> bool + ?Sized>(queues: &mut [M], compare: &C) {
    let len = queues.len();
    debug_assert!(len > 0);
    if len > 1 {
        let mut hole = 0;
        let mut child = 0;
        loop {
            child = 2 * child + 1;
            if child + 1 < len && compare(&queues[child], &queues[child + 1]) {
                child += 1;
            }
            queues.swap(hole, child);
            hole = child;
            if child > (len - 2) / 2 {
                break;
            }
        }
        let last = len - 1;
        if hole != last {
            queues.swap(hole, last);
            PushHeap(&mut queues[..hole + 1], compare);
        }
    }
}
