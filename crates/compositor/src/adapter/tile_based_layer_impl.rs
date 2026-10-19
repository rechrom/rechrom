//! Exact external Viz storage and tracing values consumed by tile quads.
//! Tile selection, visibility, checkerboard and scale tracking stay canonical.
use super::{
    geometry::{Rect, RectF},
    layer_impl::LayerImplSharedQuadState,
    layer_tree_impl::{Color4f, ResourceId},
    solid_color_layer_impl::{SolidColorLayerImplTypes, SolidColorRenderPass},
};
pub trait TileBasedLayerImplTypes:
    SolidColorLayerImplTypes<
    SharedQuadState: TileBasedSharedQuadState<Self::Transform>,
    RenderPass: TileBasedRenderPass<Self>,
>
{
}
pub trait TileBasedSharedQuadState<T>: LayerImplSharedQuadState<T> {
    fn quad_to_target_transform(&self) -> &T;
    fn quad_to_target_transform_mut(&mut self) -> &mut T;
    fn quad_layer_rect(&self) -> Rect;
    fn set_quad_layer_rect(&mut self, rect: Rect);
    fn visible_quad_layer_rect(&self) -> Rect;
    fn set_visible_quad_layer_rect(&mut self, rect: Rect);
    fn clip_rect(&self) -> Option<Rect>;
    fn set_clip_rect(&mut self, rect: Option<Rect>);
}
pub trait TileBasedRenderPass<B: TileBasedLayerImplTypes>: SolidColorRenderPass<B> {
    fn DrawQuad(&self, quad: B::DrawQuadHandle) -> &B::DrawQuad;
    fn CreateAndAppendDebugBorderDrawQuad(&mut self) -> B::DrawQuadHandle;
    fn DebugBorderDrawQuadSetNew(
        &mut self,
        quad: B::DrawQuadHandle,
        state: B::SharedQuadStateHandle,
        geometry: Rect,
        visible: Rect,
        color: Color4f,
        width: f32,
    );
    fn CreateAndAppendTileDrawQuad(&mut self) -> B::DrawQuadHandle;
    fn TileDrawQuadSetNew(
        &mut self,
        quad: B::DrawQuadHandle,
        state: B::SharedQuadStateHandle,
        geometry: Rect,
        visible: Rect,
        needs_blending: bool,
        resource: ResourceId,
        texture: RectF,
        nearest_neighbor: bool,
        force_anti_aliasing_off: bool,
    );
}
/// Synchronous trace sink; no layer, tree or tile state is retained.
pub trait TileBasedTrace {
    fn CheckerboardTrace(&self, missing_tile_count: i32);
}

/// Per-call context family. It lends the same actual tree/services only at a
/// base call site; derived hooks receive that original context unchanged.
pub trait TileBasedLayerContext<B: TileBasedLayerImplTypes> {
    fn BaseContext(&mut self) -> super::layer_impl::LayerImplContext<'_, B>;
}
impl<B: TileBasedLayerImplTypes> TileBasedLayerContext<B>
    for super::layer_impl::LayerImplContext<'_, B>
{
    fn BaseContext(&mut self) -> super::layer_impl::LayerImplContext<'_, B> {
        super::layer_impl::LayerImplContext {
            tree: self.tree,
            services: self.services,
        }
    }
}
