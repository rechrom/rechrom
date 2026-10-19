// Translation status: complete
// Copyright 2017 The Chromium Authors. BSD-style license in LICENSE.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.h:1-29;
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.cc:1-16.
// Includes/GC tracing/namespace map to modules and the Paint node's explicit
// opaque cache allocation. Weak references model non-owning GC node members,
// including a nearest-filter reference to the owning node itself.
use super::float_clip_rect::FloatClipRect;
use paint::paint_property_tree::{ClipPaintPropertyNode, TransformPaintPropertyNode};
use std::{
    cell::Cell,
    sync::{Arc, Mutex, Weak},
};
// Source single-Paint-sequence static: each runtime sequence has its own
// generation; Pages on different threads never share a mutable generation.
thread_local! { static S_GLOBAL_GENERATION: Cell<u32> = const { Cell::new(1) }; }
fn global_generation() -> u32 {
    S_GLOBAL_GENERATION.with(Cell::get)
}
// Source: third_party/blink/renderer/platform/graphics/overlay_scrollbar_clip_behavior.h:10-15.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayScrollbarClipBehavior {
    kIgnoreOverlayScrollbarSize,
    kExcludeOverlayScrollbarSizeForHitTesting,
}
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.h:31-50.
#[derive(Clone, Debug)]
pub struct ClipAndTransform {
    pub ancestor_clip: Weak<ClipPaintPropertyNode>,
    pub ancestor_transform: Weak<TransformPaintPropertyNode>,
    pub clip_behavior: OverlayScrollbarClipBehavior,
}
impl ClipAndTransform {
    pub fn new(
        clip: &Arc<ClipPaintPropertyNode>,
        transform: &Arc<TransformPaintPropertyNode>,
        behavior: OverlayScrollbarClipBehavior,
    ) -> Self {
        Self {
            ancestor_clip: Arc::downgrade(clip),
            ancestor_transform: Arc::downgrade(transform),
            clip_behavior: behavior,
        }
    }
}
impl PartialEq for ClipAndTransform {
    fn eq(&self, b: &Self) -> bool {
        Weak::ptr_eq(&self.ancestor_clip, &b.ancestor_clip)
            && Weak::ptr_eq(&self.ancestor_transform, &b.ancestor_transform)
            && self.clip_behavior == b.clip_behavior
    }
}
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.h:59-77.
#[derive(Clone, Debug, PartialEq)]
pub struct ClipCacheEntry {
    pub clip_and_transform: ClipAndTransform,
    pub clip_rect: FloatClipRect,
    pub has_transform_animation: bool,
    pub has_sticky_transform: bool,
}
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.h:79-111.
#[derive(Debug)]
pub struct GeometryMapperClipCache {
    clip_cache_: Vec<ClipCacheEntry>,
    nearest_pixel_moving_filter_clip_: Option<Weak<ClipPaintPropertyNode>>,
    cache_generation_: u32,
}
impl Default for GeometryMapperClipCache {
    fn default() -> Self {
        Self {
            clip_cache_: Vec::new(),
            nearest_pixel_moving_filter_clip_: None,
            cache_generation_: global_generation().wrapping_sub(1),
        }
    }
}
impl GeometryMapperClipCache {
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.h:53-57.
    pub fn UpdateIfNeeded(&mut self, node: &Arc<ClipPaintPropertyNode>) {
        if self.cache_generation_ != global_generation() {
            self.Update(node);
        }
        debug_assert_eq!(self.cache_generation_, global_generation());
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.cc:28-34.
    pub fn ClearCache() {
        S_GLOBAL_GENERATION.with(|generation| generation.set(generation.get().wrapping_add(1)));
    }
    pub fn IsValid(&self) -> bool {
        self.cache_generation_ == global_generation()
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.cc:36-50.
    fn Update(&mut self, node: &Arc<ClipPaintPropertyNode>) {
        debug_assert_ne!(self.cache_generation_, global_generation());
        self.cache_generation_ = global_generation();
        self.clip_cache_.clear();
        if node.pixel_moving_filter.is_some() {
            self.nearest_pixel_moving_filter_clip_ = Some(Arc::downgrade(node));
        } else if let Some(parent) = &node.parent {
            self.nearest_pixel_moving_filter_clip_ = WithClipCache(parent, |cache| {
                cache.nearest_pixel_moving_filter_clip_.clone()
            });
        } else {
            self.nearest_pixel_moving_filter_clip_ = None;
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.cc:52-69.
    pub fn GetCachedClip(&self, key: &ClipAndTransform) -> Option<&ClipCacheEntry> {
        debug_assert!(self.IsValid());
        for entry in &self.clip_cache_ {
            if entry.clip_and_transform == *key {
                return Some(entry);
            }
        }
        None
    }
    pub fn SetCachedClip(&mut self, entry: &ClipCacheEntry) {
        debug_assert!(self.IsValid());
        debug_assert!(self.GetCachedClip(&entry.clip_and_transform).is_none());
        self.clip_cache_.push(entry.clone());
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.h:91-93.
    pub fn NearestPixelMovingFilterClip(&self) -> Option<Arc<ClipPaintPropertyNode>> {
        self.nearest_pixel_moving_filter_clip_.as_ref().map(|node| {
            node.upgrade()
                .expect("source GC member is live while its owning node is live")
        })
    }
}
/// Source: clip_paint_property_node.h:249-258. This is the source node getter's
/// allocation/update bridge, not a second cache registry or cache algorithm.
/// The closure scope replaces a C++ mutable cache reference with a locked
/// borrow of that exact node-owned allocation. No guard or owner is retained.
pub fn WithClipCache<R>(
    node: &Arc<ClipPaintPropertyNode>,
    callback: impl FnOnce(&mut GeometryMapperClipCache) -> R,
) -> R {
    let allocation = node
        .geometry_mapper_cache
        .get_or_init(|| Mutex::new(GeometryMapperClipCache::default()));
    let mut cache = allocation.lock().expect("Paint clip cache poisoned");
    cache.UpdateIfNeeded(node);
    callback(&mut cache)
}
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.cc:18-26,71.
// GC Trace visits members; native Arc property owners retain their ancestors.
// Rust weak cached links avoid self cycles without copying or owning a tree.
