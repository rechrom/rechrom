//! Call-scoped actual scrollbar tree fields and callbacks; no owner is retained.
use super::input_types::ElementId;
use super::layer_impl::{LayerImplContext, LayerImplServices, LayerImplTree, LayerImplTypes};
use crate::cc::layers::scrollbar_layer_impl_base::ScrollbarLayerImplBase;
pub trait ScrollbarLayerImplTree<B: LayerImplTypes>: LayerImplTree<B> {
    /// Source LayerTreeImpl::{UnregisterScrollbar,RegisterScrollbar}. The
    /// receiver is the actual split-borrowed resident layer, including callbacks.
    fn UnregisterScrollbar(&mut self, layer: &mut ScrollbarLayerImplBase<B>);
    fn RegisterScrollbar(&mut self, layer: &mut ScrollbarLayerImplBase<B>);
    fn RequestShowScrollbars(&mut self, element: ElementId);
    fn TreeAnimationsInVizInVizProcess(&self) -> bool;
}
pub struct ScrollbarLayerImplContext<'a, B: LayerImplTypes> {
    pub tree: &'a mut dyn ScrollbarLayerImplTree<B>,
    pub services: &'a mut dyn LayerImplServices<B>,
}
impl<B: LayerImplTypes> ScrollbarLayerImplContext<'_, B> {
    pub fn Base(&mut self) -> LayerImplContext<'_, B> {
        LayerImplContext {
            tree: self.tree,
            services: self.services,
        }
    }
}

/// Style is read directly at the source call point. It is never cached on a
/// scrollbar geometry view or copied into a second state object.
pub trait ScrollbarGeometryStyle {
    fn ScrollbarAnimator(&self) -> crate::cc::trees::layer_tree_settings::ScrollbarAnimator;
    fn IsFluentOverlayScrollbarEnabled(&self) -> bool;
}
pub trait ScrollbarGeometryContext<B: super::layer_tree_impl::LayerTreeTypes>:
    super::layer_impl::LayerImplPushContext<Types = B> + ScrollbarGeometryStyle
{
    fn RequestShowScrollbars(&mut self, element: ElementId);
}
impl<B: LayerImplTypes> super::layer_impl::LayerImplPushContext
    for ScrollbarLayerImplContext<'_, B>
{
    type Types = B;
    fn id(&self) -> super::layer_tree_impl::LayerTreeId {
        self.tree.id()
    }
    fn IsActiveTree(&self) -> bool {
        self.tree.IsActiveTree()
    }
    fn TreesInVizInClientProcess(&self) -> bool {
        self.tree.TreesInVizInClientProcess()
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.tree.IsCCSlimmingEnabled()
    }
    fn AddLayerShouldPushProperties(&mut self, id: i32) {
        self.tree.AddLayerShouldPushProperties(id)
    }
    fn set_needs_update_draw_properties(&mut self) {
        self.tree.set_needs_update_draw_properties()
    }
}
impl<B: LayerImplTypes> ScrollbarGeometryStyle for ScrollbarLayerImplContext<'_, B> {
    fn ScrollbarAnimator(&self) -> crate::cc::trees::layer_tree_settings::ScrollbarAnimator {
        self.tree.settings().scrollbar_animator
    }
    fn IsFluentOverlayScrollbarEnabled(&self) -> bool {
        self.tree.settings().enable_fluent_overlay_scrollbar
    }
}
impl<B: LayerImplTypes> ScrollbarGeometryStyle for super::layer_impl::LayerImplContext<'_, B> {
    fn ScrollbarAnimator(&self) -> crate::cc::trees::layer_tree_settings::ScrollbarAnimator {
        self.tree.settings().scrollbar_animator
    }
    fn IsFluentOverlayScrollbarEnabled(&self) -> bool {
        self.tree.settings().enable_fluent_overlay_scrollbar
    }
}
impl<B: LayerImplTypes> ScrollbarGeometryContext<B> for ScrollbarLayerImplContext<'_, B> {
    fn RequestShowScrollbars(&mut self, id: ElementId) {
        self.tree.RequestShowScrollbars(id)
    }
}

/// Exact properties touched by native scrollbar opacity updates. No layer
/// table, input owner, task client or callback registration is exposed here.
pub trait ScrollbarOpacityContext<B: super::layer_tree_impl::LayerTreeTypes> {
    fn trees_in_viz_in_viz_process(&self) -> bool;
    fn TreeAnimationsInVizInVizProcess(&self) -> bool;
    fn property_trees_mut(&mut self) -> &mut super::layer_tree_impl::PropertyTreesFor<B>;
    fn set_needs_update_draw_properties(&mut self);
}
impl<B: LayerImplTypes> ScrollbarOpacityContext<B> for ScrollbarLayerImplContext<'_, B> {
    fn trees_in_viz_in_viz_process(&self) -> bool {
        self.tree.settings().trees_in_viz_in_viz_process
    }
    fn TreeAnimationsInVizInVizProcess(&self) -> bool {
        self.tree.TreeAnimationsInVizInVizProcess()
    }
    fn property_trees_mut(&mut self) -> &mut super::layer_tree_impl::PropertyTreesFor<B> {
        self.tree.property_trees_mut()
    }
    fn set_needs_update_draw_properties(&mut self) {
        self.tree.set_needs_update_draw_properties()
    }
}

/// The source LTI registration body borrows its actual registry and callbacks.
/// No layer lookup escapes the current receiver/remaining-storage scope.
pub trait ScrollbarRegistrationContext<B: LayerImplTypes>:
    ScrollbarGeometryStyle + ScrollbarOpacityContext<B>
{
    fn IsActiveTree(&self) -> bool;
    fn Registry(
        &self,
    ) -> &std::collections::BTreeMap<ElementId, crate::cc::trees::layer_tree_impl::ScrollbarLayerIds>;
    fn RegistryMut(
        &mut self,
    ) -> &mut std::collections::BTreeMap<
        ElementId,
        crate::cc::trees::layer_tree_impl::ScrollbarLayerIds,
    >;
    fn PropertyTrees(&self) -> &super::layer_tree_impl::PropertyTreesFor<B>;
    fn DidRegisterScrollbarLayer(
        &mut self,
        id: ElementId,
        orientation: crate::cc::input::scrollbar::ScrollbarOrientation,
    );
    fn DidUnregisterScrollbarLayer(&mut self, layer: &mut ScrollbarLayerImplBase<B>);
    fn RegisterScrollbarAnimationController(
        &mut self,
        layer: &mut ScrollbarLayerImplBase<B>,
        opacity: f32,
    );
}

/// Source scrollbar geometry reads only its current property trees and settings.
/// This short borrow replaces the source layer->tree pointer access, not the
/// entire mutable LayerImplContext or any cached layer/tree projection.
pub trait ScrollbarGeometryRead<B: LayerImplTypes>: ScrollbarGeometryStyle {
    fn PropertyTrees(&self) -> &super::layer_tree_impl::PropertyTreesFor<B>;
    fn Settings(&self) -> &crate::cc::trees::layer_tree_settings::LayerTreeSettings;
}
impl<B: LayerImplTypes> ScrollbarGeometryRead<B> for LayerImplContext<'_, B> {
    fn PropertyTrees(&self) -> &super::layer_tree_impl::PropertyTreesFor<B> {
        self.tree.property_trees()
    }
    fn Settings(&self) -> &crate::cc::trees::layer_tree_settings::LayerTreeSettings {
        self.tree.settings()
    }
}
