//! Source scrollbar virtual dispatch on the actual resident selected layer.
//! No layer/tree handle carries state; every operation borrows the current LTI.
use super::{engine_factory::NativeFactoryCapabilities, layer::CompositorLayer};
use crate::{
    adapter::{
        geometry::{PointF, Rect, Size},
        input_handler::InputLayerId,
        input_types::ElementId,
        layer_impl::{LayerImplOwner, LayerImplPushContext},
        layer_tree_host_impl::HostImplLayerServices,
        layer_tree_impl::{LayerTreeId, PropertyTreesFor},
        scrollbar_layer_impl_base::{
            ScrollbarGeometryRead, ScrollbarGeometryStyle, ScrollbarOpacityContext,
        },
    },
    cc::{
        input::scrollbar::{ScrollbarOrientation, ScrollbarPart},
        layers::{
            scrollbar_layer_impl_base::{
                ScrollbarLayerImplBase, ScrollbarLayerImplOwner, ScrollbarLayerImplVirtual,
            },
            solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImpl,
        },
        trees::{
            layer_tree_impl::LayerTreeImpl,
            layer_tree_settings::{LayerTreeSettings, ScrollbarAnimator},
        },
    },
    NativeCompositorTypes as B,
};
use std::{collections::BTreeSet, sync::Arc};

