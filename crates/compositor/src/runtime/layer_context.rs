//! Scoped borrows of the canonical selected tree. Views own references to the
//! original fields only; they contain no copied layer/tree state or backpointer.
use crate::{
    adapter::{
        geometry::{Rect, Vector2dF},
        input_types::ElementId,
        layer_impl::*,
        layer_tree_impl::*,
        main_layer::MainLayerImplTree,
        picture_layer_impl::*,
    },
    cc::{
        layers::layer_collections::LayerTracking,
        trees::{
            layer_tree_impl::LayerTreeLayerFields, layer_tree_settings::LayerTreeSettings,
            transform_node::TransformNode,
        },
    },
};

pub struct LayerContextTree<'a, B: PictureLayerImplTypes, Extra> {
    pub fields: LayerTreeLayerFields<'a, B>,
    pub tracking: LayerTracking<'a>,
    pub read_host: &'a dyn PictureLayerReadHost<B>,
    pub host: &'a mut dyn PictureLayerTreeHost<B>,
    pub extra: Extra,
}
impl<B: PictureLayerImplTypes, E> MainLayerImplTree<B> for LayerContextTree<'_, B, E> {
    fn source_frame_number(&self) -> i32 {
        *self.fields.source_frame_number
    }
}
impl<B: PictureLayerImplTypes, E> PictureLayerReadContext for LayerContextTree<'_, B, E> {
    fn IsRecycleTree(&self) -> bool {
        self.read_host.IsRecycleTree(self.fields.id)
    }
}
impl<B: PictureLayerImplTypes, E> LayerImplReadTree<B> for LayerContextTree<'_, B, E> {
    fn id(&self) -> LayerTreeId {
        self.fields.id
    }
    fn IsActiveTree(&self) -> bool {
        self.read_host.IsActiveTree(self.fields.id)
    }
    fn TreesInVizInClientProcess(&self) -> bool {
        self.read_host.TreesInVizInClientProcess()
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.read_host.IsCCSlimmingEnabled()
    }
    fn property_trees(&self) -> &PropertyTreesFor<B> {
        self.fields.property_trees
    }
    fn settings(&self) -> &LayerTreeSettings {
        self.read_host.settings()
    }
    fn debug_state(&self) -> &crate::cc::debug::layer_tree_debug_state::LayerTreeDebugState {
        self.read_host.debug_state()
    }
    fn device_scale_factor(&self) -> f32 {
        *self.fields.device_scale_factor
    }
    fn max_texture_size(&self) -> i32 {
        self.read_host.raster_caps().max_texture_size
    }
    fn current_page_scale_factor(&self) -> f32 {
        self.fields
            .page_scale_factor
            .borrow()
            .Current(self.IsActiveTree())
    }
    fn LifecycleAllowsPropertyTreeAccess(&self) -> bool {
        self.fields.lifecycle.AllowsPropertyTreeAccess()
    }
}
impl<B: PictureLayerImplTypes, E> LayerImplTree<B> for LayerContextTree<'_, B, E> {
    fn property_trees_mut(&mut self) -> &mut PropertyTreesFor<B> {
        self.fields.property_trees
    }
    fn AddLayerShouldPushProperties(&mut self, id: i32) {
        self.tracking.should_push_properties.insert(id);
    }
    fn set_needs_update_draw_properties(&mut self) {
        *self.fields.needs_update_draw_properties = true;
    }
}
impl<B: PictureLayerImplTypes, E> PictureLayerImplReadTree<B> for LayerContextTree<'_, B, E> {
    fn raster_caps(&self) -> &crate::cc::trees::raster_capabilities::RasterCapabilities {
        self.read_host.raster_caps()
    }
    fn GetMSAASampleCountForRaster(
        &self,
        list:&<<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource as PictureRasterSource>::DisplayItemList,
    ) -> i32 {
        self.read_host.GetMSAASampleCountForRaster(
            list,
            self.fields.id,
            self.fields.device_scale_factor,
        )
    }
    fn GetTargetColorParams(
        &self,
        usage: crate::adapter::layer_tree_host_impl::ContentColorUsage,
    ) -> B::PictureTargetColorParams {
        self.read_host
            .GetTargetColorParams(usage, self.fields.id, self.fields.display_color_spaces)
    }
    fn ViewportRectForTilePriority(&self) -> Rect {
        // Source: cc/trees/layer_tree_impl.cc:2204-2211.
        let viewport = self.read_host.ViewportRectForTilePriority();
        if viewport.IsEmpty() {
            self.GetDeviceViewport()
        } else {
            viewport
        }
    }
    fn IsReadyToActivate(&self) -> bool {
        self.read_host.IsReadyToActivate()
    }
    fn HasPendingTree(&self) -> bool {
        self.read_host.HasPendingTree()
    }
    fn GetActivelyScrollingType(
        &self,
    ) -> crate::cc::input::actively_scrolling_type::ActivelyScrollingType {
        self.read_host
            .GetActivelyScrollingType(self.fields.property_trees.scroll_tree())
    }
    fn SmoothnessTakesPriority(&self) -> bool {
        self.read_host.SmoothnessTakesPriority()
    }
    fn CurrentBeginFrameArgs(&self) -> &crate::adapter::begin_frame::BeginFrameArgs {
        self.read_host.CurrentBeginFrameArgs()
    }
    fn UseGpuRasterization(&self) -> bool {
        self.read_host.UseGpuRasterization()
    }
    fn RequiresHighResToDraw(&self) -> bool {
        self.read_host.RequiresHighResToDraw()
    }
    fn global_tile_state(
        &self,
    ) -> &crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority {
        self.read_host.global_tile_state()
    }
    fn IsPendingTree(&self) -> bool {
        self.read_host.IsPendingTree(self.fields.id)
    }
    fn IsSyncTree(&self) -> bool {
        self.read_host.IsSyncTree(self.fields.id)
    }
    fn PinchGestureActive(&self) -> bool {
        self.read_host.IsPinchGestureActive()
    }
    fn device_viewport_rect_changed(&self) -> bool {
        *self.fields.device_viewport_rect_changed
    }
    fn GetDeviceViewport(&self) -> Rect {
        crate::cc::trees::layer_tree_impl::LayerTreeImpl::<B>::GetDeviceViewportForRects(
            *self.fields.device_viewport_rect,
            self.read_host.external_viewport(),
        )
    }
    fn PageScaleTransformNode(&self) -> Option<&TransformNode<B::Transform>> {
        let id = self.fields.viewport_property_ids.page_scale_transform;
        (id != crate::cc::trees::property_ids::kInvalidPropertyNodeId)
            .then(|| self.fields.property_trees.transform_tree().Node(id))
    }
    fn external_page_scale_factor(&self) -> f32 {
        *self.fields.external_page_scale_factor
    }
}
impl<B: PictureLayerImplTypes, E> PictureLayerImplTree<B> for LayerContextTree<'_, B, E> {
    fn NotifyLayerHasAnimatedImagesChanged(&mut self, id: i32, has_images: bool) {
        if has_images {
            self.tracking.with_animated_images.insert(id);
        } else {
            self.tracking.with_animated_images.remove(&id);
        }
    }
    fn AddLayerNeedingUpdateDiscardableImageMap(&mut self, id: i32, updates: &mut Vec<i32>) {
        updates.push(id);
    }
    fn image_animation_controller(
        &mut self,
    ) -> &mut crate::cc::trees::image_animation_controller::ImageAnimationController {
        self.host.image_animation_controller()
    }
    fn UpdateImageDecodingHints(&mut self, modes: PictureImageDecodingModeMap) {
        self.host.UpdateImageDecodingHints(modes)
    }
    fn BeginDiscardableImageMapUpdate(&mut self) {
        self.fields
            .BeginDiscardableImageMapUpdate(self.read_host.IsSyncTree(self.fields.id));
    }
    fn EndDiscardableImageMapUpdate(&mut self) {
        self.fields.EndDiscardableImageMapUpdate();
    }
    fn WithDiscardableImageMapUpdater(
        &mut self,
        action: &mut dyn DiscardableImageMapUpdateAction<B>,
    ) {
        crate::cc::trees::layer_tree_impl::WithScopedPictureImageMapUpdater(self, action);
    }
    fn RequestImplSideInvalidationForRerasterTiling(&mut self) {
        self.host.RequestImplSideInvalidationForRerasterTiling()
    }
    fn NotifyLayerHasPaintWorkletsChanged(&mut self, id: i32, has_worklets: bool) {
        if has_worklets {
            self.tracking.with_worklets.insert(id);
        } else {
            self.tracking.with_worklets.remove(&id);
        }
    }
    fn paint_worklet_tracker(
        &mut self,
    ) -> &mut crate::cc::trees::animated_paint_worklet_tracker::AnimatedPaintWorkletTracker<B> {
        self.host.paint_worklet_tracker()
    }
}

