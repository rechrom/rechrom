//! Direct bindings of native Draw/Damage/Occlusion contracts to the one
//! canonical resident RenderSurfaceImpl. No state, tree, owner or cache is
//! retained by these implementations; tree-dependent queries borrow the exact
//! current PropertyTrees supplied by their source call site.
use crate::{
    adapter::{
        draw_property::DrawSurface,
        geometry::{Rect, RectF, Vector2dF},
        layer_tree_impl::PropertyTreesFor,
        property_types::MaskFilterInfo,
        render_surface_impl::RenderSurfaceTypes,
    },
    cc::layers::render_surface_impl::RenderSurfaceImpl,
};
impl<B: RenderSurfaceTypes> DrawSurface<B::Transform> for RenderSurfaceImpl<B> {
    type Types = B;
    fn EffectTreeIndex(&self) -> i32 {
        self.EffectTreeIndex()
    }
    fn TransformTreeIndex(&self, trees: &PropertyTreesFor<B>) -> i32 {
        self.TransformTreeIndexForPropertyTrees(trees)
    }
    fn render_target(&self, trees: &PropertyTreesFor<B>) -> i32 {
        self.render_targetForPropertyTrees(trees)
            .expect("source target surface")
    }
    fn common_ancestor_clip_id(&self, trees: &PropertyTreesFor<B>) -> i32 {
        self.CommonAncestorClipIdForPropertyTrees(trees)
    }
    fn set_common_ancestor_clip_id(&mut self, value: i32, own_clip_id: i32) {
        self.SetCommonAncestorClipIdForClip(value, own_clip_id);
    }
    fn has_contributing_layer_that_escapes_clip(&self) -> bool {
        self.has_contributing_layer_that_escapes_clip()
    }
    fn SetIsClipped(&mut self, value: bool) {
        self.SetIsClipped(value);
    }
    fn is_clipped(&self) -> bool {
        self.is_clipped()
    }
    fn SetDrawOpacity(&mut self, value: f32) {
        self.SetDrawOpacity(value);
    }
    fn SetScreenSpaceTransform(&mut self, value: B::Transform) {
        self.SetScreenSpaceTransform(&value);
    }
    fn screen_space_transform(&self) -> B::Transform {
        self.screen_space_transform().clone()
    }
    fn SetDrawTransform(&mut self, value: B::Transform, offset: Vector2dF) {
        self.SetDrawTransform(&value, offset);
    }
    fn SetMaskFilterInfo(&mut self, value: MaskFilterInfo, fast: bool) {
        self.SetMaskFilterInfo(&value, fast);
    }
    fn SetClipRect(&mut self, value: Rect) {
        self.SetClipRect(value);
    }
    fn pixel_alignment_offset(&self) -> Vector2dF {
        *self.pixel_alignment_offset()
    }
    fn is_render_surface_list_member(&self) -> bool {
        self.is_render_surface_list_member()
    }
    fn set_is_render_surface_list_member(&mut self, value: bool) {
        self.set_is_render_surface_list_member(value);
    }
    fn ClearAccumulatedContentRect(&mut self) {
        self.ClearAccumulatedContentRect();
    }
    fn set_has_view_transition_capture_contributions(&mut self, value: bool) {
        self.set_has_view_transition_capture_contributions(value);
    }
    fn has_view_transition_capture_contributions(&self) -> bool {
        self.has_view_transition_capture_contributions()
    }
    fn set_contributes_to_drawn_surface(&mut self, value: bool) {
        self.set_contributes_to_drawn_surface(value);
    }
    fn CopyOfOutputRequired(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.CopyOfOutputRequiredForPropertyTrees(trees)
    }
    fn IsUnbounded(&self, trees: &PropertyTreesFor<B>, enabled: bool) -> bool {
        self.IsUnboundedForPropertyTrees(trees, enabled)
    }
    fn SetNearestOcclusionImmuneAncestor(&mut self, effect_id: Option<i32>) {
        self.SetNearestOcclusionImmuneAncestor(effect_id);
    }
    fn nearest_occlusion_immune_ancestor(&self) -> Option<i32> {
        self.nearest_occlusion_immune_ancestor()
    }
    fn reset_num_contributors(&mut self) {
        self.reset_num_contributors();
    }
    fn increment_num_contributors(&mut self) {
        self.increment_num_contributors();
    }
    fn decrement_num_contributors(&mut self) {
        self.decrement_num_contributors();
    }
    fn SetContentRectToViewport(&mut self, viewport_clip: RectF) {
        self.SetContentRectToViewportForClip(viewport_clip);
    }
    fn content_rect(&self) -> Rect {
        self.content_rect()
    }
    fn view_transition_capture_content_rect(&self) -> Rect {
        self.view_transition_capture_content_rect()
    }
    fn SurfaceScale(&self, trees: &PropertyTreesFor<B>) -> B::Transform {
        self.SurfaceScaleForPropertyTrees(trees)
    }
}

