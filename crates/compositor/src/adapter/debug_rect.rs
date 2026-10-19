//! Live layer/HUD/resource reads for canonical DebugRectHistory.
use super::{
    draw_property::{DrawSurface, DrawTransform},
    geometry::{Rect, Size},
    property_tree::{PropertyTreeCopyOutputRequest, PropertyTreeFilters},
    property_types::PropertyPath,
    region_backend::SkRegionBackend,
};
use crate::cc::{base::region::Region, input::touch_action::TouchAction};
// Source dependency: cc/layers/heads_up_display_layer.h:20-24,27-31.
#[derive(Clone, Copy, Debug)]
pub enum WebVitalMetricType {
    kLayoutShift,
    kInteractionContentfulPaint,
    kNavigationContentfulPaint,
}
#[derive(Clone, Copy, Debug)]
pub struct WebVitalsDebugRect {
    pub r#type: WebVitalMetricType,
    pub rect: Rect,
}
pub trait DebugRectHud<T: super::property_tree::PropertyTreeTransform> {
    type Types: super::layer_tree_impl::LayerTreeTypes<Transform = T>;
    fn WebVitalsDebugRects(&self) -> &[WebVitalsDebugRect];
    fn ScreenSpaceTransform(
        &self,
        trees: &super::layer_tree_impl::PropertyTreesFor<Self::Types>,
    ) -> T;
    fn ClearWebVitalsDebugRects(&mut self);
}
pub trait DebugRectLayer<T: super::property_tree::PropertyTreeTransform, B: SkRegionBackend> {
    type Types: super::layer_tree_impl::LayerTreeTypes<Transform = T, RegionBackend = B>;
    fn GetInvalidationRegionForDebugging(&self) -> Region<B>;
    fn draws_content(&self) -> bool;
    fn ScreenSpaceTransform(
        &self,
        trees: &super::layer_tree_impl::PropertyTreesFor<Self::Types>,
    ) -> T;
    fn LayerPropertyChanged(
        &self,
        trees: &super::layer_tree_impl::PropertyTreesFor<Self::Types>,
    ) -> bool;
    fn bounds(&self, trees: &super::layer_tree_impl::PropertyTreesFor<Self::Types>) -> Size;
    fn touch_action_region(&self) -> impl Iterator<Item = (TouchAction, &Region<B>)>;
    fn wheel_event_handler_region(&self) -> &Region<B>;
    fn main_thread_scroll_hit_test_region(&self) -> &Region<B>;
}
// Only live fields consumed by DebugRectHistory. Drawing and feature services
// are deliberately absent; the history algorithm only reads layers/surfaces
// and exclusively clears the resident HUD web-vitals vector.
pub type DebugPropertyTrees<C> = crate::cc::trees::property_tree::PropertyTrees<
    <C as DebugRectContext>::Transform,
    <C as DebugRectContext>::Filters,
    <C as DebugRectContext>::Path,
    <C as DebugRectContext>::Surface,
    <C as DebugRectContext>::CopyRequest,
>;
pub trait DebugRectContext: Sized {
    type Types: super::layer_tree_impl::LayerTreeTypes<
        Transform = Self::Transform,
        Filters = Self::Filters,
        Path = Self::Path,
        RenderSurface = Self::Surface,
        CopyOutputRequest = Self::CopyRequest,
        RegionBackend = Self::RegionBackend,
    >;
    type Transform: DrawTransform + super::property_tree::PropertyTreeTransform;
    type Filters: PropertyTreeFilters;
    type Path: PropertyPath;
    type Surface: DrawSurface<Self::Transform, Types = Self::Types>;
    type CopyRequest: PropertyTreeCopyOutputRequest;
    type Layer: Copy + Eq;
    type Layers<'a>: Iterator<Item = Self::Layer>
    where
        Self: 'a;
    fn layers(&self) -> Self::Layers<'_>;
    type RegionBackend: SkRegionBackend;
    type DebugLayer: DebugRectLayer<Self::Transform, Self::RegionBackend, Types = Self::Types>;
    fn debug_layer(&self, id: Self::Layer) -> &Self::DebugLayer;
    fn hud_mut(
        &mut self,
        id: Self::Layer,
    ) -> Option<&mut dyn DebugRectHud<Self::Transform, Types = Self::Types>>;
    fn have_scroll_event_handlers(&self) -> bool;
    fn GetDamageRect(&self, surface: &Self::Surface) -> Rect;
}

/// Accurate optional projection of the source HUD subtype. A family with no
/// HUD owner returns None; no surrogate layer supplies fabricated HUD state.
pub trait DebugRectHudProjection<B: super::layer_tree_impl::LayerTreeTypes> {
    fn AsDebugRectHud(&mut self) -> Option<&mut dyn DebugRectHud<B::Transform, Types = B>>;
}
