// Translation status: complete
// Copyright 2011 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/render_surface_impl.h:1-40,412-413;
//         cc/layers/render_surface_impl.cc:1-37,670.
use super::{
    append_quads_context::AppendQuadsContext, append_quads_data::AppendQuadsData,
    draw_mode::DrawMode, layer_impl::LayerImpl,
};
use crate::{
    adapter::{
        damage_context::{DamageFilters, DamageTransform},
        draw_property::{DrawFilters, ViewTransitionToken},
        geometry::{
            PointF, Rect, RectF, Size, SizeF, ToEnclosedRect, ToEnclosingRect,
            ToEnclosingRectIgnoringError, Vector2dF,
        },
        input_types::{ElementId, RemapElementIdToCcNamespace},
        layer_impl::{LayerImplRenderPass, LayerImplSharedQuadState},
        layer_tree_impl::{Color4f, LayerTreeId, PropertyTreesFor, ResourceId},
        occlusion::SkBlendMode,
        property_tree::PropertyTreeTransform,
        property_types::{
            FilterOperations, MaskFilterInfo, SubtreeCaptureId, ViewTransitionElementResourceId,
        },
        render_surface_impl::*,
        solid_color_layer_impl::SolidColorRenderPass,
    },
    cc::{
        base::math_util::MathUtil,
        debug::{debug_colors::DebugColors, layer_tree_debug_state::DebugBorderType},
        trees::{
            damage_tracker::DamageTracker,
            effect_node::{EffectNode, RenderSurfaceReason},
            occlusion::Occlusion,
            property_ids::{kInvalidPropertyNodeId, kRootPropertyNodeId},
        },
    },
};
pub use compositor_draw::render_surface_policy::RenderSurfacePropertyChangedFlags;
use std::{cell::Cell, collections::BTreeSet};
// Source: cc/layers/render_surface_impl.h:337-372; cc/layers/render_surface_impl.cc:75,77.
struct DrawProperties<T> {
    draw_opacity: f32,
    draw_transform: T,
    pixel_alignment_offset: Vector2dF,
    screen_space_transform: T,
    content_rect: Rect,
    clip_rect: Rect,
    is_clipped: bool,
    mask_filter_info: MaskFilterInfo,
    is_fast_rounded_corner: bool,
}
impl<T: Default> Default for DrawProperties<T> {
    fn default() -> Self {
        Self {
            draw_opacity: 1.,
            draw_transform: T::default(),
            pixel_alignment_offset: Default::default(),
            screen_space_transform: T::default(),
            content_rect: Default::default(),
            clip_rect: Default::default(),
            is_clipped: false,
            mask_filter_info: Default::default(),
            is_fast_rounded_corner: false,
        }
    }
}
// Source: cc/layers/render_surface_impl.h:55-410. Native resident surface owner.
// Native raw tree pointer is an identity only; every access borrows the actual
// tree/context for the duration of the source operation.
pub struct RenderSurfaceImpl<B: RenderSurfaceTypes> {
    layer_tree_impl_: Option<LayerTreeId>,
    id_: ElementId,
    effect_tree_index_: i32,
    stable_id_for_shared_quad_state_: i32,
    draw_properties_: DrawProperties<B::Transform>,
    accumulated_content_rect_: Rect,
    num_contributors_: i32,
    common_ancestor_clip_id_: i32,
    surface_property_changed_: bool,
    ancestor_property_changed_: bool,
    contributes_to_drawn_surface_: bool,
    is_render_surface_list_member_: bool,
    // DamageTracker walks an immutable view of the resident tree while this
    // one source scalar is updated through contributing-surface aliases. Cell
    // preserves that live owner field without a copied surface/tree snapshot.
    intersects_damage_under_: Cell<bool>,
    occlusion_in_content_space_: Occlusion<B::Transform>,
    nearest_occlusion_immune_ancestor_: Option<i32>,
    damage_tracker_: Box<DamageTracker<i32>>,
    deferred_contributing_layers_: Vec<i32>,
    view_transition_capture_content_rect_: Rect,
    has_view_transition_capture_contributions_: bool,
}
impl<B: RenderSurfaceTypes> RenderSurfaceImpl<B> {
    // Source: cc/layers/render_surface_impl.h:57; cc/layers/render_surface_impl.cc:39-49.
    pub fn new<C: RenderSurfaceContext<B> + ?Sized>(
        tree: Option<LayerTreeId>,
        id: ElementId,
        c: &C,
    ) -> Self {
        Self::new_with_stable_id_sequence_value(tree, id, c.NextStableIdSequenceValue())
    }
    // Same source constructor body; native creation only needs the external
    // stable-id sequence value, not a borrowed tree/context owner.
    pub fn new_with_stable_id_sequence_value(
        tree: Option<LayerTreeId>,
        id: ElementId,
        sequence: i32,
    ) -> Self {
        let stable = LayerImpl::<B>::GetNextStableIdForSharedQuadStateFromSequence(sequence);
        debug_assert!(id.IsValid());
        debug_assert_ne!(stable, 0);
        Self {
            layer_tree_impl_: tree,
            id_: id,
            effect_tree_index_: kInvalidPropertyNodeId,
            stable_id_for_shared_quad_state_: stable,
            draw_properties_: Default::default(),
            accumulated_content_rect_: Default::default(),
            num_contributors_: 0,
            common_ancestor_clip_id_: kInvalidPropertyNodeId,
            surface_property_changed_: false,
            ancestor_property_changed_: false,
            contributes_to_drawn_surface_: false,
            is_render_surface_list_member_: false,
            intersects_damage_under_: Cell::new(true),
            occlusion_in_content_space_: Default::default(),
            nearest_occlusion_immune_ancestor_: None,
            damage_tracker_: DamageTracker::Create(),
            deferred_contributing_layers_: Vec::new(),
            view_transition_capture_content_rect_: Default::default(),
            has_view_transition_capture_contributions_: false,
        }
    }
    pub fn tree_id(&self) -> Option<LayerTreeId> {
        self.layer_tree_impl_
    }
    // Source: cc/layers/render_surface_impl.h:65-66; cc/layers/render_surface_impl.cc:53-73.
    // Both const/mutable raw surface pointers are the same canonical effect-tree
    // identity. The caller resolves this id through the current live tree; no
    // surface is copied and the root result is this same resident owner.
    pub fn render_target<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> Option<i32> {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.render_targetForPropertyTrees(c.property_trees())
    }
    // Rust borrow split of the source body above; reads only this resident tree.
    pub fn render_targetForPropertyTrees(&self, p: &PropertyTreesFor<B>) -> Option<i32> {
        self.render_targetForQueries(p)
    }
    // Same source body, narrowed to resident effect-node/membership queries.
    pub fn render_targetForQueries<Q: RenderSurfaceQueries<B> + ?Sized>(
        &self,
        q: &Q,
    ) -> Option<i32> {
        let node = q.EffectNode(self.EffectTreeIndex());
        if node.target_id != kRootPropertyNodeId {
            q.HasRenderSurface(node.target_id).then_some(node.target_id)
        } else {
            Some(self.EffectTreeIndex())
        }
    }
    // Source: cc/layers/render_surface_impl.h:70; cc/layers/render_surface_impl.cc:79-105.
    pub fn DrawableContentRect<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> RectF {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.DrawableContentRectForPropertyTrees(c.property_trees())
    }
    // Rust borrow split of the source body above; reads only this resident tree.
    pub fn DrawableContentRectForPropertyTrees(&self, p: &PropertyTreesFor<B>) -> RectF {
        self.DrawableContentRectForQueries(p)
    }
    // Same source body, narrowed to resident effect-node/membership queries.
    pub fn DrawableContentRectForQueries<Q: RenderSurfaceQueries<B> + ?Sized>(
        &self,
        q: &Q,
    ) -> RectF {
        if self.content_rect().IsEmpty() {
            return RectF::default();
        }
        let mut rect = self.content_rect();
        let filters = self.FiltersForQueries(q);
        if !filters.IsEmpty() {
            rect = DamageFilters::MapRect(
                filters,
                rect,
                &self.SurfaceScaleForQueries(q).ToFlattenedSkMatrix(),
            );
        }
        let mut drawable = MathUtil::MapClippedRect(self.draw_transform(), RectF::from(rect));
        if self.is_clipped() {
            drawable.Intersect(RectF::from(self.clip_rect()));
        }
        if drawable.x().is_nan()
            || drawable.y().is_nan()
            || drawable.right().is_nan()
            || drawable.bottom().is_nan()
        {
            return RectF::default();
        }
        drawable
    }
    // Source: cc/layers/render_surface_impl.h:89; cc/layers/render_surface_impl.cc:107-109.
    pub fn BlendMode<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> SkBlendMode {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.BlendModeForPropertyTrees(c.property_trees())
    }
    // Rust borrow split of the source body above; reads only this resident tree.
    pub fn BlendModeForPropertyTrees(&self, p: &PropertyTreesFor<B>) -> SkBlendMode {
        self.OwningEffectNodeForPropertyTrees(p).unwrap().blend_mode
    }
    // Source: cc/layers/render_surface_impl.h:98; cc/layers/render_surface_impl.cc:111-113.
    pub fn GetDebugBorderColor(&self) -> Color4f {
        DebugColors::SurfaceBorderColor()
    }
    // Source: cc/layers/render_surface_impl.h:99; cc/layers/render_surface_impl.cc:115-118.
    pub fn GetDebugBorderWidth<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> f32 {
        DebugColors::SurfaceBorderWidth(if self.layer_tree_impl_.is_some() {
            c.device_scale_factor()
        } else {
            1.
        }) as f32
    }
    // Source: cc/layers/render_surface_impl.h:230; cc/layers/render_surface_impl.cc:120-125.
    pub fn BackdropMaskLayer<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> Option<i32> {
        let id = self.OwningEffectNode(c).unwrap().backdrop_mask_element_id;
        if !id.IsValid() {
            return None;
        }
        c.LayerByElementId(id)
    }
    // Source: cc/layers/render_surface_impl.h:225; cc/layers/render_surface_impl.cc:127-129.
    pub fn HasMaskingContributingSurface<C: RenderSurfaceContext<B> + ?Sized>(
        &self,
        c: &C,
    ) -> bool {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.HasMaskingContributingSurfaceForPropertyTrees(c.property_trees())
    }
    // Rust borrow split of the source body above; reads only this resident tree.
    pub fn HasMaskingContributingSurfaceForPropertyTrees(&self, p: &PropertyTreesFor<B>) -> bool {
        self.OwningEffectNodeForPropertyTrees(p)
            .unwrap()
            .has_masking_child
    }
    // Source: cc/layers/render_surface_impl.h:227; cc/layers/render_surface_impl.cc:131-133.
    pub fn Filters<'a, C: RenderSurfaceContext<B> + ?Sized>(&self, c: &'a C) -> &'a B::Filters {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.FiltersForPropertyTrees(c.property_trees())
    }
    // Rust borrow split of the source body above; reads only this resident tree.
    pub fn FiltersForPropertyTrees<'a>(&self, p: &'a PropertyTreesFor<B>) -> &'a B::Filters {
        self.FiltersForQueries(p)
    }
    // Same source body, narrowed to resident effect-node/membership queries.
    pub fn FiltersForQueries<'a, Q: RenderSurfaceQueries<B> + ?Sized>(
        &self,
        q: &'a Q,
    ) -> &'a B::Filters {
        &self.OwningEffectNodeForQueries(q).unwrap().filters
    }
    // Source: cc/layers/render_surface_impl.h:231; cc/layers/render_surface_impl.cc:135-140.
    pub fn SurfaceScale<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> B::Transform {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.SurfaceScaleForPropertyTrees(c.property_trees())
    }
    // Rust borrow split of the source body above; reads only this resident tree.
    pub fn SurfaceScaleForPropertyTrees(&self, p: &PropertyTreesFor<B>) -> B::Transform {
        self.SurfaceScaleForQueries(p)
    }
    // Same source body, narrowed to resident effect-node/membership queries.
    pub fn SurfaceScaleForQueries<Q: RenderSurfaceQueries<B> + ?Sized>(
        &self,
        q: &Q,
    ) -> B::Transform {
        let mut transform = B::Transform::default();
        let scale = self
            .OwningEffectNodeForQueries(q)
            .unwrap()
            .surface_contents_scale;
        transform.Scale(scale.x(), scale.y());
        transform
    }
    // Source: cc/layers/render_surface_impl.h:228; cc/layers/render_surface_impl.cc:142-144.
    pub fn BackdropFilters<'a, C: RenderSurfaceContext<B> + ?Sized>(
        &self,
        c: &'a C,
    ) -> &'a B::Filters {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.BackdropFiltersForPropertyTrees(c.property_trees())
    }
    // Rust borrow split of the source body above; reads only this resident tree.
    pub fn BackdropFiltersForPropertyTrees<'a>(
        &self,
        p: &'a PropertyTreesFor<B>,
    ) -> &'a B::Filters {
        &self
            .OwningEffectNodeForPropertyTrees(p)
            .unwrap()
            .backdrop_filters
    }
    // Source: cc/layers/render_surface_impl.h:229; cc/layers/render_surface_impl.cc:146-148.
    pub fn BackdropFilterBounds<C: RenderSurfaceContext<B> + ?Sized>(
        &self,
        c: &C,
    ) -> Option<B::Path> {
        self.OwningEffectNode(c)
            .unwrap()
            .backdrop_filter_bounds
            .clone()
    }
    // Source: cc/layers/render_surface_impl.h:233; cc/layers/render_surface_impl.cc:150-152.
    pub fn TrilinearFiltering<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> bool {
        self.OwningEffectNode(c).unwrap().trilinear_filtering
    }
    // Source: cc/layers/render_surface_impl.h:235; cc/layers/render_surface_impl.cc:154-156.
    pub fn HasCopyRequest<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> bool {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.HasCopyRequestForPropertyTrees(c.property_trees())
    }
    // Rust borrow split of the source body above; reads only this resident tree.
    pub fn HasCopyRequestForPropertyTrees(&self, p: &PropertyTreesFor<B>) -> bool {
        self.HasCopyRequestForQueries(p)
    }
    // Same source body, narrowed to resident effect-node/membership queries.
    pub fn HasCopyRequestForQueries<Q: RenderSurfaceQueries<B> + ?Sized>(&self, q: &Q) -> bool {
        self.OwningEffectNodeForQueries(q).unwrap().has_copy_request
    }
    // Source: cc/layers/render_surface_impl.h:240; cc/layers/render_surface_impl.cc:158-160.
    pub fn SubtreeCaptureId<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> SubtreeCaptureId {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.SubtreeCaptureIdForPropertyTrees(c.property_trees())
    }
    // Rust borrow split of the source body above; reads only this resident tree.
    pub fn SubtreeCaptureIdForPropertyTrees(&self, p: &PropertyTreesFor<B>) -> SubtreeCaptureId {
        self.SubtreeCaptureIdForQueries(p)
    }
    // Same source body, narrowed to resident effect-node/membership queries.
    pub fn SubtreeCaptureIdForQueries<Q: RenderSurfaceQueries<B> + ?Sized>(
        &self,
        q: &Q,
    ) -> SubtreeCaptureId {
        self.OwningEffectNodeForQueries(q)
            .unwrap()
            .subtree_capture_id
    }
    // Source: cc/layers/render_surface_impl.h:244; cc/layers/render_surface_impl.cc:162-164.
    pub fn SubtreeSize<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> Size {
        self.OwningEffectNode(c).unwrap().subtree_size
    }
    // Source: cc/layers/render_surface_impl.h:246; cc/layers/render_surface_impl.cc:166-168.
    pub fn ShouldCacheRenderSurface<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> bool {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.ShouldCacheRenderSurfaceForPropertyTrees(c.property_trees())
    }
    // Rust borrow split of the source body above; reads only this resident tree.
    pub fn ShouldCacheRenderSurfaceForPropertyTrees(&self, p: &PropertyTreesFor<B>) -> bool {
        self.ShouldCacheRenderSurfaceForQueries(p)
    }
    // Same source body, narrowed to resident effect-node/membership queries.
    pub fn ShouldCacheRenderSurfaceForQueries<Q: RenderSurfaceQueries<B> + ?Sized>(
        &self,
        q: &Q,
    ) -> bool {
        self.OwningEffectNodeForQueries(q)
            .unwrap()
            .cache_render_surface
    }
    // Source: cc/layers/render_surface_impl.h:251; cc/layers/render_surface_impl.cc:170-173.
    pub fn CopyOfOutputRequired<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> bool {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.CopyOfOutputRequiredForPropertyTrees(c.property_trees())
    }
    // Rust borrow split of the source body above; reads only this resident tree.
    pub fn CopyOfOutputRequiredForPropertyTrees(&self, p: &PropertyTreesFor<B>) -> bool {
        self.CopyOfOutputRequiredForQueries(p)
    }
    // Same source body, narrowed to resident effect-node/membership queries.
    pub fn CopyOfOutputRequiredForQueries<Q: RenderSurfaceQueries<B> + ?Sized>(
        &self,
        q: &Q,
    ) -> bool {
        self.HasCopyRequestForQueries(q)
            || self.ShouldCacheRenderSurfaceForQueries(q)
            || self.SubtreeCaptureIdForQueries(q).is_valid()
            || self.IsViewTransitionElementForQueries(q)
    }
    // Source: cc/layers/render_surface_impl.h:290; cc/layers/render_surface_impl.cc:175-177.
    pub fn IsViewTransitionElement<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> bool {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.IsViewTransitionElementForPropertyTrees(c.property_trees())
    }
    // Rust borrow split of the source body above; reads only this resident tree.
    pub fn IsViewTransitionElementForPropertyTrees(&self, p: &PropertyTreesFor<B>) -> bool {
        self.IsViewTransitionElementForQueries(p)
    }
    // Same source body, narrowed to resident effect-node/membership queries.
    pub fn IsViewTransitionElementForQueries<Q: RenderSurfaceQueries<B> + ?Sized>(
        &self,
        q: &Q,
    ) -> bool {
        self.ViewTransitionElementResourceIdForQueries(q).IsValid()
    }
    // Source: cc/layers/render_surface_impl.h:293; cc/layers/render_surface_impl.cc:180-185.
    pub fn IsUnbounded<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> bool {
        self.IsUnboundedForPropertyTrees(c.property_trees(), c.settings().enable_unbounded_element)
    }
    pub fn IsUnboundedForPropertyTrees(&self, p: &PropertyTreesFor<B>, enabled: bool) -> bool {
        self.IsUnboundedForQueries(p, enabled)
    }
    // Same source body, narrowed to resident effect-node/membership queries.
    pub fn IsUnboundedForQueries<Q: RenderSurfaceQueries<B> + ?Sized>(
        &self,
        q: &Q,
        enabled: bool,
    ) -> bool {
        enabled
            && self.OwningEffectNodeForQueries(q).is_some_and(|node| {
                node.render_surface_reason == RenderSurfaceReason::kUnboundedElement
            })
    }
    // Source: cc/layers/render_surface_impl.h:298-299; cc/layers/render_surface_impl.cc:187-190.
    pub fn ViewTransitionElementResourceId<'a, C: RenderSurfaceContext<B> + ?Sized>(
        &self,
        c: &'a C,
    ) -> &'a ViewTransitionElementResourceId {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.ViewTransitionElementResourceIdForPropertyTrees(c.property_trees())
    }
    // Rust borrow split of the source body above; reads only this resident tree.
    pub fn ViewTransitionElementResourceIdForPropertyTrees<'a>(
        &self,
        p: &'a PropertyTreesFor<B>,
    ) -> &'a ViewTransitionElementResourceId {
        self.ViewTransitionElementResourceIdForQueries(p)
    }
    // Same source body, narrowed to resident effect-node/membership queries.
    pub fn ViewTransitionElementResourceIdForQueries<'a, Q: RenderSurfaceQueries<B> + ?Sized>(
        &self,
        q: &'a Q,
    ) -> &'a ViewTransitionElementResourceId {
        &self
            .OwningEffectNodeForQueries(q)
            .unwrap()
            .view_transition_element_resource_id
    }
    // Source: cc/layers/render_surface_impl.h:280; cc/layers/render_surface_impl.cc:192-194.
    pub fn TransformTreeIndex<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> i32 {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.TransformTreeIndexForPropertyTrees(c.property_trees())
    }
    // Rust borrow split of the source body above; reads only this resident tree.
    pub fn TransformTreeIndexForPropertyTrees(&self, p: &PropertyTreesFor<B>) -> i32 {
        self.OwningEffectNodeForPropertyTrees(p)
            .unwrap()
            .transform_id
    }
    // Source: cc/layers/render_surface_impl.h:281; cc/layers/render_surface_impl.cc:196-198.
    pub fn ClipTreeIndex<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> i32 {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.ClipTreeIndexForPropertyTrees(c.property_trees())
    }
    // Rust borrow split of the source body above; reads only this resident tree.
    pub fn ClipTreeIndexForPropertyTrees(&self, p: &PropertyTreesFor<B>) -> i32 {
        self.OwningEffectNodeForPropertyTrees(p).unwrap().clip_id
    }
    // Source: cc/layers/render_surface_impl.h:284; cc/layers/render_surface_impl.cc:200-202.
    pub fn EffectTreeIndex(&self) -> i32 {
        self.effect_tree_index_
    }
    // Source: cc/layers/render_surface_impl.h:286; cc/layers/render_surface_impl.cc:204-210.
    pub fn OwningEffectNode<'a, C: RenderSurfaceContext<B> + ?Sized>(
        &self,
        c: &'a C,
    ) -> Option<&'a EffectNode<B::Filters, B::Path>> {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.OwningEffectNodeForPropertyTrees(c.property_trees())
    }
    pub fn OwningEffectNodeForPropertyTrees<'a>(
        &self,
        p: &'a PropertyTreesFor<B>,
    ) -> Option<&'a EffectNode<B::Filters, B::Path>> {
        self.OwningEffectNodeForQueries(p)
    }
    // Same source body, narrowed to resident effect-node/membership queries.
    pub fn OwningEffectNodeForQueries<'a, Q: RenderSurfaceQueries<B> + ?Sized>(
        &self,
        q: &'a Q,
    ) -> Option<&'a EffectNode<B::Filters, B::Path>> {
        if self.EffectTreeIndex() == kInvalidPropertyNodeId {
            return None;
        }
        Some(q.EffectNode(self.EffectTreeIndex()))
    }
    // Source: cc/layers/render_surface_impl.h:287; cc/layers/render_surface_impl.cc:212-215.
    pub fn OwningEffectNodeMutableForTest<
        'a,
        C: crate::adapter::render_surface_impl::RenderSurfaceMutableContext<B> + ?Sized,
    >(
        &self,
        c: &'a mut C,
    ) -> &'a mut EffectNode<B::Filters, B::Path> {
        c.property_trees_mut()
            .effect_tree_mutable()
            .MutableNode(self.EffectTreeIndex())
    }
    // Source: cc/layers/render_surface_impl.h:125; cc/layers/render_surface_impl.cc:217-223.
    pub fn SetClipRect(&mut self, rect: Rect) {
        if rect == self.draw_properties_.clip_rect {
            return;
        }
        self.surface_property_changed_ = true;
        self.draw_properties_.clip_rect = rect;
    }
    // Source: cc/layers/render_surface_impl.h:311; cc/layers/render_surface_impl.cc:225-231.
    fn SetContentRect(&mut self, rect: Rect) {
        if rect == self.draw_properties_.content_rect {
            return;
        }
        self.surface_property_changed_ = true;
        self.draw_properties_.content_rect = rect;
    }
    // Source: cc/layers/render_surface_impl.h:173; cc/layers/render_surface_impl.cc:233-235.
    pub fn SetContentRectForTesting(&mut self, rect: Rect) {
        self.SetContentRect(rect);
    }
    // Source: cc/layers/render_surface_impl.h:313-314; cc/layers/render_surface_impl.cc:237-246.
    fn CalculateExpandedClipForFilters<C: RenderSurfaceContentContext<B>>(
        &self,
        target_to_surface: &B::Transform,
        c: &C,
    ) -> Rect {
        let clip = MathUtil::ProjectEnclosingClippedRect(target_to_surface, self.clip_rect());
        let expanded = self
            .FiltersForQueries(c)
            .ExpandRectWithMatrix(clip, &self.SurfaceScaleForQueries(c).ToFlattenedSkMatrix());
        MathUtil::MapEnclosingClippedRect(self.draw_transform(), expanded)
    }
    // Source: cc/layers/render_surface_impl.h:312; cc/layers/render_surface_impl.cc:248-294.
    fn CalculateClippedAccumulatedContentRect<C: RenderSurfaceContentContext<B>>(
        &self,
        c: &C,
    ) -> Rect {
        if !self.ShouldClip(c) || !self.is_clipped() {
            return self.accumulated_content_rect();
        }
        if self.accumulated_content_rect().IsEmpty() {
            return Rect::default();
        }
        let mut inverse = B::Transform::default();
        if !PropertyTreeTransform::GetInverse(self.draw_transform(), &mut inverse) {
            return self.accumulated_content_rect();
        }
        let target = MathUtil::MapEnclosingClippedRect(
            self.draw_transform(),
            self.accumulated_content_rect(),
        );
        if self.clip_rect().Contains(target) {
            return self.accumulated_content_rect();
        }
        let mut clipped = if DrawFilters::HasFilterThatMovesPixels(self.FiltersForQueries(c)) {
            self.CalculateExpandedClipForFilters(&inverse, c)
        } else {
            self.clip_rect()
        };
        clipped.Intersect(target);
        if clipped.IsEmpty() {
            return Rect::default();
        }
        let mut local =
            MathUtil::ProjectEnclosingClippedRectIgnoringError(&inverse, clipped, f32::EPSILON);
        local.Intersect(self.accumulated_content_rect());
        local
    }
}
impl<B: RenderSurfaceTypes> RenderSurfaceImpl<B> {
    // Source: cc/layers/render_surface_impl.h:171; cc/layers/render_surface_impl.cc:296-344.
    pub fn CalculateContentRectFromAccumulatedContentRect<C: RenderSurfaceContext<B> + ?Sized>(
        &mut self,
        max_texture_size: i32,
        c: &C,
    ) {
        self.CalculateContentRectFromAccumulatedContentRectWithContentContext(
            max_texture_size,
            &SurfaceFullContextView::<B, C>(c, std::marker::PhantomData),
        );
    }
    // Source body above, using only the live disjoint content/query facets.
    pub fn CalculateContentRectFromAccumulatedContentRectWithContentContext<
        C: RenderSurfaceContentContext<B>,
    >(
        &mut self,
        max_texture_size: i32,
        c: &C,
    ) {
        debug_assert_ne!(
            self.render_targetForQueries(c),
            Some(self.EffectTreeIndex())
        );
        for &layer in &self.deferred_contributing_layers_ {
            debug_assert!(c.LayerDrawsContent(layer));
            self.accumulated_content_rect_
                .Union(c.LayerVisibleDrawableContentRect(layer));
            self.view_transition_capture_content_rect_
                .Union(c.LayerVisibleDrawableContentRect(layer));
        }
        self.deferred_contributing_layers_.clear();
        let rect = self.CalculateClippedAccumulatedContentRect(c);
        #[cfg(debug_assertions)]
        if self.IsViewTransitionElementForQueries(c) {
            debug_assert!(rect.width <= max_texture_size);
            debug_assert!(rect.height <= max_texture_size);
        }
        let unclamped_width = rect.width;
        let unclamped_height = rect.height;
        let (rect, capture_rect, exceeded) =
            compositor_draw::render_surface_policy::ClampSurfaceContentRects(
                rect,
                self.view_transition_capture_content_rect_,
                max_texture_size,
            );
        if exceeded {
            c.LogMaxTextureExceeded(max_texture_size, unclamped_width, unclamped_height);
        }
        self.SetContentRect(rect);
        self.view_transition_capture_content_rect_ = capture_rect;
    }
    // Source: cc/layers/render_surface_impl.h:172; cc/layers/render_surface_impl.cc:346-353.
    pub fn SetContentRectToViewport<C: RenderSurfaceContext<B> + ?Sized>(&mut self, c: &C) {
        debug_assert_eq!(self.render_target(c), Some(self.EffectTreeIndex()));
        self.SetContentRectToViewportForClip(c.property_trees().clip_tree().ViewportClip());
    }
    pub fn SetContentRectToViewportForClip(&mut self, viewport_clip: RectF) {
        debug_assert!(self.deferred_contributing_layers_.is_empty());
        let viewport = ToEnclosingRect(viewport_clip);
        self.SetContentRect(viewport);
    }
    // Source: cc/layers/render_surface_impl.h:179; cc/layers/render_surface_impl.cc:355-358.
    pub fn ClearAccumulatedContentRect(&mut self) {
        self.accumulated_content_rect_ = Rect::default();
        self.view_transition_capture_content_rect_ = Rect::default();
    }
    // Source: cc/layers/render_surface_impl.h:182-185; cc/layers/render_surface_impl.cc:360-394.
    pub fn AccumulateContentRectFromContributingLayer<C: RenderSurfaceContext<B> + ?Sized>(
        &mut self,
        layer: i32,
        tokens: &BTreeSet<ViewTransitionToken>,
        c: &C,
    ) {
        self.AccumulateContentRectFromContributingLayerWithContentContext(
            layer,
            tokens,
            &SurfaceFullContextView::<B, C>(c, std::marker::PhantomData),
        );
    }
    // Source body above, using only the live disjoint content/query facets.
    pub fn AccumulateContentRectFromContributingLayerWithContentContext<
        C: RenderSurfaceContentContext<B>,
    >(
        &mut self,
        layer: i32,
        tokens: &BTreeSet<ViewTransitionToken>,
        c: &C,
    ) {
        debug_assert!(c.LayerDrawsContent(layer));
        debug_assert_eq!(self.EffectTreeIndex(), c.LayerRenderTarget(layer, c));
        let target_is_self = self.render_targetForQueries(c) == Some(self.EffectTreeIndex());
        let (resource_is_valid, resource_matches_token) = if target_is_self {
            (false, true)
        } else {
            let resource = c.LayerViewTransitionResourceId(layer);
            let valid = resource.IsValid();
            (valid, !valid || resource.MatchesToken(tokens))
        };
        match compositor_draw::render_surface_policy::ClassifyContributingLayer(
            target_is_self,
            resource_is_valid,
            resource_matches_token,
        ) {
            compositor_draw::render_surface_policy::ContributingLayerDisposition::Ignore => {}
            compositor_draw::render_surface_policy::ContributingLayerDisposition::Defer => {
                self.deferred_contributing_layers_.push(layer);
            }
            compositor_draw::render_surface_policy::ContributingLayerDisposition::Accumulate => {
                self.accumulated_content_rect_
                    .Union(c.LayerVisibleDrawableContentRect(layer));
                self.view_transition_capture_content_rect_
                    .Union(c.LayerVisibleDrawableContentRect(layer));
            }
        }
    }
    // Source: cc/layers/render_surface_impl.h:186-189; cc/layers/render_surface_impl.cc:396-434.
    pub fn AccumulateContentRectFromContributingRenderSurface<
        C: RenderSurfaceContext<B> + ?Sized,
    >(
        &mut self,
        surface: &Self,
        tokens: &BTreeSet<ViewTransitionToken>,
        c: &C,
    ) {
        self.AccumulateContentRectFromContributingRenderSurfaceWithContentContext(
            surface,
            tokens,
            &SurfaceFullContextView::<B, C>(c, std::marker::PhantomData),
        );
    }
    // Source body above, using only the live disjoint content/query facets.
    pub fn AccumulateContentRectFromContributingRenderSurfaceWithContentContext<
        C: RenderSurfaceContentContext<B>,
    >(
        &mut self,
        surface: &Self,
        tokens: &BTreeSet<ViewTransitionToken>,
        c: &C,
    ) {
        debug_assert_ne!(self.EffectTreeIndex(), surface.EffectTreeIndex());
        debug_assert_eq!(
            Some(self.EffectTreeIndex()),
            surface.render_targetForQueries(c)
        );
        let target_is_self = self.render_targetForQueries(c) == Some(self.EffectTreeIndex());
        let is_view_transition = !target_is_self && surface.IsViewTransitionElementForQueries(c);
        let resource_matches_token = !is_view_transition
            || surface
                .ViewTransitionElementResourceIdForQueries(c)
                .MatchesToken(tokens);
        let disposition = compositor_draw::render_surface_policy::ClassifyContributingSurface(
            target_is_self,
            is_view_transition,
            resource_matches_token,
        );
        if disposition
            == compositor_draw::render_surface_policy::ContributingSurfaceDisposition::Ignore
        {
            return;
        }
        let drawable = surface.DrawableContentRectForQueries(c);
        self.accumulated_content_rect_
            .Union(ToEnclosingRectIgnoringError(drawable, f32::EPSILON));
        if disposition
            == compositor_draw::render_surface_policy::ContributingSurfaceDisposition::AccumulateNormalAndCapture
        {
            self.view_transition_capture_content_rect_
                .Union(ToEnclosedRect(drawable));
        }
    }
    // Source: cc/layers/render_surface_impl.h:259; cc/layers/render_surface_impl.cc:436-441.
    pub fn SurfacePropertyChanged(&self) -> bool {
        self.surface_property_changed_
    }
    // Source: cc/layers/render_surface_impl.h:260; cc/layers/render_surface_impl.cc:443-453.
    pub fn AncestorPropertyChanged<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> bool {
        debug_assert_eq!(self.layer_tree_impl_, Some(c.tree_id()));
        self.AncestorPropertyChangedForPropertyTrees(c.property_trees())
    }
    // Rust borrow split of the source body above; reads only this resident tree.
    pub fn AncestorPropertyChangedForPropertyTrees(&self, p: &PropertyTreesFor<B>) -> bool {
        self.ancestor_property_changed_
            || p.full_tree_damaged()
            || p.transform_tree()
                .Node(self.TransformTreeIndexForPropertyTrees(p))
                .transform_changed()
            || p.effect_tree().Node(self.EffectTreeIndex()).effect_changed
    }
    // Source: cc/layers/render_surface_impl.h:261; cc/layers/render_surface_impl.cc:455-457.
    pub fn NoteAncestorPropertyChanged(&mut self) {
        self.ancestor_property_changed_ = true;
    }
    // Source: cc/layers/render_surface_impl.h:262; cc/layers/render_surface_impl.cc:459-461.
    pub fn HasDamageFromeContributingContent(&self) -> bool {
        self.damage_tracker_.has_damage_from_contributing_content()
    }
    // Source: cc/layers/render_surface_impl.h:265; cc/layers/render_surface_impl.cc:463-470.
    pub fn GetDamageRect(&self) -> Rect {
        let mut damage = Rect::default();
        let valid = self.damage_tracker_.GetDamageRectIfValid(&mut damage);
        if !valid {
            return self.content_rect();
        }
        damage
    }
    // Source: cc/layers/render_surface_impl.h:255; cc/layers/render_surface_impl.cc:472-475.
    pub fn GetPropertyChangeFlags(&self) -> RenderSurfacePropertyChangedFlags {
        RenderSurfacePropertyChangedFlags::new(
            self.surface_property_changed_,
            self.ancestor_property_changed_,
        )
    }
    // Source: cc/layers/render_surface_impl.h:256; cc/layers/render_surface_impl.cc:477-481.
    pub fn ApplyPropertyChangeFlags(&mut self, flags: &RenderSurfacePropertyChangedFlags) {
        self.surface_property_changed_ = flags.self_changed();
        self.ancestor_property_changed_ = flags.ancestor_changed();
    }
    // Source: cc/layers/render_surface_impl.h:258; cc/layers/render_surface_impl.cc:483-486.
    pub fn ResetPropertyChangedFlags(&mut self) {
        self.surface_property_changed_ = false;
        self.ancestor_property_changed_ = false;
    }
    // Source: cc/layers/render_surface_impl.h:269-271; cc/layers/render_surface_impl.cc:488-507.
    pub fn CreateRenderPass<C: RenderSurfaceContext<B> + ?Sized>(
        &self,
        tokens: &BTreeSet<ViewTransitionToken>,
        c: &mut C,
    ) -> Box<B::RenderPass> {
        let mut pass = self.CreateRenderPassCommon(self.render_pass_id(), self.content_rect(), c);
        pass.set_subtree_capture_id(self.SubtreeCaptureId(c));
        pass.set_cache_render_pass(self.ShouldCacheRenderSurface(c));
        if !self.ViewTransitionElementResourceId(c).MatchesToken(tokens) {
            pass.set_view_transition_element_resource_id(*self.ViewTransitionElementResourceId(c));
        }
        pass
    }
    // Source: cc/layers/render_surface_impl.h:272-275; cc/layers/render_surface_impl.cc:509-520.
    pub fn CreateViewTransitionCaptureRenderPass<C: RenderSurfaceContext<B> + ?Sized>(
        &self,
        _tokens: &BTreeSet<ViewTransitionToken>,
        c: &mut C,
    ) -> Box<B::RenderPass> {
        let mut pass = self.CreateRenderPassCommon(
            self.view_transition_capture_render_pass_id(),
            self.view_transition_capture_content_rect_,
            c,
        );
        debug_assert!(
            self.IsViewTransitionElement(c) || self.has_view_transition_capture_contributions()
        );
        pass.set_view_transition_element_resource_id(*self.ViewTransitionElementResourceId(c));
        pass
    }
    // Source: cc/layers/render_surface_impl.h:318-320; cc/layers/render_surface_impl.cc:522-542.
    fn CreateRenderPassCommon<C: RenderSurfaceContext<B> + ?Sized>(
        &self,
        id: B::RenderPassId,
        output: Rect,
        c: &mut C,
    ) -> Box<B::RenderPass> {
        let mut pass = c.CreateRenderPass(self.num_contributors_);
        let mut damage = self.GetDamageRect();
        damage.Intersect(output);
        pass.SetNew(
            id,
            output,
            damage,
            &self.draw_properties_.screen_space_transform,
        );
        pass.SetFilters(self.Filters(c));
        pass.SetBackdropFilters(self.BackdropFilters(c));
        pass.SetBackdropFilterBounds(self.BackdropFilterBounds(c));
        pass.set_generate_mipmap(self.TrilinearFiltering(c));
        pass.set_subtree_size(self.SubtreeSize(c));
        let size = pass.subtree_size_mut();
        *size = Size::new(
            size.width().min(output.width),
            size.height().min(output.height),
        );
        pass.set_has_damage_from_contributing_content(self.HasDamageFromeContributingContent());
        pass
    }
    // Source: cc/layers/render_surface_impl.h:276-278; cc/layers/render_surface_impl.cc:544-663.
    pub fn AppendQuads<C: RenderSurfaceContext<B> + ?Sized>(
        &self,
        context: &AppendQuadsContext,
        pass: &mut B::RenderPass,
        _data: &mut AppendQuadsData<B::SurfaceId>,
        c: &C,
    ) {
        // Source 548-566 enumerates capture combinations; this is the original
        // exact predicate, retaining non-view-transition surface contributions.
        if self.IsViewTransitionElement(c)
            && (context.for_view_transition_capture
                || !self
                    .ViewTransitionElementResourceId(c)
                    .MatchesToken(&context.capture_view_transition_tokens))
        {
            return;
        }
        let output = if context.for_view_transition_capture
            && self.has_view_transition_capture_contributions()
        {
            self.view_transition_capture_content_rect_
        } else {
            self.content_rect()
        };
        let unoccluded = self
            .occlusion_in_content_space()
            .GetUnoccludedContentRect(output);
        if unoccluded.IsEmpty() && !self.Filters(c).HasReferenceFilter() {
            return;
        }
        let sorting = c
            .property_trees()
            .transform_tree()
            .Node(self.TransformTreeIndex(c))
            .sorting_context_id;
        let contents_opaque = false;
        let state = pass.CreateAndAppendSharedQuadState();
        let clip = if self.draw_properties_.is_clipped {
            Some(self.draw_properties_.clip_rect)
        } else {
            None
        };
        pass.SharedQuadStateMutable(state).SetAll(
            self.draw_transform(),
            output,
            output,
            self.mask_filter_info(),
            clip,
            contents_opaque,
            self.draw_properties_.draw_opacity,
            self.BlendMode(c),
            sorting,
            self.stable_id_for_shared_quad_state_ as u32,
            self.is_fast_rounded_corner(),
        );
        if c.debug_state()
            .show_debug_borders
            .test(DebugBorderType::RENDERPASS as usize)
        {
            pass.AppendDebugBorderQuad(
                state,
                output,
                unoccluded,
                self.GetDebugBorderColor(),
                self.GetDebugBorderWidth(c),
            );
        }
        let mask = self.BackdropMaskLayer(c);
        let mut resource = ResourceId::default();
        let mut texture = Size::default();
        let mut uv_rect = RectF::default();
        let scale = self.OwningEffectNode(c).unwrap().surface_contents_scale;
        if context.draw_mode != DrawMode::DRAW_MODE_RESOURCELESS_SOFTWARE
            && mask
                .is_some_and(|layer| c.LayerDrawsContent(layer) && !c.LayerBounds(layer).IsEmpty())
        {
            let layer = mask.unwrap();
            debug_assert_ne!(
                self.BlendMode(c),
                SkBlendMode::kDstIn,
                "kDstIn blend mode with mask layer is unsupported."
            );
            c.TraceMaskLayerGpuMemoryUsage(c.LayerGpuMemoryUsageInBytes(layer));
            let mut uv_size = SizeF::default();
            c.LayerGetContentsResourceId(layer, &mut resource, &mut texture, &mut uv_size);
            let bounds = c.LayerBounds(layer);
            let unclipped = SizeF::new(
                bounds.width() as f32 * scale.x(),
                bounds.height() as f32 * scale.y(),
            );
            let offset = c.LayerOffsetToTransformParent(layer);
            let offset = Vector2dF::new(offset.x() * scale.x(), offset.y() * scale.y());
            // Native gfx::ScaleRect(RectF(output)-offset, uv/unclipped) external
            // value arithmetic; the decisions remain above in canonical cc.
            let x = uv_size.width() / unclipped.width();
            let y = uv_size.height() / unclipped.height();
            uv_rect = RectF::new(
                (output.x as f32 - offset.x()) * x,
                (output.y as f32 - offset.y()) * y,
                output.width as f32 * x,
                output.height as f32 * y,
            );
        }
        let quad = pass.CreateAndAppendRenderPassDrawQuad();
        let id = if context.for_view_transition_capture
            && self.has_view_transition_capture_contributions()
        {
            self.view_transition_capture_render_pass_id()
        } else {
            self.render_pass_id()
        };
        pass.RenderPassDrawQuadSetAll(
            quad,
            state,
            output,
            unoccluded,
            true,
            id,
            resource,
            uv_rect,
            texture,
            scale,
            PointF::default(),
            !c.settings().enable_edge_anti_aliasing,
            self.OwningEffectNode(c).unwrap().backdrop_filter_quality,
            self.intersects_damage_under_.get(),
        );
    }
    // Source: cc/layers/render_surface_impl.h:324; cc/layers/render_surface_impl.cc:665-668.
    fn ShouldClip<C: RenderSurfaceContentContext<B>>(&self, c: &C) -> bool {
        !self.HasCopyRequestForQueries(c)
            && !self.ShouldCacheRenderSurfaceForQueries(c)
            && !self.IsViewTransitionElementForQueries(c)
            && !self.IsUnboundedForQueries(c, c.enable_unbounded_element())
    }
}
// Source: cc/layers/render_surface_impl.h:59; cc/layers/render_surface_impl.cc:51.
// Default destructor owns no external callback. Its resident unique DamageTracker
// and deferred native identity vector are destroyed with this unique owner.
// Source: cc/layers/render_surface_impl.h:315-317. TileMaskLayer is a declaration
// with no definition/call in this Chromium stem; no implementation is invented.

