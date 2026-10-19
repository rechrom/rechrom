//! Concrete, call-scoped native draw/damage/occlusion contexts. Each reference
//! borrows the actual owner storage only for the source walk. No context is
//! stored in a layer or surface, and no resident graph/state is copied.
use crate::{
    adapter::{
        damage_context::{DamageContext as NativeDamageContext, DamageLayer, DamageSurface},
        draw_property::{
            DrawContext as NativeDrawContext, DrawLayer, DrawLayerMut, DrawPropertyTrees,
            ViewTransitionToken,
        },
        geometry::{Rect, RectF},
        input_types::ElementId,
        layer_impl::LayerImplOwner,
        layer_tree_impl::{
            draw_host::LayerTreeDrawHost, LayerTreeDrawState, LayerTreeHost, LayerTreeLayer,
            LayerTreeLayerList, LayerTreeTypes, PropertyTreesFor,
        },
        occlusion::{
            OcclusionContext as NativeOcclusionContext, OcclusionLayer, OcclusionLayerTree,
            OcclusionRenderSurface,
        },
        property_tree::PropertyTreeFeatures,
        property_types::ViewTransitionElementResourceId,
        render_surface_impl::{
            RenderSurfaceContentClient, RenderSurfaceQueries, RenderSurfaceTypes,
        },
    },
    cc::{
        layers::{layer_impl::LayerImpl, render_surface_impl::RenderSurfaceImpl},
        trees::{
            damage_tracker::DamageTracker, layer_tree_impl::LayerTreeImpl,
            layer_tree_settings::LayerTreeSettings, property_ids::kInvalidPropertyNodeId,
            viewport_property_ids::ViewportPropertyIds,
        },
    },
};
use std::collections::BTreeSet;
fn layer_id<B: LayerTreeTypes>(layer: &B::Layer) -> i32 {
    layer.id()
}
pub type LayerIds<'a, B> = std::iter::Map<
    <<B as LayerTreeTypes>::LayerList as LayerTreeLayerList<B>>::Iter<'a>,
    fn(&<B as LayerTreeTypes>::Layer) -> i32,
>;

