// Translation status: complete
// Copyright 2017 The Chromium Authors. BSD-style license in LICENSE.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.h:1-39;
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.cc:1-16.
// Includes/namespace map to native modules. The cache's node members are
// traced Member<> references in Blink (geometry_mapper_transform_cache.cc
// Trace), so they keep their nodes alive: here they are strong Arcs, with
// `CachedNode::Owner` for the node that owns this cache, which an Arc cannot
// hold without a cycle. The actual Paint node owns the cache allocation.
use super::matrix::{GeometryMapperMatrix, SharedGeometryTransform};
use crate::geometry_mapper_source::native::NativeTrees;
use crate::paint_property_tree::{FrozenPropertyTrees, TransformPaintPropertyNode};
use crate::{
    geometry_mapper_source::traits::TransformCacheNode,
    source_geometry::{Transform, Vector2dF},
    source_transform::PropertyTransform,
    source_transform::PropertyTreeTransform,
};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};
static S_GLOBAL_GENERATION: AtomicU32 = AtomicU32::new(1);
fn global_generation() -> u32 {
    S_GLOBAL_GENERATION.load(Ordering::Relaxed)
}
/// A cached Member<const TransformPaintPropertyNode>: the node owning this
/// cache, or another node (a transform- or scroll-tree ancestor) it keeps
/// alive, so a cache entry can never outlive the node it names.
#[derive(Clone)]
enum CachedNode {
    Owner,
    Node(Arc<TransformPaintPropertyNode>),
}
impl CachedNode {
    fn of(node: Arc<TransformPaintPropertyNode>, owner: &Arc<TransformPaintPropertyNode>) -> Self {
        if Arc::ptr_eq(&node, owner) {
            Self::Owner
        } else {
            Self::Node(node)
        }
    }
    fn resolve(&self, owner: &Arc<TransformPaintPropertyNode>) -> Arc<TransformPaintPropertyNode> {
        match self {
            Self::Owner => owner.clone(),
            Self::Node(node) => node.clone(),
        }
    }
    /// The parent cache's member, as held by a child's cache.
    fn inherited(&self, parent: &Arc<TransformPaintPropertyNode>) -> Self {
        Self::Node(self.resolve(parent))
    }
}
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.h:173-237.
struct PlaneRootTransform {
    to_plane_root: SharedGeometryTransform,
    from_plane_root: SharedGeometryTransform,
    plane_root: CachedNode,
    has_animation: bool,
}
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.h:239-245.
struct ScreenTransform<M> {
    to_screen: M,
    projection_from_screen: M,
    projection_from_screen_is_valid: bool,
    has_animation: bool,
}
impl<M: Default> Default for ScreenTransform<M> {
    fn default() -> Self {
        Self {
            to_screen: M::default(),
            projection_from_screen: M::default(),
            projection_from_screen_is_valid: false,
            has_animation: false,
        }
    }
}
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.h:25-32,161-172,247-265.
struct NativeGeometryMapperTransformCache {
    to_2d_translation_root_: Vector2dF,
    root_of_2d_translation_: Option<CachedNode>,
    plane_root_transform_: Option<Box<PlaneRootTransform>>,
    screen_transform_: Option<ScreenTransform<SharedGeometryTransform>>,
    nearest_scroll_translation_: Option<CachedNode>,
    scroll_translation_state_: Option<CachedNode>,
    nearest_directly_composited_ancestor_: Option<CachedNode>,
    has_sticky_or_anchor_position_: bool,
    is_backface_hidden_: bool,
    screen_transform_updated_: bool,
    cache_generation_: u32,
    /// The published property trees this cache was computed in.
    tree_scope_: CacheTreeScope,
}
/// Which published property trees a geometry cache was computed in. Blink has
/// one live property tree, so the generation alone proves validity there.
/// Here an unchanged node is shared by consecutive published trees (a scroll
/// publishes a new one while the compositor still maps the previous one), and
/// one shared cache slot must not hand values computed against one tree's
/// ancestors to a query in the other. The Weak only names the tree; it pins
/// the allocation, so the address cannot be reused while it is compared.
#[derive(Clone, Debug, Default)]
pub(crate) struct CacheTreeScope(Option<std::sync::Weak<FrozenPropertyTrees>>);
impl CacheTreeScope {
    pub(crate) fn of(trees: &NativeTrees) -> Self {
        Self(trees.as_ref().map(Arc::downgrade))
    }
    pub(crate) fn is(&self, trees: &NativeTrees) -> bool {
        match (&self.0, trees) {
            (None, None) => true,
            (Some(scope), Some(trees)) => std::ptr::eq(scope.as_ptr(), Arc::as_ptr(trees)),
            _ => false,
        }
    }
}
impl Default for NativeGeometryMapperTransformCache {
    fn default() -> Self {
        Self {
            to_2d_translation_root_: Vector2dF::default(),
            root_of_2d_translation_: None,
            plane_root_transform_: None,
            screen_transform_: None,
            nearest_scroll_translation_: None,
            scroll_translation_state_: None,
            nearest_directly_composited_ancestor_: None,
            has_sticky_or_anchor_position_: false,
            is_backface_hidden_: false,
            screen_transform_updated_: false,
            cache_generation_: global_generation().wrapping_sub(1),
            tree_scope_: CacheTreeScope::default(),
        }
    }
}
/// Invalidate every transform cache (GeometryMapperTransformCache::ClearCache).
pub fn ClearTransformCaches() {
    NativeGeometryMapperTransformCache::ClearCache();
}
impl NativeGeometryMapperTransformCache {
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.cc:18-24.
    pub fn ClearCache() {
        S_GLOBAL_GENERATION.fetch_add(1, Ordering::Relaxed);
    }
    pub fn IsValid(&self) -> bool {
        self.cache_generation_ == global_generation()
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.h:44-58.
    pub fn UpdateIfNeeded<N: TransformCacheNode>(&mut self, node: &N) {
        if self.cache_generation_ != global_generation() || !self.tree_scope_.is(node.Trees()) {
            self.Update(node);
        }
        debug_assert_eq!(self.cache_generation_, global_generation());
    }
    pub fn to_2d_translation_root(&self) -> &Vector2dF {
        &self.to_2d_translation_root_
    }
    fn root_of_2d_translation(
        &self,
        owner: &Arc<TransformPaintPropertyNode>,
    ) -> Arc<TransformPaintPropertyNode> {
        self.root_of_2d_translation_
            .as_ref()
            .expect("source initialized root")
            .resolve(owner)
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.h:62-101.
    pub fn to_screen(&self) -> &SharedGeometryTransform {
        debug_assert!(self.screen_transform_updated_);
        &self
            .screen_transform_
            .as_ref()
            .expect("source screen_transform_->to_screen")
            .to_screen
    }
    pub fn projection_from_screen(&self) -> &SharedGeometryTransform {
        debug_assert!(self.screen_transform_updated_);
        &self
            .screen_transform_
            .as_ref()
            .expect("source screen_transform_->projection_from_screen")
            .projection_from_screen
    }
    pub fn projection_from_screen_is_valid(&self) -> bool {
        debug_assert!(self.screen_transform_updated_);
        let Some(screen) = &self.screen_transform_ else {
            return true;
        };
        screen.projection_from_screen_is_valid
    }
    pub fn ApplyToScreen(&self, m: &mut SharedGeometryTransform) {
        debug_assert!(self.screen_transform_updated_);
        if self.screen_transform_.is_some() {
            m.PreConcat(self.to_screen());
        } else {
            self.ApplyToPlaneRoot(m);
        }
    }
    pub fn ApplyProjectionFromScreen(&self, m: &mut SharedGeometryTransform) {
        debug_assert!(self.screen_transform_updated_);
        if self.screen_transform_.is_some() {
            m.PreConcat(self.projection_from_screen());
        } else {
            self.ApplyFromPlaneRoot(m);
        }
    }
    pub fn has_animation_to_screen(&self) -> bool {
        debug_assert!(self.screen_transform_updated_);
        if let Some(screen) = &self.screen_transform_ {
            return screen.has_animation;
        }
        self.has_animation_to_plane_root()
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.h:103-136.
    pub fn to_plane_root(&self) -> &SharedGeometryTransform {
        &self
            .plane_root_transform_
            .as_ref()
            .expect("source DCHECK(plane_root_transform_)")
            .to_plane_root
    }
    pub fn from_plane_root(&self) -> &SharedGeometryTransform {
        &self
            .plane_root_transform_
            .as_ref()
            .expect("source DCHECK(plane_root_transform_)")
            .from_plane_root
    }
    pub fn ApplyToPlaneRoot(&self, m: &mut SharedGeometryTransform) {
        if self.plane_root_transform_.is_some() {
            m.PreConcat(self.to_plane_root());
        } else {
            m.Translate(
                self.to_2d_translation_root_.x(),
                self.to_2d_translation_root_.y(),
            );
        }
    }
    pub fn ApplyFromPlaneRoot(&self, m: &mut SharedGeometryTransform) {
        if self.plane_root_transform_.is_some() {
            m.PreConcat(self.from_plane_root());
        } else {
            m.Translate(
                -self.to_2d_translation_root_.x(),
                -self.to_2d_translation_root_.y(),
            );
        }
    }
    fn plane_root(&self, owner: &Arc<TransformPaintPropertyNode>) -> Arc<TransformPaintPropertyNode> {
        match &self.plane_root_transform_ {
            Some(plane) => plane.plane_root.resolve(owner),
            None => self.root_of_2d_translation(owner),
        }
    }
    pub fn has_animation_to_plane_root(&self) -> bool {
        if let Some(plane) = &self.plane_root_transform_ {
            return plane.has_animation;
        }
        false
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.h:138-159.
    pub fn has_sticky_or_anchor_position(&self) -> bool {
        self.has_sticky_or_anchor_position_
    }
    pub fn is_backface_hidden(&self) -> bool {
        self.is_backface_hidden_
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.cc:26-133.
    fn Update<N: TransformCacheNode>(&mut self, node: &N) {
        self.cache_generation_ = global_generation();
        self.tree_scope_ = CacheTreeScope::of(node.Trees());
        if node.IsRoot() {
            debug_assert!(node.IsIdentity());
            self.to_2d_translation_root_ = Vector2dF::default();
            self.root_of_2d_translation_ = Some(CachedNode::Owner);
            self.plane_root_transform_ = None;
            self.screen_transform_ = None;
            self.screen_transform_updated_ = true;
            debug_assert!(node.HasScrollNode());
            self.nearest_scroll_translation_ = Some(CachedNode::Owner);
            self.scroll_translation_state_ = Some(CachedNode::Owner);
            return;
        }
        let owner = node.NodeArc();
        let parent_node = node.Parent().expect("source node.UnaliasedParent()");
        let parent_arc = parent_node.NodeArc();
        let inherited = |member: &Option<CachedNode>| {
            member.as_ref().map(|member| member.inherited(&parent_arc))
        };
        WithTransformCache(&parent_node, |parent| {
            self.has_sticky_or_anchor_position_ = node.RequiresCompositingForStickyPosition()
                || node.RequiresCompositingForAnchorPosition()
                || parent.data.has_sticky_or_anchor_position_;
            self.is_backface_hidden_ =
                node.IsBackfaceHiddenInternal(parent.data.is_backface_hidden_);
            self.nearest_scroll_translation_ = if node.HasScrollNode() {
                Some(CachedNode::Owner)
            } else {
                inherited(&parent.data.nearest_scroll_translation_)
            };
            if let Some(scroll_parent) = node.ScrollParentScrollTranslation() {
                self.scroll_translation_state_ =
                    Some(CachedNode::of(scroll_parent.NodeArc(), &owner));
            } else if node.HasScrollNode() {
                self.scroll_translation_state_ = Some(CachedNode::Owner);
            } else {
                self.scroll_translation_state_ = inherited(&parent.data.scroll_translation_state_);
            }
            self.nearest_directly_composited_ancestor_ = if node.HasDirectCompositingReasons() {
                Some(CachedNode::Owner)
            } else {
                inherited(&parent.data.nearest_directly_composited_ancestor_)
            };
            if node.IsIdentityOr2dTranslation() && !node.HasActiveTransformAnimation() {
                self.root_of_2d_translation_ = inherited(&parent.data.root_of_2d_translation_);
                self.to_2d_translation_root_ = parent.data.to_2d_translation_root_;
                let translation = node.Get2dTranslation();
                self.to_2d_translation_root_ += translation;
                if parent.data.plane_root_transform_.is_some() {
                    self.plane_root_transform_ = Some(Box::new(PlaneRootTransform {
                        plane_root: CachedNode::of(parent.data.plane_root(&parent_arc), &owner),
                        to_plane_root: parent.data.to_plane_root().clone(),
                        from_plane_root: SharedGeometryTransform::default(),
                        has_animation: false,
                    }));
                    let plane = self.plane_root_transform_.as_mut().unwrap();
                    plane
                        .to_plane_root
                        .Translate(translation.x(), translation.y());
                    plane.from_plane_root = parent.data.from_plane_root().clone();
                    plane
                        .from_plane_root
                        .PostTranslate(Vector2dF::new(-translation.x(), -translation.y()));
                    plane.has_animation = parent.data.has_animation_to_plane_root();
                } else {
                    self.plane_root_transform_ = None;
                }
            } else {
                self.root_of_2d_translation_ = Some(CachedNode::Owner);
                self.to_2d_translation_root_ = Vector2dF::default();
                let local = node.MatrixWithOriginApplied().ToGeometryMapperMatrix();
                let is_plane_root = !local.IsFlat() || !local.IsInvertible();
                if is_plane_root {
                    self.plane_root_transform_ = None;
                } else {
                    self.plane_root_transform_ = Some(Box::new(PlaneRootTransform {
                        plane_root: CachedNode::of(parent.data.plane_root(&parent_arc), &owner),
                        to_plane_root: SharedGeometryTransform::default(),
                        from_plane_root: SharedGeometryTransform::default(),
                        has_animation: false,
                    }));
                    let plane = self.plane_root_transform_.as_mut().unwrap();
                    plane.to_plane_root = SharedGeometryTransform::default();
                    parent.data.ApplyToPlaneRoot(&mut plane.to_plane_root);
                    plane.to_plane_root.PreConcat(&local);
                    plane.from_plane_root = local.GetCheckedInverse();
                    parent.data.ApplyFromPlaneRoot(&mut plane.from_plane_root);
                    plane.has_animation = parent.data.has_animation_to_plane_root()
                        || node.HasActiveTransformAnimation();
                }
            }
        });
        if self.plane_root(&owner).parent.is_none() {
            self.screen_transform_ = None;
            self.screen_transform_updated_ = true;
        } else {
            self.screen_transform_updated_ = false;
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.cc:135-165.
    pub fn UpdateScreenTransform<N: TransformCacheNode>(&mut self, node: &N) {
        debug_assert_eq!(self.cache_generation_, global_generation());
        if self.screen_transform_updated_ {
            return;
        }
        self.screen_transform_updated_ = true;
        debug_assert!(!self.plane_root(&node.NodeArc()).parent.is_none());
        debug_assert!(!node.IsRoot());
        let parent_node = node.Parent().expect("source parent_node");
        WithTransformCache(&parent_node, |parent| {
            parent.data.UpdateScreenTransform(&parent_node);
        });
        WithTransformCache(&parent_node, |parent| {
            self.screen_transform_ = Some(ScreenTransform::<SharedGeometryTransform>::default());
            let screen = self.screen_transform_.as_mut().unwrap();
            parent.data.ApplyToScreen(&mut screen.to_screen);
            if node.FlattensInheritedTransform() {
                screen.to_screen.Flatten();
            }
            if node.IsIdentityOr2dTranslation() {
                let translation = node.Get2dTranslation();
                screen.to_screen.Translate(translation.x(), translation.y());
            } else {
                screen
                    .to_screen
                    .PreConcat(&node.MatrixWithOriginApplied().ToGeometryMapperMatrix());
            }
            let mut flattened = screen.to_screen.clone();
            flattened.Flatten();
            screen.projection_from_screen_is_valid =
                flattened.GetInverse(&mut screen.projection_from_screen);
            screen.has_animation |= node.HasActiveTransformAnimation();
        });
    }
}
/// Source: third_party/blink/renderer/platform/graphics/paint/transform_paint_property_node.h:472-485. One allocation in the real
/// node, recomputed by the source generation check. A call-scoped lock lends
/// that same cache; it never clones a mutable cache or retains an owner/facade.
pub fn WithTransformCache<N: TransformCacheNode, R>(
    node: &N,
    callback: impl FnOnce(&mut GeometryMapperTransformCache<'_, N>) -> R,
) -> R {
    let allocation = node
        .CacheSlot()
        .get_or_init(|| Mutex::new(NativeGeometryMapperTransformCache::default()));
    let mut data = allocation.lock().expect("Paint transform cache poisoned");
    data.UpdateIfNeeded(node);
    callback(&mut GeometryMapperTransformCache {
        data: &mut data,
        node,
    })
}

/// A typed value projection of the one native cache allocation. No backend
/// wrapper, engine owner or second mutable cache is retained by a Paint node.
pub struct GeometryMapperTransformCache<'a, N: TransformCacheNode> {
    data: &'a mut NativeGeometryMapperTransformCache,
    /// The node whose cache this is; cached ancestors link through it.
    node: &'a N,
}
impl<N: TransformCacheNode> GeometryMapperTransformCache<'_, N> {
    fn link(&self, member: &CachedNode) -> N {
        match member {
            CachedNode::Owner => self.node.clone(),
            CachedNode::Node(node) => self.node.Linked(node.clone()),
        }
    }
    pub fn ClearCache() {
        NativeGeometryMapperTransformCache::ClearCache();
    }
    pub fn IsValid(&self) -> bool {
        self.data.IsValid() && self.data.tree_scope_.is(self.node.Trees())
    }
    pub fn UpdateIfNeeded(&mut self, node: &N) {
        self.data.UpdateIfNeeded(node);
    }
    pub fn UpdateScreenTransform(&mut self, node: &N) {
        self.data.UpdateScreenTransform(node);
    }
    pub fn to_2d_translation_root(&self) -> &Vector2dF {
        self.data.to_2d_translation_root()
    }
    pub fn root_of_2d_translation(&self) -> N {
        self.link(
            self.data
                .root_of_2d_translation_
                .as_ref()
                .expect("source initialized root"),
        )
    }
    pub fn plane_root(&self) -> N {
        match &self.data.plane_root_transform_ {
            Some(plane) => self.link(&plane.plane_root),
            None => self.root_of_2d_translation(),
        }
    }
    pub fn nearest_scroll_translation(&self) -> N {
        self.link(
            self.data
                .nearest_scroll_translation_
                .as_ref()
                .expect("source initialized scroll"),
        )
    }
    pub fn scroll_translation_state(&self) -> N {
        self.link(
            self.data
                .scroll_translation_state_
                .as_ref()
                .expect("source initialized scroll state"),
        )
    }
    pub fn nearest_directly_composited_ancestor(&self) -> Option<N> {
        self.data
            .nearest_directly_composited_ancestor_
            .as_ref()
            .map(|member| self.link(member))
    }
    pub fn projection_from_screen_is_valid(&self) -> bool {
        self.data.projection_from_screen_is_valid()
    }
    pub fn has_animation_to_screen(&self) -> bool {
        self.data.has_animation_to_screen()
    }
    pub fn has_animation_to_plane_root(&self) -> bool {
        self.data.has_animation_to_plane_root()
    }
    pub fn has_sticky_or_anchor_position(&self) -> bool {
        self.data.has_sticky_or_anchor_position()
    }
    pub fn is_backface_hidden(&self) -> bool {
        self.data.is_backface_hidden()
    }
    pub fn to_screen(&self) -> N::Matrix {
        N::Matrix::FromGeometryMapperMatrix(self.data.to_screen().clone())
    }
    pub fn projection_from_screen(&self) -> N::Matrix {
        N::Matrix::FromGeometryMapperMatrix(self.data.projection_from_screen().clone())
    }
    pub fn to_plane_root(&self) -> N::Matrix {
        N::Matrix::FromGeometryMapperMatrix(self.data.to_plane_root().clone())
    }
    pub fn from_plane_root(&self) -> N::Matrix {
        N::Matrix::FromGeometryMapperMatrix(self.data.from_plane_root().clone())
    }
    pub fn ApplyToScreen(&self, m: &mut N::Matrix) {
        let mut value = m.ToGeometryMapperMatrix();
        self.data.ApplyToScreen(&mut value);
        *m = N::Matrix::FromGeometryMapperMatrix(value);
    }
    pub fn ApplyProjectionFromScreen(&self, m: &mut N::Matrix) {
        let mut value = m.ToGeometryMapperMatrix();
        self.data.ApplyProjectionFromScreen(&mut value);
        *m = N::Matrix::FromGeometryMapperMatrix(value);
    }
    pub fn ApplyToPlaneRoot(&self, m: &mut N::Matrix) {
        let mut value = m.ToGeometryMapperMatrix();
        self.data.ApplyToPlaneRoot(&mut value);
        *m = N::Matrix::FromGeometryMapperMatrix(value);
    }
    pub fn ApplyFromPlaneRoot(&self, m: &mut N::Matrix) {
        let mut value = m.ToGeometryMapperMatrix();
        self.data.ApplyFromPlaneRoot(&mut value);
        *m = N::Matrix::FromGeometryMapperMatrix(value);
    }
}
