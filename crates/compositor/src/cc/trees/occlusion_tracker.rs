// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/occlusion_tracker.h:1-119
// Source: cc/trees/occlusion_tracker.cc:1-407
use super::occlusion::Occlusion;
use crate::{
    adapter::{
        geometry::{Rect, Size},
        occlusion::{
            EffectTreeLayerListIteratorPosition, EffectTreeLayerListIteratorState,
            OcclusionContext, OcclusionEffectNode, OcclusionLayerTree, OcclusionTransform,
            SkBlendMode, SkMatrix,
        },
        property_tree::PropertyTreeTransform as _,
        region_backend::SkRegionBackend,
    },
    cc::base::{
        math_util::MathUtil, region::Region, simple_enclosed_region::SimpleEnclosedRegion,
    },
};

// Source: cc/trees/occlusion_tracker.h:62-76
pub struct StackObject<S: Copy + Eq> {
    target: Option<S>,
    occlusion_from_outside_target: SimpleEnclosedRegion,
    occlusion_from_inside_target: SimpleEnclosedRegion,
    ignores_parent_occlusion: bool,
}
impl<S: Copy + Eq> Default for StackObject<S> {
    // Source: cc/trees/occlusion_tracker.h:63
    fn default() -> Self {
        Self {
            target: None,
            occlusion_from_outside_target: SimpleEnclosedRegion::default(),
            occlusion_from_inside_target: SimpleEnclosedRegion::default(),
            ignores_parent_occlusion: false,
        }
    }
}
impl<S: Copy + Eq> StackObject<S> {
    // Source: cc/trees/occlusion_tracker.h:64
    fn new(target: S) -> Self {
        Self {
            target: Some(target),
            ..Self::default()
        }
    }
}

// Source: cc/trees/occlusion_tracker.h:31-119
pub struct OcclusionTracker<T: OcclusionTransform, S: Copy + Eq, L: Copy + Eq> {
    // Source: cc/trees/occlusion_tracker.h:78-93
    stack_: Vec<StackObject<S>>,
    // Source: cc/trees/occlusion_tracker.h:113-114
    screen_space_clip_rect_: Rect,
    minimum_tracking_size_: Size,
    // Rust type routing only; the context is borrowed at each traversal call.
    context_: std::marker::PhantomData<fn() -> (T, L)>,
}
impl<T: OcclusionTransform, S: Copy + Eq, L: Copy + Eq> OcclusionTracker<T, S, L> {
    // Source: cc/trees/occlusion_tracker.cc:22-25
    pub fn new(screen_space_clip_rect: Rect) -> Self {
        Self {
            stack_: Vec::new(),
            screen_space_clip_rect_: screen_space_clip_rect,
            minimum_tracking_size_: Size::default(),
            context_: std::marker::PhantomData,
        }
    }
    // Source: cc/trees/occlusion_tracker.cc:27-33
    pub fn GetCurrentOcclusionForLayer(&self, draw_transform: &T) -> Occlusion<T> {
        debug_assert!(!self.stack_.is_empty());
        let back = self.stack_.last().unwrap();
        Occlusion::new(
            draw_transform,
            back.occlusion_from_outside_target,
            back.occlusion_from_inside_target,
        )
    }
    // Source: cc/trees/occlusion_tracker.cc:35-46
    pub fn GetCurrentOcclusionForContributingSurface(&self, draw_transform: &T) -> Occlusion<T> {
        debug_assert!(!self.stack_.is_empty());
        if self.stack_.len() < 2 || self.stack_.last().unwrap().ignores_parent_occlusion {
            return Occlusion::default();
        }
        let second_last = &self.stack_[self.stack_.len() - 2];
        Occlusion::new(
            draw_transform,
            second_last.occlusion_from_outside_target,
            second_last.occlusion_from_inside_target,
        )
    }
    // Source: cc/trees/occlusion_tracker.cc:48-56
    pub fn OcclusionSurfaceForContributingSurface(&self) -> Option<S> {
        if self.stack_.len() < 2 || self.stack_.last().unwrap().ignores_parent_occlusion {
            return None;
        }
        self.stack_[self.stack_.len() - 2].target
    }
    // Source: cc/trees/occlusion_tracker.cc:58-68
    pub fn EnterLayer<C: OcclusionContext<Transform = T, Surface = S, Layer = L>>(
        &mut self,
        iterator: &EffectTreeLayerListIteratorPosition<S, L>,
        context: &C,
    ) {
        let render_target = iterator.target_render_surface;
        if iterator.state == EffectTreeLayerListIteratorState::kLayer {
            self.EnterRenderTarget(render_target, context);
        } else if iterator.state == EffectTreeLayerListIteratorState::kTargetSurface {
            self.FinishedRenderTarget(render_target, context);
        }
    }
    // Source: cc/trees/occlusion_tracker.cc:70-82
    pub fn LeaveLayer<C: OcclusionContext<Transform = T, Surface = S, Layer = L>>(
        &mut self,
        iterator: &EffectTreeLayerListIteratorPosition<S, L>,
        context: &C,
    ) {
        let render_target = iterator.target_render_surface;
        if iterator.state == EffectTreeLayerListIteratorState::kLayer {
            self.MarkOccludedBehindLayer(
                iterator
                    .current_layer
                    .expect("layer position without a layer"),
                context,
            );
        } else if iterator.state == EffectTreeLayerListIteratorState::kContributingSurface {
            self.LeaveToRenderTarget(render_target, context);
        }
    }
    // Source: cc/trees/occlusion_tracker.h:58-60
    pub fn set_minimum_tracking_size(&mut self, size: Size) {
        self.minimum_tracking_size_ = size;
    }
}