use crate::adapter::{
    damage_context::{DamageFilters, DamageSurface},
    input_types::ElementId,
    occlusion::{OcclusionFilters, OcclusionRenderSurface, SkBlendMode, SkMatrix},
    property_types::{FilterOperations, PropertyPath, ViewTransitionElementResourceId},
};
use crate::cc::trees::effect_node::EffectNode;
impl<B: RenderSurfaceTypes> DamageSurface<B::Transform, i32> for RenderSurfaceImpl<B> {
    type Trees = PropertyTreesFor<B>;
    fn id(&self) -> ElementId {
        self.id()
    }
    fn render_target(&self, trees: &PropertyTreesFor<B>) -> i32 {
        self.render_targetForPropertyTrees(trees)
            .expect("source target surface")
    }
    fn EffectTreeIndex(&self) -> i32 {
        self.EffectTreeIndex()
    }
    fn ViewTransitionElementResourceId(
        &self,
        trees: &PropertyTreesFor<B>,
    ) -> ViewTransitionElementResourceId {
        self.OwningEffectNodeForPropertyTrees(trees)
            .expect("source owning effect")
            .view_transition_element_resource_id
    }
    fn BackdropFiltersMovePixels(&self, trees: &PropertyTreesFor<B>) -> bool {
        DamageFilters::HasFilterThatMovesPixels(self.BackdropFiltersForPropertyTrees(trees))
    }
    fn MapFiltersRect(
        &self,
        rect: Rect,
        transform: &SkMatrix,
        trees: &PropertyTreesFor<B>,
    ) -> Rect {
        DamageFilters::MapRect(self.FiltersForPropertyTrees(trees), rect, transform)
    }
    fn intersects_damage_under(&self) -> bool {
        self.intersects_damage_under()
    }
    fn set_intersects_damage_under(&self, value: bool) {
        self.set_intersects_damage_under(value);
    }
    fn SurfacePropertyChanged(&self) -> bool {
        self.SurfacePropertyChanged()
    }
    fn AncestorPropertyChanged(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.AncestorPropertyChangedForPropertyTrees(trees)
    }
    fn SurfaceScale(&self, trees: &PropertyTreesFor<B>) -> B::Transform {
        self.SurfaceScaleForPropertyTrees(trees)
    }
    fn content_rect(&self) -> Rect {
        self.content_rect()
    }
    fn DrawableContentRect(&self, trees: &PropertyTreesFor<B>) -> RectF {
        self.DrawableContentRectForPropertyTrees(trees)
    }
    fn draw_transform(&self) -> &B::Transform {
        self.draw_transform()
    }
}
impl<B: RenderSurfaceTypes> OcclusionRenderSurface<B::Transform, i32> for RenderSurfaceImpl<B>
where
    B::Filters: OcclusionFilters,
{
    type Trees = PropertyTreesFor<B>;
    fn screen_space_transform(&self) -> &B::Transform {
        self.screen_space_transform()
    }
    fn content_rect(&self) -> Rect {
        self.content_rect()
    }
    fn nearest_occlusion_immune_ancestor(&self) -> Option<i32> {
        self.nearest_occlusion_immune_ancestor()
    }
    fn render_target(&self, trees: &PropertyTreesFor<B>) -> i32 {
        self.render_targetForPropertyTrees(trees)
            .expect("source target surface")
    }
    fn screen_space_opacity(&self, trees: &PropertyTreesFor<B>) -> f32 {
        self.OwningEffectNodeForPropertyTrees(trees)
            .expect("source owning effect")
            .screen_space_opacity
    }
    fn view_transition_element_resource_id_is_valid(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.OwningEffectNodeForPropertyTrees(trees)
            .expect("source owning effect")
            .view_transition_element_resource_id
            .IsValid()
    }
    fn CopyOfOutputRequired(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.CopyOfOutputRequiredForPropertyTrees(trees)
    }
    fn HasMaskingContributingSurface(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.HasMaskingContributingSurfaceForPropertyTrees(trees)
    }
    fn draw_opacity(&self) -> f32 {
        self.draw_opacity()
    }
    fn BlendMode(&self, trees: &PropertyTreesFor<B>) -> SkBlendMode {
        self.BlendModeForPropertyTrees(trees)
    }
    fn FiltersAffectOpacity(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.FiltersForPropertyTrees(trees)
            .HasFilterThatAffectsOpacity()
    }
    fn BackdropFiltersMovePixels(&self, trees: &PropertyTreesFor<B>) -> bool {
        OcclusionFilters::HasFilterThatMovesPixels(self.BackdropFiltersForPropertyTrees(trees))
    }
    fn MapBackdropFilterRectReverse(
        &self,
        rect: Rect,
        transform: &SkMatrix,
        trees: &PropertyTreesFor<B>,
    ) -> Rect {
        self.BackdropFiltersForPropertyTrees(trees)
            .MapRectReverse(rect, transform)
    }
    fn is_clipped(&self) -> bool {
        self.is_clipped()
    }
    fn clip_rect(&self) -> Rect {
        self.clip_rect()
    }
    fn draw_transform(&self) -> &B::Transform {
        self.draw_transform()
    }
}

