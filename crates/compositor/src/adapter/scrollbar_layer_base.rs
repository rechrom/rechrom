//! Native scrollbar handles and same-package derived constructor/setter dispatch.
//! No selection, reuse, push, or scrollbar policy is implemented by this boundary.
use super::{
    input_types::ElementId,
    layer_tree_impl::Color4f,
    main_layer::{LayerContext, MainLayerImpl, MainLayerImplTree, MainLayerTypes},
};
use crate::cc::{
    input::scrollbar::{Scrollbar, ScrollbarOrientation},
    layers::layer::{LayerBorrow, LayerRef},
};
use std::sync::Arc;
pub trait ScrollbarLayerTypes: MainLayerTypes<Layer: ScrollbarLayerImplTarget<Self>> {
    type ScrollbarCanvas: ?Sized;
    type Scrollbar: Scrollbar<Self::ScrollbarCanvas> + Send + Sync + 'static;
    /// Optional same-package constructor capabilities. The default CPU family
    /// declares absence for owners outside its canonical closure. A capability
    /// is the exact native constructor function, never a replacement algorithm.
    fn PaintedScrollbarConstructor() -> Option<ScrollbarConstructor<Self>> {
        None
    }
    fn NinePatchScrollbarConstructor() -> Option<ScrollbarConstructor<Self>> {
        None
    }
}
/// This function pointer contains no service/host/context capture. The actual
/// canonical derived constructor runs immediately with the caller's live scope.
pub type ScrollbarConstructor<B> = for<'a, 'c> fn(
    Arc<<B as ScrollbarLayerTypes>::Scrollbar>,
    Option<LayerBorrow<B>>,
    &'a mut LayerContext<'c, B>,
) -> LayerRef<B>;

/// Typed external capability failure, surfaced by the Page command boundary.
/// This is not a compositor fallback or a fabricated derived layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarConstructorUnavailable {
    Painted,
    NinePatchThumb,
}

pub trait ScrollbarLayerImplTarget<B: ScrollbarLayerTypes>: MainLayerImpl<B> {
    fn scrollbar_orientation(&self) -> ScrollbarOrientation;
    fn scrollbar_is_left_side_vertical(&self) -> bool;
    fn scrollbar_SetHasFindInPageTickmarks(&mut self, value: bool, tree: &mut B::ImplContext<'_>);
    fn scrollbar_SetScrollElementId(&mut self, value: ElementId, tree: &mut B::ImplContext<'_>);
}
/// Native static_cast to the actual solid-color subtype; implementation only
/// exposes that resident object's native fields/setter, never substitutes one.
pub trait SolidColorScrollbarLayerObject<B: MainLayerTypes> {
    fn thumb_thickness(&self) -> i32;
    fn track_start(&self) -> i32;
    fn SetColor(&self, color: Color4f, c: &mut LayerContext<'_, B>);
}