impl<B: RenderSurfaceTypes> RenderSurfaceImpl<B> {
    // Source: cc/layers/render_surface_impl.h:72-74.
    pub fn SetDrawOpacity(&mut self, opacity: f32) {
        self.draw_properties_.draw_opacity = opacity;
    }
    // Source: cc/layers/render_surface_impl.h:75.
    pub fn draw_opacity(&self) -> f32 {
        self.draw_properties_.draw_opacity
    }
    // Source: cc/layers/render_surface_impl.h:77-81.
    pub fn SetMaskFilterInfo(&mut self, mask: &MaskFilterInfo, fast: bool) {
        self.draw_properties_.mask_filter_info = mask.clone();
        self.draw_properties_.is_fast_rounded_corner = fast;
    }
    // Source: cc/layers/render_surface_impl.h:82-84.
    pub fn mask_filter_info(&self) -> &MaskFilterInfo {
        &self.draw_properties_.mask_filter_info
    }
    // Source: cc/layers/render_surface_impl.h:85-87.
    pub fn is_fast_rounded_corner(&self) -> bool {
        self.draw_properties_.is_fast_rounded_corner
    }
    // Source: cc/layers/render_surface_impl.h:91-93.
    pub fn SetNearestOcclusionImmuneAncestor(&mut self, surface: Option<i32>) {
        self.nearest_occlusion_immune_ancestor_ = surface;
    }
    // Source: cc/layers/render_surface_impl.h:94-96.
    pub fn nearest_occlusion_immune_ancestor(&self) -> Option<i32> {
        self.nearest_occlusion_immune_ancestor_
    }
    // Source: cc/layers/render_surface_impl.h:101-105.
    pub fn SetDrawTransform(&mut self, transform: &B::Transform, offset: Vector2dF) {
        self.draw_properties_.draw_transform = transform.clone();
        self.draw_properties_.pixel_alignment_offset = offset;
    }
    // Source: cc/layers/render_surface_impl.h:106-108.
    pub fn draw_transform(&self) -> &B::Transform {
        &self.draw_properties_.draw_transform
    }
    // Source: cc/layers/render_surface_impl.h:109-111.
    pub fn pixel_alignment_offset(&self) -> &Vector2dF {
        &self.draw_properties_.pixel_alignment_offset
    }
    // Source: cc/layers/render_surface_impl.h:113-115.
    pub fn SetScreenSpaceTransform(&mut self, transform: &B::Transform) {
        self.draw_properties_.screen_space_transform = transform.clone();
    }
    // Source: cc/layers/render_surface_impl.h:116-118.
    pub fn screen_space_transform(&self) -> &B::Transform {
        &self.draw_properties_.screen_space_transform
    }
    // Source: cc/layers/render_surface_impl.h:120-122.
    pub fn SetIsClipped(&mut self, clipped: bool) {
        self.draw_properties_.is_clipped = clipped;
    }
    // Source: cc/layers/render_surface_impl.h:123.
    pub fn is_clipped(&self) -> bool {
        self.draw_properties_.is_clipped
    }
    // Source: cc/layers/render_surface_impl.h:126.
    pub fn clip_rect(&self) -> Rect {
        self.draw_properties_.clip_rect
    }
    // Source: cc/layers/render_surface_impl.h:132-134.
    pub fn contributes_to_drawn_surface(&self) -> bool {
        self.contributes_to_drawn_surface_
    }
    // Source: cc/layers/render_surface_impl.h:135-137.
    pub fn set_contributes_to_drawn_surface(&mut self, flag: bool) {
        self.contributes_to_drawn_surface_ = flag;
    }
    // Source: cc/layers/render_surface_impl.h:145-149.
    pub fn set_common_ancestor_clip_id<C: RenderSurfaceContext<B> + ?Sized>(
        &mut self,
        id: i32,
        c: &C,
    ) {
        self.SetCommonAncestorClipIdForClip(id, self.ClipTreeIndex(c));
    }
    pub fn SetCommonAncestorClipIdForClip(&mut self, id: i32, own_clip_id: i32) {
        debug_assert_ne!(id, own_clip_id);
        debug_assert!(id < own_clip_id || id == kInvalidPropertyNodeId);
        self.common_ancestor_clip_id_ = id;
    }
    // Source: cc/layers/render_surface_impl.h:150-154.
    pub fn common_ancestor_clip_id<C: RenderSurfaceContext<B> + ?Sized>(&self, c: &C) -> i32 {
        self.CommonAncestorClipIdForPropertyTrees(c.property_trees())
    }
    pub fn CommonAncestorClipIdForPropertyTrees(&self, p: &PropertyTreesFor<B>) -> i32 {
        if self.common_ancestor_clip_id_ == kInvalidPropertyNodeId {
            self.ClipTreeIndexForPropertyTrees(p)
        } else {
            self.common_ancestor_clip_id_
        }
    }
    // Source: cc/layers/render_surface_impl.h:155-157.
    pub fn has_contributing_layer_that_escapes_clip(&self) -> bool {
        self.common_ancestor_clip_id_ != kInvalidPropertyNodeId
    }
    // Source: cc/layers/render_surface_impl.h:159-161.
    pub fn set_is_render_surface_list_member(&mut self, flag: bool) {
        self.is_render_surface_list_member_ = flag;
    }
    // Source: cc/layers/render_surface_impl.h:162-164.
    pub fn is_render_surface_list_member(&self) -> bool {
        self.is_render_surface_list_member_
    }
    // Source: cc/layers/render_surface_impl.h:166-168.
    pub fn set_intersects_damage_under(&self, flag: bool) {
        self.intersects_damage_under_.set(flag);
    }
    // Source: cc/layers/render_surface_impl.h:169.
    pub fn intersects_damage_under(&self) -> bool {
        self.intersects_damage_under_.get()
    }
    // Source: cc/layers/render_surface_impl.h:174.
    pub fn content_rect(&self) -> Rect {
        self.draw_properties_.content_rect
    }
    // Source: cc/layers/render_surface_impl.h:175-177.
    pub fn view_transition_capture_content_rect(&self) -> Rect {
        self.view_transition_capture_content_rect_
    }
    // Source: cc/layers/render_surface_impl.h:191-193.
    pub fn accumulated_content_rect(&self) -> Rect {
        self.accumulated_content_rect_
    }
    // Source: cc/layers/render_surface_impl.h:195.
    pub fn increment_num_contributors(&mut self) {
        self.num_contributors_ += 1;
    }
    // Source: cc/layers/render_surface_impl.h:196-199.
    pub fn decrement_num_contributors(&mut self) {
        self.num_contributors_ -= 1;
        debug_assert!(self.num_contributors_ >= 0);
    }
    // Source: cc/layers/render_surface_impl.h:200.
    pub fn reset_num_contributors(&mut self) {
        self.num_contributors_ = 0;
    }
    // Source: cc/layers/render_surface_impl.h:201.
    pub fn num_contributors(&self) -> i32 {
        self.num_contributors_
    }
    // Source: cc/layers/render_surface_impl.h:203-205.
    pub fn occlusion_in_content_space(&self) -> &Occlusion<B::Transform> {
        &self.occlusion_in_content_space_
    }
    // Source: cc/layers/render_surface_impl.h:206-208.
    pub fn set_occlusion_in_content_space(&mut self, occlusion: &Occlusion<B::Transform>) {
        self.occlusion_in_content_space_ = occlusion.clone();
    }
    // Source: cc/layers/render_surface_impl.h:210.
    pub fn id(&self) -> ElementId {
        self.id_
    }
    // Source: cc/layers/render_surface_impl.h:211-213.
    pub fn render_pass_id(&self) -> B::RenderPassId {
        self.id().GetInternalValue().into()
    }
    // Source: cc/layers/render_surface_impl.h:220-223.
    pub fn view_transition_capture_render_pass_id(&self) -> B::RenderPassId {
        RemapElementIdToCcNamespace(self.id())
            .GetInternalValue()
            .into()
    }
    // Source: cc/layers/render_surface_impl.h:264.
    pub fn damage_tracker(&self) -> &DamageTracker<i32> {
        &self.damage_tracker_
    }
    // Source: cc/layers/render_surface_impl.h:283.
    pub fn set_effect_tree_index(&mut self, index: i32) {
        self.effect_tree_index_ = index;
    }
    // Source: cc/layers/render_surface_impl.h:303-305.
    pub fn has_view_transition_capture_contributions(&self) -> bool {
        self.has_view_transition_capture_contributions_
    }
    // Source: cc/layers/render_surface_impl.h:306-308.
    pub fn set_has_view_transition_capture_contributions(&mut self, flag: bool) {
        self.has_view_transition_capture_contributions_ = flag;
    }
}

