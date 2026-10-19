//! Live owner facets used by cc/trees/draw_property_utils. Layer identities and
//! effect ids resolve retained objects, never projections or copies of trees.
//! Setters mutate only the resident draw-property fields of those objects.
use super::{
    geometry::{Rect, RectF, Size, Vector2dF},
    input_types::ElementId,
    layer_tree_impl::{LayerTreeTypes, PropertyTreesFor},
    property_tree::{
        PropertyTreeCopyOutputRequest, PropertyTreeFeatures, PropertyTreeFilters,
        PropertyTreeRenderSurface, PropertyTreeTransform,
    },
    property_types::{MaskFilterInfo, PropertyPath, ViewTransitionElementResourceId},
};
use crate::cc::trees::{property_tree::PropertyTrees, viewport_property_ids::ViewportPropertyIds};
use std::collections::BTreeSet;
pub type ViewTransitionToken = [u64; 2];
pub trait DrawTransform: PropertyTreeTransform {
    fn InverseOrIdentity(&self) -> Self;
    fn MapRectInt(&self, rect: Rect) -> Rect;
    fn ApplyMaskFilterInfo(&self, info: &mut MaskFilterInfo);
}
pub trait DrawFilters: PropertyTreeFilters {
    fn HasReferenceFilter(&self) -> bool;
    fn HasFilterThatMovesPixels(&self) -> bool;
    fn ExpandRect(&self, rect: Rect, scale: Vector2dF) -> Rect;
    fn HasBlurFilter(&self) -> bool;
}
pub trait DrawLayer<T: DrawTransform> {
    type Types: LayerTreeTypes<Transform = T>;
    fn transform_tree_index(&self) -> i32;
    fn effect_tree_index(&self) -> i32;
    fn clip_tree_index(&self) -> i32;
    fn render_target_effect_tree_index(&self, trees: &PropertyTreesFor<Self::Types>) -> i32;
    fn render_target(&self, trees: &PropertyTreesFor<Self::Types>) -> Option<i32>;
    fn offset_to_transform_parent(&self) -> Vector2dF;
    fn draws_content(&self) -> bool;
    fn bounds(&self, trees: &PropertyTreesFor<Self::Types>) -> Size;
    fn should_check_backface_visibility(&self) -> bool;
    fn EnsureValidPropertyTreeIndices(&self, trees: &PropertyTreesFor<Self::Types>);
    fn is_clipped(&self) -> bool;
    fn clip_rect(&self) -> Rect;
    fn visible_layer_rect(&self) -> Rect;
    fn target_space_transform(&self, trees: &PropertyTreesFor<Self::Types>) -> T;
    fn contributes_to_drawn_render_surface(&self) -> bool;
    fn ViewTransitionResourceId(&self) -> ViewTransitionElementResourceId;
    fn IsViewTransitionContentLayer(&self) -> bool;
    fn DebugName(&self) -> String;
}
/// Unique, call-scoped access to the same resident LayerImpl returned by
/// `DrawContext::layer`. The source mutates these fields directly; keeping the
/// setters on `&mut self` avoids an interior-mutability owner or copied draw
/// properties while preserving the source algorithm in draw_property_utils.
pub trait DrawLayerMut<T: DrawTransform>: DrawLayer<T> {
    fn SetScreenSpaceTransform(&mut self, transform: T);
    fn SetTargetSpaceTransform(&mut self, transform: T);
    fn SetScreenSpaceTransformIsAnimating(&mut self, value: bool);
    fn SetMaskFilterInfo(&mut self, info: MaskFilterInfo, is_fast_rounded_corner: bool);
    fn ApplyMaskFilterInfoTransform(&mut self, transform: &T);
    fn SetDrawOpacity(&mut self, value: f32);
    fn SetIsClipped(&mut self, value: bool);
    fn SetClipRect(&mut self, value: Rect);
    fn SetVisibleLayerRect(&mut self, value: Rect);
    fn SetVisibleDrawableContentRect(&mut self, value: Rect);
    fn set_contributes_to_drawn_render_surface(&mut self, value: bool);
    fn set_raster_even_if_not_drawn(&mut self, value: bool);
    /// Native static_cast after the valid VT-resource check. None means this
    /// actual resident owner is not the specialized scope-external subtype.
    fn AsViewTransitionContentLayer(&mut self) -> Option<&mut dyn ViewTransitionContentDrawLayer> {
        None
    }
}
pub trait ViewTransitionContentDrawLayer {
    fn SetOriginatingSurfaceContentRect(&mut self, rect: Rect);
}
pub trait DrawSurface<T: DrawTransform>: PropertyTreeRenderSurface {
    type Types: LayerTreeTypes<Transform = T>;
    fn EffectTreeIndex(&self) -> i32;
    fn TransformTreeIndex(&self, trees: &PropertyTreesFor<Self::Types>) -> i32;
    fn render_target(&self, trees: &PropertyTreesFor<Self::Types>) -> i32;
    fn common_ancestor_clip_id(&self, trees: &PropertyTreesFor<Self::Types>) -> i32;
    fn set_common_ancestor_clip_id(&mut self, value: i32, own_clip_id: i32);
    fn has_contributing_layer_that_escapes_clip(&self) -> bool;
    fn SetIsClipped(&mut self, value: bool);
    fn is_clipped(&self) -> bool;
    fn SetDrawOpacity(&mut self, value: f32);
    fn SetScreenSpaceTransform(&mut self, value: T);
    fn screen_space_transform(&self) -> T;
    fn SetDrawTransform(&mut self, value: T, offset: Vector2dF);
    fn SetMaskFilterInfo(&mut self, value: MaskFilterInfo, is_fast_rounded_corner: bool);
    fn SetClipRect(&mut self, value: Rect);
    fn pixel_alignment_offset(&self) -> Vector2dF;
    fn is_render_surface_list_member(&self) -> bool;
    fn set_is_render_surface_list_member(&mut self, value: bool);
    fn ClearAccumulatedContentRect(&mut self);
    fn set_has_view_transition_capture_contributions(&mut self, value: bool);
    fn has_view_transition_capture_contributions(&self) -> bool;
    fn set_contributes_to_drawn_surface(&mut self, value: bool);
    fn CopyOfOutputRequired(&self, trees: &PropertyTreesFor<Self::Types>) -> bool;
    fn IsUnbounded(&self, trees: &PropertyTreesFor<Self::Types>, enabled: bool) -> bool;
    fn SetNearestOcclusionImmuneAncestor(&mut self, effect_id: Option<i32>);
    fn nearest_occlusion_immune_ancestor(&self) -> Option<i32>;
    fn reset_num_contributors(&mut self);
    fn increment_num_contributors(&mut self);
    fn decrement_num_contributors(&mut self);
    fn SetContentRectToViewport(&mut self, viewport_clip: RectF);
    fn content_rect(&self) -> Rect;
    fn view_transition_capture_content_rect(&self) -> Rect;
    fn SurfaceScale(&self, trees: &PropertyTreesFor<Self::Types>) -> T;
}
pub trait DrawContext: Sized + PropertyTreeFeatures {
    type Types: LayerTreeTypes<
        Transform = Self::Transform,
        Filters = Self::Filters,
        Path = Self::Path,
        RenderSurface = Self::Surface,
        CopyOutputRequest = Self::CopyRequest,
    >;
    type Transform: DrawTransform;
    type Filters: DrawFilters;
    type Path: PropertyPath;
    type Surface: DrawSurface<Self::Transform, Types = Self::Types>;
    type CopyRequest: PropertyTreeCopyOutputRequest;
    type Layer: Copy + Eq;
    type Layers<'a>: Iterator<Item = Self::Layer>
    where
        Self: 'a;
    fn layers(&self) -> Self::Layers<'_>;
    /// Indexed access to the same resident layer list lets mutating source
    /// loops end each read borrow before borrowing the selected layer mutably.
    fn layer_count(&self) -> usize;
    fn layer_id_at(&self, index: usize) -> Self::Layer;
    fn layer(&self, id: Self::Layer) -> &dyn DrawLayer<Self::Transform, Types = Self::Types>;
    fn layer_mut(
        &mut self,
        id: Self::Layer,
    ) -> &mut dyn DrawLayerMut<Self::Transform, Types = Self::Types>;
    fn IsRootLayer(&self, id: Self::Layer) -> bool;
    fn enable_backface_visibility_interop(&self) -> bool;
    fn viewport_property_ids(&self) -> &ViewportPropertyIds;
    fn GetCaptureViewTransitionTokens(&self) -> &BTreeSet<ViewTransitionToken>;
    fn MatchesCaptureViewTransitionToken(&self, id: ViewTransitionElementResourceId) -> bool {
        MatchesToken(id, self.GetCaptureViewTransitionTokens())
    }
    fn AllowUndamagedNonrootRenderPassToSkip(&self) -> bool;
    fn enable_unbounded_element(&self) -> bool;
    fn ViewTransitionsNewRoundingChange(&self) -> bool;
    // These owner calls perform the surface methods themselves; the draw-property
    // walk/decisions remain in the translated module. Both arguments resolve live.
    /// These three methods only split the actual property-tree nodes/surfaces
    /// from live layer/settings/services and invoke canonical RenderSurfaceImpl.
    /// All bounds/filter/deferred-content decisions stay in that source stem.
    fn AccumulateContentRectFromContributingLayer(
        &self,
        trees: &mut DrawPropertyTrees<Self>,
        surface: i32,
        layer: Self::Layer,
    );
    fn AccumulateContentRectFromContributingRenderSurface(
        &self,
        trees: &mut DrawPropertyTrees<Self>,
        surface: i32,
        contributor: i32,
    );
    fn CalculateContentRectFromAccumulatedContentRect(
        &self,
        trees: &mut DrawPropertyTrees<Self>,
        surface: i32,
        max_texture_size: i32,
    );
    fn SetViewTransitionContentRect(&mut self, id: ViewTransitionElementResourceId, rect: RectF);
    fn PageScaleTransformNodeId(&self) -> Option<i32>;
    fn current_page_scale_factor(&self) -> f32;
    fn GetDeviceViewport(&self) -> Rect;
    fn device_scale_factor(&self) -> f32;
    fn DrawTransform(&self) -> Self::Transform;
    fn external_page_scale_factor(&self) -> f32;
    fn max_texture_size(&self) -> i32;
    #[cfg(debug_assertions)]
    fn log_on_ui_double_background_blur(&self) -> bool;
    fn LayerByElementId(&self, id: ElementId) -> Option<Self::Layer>;
    fn LogDoubleBlur(&self, first: &str, second: &str);
    fn TraceCategoryEnabled(&self) -> bool;
    fn TraceRenderSurfaceReasonCount(&self, name: &str, count: usize);
    fn TraceScope(&self, name: &str, count: Option<usize>);
}
pub type DrawPropertyTrees<C> = PropertyTrees<
    <C as DrawContext>::Transform,
    <C as DrawContext>::Filters,
    <C as DrawContext>::Path,
    <C as DrawContext>::Surface,
    <C as DrawContext>::CopyRequest,
