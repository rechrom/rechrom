//! The actual most-derived resident LayerImpl owner for the default CPU family.
//! Variants compose canonical classes; this enum stores no tree/context, keeps
//! no parallel layer state and performs only the source virtual projections.
use crate::{
    adapter::{
        layer_impl::LayerImplOwner,
        picture_layer_impl::{PictureLayerImplOwner, PictureLayerImplTypes},
        solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImplTypes,
    },
    cc::layers::{
        layer_impl::LayerImpl,
        picture_layer_impl::PictureLayerImpl,
        scrollbar_layer_impl_base::{ScrollbarLayerImplBase, ScrollbarLayerImplOwner},
        solid_color_layer_impl::SolidColorLayerImpl,
        solid_color_scrollbar_layer_impl::SolidColorScrollbarLayerImpl,
    },
};

/// Only real canonical owners appear here. Unselected external Painted and
/// NinePatch constructors are capabilities rather than fabricated variants.
pub enum CompositorLayer<B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes> {
    Base(LayerImpl<B>),
    SolidColor(SolidColorLayerImpl<B>),
    Picture(PictureLayerImpl<B>),
    SolidColorScrollbar(SolidColorScrollbarLayerImpl<B>),
}
impl<B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes> LayerImplOwner<B>
    for CompositorLayer<B>
{
    fn impl_base(&self) -> &LayerImpl<B> {
        match self {
            Self::Base(layer) => layer,
            Self::SolidColor(layer) => layer.base(),
            Self::Picture(layer) => layer.base(),
            Self::SolidColorScrollbar(layer) => layer.base(),
        }
    }
    fn impl_base_mut(&mut self) -> &mut LayerImpl<B> {
        match self {
            Self::Base(layer) => layer,
            Self::SolidColor(layer) => layer.base_mut(),
            Self::Picture(layer) => layer.base_mut(),
            Self::SolidColorScrollbar(layer) => layer.base_mut(),
        }
    }
}
impl<B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes> PictureLayerImplOwner<B>
    for CompositorLayer<B>
{
    fn picture_impl(&self) -> Option<&PictureLayerImpl<B>> {
        if let Self::Picture(layer) = self {
            Some(layer)
        } else {
            None
        }
    }
    fn picture_impl_mut(&mut self) -> Option<&mut PictureLayerImpl<B>> {
        if let Self::Picture(layer) = self {
            Some(layer)
        } else {
            None
        }
    }
}
impl<B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes> ScrollbarLayerImplOwner<B>
    for CompositorLayer<B>
{
    fn AsScrollbarBase(&self) -> Option<&ScrollbarLayerImplBase<B>> {
        if let Self::SolidColorScrollbar(layer) = self {
            Some(layer.scrollbar_base())
        } else {
            None
        }
    }
    fn AsScrollbarBaseMutable(&mut self) -> Option<&mut ScrollbarLayerImplBase<B>> {
        if let Self::SolidColorScrollbar(layer) = self {
            Some(layer.scrollbar_base_mut())
        } else {
            None
        }
    }
}

