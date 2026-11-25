//! Same-package concrete solid-scrollbar setter dispatch only. The LayerImpl
//! constructor is reached through `main_layer::LayerImplFactory`.
use super::{
    layer_tree_impl::Color4f,
    scrollbar_layer_base::{ScrollbarLayerImplTarget, ScrollbarLayerTypes},
};
pub trait SolidColorScrollbarLayerTypes:
    ScrollbarLayerTypes
{
}
pub trait SolidColorScrollbarLayerImplTarget<B: SolidColorScrollbarLayerTypes>:
    ScrollbarLayerImplTarget<B>
{
    fn scrollbar_set_color(&mut self, color: Color4f, tree: &mut Self::ImplContext<'_>);
}