pub struct PictureContext<'a, B: PictureLayerImplTypes> {
    pub tree: LayerContextTree<'a, B, ()>,
    pub services: &'a mut dyn PictureLayerImplServices<B>,
    pub tiling: &'a mut dyn PictureLayerImplTilingEnvironment<B>,
}
impl<B: PictureLayerImplTypes> PictureLayerReadContext for PictureContext<'_, B> {
    fn IsRecycleTree(&self) -> bool {
        self.tree.IsRecycleTree()
    }
}
impl<B: PictureLayerImplTypes> LayerTreePictureContext<B> for PictureContext<'_, B> {
    fn WithPicture<Output>(
        &mut self,
        callback: impl FnOnce(&mut PictureLayerImplContext<'_, B>) -> Output,
    ) -> Output {
        callback(&mut PictureLayerImplContext {
            tree: &mut self.tree,
            services: self.services,
            tiling: self.tiling,
        })
    }
}

/// Source has no mutable host, services, registration or scroll capability.
/// Its resident field borrows are private and only readonly traits are exposed.
pub struct SourceContext<'a, B: PictureLayerImplTypes> {
    fields: LayerTreeLayerFields<'a, B>,
    read_host: &'a dyn PictureLayerReadHost<B>,
}
impl<'a, B: PictureLayerImplTypes> SourceContext<'a, B> {
    pub fn new(
        fields: LayerTreeLayerFields<'a, B>,
        read_host: &'a dyn PictureLayerReadHost<B>,
    ) -> Self {
        Self { fields, read_host }
    }
}
impl<B: PictureLayerImplTypes> MainLayerImplTree<B> for SourceContext<'_, B> {
    fn source_frame_number(&self) -> i32 {
        *self.fields.source_frame_number
    }
}
impl<B: PictureLayerImplTypes> PictureLayerReadContext for SourceContext<'_, B> {
    fn IsRecycleTree(&self) -> bool {
        self.read_host.IsRecycleTree(self.fields.id)
    }
}
impl<B: PictureLayerImplTypes> LayerImplReadTree<B> for SourceContext<'_, B> {
    fn id(&self) -> LayerTreeId {
        self.fields.id
    }
    fn IsActiveTree(&self) -> bool {
        self.read_host.IsActiveTree(self.fields.id)
    }
    fn TreesInVizInClientProcess(&self) -> bool {
        self.read_host.TreesInVizInClientProcess()
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.read_host.IsCCSlimmingEnabled()
    }
    fn property_trees(&self) -> &PropertyTreesFor<B> {
        self.fields.property_trees
    }
    fn settings(&self) -> &LayerTreeSettings {
        self.read_host.settings()
    }
    fn debug_state(&self) -> &crate::cc::debug::layer_tree_debug_state::LayerTreeDebugState {
        self.read_host.debug_state()
    }
    fn device_scale_factor(&self) -> f32 {
        *self.fields.device_scale_factor
    }
    fn max_texture_size(&self) -> i32 {
        self.read_host.raster_caps().max_texture_size
    }
    fn current_page_scale_factor(&self) -> f32 {
        self.fields
            .page_scale_factor
            .borrow()
            .Current(self.IsActiveTree())
    }
    fn LifecycleAllowsPropertyTreeAccess(&self) -> bool {
        self.fields.lifecycle.AllowsPropertyTreeAccess()
    }
}
impl<B: PictureLayerImplTypes> PictureLayerImplReadTree<B> for SourceContext<'_, B> {
    fn raster_caps(&self) -> &crate::cc::trees::raster_capabilities::RasterCapabilities {
        self.read_host.raster_caps()
    }
    fn GetMSAASampleCountForRaster(
        &self,
        list:&<<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource as PictureRasterSource>::DisplayItemList,
    ) -> i32 {
        self.read_host.GetMSAASampleCountForRaster(
            list,
            self.fields.id,
            self.fields.device_scale_factor,
        )
    }
    fn GetTargetColorParams(
        &self,
        usage: crate::adapter::layer_tree_host_impl::ContentColorUsage,
    ) -> B::PictureTargetColorParams {
        self.read_host
            .GetTargetColorParams(usage, self.fields.id, self.fields.display_color_spaces)
    }
    fn ViewportRectForTilePriority(&self) -> Rect {
        // Source: cc/trees/layer_tree_impl.cc:2204-2211.
        let viewport = self.read_host.ViewportRectForTilePriority();
        if viewport.IsEmpty() {
            self.GetDeviceViewport()
        } else {
            viewport
        }
    }
    fn IsReadyToActivate(&self) -> bool {
        self.read_host.IsReadyToActivate()
    }
    fn HasPendingTree(&self) -> bool {
        self.read_host.HasPendingTree()
    }
    fn GetActivelyScrollingType(
        &self,
    ) -> crate::cc::input::actively_scrolling_type::ActivelyScrollingType {
        self.read_host
            .GetActivelyScrollingType(self.fields.property_trees.scroll_tree())
    }
    fn SmoothnessTakesPriority(&self) -> bool {
        self.read_host.SmoothnessTakesPriority()
    }
    fn CurrentBeginFrameArgs(&self) -> &crate::adapter::begin_frame::BeginFrameArgs {
        self.read_host.CurrentBeginFrameArgs()
    }
    fn UseGpuRasterization(&self) -> bool {
        self.read_host.UseGpuRasterization()
    }
    fn RequiresHighResToDraw(&self) -> bool {
        self.read_host.RequiresHighResToDraw()
    }
    fn global_tile_state(
        &self,
    ) -> &crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority {
        self.read_host.global_tile_state()
    }
    fn IsPendingTree(&self) -> bool {
        self.read_host.IsPendingTree(self.fields.id)
    }
    fn IsSyncTree(&self) -> bool {
        self.read_host.IsSyncTree(self.fields.id)
    }
    fn PinchGestureActive(&self) -> bool {
        self.read_host.IsPinchGestureActive()
    }
    fn device_viewport_rect_changed(&self) -> bool {
        *self.fields.device_viewport_rect_changed
    }
    fn GetDeviceViewport(&self) -> Rect {
        crate::cc::trees::layer_tree_impl::LayerTreeImpl::<B>::GetDeviceViewportForRects(
            *self.fields.device_viewport_rect,
            self.read_host.external_viewport(),
        )
    }
    fn PageScaleTransformNode(&self) -> Option<&TransformNode<B::Transform>> {
        let id = self.fields.viewport_property_ids.page_scale_transform;
        (id != crate::cc::trees::property_ids::kInvalidPropertyNodeId)
            .then(|| self.fields.property_trees.transform_tree().Node(id))
    }
    fn external_page_scale_factor(&self) -> f32 {
        *self.fields.external_page_scale_factor
    }
}
impl<B: PictureLayerImplTypes> LayerTreeSourceContext<B> for SourceContext<'_, B> {
    fn WithBaseSource<Output>(
        &self,
        callback: impl FnOnce(&LayerImplReadContext<'_, B>) -> Output,
    ) -> Output {
        callback(&LayerImplReadContext { tree: self })
    }
    fn WithPictureSource<Output>(
        &self,
        callback: impl FnOnce(&PictureLayerSourceContext<'_, B>) -> Output,
    ) -> Output {
        callback(&PictureLayerSourceContext { tree: self })
    }
}

/// The narrow Picture factory splits native storage/tracking once and lends
/// only short host capabilities. The FnOnce slot is callback ownership only,
/// not a missing runtime capability or a saved tree/owner.
pub fn WithPictureLayersContext<B, Output>(
    tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    host: &mut dyn PictureLayerContextHost<B>,
    callback: impl FnOnce(&mut B::LayerStorage<'_>, &mut B::PictureLayerContext<'_>) -> Output,
) -> Output
where
    B: PictureLayerImplTypes
        + crate::adapter::solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImplTypes
        + LayerTreeTypes<
            Layer = super::layer::CompositorLayer<B>,
            LayerList = crate::cc::layers::layer_collections::OwnedLayerImplList<
                super::layer::CompositorLayer<B>,
            >,
        >,
    for<'a> B: LayerTreeTypes<
        LayerStorage<'a> = crate::cc::layers::layer_collections::LayerStorage<
            'a,
            super::layer::CompositorLayer<B>,
        >,
        PictureLayerContext<'a> = PictureContext<'a, B>,
    >,
    super::layer::CompositorLayer<B>: LayerTreeLayer<B>,
{
    let (layers, mut fields) = tree.SplitLayerTreeFields();
    let selected = fields.id;
    let mut callback = Some(callback);
    let mut result = None;
    host.WithPictureContextParts(selected, &mut |read_host, host, services, tiling| {
        assert!(
            result.is_none(),
            "source host field factory invokes synchronously once"
        );
        layers.WithStorageAndTracking(|storage, tracking| {
            let mut context = PictureContext {
                tree: LayerContextTree {
                    fields: fields.reborrow(),
                    tracking: tracking.reborrow(),
                    read_host,
                    host,
                    extra: (),
                },
                services,
                tiling,
            };
            result = Some(callback.take().expect("source field factory invokes once")(
                storage,
                &mut context,
            ));
        });
    });
    result.expect("source field factory must invoke its current synchronous callback")
}

use crate::adapter::scrollbar_layer_impl_base::{
    ScrollbarGeometryContext, ScrollbarGeometryStyle, ScrollbarLayerImplContext,
    ScrollbarLayerImplTree, ScrollbarOpacityContext, ScrollbarRegistrationContext,
};
use crate::cc::input::scrollbar::ScrollbarOrientation;
use crate::cc::layers::scrollbar_layer_impl_base::ScrollbarLayerImplBase;
use crate::cc::trees::layer_tree_impl::{CurrentScrollbarInitializationClient, ScrollbarLayerIds};
use std::collections::{BTreeMap, BTreeSet};

pub struct RegistrationParts<'a, B: PictureLayerImplTypes> {
    pub remaining: B::LayerRemaining<'a>,
    pub host: &'a mut dyn LayerTreeScrollbarRegistrationHost<B>,
    pub current_identity: usize,
}
/// Independent mutable leaves consumed while native controller constructors
/// adjust thumb scale. It contains no selected/other-layer lookup.
struct GeometryContext<'a, B: PictureLayerImplTypes> {
    id: LayerTreeId,
    read: &'a dyn PictureLayerReadHost<B>,
    push: &'a mut BTreeSet<i32>,
    draw: &'a mut bool,
    show: &'a mut BTreeSet<ElementId>,
}
impl<B: PictureLayerImplTypes> LayerImplPushContext for GeometryContext<'_, B> {
    type Types = B;
    fn id(&self) -> LayerTreeId {
        self.id
    }
    fn IsActiveTree(&self) -> bool {
        self.read.IsActiveTree(self.id)
    }
    fn TreesInVizInClientProcess(&self) -> bool {
        self.read.TreesInVizInClientProcess()
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.read.IsCCSlimmingEnabled()
    }
    fn AddLayerShouldPushProperties(&mut self, id: i32) {
        self.push.insert(id);
    }
    fn set_needs_update_draw_properties(&mut self) {
        *self.draw = true;
    }
}
impl<B: PictureLayerImplTypes> ScrollbarGeometryStyle for GeometryContext<'_, B> {
    fn ScrollbarAnimator(&self) -> crate::cc::trees::layer_tree_settings::ScrollbarAnimator {
        self.read.settings().scrollbar_animator
    }
    fn IsFluentOverlayScrollbarEnabled(&self) -> bool {
        self.read.settings().enable_fluent_overlay_scrollbar
    }
}
impl<B: PictureLayerImplTypes> ScrollbarGeometryContext<B> for GeometryContext<'_, B> {
    fn RequestShowScrollbars(&mut self, id: ElementId) {
        self.show.insert(id);
    }
}
impl<B: PictureLayerImplTypes, E> ScrollbarGeometryStyle for LayerContextTree<'_, B, E> {
    fn ScrollbarAnimator(&self) -> crate::cc::trees::layer_tree_settings::ScrollbarAnimator {
        self.read_host.settings().scrollbar_animator
    }
    fn IsFluentOverlayScrollbarEnabled(&self) -> bool {
        self.read_host.settings().enable_fluent_overlay_scrollbar
    }
}
impl<B: PictureLayerImplTypes, E> ScrollbarOpacityContext<B> for LayerContextTree<'_, B, E> {
    fn trees_in_viz_in_viz_process(&self) -> bool {
        self.read_host.settings().trees_in_viz_in_viz_process
    }
    fn TreeAnimationsInVizInVizProcess(&self) -> bool {
        self.read_host.TreeAnimationsInVizInVizProcess()
    }
    fn property_trees_mut(&mut self) -> &mut PropertyTreesFor<B> {
        self.fields.property_trees
    }
    fn set_needs_update_draw_properties(&mut self) {
        *self.fields.needs_update_draw_properties = true;
    }
}
impl<B: PictureLayerImplTypes> ScrollbarRegistrationContext<B>
    for LayerContextTree<'_, B, RegistrationParts<'_, B>>
{
    fn IsActiveTree(&self) -> bool {
        self.read_host.IsActiveTree(self.fields.id)
    }
    fn Registry(&self) -> &BTreeMap<ElementId, ScrollbarLayerIds> {
        self.fields.scrollbar_ids
    }
    fn RegistryMut(&mut self) -> &mut BTreeMap<ElementId, ScrollbarLayerIds> {
        self.fields.scrollbar_ids
    }
    fn PropertyTrees(&self) -> &PropertyTreesFor<B> {
        self.fields.property_trees
    }
    fn DidRegisterScrollbarLayer(&mut self, id: ElementId, orientation: ScrollbarOrientation) {
        self.extra.host.DidRegisterScrollbarLayer(id, orientation)
    }
    fn DidUnregisterScrollbarLayer(&mut self, layer: &mut ScrollbarLayerImplBase<B>) {
        let id = layer.scroll_element_id();
        let orientation = layer.orientation();
        let mut geometry = GeometryContext {
            id: self.fields.id,
            read: self.read_host,
            push: self.tracking.should_push_properties,
            draw: self.fields.needs_update_draw_properties,
            show: self.fields.show_scrollbar_requests,
        };
        let mut client = CurrentScrollbarInitializationClient {
            tree: self.fields.id,
            current_identity: self.extra.current_identity,
            current: layer,
            remaining: &mut self.extra.remaining,
            registry: self.fields.scrollbar_ids,
            geometry: &mut geometry,
        };
        self.extra
            .host
            .DidUnregisterScrollbarLayer(id, orientation, &mut client);
    }
    fn RegisterScrollbarAnimationController(
        &mut self,
        layer: &mut ScrollbarLayerImplBase<B>,
        opacity: f32,
    ) {
        let id = layer.scroll_element_id();
        let tree = ScrollbarControllerTreeFields::<B> {
            property_trees: self.fields.property_trees,
            viewport: self.fields.viewport_property_ids,
        };
        let mut geometry = GeometryContext {
            id: self.fields.id,
            read: self.read_host,
            push: self.tracking.should_push_properties,
            draw: self.fields.needs_update_draw_properties,
            show: self.fields.show_scrollbar_requests,
        };
        let mut client = CurrentScrollbarInitializationClient {
            tree: self.fields.id,
            current_identity: self.extra.current_identity,
            current: layer,
            remaining: &mut self.extra.remaining,
            registry: self.fields.scrollbar_ids,
            geometry: &mut geometry,
        };
        self.extra
            .host
            .RegisterScrollbarAnimationController(id, opacity, &tree, &mut client);
    }
}
impl<B: PictureLayerImplTypes> ScrollbarLayerImplTree<B>
    for LayerContextTree<'_, B, RegistrationParts<'_, B>>
{
    fn UnregisterScrollbar(&mut self, layer: &mut ScrollbarLayerImplBase<B>) {
        crate::cc::trees::layer_tree_impl::UnregisterScrollbarWithContext(layer, self)
    }
    fn RegisterScrollbar(&mut self, layer: &mut ScrollbarLayerImplBase<B>) {
        crate::cc::trees::layer_tree_impl::RegisterScrollbarWithContext(layer, self)
    }
    fn RequestShowScrollbars(&mut self, id: ElementId) {
        self.fields.show_scrollbar_requests.insert(id);
    }
    fn TreeAnimationsInVizInVizProcess(&self) -> bool {
        self.read_host.TreeAnimationsInVizInVizProcess()
    }
}