impl<B> crate::adapter::main_layer::MainLayerImpl<B> for CompositorLayer<B>
where
    B: PictureLayerImplTypes
        + SolidColorScrollbarLayerImplTypes
        + crate::adapter::layer_tree_impl::LayerTreeTypes<Layer = CompositorLayer<B>>,
    for<'a> B: crate::adapter::main_layer::MainLayerTypes<
        ImplContext<'a> = crate::adapter::layer_impl::LayerImplContext<'a, B>,
    >,
{
    fn Create(
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        id: i32,
        context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
    ) -> Box<Self> {
        B::WithLayerImplContext(tree, context, |tree_id, c| {
            LayerImpl::<B>::Create(tree_id, id, c)
        })
    }
    fn SetTransformTreeIndex(&mut self, index: i32, impl_tree: &mut B::ImplContext<'_>) {
        self.impl_base_mut().SetTransformTreeIndex(index, impl_tree);
    }
    fn SetHasTransformNode(&mut self, value: bool, impl_tree: &mut B::ImplContext<'_>) {
        self.impl_base_mut().SetHasTransformNode(value, impl_tree);
    }
    fn SetEffectTreeIndex(&mut self, index: i32, impl_tree: &mut B::ImplContext<'_>) {
        self.impl_base_mut().SetEffectTreeIndex(index, impl_tree);
    }
    fn SetClipTreeIndex(&mut self, index: i32, impl_tree: &mut B::ImplContext<'_>) {
        self.impl_base_mut().SetClipTreeIndex(index, impl_tree);
    }
    fn SetScrollTreeIndex(&mut self, index: i32, impl_tree: &mut B::ImplContext<'_>) {
        self.impl_base_mut().SetScrollTreeIndex(index, impl_tree);
    }
    fn SetElementId(&mut self, element: ElementId, impl_tree: &mut B::ImplContext<'_>) {
        self.impl_base_mut().SetElementId(element, impl_tree);
    }
    fn SetBackgroundColor(
        &mut self,
        color: crate::adapter::layer_tree_impl::Color4f,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.impl_base_mut().SetBackgroundColor(color, impl_tree);
    }
    fn SetSafeOpaqueBackgroundColor(
        &mut self,
        color: crate::adapter::layer_tree_impl::Color4f,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.impl_base_mut()
            .SetSafeOpaqueBackgroundColor(color, impl_tree);
    }
    fn SetBounds(
        &mut self,
        bounds: &crate::adapter::geometry::Size,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.impl_base_mut().SetBounds(*bounds, impl_tree);
    }
    fn SetOffsetToTransformParent(
        &mut self,
        offset: Vector2dF,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.impl_base_mut()
            .SetOffsetToTransformParent(offset, impl_tree);
    }
    fn SetDrawsContent(&mut self, value: bool, impl_tree: &mut B::ImplContext<'_>) {
        self.impl_base_mut().SetDrawsContent(value, impl_tree);
    }
    fn SetHitTestOpaqueness(
        &mut self,
        value: crate::cc::input::hit_test_opaqueness::HitTestOpaqueness,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.impl_base_mut().SetHitTestOpaqueness(value, impl_tree);
    }
    fn NoteLayerPropertyChanged(&mut self, impl_tree: &mut B::ImplContext<'_>) {
        self.impl_base_mut().NoteLayerPropertyChanged(impl_tree);
    }
    fn SetTouchActionRegion(
        &mut self,
        value: &crate::cc::layers::touch_action_region::TouchActionRegion<
            <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::RegionBackend,
        >,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.impl_base_mut()
            .SetTouchActionRegion(value.clone(), impl_tree);
    }
    fn SetContentsOpaque(&mut self, value: bool, impl_tree: &mut B::ImplContext<'_>) {
        self.impl_base_mut().SetContentsOpaque(value, impl_tree);
    }
    fn SetContentsOpaqueForText(&mut self, value: bool, impl_tree: &mut B::ImplContext<'_>) {
        self.impl_base_mut()
            .SetContentsOpaqueForText(value, impl_tree);
    }
    fn SetShouldCheckBackfaceVisibility(
        &mut self,
        value: bool,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.impl_base_mut()
            .SetShouldCheckBackfaceVisibility(value, impl_tree);
    }
    fn LifecycleAllowsPropertyTreeAccess(&self, impl_tree: &mut B::ImplContext<'_>) -> bool {
        impl_tree.tree.LifecycleAllowsPropertyTreeAccess()
    }
    fn UnionUpdateRect(&mut self, rect: &Rect, impl_tree: &mut B::ImplContext<'_>) {
        self.impl_base_mut().UnionUpdateRect(*rect, impl_tree);
    }
    fn UpdateDebugInfo(
        &mut self,
        info: Option<&mut crate::cc::layers::layer::LayerDebugInfo<B::NodeId>>,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.impl_base_mut().UpdateDebugInfo(info, impl_tree);
    }
    fn SetFilterQuality(&mut self, value: B::FilterQuality, impl_tree: &mut B::ImplContext<'_>) {
        self.impl_base_mut().SetFilterQuality(value, impl_tree);
    }
    fn SetDynamicRangeLimit(
        &mut self,
        value: B::DynamicRangeLimit,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.impl_base_mut().SetDynamicRangeLimit(value, impl_tree);
    }
    fn SetMainThreadScrollHitTestRegion(
        &mut self,
        value: &crate::cc::base::region::Region<
            <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::RegionBackend,
        >,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.impl_base_mut()
            .SetMainThreadScrollHitTestRegion(value, impl_tree);
    }
    fn SetNonCompositedScrollHitTestRects(
        &mut self,
        value: &[crate::cc::layers::scroll_hit_test_rect::ScrollHitTestRect],
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.impl_base_mut()
            .SetNonCompositedScrollHitTestRects(value, impl_tree);
    }
    fn SetCaptureBounds(
        &mut self,
        value: &B::RegionCaptureBounds,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.impl_base_mut()
            .SetCaptureBounds(value.clone(), impl_tree);
    }
    fn SetTrackedElementRects(
        &mut self,
        value: &B::TrackedElementRects,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.impl_base_mut()
            .SetTrackedElementRects(value.clone(), impl_tree);
    }
    fn SetWheelEventHandlerRegion(
        &mut self,
        value: &crate::cc::base::region::Region<
            <B as crate::adapter::layer_tree_impl::LayerTreeTypes>::RegionBackend,
        >,
        impl_tree: &mut B::ImplContext<'_>,
    ) {
        self.impl_base_mut()
            .SetWheelEventHandlerRegion(value, impl_tree);
    }
    fn SetCanvasChildId(&mut self, value: ElementId, impl_tree: &mut B::ImplContext<'_>) {
        self.impl_base_mut().SetCanvasChildId(value, impl_tree);
    }
    fn ResetRareProperties(&mut self, impl_tree: &mut B::ImplContext<'_>) {
        self.impl_base_mut().ResetRareProperties(impl_tree);
    }
    fn SetNeedsPushProperties(&mut self, value: u8, impl_tree: &mut B::ImplContext<'_>) {
        self.impl_base_mut()
            .SetNeedsPushProperties(value, impl_tree);
    }
}
use crate::adapter::{
    geometry::{Rect, Vector2dF},
    input_types::ElementId,
};