// Source: cc/trees/occlusion_tracker.cc:84-94
fn ScreenSpaceClipRectInTargetSurface<C: OcclusionContext>(
    target_surface: C::Surface,
    screen_space_clip_rect: Rect,
    context: &C,
) -> Rect {
    let mut inverse_screen_space_transform = C::Transform::default();
    if !context
        .surface(target_surface)
        .screen_space_transform()
        .GetInverse(&mut inverse_screen_space_transform)
    {
        return context.surface(target_surface).content_rect();
    }
    MathUtil::ProjectEnclosingClippedRect(&inverse_screen_space_transform, screen_space_clip_rect)
}

// Source: cc/trees/occlusion_tracker.cc:96-122
fn TransformSurfaceOpaqueRegion<T: OcclusionTransform>(
    region: SimpleEnclosedRegion,
    have_clip_rect: bool,
    clip_rect_in_new_target: Rect,
    transform: &T,
) -> SimpleEnclosedRegion {
    if region.IsEmpty() {
        return region;
    }
    if !transform.NonDegeneratePreserves2dAxisAlignment() {
        return SimpleEnclosedRegion::default();
    }
    let mut transformed_region = SimpleEnclosedRegion::default();
    for i in 0..region.GetRegionComplexity() {
        let mut transformed_rect =
            MathUtil::MapEnclosedRectWith2dAxisAlignedTransform(transform, region.GetRect(i));
        if have_clip_rect {
            transformed_rect.Intersect(clip_rect_in_new_target);
        }
        transformed_region.Union(transformed_rect);
    }
    transformed_region
}

