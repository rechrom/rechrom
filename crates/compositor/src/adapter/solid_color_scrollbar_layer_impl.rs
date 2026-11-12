//! Same-package owned solid-scrollbar enum injection; no source decisions.
use super::solid_color_layer_impl::SolidColorLayerImplTypes;
use crate::cc::layers::painted_scrollbar_layer_impl::PaintedScrollbarLayerImpl;
use crate::cc::layers::solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImpl;
pub trait SolidColorScrollbarLayerImplTypes: SolidColorLayerImplTypes {
    fn WrapSolidColorScrollbarLayerImpl(layer: SolidColorScrollbarLayerImpl<Self>) -> Self::Layer;
    fn WrapPaintedScrollbarLayerImpl(layer: PaintedScrollbarLayerImpl<Self>) -> Self::Layer;
}