impl<B: RenderSurfaceTypes> crate::adapter::render_surface_impl::RenderSurfaceQueries<B>
    for PropertyTreesFor<B>
{
    fn EffectNode(&self, id: i32) -> &EffectNode<B::Filters, B::Path> {
        self.effect_tree().Node(id)
    }
    fn HasRenderSurface(&self, id: i32) -> bool {
        self.effect_tree().GetRenderSurface(id).is_some()
    }
}

use crate::adapter::render_surface_impl::{RenderSurfaceContentClient, RenderSurfaceQueries};
use crate::cc::trees::property_tree::EffectTreeSurfaceQueries;
impl<B: RenderSurfaceTypes> RenderSurfaceQueries<B>
    for EffectTreeSurfaceQueries<'_, B::Filters, B::Path, B::RenderSurface>
{
    fn EffectNode(&self, id: i32) -> &EffectNode<B::Filters, B::Path> {
        self.Node(id)
    }
    fn HasRenderSurface(&self, id: i32) -> bool {
        self.HasRenderSurface(id)
    }
}
struct ContentContext<
    'a,
    B: RenderSurfaceTypes,
    Q: RenderSurfaceQueries<B>,
    C: RenderSurfaceContentClient<B>,
> {
    queries: &'a Q,
    client: &'a C,
    marker: std::marker::PhantomData<fn() -> B>,
}
impl<B: RenderSurfaceTypes, Q: RenderSurfaceQueries<B>, C: RenderSurfaceContentClient<B>>
    RenderSurfaceQueries<B> for ContentContext<'_, B, Q, C>
{
    fn EffectNode(&self, id: i32) -> &EffectNode<B::Filters, B::Path> {
        self.queries.EffectNode(id)
    }
    fn HasRenderSurface(&self, id: i32) -> bool {
        self.queries.HasRenderSurface(id)
    }
}
impl<B: RenderSurfaceTypes, Q: RenderSurfaceQueries<B>, C: RenderSurfaceContentClient<B>>
    RenderSurfaceContentClient<B> for ContentContext<'_, B, Q, C>
{
    fn LayerDrawsContent(&self, id: i32) -> bool {
        self.client.LayerDrawsContent(id)
    }
    fn LayerRenderTarget(&self, id: i32, queries: &dyn RenderSurfaceQueries<B>) -> i32 {
        self.client.LayerRenderTarget(id, queries)
    }
    fn LayerVisibleDrawableContentRect(&self, id: i32) -> Rect {
        self.client.LayerVisibleDrawableContentRect(id)
    }
    fn LayerViewTransitionResourceId(&self, id: i32) -> ViewTransitionElementResourceId {
        self.client.LayerViewTransitionResourceId(id)
    }
    fn enable_unbounded_element(&self) -> bool {
        self.client.enable_unbounded_element()
    }
    fn LogMaxTextureExceeded(&self, maximum: i32, width: i32, height: i32) {
        self.client.LogMaxTextureExceeded(maximum, width, height);
    }
}
/// These are pure call-scoped borrow builders. The selected real Box remains
/// resident in its original slot; all policy executes in the canonical source.
pub fn CalculateContentRect<B, C>(
    trees: &mut PropertyTreesFor<B>,
    id: i32,
    maximum: i32,
    client: &C,
) where
    B: RenderSurfaceTypes<RenderSurface = RenderSurfaceImpl<B>>,
    C: RenderSurfaceContentClient<B>,
{
    let (surface, queries) = trees
        .effect_tree_mutable()
        .GetRenderSurfaceMutableWithQueries(id)
        .expect("source resident surface");
    assert_eq!(
        surface.EffectTreeIndex(),
        id,
        "reserved identity must name this actual surface"
    );
    let context = ContentContext {
        queries: &queries,
        client,
        marker: std::marker::PhantomData,
    };
    surface.CalculateContentRectFromAccumulatedContentRectWithContentContext(maximum, &context);
}
pub fn AccumulateLayer<B, C>(
    trees: &mut PropertyTreesFor<B>,
    id: i32,
    layer: i32,
    tokens: &std::collections::BTreeSet<crate::adapter::draw_property::ViewTransitionToken>,
    client: &C,
) where
    B: RenderSurfaceTypes<RenderSurface = RenderSurfaceImpl<B>>,
    C: RenderSurfaceContentClient<B>,
{
    let (surface, queries) = trees
        .effect_tree_mutable()
        .GetRenderSurfaceMutableWithQueries(id)
        .expect("source resident surface");
    assert_eq!(
        surface.EffectTreeIndex(),
        id,
        "reserved identity must name this actual surface"
    );
    let context = ContentContext {
        queries: &queries,
        client,
        marker: std::marker::PhantomData,
    };
    surface.AccumulateContentRectFromContributingLayerWithContentContext(layer, tokens, &context);
}
pub fn AccumulateSurface<B, C>(
    trees: &mut PropertyTreesFor<B>,
    id: i32,
    contributor_id: i32,
    tokens: &std::collections::BTreeSet<crate::adapter::draw_property::ViewTransitionToken>,
    client: &C,
) where
    B: RenderSurfaceTypes<RenderSurface = RenderSurfaceImpl<B>>,
    C: RenderSurfaceContentClient<B>,
{
    let (surface, contributor, queries) = trees
        .effect_tree_mutable()
        .GetRenderSurfaceMutableAndOtherWithQueries(id, contributor_id)
        .expect("source resident target and contributor");
    assert_eq!(
        surface.EffectTreeIndex(),
        id,
        "reserved identity must name actual target"
    );
    assert_eq!(
        contributor.EffectTreeIndex(),
        contributor_id,
        "reserved identity must name actual contributor"
    );
    let context = ContentContext {
        queries: &queries,
        client,
        marker: std::marker::PhantomData,
    };
    surface.AccumulateContentRectFromContributingRenderSurfaceWithContentContext(
        contributor,
        tokens,
        &context,
    );
}