use crate::adapter::{
    damage_context::DamageLayer,
    draw_property::{DrawLayer, DrawLayerMut, DrawTransform},
    geometry::Size,
    layer_impl::LayerType,
    layer_tree_impl::{LayerTreeTypes, PropertyTreesFor},
    occlusion::OcclusionLayer,
    property_types::{MaskFilterInfo, ViewTransitionElementResourceId},
};
impl<B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes> CompositorLayer<B> {
    pub fn layer_type(&self) -> LayerType {
        match self {
            Self::Base(layer) => layer.GetLayerType(),
            Self::SolidColor(layer) => layer.GetLayerType(),
            Self::Picture(layer) => layer.GetLayerType(),
            Self::SolidColorScrollbar(layer) => layer.GetLayerType(),
        }
    }
    fn native_damage_rect(&self) -> Rect {
        match self {
            Self::Picture(layer) => layer.GetDamageRectForDispatch(),
            _ => self.impl_base().GetDamageRect(),
        }
    }
}
impl<B> DrawLayer<B::Transform> for CompositorLayer<B>
where
    B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes,
{
    type Types = B;
    fn transform_tree_index(&self) -> i32 {
        self.impl_base().transform_tree_index()
    }
    fn effect_tree_index(&self) -> i32 {
        self.impl_base().effect_tree_index()
    }
    fn clip_tree_index(&self) -> i32 {
        self.impl_base().clip_tree_index()
    }
    fn offset_to_transform_parent(&self) -> Vector2dF {
        self.impl_base().offset_to_transform_parent()
    }
    fn draws_content(&self) -> bool {
        self.impl_base().draws_content()
    }
    fn should_check_backface_visibility(&self) -> bool {
        self.impl_base().should_check_backface_visibility()
    }
    fn is_clipped(&self) -> bool {
        self.impl_base().is_clipped()
    }
    fn clip_rect(&self) -> Rect {
        self.impl_base().clip_rect()
    }
    fn visible_layer_rect(&self) -> Rect {
        self.impl_base().visible_layer_rect()
    }
    fn contributes_to_drawn_render_surface(&self) -> bool {
        self.impl_base().contributes_to_drawn_render_surface()
    }
    fn ViewTransitionResourceId(&self) -> ViewTransitionElementResourceId {
        self.impl_base().ViewTransitionResourceId()
    }
    fn DebugName(&self) -> String {
        self.impl_base().DebugName()
    }
    fn render_target_effect_tree_index(&self, trees: &PropertyTreesFor<B>) -> i32 {
        self.impl_base()
            .render_target_effect_tree_index_for_property_trees(trees)
    }
    fn render_target(&self, trees: &PropertyTreesFor<B>) -> Option<i32> {
        let id = self
            .impl_base()
            .render_target_effect_tree_index_for_property_trees(trees);
        trees.effect_tree().GetRenderSurface(id).map(|_| id)
    }
    fn bounds(&self, trees: &PropertyTreesFor<B>) -> Size {
        self.impl_base().bounds_for_property_trees(trees)
    }
    fn EnsureValidPropertyTreeIndices(&self, _trees: &PropertyTreesFor<B>) {
        self.impl_base().EnsureValidPropertyTreeIndices()
    }
    fn target_space_transform(&self, _trees: &PropertyTreesFor<B>) -> B::Transform {
        self.impl_base()
            .draw_properties()
            .target_space_transform
            .clone()
    }
    fn IsViewTransitionContentLayer(&self) -> bool {
        self.layer_type() == LayerType::kViewTransitionContent
    }
}
impl<B> DrawLayerMut<B::Transform> for CompositorLayer<B>
where
    B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes,
{
    fn SetScreenSpaceTransform(&mut self, value: B::Transform) {
        self.impl_base_mut()
            .draw_properties_mut()
            .screen_space_transform = value;
    }
    fn SetTargetSpaceTransform(&mut self, value: B::Transform) {
        self.impl_base_mut()
            .draw_properties_mut()
            .target_space_transform = value;
    }
    fn SetScreenSpaceTransformIsAnimating(&mut self, value: bool) {
        self.impl_base_mut()
            .draw_properties_mut()
            .screen_space_transform_is_animating = value;
    }
    fn SetDrawOpacity(&mut self, value: f32) {
        self.impl_base_mut().draw_properties_mut().opacity = value;
    }
    fn SetIsClipped(&mut self, value: bool) {
        self.impl_base_mut().draw_properties_mut().is_clipped = value;
    }
    fn SetClipRect(&mut self, value: Rect) {
        self.impl_base_mut().draw_properties_mut().clip_rect = value;
    }
    fn SetVisibleLayerRect(&mut self, value: Rect) {
        self.impl_base_mut()
            .draw_properties_mut()
            .visible_layer_rect = value;
    }
    fn SetVisibleDrawableContentRect(&mut self, value: Rect) {
        self.impl_base_mut()
            .draw_properties_mut()
            .visible_drawable_content_rect = value;
    }
    fn SetMaskFilterInfo(&mut self, info: MaskFilterInfo, is_fast_rounded_corner: bool) {
        let draw = self.impl_base_mut().draw_properties_mut();
        draw.mask_filter_info = info;
        draw.is_fast_rounded_corner = is_fast_rounded_corner;
    }
    fn ApplyMaskFilterInfoTransform(&mut self, transform: &B::Transform) {
        DrawTransform::ApplyMaskFilterInfo(
            transform,
            &mut self.impl_base_mut().draw_properties_mut().mask_filter_info,
        );
    }
    fn set_contributes_to_drawn_render_surface(&mut self, value: bool) {
        self.impl_base_mut()
            .set_contributes_to_drawn_render_surface(value)
    }
    fn set_raster_even_if_not_drawn(&mut self, value: bool) {
        self.impl_base_mut().set_raster_even_if_not_drawn(value)
    }
    // No represented variant is the specialized ViewTransitionContent owner;
    // the optional source subtype projection therefore remains None.
}
impl<B> DamageLayer<B::Transform, i32> for CompositorLayer<B>
where
    B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes,
    B::Transform: crate::adapter::damage_context::DamageTransform,
{
    type Trees = PropertyTreesFor<B>;
    fn id(&self) -> i32 {
        self.impl_base().id()
    }
    fn contributes_to_drawn_render_surface(&self) -> bool {
        self.impl_base().contributes_to_drawn_render_surface()
    }
    fn ViewTransitionResourceId(&self) -> ViewTransitionElementResourceId {
        self.impl_base().ViewTransitionResourceId()
    }
    fn update_rect(&self) -> Rect {
        *self.impl_base().update_rect()
    }
    fn visible_layer_rect(&self) -> Rect {
        self.impl_base().visible_layer_rect()
    }
    fn effect_tree_index(&self) -> i32 {
        self.impl_base().effect_tree_index()
    }
    fn transform_tree_index(&self) -> i32 {
        self.impl_base().transform_tree_index()
    }
    fn LayerPropertyChangedNotFromPropertyTrees(&self) -> bool {
        self.impl_base().LayerPropertyChangedNotFromPropertyTrees()
    }
    fn render_target_effect_tree_index(&self, trees: &PropertyTreesFor<B>) -> i32 {
        self.impl_base()
            .render_target_effect_tree_index_for_property_trees(trees)
    }
    fn render_target(&self, trees: &PropertyTreesFor<B>) -> i32 {
        self.impl_base()
            .render_target_effect_tree_index_for_property_trees(trees)
    }
    fn LayerPropertyChanged(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.impl_base().LayerPropertyChangedForPropertyTrees(trees)
    }
    fn bounds(&self, trees: &PropertyTreesFor<B>) -> Size {
        self.impl_base().bounds_for_property_trees(trees)
    }
    fn DrawTransform(&self, trees: &PropertyTreesFor<B>) -> B::Transform {
        self.impl_base().DrawTransformForPropertyTrees(trees)
    }
    fn LayerPropertyChangedFromPropertyTrees(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.impl_base()
            .LayerPropertyChangedFromPropertyTreesForPropertyTrees(trees)
    }
    fn GetDamageRect(&self) -> Rect {
        self.native_damage_rect()
    }
    fn GetDamageReasons(
        &self,
        trees: &PropertyTreesFor<B>,
    ) -> crate::cc::trees::damage_reason::DamageReasonSet {
        self.impl_base()
            .GetDamageReasonsForPropertyTrees(trees, || self.native_damage_rect())
    }
    fn GetEnclosingVisibleRectInTargetSpace(&self, trees: &PropertyTreesFor<B>) -> Rect {
        match self {
            Self::Picture(layer) => layer.GetEnclosingVisibleRectForPropertyTrees(trees),
            _ => self
                .impl_base()
                .GetScaledEnclosingVisibleRectInTargetSpaceForPropertyTrees(1., trees),
        }
    }
}
impl<B> OcclusionLayer<B::Transform, i32> for CompositorLayer<B>
where
    B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes,
{
    type Trees = PropertyTreesFor<B>;
    fn render_target(&self, trees: &PropertyTreesFor<B>) -> i32 {
        self.impl_base()
            .render_target_effect_tree_index_for_property_trees(trees)
    }
    fn Is3dSorted(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.impl_base().Is3dSortedForPropertyTrees(trees)
    }
    fn DrawTransform(&self, trees: &PropertyTreesFor<B>) -> B::Transform {
        self.impl_base().DrawTransformForPropertyTrees(trees)
    }
    fn effect_tree_index(&self) -> i32 {
        self.impl_base().effect_tree_index()
    }
    fn visible_layer_rect(&self) -> Rect {
        self.impl_base().visible_layer_rect()
    }
    fn is_clipped(&self) -> bool {
        self.impl_base().is_clipped()
    }
    fn clip_rect(&self) -> Rect {
        self.impl_base().clip_rect()
    }
    fn draw_opacity(&self) -> f32 {
        self.impl_base().draw_properties().opacity
    }
    fn MaskFilterInfoIsEmpty(&self) -> bool {
        self.impl_base()
            .draw_properties()
            .mask_filter_info
            .IsEmpty()
    }
    fn VisibleOpaqueRegion(&self) -> crate::cc::base::simple_enclosed_region::SimpleEnclosedRegion {
        self.impl_base().VisibleOpaqueRegion()
    }
}