pub struct DrawContext<'a, B: RenderSurfaceTypes> {
    state: LayerTreeDrawState<'a, B>,
    host: &'a mut dyn LayerTreeDrawHost<B>,
}
impl<'a, B: RenderSurfaceTypes> DrawContext<'a, B> {
    pub fn new(state: LayerTreeDrawState<'a, B>, host: &'a mut dyn LayerTreeDrawHost<B>) -> Self {
        Self { state, host }
    }
}
impl<B: RenderSurfaceTypes> PropertyTreeFeatures for DrawContext<'_, B> {
    fn SizeOopifEffectSurfacesAtExternalScale(&self) -> bool {
        self.host
            .DrawServices()
            .SizeOopifEffectSurfacesAtExternalScale()
    }
}
impl<B> RenderSurfaceContentClient<B> for DrawContext<'_, B>
where
    B: RenderSurfaceTypes<Settings = LayerTreeSettings>,
    B::Layer: DrawLayer<B::Transform, Types = B>,
{
    fn LayerDrawsContent(&self, id: i32) -> bool {
        DrawLayer::draws_content(self.state.layers.LayerById(id).expect("source layer"))
    }
    fn LayerRenderTarget(&self, id: i32, queries: &dyn RenderSurfaceQueries<B>) -> i32 {
        let base = self
            .state
            .layers
            .LayerById(id)
            .expect("source layer")
            .impl_base();
        let node = queries.EffectNode(base.effect_tree_index());
        let has_surface = queries.HasRenderSurface(base.effect_tree_index());
        LayerImpl::<B>::render_target_effect_tree_index_for_effect_node(node, has_surface)
    }
    fn LayerVisibleDrawableContentRect(&self, id: i32) -> Rect {
        self.state
            .layers
            .LayerById(id)
            .expect("source layer")
            .impl_base()
            .visible_drawable_content_rect()
    }
    fn LayerViewTransitionResourceId(&self, id: i32) -> ViewTransitionElementResourceId {
        DrawLayer::ViewTransitionResourceId(self.state.layers.LayerById(id).expect("source layer"))
    }
    fn enable_unbounded_element(&self) -> bool {
        self.host.settings().enable_unbounded_element
    }
    fn LogMaxTextureExceeded(&self, maximum: i32, width: i32, height: i32) {
        self.host
            .DrawServices()
            .LogMaxTextureExceeded(maximum, width, height);
    }
}
impl<B> NativeDrawContext for DrawContext<'_, B>
where
    B: RenderSurfaceTypes<RenderSurface = RenderSurfaceImpl<B>, Settings = LayerTreeSettings>,
    B::Layer: DrawLayerMut<B::Transform, Types = B>,
{
    type Types = B;
    type Transform = B::Transform;
    type Filters = B::Filters;
    type Path = B::Path;
    type Surface = RenderSurfaceImpl<B>;
    type CopyRequest = B::CopyOutputRequest;
    type Layer = i32;
    type Layers<'a>
        = LayerIds<'a, B>
    where
        Self: 'a;
    fn layers(&self) -> Self::Layers<'_> {
        self.state.layers.iter().map(layer_id::<B>)
    }
    fn layer_count(&self) -> usize {
        self.state.layers.len()
    }
    fn layer_id_at(&self, index: usize) -> i32 {
        self.state.layers.LayerAt(index).id()
    }
    fn layer(&self, id: i32) -> &dyn DrawLayer<B::Transform, Types = B> {
        self.state
            .layers
            .LayerById(id)
            .expect("source live layer id")
    }
    fn layer_mut(&mut self, id: i32) -> &mut dyn DrawLayerMut<B::Transform, Types = B> {
        self.state
            .layers
            .LayerByIdMut(id)
            .expect("source live layer id")
    }
    fn IsRootLayer(&self, id: i32) -> bool {
        let layer = self
            .state
            .layers
            .LayerById(id)
            .expect("source live layer id");
        self.state
            .layers
            .iter()
            .next()
            .is_some_and(|root| std::ptr::eq(root, layer))
    }
    fn enable_backface_visibility_interop(&self) -> bool {
        self.host.settings().enable_backface_visibility_interop
    }
    fn viewport_property_ids(&self) -> &ViewportPropertyIds {
        self.state.viewport
    }
    fn GetCaptureViewTransitionTokens(&self) -> &BTreeSet<ViewTransitionToken> {
        self.state.capture_tokens
    }
    fn AllowUndamagedNonrootRenderPassToSkip(&self) -> bool {
        self.host
            .DrawServices()
            .AllowUndamagedNonrootRenderPassToSkip()
    }
    fn enable_unbounded_element(&self) -> bool {
        self.host.settings().enable_unbounded_element
    }
    fn ViewTransitionsNewRoundingChange(&self) -> bool {
        self.host.DrawServices().ViewTransitionsNewRoundingChange()
    }
    fn AccumulateContentRectFromContributingLayer(
        &self,
        trees: &mut DrawPropertyTrees<Self>,
        surface: i32,
        layer: i32,
    ) {
        super::surface::AccumulateLayer::<B, _>(
            trees,
            surface,
            layer,
            self.state.capture_tokens,
            self,
        );
    }
    fn AccumulateContentRectFromContributingRenderSurface(
        &self,
        trees: &mut DrawPropertyTrees<Self>,
        surface: i32,
        contributor: i32,
    ) {
        super::surface::AccumulateSurface::<B, _>(
            trees,
            surface,
            contributor,
            self.state.capture_tokens,
            self,
        );
    }
    fn CalculateContentRectFromAccumulatedContentRect(
        &self,
        trees: &mut DrawPropertyTrees<Self>,
        surface: i32,
        maximum: i32,
    ) {
        super::surface::CalculateContentRect::<B, _>(trees, surface, maximum, self);
    }
    fn SetViewTransitionContentRect(&mut self, id: ViewTransitionElementResourceId, rect: RectF) {
        LayerTreeImpl::<B>::SetViewTransitionContentRectForRequests(
            self.state.view_transition_requests,
            &id,
            &rect,
            self.host,
        );
    }
    fn PageScaleTransformNodeId(&self) -> Option<i32> {
        let id = self.state.viewport.page_scale_transform;
        (id != kInvalidPropertyNodeId).then_some(id)
    }
    fn current_page_scale_factor(&self) -> f32 {
        self.state
            .page_scale
            .borrow()
            .Current(self.host.IsActiveTree(self.state.tree_id))
    }
    fn GetDeviceViewport(&self) -> Rect {
        LayerTreeImpl::<B>::GetDeviceViewportForRects(
            *self.state.device_viewport,
            self.host.external_viewport(),
        )
    }
    fn device_scale_factor(&self) -> f32 {
        *self.state.device_scale
    }
    fn DrawTransform(&self) -> B::Transform {
        self.host.DrawTransform().clone()
    }
    fn external_page_scale_factor(&self) -> f32 {
        *self.state.external_scale
    }
    fn max_texture_size(&self) -> i32 {
        self.host.max_texture_size()
    }
    #[cfg(debug_assertions)]
    fn log_on_ui_double_background_blur(&self) -> bool {
        self.host.settings().log_on_ui_double_background_blur
    }
    fn LayerByElementId(&self, id: ElementId) -> Option<i32> {
        self.state.layers.LayerByElementId(id).map(layer_id::<B>)
    }
    fn LogDoubleBlur(&self, first: &str, second: &str) {
        self.host.DrawServices().LogDoubleBlur(first, second);
    }
    fn TraceCategoryEnabled(&self) -> bool {
        self.host.DrawServices().TraceCategoryEnabled()
    }
    fn TraceRenderSurfaceReasonCount(&self, name: &str, count: usize) {
        self.host
            .DrawServices()
            .TraceRenderSurfaceReasonCount(name, count);
    }
    fn TraceScope(&self, name: &str, count: Option<usize>) {
        self.host.DrawServices().TraceScope(name, count);
    }
}