pub struct FullLayerContext<'a, B: PictureLayerImplTypes> {
    pub tree: LayerContextTree<'a, B, RegistrationParts<'a, B>>,
    pub services: &'a mut dyn PictureLayerImplServices<B>,
    pub tiling: &'a mut dyn PictureLayerImplTilingEnvironment<B>,
}
impl<B: PictureLayerImplTypes> PictureLayerReadContext for FullLayerContext<'_, B> {
    fn IsRecycleTree(&self) -> bool {
        self.tree.IsRecycleTree()
    }
}
impl<B: PictureLayerImplTypes> LayerTreePictureContext<B> for FullLayerContext<'_, B> {
    fn WithPicture<Output>(
        &mut self,
        callback: impl FnOnce(&mut PictureLayerImplContext<'_, B>) -> Output,
    ) -> Output {
        callback(&mut PictureLayerImplContext {
            tree: &mut self.tree,
            services: self.services,
            tiling: self.tiling,
        })
    }
}
impl<B: PictureLayerImplTypes + LayerTreeTypes<ScrollbarLayer = ScrollbarLayerImplBase<B>>>
    LayerTreeLayerContext<B> for FullLayerContext<'_, B>
{
    fn RegisterScrollbar(&mut self, layer: &mut B::ScrollbarLayer) {
        self.tree.RegisterScrollbar(layer)
    }
    fn UnregisterScrollbar(&mut self, layer: &mut B::ScrollbarLayer) {
        self.tree.UnregisterScrollbar(layer)
    }
    fn WithScrollbarOpacity<Output>(
        &mut self,
        callback: impl FnOnce(&mut dyn ScrollbarOpacityContext<B>) -> Output,
    ) -> Output {
        callback(&mut self.tree)
    }
    fn PropertyTrees(&self) -> &PropertyTreesFor<B> {
        self.tree.fields.property_trees
    }
    fn WithBase<Output>(
        &mut self,
        callback: impl FnOnce(&mut LayerImplContext<'_, B>) -> Output,
    ) -> Output {
        callback(&mut LayerImplContext {
            tree: &mut self.tree,
            services: self.services,
        })
    }
    fn WithScrollbar<Output>(
        &mut self,
        callback: impl FnOnce(&mut ScrollbarLayerImplContext<'_, B>) -> Output,
    ) -> Output {
        callback(&mut ScrollbarLayerImplContext {
            tree: &mut self.tree,
            services: self.services,
        })
    }
}

/// One native current receiver and the disjoint remaining vector slices. This
/// factory performs field projection only; source decisions remain canonical.
pub fn WithLayerContext<B, Output>(
    tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    tree_context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
    id: i32,
    callback: impl FnOnce(&mut B::Layer, &mut B::LayerContext<'_>) -> Output,
) -> Output
where
    B: PictureLayerImplTypes
        + crate::adapter::solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImplTypes
        + LayerTreeTypes<
            Layer = super::layer::CompositorLayer<B>,
            LayerList = crate::cc::layers::layer_collections::OwnedLayerImplList<
                super::layer::CompositorLayer<B>,
            >,
            ScrollbarLayer = ScrollbarLayerImplBase<B>,
        >,
    for<'a> B: LayerTreeTypes<
        LayerRemaining<'a> = crate::cc::layers::layer_collections::LayerStorageRemaining<
            'a,
            super::layer::CompositorLayer<B>,
        >,
        LayerContext<'a> = FullLayerContext<'a, B>,
    >,
    super::layer::CompositorLayer<B>: LayerTreeLayer<B>,
{
    let (layers, mut fields) = tree.SplitLayerTreeFields();
    let selected = fields.id;
    let mut callback = Some(callback);
    let mut output = None;
    tree_context.host.WithLayerContextParts(
        selected,
        &mut |read_host, host, services, tiling, registration| {
            layers.WithStorageAndTracking(|storage, tracking| {
                storage.WithLayer(id, |layer, remaining| {
                    let current_identity =
                        layer as *const super::layer::CompositorLayer<B> as usize;
                    let mut context = FullLayerContext {
                        tree: LayerContextTree {
                            fields: fields.reborrow(),
                            tracking: tracking.reborrow(),
                            read_host,
                            host,
                            extra: RegistrationParts {
                                remaining,
                                host: registration,
                                current_identity,
                            },
                        },
                        services,
                        tiling,
                    };
                    output = Some(
                        callback.take().expect("one synchronous native layer scope")(
                            layer,
                            &mut context,
                        ),
                    );
                })
            });
        },
    );
    output.expect("native factory invokes its current synchronous callback")
}

pub fn WithDetachedLayerContext<B, Output>(
    tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    tree_context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
    layer: &mut B::Layer,
    callback: impl FnOnce(&mut B::Layer, &mut B::LayerContext<'_>) -> Output,
) -> Output
where
    B: PictureLayerImplTypes
        + crate::adapter::solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImplTypes
        + LayerTreeTypes<
            Layer = super::layer::CompositorLayer<B>,
            LayerList = crate::cc::layers::layer_collections::OwnedLayerImplList<
                super::layer::CompositorLayer<B>,
            >,
            ScrollbarLayer = ScrollbarLayerImplBase<B>,
        >,
    for<'a> B: LayerTreeTypes<
        LayerRemaining<'a> = crate::cc::layers::layer_collections::LayerStorageRemaining<
            'a,
            super::layer::CompositorLayer<B>,
        >,
        LayerContext<'a> = FullLayerContext<'a, B>,
    >,
    super::layer::CompositorLayer<B>: LayerTreeLayer<B>,
{
    let (layers, mut fields) = tree.SplitLayerTreeFields();
    let selected = fields.id;
    let mut callback = Some(callback);
    let mut output = None;
    tree_context.host.WithLayerContextParts(
        selected,
        &mut |read_host, host, services, tiling, registration| {
            layers.WithStorageAndTracking(|storage, tracking| {
                storage.WithDetachedLayer(layer, |layer, remaining| {
                    let current_identity =
                        layer as *const super::layer::CompositorLayer<B> as usize;
                    let mut context = FullLayerContext {
                        tree: LayerContextTree {
                            fields: fields.reborrow(),
                            tracking: tracking.reborrow(),
                            read_host,
                            host,
                            extra: RegistrationParts {
                                remaining,
                                host: registration,
                                current_identity,
                            },
                        },
                        services,
                        tiling,
                    };
                    output = Some(callback
                        .take()
                        .expect("one synchronous detached layer scope")(
                        layer, &mut context
                    ));
                })
            });
        },
    );
    output.expect("native factory invokes its detached layer callback")
}

/// Native source and target are lent together. Source context exposes only
/// readonly source-tree facts, while target owns its actual mutation fields.
pub fn WithLayerSyncContext<B, Output>(
    source: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    target: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    tree_context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
    id: i32,
    callback: impl FnOnce(
        &mut B::Layer,
        &B::LayerSourceContext<'_>,
        &mut B::Layer,
        &mut B::LayerContext<'_>,
    ) -> Output,
) -> Output
where
    B: PictureLayerImplTypes
        + crate::adapter::solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImplTypes
        + LayerTreeTypes<
            Layer = super::layer::CompositorLayer<B>,
            LayerList = crate::cc::layers::layer_collections::OwnedLayerImplList<
                super::layer::CompositorLayer<B>,
            >,
            ScrollbarLayer = ScrollbarLayerImplBase<B>,
        >,
    for<'a> B: LayerTreeTypes<
        LayerRemaining<'a> = crate::cc::layers::layer_collections::LayerStorageRemaining<
            'a,
            super::layer::CompositorLayer<B>,
        >,
        LayerContext<'a> = FullLayerContext<'a, B>,
        LayerSourceContext<'a> = SourceContext<'a, B>,
    >,
    super::layer::CompositorLayer<B>: LayerTreeLayer<B>,
{
    let (source_layers, mut source_fields) = source.SplitLayerTreeFields();
    let (target_layers, mut target_fields) = target.SplitLayerTreeFields();
    let selected = target_fields.id;
    let mut callback = Some(callback);
    let mut output = None;
    tree_context.host.WithLayerContextParts(
        selected,
        &mut |read_host, host, services, tiling, registration| {
            source_layers.WithStorageAndTracking(|source_storage, _| {
                source_storage.WithLayer(id, |source_layer, _| {
                    target_layers.WithStorageAndTracking(|target_storage, tracking| {
                        target_storage.WithLayer(id, |target_layer, remaining| {
                            let current_identity =
                                target_layer as *const super::layer::CompositorLayer<B> as usize;
                            let source_context =
                                SourceContext::new(source_fields.reborrow(), read_host);
                            let mut target_context = FullLayerContext {
                                tree: LayerContextTree {
                                    fields: target_fields.reborrow(),
                                    tracking: tracking.reborrow(),
                                    read_host,
                                    host,
                                    extra: RegistrationParts {
                                        remaining,
                                        host: registration,
                                        current_identity,
                                    },
                                },
                                services,
                                tiling,
                            };
                            output = Some(callback
                                .take()
                                .expect("one synchronous source/target scope")(
                                source_layer,
                                &source_context,
                                target_layer,
                                &mut target_context,
                            ));
                        })
                    });
                })
            });
        },
    );
    output.expect("native factory invokes its current synchronous callback")
}

/// Constructors use the actual tree's property/push fields and services, with
/// no selected layer yet. They do not acquire scroll/registration capability.
pub fn WithLayerImplContext<B, Output>(
    tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    tree_context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
    callback: impl FnOnce(LayerTreeId, &mut LayerImplContext<'_, B>) -> Output,
) -> Output
where
    B: PictureLayerImplTypes
        + crate::adapter::solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImplTypes
        + LayerTreeTypes<
            Layer = super::layer::CompositorLayer<B>,
            LayerList = crate::cc::layers::layer_collections::OwnedLayerImplList<
                super::layer::CompositorLayer<B>,
            >,
        >,
    super::layer::CompositorLayer<B>: LayerTreeLayer<B>,
{
    let (layers, mut fields) = tree.SplitLayerTreeFields();
    let selected = fields.id;
    let mut callback = Some(callback);
    let mut output = None;
    tree_context
        .host
        .WithPictureContextParts(selected, &mut |read_host, host, services, _| {
            layers.WithStorageAndTracking(|_, tracking| {
                let mut native = LayerContextTree {
                    fields: fields.reborrow(),
                    tracking: tracking.reborrow(),
                    read_host,
                    host,
                    extra: (),
                };
                output = Some(callback.take().expect("one synchronous native ctor scope")(
                    selected,
                    &mut LayerImplContext {
                        tree: &mut native,
                        services,
                    },
                ));
            });
        });
    output.expect("native factory invokes its current synchronous callback")
}

/// The native raw current pointer resolves to the explicit receiver; every
/// other id resolves to its original disjoint vector slot. Tracking remains a
/// separate leaf of the one native LayerList owner.
struct CurrentScrollStore<'a, B: LayerTreeTypes> {
    current: &'a mut dyn LayerTreeCurrentReceiver<B>,
    remaining: &'a mut dyn LayerTreeRemainingStorage<B>,
    push: &'a mut BTreeSet<i32>,
}
impl<B: LayerTreeTypes> LayerTreeScrollStore<B> for CurrentScrollStore<'_, B> {
    fn WithScrollbarGeometry(
        &mut self,
        id: i32,
        callback: &mut dyn FnMut(&mut dyn LayerTreeScrollbarGeometry<B>, &mut BTreeSet<i32>),
    ) {
        let push = &mut *self.push;
        if id == self.current.id() {
            self.current
                .WithScrollbarGeometry(&mut |layer| callback(layer, push));
        } else {
            self.remaining
                .WithScrollbarGeometry(id, &mut |layer| callback(layer, push));
        }
    }
}
/// Property/scroll callbacks borrow only their real selected tree fields and
/// the native narrow synchronous scroll host. No Picture, registration, task,
/// input owner or synthetic default service is claimed by this context.
pub struct ScrollContextTree<'a, B: LayerImplTypes> {
    fields: LayerTreeLayerFields<'a, B>,
    tracking: LayerTracking<'a>,
    remaining: B::LayerRemaining<'a>,
    host: &'a mut dyn mutation::LayerTreeScrollHost<B>,
    debug: &'a crate::cc::debug::layer_tree_debug_state::LayerTreeDebugState,
    caps: &'a crate::cc::trees::raster_capabilities::RasterCapabilities,
}
impl<B: LayerImplTypes> MainLayerImplTree<B> for ScrollContextTree<'_, B> {
    fn source_frame_number(&self) -> i32 {
        *self.fields.source_frame_number
    }
}
impl<B: LayerImplTypes + LayerTreeTypes<Settings = LayerTreeSettings>> LayerImplReadTree<B>
    for ScrollContextTree<'_, B>
{
    fn id(&self) -> LayerTreeId {
        self.fields.id
    }
    fn IsActiveTree(&self) -> bool {
        self.host.IsActiveTree(self.fields.id)
    }
    fn TreesInVizInClientProcess(&self) -> bool {
        self.host.TreesInVizInClientProcess()
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.host.IsCCSlimmingEnabled()
    }
    fn property_trees(&self) -> &PropertyTreesFor<B> {
        self.fields.property_trees
    }
    fn settings(&self) -> &LayerTreeSettings {
        self.host.settings()
    }
    fn debug_state(&self) -> &crate::cc::debug::layer_tree_debug_state::LayerTreeDebugState {
        self.debug
    }
    fn device_scale_factor(&self) -> f32 {
        *self.fields.device_scale_factor
    }
    fn max_texture_size(&self) -> i32 {
        self.caps.max_texture_size
    }
    fn current_page_scale_factor(&self) -> f32 {
        self.fields
            .page_scale_factor
            .borrow()
            .Current(self.IsActiveTree())
    }
    fn LifecycleAllowsPropertyTreeAccess(&self) -> bool {
        self.fields.lifecycle.AllowsPropertyTreeAccess()
    }
}
impl<B: LayerImplTypes + LayerTreeTypes<Settings = LayerTreeSettings>> LayerImplTree<B>
    for ScrollContextTree<'_, B>
{
    fn property_trees_mut(&mut self) -> &mut PropertyTreesFor<B> {
        self.fields.property_trees
    }
    fn AddLayerShouldPushProperties(&mut self, id: i32) {
        self.tracking.should_push_properties.insert(id);
    }
    fn set_needs_update_draw_properties(&mut self) {
        *self.fields.needs_update_draw_properties = true;
    }
}
impl<B: LayerImplTypes + LayerTreeTypes<Settings = LayerTreeSettings>> LayerImplScrollTree<B>
    for ScrollContextTree<'_, B>
{
    fn ScrollTreeScrollBy(
        &mut self,
        current: &mut dyn LayerTreeCurrentReceiver<B>,
        node: i32,
        scroll: Vector2dF,
    ) -> Vector2dF {
        let mut storage = CurrentScrollStore {
            current,
            remaining: &mut self.remaining,
            push: self.tracking.should_push_properties,
        };
        self.fields
            .ScrollNodeByWithStore(node, &scroll, self.host, &mut storage)
    }
    fn DidUpdateScrollOffset(
        &mut self,
        current: &mut dyn LayerTreeCurrentReceiver<B>,
        element: ElementId,
        pushed: bool,
    ) {
        let mut storage = CurrentScrollStore {
            current,
            remaining: &mut self.remaining,
            push: self.tracking.should_push_properties,
        };
        self.fields
            .DidUpdateScrollOffsetWithStore(element, pushed, self.host, &mut storage)
    }
}
/// Assembly supplies the same native host's field-split readonly diagnostic
/// fields and mutable scroll callback facet. The callback cannot escape this
/// selected receiver or the remaining native storage borrow.
pub fn WithLayerScrollContext<B, Output>(
    tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    host: &mut dyn mutation::LayerTreeScrollHost<B>,
    debug: &crate::cc::debug::layer_tree_debug_state::LayerTreeDebugState,
    caps: &crate::cc::trees::raster_capabilities::RasterCapabilities,
    id: i32,
    callback: impl FnOnce(&mut B::Layer, &mut LayerImplScrollContext<'_, B>) -> Output,
) -> Output
where
    B: PictureLayerImplTypes
        + crate::adapter::solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImplTypes
        + LayerTreeTypes<
            Layer = super::layer::CompositorLayer<B>,
            LayerList = crate::cc::layers::layer_collections::OwnedLayerImplList<
                super::layer::CompositorLayer<B>,
            >,
            Settings = LayerTreeSettings,
        >,
    for<'a> B: LayerTreeTypes<
        LayerRemaining<'a> = crate::cc::layers::layer_collections::LayerStorageRemaining<
            'a,
            super::layer::CompositorLayer<B>,
        >,
    >,
    super::layer::CompositorLayer<B>: LayerTreeLayer<B>,
{
    let (layers, fields) = tree.SplitLayerTreeFields();
    layers.WithStorageAndTracking(|storage, tracking| {
        storage.WithLayer(id, |layer, remaining| {
            let mut tree = ScrollContextTree {
                fields,
                tracking: tracking.reborrow(),
                remaining,
                host,
                debug,
                caps,
            };
            callback(layer, &mut LayerImplScrollContext { tree: &mut tree })
        })
    })
}