use crate::adapter::layer_tree_impl::{
    LayerTreeLayer, LayerTreeLayerContext, LayerTreePictureContext, LayerTreePictureLayer,
};
use crate::cc::layers::layer_impl::{LayerImplBaseVirtual, LayerImplVirtual};
// Read-only native vtable projection used by base tracing. The source derived
// LayerType is queried on the actual owner; no tag/owner state is duplicated.
struct LayerReadVirtual<'a, B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes>(
    &'a CompositorLayer<B>,
);
impl<B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes> LayerImplVirtual<B>
    for LayerReadVirtual<'_, B>
{
    fn GetLayerType(&self, _base: &LayerImpl<B>) -> LayerType {
        self.0.layer_type()
    }
}
impl<B> LayerTreeLayer<B> for CompositorLayer<B>
where
    B: PictureLayerImplTypes
        + SolidColorScrollbarLayerImplTypes
        + LayerTreeTypes<
            Layer = CompositorLayer<B>,
            PictureLayer = PictureLayerImpl<B>,
            ScrollbarLayer = ScrollbarLayerImplBase<B>,
            PrioritizedTile = crate::cc::tiles::prioritized_tile::PrioritizedTile,
        >,
{
    fn clip_tree_index(&self) -> i32 {
        self.impl_base().clip_tree_index()
    }
    fn effect_tree_index(&self) -> i32 {
        self.impl_base().effect_tree_index()
    }
    fn transform_tree_index(&self) -> i32 {
        self.impl_base().transform_tree_index()
    }
    fn has_touch_action_regions(&self) -> bool {
        self.impl_base().has_touch_action_regions()
    }
    fn visible_layer_rect(&self) -> Rect {
        self.impl_base().visible_layer_rect()
    }
    fn id(&self) -> i32 {
        self.impl_base().id()
    }
    fn element_id(&self) -> ElementId {
        self.impl_base().element_id()
    }
    fn contributes_to_drawn_render_surface(&self) -> bool {
        self.impl_base().contributes_to_drawn_render_surface()
    }
    fn bounds(&self, trees: &PropertyTreesFor<B>) -> Size {
        self.impl_base().bounds_for_property_trees(trees)
    }
    fn ScreenSpaceTransform(&self, trees: &PropertyTreesFor<B>) -> B::Transform {
        self.impl_base().ScreenSpaceTransformForPropertyTrees(trees)
    }
    fn render_target_effect_tree_index(&self, trees: &PropertyTreesFor<B>) -> i32 {
        self.impl_base()
            .render_target_effect_tree_index_for_property_trees(trees)
    }
    fn HitTestable(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.impl_base().HitTestableForPropertyTrees(trees)
    }
    fn Is3dSorted(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.impl_base().Is3dSortedForPropertyTrees(trees)
    }
    fn GetSortingContextId(&self, trees: &PropertyTreesFor<B>) -> i32 {
        self.impl_base().GetSortingContextIdForPropertyTrees(trees)
    }
    fn OpaqueToHitTest(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.impl_base().OpaqueToHitTestForPropertyTrees(trees)
    }
    fn DrawTransform(&self, trees: &PropertyTreesFor<B>) -> B::Transform {
        self.impl_base().DrawTransformForPropertyTrees(trees)
    }
    fn LayerPropertyChanged(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.impl_base().LayerPropertyChangedForPropertyTrees(trees)
    }
    fn LayerPropertyChangedFromPropertyTrees(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.impl_base()
            .LayerPropertyChangedFromPropertyTreesForPropertyTrees(trees)
    }
    fn GetAllowedTouchAction(&self, rect: Rect) -> crate::cc::input::touch_action::TouchAction {
        self.impl_base()
            .touch_action_region()
            .GetAllowedTouchAction(&rect)
    }
    fn IsScrollerOrScrollbar(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.impl_base().IsScrollerOrScrollbarForPropertyTrees(
            matches!(self, Self::SolidColorScrollbar(_)),
            trees,
        )
    }
    fn GetAllTouchActionRegions(
        &self,
    ) -> &crate::cc::base::region::Region<<B as LayerTreeTypes>::RegionBackend> {
        self.impl_base().GetAllTouchActionRegions()
    }
    fn wheel_event_handler_region(
        &self,
    ) -> impl std::ops::Deref<
        Target = crate::cc::base::region::Region<<B as LayerTreeTypes>::RegionBackend>,
    > + '_ {
        self.impl_base().wheel_event_handler_region()
    }
    fn main_thread_scroll_hit_test_region(
        &self,
    ) -> impl std::ops::Deref<
        Target = crate::cc::base::region::Region<<B as LayerTreeTypes>::RegionBackend>,
    > + '_ {
        self.impl_base().main_thread_scroll_hit_test_region()
    }
    fn non_composited_scroll_hit_test_rects(
        &self,
    ) -> Option<&[crate::cc::layers::scroll_hit_test_rect::ScrollHitTestRect]> {
        self.impl_base()
            .non_composited_scroll_hit_test_rects()
            .map(Vec::as_slice)
    }
    fn ResetChangeTracking(&mut self) {
        match self {
            Self::Picture(layer) => layer.ResetChangeTracking(),
            _ => self.impl_base_mut().ResetChangeTracking(),
        }
    }
    fn SetOcclusionInContentSpace(
        &mut self,
        occlusion: crate::cc::trees::occlusion::Occlusion<B::Transform>,
    ) {
        self.impl_base_mut()
            .draw_properties_mut()
            .occlusion_in_content_space = occlusion;
    }
    fn SetElementId(&mut self, id: ElementId, context: &mut B::LayerContext<'_>) {
        context.WithBase(|c| self.impl_base_mut().SetElementId(id, c))
    }
    fn ReleaseResources(&mut self, context: &mut B::LayerContext<'_>) {
        match self {
            Self::Picture(layer) => context.WithPicture(|c| layer.ReleaseResources(c)),
            _ => context.WithBase(|c| self.impl_base_mut().ReleaseResources(c)),
        }
    }
    fn ReleaseTileResources(&mut self, context: &mut B::LayerContext<'_>) {
        match self {
            Self::Picture(layer) => context.WithPicture(|c| layer.ReleaseTileResources(c)),
            _ => context.WithBase(|c| self.impl_base_mut().ReleaseTileResources(c)),
        }
    }
    fn RecreateTileResources(&mut self, context: &mut B::LayerContext<'_>) {
        match self {
            Self::Picture(layer) => context.WithPicture(|c| layer.RecreateTileResources(c)),
            _ => context.WithBase(|c| self.impl_base_mut().RecreateTileResources(c)),
        }
    }
    // Inherited source LayerImpl::OnPurgeMemory calls the actual virtual
    // ReleaseResources; it must retain Picture's derived dispatch.
    fn OnPurgeMemory(&mut self, context: &mut B::LayerContext<'_>) {
        LayerTreeLayer::<B>::ReleaseResources(self, context)
    }
    fn SetInInvisibleLayerTree(&mut self) {
        LayerImplVirtual::SetInInvisibleLayerTree(&mut LayerImplBaseVirtual, self.impl_base_mut())
    }
    fn DidBecomeActive(&mut self, context: &mut B::LayerContext<'_>) {
        context.WithBase(|c| {
            LayerImplVirtual::DidBecomeActive(&mut LayerImplBaseVirtual, self.impl_base_mut(), c)
        })
    }
    fn GetAllPrioritizedTilesForTracing(
        &mut self,
        out: &mut Vec<B::PrioritizedTile>,
        context: &mut B::LayerContext<'_>,
    ) {
        match self {
            Self::Picture(layer) => {
                context.WithPicture(|c| layer.GetAllPrioritizedTilesForTracing(out, c))
            }
            _ => self.impl_base().GetAllPrioritizedTilesForTracing(out),
        }
    }
    fn AsValueInto(&self, value: &mut B::LayerTraceValue, context: &mut B::LayerContext<'_>) {
        match self {
            Self::Picture(layer) => context.WithPicture(|c| layer.AsValueInto(value, c)),
            _ => context.WithBase(|c| {
                self.impl_base()
                    .AsValueInto(&LayerReadVirtual(self), value, c)
            }),
        }
    }
    fn NoteLayerPropertyChangedFromPropertyTrees(&mut self, context: &mut B::LayerContext<'_>) {
        context.WithBase(|c| {
            self.impl_base_mut()
                .NoteLayerPropertyChangedFromPropertyTrees(c)
        })
    }
    fn set_is_inner_viewport_scroll_layer(&mut self) {
        self.impl_base_mut().set_is_inner_viewport_scroll_layer()
    }
    fn AsScrollbarLayer(&self) -> Option<&B::ScrollbarLayer> {
        self.AsScrollbarBase()
    }
    fn AsScrollbarLayerMut(&mut self) -> Option<&mut B::ScrollbarLayer> {
        self.AsScrollbarBaseMutable()
    }
    fn AsPictureLayer(&self) -> Option<&B::PictureLayer> {
        self.picture_impl()
    }
    fn AsPictureLayerMut(&mut self) -> Option<&mut B::PictureLayer> {
        self.picture_impl_mut()
    }
    fn Destroy(&mut self, context: &mut B::LayerContext<'_>) {
        use crate::adapter::layer_tree_impl::LayerTreePictureContext;
        match self {
            Self::Picture(layer) => {
                context.WithPicture(|c| layer.DestroyAfterTwinDetached(c.tiling))
            }
            Self::SolidColorScrollbar(layer) => {
                context.WithScrollbar(|c| layer.scrollbar_base_mut().Shutdown(c))
            }
            Self::Base(_) | Self::SolidColor(_) => {}
        }
    }
}
impl<B> LayerTreePictureLayer<B> for PictureLayerImpl<B>
where
    B: PictureLayerImplTypes
        + LayerTreeTypes<
            AnimatedImageDriverMap = crate::adapter::image_animation::AnimatedImageDriverMap,
            PaintImageId = crate::adapter::image_animation::PaintImageId,
        >,
{
    fn IdentityForDestruction(&self) -> crate::cc::layers::picture_layer_impl::PictureLayerImplId {
        self.identity()
    }
    fn TwinForDestruction(
        &self,
    ) -> Option<crate::cc::layers::picture_layer_impl::PictureLayerImplId> {
        PictureLayerImpl::TwinForDestruction(self)
    }
    fn ClearTwinForDestruction(
        &mut self,
    ) -> crate::cc::layers::picture_layer_impl::PictureLayerImplId {
        PictureLayerImpl::ClearTwinForDestruction(self)
    }
    fn HasValidTilePriorities(
        &self,
        context: &dyn crate::adapter::picture_layer_impl::PictureLayerReadContext,
    ) -> bool {
        self.HasValidTilePrioritiesWithReadContext(context)
    }
    fn AnnotateAnimatedImages(
        &self,
        map: &mut B::AnimatedImageDriverMap,
        context: &dyn crate::adapter::picture_layer_impl::PictureLayerReadContext,
    ) {
        self.AnnotateAnimatedImagesWithReadContext(map, context)
    }
    fn ResetRasterScale(&mut self) {
        PictureLayerImpl::ResetRasterScale(self)
    }
    fn UpdateTiles(&mut self, context: &mut B::PictureLayerContext<'_>) -> bool {
        context.WithPicture(|c| PictureLayerImpl::UpdateTiles(self, c))
    }
    fn ReleaseResources(&mut self, context: &mut B::PictureLayerContext<'_>) {
        context.WithPicture(|c| PictureLayerImpl::ReleaseResources(self, c))
    }
    fn CommitPendingRasterSource(
        &mut self,
        updates: &mut Vec<i32>,
        context: &mut B::PictureLayerContext<'_>,
    ) {
        context.WithPicture(|c| PictureLayerImpl::CommitPendingRasterSource(self, updates, c))
    }
    fn RegenerateDiscardableImageMap(&mut self, context: &mut B::PictureLayerContext<'_>) {
        context.WithPicture(|c| PictureLayerImpl::RegenerateDiscardableImageMap(self, c))
    }
    fn InvalidateRegionForImages(
        &mut self,
        images: &std::collections::BTreeSet<<B as LayerTreeTypes>::PaintImageId>,
        context: &mut B::PictureLayerContext<'_>,
    ) -> crate::adapter::layer_tree_impl::ImageInvalidationResult {
        context.WithPicture(|c| PictureLayerImpl::InvalidateRegionForImages(self, images, c))
    }
    fn InvalidateRasterInducingScrolls(
        &mut self,
        scrolls: &std::collections::BTreeSet<ElementId>,
        context: &mut B::PictureLayerContext<'_>,
    ) {
        context.WithPicture(|c| PictureLayerImpl::InvalidateRasterInducingScrolls(self, scrolls, c))
    }
}

