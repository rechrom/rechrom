//! Same-package painted-scrollbar constructor dispatch.
use super::painted_scrollbar_layer_impl::PaintedScrollbarLayerImplTypes;
use super::scrollbar_layer_base::{ScrollbarLayerImplTarget, ScrollbarLayerTypes};
use crate::cc::layers::painted_scrollbar_layer_impl::PaintedScrollbarLayerImpl;

pub trait PaintedScrollbarLayerTypes:
    ScrollbarLayerTypes<Layer: PaintedScrollbarLayerImplTarget<Self>> + PaintedScrollbarLayerImplTypes
{
}

pub trait PaintedScrollbarLayerImplTarget<B: PaintedScrollbarLayerTypes>:
    ScrollbarLayerImplTarget<B>
{
    fn painted_scrollbar(&mut self) -> Option<&mut PaintedScrollbarLayerImpl<B>>;
}
