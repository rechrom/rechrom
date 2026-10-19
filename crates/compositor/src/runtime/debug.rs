//! The diagnostic source walk borrows the actual layer table. Its tree queries
//! receive the caller's property trees and HUD access is a real subtype projection.
use super::{draw::LayerIds, layer::CompositorLayer};
use crate::{
    adapter::{
        debug_rect::{DebugRectContext, DebugRectHud, DebugRectHudProjection, DebugRectLayer},
        geometry::{Rect, Size},
        layer_impl::LayerImplOwner,
        layer_tree_host_impl::HostDebugDrawState,
        layer_tree_impl::{LayerTreeLayer, LayerTreeLayerList, PropertyTreesFor},
        picture_layer_impl::PictureLayerImplTypes,
        render_surface_impl::RenderSurfaceTypes,
        solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImplTypes,
    },
    cc::{
        base::region::Region, input::touch_action::TouchAction,
        layers::render_surface_impl::RenderSurfaceImpl,
    },
};
impl<B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes>
    DebugRectLayer<
        B::Transform,
        <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::RegionBackend,
    > for CompositorLayer<B>
{
    type Types = B;
    fn GetInvalidationRegionForDebugging(
        &self,
    ) -> Region<<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::RegionBackend> {
        crate::adapter::layer_tree_host_impl::HostImplLayer::GetInvalidationRegionForDebugging(self)
    }
    fn draws_content(&self) -> bool {
        self.impl_base().draws_content()
    }
    fn ScreenSpaceTransform(&self, p: &PropertyTreesFor<B>) -> B::Transform {
        self.impl_base().ScreenSpaceTransformForPropertyTrees(p)
    }
    fn LayerPropertyChanged(&self, p: &PropertyTreesFor<B>) -> bool {
        self.impl_base().LayerPropertyChangedForPropertyTrees(p)
    }
    fn bounds(&self, p: &PropertyTreesFor<B>) -> Size {
        self.impl_base().bounds_for_property_trees(p)
    }
    fn touch_action_region(
        &self,
    ) -> impl Iterator<
        Item = (
            TouchAction,
            &Region<<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::RegionBackend>,
        ),
    > {
        self.impl_base()
            .touch_action_region()
            .iter()
            .map(|(action, region)| (*action, region))
    }
    fn wheel_event_handler_region(
        &self,
    ) -> &Region<<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::RegionBackend> {
        self.impl_base().wheel_event_handler_region()
    }
    fn main_thread_scroll_hit_test_region(
        &self,
    ) -> &Region<<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::RegionBackend> {
        self.impl_base().main_thread_scroll_hit_test_region()
    }
}
impl<B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes> DebugRectHudProjection<B>
    for CompositorLayer<B>
{
    fn AsDebugRectHud(&mut self) -> Option<&mut dyn DebugRectHud<B::Transform, Types = B>> {
        // This actual enum contains Base/Solid/Picture/SolidScrollbar only.
        // No fabricated HUD fields or clear operation is attached to any one.
        None
    }
}
pub struct DebugContext<'a, B: RenderSurfaceTypes> {
    state: HostDebugDrawState<'a, B>,
}
impl<'a, B: RenderSurfaceTypes> DebugContext<'a, B> {
    pub fn new(state: HostDebugDrawState<'a, B>) -> Self {
        Self { state }
    }
}
impl<B> DebugRectContext for DebugContext<'_, B>
where
    B: RenderSurfaceTypes<RenderSurface = RenderSurfaceImpl<B>>,
    B::Layer: DebugRectLayer<B::Transform, B::RegionBackend, Types = B> + DebugRectHudProjection<B>,
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
    type RegionBackend = B::RegionBackend;
    type DebugLayer = B::Layer;
    fn layers(&self) -> Self::Layers<'_> {
        self.state.layers.iter().map(LayerTreeLayer::id)
    }
    fn debug_layer(&self, id: i32) -> &B::Layer {
        self.state
            .layers
            .LayerById(id)
            .expect("source live debug layer")
    }
    fn hud_mut(&mut self, id: i32) -> Option<&mut dyn DebugRectHud<B::Transform, Types = B>> {
        self.state
            .layers
            .LayerByIdMut(id)
            .expect("source live HUD layer")
            .AsDebugRectHud()
    }
    fn have_scroll_event_handlers(&self) -> bool {
        self.state.have_scroll_event_handlers
    }
    fn GetDamageRect(&self, surface: &Self::Surface) -> Rect {
        surface.GetDamageRect()
    }
}