// Native TreeSynchronizer virtual dispatch: only the actual most-derived
// owner is selected here; property-copy algorithms stay in canonical classes.
impl<B> crate::adapter::tree_sync::SyncImplLayer<B> for CompositorLayer<B>
where
    B: crate::adapter::tree_sync::SyncTypes
        + PictureLayerImplTypes
        + SolidColorScrollbarLayerImplTypes
        + crate::adapter::layer_tree_impl::LayerTreeTypes<Layer = Self>,
    Self: crate::adapter::layer_tree_impl::LayerTreeLayer<B>,
{
    fn scroll_tree_index(&self) -> i32 {
        self.impl_base().scroll_tree_index()
    }
    fn CreateLayerImpl(
        &self,
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        c: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
    ) -> Box<Self> {
        use crate::adapter::layer_tree_impl::LayerTreePictureContext;
        if let Self::Picture(picture) = self {
            B::WithPictureLayersContext(tree, c.host, |_, ctx| {
                ctx.WithPicture(|pc| picture.CreateLayerImpl(pc))
            })
        } else {
            B::WithLayerImplContext(tree, c, |id, ctx| match self {
                Self::Base(layer) => layer.CreateLayerImpl(id, ctx),
                Self::SolidColor(layer) => layer.CreateLayerImpl(id, ctx),
                Self::SolidColorScrollbar(layer) => layer.CreateLayerImpl(id, ctx),
                Self::Picture(_) => unreachable!("picture handled by native constructor above"),
            })
        }
    }
    fn MovePropertiesToActiveLayer(
        &mut self,
        target: &mut Self,
        source: &B::LayerSourceContext<'_>,
        c: &mut B::LayerContext<'_>,
    ) {
        use crate::adapter::layer_tree_impl::{
            LayerTreeLayerContext, LayerTreePictureContext, LayerTreeSourceContext,
        };
        match self {
            Self::Picture(layer) => {
                let Self::Picture(target) = target else {
                    panic!("native PictureLayerImpl target subtype")
                };
                source.WithPictureSource(|sc| {
                    c.WithPicture(|tc| layer.MovePropertiesToActiveLayer(target, sc, tc))
                });
            }
            Self::SolidColorScrollbar(layer) => {
                let Self::SolidColorScrollbar(target) = target else {
                    panic!("native SolidColorScrollbarLayerImpl target subtype")
                };
                source.WithBaseSource(|sc| {
                    c.WithScrollbar(|tc| layer.CopyPropertiesTo(target, sc, tc))
                });
            }
            _ => source.WithBaseSource(|sc| {
                c.WithBase(|tc| {
                    self.impl_base()
                        .CopyPropertiesTo(target.impl_base_mut(), sc, tc)
                })
            }),
        }
    }
}
impl<B: crate::adapter::tree_sync::SyncTypes + PictureLayerImplTypes>
    crate::adapter::tree_sync::SyncPictureLayer for PictureLayerImpl<B>
{
    fn id(&self) -> i32 {
        self.base().id()
    }
}