pub struct DamageContext<'a, B: RenderSurfaceTypes> {
    tree: &'a LayerTreeImpl<B>,
}
impl<'a, B: RenderSurfaceTypes> DamageContext<'a, B> {
    pub fn new(tree: &'a LayerTreeImpl<B>, _host: &'a dyn LayerTreeHost<B>) -> Self {
        Self { tree }
    }
}
impl<B> NativeDamageContext for DamageContext<'_, B>
where
    B: RenderSurfaceTypes<RenderSurface = RenderSurfaceImpl<B>>
        + super::frame::ViewTransitionGeometry<FrameTransform = B::Transform>,
    B::Layer: DamageLayer<B::Transform, i32, Types = B>,
{
    type Types = B;
    type Transform = B::Transform;
    type Surface = i32;
    type Layer = i32;
    type LayerIterator<'a>
        = LayerIds<'a, B>
    where
        Self: 'a;
    type SurfaceIterator<'a>
        = std::iter::Copied<std::slice::Iter<'a, i32>>
    where
        Self: 'a;
    fn layers(&self) -> Self::LayerIterator<'_> {
        self.tree.begin().map(layer_id::<B>)
    }
    fn property_trees(&self) -> &PropertyTreesFor<B> {
        self.tree.property_trees()
    }
    fn hud_layer(&self) -> Option<i32> {
        self.tree.hud_layer().map(layer_id::<B>)
    }
    fn GetRenderSurfaceList(&self) -> Self::SurfaceIterator<'_> {
        self.tree.GetRenderSurfaceList().iter().copied()
    }
    fn GetRenderSurface(&self, effect: i32) -> Option<i32> {
        self.tree.GetRenderSurface(effect).map(|_| effect)
    }
    fn layer(&self, id: i32) -> &dyn DamageLayer<B::Transform, i32, Types = B> {
        self.tree.LayerById(id).expect("source live layer id")
    }
    fn surface(&self, id: i32) -> &dyn DamageSurface<B::Transform, i32, Types = B> {
        self.tree
            .GetRenderSurface(id)
            .expect("source live surface id")
    }
    fn damage_tracker(&self, id: i32) -> &DamageTracker<i32> {
        self.tree
            .GetRenderSurface(id)
            .expect("source live surface id")
            .damage_tracker()
    }
    fn GetViewTransitionTransform(&self, shared: Rect, output: Rect) -> B::Transform {
        B::GetViewTransitionTransform(shared, output)
    }
}

pub struct OcclusionContext<'a, B: RenderSurfaceTypes> {
    layers: &'a B::LayerList,
    trees: &'a PropertyTreesFor<B>,
}
impl<'a, B: RenderSurfaceTypes> OcclusionContext<'a, B> {
    pub fn new(
        layers: &'a B::LayerList,
        trees: &'a PropertyTreesFor<B>,
        _surfaces: &'a [i32],
    ) -> Self {
        Self { layers, trees }
    }
}
impl<B> NativeOcclusionContext for OcclusionContext<'_, B>
where
    B: RenderSurfaceTypes<RenderSurface = RenderSurfaceImpl<B>>,
    B::Filters: crate::adapter::occlusion::OcclusionFilters,
    B::Layer: OcclusionLayer<B::Transform, i32, Types = B>,
{
    type Types = B;
    type Transform = B::Transform;
    type Surface = i32;
    type Layer = i32;
    fn property_trees(&self) -> &PropertyTreesFor<B> {
        self.trees
    }
    fn surface(&self, id: i32) -> &dyn OcclusionRenderSurface<B::Transform, i32, Types = B> {
        self.trees
            .effect_tree()
            .GetRenderSurface(id)
            .expect("source live surface id")
    }
    fn layer(&self, id: i32) -> &dyn OcclusionLayer<B::Transform, i32, Types = B> {
        self.layers.LayerById(id).expect("source live layer id")
    }
}
impl<B: RenderSurfaceTypes<RenderSurface = RenderSurfaceImpl<B>>> OcclusionLayerTree<i32>
    for OcclusionContext<'_, B>
{
    fn RootRenderSurface(&self) -> Option<i32> {
        LayerTreeImpl::<B>::RootRenderSurfaceForPropertyTrees(self.trees)
            .map(|surface| surface.EffectTreeIndex())
    }
}