impl<T: OcclusionTransform, S: Copy + Eq, L: Copy + Eq> OcclusionTracker<T, S, L> {
    // Source: cc/trees/occlusion_tracker.cc:124-180
    fn EnterRenderTarget<C: OcclusionContext<Transform = T, Surface = S, Layer = L>>(
        &mut self,
        new_target_surface: S,
        context: &C,
    ) {
        if !self.stack_.is_empty() && self.stack_.last().unwrap().target == Some(new_target_surface)
        {
            return;
        }
        let mut old_target_surface = None;
        let mut old_occlusion_immune_ancestor = None;
        if !self.stack_.is_empty() {
            old_target_surface = self.stack_.last().unwrap().target;
            old_occlusion_immune_ancestor = context
                .surface(old_target_surface.unwrap())
                .nearest_occlusion_immune_ancestor();
        }
        let new_occlusion_immune_ancestor = context
            .surface(new_target_surface)
            .nearest_occlusion_immune_ancestor();
        self.stack_.push(StackObject::new(new_target_surface));
        let entering_unoccluded_subtree = new_occlusion_immune_ancestor.is_some()
            && new_occlusion_immune_ancestor != old_occlusion_immune_ancestor;
        let mut inverse_new_target_screen_space_transform = T::default();
        let have_transform_from_screen_to_new_target = context
            .surface(new_target_surface)
            .screen_space_transform()
            .GetInverse(&mut inverse_new_target_screen_space_transform);
        let entering_root_target = context
            .surface(new_target_surface)
            .render_target(context.property_trees())
            == new_target_surface;
        let copy_outside_occlusion_forward = self.stack_.len() > 1
            && !entering_unoccluded_subtree
            && have_transform_from_screen_to_new_target
            && !entering_root_target;
        if !copy_outside_occlusion_forward {
            self.stack_.last_mut().unwrap().ignores_parent_occlusion = true;
            return;
        }
        let last_index = self.stack_.len() - 1;
        let old_target_to_new_target_transform = inverse_new_target_screen_space_transform
            .Multiply(
                context
                    .surface(old_target_surface.unwrap())
                    .screen_space_transform(),
            );
        self.stack_[last_index].occlusion_from_outside_target = TransformSurfaceOpaqueRegion(
            self.stack_[last_index - 1].occlusion_from_outside_target,
            false,
            Rect::default(),
            &old_target_to_new_target_transform,
        );
        let old_inside = TransformSurfaceOpaqueRegion(
            self.stack_[last_index - 1].occlusion_from_inside_target,
            false,
            Rect::default(),
            &old_target_to_new_target_transform,
        );
        self.stack_[last_index]
            .occlusion_from_outside_target
            .UnionRegion(&old_inside);
    }
}

// Source: cc/trees/occlusion_tracker.cc:182-186
fn IsOccludingBlendMode(blend_mode: SkBlendMode) -> bool {
    blend_mode == SkBlendMode::kSrc || blend_mode == SkBlendMode::kSrcOver
}

impl<T: OcclusionTransform, S: Copy + Eq, L: Copy + Eq> OcclusionTracker<T, S, L> {
    // Source: cc/trees/occlusion_tracker.cc:188-213
    fn FinishedRenderTarget<C: OcclusionContext<Transform = T, Surface = S, Layer = L>>(
        &mut self,
        finished_target_surface: S,
        context: &C,
    ) {
        self.EnterRenderTarget(finished_target_surface, context);
        let surface = context.surface(finished_target_surface);
        let is_hidden = surface
            .OwningEffectNode(context.property_trees())
            .screen_space_opacity()
            == 0.0;
        let target_is_only_for_copy_request_or_force_render_surface =
            is_hidden && surface.CopyOfOutputRequired(context.property_trees());
        if surface.HasMaskingContributingSurface(context.property_trees())
            || surface.draw_opacity() < 1.0
            || !IsOccludingBlendMode(surface.BlendMode(context.property_trees()))
            || target_is_only_for_copy_request_or_force_render_surface
            || surface
                .Filters(context.property_trees())
                .HasFilterThatAffectsOpacity()
            || surface
                .OwningEffectNode(context.property_trees())
                .view_transition_element_resource_id_is_valid()
        {
            self.stack_
                .last_mut()
                .unwrap()
                .occlusion_from_outside_target
                .Clear();
            self.stack_
                .last_mut()
                .unwrap()
                .occlusion_from_inside_target
                .Clear();
        }
    }
}