// The CPU family's only represented scrollbar subtype is SolidColorScrollbar,
// whose source GetScrollbarAnimator inherits the canonical base style query.
// Painted/NinePatch owners have no fabricated projection into this family.
impl<B: crate::adapter::layer_impl::LayerImplTypes>
    crate::adapter::layer_tree_impl::LayerTreeScrollbarGeometry<B> for ScrollbarLayerImplBase<B>
{
    fn is_overlay_scrollbar(&self) -> bool {
        self.is_overlay_scrollbar()
    }
    fn thumb_thickness_scale_factor(&self) -> f32 {
        self.thumb_thickness_scale_factor()
    }
    fn SetThumbThicknessScaleFactor(
        &mut self,
        value: f32,
        c: &mut dyn crate::adapter::scrollbar_layer_impl_base::ScrollbarGeometryContext<B>,
    ) {
        let (base, mut fields) = self.SplitGeometryFields();
        fields.SetThumbThicknessScaleFactor(base, value, c);
    }

    fn orientation(&self) -> crate::cc::input::scrollbar::ScrollbarOrientation {
        self.orientation()
    }
    fn SetCurrentPos(
        &mut self,
        value: f32,
        c: &mut dyn crate::adapter::scrollbar_layer_impl_base::ScrollbarGeometryContext<B>,
    ) {
        let (base, mut fields) = self.SplitGeometryFields();
        fields.SetCurrentPos(base, value, c);
    }
    fn SetClipLayerLength(
        &mut self,
        value: f32,
        c: &mut dyn crate::adapter::scrollbar_layer_impl_base::ScrollbarGeometryContext<B>,
    ) {
        let (base, mut fields) = self.SplitGeometryFields();
        fields.SetClipLayerLength(base, value, c, |c| c.ScrollbarAnimator());
    }
    fn SetScrollLayerLength(
        &mut self,
        value: f32,
        c: &mut dyn crate::adapter::scrollbar_layer_impl_base::ScrollbarGeometryContext<B>,
    ) {
        let (base, mut fields) = self.SplitGeometryFields();
        fields.SetScrollLayerLength(base, value, c, |c| c.ScrollbarAnimator());
    }
    fn SetVerticalAdjust(
        &mut self,
        value: f32,
        c: &mut dyn crate::adapter::scrollbar_layer_impl_base::ScrollbarGeometryContext<B>,
    ) {
        let (base, mut fields) = self.SplitGeometryFields();
        fields.SetVerticalAdjust(base, value, c);
    }
}
impl<B: crate::adapter::layer_impl::LayerImplTypes>
    crate::adapter::layer_tree_impl::LayerTreeScrollbar<B> for ScrollbarLayerImplBase<B>
{
    fn id(&self) -> i32 {
        self.base().id()
    }

    fn scroll_element_id(&self) -> crate::adapter::input_types::ElementId {
        self.scroll_element_id()
    }
    fn GetScrollbarAnimator(
        &self,
        c: &dyn crate::adapter::scrollbar_layer_impl_base::ScrollbarGeometryStyle,
    ) -> crate::cc::trees::layer_tree_settings::ScrollbarAnimator {
        self.GetScrollbarAnimatorWithStyle(c)
    }
    fn IsFluentOverlayScrollbarEnabled(
        &self,
        c: &dyn crate::adapter::scrollbar_layer_impl_base::ScrollbarGeometryStyle,
    ) -> bool {
        self.IsFluentOverlayScrollbarEnabledWithStyle(c)
    }
    fn SetOverlayScrollbarLayerOpacityAnimated(
        &mut self,
        opacity: f32,
        fade_out: bool,
        c: &mut dyn crate::adapter::scrollbar_layer_impl_base::ScrollbarOpacityContext<B>,
    ) {
        self.SetOverlayScrollbarLayerOpacityAnimatedWithContext(opacity, fade_out, c);
    }
    fn Opacity(&self, p: &crate::adapter::layer_tree_impl::PropertyTreesFor<B>) -> f32 {
        self.base().OpacityForPropertyTrees(p)
    }
    fn has_find_in_page_tickmarks(&self) -> bool {
        self.has_find_in_page_tickmarks()
    }
}