// Source: cc/layers/render_surface_impl.h:188,261; cc/trees/layer_tree_impl.cc:1176-1181.
impl<B: RenderSurfaceTypes> crate::adapter::layer_tree_impl::LayerTreeRenderSurface<B>
    for RenderSurfaceImpl<B>
{
    fn SetOcclusionInContentSpace(
        &mut self,
        occlusion: crate::cc::trees::occlusion::Occlusion<B::Transform>,
    ) {
        self.set_occlusion_in_content_space(&occlusion);
    }
    fn AncestorPropertyChanged(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.AncestorPropertyChangedForPropertyTrees(trees)
    }
    fn NoteAncestorPropertyChanged(&mut self) {
        RenderSurfaceImpl::NoteAncestorPropertyChanged(self);
    }
}

/// A call-scoped render-pass/quad context over the actual native layer list and
/// property trees. Only its independent output/trace service is mutable.
pub struct RenderPassContext<'a, 's, B: RenderSurfaceTypes> {
    pub tree_id: crate::adapter::layer_tree_impl::LayerTreeId,
    pub trees: &'a PropertyTreesFor<B>,
    pub layers: &'a B::LayerList,
    pub settings: &'a crate::cc::trees::layer_tree_settings::LayerTreeSettings,
    pub debug: &'a crate::cc::debug::layer_tree_debug_state::LayerTreeDebugState,
    pub scale: f32,
    pub services: &'s mut dyn crate::adapter::render_surface_impl::RenderSurfaceServices<B>,
}
impl<B> crate::adapter::render_surface_impl::RenderSurfaceContext<B>
    for RenderPassContext<'_, '_, B>
