//! Same-package concrete solid-scrollbar constructor/setter dispatch only.
use super::{
    layer_tree_impl::Color4f,
    main_layer::MainLayerImplTree,
    scrollbar_layer_base::{ScrollbarLayerImplTarget, ScrollbarLayerTypes},
};
use crate::cc::{
    input::scrollbar::ScrollbarOrientation,
    trees::layer_tree_impl::{LayerTreeContext, LayerTreeImpl},
};
pub trait SolidColorScrollbarLayerTypes:
    ScrollbarLayerTypes<Layer: SolidColorScrollbarLayerImplTarget<Self>>
{
}
pub trait SolidColorScrollbarLayerImplTarget<B: SolidColorScrollbarLayerTypes>:
    ScrollbarLayerImplTarget<B>
{
    fn CreateSolidColorScrollbar(
        tree: &mut LayerTreeImpl<B>,
        id: i32,
        orientation: ScrollbarOrientation,
        thumb_thickness: i32,
        track_start: i32,
        left: bool,
        context: &mut LayerTreeContext<'_, B>,
    ) -> Box<Self>;
    fn scrollbar_set_color(&mut self, color: Color4f, tree: &mut B::ImplContext<'_>);
}