// Source: cc/trees/occlusion_tracker.cc:215-269
fn ReduceOcclusionBelowSurface<C: OcclusionContext>(
    contributing_surface: C::Surface,
    surface_rect: Rect,
    surface_transform: &C::Transform,
    occlusion_from_inside_target: &mut SimpleEnclosedRegion,
    context: &C,
) {
    if surface_rect.IsEmpty() {
        return;
    }
    let mut target_rect = MathUtil::MapEnclosingClippedRect(surface_transform, surface_rect);
    let surface = context.surface(contributing_surface);
    if surface.is_clipped() {
        target_rect.Intersect(surface.clip_rect());
    }
    if target_rect.IsEmpty() {
        return;
    }
    let mut affected_area_in_target = surface
        .BackdropFilters(context.property_trees())
        .MapRectReverse(target_rect, &SkMatrix::I());
    affected_area_in_target.Union(target_rect);
    let mut affected_occlusion = *occlusion_from_inside_target;
    affected_occlusion.Intersect(affected_area_in_target);
    occlusion_from_inside_target.Subtract(affected_area_in_target);
    for i in 0..affected_occlusion.GetRegionComplexity() {
        let mut occlusion_rect = affected_occlusion.GetRect(i);
        let shrink_left = if occlusion_rect.x == affected_area_in_target.x {
            0
        } else {
            affected_area_in_target.right() - target_rect.right()
        };
        let shrink_top = if occlusion_rect.y == affected_area_in_target.y {
            0
        } else {
            affected_area_in_target.bottom() - target_rect.bottom()
        };
        let shrink_right = if occlusion_rect.right() == affected_area_in_target.right() {
            0
        } else {
            target_rect.x - affected_area_in_target.x
        };
        let shrink_bottom = if occlusion_rect.bottom() == affected_area_in_target.bottom() {
            0
        } else {
            target_rect.y - affected_area_in_target.y
        };
        occlusion_rect.Inset(shrink_top, shrink_left, shrink_bottom, shrink_right);
        occlusion_from_inside_target.Union(occlusion_rect);
    }
}

impl<T: OcclusionTransform, S: Copy + Eq, L: Copy + Eq> OcclusionTracker<T, S, L> {
    // Source: cc/trees/occlusion_tracker.cc:271-338
    fn LeaveToRenderTarget<C: OcclusionContext<Transform = T, Surface = S, Layer = L>>(
        &mut self,
        new_target_surface: S,
        context: &C,
    ) {
        debug_assert!(!self.stack_.is_empty());
        let last_index = self.stack_.len() - 1;
        let surface_will_be_at_top_after_pop =
            self.stack_.len() > 1 && self.stack_[last_index - 1].target == Some(new_target_surface);
        let old_surface = self.stack_[last_index].target.unwrap();
        let old_occlusion_from_inside_target_in_new_target = TransformSurfaceOpaqueRegion(
            self.stack_[last_index].occlusion_from_inside_target,
            context.surface(old_surface).is_clipped(),
            context.surface(old_surface).clip_rect(),
            context.surface(old_surface).draw_transform(),
        );
        let old_occlusion_from_outside_target_in_new_target = TransformSurfaceOpaqueRegion(
            self.stack_[last_index].occlusion_from_outside_target,
            false,
            Rect::default(),
            context.surface(old_surface).draw_transform(),
        );
        let mut unoccluded_surface_rect = Rect::default();
        if context
            .surface(old_surface)
            .BackdropFilters(context.property_trees())
            .HasFilterThatMovesPixels()
        {
            let surface_occlusion = self.GetCurrentOcclusionForContributingSurface(
                context.surface(old_surface).draw_transform(),
            );
            unoccluded_surface_rect = surface_occlusion
                .GetUnoccludedContentRect(context.surface(old_surface).content_rect());
        }
        let is_root = context
            .surface(new_target_surface)
            .render_target(context.property_trees())
            == new_target_surface;
        if surface_will_be_at_top_after_pop {
            self.stack_[last_index - 1]
                .occlusion_from_inside_target
                .UnionRegion(&old_occlusion_from_inside_target_in_new_target);
            if !is_root {
                self.stack_[last_index - 1]
                    .occlusion_from_outside_target
                    .UnionRegion(&old_occlusion_from_outside_target_in_new_target);
            }
            self.stack_.pop();
        } else {
            self.stack_.last_mut().unwrap().target = Some(new_target_surface);
            self.stack_.last_mut().unwrap().occlusion_from_inside_target =
                old_occlusion_from_inside_target_in_new_target;
            if !is_root {
                self.stack_
                    .last_mut()
                    .unwrap()
                    .occlusion_from_outside_target =
                    old_occlusion_from_outside_target_in_new_target;
            } else {
                self.stack_
                    .last_mut()
                    .unwrap()
                    .occlusion_from_outside_target
                    .Clear();
            }
        }
        if !context
            .surface(old_surface)
            .BackdropFilters(context.property_trees())
            .HasFilterThatMovesPixels()
        {
            return;
        }
        ReduceOcclusionBelowSurface(
            old_surface,
            unoccluded_surface_rect,
            context.surface(old_surface).draw_transform(),
            &mut self.stack_.last_mut().unwrap().occlusion_from_inside_target,
            context,
        );
        ReduceOcclusionBelowSurface(
            old_surface,
            unoccluded_surface_rect,
            context.surface(old_surface).draw_transform(),
            &mut self
                .stack_
                .last_mut()
                .unwrap()
                .occlusion_from_outside_target,
            context,
        );
    }