where
    B: RenderSurfaceTypes,
    B::Layer: crate::adapter::render_surface_impl::RenderSurfaceLayer<B>,
{
    fn tree_id(&self) -> crate::adapter::layer_tree_impl::LayerTreeId {
        self.tree_id
    }
    fn property_trees(&self) -> &PropertyTreesFor<B> {
        self.trees
    }
    fn settings(&self) -> &crate::cc::trees::layer_tree_settings::LayerTreeSettings {
        self.settings
    }
    fn debug_state(&self) -> &crate::cc::debug::layer_tree_debug_state::LayerTreeDebugState {
        self.debug
    }
    fn device_scale_factor(&self) -> f32 {
        self.scale
    }
    fn NextStableIdSequenceValue(&self) -> i32 {
        self.services.NextStableIdSequenceValue()
    }
    fn LayerByElementId(&self, element: ElementId) -> Option<i32> {
        use crate::adapter::layer_tree_impl::{LayerTreeLayer, LayerTreeLayerList};
        self.layers
            .LayerByElementId(element)
            .map(|layer| layer.id())
    }
    fn LayerDrawsContent(&self, layer: i32) -> bool {
        use crate::adapter::layer_tree_impl::{LayerTreeLayer, LayerTreeLayerList};
        crate::adapter::layer_impl::LayerImplOwner::impl_base(
            self.layers.LayerById(layer).expect("source resident layer"),
        )
        .draws_content()
    }
    fn LayerBounds(&self, layer: i32) -> crate::adapter::geometry::Size {
        use crate::adapter::layer_tree_impl::{LayerTreeLayer, LayerTreeLayerList};
        self.layers
            .LayerById(layer)
            .expect("source resident layer")
            .bounds(self.trees)
    }
    fn LayerRenderTarget(&self, layer: i32) -> i32 {
        use crate::adapter::layer_tree_impl::{LayerTreeLayer, LayerTreeLayerList};
        self.layers
            .LayerById(layer)
            .expect("source resident layer")
            .render_target_effect_tree_index(self.trees)
    }
    fn LayerVisibleDrawableContentRect(&self, layer: i32) -> Rect {
        use crate::adapter::{layer_impl::LayerImplOwner, layer_tree_impl::LayerTreeLayerList};
        self.layers
            .LayerById(layer)
            .expect("source resident layer")
            .impl_base()
            .visible_drawable_content_rect()
    }
    fn LayerViewTransitionResourceId(
        &self,
        layer: i32,
    ) -> crate::adapter::property_types::ViewTransitionElementResourceId {
        use crate::adapter::{layer_impl::LayerImplOwner, layer_tree_impl::LayerTreeLayerList};
        self.layers
            .LayerById(layer)
            .expect("source resident layer")
            .impl_base()
            .ViewTransitionResourceId()
    }
    fn LayerGpuMemoryUsageInBytes(&self, layer: i32) -> usize {
        use crate::adapter::{
            layer_tree_impl::LayerTreeLayerList, render_surface_impl::RenderSurfaceLayer,
        };
        self.layers
            .LayerById(layer)
            .expect("source resident layer")
            .GpuMemoryUsageInBytes()
    }
    fn LayerGetContentsResourceId(
        &self,
        layer: i32,
        resource: &mut crate::adapter::layer_tree_impl::ResourceId,
        size: &mut crate::adapter::geometry::Size,
        uv: &mut crate::adapter::geometry::SizeF,
    ) {
        use crate::adapter::{
            layer_tree_impl::LayerTreeLayerList, render_surface_impl::RenderSurfaceLayer,
        };
        self.layers
            .LayerById(layer)
            .expect("source resident layer")
            .GetContentsResourceId(resource, size, uv)
    }
    fn LayerOffsetToTransformParent(&self, layer: i32) -> crate::adapter::geometry::Vector2dF {
        use crate::adapter::{layer_impl::LayerImplOwner, layer_tree_impl::LayerTreeLayerList};
        self.layers
            .LayerById(layer)
            .expect("source resident layer")
            .impl_base()
            .offset_to_transform_parent()
    }
    fn TraceMaskLayerGpuMemoryUsage(&self, bytes: usize) {
        self.services.TraceMaskLayerGpuMemoryUsage(bytes)
    }
    fn LogMaxTextureExceeded(&self, maximum: i32, width: i32, height: i32) {
        self.services.LogMaxTextureExceeded(maximum, width, height)
    }
    fn CreateRenderPass(&mut self, contributors: i32) -> Box<B::RenderPass> {
        self.services.CreateRenderPass(contributors)
    }
}