impl<B: PictureLayerImplTypes + SolidColorScrollbarLayerImplTypes> CompositorLayer<B> {
    pub fn ScrollBy(
        &mut self,
        scroll: crate::adapter::geometry::Vector2dF,
        c: &mut crate::adapter::layer_impl::LayerImplScrollContext<'_, B>,
    ) -> crate::adapter::geometry::Vector2dF {
        if let Self::SolidColorScrollbar(layer) = self {
            let (base, mut geometry) = layer.scrollbar_base_mut().SplitGeometryFields();
            base.ScrollByWithGeometry(scroll, Some(&mut geometry), c)
        } else {
            self.impl_base_mut().ScrollBy(scroll, c)
        }
    }
    pub fn SetCurrentScrollOffset(
        &mut self,
        offset: crate::adapter::geometry::PointF,
        c: &mut crate::adapter::layer_impl::LayerImplScrollContext<'_, B>,
    ) {
        if let Self::SolidColorScrollbar(layer) = self {
            let (base, mut geometry) = layer.scrollbar_base_mut().SplitGeometryFields();
            base.SetCurrentScrollOffsetWithGeometry(offset, Some(&mut geometry), c)
        } else {
            self.impl_base_mut().SetCurrentScrollOffset(offset, c)
        }
    }
}

impl<B: PictureLayerImplTypes> crate::adapter::layer_tree_host_impl::HostTilePictureLayer
    for crate::cc::layers::picture_layer_impl::PictureLayerImpl<B>
{
    fn ResidentDisplayTiling(
        &self,
        id: crate::adapter::tile::PictureLayerTilingId,
    ) -> Option<&dyn crate::adapter::layer_tree_host_impl::HostDisplayTileTiling> {
        self.picture_layer_tiling_set()
            .TilingByIdIfPresent(id)
            .map(|tiling| {
                &**tiling as &dyn crate::adapter::layer_tree_host_impl::HostDisplayTileTiling
            })
    }
    fn should_batch_updated_tiles(&self) -> bool {
        self.should_batch_updated_tiles()
    }
    fn ResidentTilingTree(
        &self,
        id: crate::adapter::tile::PictureLayerTilingId,
    ) -> Option<crate::cc::tiles::tile_priority::WhichTree> {
        self.picture_layer_tiling_set()
            .TilingByIdIfPresent(id)
            .map(|tiling| tiling.tree())
    }
}
impl<B> crate::adapter::layer_tree_host_impl::HostTileLayer<B> for CompositorLayer<B>
where
    B: PictureLayerImplTypes
        + SolidColorScrollbarLayerImplTypes
        + crate::adapter::layer_tree_host_impl::HostImplTypes
        + crate::adapter::layer_tree_host_impl::HostImplCTypes<Tile = crate::cc::tiles::tile::Tile<B>>,
{
    fn NotifyTileStateChanged(
        &mut self,
        tile: &B::Tile,
        damage: bool,
        c: &mut B::PictureLayerContext<'_>,
    ) {
        use crate::adapter::layer_tree_impl::LayerTreePictureContext;
        match self {
            Self::Picture(layer) => c.WithPicture(|context| {
                layer.NotifyTileStateChanged(tile, damage, &mut context.BaseContext())
            }),
            // Source cc/layers/layer_impl.h:169 base virtual is empty.
            _ => {}
        }
    }
}

use super::metadata::{
    RegionCaptureBounds, TrackedElementFeature, TrackedElementRect, TrackedElementRects,
};
use crate::adapter::layer_tree_host_impl::{
    HostDrawLayer, HostImplDTypes, HostImplTypes, HostMetadataLayer,
};
use crate::adapter::main_layer::MainLayerTypes;
use crate::adapter::render_metadata::RenderFrameMetadataTypes;
type TrackedFlagIter<'a> =
    std::iter::Map<std::slice::Iter<'a, TrackedElementRect>, fn(&TrackedElementRect) -> bool>;
type TrackedListIter<'a> = std::iter::Flatten<
    std::option::IntoIter<
        std::iter::Map<
            std::collections::btree_map::Values<'a, TrackedElementFeature, Vec<TrackedElementRect>>,
            fn(&Vec<TrackedElementRect>) -> TrackedFlagIter<'_>,
        >,
    >,
>;
fn tracked_flag(rect: &TrackedElementRect) -> bool {
    rect.should_exclude_fixed_and_sticky_occlusions
}
fn tracked_flags(rects: &Vec<TrackedElementRect>) -> TrackedFlagIter<'_> {
    rects
        .iter()
        .map(tracked_flag as fn(&TrackedElementRect) -> bool)
}

