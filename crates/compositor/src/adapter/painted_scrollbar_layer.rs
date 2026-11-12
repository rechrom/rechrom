//! Same-package painted-scrollbar constructor dispatch.
use super::painted_scrollbar_layer_impl::PaintedScrollbarLayerImplTypes;
use super::scrollbar_layer_base::{ScrollbarLayerImplTarget, ScrollbarLayerTypes};
use crate::cc::input::scrollbar::ScrollbarOrientation;
use crate::cc::layers::painted_scrollbar_layer_impl::PaintedScrollbarLayerImpl;
use crate::cc::trees::layer_tree_impl::{LayerTreeContext, LayerTreeImpl};

pub trait PaintedScrollbarLayerTypes:
    ScrollbarLayerTypes<Layer: PaintedScrollbarLayerImplTarget<Self>> + PaintedScrollbarLayerImplTypes
{
}

pub trait PaintedScrollbarLayerImplTarget<B: PaintedScrollbarLayerTypes>:
    ScrollbarLayerImplTarget<B>
{
    fn CreatePaintedScrollbar(
        tree: &mut LayerTreeImpl<B>,
        id: i32,
        orientation: ScrollbarOrientation,
        left: bool,
        context: &mut LayerTreeContext<'_, B>,
    ) -> Box<Self>;
    fn painted_scrollbar(&mut self) -> Option<&mut PaintedScrollbarLayerImpl<B>>;
}
