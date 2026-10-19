// Translation status: complete
// Copyright 2017 The Chromium Authors. BSD-style license in LICENSE.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.h:1-39;
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.cc:1-16.
// Includes/namespace/GC tracing map to native modules and non-owning weak node
// handles. The actual Paint node owns the synchronized cache allocation.
use crate::adapter::{
    geometry::{Transform, Vector2dF},
    paint_geometry::TransformCacheNode,
    property_tree::PropertyTreeTransform,
    property_types::PropertyTransform,
};
use std::{cell::Cell, sync::Mutex};
thread_local! {static S_GLOBAL_GENERATION:Cell<u32>=const{Cell::new(1)};}
fn global_generation() -> u32 {
    S_GLOBAL_GENERATION.with(Cell::get)
}
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.h:173-237.
struct PlaneRootTransform<N: TransformCacheNode> {
    to_plane_root: N::Matrix,
    from_plane_root: N::Matrix,
    plane_root: N::Weak,
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
pub struct GeometryMapperTransformCache<N: TransformCacheNode> {
    to_2d_translation_root_: Vector2dF,
    root_of_2d_translation_: Option<N::Weak>,
    plane_root_transform_: Option<Box<PlaneRootTransform<N>>>,
    screen_transform_: Option<ScreenTransform<N::Matrix>>,
    nearest_scroll_translation_: Option<N::Weak>,
    scroll_translation_state_: Option<N::Weak>,
    nearest_directly_composited_ancestor_: Option<N::Weak>,
    has_sticky_or_anchor_position_: bool,
    is_backface_hidden_: bool,
    screen_transform_updated_: bool,
    cache_generation_: u32,
}
impl<N: TransformCacheNode> Default for GeometryMapperTransformCache<N> {
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
        }
    }
}
impl<N: TransformCacheNode> GeometryMapperTransformCache<N> {
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.cc:18-24.
    pub fn ClearCache() {
        S_GLOBAL_GENERATION.with(|generation| generation.set(generation.get().wrapping_add(1)));
    }
    pub fn IsValid(&self) -> bool {
        self.cache_generation_ == global_generation()
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.h:44-58.
    pub fn UpdateIfNeeded(&mut self, node: &N) {
        if self.cache_generation_ != global_generation() {
            self.Update(node);
        }
        debug_assert_eq!(self.cache_generation_, global_generation());
    }
    pub fn to_2d_translation_root(&self) -> &Vector2dF {
        &self.to_2d_translation_root_
    }
    pub fn root_of_2d_translation(&self) -> N {
        N::Upgrade(
            self.root_of_2d_translation_
                .as_ref()
                .expect("source initialized root"),
        )
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.h:62-101.
    pub fn to_screen(&self) -> &N::Matrix {
        debug_assert!(self.screen_transform_updated_);
        &self
            .screen_transform_
            .as_ref()
            .expect("source screen_transform_->to_screen")
            .to_screen
    }
    pub fn projection_from_screen(&self) -> &N::Matrix {
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
    pub fn ApplyToScreen(&self, m: &mut N::Matrix) {
        debug_assert!(self.screen_transform_updated_);
        if self.screen_transform_.is_some() {
            m.PreConcat(self.to_screen());
        } else {
            self.ApplyToPlaneRoot(m);
        }
    }
    pub fn ApplyProjectionFromScreen(&self, m: &mut N::Matrix) {
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
    pub fn to_plane_root(&self) -> &N::Matrix {
        &self
            .plane_root_transform_
            .as_ref()
            .expect("source DCHECK(plane_root_transform_)")
            .to_plane_root
    }
    pub fn from_plane_root(&self) -> &N::Matrix {
        &self
            .plane_root_transform_
            .as_ref()
            .expect("source DCHECK(plane_root_transform_)")
            .from_plane_root
    }
    pub fn ApplyToPlaneRoot(&self, m: &mut N::Matrix) {
        if self.plane_root_transform_.is_some() {
            m.PreConcat(self.to_plane_root());
        } else {
            m.Translate(
                self.to_2d_translation_root_.x(),
                self.to_2d_translation_root_.y(),
            );
        }
    }
    pub fn ApplyFromPlaneRoot(&self, m: &mut N::Matrix) {
        if self.plane_root_transform_.is_some() {
            m.PreConcat(self.from_plane_root());
        } else {
            m.Translate(
                -self.to_2d_translation_root_.x(),
                -self.to_2d_translation_root_.y(),
            );
        }
    }
    pub fn plane_root(&self) -> N {
        if let Some(plane) = &self.plane_root_transform_ {
            return N::Upgrade(&plane.plane_root);
        }
        self.root_of_2d_translation()
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
    pub fn nearest_scroll_translation(&self) -> N {
        N::Upgrade(
            self.nearest_scroll_translation_
                .as_ref()
                .expect("source DCHECK(nearest_scroll_translation_)"),
        )
    }
    pub fn scroll_translation_state(&self) -> N {
        N::Upgrade(
            self.scroll_translation_state_
                .as_ref()
                .expect("source DCHECK(scroll_translation_state_)"),
        )
    }
    pub fn nearest_directly_composited_ancestor(&self) -> Option<N> {
        self.nearest_directly_composited_ancestor_
            .as_ref()
            .map(N::Upgrade)
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.cc:26-133.
    fn Update(&mut self, node: &N) {
        debug_assert_ne!(self.cache_generation_, global_generation());
        self.cache_generation_ = global_generation();
        if node.IsRoot() {
            debug_assert!(node.IsIdentity());
            self.to_2d_translation_root_ = Vector2dF::default();
            self.root_of_2d_translation_ = Some(node.Downgrade());
            self.plane_root_transform_ = None;
            self.screen_transform_ = None;
            self.screen_transform_updated_ = true;
            debug_assert!(node.HasScrollNode());
            self.nearest_scroll_translation_ = Some(node.Downgrade());
            self.scroll_translation_state_ = Some(node.Downgrade());
            return;
        }
        let parent_node = node.Parent().expect("source node.UnaliasedParent()");
        WithTransformCache(&parent_node, |parent| {
            self.has_sticky_or_anchor_position_ = node.RequiresCompositingForStickyPosition()
                || node.RequiresCompositingForAnchorPosition()
                || parent.has_sticky_or_anchor_position_;
            self.is_backface_hidden_ = node.IsBackfaceHiddenInternal(parent.is_backface_hidden_);
            self.nearest_scroll_translation_ = if node.HasScrollNode() {
                Some(node.Downgrade())
            } else {
                parent.nearest_scroll_translation_.clone()
            };
            if let Some(scroll_parent) = node.ScrollParentScrollTranslation() {
                self.scroll_translation_state_ = Some(scroll_parent.Downgrade());
            } else if node.HasScrollNode() {
                self.scroll_translation_state_ = Some(node.Downgrade());
            } else {
                self.scroll_translation_state_ = parent.scroll_translation_state_.clone();
            }
            self.nearest_directly_composited_ancestor_ = if node.HasDirectCompositingReasons() {
                Some(node.Downgrade())
            } else {
                parent.nearest_directly_composited_ancestor_.clone()
            };
            if node.IsIdentityOr2dTranslation() && !node.HasActiveTransformAnimation() {
                self.root_of_2d_translation_ = parent.root_of_2d_translation_.clone();
                self.to_2d_translation_root_ = parent.to_2d_translation_root_;
                let translation = node.Get2dTranslation();
                self.to_2d_translation_root_ += translation;
                if parent.plane_root_transform_.is_some() {
                    self.plane_root_transform_ = Some(Box::new(PlaneRootTransform {
                        plane_root: parent.plane_root().Downgrade(),
                        to_plane_root: parent.to_plane_root().clone(),
                        from_plane_root: N::Matrix::default(),
                        has_animation: false,
                    }));
                    let plane = self.plane_root_transform_.as_mut().unwrap();
                    plane
                        .to_plane_root
                        .Translate(translation.x(), translation.y());
                    plane.from_plane_root = parent.from_plane_root().clone();
                    plane
                        .from_plane_root
                        .PostTranslate(Vector2dF::new(-translation.x(), -translation.y()));
                    plane.has_animation = parent.has_animation_to_plane_root();
                } else {
                    self.plane_root_transform_ = None;
                }
            } else {
                self.root_of_2d_translation_ = Some(node.Downgrade());
                self.to_2d_translation_root_ = Vector2dF::default();
                let local = node.MatrixWithOriginApplied();
                let is_plane_root = !local.IsFlat() || !local.IsInvertible();
                if is_plane_root {
                    self.plane_root_transform_ = None;
                } else {
                    self.plane_root_transform_ = Some(Box::new(PlaneRootTransform {
                        plane_root: parent.plane_root().Downgrade(),
                        to_plane_root: N::Matrix::default(),
                        from_plane_root: N::Matrix::default(),
                        has_animation: false,
                    }));
                    let plane = self.plane_root_transform_.as_mut().unwrap();
                    plane.to_plane_root = N::Matrix::default();
                    parent.ApplyToPlaneRoot(&mut plane.to_plane_root);
                    plane.to_plane_root.PreConcat(&local);
                    plane.from_plane_root = local.GetCheckedInverse();
                    parent.ApplyFromPlaneRoot(&mut plane.from_plane_root);
                    plane.has_animation =
                        parent.has_animation_to_plane_root() || node.HasActiveTransformAnimation();
                }
            }
        });
        if self.plane_root().IsRoot() {
            self.screen_transform_ = None;
            self.screen_transform_updated_ = true;
        } else {
            self.screen_transform_updated_ = false;
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.cc:135-165.
    pub fn UpdateScreenTransform(&mut self, node: &N) {
        debug_assert_eq!(self.cache_generation_, global_generation());
        if self.screen_transform_updated_ {
            return;
        }
        self.screen_transform_updated_ = true;
        debug_assert!(!self.plane_root().IsRoot());
        debug_assert!(!node.IsRoot());
        let parent_node = node.Parent().expect("source parent_node");
        WithTransformCache(&parent_node, |parent| {
            parent.UpdateScreenTransform(&parent_node);
        });
        WithTransformCache(&parent_node, |parent| {
            self.screen_transform_ = Some(ScreenTransform::<N::Matrix>::default());
            let screen = self.screen_transform_.as_mut().unwrap();
            parent.ApplyToScreen(&mut screen.to_screen);
            if node.FlattensInheritedTransform() {
                screen.to_screen.Flatten();
            }
            if node.IsIdentityOr2dTranslation() {
                let translation = node.Get2dTranslation();
                screen.to_screen.Translate(translation.x(), translation.y());
            } else {
                screen.to_screen.PreConcat(&node.MatrixWithOriginApplied());
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
    callback: impl FnOnce(&mut GeometryMapperTransformCache<N>) -> R,
) -> R {
    let allocation = node
        .CacheSlot()
        .get_or_init(|| Mutex::new(GeometryMapperTransformCache::<N>::default()));
    let mut cache = allocation.lock().expect("Paint transform cache poisoned");
    cache.UpdateIfNeeded(node);
    callback(&mut cache)
}
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.cc:167.
// Namespace closure is the Rust module boundary.