// Pure resident PropertyTree surface-value interface. Tree-dependent methods
// remain explicitly call-scoped above, never a saved host/tree backpointer.
impl<B: RenderSurfaceTypes> crate::adapter::property_tree::PropertyTreeRenderSurface
    for RenderSurfaceImpl<B>
{
    fn id(&self) -> ElementId {
        self.id()
    }
    fn set_effect_tree_index(&mut self, id: i32) {
        self.set_effect_tree_index(id);
    }
    fn GetPropertyChangeFlags(&self) -> RenderSurfacePropertyChangedFlags {
        self.GetPropertyChangeFlags()
    }
    fn ApplyPropertyChangeFlags(&mut self, flags: RenderSurfacePropertyChangedFlags) {
        self.ApplyPropertyChangeFlags(&flags);
    }
    fn ResetPropertyChangedFlags(&mut self) {
        self.ResetPropertyChangedFlags();
    }
}
impl<B: RenderSurfaceTypes> crate::adapter::layer_tree_impl::LayerTreeSurfaceTransform
    for RenderSurfaceImpl<B>
{
    type Transform = B::Transform;
}

struct SurfaceFullContextView<'a, B: RenderSurfaceTypes, C: RenderSurfaceContext<B> + ?Sized>(
    &'a C,
    std::marker::PhantomData<fn() -> B>,
);
impl<B: RenderSurfaceTypes, C: RenderSurfaceContext<B> + ?Sized> RenderSurfaceQueries<B>
    for SurfaceFullContextView<'_, B, C>
{
    fn EffectNode(&self, id: i32) -> &EffectNode<B::Filters, B::Path> {
        self.0.property_trees().effect_tree().Node(id)
    }
    fn HasRenderSurface(&self, id: i32) -> bool {
        self.0
            .property_trees()
            .effect_tree()
            .GetRenderSurface(id)
            .is_some()
    }
}
impl<B: RenderSurfaceTypes, C: RenderSurfaceContext<B> + ?Sized> RenderSurfaceContentClient<B>
    for SurfaceFullContextView<'_, B, C>
{
    fn LayerDrawsContent(&self, id: i32) -> bool {
        self.0.LayerDrawsContent(id)
    }
    fn LayerRenderTarget(&self, id: i32, _queries: &dyn RenderSurfaceQueries<B>) -> i32 {
        self.0.LayerRenderTarget(id)
    }
    fn LayerVisibleDrawableContentRect(&self, id: i32) -> Rect {
        self.0.LayerVisibleDrawableContentRect(id)
    }
    fn LayerViewTransitionResourceId(&self, id: i32) -> ViewTransitionElementResourceId {
        self.0.LayerViewTransitionResourceId(id)
    }
    fn enable_unbounded_element(&self) -> bool {
        self.0.settings().enable_unbounded_element
    }
    fn LogMaxTextureExceeded(&self, maximum: i32, width: i32, height: i32) {
        self.0.LogMaxTextureExceeded(maximum, width, height);
    }
}
