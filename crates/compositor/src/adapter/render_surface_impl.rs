//! Borrowed resident siblings and external Viz/gfx values used by RenderSurfaceImpl.
//! All compositor decisions remain in the translated owner. No tree/host or
//! layer projection is retained between calls, and queries read the actual owner.
use super::{
    damage_context::{DamageFilters, DamageTransform},
    draw_property::DrawFilters,
    geometry::{PointF, Rect, RectF, Size, SizeF, Vector2dF},
    input_types::ElementId,
    layer_tree_impl::{LayerTreeId, PropertyTreesFor, ResourceId},
    occlusion::SkMatrix,
    property_types::{SubtreeCaptureId, ViewTransitionElementResourceId},
};
use crate::cc::{
    debug::layer_tree_debug_state::LayerTreeDebugState,
    trees::layer_tree_settings::LayerTreeSettings,
};
pub trait RenderSurfaceTypes:
    super::solid_color_layer_impl::SolidColorLayerImplTypes<
    Transform: DamageTransform,
    Filters: RenderSurfaceFilters,
    RenderPass: RenderSurfaceRenderPass<Self>,
>
{
}
pub trait RenderSurfaceFilters: DrawFilters + DamageFilters {
    /// Native FilterOperations::ExpandRect(rect, flattened SkMatrix).
    fn ExpandRectWithMatrix(&self, rect: Rect, matrix: &SkMatrix) -> Rect;
}
pub trait RenderSurfaceContext<B: RenderSurfaceTypes> {
    fn tree_id(&self) -> LayerTreeId;
    fn property_trees(&self) -> &PropertyTreesFor<B>;
    fn settings(&self) -> &LayerTreeSettings;
    fn debug_state(&self) -> &LayerTreeDebugState;
    fn device_scale_factor(&self) -> f32;
    fn NextStableIdSequenceValue(&self) -> i32;
    /// Native LayerTreeImpl::LayerByElementId, returning only that live layer's
    /// native id. All following methods resolve the same resident instance.
    fn LayerByElementId(&self, element: ElementId) -> Option<i32>;
    fn LayerDrawsContent(&self, layer: i32) -> bool;
    fn LayerBounds(&self, layer: i32) -> Size;
    fn LayerRenderTarget(&self, layer: i32) -> i32;
    fn LayerVisibleDrawableContentRect(&self, layer: i32) -> Rect;
    fn LayerViewTransitionResourceId(&self, layer: i32) -> ViewTransitionElementResourceId;
    fn LayerGpuMemoryUsageInBytes(&self, layer: i32) -> usize;
    fn LayerGetContentsResourceId(
        &self,
        layer: i32,
        resource: &mut ResourceId,
        size: &mut Size,
        uv: &mut SizeF,
    );
    fn LayerOffsetToTransformParent(&self, layer: i32) -> Vector2dF;
    // External trace/logger/output allocation only. No surface policy.
    fn TraceMaskLayerGpuMemoryUsage(&self, bytes: usize);
    fn LogMaxTextureExceeded(&self, maximum: i32, width: i32, height: i32);
    fn CreateRenderPass(&mut self, num_contributors: i32) -> Box<B::RenderPass>;
}
/// Only the source test-only mutable effect-node escape hatch needs writes.
/// Production render-pass/quad creation borrows the resident trees read-only.
pub trait RenderSurfaceMutableContext<B: RenderSurfaceTypes>: RenderSurfaceContext<B> {
    fn property_trees_mut(&mut self) -> &mut PropertyTreesFor<B>;
}
/// External output allocation/trace primitives, independent of any tree/host.
pub trait RenderSurfaceServices<B: RenderSurfaceTypes> {
    fn NextStableIdSequenceValue(&self) -> i32;
    fn TraceMaskLayerGpuMemoryUsage(&self, bytes: usize);
    fn LogMaxTextureExceeded(&self, maximum: i32, width: i32, height: i32);
    fn CreateRenderPass(&mut self, num_contributors: i32) -> Box<B::RenderPass>;
}
/// Source virtual layer queries on the actual resident most-derived owner.
pub trait RenderSurfaceLayer<B: RenderSurfaceTypes> {
    fn GpuMemoryUsageInBytes(&self) -> usize;
    fn GetContentsResourceId(&self, resource: &mut ResourceId, size: &mut Size, uv: &mut SizeF);
}
pub trait RenderSurfaceRenderPass<B: RenderSurfaceTypes>:
    super::solid_color_layer_impl::SolidColorRenderPass<B>
{
    fn SetNew(&mut self, id: B::RenderPassId, output: Rect, damage: Rect, transform: &B::Transform);
    fn SetFilters(&mut self, value: &B::Filters);
    fn SetBackdropFilters(&mut self, value: &B::Filters);
    fn SetBackdropFilterBounds(&mut self, value: Option<B::Path>);
    fn set_generate_mipmap(&mut self, value: bool);
    fn set_subtree_size(&mut self, value: Size);
    fn subtree_size_mut(&mut self) -> &mut Size;
    fn set_has_damage_from_contributing_content(&mut self, value: bool);
    fn set_subtree_capture_id(&mut self, value: SubtreeCaptureId);
    fn set_cache_render_pass(&mut self, value: bool);
    fn set_view_transition_element_resource_id(&mut self, value: ViewTransitionElementResourceId);
    fn CreateAndAppendRenderPassDrawQuad(&mut self) -> B::DrawQuadHandle;
    fn RenderPassDrawQuadSetAll(
        &mut self,
        quad: B::DrawQuadHandle,
        shared: B::SharedQuadStateHandle,
        rect: Rect,
        visible: Rect,
        needs_blending: bool,
        pass: B::RenderPassId,
        mask: ResourceId,
        mask_uv: RectF,
        mask_texture: Size,
        filters_scale: Vector2dF,
        filters_origin: PointF,
        force_anti_aliasing_off: bool,
        backdrop_quality: f32,
        intersects_damage_under: bool,
    );
}

/// Read-only resident effect nodes and surface membership. This is the exact
/// subset needed while a surface's own mutable fields are exclusively borrowed.
/// It cannot return surface fields or mutate the effect-tree structure.
pub trait RenderSurfaceQueries<B: RenderSurfaceTypes> {
    fn EffectNode(
        &self,
        id: i32,
    ) -> &crate::cc::trees::effect_node::EffectNode<B::Filters, B::Path>;
    fn HasRenderSurface(&self, id: i32) -> bool;
}
/// Live layer/settings/logging input for the content-rectangle source methods.
/// Implementations only resolve actual resident layers and call their canonical
/// getters; the effect query parameter excludes the currently borrowed surface.
pub trait RenderSurfaceContentClient<B: RenderSurfaceTypes> {
    fn LayerDrawsContent(&self, layer: i32) -> bool;
    fn LayerRenderTarget(&self, layer: i32, queries: &dyn RenderSurfaceQueries<B>) -> i32;
    fn LayerVisibleDrawableContentRect(&self, layer: i32) -> Rect;
    fn LayerViewTransitionResourceId(&self, layer: i32) -> ViewTransitionElementResourceId;
    fn enable_unbounded_element(&self) -> bool;
    fn LogMaxTextureExceeded(&self, maximum: i32, width: i32, height: i32);
}
pub trait RenderSurfaceContentContext<B: RenderSurfaceTypes>:
    RenderSurfaceQueries<B> + RenderSurfaceContentClient<B>
{
}
impl<B: RenderSurfaceTypes, C: RenderSurfaceQueries<B> + RenderSurfaceContentClient<B>>
    RenderSurfaceContentContext<B> for C
{
}
