// Translation status: complete
// Copyright 2017 The Chromium Authors. BSD-style license in LICENSE.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.h:1-29;
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.cc:1-16.
// Includes/GC tracing/namespace map to modules and the Paint node's explicit
// opaque cache allocation. Key Weak references only compare node identity;
// the nearest-filter member is held strongly (CachedClip), as Blink traces it.
use super::float_clip_rect::FloatClipRect;
use super::geometry_mapper_transform_cache::CacheTreeScope;
use super::native::NativeTrees;
use crate::paint_property_tree::{ClipPaintPropertyNode, TransformPaintPropertyNode};
pub use foundation::OverlayScrollbarClipBehavior;
use std::sync::{Arc, Mutex, Weak};
// Source: geometry_mapper_clip_cache.h:65-67 `s_global_generation`, kept per
// document sequence (paint_model::property_cell).
fn global_generation() -> u32 {
    crate::property_cell::ClipCacheGeneration()
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
    nearest_pixel_moving_filter_clip_: Option<CachedClip>,
    cache_generation_: u32,
    /// The published property trees this cache was computed in (see
    /// CacheTreeScope): its entries were mapped through that tree's ancestors.
    tree_scope_: CacheTreeScope,
}
/// A cached Member<const ClipPaintPropertyNode> (traced in Blink, so it keeps
/// the node alive): the clip owning this cache, which an Arc cannot hold
/// without a cycle, or an ancestor clip held strongly.
#[derive(Clone, Debug)]
enum CachedClip {
    Owner,
    Node(Arc<ClipPaintPropertyNode>),
}
impl CachedClip {
    fn resolve(&self, owner: &Arc<ClipPaintPropertyNode>) -> Arc<ClipPaintPropertyNode> {
        match self {
            Self::Owner => owner.clone(),
            Self::Node(node) => node.clone(),
        }
    }
}
impl Default for GeometryMapperClipCache {
    fn default() -> Self {
        Self {
            clip_cache_: Vec::new(),
            nearest_pixel_moving_filter_clip_: None,
            cache_generation_: global_generation().wrapping_sub(1),
            tree_scope_: CacheTreeScope::default(),
        }
    }
}
impl GeometryMapperClipCache {
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.h:53-57.
    pub fn UpdateIfNeeded(
        &mut self,
        node: &Arc<ClipPaintPropertyNode>,
        trees: &NativeTrees,
        parent_of: ClipParentFn<'_>,
    ) {
        if self.cache_generation_ != global_generation() || !self.tree_scope_.is(trees) {
            self.Update(node, trees, parent_of);
        }
        debug_assert_eq!(self.cache_generation_, global_generation());
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.cc:28-34.
    pub fn ClearCache() {
        crate::property_cell::ClearClipCache();
    }
    pub fn IsValid(&self) -> bool {
        self.cache_generation_ == global_generation()
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.cc:36-50.
    fn Update(
        &mut self,
        node: &Arc<ClipPaintPropertyNode>,
        trees: &NativeTrees,
        parent_of: ClipParentFn<'_>,
    ) {
        self.cache_generation_ = global_generation();
        self.tree_scope_ = CacheTreeScope::of(trees);
        self.clip_cache_.clear();
        if crate::property_cell::with_access(|access| node.PixelMovingFilter(access).is_some()) {
            self.nearest_pixel_moving_filter_clip_ = Some(CachedClip::Owner);
        } else if let Some(parent) = parent_of(node) {
            self.nearest_pixel_moving_filter_clip_ = WithClipCacheBy(&parent, trees, parent_of, |cache| {
                cache
                    .nearest_pixel_moving_filter_clip_
                    .as_ref()
                    .map(|member| CachedClip::Node(member.resolve(&parent)))
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
    /// `owner` is the clip whose cache this is.
    pub fn NearestPixelMovingFilterClip(
        &self,
        owner: &Arc<ClipPaintPropertyNode>,
    ) -> Option<Arc<ClipPaintPropertyNode>> {
        self.nearest_pixel_moving_filter_clip_
            .as_ref()
            .map(|member| member.resolve(owner))
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
    WithClipCacheBy(
        node,
        &None,
        &|clip| crate::property_cell::with_access(|access| clip.Parent(access).cloned()),
        callback,
    )
}
/// A clip's current parent (a resolver over its parent link).
pub type ClipParentFn<'a> =
    &'a dyn Fn(&Arc<ClipPaintPropertyNode>) -> Option<Arc<ClipPaintPropertyNode>>;
/// WithClipCache for a query in `trees`, whose update follows parents through
/// `parent_of`.
pub fn WithClipCacheBy<R>(
    node: &Arc<ClipPaintPropertyNode>,
    trees: &NativeTrees,
    parent_of: ClipParentFn<'_>,
    callback: impl FnOnce(&mut GeometryMapperClipCache) -> R,
) -> R {
    let allocation = node
        .geometry_mapper_cache
        .get_or_init(|| Mutex::new(GeometryMapperClipCache::default()));
    let mut cache = allocation.lock().expect("Paint clip cache poisoned");
    cache.UpdateIfNeeded(node, trees, parent_of);
    callback(&mut cache)
}
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.cc:18-26,71.
// GC Trace visits members; native Arc property owners retain their ancestors.
// Cached members are strong (CachedClip); key Weak links only compare identity.