impl<B> crate::adapter::render_surface_impl::RenderSurfaceLayer<B>
    for super::layer::CompositorLayer<B>
where
    B: crate::adapter::picture_layer_impl::PictureLayerImplTypes
        + crate::adapter::solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImplTypes
        + RenderSurfaceTypes,
{
    fn GpuMemoryUsageInBytes(&self) -> usize {
        use crate::adapter::layer_impl::LayerImplOwner;
        match self {
            Self::Picture(layer) => layer.GPUMemoryUsageInBytes(),
            _ => self.impl_base().GPUMemoryUsageInBytes(),
        }
    }
    fn GetContentsResourceId(
        &self,
        resource: &mut crate::adapter::layer_tree_impl::ResourceId,
        size: &mut crate::adapter::geometry::Size,
        uv: &mut crate::adapter::geometry::SizeF,
    ) {
        use crate::adapter::layer_impl::LayerImplOwner;
        // These native subtypes inherit the source base NOTREACHED method.
        // No CPU subtype pretends to have an unsupported texture/mask resource.
        self.impl_base().GetContentsResourceId(resource, size, uv)
    }
}
impl<B> crate::adapter::layer_tree_host_impl::HostDrawSurface<B> for RenderSurfaceImpl<B>
where
    B: crate::adapter::layer_tree_host_impl::HostImplTypes,
{
    fn GetDamageRect(&self) -> Rect {
        RenderSurfaceImpl::GetDamageRect(self)
    }
    fn contributes_to_drawn_surface(&self) -> bool {
        RenderSurfaceImpl::contributes_to_drawn_surface(self)
    }
    fn ViewTransitionElementResourceId(
        &self,
        trees: &PropertyTreesFor<B>,
    ) -> crate::adapter::property_types::ViewTransitionElementResourceId {
        *self.ViewTransitionElementResourceIdForPropertyTrees(trees)
    }
    fn CreateViewTransitionCaptureRenderPass(
        &self,
        tokens: &std::collections::BTreeSet<crate::adapter::draw_property::ViewTransitionToken>,
        c: &mut dyn crate::adapter::render_surface_impl::RenderSurfaceContext<B>,
    ) -> Box<B::RenderPass> {
        RenderSurfaceImpl::CreateViewTransitionCaptureRenderPass(self, tokens, c)
    }
    fn CreateRenderPass(
        &self,
        tokens: &std::collections::BTreeSet<crate::adapter::draw_property::ViewTransitionToken>,
        c: &mut dyn crate::adapter::render_surface_impl::RenderSurfaceContext<B>,
    ) -> Box<B::RenderPass> {
        RenderSurfaceImpl::CreateRenderPass(self, tokens, c)
    }
    fn render_pass_id(&self) -> B::RenderPassId {
        RenderSurfaceImpl::render_pass_id(self)
    }
    fn view_transition_capture_render_pass_id(&self) -> B::RenderPassId {
        RenderSurfaceImpl::view_transition_capture_render_pass_id(self)
    }
    fn HasCopyRequest(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.HasCopyRequestForPropertyTrees(trees)
    }
    fn AppendQuads(
        &self,
        c: &crate::cc::layers::append_quads_context::AppendQuadsContext,
        target: &mut B::RenderPass,
        data: &mut crate::cc::layers::append_quads_data::AppendQuadsData<B::SurfaceId>,
        surface_context: &dyn crate::adapter::render_surface_impl::RenderSurfaceContext<B>,
    ) {
        RenderSurfaceImpl::AppendQuads(self, c, target, data, surface_context)
    }
    fn draw_transform(&self) -> &<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::Transform {
        RenderSurfaceImpl::draw_transform(self)
    }
}

impl<B: RenderSurfaceTypes> crate::adapter::layer_tree_host_impl::HostScrollRenderSurface<B>
    for RenderSurfaceImpl<B>
{
    fn DrawableContentRect(&self, trees: &PropertyTreesFor<B>) -> RectF {
        self.DrawableContentRectForPropertyTrees(trees)
    }
}
