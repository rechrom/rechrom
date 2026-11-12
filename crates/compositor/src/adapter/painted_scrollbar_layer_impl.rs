//! Viz output capability used by PaintedScrollbarLayerImpl.
use super::solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImplTypes;

pub trait PaintedScrollbarLayerImplTypes: SolidColorScrollbarLayerImplTypes {}
impl<T: SolidColorScrollbarLayerImplTypes> PaintedScrollbarLayerImplTypes for T {}
