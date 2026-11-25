//! Same-package painted-scrollbar push-target setters. The LayerImpl
//! constructor is reached through `main_layer::LayerImplFactory`; the
//! PaintedScrollbarLayerImpl itself stays in the impl half.
use super::{
    geometry::{Rect, Size},
    layer_tree_impl::UIResourceId,
    scrollbar_layer_base::{ScrollbarLayerImplTarget, ScrollbarLayerTypes},
};

pub trait PaintedScrollbarLayerTypes:
    ScrollbarLayerTypes
{
}

/// The PaintedScrollbarLayerImpl setters that
/// PaintedScrollbarLayer::PushPropertiesTo calls
/// (cc/layers/painted_scrollbar_layer.cc PushPropertiesTo), on the resident
/// push target. Implementors forward each call to the native setter.
pub trait PaintedScrollbarLayerImplTarget<B: PaintedScrollbarLayerTypes>:
    ScrollbarLayerImplTarget<B>
{
    fn painted_SetInternalContentsScaleAndBounds(&mut self, scale: f32, bounds: Size);
    fn painted_SetThumbUIResourceId(&mut self, id: UIResourceId);
    fn painted_SetScrollbarPaintedOpacity(&mut self, opacity: f32);
    fn painted_SetTrackRect(&mut self, rect: Rect);
    fn painted_SetMinimumThumbLength(&mut self, length: i32);
    fn painted_SetThumbThickness(&mut self, thickness: i32);
    fn painted_set_is_overlay_scrollbar(&mut self, is_overlay: bool);
}
