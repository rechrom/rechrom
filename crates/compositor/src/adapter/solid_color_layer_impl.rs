//! Exact native Viz owned-output access used by solid-color quad emission.
use super::{
    geometry::Rect,
    layer_impl::{LayerImplRenderPass, LayerImplTypes},
    layer_tree_impl::Color4f,
};
use crate::cc::layers::solid_color_layer_impl::SolidColorLayerImpl;
pub trait SolidColorLayerImplTypes: LayerImplTypes<RenderPass: SolidColorRenderPass<Self>> {
    fn WrapSolidColorLayerImpl(layer: SolidColorLayerImpl<Self>) -> Self::Layer;
}
pub trait SolidColorRenderPass<B: SolidColorLayerImplTypes>: LayerImplRenderPass<B> {
    fn CreateAndAppendSharedQuadState(&mut self) -> B::SharedQuadStateHandle;
    fn SharedQuadState(&self, state: B::SharedQuadStateHandle) -> &B::SharedQuadState;
    fn SharedQuadStateMutable(
        &mut self,
        state: B::SharedQuadStateHandle,
    ) -> &mut B::SharedQuadState;
    fn CreateAndAppendSolidColorDrawQuad(&mut self) -> B::DrawQuadHandle;
    fn SolidColorDrawQuadSetNew(
        &mut self,
        quad: B::DrawQuadHandle,
        state: B::SharedQuadStateHandle,
        rect: Rect,
        visible: Rect,
        color: Color4f,
        force_anti_aliasing_off: bool,
    );
}