    // Source: cc/trees/occlusion_tracker.cc:340-394
    fn MarkOccludedBehindLayer<C: OcclusionContext<Transform = T, Surface = S, Layer = L>>(
        &mut self,
        layer: L,
        context: &C,
    ) {
        debug_assert!(!self.stack_.is_empty());
        let layer = context.layer(layer);
        debug_assert!(
            Some(layer.render_target(context.property_trees()))
                == self.stack_.last().unwrap().target
        );
        if layer.draw_opacity() < 1.0 {
            return;
        }
        if layer.Is3dSorted(context.property_trees()) {
            return;
        }
        if !layer.draw_properties().mask_filter_info().IsEmpty() {
            return;
        }
        let opaque_layer_region = layer.VisibleOpaqueRegion();
        if opaque_layer_region.IsEmpty() {
            return;
        }
        let effect_node = context
            .property_trees()
            .effect_tree()
            .Node(layer.effect_tree_index());
        if !effect_node.HasRenderSurface() && !IsOccludingBlendMode(effect_node.blend_mode()) {
            return;
        }
        debug_assert!(layer
            .visible_layer_rect()
            .Contains(opaque_layer_region.bounds()));
        let draw_transform = layer.DrawTransform(context.property_trees());
        if !draw_transform.NonDegeneratePreserves2dAxisAlignment() {
            return;
        }
        let mut clip_rect_in_target = ScreenSpaceClipRectInTargetSurface(
            layer.render_target(context.property_trees()),
            self.screen_space_clip_rect_,
            context,
        );
        if layer.is_clipped() {
            clip_rect_in_target.Intersect(layer.clip_rect());
        } else {
            clip_rect_in_target.Intersect(
                context
                    .surface(layer.render_target(context.property_trees()))
                    .content_rect(),
            );
        }
        for i in 0..opaque_layer_region.GetRegionComplexity() {
            let mut transformed_rect = MathUtil::MapEnclosedRectWith2dAxisAlignedTransform(
                &draw_transform,
                opaque_layer_region.GetRect(i),
            );
            transformed_rect.Intersect(clip_rect_in_target);
            if transformed_rect.width < self.minimum_tracking_size_.width()
                && transformed_rect.height < self.minimum_tracking_size_.height()
            {
                continue;
            }
            self.stack_
                .last_mut()
                .unwrap()
                .occlusion_from_inside_target
                .Union(transformed_rect);
        }
    }

    // Source: cc/trees/occlusion_tracker.cc:396-405
    pub fn ComputeVisibleRegionInScreen<B: SkRegionBackend>(
        &self,
        layer_tree: &dyn OcclusionLayerTree<S>,
    ) -> Region<B> {
        debug_assert!(layer_tree.RootRenderSurface() == self.stack_.last().unwrap().target);
        let occluded = &self.stack_.last().unwrap().occlusion_from_inside_target;
        let mut visible_region = Region::new(self.screen_space_clip_rect_);
        for i in 0..occluded.GetRegionComplexity() {
            visible_region.Subtract(&occluded.GetRect(i));
        }
        visible_region
    }
}