pub(crate) struct NativeScrollbarLayerServices {
    native: Arc<NativeFactoryCapabilities>,
    settings: LayerTreeSettings,
}
struct Read<'a> {
    trees: &'a PropertyTreesFor<B>,
    settings: &'a LayerTreeSettings,
}
impl ScrollbarGeometryStyle for Read<'_> {
    fn ScrollbarAnimator(&self) -> ScrollbarAnimator {
        self.settings.scrollbar_animator
    }
    fn IsFluentOverlayScrollbarEnabled(&self) -> bool {
        self.settings.enable_fluent_overlay_scrollbar
    }
}
impl ScrollbarGeometryRead<B> for Read<'_> {
    fn PropertyTrees(&self) -> &PropertyTreesFor<B> {
        self.trees
    }
    fn Settings(&self) -> &LayerTreeSettings {
        self.settings
    }
}
struct Mutation<'a> {
    id: LayerTreeId,
    trees: &'a mut PropertyTreesFor<B>,
    draw: &'a mut bool,
    push: &'a mut BTreeSet<i32>,
    native: &'a NativeFactoryCapabilities,
    settings: &'a LayerTreeSettings,
}
impl LayerImplPushContext for Mutation<'_> {
    type Types = B;
    fn id(&self) -> LayerTreeId {
        self.id
    }
    fn IsActiveTree(&self) -> bool {
        self.trees.is_active()
    }
    fn TreesInVizInClientProcess(&self) -> bool {
        self.native.picture.trees_in_viz && !self.settings.trees_in_viz_in_viz_process
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.native.picture.cc_slimming
    }
    fn AddLayerShouldPushProperties(&mut self, id: i32) {
        self.push.insert(id);
    }
    fn set_needs_update_draw_properties(&mut self) {
        *self.draw = true;
    }
}
impl ScrollbarOpacityContext<B> for Mutation<'_> {
    fn trees_in_viz_in_viz_process(&self) -> bool {
        self.settings.trees_in_viz_in_viz_process
    }
    fn TreeAnimationsInVizInVizProcess(&self) -> bool {
        self.native.picture.tree_animations_in_viz && self.settings.trees_in_viz_in_viz_process
    }
    fn property_trees_mut(&mut self) -> &mut PropertyTreesFor<B> {
        self.trees
    }
    fn set_needs_update_draw_properties(&mut self) {
        *self.draw = true;
    }
}
impl NativeScrollbarLayerServices {
    pub(crate) fn new(native: Arc<NativeFactoryCapabilities>, settings: LayerTreeSettings) -> Self {
        Self { native, settings }
    }
    fn With<R>(
        &self,
        tree: &LayerTreeImpl<B>,
        id: &InputLayerId,
        callback: impl FnOnce(&ScrollbarLayerImplBase<B>, &Read<'_>) -> R,
    ) -> R {
        assert_eq!(tree.id, id.tree);
        let layer = tree.LayerById(id.id).expect("source live scrollbar layer");
        let base = layer
            .AsScrollbarBase()
            .expect("source ToScrollbarLayer subtype");
        assert_eq!(base.base().tree_id(), tree.id);
        callback(
            base,
            &Read {
                trees: tree.property_trees(),
                settings: &self.settings,
            },
        )
    }
    fn Geometry<R>(
        &self,
        tree: &LayerTreeImpl<B>,
        id: &InputLayerId,
        callback: impl FnOnce(&SolidColorScrollbarLayerImpl<B>, &Read<'_>) -> R,
    ) -> R {
        assert_eq!(tree.id, id.tree);
        let CompositorLayer::SolidColorScrollbar(layer) =
            tree.LayerById(id.id).expect("source live scrollbar layer")
        else {
            panic!("source scrollbar subtype outside the configured CPU family")
        };
        callback(
            layer,
            &Read {
                trees: tree.property_trees(),
                settings: &self.settings,
            },
        )
    }
    fn Mutate<R>(
        &self,
        tree: &mut LayerTreeImpl<B>,
        id: &InputLayerId,
        callback: impl FnOnce(&mut ScrollbarLayerImplBase<B>, &mut Mutation<'_>) -> R,
    ) -> R {
        assert_eq!(tree.id, id.tree);
        let (layers, fields) = tree.SplitLayerTreeFields();
        layers.WithStorageAndTracking(|storage, tracking| {
            let layer = storage
                .find_mut(id.id)
                .expect("source live scrollbar layer")
                .AsScrollbarBaseMutable()
                .expect("source ToScrollbarLayer subtype");
            let mut context = Mutation {
                id: fields.id,
                trees: fields.property_trees,
                draw: fields.needs_update_draw_properties,
                push: tracking.should_push_properties,
                native: &self.native,
                settings: &self.settings,
            };
            callback(layer, &mut context)
        })
    }
}
impl HostImplLayerServices<B> for NativeScrollbarLayerServices {
    fn Layer_Scrollbar_orientation(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> ScrollbarOrientation {
        self.With(tree, layer, |base, _c| base.orientation())
    }
    fn Layer_Scrollbar_scroll_element_id(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> ElementId {
        self.With(tree, layer, |base, _c| base.scroll_element_id())
    }
    fn Layer_Scrollbar_OverlayScrollbarOpacity(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> f32 {
        self.With(tree, layer, |base, c| base.OverlayScrollbarOpacity(c))
    }
    fn Layer_Scrollbar_UsesAuraOverlayAnimator(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool {
        self.With(tree, layer, |base, c| {
            base.GetScrollbarAnimatorWithStyle(c) == ScrollbarAnimator::AURA_OVERLAY
        })
    }
    fn Layer_Scrollbar_has_find_in_page_tickmarks(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool {
        self.With(tree, layer, |base, _c| base.has_find_in_page_tickmarks())
    }
    fn Layer_Scrollbar_JumpOnTrackClick(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool {
        self.With(tree, layer, |base, _c| base.JumpOnTrackClick())
    }
    fn Layer_Scrollbar_current_pos(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> f32 {
        self.With(tree, layer, |base, _c| base.current_pos())
    }
    fn Layer_Scrollbar_scroll_layer_length(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> f32 {
        self.With(tree, layer, |base, _c| base.scroll_layer_length())
    }
    fn Layer_Scrollbar_SupportsDragSnapBack(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool {
        self.With(tree, layer, |base, _c| base.SupportsDragSnapBack())
    }
    fn Layer_Scrollbar_ForwardTrackRect(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Rect {
        self.With(tree, layer, |base, _c| base.ForwardTrackRect())
    }
    fn Layer_Scrollbar_BackTrackRect(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Rect {
        self.With(tree, layer, |base, _c| base.BackTrackRect())
    }
    fn Layer_Scrollbar_BackButtonRect(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Rect {
        self.With(tree, layer, |base, _c| base.BackButtonRect())
    }
    fn Layer_Scrollbar_ForwardButtonRect(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Rect {
        self.With(tree, layer, |base, _c| base.ForwardButtonRect())
    }
    fn Layer_Scrollbar_ScreenSpaceTransform(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> super::geometry::Transform<super::skia::SkRegion> {
        self.With(tree, layer, |base, c| {
            base.base().ScreenSpaceTransformForPropertyTrees(c.trees)
        })
    }
    fn Layer_Scrollbar_is_overlay_scrollbar(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool {
        self.With(tree, layer, |base, _c| base.is_overlay_scrollbar())
    }
    fn Layer_Scrollbar_bounds(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> Size {
        self.With(tree, layer, |base, c| {
            base.base().bounds_for_property_trees(c.trees)
        })
    }
    fn Layer_Scrollbar_CanScrollOrientation(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> bool {
        self.With(tree, layer, |base, c| base.CanScrollOrientation(c))
    }
    fn Layer_Scrollbar_thumb_thickness_scale_factor(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        layer: &InputLayerId,
    ) -> f32 {
        self.With(tree, layer, |base, _c| base.thumb_thickness_scale_factor())
    }
    fn Layer_Scrollbar_ComputeThumbQuadRect(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        id: &InputLayerId,
    ) -> Rect {
        self.Geometry(tree, id, |layer, c| {
            layer
                .scrollbar_base()
                .ComputeThumbQuadRect(&layer.Geometry(), c)
        })
    }
    fn Layer_Scrollbar_ThumbThickness(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        id: &InputLayerId,
    ) -> i32 {
        self.Geometry(tree, id, |layer, c| layer.ThumbThickness(c))
    }
    fn Layer_Scrollbar_ThumbLength(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        id: &InputLayerId,
    ) -> i32 {
        self.Geometry(tree, id, |layer, c| {
            layer.scrollbar_base().ThumbLength(&layer.Geometry(), c)
        })
    }
    fn Layer_Scrollbar_TrackLength(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        id: &InputLayerId,
    ) -> f32 {
        self.Geometry(tree, id, |layer, c| layer.TrackLength(c))
    }
    fn Layer_Scrollbar_ComputeHitTestableExpandedThumbQuadRect(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        id: &InputLayerId,
    ) -> Rect {
        self.Geometry(tree, id, |layer, c| {
            layer
                .scrollbar_base()
                .ComputeHitTestableExpandedThumbQuadRect(&layer.Geometry(), c)
        })
    }
    fn Layer_Scrollbar_IdentifyScrollbarPart(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        id: &InputLayerId,
        position: PointF,
    ) -> ScrollbarPart {
        self.Geometry(tree, id, |layer, c| {
            layer
                .scrollbar_base()
                .IdentifyScrollbarPart(&layer.Geometry(), position, c)
        })
    }
    fn Layer_Scrollbar_device_scale_factor(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        id: &InputLayerId,
    ) -> f32 {
        assert_eq!(tree.id, id.tree);
        tree.device_scale_factor()
    }
    fn Layer_Scrollbar_SetThumbThicknessScaleFactor(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        id: &InputLayerId,
        scale: f32,
    ) {
        self.Mutate(tree, id, |layer, c| {
            let (base, mut fields) = layer.SplitGeometryFields();
            fields.SetThumbThicknessScaleFactor(base, scale, c);
        });
    }
    fn Layer_Scrollbar_SetOverlayScrollbarLayerOpacityAnimated(
        &mut self,
        tree: &mut LayerTreeImpl<B>,
        id: &InputLayerId,
        opacity: f32,
        fade: bool,
    ) {
        self.Mutate(tree, id, |layer, c| {
            layer.SetOverlayScrollbarLayerOpacityAnimatedWithContext(opacity, fade, c)
        });
    }
}