impl<B> HostDrawLayer<B> for CompositorLayer<B>
where
    B: PictureLayerImplTypes
        + SolidColorScrollbarLayerImplTypes
        + HostImplTypes
        + MainLayerTypes<TrackedElementRects = TrackedElementRects>,
{
    type TrackedRectLists<'a>
        = TrackedListIter<'a>
    where
        Self: 'a;
    type TrackedRectList<'a>
        = TrackedFlagIter<'a>
    where
        Self: 'a;
    fn tracked_element_rects_empty(&self) -> Option<bool> {
        self.impl_base()
            .tracked_element_rects()
            .map(|rects| rects.0.is_empty())
    }
    fn tracked_element_rects(&self) -> Self::TrackedRectLists<'_> {
        self.impl_base()
            .tracked_element_rects()
            .map(|rects| {
                rects
                    .0
                    .values()
                    .map(tracked_flags as fn(&Vec<TrackedElementRect>) -> TrackedFlagIter<'_>)
            })
            .into_iter()
            .flatten()
    }
    fn WillDraw(
        &mut self,
        mode: crate::cc::layers::draw_mode::DrawMode,
        c: &mut B::PictureLayerContext<'_>,
    ) -> bool {
        use crate::adapter::layer_tree_impl::LayerTreePictureContext;
        c.WithPicture(|context| self.impl_base().WillDraw(mode, &context.BaseContext()))
    }
    fn layer_tree_impl_id(&self) -> crate::adapter::layer_tree_impl::LayerTreeId {
        self.impl_base().tree_id()
    }
    fn NotifyKnownResourceIdsBeforeAppendQuads(
        &mut self,
        ids: &std::collections::BTreeSet<
            crate::adapter::property_types::ViewTransitionElementResourceId,
        >,
    ) {
        use crate::cc::layers::layer_impl::{LayerImplBaseVirtual, LayerImplVirtual};
        LayerImplBaseVirtual.NotifyKnownResourceIdsBeforeAppendQuads(self.impl_base_mut(), ids);
    }
    fn AppendQuads(
        &mut self,
        append: &crate::cc::layers::append_quads_context::AppendQuadsContext,
        pass: &mut B::RenderPass,
        data: &mut crate::cc::layers::append_quads_data::AppendQuadsData<B::SurfaceId>,
        c: &mut B::PictureLayerContext<'_>,
    ) {
        use crate::adapter::layer_tree_impl::LayerTreePictureContext;
        c.WithPicture(|context| match self {
            Self::Picture(layer) => layer.AppendQuads(append, pass, data, context),
            Self::SolidColor(layer) => {
                layer.AppendQuads(append, pass, data, &mut context.BaseContext())
            }
            Self::SolidColorScrollbar(layer) => {
                layer.AppendQuads(append, pass, data, &mut context.BaseContext())
            }
            Self::Base(layer) => {
                use crate::cc::layers::layer_impl::{LayerImplBaseVirtual, LayerImplVirtual};
                LayerImplBaseVirtual.AppendQuads(
                    layer,
                    append,
                    pass,
                    data,
                    &mut context.BaseContext(),
                );
            }
        });
    }
    fn HasMissingTiles(&self, c: &mut B::PictureLayerContext<'_>) -> bool {
        use crate::adapter::layer_tree_impl::LayerTreePictureContext;
        match self {
            Self::Picture(layer) => {
                c.WithPicture(|context| layer.HasMissingTiles(&context.BaseContext()))
            }
            _ => self.impl_base().HasMissingTiles(),
        }
    }
    fn screen_space_transform_is_animating(&self) -> bool {
        self.impl_base().screen_space_transform_is_animating()
    }
    fn ComputeCheckerboardedNeedsRecord(&mut self, c: &mut B::PictureLayerContext<'_>) -> bool {
        use crate::adapter::layer_tree_impl::LayerTreePictureContext;
        match self {
            Self::Picture(layer) => {
                c.WithPicture(|context| layer.ComputeCheckerboardedNeedsRecord(context))
            }
            _ => self.impl_base_mut().ComputeCheckerboardedNeedsRecord(),
        }
    }
}
impl<B> HostMetadataLayer<B> for CompositorLayer<B>
where
    B: PictureLayerImplTypes
        + SolidColorScrollbarLayerImplTypes
        + HostImplTypes
        + MainLayerTypes<
            TrackedElementRects = TrackedElementRects,
            RegionCaptureBounds = RegionCaptureBounds,
        > + RenderFrameMetadataTypes<TrackedElementRects = TrackedElementRects>
        + HostImplDTypes<RegionCaptureBounds = RegionCaptureBounds>,
{
    fn tracked_element_rects_value(&self) -> Option<&TrackedElementRects> {
        self.impl_base().tracked_element_rects()
    }
    fn capture_bounds(&self) -> Option<&RegionCaptureBounds> {
        self.impl_base().capture_bounds()
    }
    fn draw_opacity(&self) -> f32 {
        self.impl_base().draw_opacity()
    }
    fn DidBeginTracing(&mut self) {
        match self {
            Self::Picture(layer) => layer.DidBeginTracing(),
            _ => self.impl_base_mut().DidBeginTracing(),
        }
    }
    fn DidDraw(&mut self, c: &mut B::PictureLayerContext<'_>) {
        use crate::adapter::layer_tree_impl::LayerTreePictureContext;
        c.WithPicture(|context| match self {
            Self::Picture(layer) => layer.DidDraw(context),
            _ => {
                use crate::cc::layers::layer_impl::{LayerImplBaseVirtual, LayerImplVirtual};
                LayerImplBaseVirtual.DidDraw(self.impl_base_mut(), &mut context.BaseContext());
            }
        });
    }
}

impl<B> crate::adapter::picture_layer::PictureLayerImplTarget<B> for CompositorLayer<B>
where
 B:crate::adapter::picture_layer::PictureLayerTypes+PictureLayerImplTypes+SolidColorScrollbarLayerImplTypes+crate::adapter::layer_tree_impl::LayerTreeTypes<Layer=CompositorLayer<B>>,
 B::Recording:crate::adapter::recording_source::RecordingSourceTypes<RasterSource=<B as crate::adapter::picture_layer_tiling::PictureLayerTilingTypes>::RasterSource>,
 for<'a>B:MainLayerTypes<ImplContext<'a>=crate::adapter::layer_impl::LayerImplContext<'a,B>>,
{
 fn CreatePicture(tree:&mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,id:i32,context:&mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_,B>)->Box<Self>{
  use crate::adapter::layer_tree_impl::LayerTreePictureContext;
  let tree_id=tree.id;
  B::WithPictureLayersContext(tree,context.host,|_,c|c.WithPicture(|c|PictureLayerImpl::<B>::Create(tree_id,id,c)))
 }
 fn set_has_non_animated_image_update_rect(&mut self){self.picture_impl_mut().expect("native Picture target").set_has_non_animated_image_update_rect()}
 fn set_gpu_raster_max_texture_size(&mut self,size:crate::adapter::geometry::Size,tree:&mut B::ImplContext<'_>){self.picture_impl_mut().expect("native Picture target").set_gpu_raster_max_texture_size(size,tree)}
 fn SetIsBackdropFilterMask(&mut self,value:bool,tree:&mut B::ImplContext<'_>){self.picture_impl_mut().expect("native Picture target").SetIsBackdropFilterMask(value,tree)}
 fn StageNewRasterSourceForCommit(&mut self,source:std::sync::Arc<<B::Recording as crate::adapter::recording_source::RecordingSourceTypes>::RasterSource>,invalidation:crate::cc::base::region::Region<<B as crate::adapter::layer_tree_impl::LayerTreeTypes>::RegionBackend>){self.picture_impl_mut().expect("native Picture target").StageNewRasterSourceForCommit(source,invalidation)}
 fn set_should_batch_updated_tiles(&mut self){self.picture_impl_mut().expect("native Picture target").set_should_batch_updated_tiles()}
}
