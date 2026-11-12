//! Typed Viz boundary used by cc::SurfaceLayerImpl. The resident layer owns
//! only Chromium's SurfaceRange and embedding flags; Surface lifetime and
//! resolution remain in the viz package.
use super::{
    geometry::Rect,
    layer_tree_impl::Color4f,
    solid_color_layer_impl::{SolidColorLayerImplTypes, SolidColorRenderPass},
};
use crate::cc::layers::surface_layer_impl::SurfaceLayerImpl;

pub trait SurfaceLayerImplTypes:
    SolidColorLayerImplTypes<RenderPass: SurfaceRenderPass<Self>>
{
    fn WrapSurfaceLayerImpl(layer: SurfaceLayerImpl<Self>) -> Self::Layer;
    fn DefaultSurfaceRange() -> Self::SurfaceRange;
    fn SurfaceRangesEqual(a: &Self::SurfaceRange, b: &Self::SurfaceRange) -> bool;
    fn SurfaceRangeIsValid(range: &Self::SurfaceRange) -> bool;
    fn SurfaceRangeEnd(range: &Self::SurfaceRange) -> Self::SurfaceId;
}

pub trait SurfaceRenderPass<B: SurfaceLayerImplTypes>: SolidColorRenderPass<B> {
    fn AppendSurfaceDrawQuad(
        &mut self,
        state: B::SharedQuadStateHandle,
        rect: Rect,
        visible: Rect,
        range: &B::SurfaceRange,
        background_color: Color4f,
        stretch_content_to_fill_bounds: bool,
        is_reflection: bool,
        allow_merge: bool,
    );
}