>;
// Source dependency: components/viz/common/view_transition_element_resource_id.cc:49-56.
pub fn MatchesToken(
    id: ViewTransitionElementResourceId,
    tokens: &BTreeSet<ViewTransitionToken>,
) -> bool {
    !id.for_scope_snapshot && id.transition_token.is_some_and(|t| tokens.contains(&t))
}

// Source dependency: ui/gfx/geometry/rect_f.h:219-226.
impl RectF {
    pub fn Scale(&mut self, x_scale: f32, y_scale: f32) {
        *self = Self::new(
            self.x() * x_scale,
            self.y() * y_scale,
            self.width() * x_scale,
            self.height() * y_scale,
        );
    }
    // Source dependency: ui/gfx/geometry/rect_f.cc:99-116.
    pub fn Intersect(&mut self, rect: Self) {
        if self.IsEmpty() || rect.IsEmpty() {
            *self = Self::default();
            return;
        }
        let rx = super::geometry::cpp_max(self.x(), rect.x());
        let ry = super::geometry::cpp_max(self.y(), rect.y());
        let rr = super::geometry::cpp_min(self.right(), rect.right());
        let rb = super::geometry::cpp_min(self.bottom(), rect.bottom());
        if rx >= rr || ry >= rb {
            *self = Self::default();
            return;
        }
        *self = Self::new(rx, ry, rr - rx, rb - ry);
    }
}

/// Narrow live geometry consumed by source LayerVisibleRect (cc:1648-1725).
/// Splitting this read-only facet avoids manufacturing DrawLayer's unrelated
/// mutation capabilities when a resident LayerImpl is queried during hit-test.
pub trait LayerVisibleRectLayer<T: DrawTransform> {
    type Types: LayerTreeTypes<Transform = T>;
    fn effect_tree_index(&self) -> i32;
    fn clip_tree_index(&self) -> i32;
    fn transform_tree_index(&self) -> i32;
    fn offset_to_transform_parent(&self) -> Vector2dF;
}
impl<T: DrawTransform, B: LayerTreeTypes<Transform = T>> LayerVisibleRectLayer<T>
    for dyn DrawLayer<T, Types = B> + '_
{
    type Types = B;
    fn effect_tree_index(&self) -> i32 {
        DrawLayer::effect_tree_index(self)
    }
    fn clip_tree_index(&self) -> i32 {
        DrawLayer::clip_tree_index(self)
    }
    fn transform_tree_index(&self) -> i32 {
        DrawLayer::transform_tree_index(self)
    }
    fn offset_to_transform_parent(&self) -> Vector2dF {
        DrawLayer::offset_to_transform_parent(self)
    }
}
