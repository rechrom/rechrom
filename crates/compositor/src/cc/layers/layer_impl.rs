// Translation status: complete
// Copyright 2011 The Chromium Authors
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/layer_impl.h:1-65; cc/layers/layer_impl.cc:1-54.
use crate::{
    adapter::{
        geometry::{PointF, Rect, Size, SizeF, Vector2dF},
        input_types::ElementId,
        layer_impl::*,
        layer_tree_impl::{Color4f, ElementListType, LayerTreeId, PropertyTreesFor, ResourceId},
        main_layer::LayerValue,
        occlusion::SkBlendMode,
    },
    cc::{
        base::{region::Region, simple_enclosed_region::SimpleEnclosedRegion},
        input::hit_test_opaqueness::HitTestOpaqueness,
        layers::{
            draw_properties::DrawProperties, layer::LayerDebugInfo,
            scroll_hit_test_rect::ScrollHitTestRect, touch_action_region::TouchActionRegion,
        },
        trees::{
            damage_reason::{DamageReason, DamageReasonSet},
            property_ids::kInvalidPropertyNodeId,
        },
    },
};
use std::cell::OnceCell;
// Source: cc/layers/layer_impl.h:66-73.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewportLayerType {
    NOT_VIEWPORT_LAYER,
    INNER_VIEWPORT_CONTAINER,
    OUTER_VIEWPORT_CONTAINER,
    INNER_VIEWPORT_SCROLL,
    OUTER_VIEWPORT_SCROLL,
}
impl ViewportLayerType {
    pub const LAST_VIEWPORT_LAYER_TYPE: Self = Self::OUTER_VIEWPORT_SCROLL;
}
// Source: cc/layers/layer_impl.cc:54-57.
fn ClonePtr<T: Clone>(value: &Option<Box<T>>) -> Option<Box<T>> {
    value.as_ref().map(|value| Box::new((**value).clone()))
}
// Source: cc/layers/layer_impl.cc:59-92.
pub fn LayerTypeAsString(value: LayerType) -> &'static str {
    match value {
        LayerType::kLayer => "cc::LayerImpl",
        LayerType::kSolidColor => "cc::SolidColorLayerImpl",
        LayerType::kTexture => "cc::TextureLayerImpl",
        LayerType::kSurface => "cc::SurfaceLayerImpl",
        LayerType::kPicture => "cc::PictureLayerImpl",
        LayerType::kTileDisplay => "cc::TileDisplayLayerImpl",
        LayerType::kMirror => "cc::MirrorLayerImpl",
        LayerType::kHeadsUpDisplay => "cc::HeadsUpDisplayLayerImpl",
        LayerType::kUIResource => "cc::UIResourceLayerImpl",
        LayerType::kNinePatch => "cc::NinePatchLayerImpl",
        LayerType::kSolidColorScrollbar => "cc::SolidColorScrollbarLayerImpl",
        LayerType::kPaintedScrollbar => "cc::PaintedScrollbarLayerImpl",
        LayerType::kNinePatchThumbScrollbar => "cc::NinePatchThumbScrollbarLayerImpl",
        LayerType::kVideo => "cc::VideoLayerImpl",
        LayerType::kViewTransitionContent => "cc::ViewTransitionContentLayerImpl",
    }
}
// Source: cc/layers/layer_impl.h:284-301; cc/layers/layer_impl.cc:98-100.
pub struct RareProperties<B: LayerImplTypes> {
    pub capture_bounds: B::RegionCaptureBounds,
    pub tracked_element_rects: B::TrackedElementRects,
    pub main_thread_scroll_hit_test_region: Region<B::RegionBackend>,
    pub non_composited_scroll_hit_test_rects: Vec<ScrollHitTestRect>,
    pub wheel_event_handler_region: Region<B::RegionBackend>,
    pub canvas_child_id: ElementId,
    pub filter_quality: B::FilterQuality,
    pub dynamic_range_limit: B::DynamicRangeLimit,
}
impl<B: LayerImplTypes> Default for RareProperties<B> {
    fn default() -> Self {
        Self {
            capture_bounds: Default::default(),
            tracked_element_rects: Default::default(),
            main_thread_scroll_hit_test_region: Default::default(),
            non_composited_scroll_hit_test_rects: Vec::new(),
            wheel_event_handler_region: Default::default(),
            canvas_child_id: Default::default(),
            filter_quality: Default::default(),
            dynamic_range_limit: Default::default(),
        }
    }
}
impl<B: LayerImplTypes> Clone for RareProperties<B> {
    fn clone(&self) -> Self {
        Self {
            capture_bounds: self.capture_bounds.clone(),
            tracked_element_rects: self.tracked_element_rects.clone(),
            main_thread_scroll_hit_test_region: self.main_thread_scroll_hit_test_region.clone(),
            non_composited_scroll_hit_test_rects: self.non_composited_scroll_hit_test_rects.clone(),
            wheel_event_handler_region: self.wheel_event_handler_region.clone(),
            canvas_child_id: self.canvas_child_id,
            filter_quality: self.filter_quality,
            dynamic_range_limit: self.dynamic_range_limit,
        }
    }
}
// Source: cc/layers/layer_impl.h:601-680. Every field is resident source state.
// The native nonowning layer_tree_impl_ is only an identity; actual tree fields
// are borrowed synchronously through LayerImplContext and never cached here.
pub struct LayerImpl<B: LayerImplTypes> {
    layer_id_: i32,
    layer_tree_impl_: LayerTreeId,
    stable_id_for_shared_quad_state_: i32,
    bounds_: Size,
    offset_to_transform_parent_: Vector2dF,
    layer_property_changed_not_from_property_trees_: bool,
    layer_property_changed_from_property_trees_: bool,
    contents_opaque_: bool,
    contents_opaque_for_text_: bool,
    should_check_backface_visibility_: bool,
    draws_content_: bool,
    contributes_to_drawn_render_surface_: bool,
    is_inner_viewport_scroll_layer_: bool,
    hit_test_opaqueness_: HitTestOpaqueness,
    touch_action_region_: TouchActionRegion<B::RegionBackend>,
    background_color_: Color4f,
    safe_opaque_background_color_: Color4f,
    transform_tree_index_: i32,
    effect_tree_index_: i32,
    clip_tree_index_: i32,
    scroll_tree_index_: i32,
    rare_properties_: Option<Box<RareProperties<B>>>,
    element_id_: ElementId,
    update_rect_: Rect,
    draw_properties_: DrawProperties<B::Transform>,
    debug_info_: Option<Box<std::sync::RwLock<LayerDebugInfo<B::NodeId>>>>,
    all_touch_action_regions_: OnceCell<Box<Region<B::RegionBackend>>>,
    changed_properties_: u8,
    needs_push_properties_: bool,
    raster_even_if_not_drawn_: bool,
    has_transform_node_: bool,
    deletion_trace_: B::DeletionTrace,
}
impl<B: LayerImplTypes> LayerImpl<B> {
    // Source: cc/layers/layer_impl.h:564-570.
    pub const kChangedPropertyTreeIndex: u8 = 1 << 0;
    pub const kChangedGeneralProperty: u8 = 1 << 1;
    pub const kChangedTile: u8 = 1 << 2;
    pub const kChangedAllProperties: u8 =
        Self::kChangedPropertyTreeIndex | Self::kChangedGeneralProperty | Self::kChangedTile;
    // Source: cc/layers/layer_impl.cc:102-105.
    pub fn GetNextStableIdForSharedQuadState(services: &dyn LayerImplServices<B>) -> i32 {
        Self::GetNextStableIdForSharedQuadStateFromSequence(services.NextStableIdSequenceValue())
    }
    // Source: cc/layers/layer_impl.cc:103-105. The injected global sequence
    // operation occurs before this single canonical arithmetic body.
    pub(crate) fn GetNextStableIdForSharedQuadStateFromSequence(next: i32) -> i32 {
        next + 1
    }
    // Source: cc/layers/layer_impl.cc:107-123.
    pub fn new(tree: LayerTreeId, id: i32, c: &mut LayerImplContext<'_, B>) -> Self {
        let stable = Self::GetNextStableIdForSharedQuadState(c.services);
        debug_assert!(id > 0);
        debug_assert!(stable > 0);
        debug_assert_eq!(tree, c.tree.id());
        let mut result = Self {
            layer_id_: id,
            layer_tree_impl_: tree,
            stable_id_for_shared_quad_state_: stable,
            bounds_: Size::default(),
            offset_to_transform_parent_: Vector2dF::default(),
            layer_property_changed_not_from_property_trees_: false,
            layer_property_changed_from_property_trees_: false,
            contents_opaque_: false,
            contents_opaque_for_text_: false,
            should_check_backface_visibility_: false,
            draws_content_: false,
            contributes_to_drawn_render_surface_: false,
            is_inner_viewport_scroll_layer_: false,
            hit_test_opaqueness_: HitTestOpaqueness::kTransparent,
            touch_action_region_: Default::default(),
            background_color_: Color4f::default(),
            safe_opaque_background_color_: Color4f::default(),
            transform_tree_index_: kInvalidPropertyNodeId,
            effect_tree_index_: kInvalidPropertyNodeId,
            clip_tree_index_: kInvalidPropertyNodeId,
            scroll_tree_index_: kInvalidPropertyNodeId,
            rare_properties_: None,
            element_id_: ElementId::default(),
            update_rect_: Rect::default(),
            draw_properties_: DrawProperties::default(),
            debug_info_: None,
            all_touch_action_regions_: OnceCell::new(),
            changed_properties_: 0,
            needs_push_properties_: false,
            raster_even_if_not_drawn_: false,
            has_transform_node_: false,
            deletion_trace_: c.services.DeletionTrace(),
        };
        if !c.tree.IsActiveTree() || !c.tree.TreesInVizInClientProcess() {
            result.changed_properties_ = Self::kChangedAllProperties;
        }
        result
    }
    // Source: cc/layers/layer_impl.h:77-79.
    pub fn Create(tree: LayerTreeId, id: i32, c: &mut LayerImplContext<'_, B>) -> Box<B::Layer> {
        Box::new(B::WrapLayerImpl(Self::new(tree, id, c)))
    }
    // Source: cc/layers/layer_impl.cc:131-133.
    pub fn GetLayerType(&self) -> LayerType {
        LayerType::kLayer
    }
    // Source: cc/layers/layer_impl.cc:135-137.
    pub fn GetElementTypeForAnimation(&self, c: &LayerImplContext<'_, B>) -> ElementListType {
        if self.IsActive(c) {
            ElementListType::ACTIVE
        } else {
            ElementListType::PENDING
        }
    }
    // Source: cc/layers/layer_impl.h:88-88.
    pub fn id(&self) -> i32 {
        self.layer_id_
    }
    // Source: cc/layers/layer_impl.h:90-92.
    pub fn stable_id_for_shared_quad_state(&self) -> i32 {
        self.stable_id_for_shared_quad_state_
    }
    /// Validates the native nonowning pointer's live current allocation identity.
    pub fn tree_id(&self) -> LayerTreeId {
        self.layer_tree_impl_
    }
    fn CheckTree(&self, c: &LayerImplContext<'_, B>) {
        debug_assert_eq!(self.layer_tree_impl_, c.tree.id());
    }
    // Source: cc/layers/layer_impl.cc:794-810.
    pub fn SetNeedsPushProperties(&mut self, changed: u8, c: &mut LayerImplContext<'_, B>) {
        self.SplitTileClient().0.SetNeedsPushProperties(changed, c);
    }
    /// Short borrows of the actual fields; draw properties remain disjoint
    /// while a native tiling callback writes only source push-property leaves.
    pub(crate) fn SplitTileClient(
        &mut self,
    ) -> (LayerImplTileClient<'_>, &DrawProperties<B::Transform>) {
        (
            LayerImplTileClient {
                id: &self.layer_id_,
                tree: &self.layer_tree_impl_,
                changed: &mut self.changed_properties_,
                needs_push: &mut self.needs_push_properties_,
                opaque: &self.contents_opaque_,
                contributes: &self.contributes_to_drawn_render_surface_,
                raster_even_if_not_drawn: &self.raster_even_if_not_drawn_,
            },
            &self.draw_properties_,
        )
    }
    // Source: cc/layers/layer_impl.cc:139-159.
    pub fn UpdateDebugInfo(
        &mut self,
        info: Option<&mut LayerDebugInfo<B::NodeId>>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
        let Some(info) = info else {
            self.debug_info_ = None;
            return;
        };
        let new_invalidations = std::mem::take(&mut info.invalidations);
        if self.debug_info_.is_none() {
            let mut value = Box::new(info.clone());
            value.invalidations = new_invalidations;
            self.debug_info_ = Some(Box::new(std::sync::RwLock::new(*value)));
            return;
        }
        let mut value = self.debug_info_.as_mut().unwrap().write().unwrap();
        let existing = std::mem::take(&mut value.invalidations);
        *value = info.clone();
        value.invalidations.splice(0..0, existing);
    }
    // Source: cc/layers/layer_impl.cc:161-168.
    pub fn SetHasTransformNode(&mut self, value: bool, c: &mut LayerImplContext<'_, B>) {
        if self.has_transform_node_ == value {
            return;
        }
        self.has_transform_node_ = value;
        self.SetNeedsPushProperties(Self::kChangedPropertyTreeIndex, c);
    }
    // Source: cc/layers/layer_impl.cc:170-177.
    pub fn SetTransformTreeIndex(&mut self, value: i32, c: &mut LayerImplContext<'_, B>) {
        if self.transform_tree_index_ == value {
            return;
        }
        self.transform_tree_index_ = value;
        self.SetNeedsPushProperties(Self::kChangedPropertyTreeIndex, c);
    }
    // Source: cc/layers/layer_impl.cc:179-186.
    pub fn SetClipTreeIndex(&mut self, value: i32, c: &mut LayerImplContext<'_, B>) {
        if self.clip_tree_index_ == value {
            return;
        }
        self.clip_tree_index_ = value;
        self.SetNeedsPushProperties(Self::kChangedPropertyTreeIndex, c);
    }
    // Source: cc/layers/layer_impl.cc:188-195.
    pub fn SetEffectTreeIndex(&mut self, value: i32, c: &mut LayerImplContext<'_, B>) {
        if self.effect_tree_index_ == value {
            return;
        }
        self.effect_tree_index_ = value;
        self.SetNeedsPushProperties(Self::kChangedPropertyTreeIndex, c);
    }
    // Source: cc/layers/layer_impl.cc:197-203.
    pub fn render_target_effect_tree_index(&self, c: &LayerImplContext<'_, B>) -> i32 {
        self.CheckTree(c);
        self.render_target_effect_tree_index_for_property_trees(c.tree.property_trees())
    }
    // Same source body, borrowing the caller's actual resident property trees.
    pub fn render_target_effect_tree_index_for_property_trees(
        &self,
        trees: &PropertyTreesFor<B>,
    ) -> i32 {
        let effects = trees.effect_tree();
        let node = effects.Node(self.effect_tree_index_);
        Self::render_target_effect_tree_index_for_effect_node(
            node,
            effects.GetRenderSurface(self.effect_tree_index_).is_some(),
        )
    }
    // Source: cc/layers/layer_impl.cc:197-203. This projection allows the
    // selected surface to be mutably borrowed while the actual sibling effect
    // node/presence query remains a short readonly borrow.
    pub fn render_target_effect_tree_index_for_effect_node(
        node: &crate::cc::trees::effect_node::EffectNode<B::Filters, B::Path>,
        has_render_surface: bool,
    ) -> i32 {
        if has_render_surface {
            node.id
        } else {
            node.target_id
        }
    }
    // Source: cc/layers/layer_impl.cc:205-212.
    pub fn SetScrollTreeIndex(&mut self, value: i32, c: &mut LayerImplContext<'_, B>) {
        if self.scroll_tree_index_ == value {
            return;
        }
        self.scroll_tree_index_ = value;
        self.SetNeedsPushProperties(Self::kChangedPropertyTreeIndex, c);
    }
    // Source: cc/layers/layer_impl.cc:214-221.
    pub fn SetOffsetToTransformParent(
        &mut self,
        value: Vector2dF,
        c: &mut LayerImplContext<'_, B>,
    ) {
        if self.offset_to_transform_parent_ == value {
            return;
        }
        self.offset_to_transform_parent_ = value;
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer_impl.cc:629-631.
    pub fn IsScrollbarLayer(&self) -> bool {
        false
    }
    // Source: cc/layers/layer_impl.cc:606-608.
    pub fn IsActive(&self, c: &LayerImplContext<'_, B>) -> bool {
        self.CheckTree(c);
        c.tree.IsActiveTree()
    }
}
// Source: cc/layers/layer_impl.cc:125-129. Emitted synchronously before member
// destruction; the stored trace capability carries no tree/host/Layer pointer.
impl<B: LayerImplTypes> Drop for LayerImpl<B> {
    fn drop(&mut self) {
        self.deletion_trace_
            .LayerDeleted(self.layer_id_, self.stable_id_for_shared_quad_state_);
        // Source: cc/layers/layer_impl.h:668,664,662,642,632. Native owned member
        // destruction follows the destructor body in reverse declaration order.
        drop(self.all_touch_action_regions_.take());
        drop(self.debug_info_.take());
        let draw_properties =
            std::mem::replace(&mut self.draw_properties_, DrawProperties::default());
        drop(draw_properties);
        drop(self.rare_properties_.take());
        let touch_action_region =
            std::mem::replace(&mut self.touch_action_region_, TouchActionRegion::default());
        drop(touch_action_region);
    }
}
impl<B: LayerImplTypes> LayerImpl<B> {
    // Source: cc/layers/layer_impl.h:101-101.
    pub fn has_transform_node(&self) -> bool {
        self.has_transform_node_
    }
    // Source: cc/layers/layer_impl.h:103-103.
    pub fn set_property_tree_sequence_number(&mut self, _sequence: i32) {}
    // Source: cc/layers/layer_impl.h:106-106.
    pub fn transform_tree_index(&self) -> i32 {
        self.transform_tree_index_
    }
    // Source: cc/layers/layer_impl.h:109-109.
    pub fn clip_tree_index(&self) -> i32 {
        self.clip_tree_index_
    }
    // Source: cc/layers/layer_impl.h:112-112.
    pub fn effect_tree_index(&self) -> i32 {
        self.effect_tree_index_
    }
    // Source: cc/layers/layer_impl.h:116-116.
    pub fn scroll_tree_index(&self) -> i32 {
        self.scroll_tree_index_
    }
    // Source: cc/layers/layer_impl.h:119-121.
    pub fn offset_to_transform_parent(&self) -> Vector2dF {
        self.offset_to_transform_parent_
    }
    // Source: cc/layers/layer_impl.h:123-123.
    pub fn is_clipped(&self) -> bool {
        self.draw_properties_.is_clipped
    }
    // Source: cc/layers/layer_impl.h:177-177.
    pub fn draws_content(&self) -> bool {
        self.draws_content_
    }
    // Source: cc/layers/layer_impl.h:179-179.
    pub fn hit_test_opaqueness(&self) -> HitTestOpaqueness {
        self.hit_test_opaqueness_
    }
    // Source: cc/layers/layer_impl.h:185-185.
    pub fn background_color(&self) -> Color4f {
        self.background_color_
    }
    // Source: cc/layers/layer_impl.h:187-191.
    pub fn safe_opaque_background_color(&self) -> Color4f {
        debug_assert_eq!(
            self.contents_opaque(),
            self.safe_opaque_background_color_.alpha == 1.
        );
        self.safe_opaque_background_color_
    }
    // Source: cc/layers/layer_impl.h:196-196.
    pub fn contents_opaque(&self) -> bool {
        self.contents_opaque_
    }
    // Source: cc/layers/layer_impl.h:198-198.
    pub fn contents_opaque_for_text(&self) -> bool {
        self.contents_opaque_for_text_
    }
    // Source: cc/layers/layer_impl.h:204-204.
    pub fn element_id(&self) -> ElementId {
        self.element_id_
    }
    // Source: cc/layers/layer_impl.h:208-208.
    pub fn Is3dSorted(&self, c: &LayerImplContext<'_, B>) -> bool {
        self.Is3dSortedForPropertyTrees(c.tree.property_trees())
    }
    // Same source body, borrowing the caller's actual resident property trees.
    pub fn Is3dSortedForPropertyTrees(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.GetSortingContextIdForPropertyTrees(trees) != 0
    }
    // Source: cc/layers/layer_impl.h:211-213.
    pub fn should_check_backface_visibility(&self) -> bool {
        self.should_check_backface_visibility_
    }
    pub fn draw_properties(&self) -> &DrawProperties<B::Transform> {
        &self.draw_properties_
    }
    pub fn draw_properties_mut(&mut self) -> &mut DrawProperties<B::Transform> {
        &mut self.draw_properties_
    }
    // Source: cc/layers/layer_impl.h:241-243.
    pub fn SetVisibleLayerRectForTesting(&mut self, rect: Rect) {
        self.draw_properties_.visible_layer_rect = rect;
    }
    // Source: cc/layers/layer_impl.h:247-247.
    pub fn draw_opacity(&self) -> f32 {
        self.draw_properties_.opacity
    }
    // Source: cc/layers/layer_impl.h:248-250.
    pub fn screen_space_transform_is_animating(&self) -> bool {
        self.draw_properties_.screen_space_transform_is_animating
    }
    // Source: cc/layers/layer_impl.h:251-251.
    pub fn clip_rect(&self) -> Rect {
        self.draw_properties_.clip_rect
    }
    // Source: cc/layers/layer_impl.h:252-254.
    pub fn visible_drawable_content_rect(&self) -> Rect {
        self.draw_properties_.visible_drawable_content_rect
    }
    // Source: cc/layers/layer_impl.h:261-263.
    pub fn visible_layer_rect(&self) -> Rect {
        self.draw_properties_.visible_layer_rect
    }
    // Source: cc/layers/layer_impl.h:272-274.
    pub fn set_is_inner_viewport_scroll_layer(&mut self) {
        self.is_inner_viewport_scroll_layer_ = true;
    }
    // Source: cc/layers/layer_impl.h:303-303.
    pub fn HasAnyRarePropertySet(&self) -> bool {
        self.rare_properties_.is_some()
    }
    // Source: cc/layers/layer_impl.h:305-310.
    pub fn EnsureRareProperties(&mut self) -> &mut RareProperties<B> {
        if self.rare_properties_.is_none() {
            self.rare_properties_ = Some(Box::new(RareProperties::default()));
        }
        self.rare_properties_.as_deref_mut().unwrap()
    }
    // Source: cc/layers/layer_impl.h:312-317.
    pub fn ResetRareProperties(&mut self, c: &mut LayerImplContext<'_, B>) {
        if self.rare_properties_.is_some() {
            self.rare_properties_ = None;
            self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
        }
    }
    // Source: cc/layers/layer_impl.h:319-324.
    pub fn SetMainThreadScrollHitTestRegion(
        &mut self,
        region: &Region<B::RegionBackend>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        if self.rare_properties_.is_some() || !region.IsEmpty() {
            self.EnsureRareProperties()
                .main_thread_scroll_hit_test_region = region.clone();
            self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
        }
    }
    // Source: cc/layers/layer_impl.h:325-329.
    pub fn main_thread_scroll_hit_test_region(&self) -> &Region<B::RegionBackend> {
        match &self.rare_properties_ {
            Some(rare) => &rare.main_thread_scroll_hit_test_region,
            // Native Region::Empty is the backend's immutable NoDestructor
            // value (Region::Empty); no registry or owner state is created.
            None => Region::<B::RegionBackend>::Empty(),
        }
    }
    // Source: cc/layers/layer_impl.h:331-337.
    pub fn SetNonCompositedScrollHitTestRects(
        &mut self,
        rects: &[ScrollHitTestRect],
        c: &mut LayerImplContext<'_, B>,
    ) {
        if self.rare_properties_.is_some() || !rects.is_empty() {
            self.EnsureRareProperties()
                .non_composited_scroll_hit_test_rects = rects.to_vec();
            self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
        }
    }
    // Source: cc/layers/layer_impl.h:338-343.
    pub fn non_composited_scroll_hit_test_rects(&self) -> Option<&Vec<ScrollHitTestRect>> {
        self.rare_properties_
            .as_ref()
            .map(|rare| &rare.non_composited_scroll_hit_test_rects)
    }
    // Source: cc/layers/layer_impl.h:346-348.
    pub fn touch_action_region(&self) -> &TouchActionRegion<B::RegionBackend> {
        &self.touch_action_region_
    }
    // Source: cc/layers/layer_impl.h:350-352.
    pub fn has_touch_action_regions(&self) -> bool {
        !self.touch_action_region_.IsEmpty()
    }
    // Source: cc/layers/layer_impl.h:355-357.
    pub fn capture_bounds(&self) -> Option<&B::RegionCaptureBounds> {
        self.rare_properties_
            .as_ref()
            .map(|rare| &rare.capture_bounds)
    }
    // Source: cc/layers/layer_impl.h:360-363.
    pub fn tracked_element_rects(&self) -> Option<&B::TrackedElementRects> {
        self.rare_properties_
            .as_ref()
            .map(|rare| &rare.tracked_element_rects)
    }
    // Source: cc/layers/layer_impl.h:366-368.
    pub fn canvas_child_id(&self) -> ElementId {
        self.rare_properties_
            .as_ref()
            .map_or_else(ElementId::default, |rare| rare.canvas_child_id)
    }
    // Source: cc/layers/layer_impl.h:373-379.
    pub fn SetWheelEventHandlerRegion(
        &mut self,
        region: &Region<B::RegionBackend>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        if self.rare_properties_.is_some() || !region.IsEmpty() {
            self.EnsureRareProperties().wheel_event_handler_region = region.clone();
            self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
        }
    }
    // Source: cc/layers/layer_impl.h:380-383.
    pub fn wheel_event_handler_region(&self) -> &Region<B::RegionBackend> {
        match &self.rare_properties_ {
            Some(rare) => &rare.wheel_event_handler_region,
            // Native Region::Empty is the backend's immutable NoDestructor
            // value (Region::Empty); no registry or owner state is created.
            None => Region::<B::RegionBackend>::Empty(),
        }
    }
    // Source: cc/layers/layer_impl.h:390-390.
    pub fn update_rect(&self) -> &Rect {
        &self.update_rect_
    }
    // Source: cc/layers/layer_impl.h:473-473.
    pub fn needs_push_properties(&self) -> bool {
        self.needs_push_properties_
    }
    // Source: cc/layers/layer_impl.h:479-481.
    pub fn set_contributes_to_drawn_render_surface(&mut self, value: bool) {
        self.contributes_to_drawn_render_surface_ = value;
    }
    // Source: cc/layers/layer_impl.h:483-485.
    pub fn contributes_to_drawn_render_surface(&self) -> bool {
        self.contributes_to_drawn_render_surface_
    }
    // Source: cc/layers/layer_impl.h:524-527.
    pub fn GetFilterQuality(&self) -> B::FilterQuality {
        self.rare_properties_
            .as_ref()
            .map_or_else(B::FilterQuality::default, |rare| rare.filter_quality)
    }
    // Source: cc/layers/layer_impl.h:531-535.
    pub fn GetDynamicRangeLimit(&self) -> B::DynamicRangeLimit {
        self.rare_properties_
            .as_ref()
            .map_or_else(B::DynamicRangeLimit::default, |rare| {
                rare.dynamic_range_limit
            })
    }
    // Source: cc/layers/layer_impl.h:542-544.
    pub fn set_raster_even_if_not_drawn(&mut self, value: bool) {
        self.raster_even_if_not_drawn_ = value;
    }
    // Source: cc/layers/layer_impl.h:545-545.
    pub fn raster_even_if_not_drawn(&self) -> bool {
        self.raster_even_if_not_drawn_
    }
    // Source: cc/layers/layer_impl.h:572-572.
    pub fn GetChangeFlag(&self, mask: u8) -> bool {
        self.changed_properties_ & mask != 0
    }
    // Source: cc/layers/layer_impl.cc:388-392.
    pub fn ScrollBy(
        &mut self,
        scroll: Vector2dF,
        c: &mut crate::adapter::layer_impl::LayerImplScrollContext<'_, B>,
    ) -> Vector2dF {
        self.ScrollByWithGeometry(scroll, None, c)
    }
    // Source: cc/layers/layer_impl.cc:388-392. Same unique source body;
    // the explicit current receiver replaces its native owning-tree raw_ptr.
    pub fn ScrollByWithGeometry(
        &mut self,
        scroll: Vector2dF,
        geometry: Option<&mut super::scrollbar_layer_impl_base::ScrollbarGeometryFields<'_>>,
        c: &mut crate::adapter::layer_impl::LayerImplScrollContext<'_, B>,
    ) -> Vector2dF {
        let node_id = self.scroll_tree_index();
        let mut current = LayerImplCurrentReceiver {
            base: self,
            geometry,
        };
        c.tree.ScrollTreeScrollBy(&mut current, node_id, scroll)
    }
    // Source: cc/layers/layer_impl.cc:394-401.
    pub fn SetTouchActionRegion(
        &mut self,
        region: TouchActionRegion<B::RegionBackend>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        if self.touch_action_region_ == region {
            return;
        }
        self.touch_action_region_ = region;
        self.all_touch_action_regions_.take();
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer_impl.cc:403-412.
    pub fn GetAllTouchActionRegions(&self) -> &Region<B::RegionBackend> {
        if self.all_touch_action_regions_.get().is_none() {
            let _ = self
                .all_touch_action_regions_
                .set(Box::new(self.touch_action_region_.GetAllRegions()));
        } else {
            debug_assert!(
                self.touch_action_region_.GetAllRegions()
                    == **self.all_touch_action_regions_.get().unwrap()
            );
        }
        self.all_touch_action_regions_.get().unwrap()
    }
    // Source: cc/layers/layer_impl.cc:414-419.
    pub fn SetCaptureBounds(
        &mut self,
        bounds: B::RegionCaptureBounds,
        c: &mut LayerImplContext<'_, B>,
    ) {
        if self.rare_properties_.is_some() || !bounds.IsEmpty() {
            self.EnsureRareProperties().capture_bounds = bounds;
            self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
        }
    }
    // Source: cc/layers/layer_impl.cc:421-426.
    pub fn SetTrackedElementRects(
        &mut self,
        rects: B::TrackedElementRects,
        c: &mut LayerImplContext<'_, B>,
    ) {
        if self.rare_properties_.is_some() || !rects.IsEmpty() {
            self.EnsureRareProperties().tracked_element_rects = rects;
            self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
        }
    }
    // Source: cc/layers/layer_impl.cc:428-433.
    pub fn SetCanvasChildId(&mut self, id: ElementId, c: &mut LayerImplContext<'_, B>) {
        if self.rare_properties_.is_some() || id.IsValid() {
            self.EnsureRareProperties().canvas_child_id = id;
            self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
        }
    }
    // Source: cc/layers/layer_impl.cc:440-442.
    pub fn IsSnappedToPixelGridInTarget(&self) -> bool {
        false
    }
    // Source: cc/layers/layer_impl.cc:508-512.
    pub fn IsAffectedByPageScale(&self, c: &LayerImplContext<'_, B>) -> bool {
        c.tree
            .property_trees()
            .transform_tree()
            .Node(self.transform_tree_index())
            .in_subtree_of_page_scale_layer
    }
    // Source: cc/layers/layer_impl.cc:514-533.
    pub fn GetDamageReasonsFromLayerPropertyChange(
        &self,
        c: &LayerImplContext<'_, B>,
    ) -> DamageReasonSet {
        self.GetDamageReasonsFromLayerPropertyChangeForPropertyTrees(c.tree.property_trees())
    }
    // Same source body, borrowing the caller's actual resident property trees.
    pub fn GetDamageReasonsFromLayerPropertyChangeForPropertyTrees(
        &self,
        trees: &PropertyTreesFor<B>,
    ) -> DamageReasonSet {
        let mut reasons = DamageReasonSet::default();
        if self.layer_property_changed_not_from_property_trees_
            || self.layer_property_changed_from_property_trees_
            || trees.full_tree_damaged()
        {
            reasons.Put(DamageReason::kUntracked);
        }
        if self.transform_tree_index() != kInvalidPropertyNodeId {
            reasons.PutAll(
                &trees
                    .transform_tree()
                    .Node(self.transform_tree_index())
                    .damage_reasons(),
            );
        }
        if self.effect_tree_index() != kInvalidPropertyNodeId {
            if trees
                .effect_tree()
                .Node(self.effect_tree_index())
                .effect_changed
            {
                reasons.Put(DamageReason::kUntracked);
            }
        }
        reasons
    }
    // Source: cc/layers/layer_impl.cc:535-538.
    pub fn LayerPropertyChanged(&self, c: &LayerImplContext<'_, B>) -> bool {
        self.LayerPropertyChangedForPropertyTrees(c.tree.property_trees())
    }
    // Same source body, borrowing the caller's actual resident property trees.
    pub fn LayerPropertyChangedForPropertyTrees(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.layer_property_changed_not_from_property_trees_
            || self.LayerPropertyChangedFromPropertyTreesForPropertyTrees(trees)
    }
    // Source: cc/layers/layer_impl.cc:540-558.
    pub fn LayerPropertyChangedFromPropertyTrees(&self, c: &LayerImplContext<'_, B>) -> bool {
        self.LayerPropertyChangedFromPropertyTreesForPropertyTrees(c.tree.property_trees())
    }
    // Same source body, borrowing the caller's actual resident property trees.
    pub fn LayerPropertyChangedFromPropertyTreesForPropertyTrees(
        &self,
        trees: &PropertyTreesFor<B>,
    ) -> bool {
        if self.layer_property_changed_from_property_trees_ || trees.full_tree_damaged() {
            return true;
        }
        if self.transform_tree_index() == kInvalidPropertyNodeId {
            return false;
        }
        if trees
            .transform_tree()
            .Node(self.transform_tree_index())
            .transform_changed()
        {
            return true;
        }
        if self.effect_tree_index() == kInvalidPropertyNodeId {
            return false;
        }
        if trees
            .effect_tree()
            .Node(self.effect_tree_index())
            .effect_changed
        {
            return true;
        }
        false
    }
    // Source: cc/layers/layer_impl.cc:560-562.
    pub fn LayerPropertyChangedNotFromPropertyTrees(&self) -> bool {
        self.layer_property_changed_not_from_property_trees_
    }
    // Source: cc/layers/layer_impl.cc:564-568.
    pub fn NoteLayerPropertyChanged(&mut self, c: &mut LayerImplContext<'_, B>) {
        self.NoteLayerPropertyChangedWithPushContext(c);
    }
    // Same unique source body, with only the actual fields it consumes lent.
    // Source: cc/layers/layer_impl.cc:563-568.
    pub(crate) fn NoteLayerPropertyChangedWithPushContext(
        &mut self,
        c: &mut (impl crate::adapter::layer_impl::LayerImplPushContext<Types = B> + ?Sized),
    ) {
        self.layer_property_changed_not_from_property_trees_ = true;
        self.SplitTileClient()
            .0
            .SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
        c.set_needs_update_draw_properties();
    }
    // Source: cc/layers/layer_impl.cc:570-574.
    pub fn NoteLayerPropertyChangedFromPropertyTrees(&mut self, c: &mut LayerImplContext<'_, B>) {
        self.layer_property_changed_from_property_trees_ = true;
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
        c.tree.set_needs_update_draw_properties();
    }
    // Source: cc/layers/layer_impl.cc:595-604.
    pub fn ResetChangeTracking(&mut self) {
        self.layer_property_changed_not_from_property_trees_ = false;
        self.layer_property_changed_from_property_trees_ = false;
        self.needs_push_properties_ = false;
        self.changed_properties_ = 0;
        self.update_rect_ = Rect::new(0, 0, 0, 0);
        if let Some(debug) = &mut self.debug_info_ {
            debug.write().unwrap().invalidations.clear();
        }
    }
    // Source: cc/layers/layer_impl.cc:610-618.
    pub fn bounds(&self, c: &LayerImplContext<'_, B>) -> Size {
        self.bounds_for_tree(c.tree)
    }
    // Same native readonly body, lending only the actual tree query facet.
    pub fn bounds_for_tree(
        &self,
        tree: &dyn crate::adapter::layer_impl::LayerImplReadTree<B>,
    ) -> Size {
        self.bounds_for_property_trees(tree.property_trees())
    }
    // Source: cc/layers/layer_impl.cc:610-618; same body with a scoped tree query.
    pub fn bounds_for_property_trees(&self, trees: &PropertyTreesFor<B>) -> Size {
        if !self.is_inner_viewport_scroll_layer_ {
            return self.bounds_;
        }
        let delta = trees.inner_viewport_scroll_bounds_delta();
        Size::new(
            self.bounds_.width().wrapping_add(delta.x().ceil() as i32),
            self.bounds_.height().wrapping_add(delta.y().ceil() as i32),
        )
    }
    // Source: cc/layers/layer_impl.cc:658-671.
    pub fn HitTestable(&self, c: &LayerImplContext<'_, B>) -> bool {
        self.HitTestableForPropertyTrees(c.tree.property_trees())
    }
    // Same source body, borrowing the caller's actual resident property trees.
    pub fn HitTestableForPropertyTrees(&self, trees: &PropertyTreesFor<B>) -> bool {
        if self.effect_tree_index() != kInvalidPropertyNodeId {
            if trees
                .effect_tree()
                .Node(self.effect_tree_index())
                .subtree_hidden
            {
                return false;
            }
        }
        self.hit_test_opaqueness_ != HitTestOpaqueness::kTransparent
    }
    // Source: cc/layers/layer_impl.cc:673-678.
    pub fn OpaqueToHitTest(&self, c: &LayerImplContext<'_, B>) -> bool {
        self.OpaqueToHitTestForPropertyTrees(c.tree.property_trees())
    }
    // Same source body, borrowing the caller's actual resident property trees.
    pub fn OpaqueToHitTestForPropertyTrees(&self, trees: &PropertyTreesFor<B>) -> bool {
        self.HitTestableForPropertyTrees(trees)
            && self.hit_test_opaqueness_ == HitTestOpaqueness::kOpaque
            && !trees
                .effect_tree()
                .Node(self.effect_tree_index())
                .node_or_ancestor_has_fast_rounded_corner
    }
    // Source: cc/layers/layer_impl.cc:689-696.
    pub fn SetSafeOpaqueBackgroundColor(
        &mut self,
        color: Color4f,
        c: &mut LayerImplContext<'_, B>,
    ) {
        if self.safe_opaque_background_color_ == color {
            return;
        }
        self.safe_opaque_background_color_ = color;
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer_impl.cc:698-706.
    pub fn SetContentsOpaque(&mut self, opaque: bool, c: &mut LayerImplContext<'_, B>) {
        if self.contents_opaque_ == opaque && self.contents_opaque_for_text_ == opaque {
            return;
        }
        self.contents_opaque_ = opaque;
        self.contents_opaque_for_text_ = opaque;
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer_impl.cc:708-717.
    pub fn SetContentsOpaqueForText(&mut self, opaque: bool, c: &mut LayerImplContext<'_, B>) {
        debug_assert!(!self.contents_opaque_ || opaque);
        if self.contents_opaque_for_text_ == opaque {
            return;
        }
        self.contents_opaque_for_text_ = opaque;
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer_impl.cc:719-725.
    pub fn Opacity(&self, c: &LayerImplContext<'_, B>) -> f32 {
        self.OpacityForPropertyTrees(c.tree.property_trees())
    }
    // Source: cc/layers/layer_impl.cc:720-725. Same live readonly source body.
    pub fn OpacityForPropertyTrees(&self, trees: &PropertyTreesFor<B>) -> f32 {
        if self.effect_tree_index() != kInvalidPropertyNodeId {
            return trees.effect_tree().Node(self.effect_tree_index()).opacity;
        }
        1.
    }
    // Source: cc/layers/layer_impl.cc:727-735.
    pub fn SetElementId(&mut self, id: ElementId, c: &mut LayerImplContext<'_, B>) {
        if self.element_id_ == id {
            return;
        }
        c.services.TraceElement("LayerImpl::SetElementId", id);
        self.element_id_ = id;
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer_impl.cc:737-745.
    pub fn SetShouldCheckBackfaceVisibility(
        &mut self,
        value: bool,
        c: &mut LayerImplContext<'_, B>,
    ) {
        if self.should_check_backface_visibility_ == value {
            return;
        }
        self.should_check_backface_visibility_ = value;
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer_impl.cc:747-754.
    pub fn UnionUpdateRect(&mut self, rect: Rect, c: &mut LayerImplContext<'_, B>) {
        if self.update_rect_ == rect {
            return;
        }
        self.update_rect_.Union(rect);
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer_impl.cc:756-758.
    pub fn GetDamageRect(&self) -> Rect {
        Rect::default()
    }
    // Source: cc/layers/layer_impl.cc:776-780.
    pub fn VisibleOpaqueRegion(&self) -> SimpleEnclosedRegion {
        if self.contents_opaque() {
            SimpleEnclosedRegion::new(self.visible_layer_rect())
        } else {
            SimpleEnclosedRegion::default()
        }
    }
    // Source: cc/layers/layer_impl.cc:961-963.
    pub fn GetSortingContextId(&self, c: &LayerImplContext<'_, B>) -> i32 {
        self.GetSortingContextIdForPropertyTrees(c.tree.property_trees())
    }
    // Same source body, borrowing the caller's actual resident property trees.
    pub fn GetSortingContextIdForPropertyTrees(&self, trees: &PropertyTreesFor<B>) -> i32 {
        trees
            .transform_tree()
            .Node(self.transform_tree_index())
            .sorting_context_id
    }
    // Source: cc/layers/layer_impl.cc:965-967.
    pub fn GetInvalidationRegionForDebugging(&self) -> Region<B::RegionBackend> {
        Region::new(self.update_rect_)
    }
    // Source: cc/layers/layer_impl.cc:1058-1066.
    pub fn GetPreferredRasterScale(scale: Vector2dF) -> f32 {
        let lower = crate::adapter::geometry::cpp_min(scale.x(), scale.y());
        let higher = crate::adapter::geometry::cpp_max(scale.x(), scale.y());
        crate::adapter::geometry::cpp_min(5. * lower, higher)
    }
    // Source: cc/layers/layer_impl.cc:1068-1074.
    pub fn SetFilterQuality(&mut self, value: B::FilterQuality, c: &mut LayerImplContext<'_, B>) {
        if self.GetFilterQuality() == value {
            return;
        }
        self.EnsureRareProperties().filter_quality = value;
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer_impl.cc:1076-1084.
    pub fn SetDynamicRangeLimit(
        &mut self,
        value: B::DynamicRangeLimit,
        c: &mut LayerImplContext<'_, B>,
    ) {
        if self.GetDynamicRangeLimit() == value {
            return;
        }
        self.EnsureRareProperties().dynamic_range_limit = value;
        self.NoteLayerPropertyChanged(c);
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer_impl.cc:1106-1111.
    pub fn EnsureValidPropertyTreeIndices(&self) {
        assert!(self.transform_tree_index() > kInvalidPropertyNodeId);
        assert!(self.effect_tree_index() > kInvalidPropertyNodeId);
        assert!(self.clip_tree_index() > kInvalidPropertyNodeId);
        assert!(self.scroll_tree_index() > kInvalidPropertyNodeId);
    }
    // Source: cc/layers/layer_impl.cc:1113-1115.
    pub fn is_surface_layer(&self) -> bool {
        false
    }
    // Source: cc/layers/layer_impl.cc:1117-1119.
    pub fn DebugName(&self) -> String {
        self.debug_info_
            .as_ref()
            .map_or_else(String::new, |info| info.read().unwrap().name.clone())
    }
    // Source: cc/layers/layer_impl.cc:1121-1123.
    pub fn GetContentColorUsage(&self) -> crate::adapter::layer_tree_host_impl::ContentColorUsage {
        crate::adapter::layer_tree_host_impl::ContentColorUsage::kSRGB
    }
    // Source: cc/layers/layer_impl.cc:1125-1128.
    pub fn ViewTransitionResourceId(
        &self,
    ) -> crate::adapter::property_types::ViewTransitionElementResourceId {
        Default::default()
    }
}
// Optional source const-ref empty singleton representation. Nonempty state is
// always borrowed, and only the native zero/empty fallback owns a default.
enum ValueOrEmpty<'a, T> {
    Borrow(&'a T),
    Empty(T),
}
impl<T> std::ops::Deref for ValueOrEmpty<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        match self {
            Self::Borrow(value) => value,
            Self::Empty(value) => value,
        }
    }
}

impl<B: LayerImplTypes> LayerImpl<B> {
    // Source: cc/layers/layer_impl.cc:620-627.
    pub fn SetBounds(&mut self, value: Size, c: &mut LayerImplContext<'_, B>) {
        if self.bounds_ == value {
            return;
        }
        self.bounds_ = value;
        self.NoteLayerPropertyChanged(c);
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer_impl.cc:639-646.
    pub fn SetDrawsContent(&mut self, value: bool, c: &mut LayerImplContext<'_, B>) {
        if self.draws_content_ == value {
            return;
        }
        self.draws_content_ = value;
        self.NoteLayerPropertyChanged(c);
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer_impl.cc:648-656.
    pub fn SetHitTestOpaqueness(
        &mut self,
        value: HitTestOpaqueness,
        c: &mut LayerImplContext<'_, B>,
    ) {
        if self.hit_test_opaqueness_ == value {
            return;
        }
        self.hit_test_opaqueness_ = value;
        self.NoteLayerPropertyChanged(c);
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
    // Source: cc/layers/layer_impl.cc:680-687.
    pub fn SetBackgroundColor(&mut self, value: Color4f, c: &mut LayerImplContext<'_, B>) {
        if self.background_color_ == value {
            return;
        }
        self.background_color_ = value;
        self.NoteLayerPropertyChanged(c);
        self.SetNeedsPushProperties(Self::kChangedGeneralProperty, c);
    }
}
impl<B: LayerImplTypes> LayerImpl<B> {
    // Source: cc/layers/layer_impl.cc:586-593.
    pub fn GetScaledDrawTransform(&self, scale: f32) -> B::Transform {
        use crate::adapter::property_tree::PropertyTreeTransform;
        let mut value = self.draw_properties_.target_space_transform.clone();
        value.Scale(1. / scale, 1. / scale);
        value
    }
    // Source: cc/layers/layer_impl.cc:928-936.
    pub fn DrawTransform(&self, c: &LayerImplContext<'_, B>) -> B::Transform {
        self.DrawTransformForPropertyTrees(c.tree.property_trees())
    }
    // Same source body, borrowing the caller's actual resident property trees.
    pub fn DrawTransformForPropertyTrees(&self, trees: &PropertyTreesFor<B>) -> B::Transform {
        if !self.contributes_to_drawn_render_surface() {
            return crate::cc::trees::draw_property_utils::DrawTransformForLayer::<
                B::DrawContext<'_>,
            >(
                self.offset_to_transform_parent(),
                self.transform_tree_index(),
                self.render_target_effect_tree_index_for_property_trees(trees),
                trees,
            );
        }
        self.draw_properties().target_space_transform.clone()
    }
    // Source: cc/layers/layer_impl.cc:938-945.
    pub fn ScreenSpaceTransform(&self, c: &LayerImplContext<'_, B>) -> B::Transform {
        self.ScreenSpaceTransformForPropertyTrees(c.tree.property_trees())
    }
    // Same source body, borrowing the caller's actual resident property trees.
    pub fn ScreenSpaceTransformForPropertyTrees(
        &self,
        trees: &PropertyTreesFor<B>,
    ) -> B::Transform {
        if !self.contributes_to_drawn_render_surface() {
            return crate::cc::trees::draw_property_utils::ScreenSpaceTransformForLayer(
                self.offset_to_transform_parent(),
                self.transform_tree_index(),
                trees.transform_tree(),
            );
        }
        self.draw_properties().screen_space_transform.clone()
    }
    // Source: cc/layers/layer_impl.cc:947-959.
    pub fn VisibleLayerRect(&self, c: &LayerImplContext<'_, B>) -> Rect {
        self.VisibleLayerRectForPropertyTrees(c.tree.property_trees())
    }
    // Source: cc/layers/layer_impl.cc:947-959. Same native body, explicitly
    // lending only the caller's actual property-tree output.
    pub fn VisibleLayerRectForPropertyTrees(&self, trees: &PropertyTreesFor<B>) -> Rect {
        let view = LayerImplGeometry { layer: self };
        if !self.contributes_to_drawn_render_surface() {
            return crate::cc::trees::draw_property_utils::LayerVisibleRect::<
                B::DrawContext<'_>,
            >(&view, self.bounds_for_property_trees(trees), Some(trees));
        }
        #[cfg(debug_assertions)]
        {
            let computed = crate::cc::trees::draw_property_utils::LayerVisibleRect::<
                B::DrawContext<'_>,
            >(&view, self.bounds_for_property_trees(trees), Some(trees));
            debug_assert_eq!(self.visible_layer_rect(), computed);
        }
        self.visible_layer_rect()
    }
    // Source: cc/layers/layer_impl.cc:969-971.
    pub fn GetEnclosingVisibleRectInTargetSpace(&self, c: &LayerImplContext<'_, B>) -> Rect {
        self.GetScaledEnclosingVisibleRectInTargetSpace(1., c)
    }
    // Source: cc/layers/layer_impl.cc:973-993.
    pub fn GetScaledEnclosingVisibleRectInTargetSpace(
        &self,
        scale: f32,
        c: &LayerImplContext<'_, B>,
    ) -> Rect {
        self.GetScaledEnclosingVisibleRectInTargetSpaceForPropertyTrees(
            scale,
            c.tree.property_trees(),
        )
    }
    // Source: cc/layers/layer_impl.cc:973-993. Same source body, with only the
    // property-tree borrow consumed by its readonly geometry decisions.
    pub fn GetScaledEnclosingVisibleRectInTargetSpaceForPropertyTrees(
        &self,
        scale: f32,
        trees: &PropertyTreesFor<B>,
    ) -> Rect {
        debug_assert!(scale > 0.);
        let only_visible = trees
            .effect_tree()
            .Node(self.effect_tree_index())
            .only_draws_visible_content;
        let mut bounds = self.visible_layer_rect();
        if !only_visible {
            let size = self.bounds_for_property_trees(trees);
            bounds = Rect::new(0, 0, size.width(), size.height());
        }
        let transform = self.GetScaledDrawTransform(scale);
        let bounds = crate::adapter::geometry::ScaleToEnclosingRect(bounds, scale);
        crate::cc::base::math_util::MathUtil::MapEnclosingClippedRect(
            &transform, bounds,
        )
    }
    // Source: cc/layers/layer_impl.cc:995-1001. Both raw surface accessors borrow
    // the actual resident surface; the mutable variant is split by caller.
    pub fn render_target<'a>(
        &self,
        c: &'a LayerImplContext<'_, B>,
    ) -> Option<&'a B::RenderSurface> {
        c.tree
            .property_trees()
            .effect_tree()
            .GetRenderSurface(self.render_target_effect_tree_index(c))
    }
    // Source: cc/layers/layer_impl.cc:1003-1052.
    pub fn GetIdealContentsScale(&self, c: &LayerImplContext<'_, B>) -> Vector2dF {
        use crate::adapter::geometry::{cpp_max, cpp_min};
        let transform = self.ScreenSpaceTransform(c);
        if let Some(mut scales) = c.services.TryComputeTransform2dScaleComponents(&transform) {
            let cap = Self::GetPreferredRasterScale(scales);
            scales = Vector2dF::new(cpp_min(scales.x(), cap), cpp_min(scales.y(), cap));
            return scales;
        }
        let page = if self.IsAffectedByPageScale(c) {
            c.tree.current_page_scale_factor()
        } else {
            1.
        };
        let device = c.tree.device_scale_factor();
        let default_scale = page * device;
        let mut scale = c.services.ComputeApproximateMaxScale(&transform);
        let bounds = self.bounds(c);
        let tile = c.tree.settings().default_tile_size;
        let width_cap = ((tile.width() - 2) * 5) as f32;
        // Source performs each axis division before std::min.
        let cap = cpp_min(
            width_cap / bounds.width() as f32,
            ((tile.height() - 2) * 5) as f32 / bounds.height() as f32,
        );
        scale = cpp_min(scale, cap);
        scale = scale.round();
        scale = cpp_max(scale, default_scale);
        Vector2dF::new(scale, scale)
    }
    // Source: cc/layers/layer_impl.cc:1054-1056.
    pub fn GetIdealContentsScaleKey(&self, c: &LayerImplContext<'_, B>) -> f32 {
        Self::GetPreferredRasterScale(self.GetIdealContentsScale(c))
    }
}
// Pure live read view for the unique canonical LayerVisibleRect implementation.
struct LayerImplGeometry<'a, B: LayerImplTypes> {
    layer: &'a LayerImpl<B>,
}
impl<B: LayerImplTypes> crate::adapter::draw_property::LayerVisibleRectLayer<B::Transform>
    for LayerImplGeometry<'_, B>
{
    type Types = B;
    // Source: cc/layers/layer_impl.h:112-112.
    fn effect_tree_index(&self) -> i32 {
        self.layer.effect_tree_index()
    }
    // Source: cc/layers/layer_impl.h:109-109.
    fn clip_tree_index(&self) -> i32 {
        self.layer.clip_tree_index()
    }
    // Source: cc/layers/layer_impl.h:106-106.
    fn transform_tree_index(&self) -> i32 {
        self.layer.transform_tree_index()
    }
    // Source: cc/layers/layer_impl.h:119-121.
    fn offset_to_transform_parent(&self) -> Vector2dF {
        self.layer.offset_to_transform_parent()
    }
}
impl<B: LayerImplTypes> LayerImpl<B> {
    // Source: cc/layers/layer_impl.cc:223-239.
    pub fn PopulateSharedQuadState(
        &self,
        state: &mut B::SharedQuadState,
        opaque: bool,
        c: &LayerImplContext<'_, B>,
    ) {
        let effect = c
            .tree
            .property_trees()
            .effect_tree()
            .Node(self.effect_tree_index_);
        let clip = if self.draw_properties_.is_clipped {
            Some(self.draw_properties_.clip_rect)
        } else {
            None
        };
        let size = self.bounds(c);
        state.SetAll(
            &self.draw_properties_.target_space_transform,
            Rect::new(0, 0, size.width(), size.height()),
            self.draw_properties_.visible_layer_rect,
            &self.draw_properties_.mask_filter_info,
            clip,
            opaque,
            self.draw_properties_.opacity,
            if effect.HasRenderSurface() {
                SkBlendMode::kSrcOver
            } else {
                effect.blend_mode
            },
            self.GetSortingContextId(c),
            self.stable_id_for_shared_quad_state() as u32,
            self.draw_properties_.is_fast_rounded_corner,
        );
    }
    // Source: cc/layers/layer_impl.cc:241-253.
    pub fn PopulateScaledSharedQuadState(
        &self,
        state: &mut B::SharedQuadState,
        scale: f32,
        opaque: bool,
        c: &LayerImplContext<'_, B>,
    ) {
        let size = c.services.ScaleToCeiledSize(self.bounds(c), scale);
        let mut visible =
            crate::adapter::geometry::ScaleToEnclosingRect(self.visible_layer_rect(), scale);
        let rect = Rect::new(0, 0, size.width(), size.height());
        visible.Intersect(rect);
        self.PopulateScaledSharedQuadStateWithContentRects(state, scale, rect, visible, opaque, c);
    }
    // Source: cc/layers/layer_impl.cc:255-277.
    pub fn PopulateScaledSharedQuadStateWithContentRects(
        &self,
        state: &mut B::SharedQuadState,
        scale: f32,
        rect: Rect,
        visible: Rect,
        opaque: bool,
        c: &LayerImplContext<'_, B>,
    ) {
        let transform = self.GetScaledDrawTransform(scale);
        let effect = c
            .tree
            .property_trees()
            .effect_tree()
            .Node(self.effect_tree_index_);
        let clip = if self.draw_properties().is_clipped {
            Some(self.draw_properties().clip_rect)
        } else {
            None
        };
        state.SetAll(
            &transform,
            rect,
            visible,
            &self.draw_properties().mask_filter_info,
            clip,
            opaque,
            self.draw_properties().opacity,
            if effect.HasRenderSurface() {
                SkBlendMode::kSrcOver
            } else {
                effect.blend_mode
            },
            self.GetSortingContextId(c),
            self.stable_id_for_shared_quad_state() as u32,
            self.draw_properties().is_fast_rounded_corner,
        );
    }
    // Source: cc/layers/layer_impl.cc:279-301.
    pub fn WillDraw(&self, mode: super::draw_mode::DrawMode, c: &LayerImplContext<'_, B>) -> bool {
        if self.visible_layer_rect().IsEmpty()
            || self
                .draw_properties()
                .occlusion_in_content_space
                .IsOccluded(self.visible_layer_rect())
        {
            return false;
        }
        if mode == super::draw_mode::DrawMode::DRAW_MODE_RESOURCELESS_SOFTWARE {
            if self.effect_tree_index() != kInvalidPropertyNodeId {
                let effect = c
                    .tree
                    .property_trees()
                    .effect_tree()
                    .Node(self.effect_tree_index());
                if effect.blend_mode == SkBlendMode::kDstIn {
                    return false;
                }
            }
        }
        true
    }
    // Source: cc/layers/layer_impl.cc:303-305.
    pub fn HasMissingTiles(&self) -> bool {
        false
    }
    // Source: cc/layers/layer_impl.cc:307-309.
    pub fn ComputeCheckerboardedNeedsRecord(&mut self) -> bool {
        false
    }
    // Source: cc/layers/layer_impl.cc:311-313.
    pub fn ShowDebugBorders(
        &self,
        kind: crate::cc::debug::layer_tree_debug_state::DebugBorderType,
        c: &LayerImplContext<'_, B>,
    ) -> bool {
        c.tree.debug_state().show_debug_borders.test(kind as usize)
    }
    // Source: cc/layers/layer_impl.cc:315-327. Constructed LayerImpl always has
    // its source nonnull tree; the typed current tree supplies device scale.
    pub fn GetDebugBorderProperties(&self, c: &LayerImplContext<'_, B>) -> (Color4f, f32) {
        use crate::cc::debug::debug_colors::DebugColors;
        let scale = c.tree.device_scale_factor();
        if self.draws_content_ {
            return (
                DebugColors::ContentLayerBorderColor(),
                DebugColors::ContentLayerBorderWidth(scale) as f32,
            );
        }
        (
            DebugColors::ContainerLayerBorderColor(),
            DebugColors::ContainerLayerBorderWidth(scale) as f32,
        )
    }
    // Source: cc/layers/layer_impl.cc:329-339. Native base-class dispatch entry;
    // derived receivers provide their actual virtual color then call the exact
    // second overload below (no border/color policy is copied into a delegate).
    pub fn AppendDebugBorderQuad(
        &self,
        pass: &mut B::RenderPass,
        rect: Rect,
        state: B::SharedQuadStateHandle,
        data: &mut super::append_quads_data::AppendQuadsData<B::SurfaceId>,
        c: &LayerImplContext<'_, B>,
    ) {
        let (color, width) = self.GetDebugBorderProperties(c);
        self.AppendDebugBorderQuadWithColor(pass, rect, state, data, color, width, c);
    }
    // Source: cc/layers/layer_impl.cc:341-380.
    pub fn AppendDebugBorderQuadWithColor(
        &self,
        pass: &mut B::RenderPass,
        rect: Rect,
        state: B::SharedQuadStateHandle,
        _data: &mut super::append_quads_data::AppendQuadsData<B::SurfaceId>,
        color: Color4f,
        width: f32,
        c: &LayerImplContext<'_, B>,
    ) {
        if !self.ShowDebugBorders(
            crate::cc::debug::layer_tree_debug_state::DebugBorderType::LAYER,
            c,
        ) {
            return;
        }
        if rect.width.checked_mul(rect.height).is_none() {
            return;
        }
        let visible = rect;
        pass.AppendDebugBorderQuad(state, rect, visible, color, width);
        if self.contents_opaque() {
            let mut fill_color = color;
            fill_color.alpha *= 0.3;
            let fill_width = width * 3.;
            let mut fill_rect = rect;
            let inset = (fill_width / 2.) as i32;
            fill_rect.Inset(inset, inset, inset, inset);
            if fill_rect.IsEmpty() {
                return;
            }
            let mut visible_fill = visible;
            visible_fill.Intersect(fill_rect);
            pass.AppendDebugBorderQuad(state, fill_rect, visible_fill, fill_color, fill_width);
        }
    }
    // Source: cc/layers/layer_impl.h:159-163.
    pub fn ValidateQuadResources(&self, quad: &B::DrawQuad, c: &LayerImplContext<'_, B>) {
        #[cfg(debug_assertions)]
        {
            self.ValidateQuadResourcesInternal(quad, c);
        }
    }
    // Source: cc/layers/layer_impl.cc:576-584.
    fn ValidateQuadResourcesInternal(&self, quad: &B::DrawQuad, c: &LayerImplContext<'_, B>) {
        #[cfg(debug_assertions)]
        {
            if quad.resource_id() != ResourceId(0) {
                c.services.ValidateResource(quad.resource_id());
            }
        }
    }
    // Source: cc/layers/layer_impl.cc:382-386.
    pub fn GetContentsResourceId(
        &self,
        _resource: &mut ResourceId,
        _size: &mut Size,
        _uv: &mut SizeF,
    ) {
        unreachable!("source NOTREACHED base LayerImpl::GetContentsResourceId");
    }
    // Source: cc/layers/layer_impl.cc:768-774.
    pub fn SetCurrentScrollOffset(
        &mut self,
        offset: PointF,
        c: &mut crate::adapter::layer_impl::LayerImplScrollContext<'_, B>,
    ) {
        self.SetCurrentScrollOffsetWithGeometry(offset, None, c)
    }
    // Source: cc/layers/layer_impl.cc:768-774. Same unique source body with
    // the current scrollbar's actual derived fields, when it is that subtype.
    pub fn SetCurrentScrollOffsetWithGeometry(
        &mut self,
        offset: PointF,
        geometry: Option<&mut super::scrollbar_layer_impl_base::ScrollbarGeometryFields<'_>>,
        c: &mut crate::adapter::layer_impl::LayerImplScrollContext<'_, B>,
    ) {
        debug_assert_eq!(self.layer_tree_impl_, c.tree.id());
        debug_assert!(c.tree.IsActiveTree());
        let element = self.element_id();
        let changed = {
            let (tree, _, state) = c.tree.property_trees_mut().scroll_tree_with_context();
            tree.SetScrollOffset(element, &offset, state)
        };
        if changed {
            let mut current = LayerImplCurrentReceiver {
                base: self,
                geometry,
            };
            c.tree.DidUpdateScrollOffset(&mut current, element, false);
        }
    }
    // Source: cc/layers/layer_impl.cc:435-438.
    pub fn CreateLayerImpl(
        &self,
        tree: LayerTreeId,
        c: &mut LayerImplContext<'_, B>,
    ) -> Box<B::Layer> {
        Self::Create(tree, self.layer_id_, c)
    }
    // Source: cc/layers/layer_impl.cc:444-502.
    pub fn CopyPropertiesTo(
        &self,
        target: &mut LayerImpl<B>,
        source: &crate::adapter::layer_impl::LayerImplReadContext<'_, B>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        debug_assert!(target.IsActive(c));
        if self.GetChangeFlag(Self::kChangedPropertyTreeIndex) {
            target.transform_tree_index_ = self.transform_tree_index_;
            target.has_transform_node_ = self.has_transform_node_;
            target.effect_tree_index_ = self.effect_tree_index_;
            target.clip_tree_index_ = self.clip_tree_index_;
            target.scroll_tree_index_ = self.scroll_tree_index_;
        }
        if self.GetChangeFlag(Self::kChangedGeneralProperty) {
            target.SetElementId(self.element_id_, c);
            target.offset_to_transform_parent_ = self.offset_to_transform_parent_;
            target.contents_opaque_ = self.contents_opaque_;
            target.contents_opaque_for_text_ = self.contents_opaque_for_text_;
            target.should_check_backface_visibility_ = self.should_check_backface_visibility_;
            target.draws_content_ = self.draws_content_;
            target.hit_test_opaqueness_ = self.hit_test_opaqueness_;
            target.touch_action_region_ = self.touch_action_region_.clone();
            target.all_touch_action_regions_ = OnceCell::new();
            if let Some(region) = self.all_touch_action_regions_.get() {
                let _ = target
                    .all_touch_action_regions_
                    .set(Box::new((**region).clone()));
            }
            target.background_color_ = self.background_color_;
            target.safe_opaque_background_color_ = self.safe_opaque_background_color_;
            if self.layer_property_changed_not_from_property_trees_
                || self.layer_property_changed_from_property_trees_
            {
                c.tree.set_needs_update_draw_properties();
            }
            if self.layer_property_changed_not_from_property_trees_ {
                target.layer_property_changed_not_from_property_trees_ = true;
            }
            if self.layer_property_changed_from_property_trees_ {
                target.layer_property_changed_from_property_trees_ = true;
            }
            target.SetBounds(self.bounds_, c);
            target.UnionUpdateRect(self.update_rect_, c);
            match &self.debug_info_ {
                Some(info) => target.UpdateDebugInfo(Some(&mut info.write().unwrap()), c),
                None => target.UpdateDebugInfo(None, c),
            }
            target.rare_properties_ = ClonePtr(&self.rare_properties_);
        }
        if source.tree.TreesInVizInClientProcess() {
            target.SetNeedsPushProperties(self.changed_properties_, c);
        }
    }
}
// Explicit native virtual receiver. A derived owner passes a synchronous facade
// borrowing only its sibling fields, independently of this resident base. It
// owns no copied flags, tree or base backpointer. Defaults are actual source
// base implementations, rather than placeholders for derived behavior.
pub trait LayerImplVirtual<B: LayerImplTypes> {
    // Source: cc/layers/layer_impl.h:84-84.
    fn GetLayerType(&self, base: &LayerImpl<B>) -> LayerType {
        base.GetLayerType()
    }
    // Source: cc/layers/layer_impl.h:147-148.
    fn WillDraw(
        &mut self,
        base: &mut LayerImpl<B>,
        mode: super::draw_mode::DrawMode,
        _provider: Option<&mut B::ClientResourceProvider>,
        c: &mut LayerImplContext<'_, B>,
    ) -> bool {
        base.WillDraw(mode, c)
    }
    // Source: cc/layers/layer_impl.h:149-151.
    fn AppendQuads(
        &mut self,
        _base: &mut LayerImpl<B>,
        _context: &super::append_quads_context::AppendQuadsContext,
        _pass: &mut B::RenderPass,
        _data: &mut super::append_quads_data::AppendQuadsData<B::SurfaceId>,
        _c: &mut LayerImplContext<'_, B>,
    ) {
    }
    // Source: cc/layers/layer_impl.h:152-152.
    fn DidDraw(&mut self, _base: &mut LayerImpl<B>, _c: &mut LayerImplContext<'_, B>) {}
    // Source: cc/layers/layer_impl.h:154-154.
    fn HasMissingTiles(&self, base: &LayerImpl<B>) -> bool {
        base.HasMissingTiles()
    }
    // Source: cc/layers/layer_impl.h:156-156.
    fn ComputeCheckerboardedNeedsRecord(&mut self, base: &mut LayerImpl<B>) -> bool {
        base.ComputeCheckerboardedNeedsRecord()
    }
    // Source: cc/layers/layer_impl.h:165-167.
    fn GetContentsResourceId(
        &self,
        base: &LayerImpl<B>,
        resource: &mut ResourceId,
        size: &mut Size,
        uv: &mut SizeF,
    ) {
        base.GetContentsResourceId(resource, size, uv)
    }
    // Source: cc/layers/layer_impl.h:185. Native no-op notification; the typed
    // tile identity preserves the incoming nonowning pointer without ownership.
    // Source: cc/layers/layer_impl.h:169-169.
    fn NotifyTileStateChanged(
        &mut self,
        _base: &mut LayerImpl<B>,
        _tile: crate::adapter::tile::TileId,
        _damage: bool,
        _c: &mut LayerImplContext<'_, B>,
    ) {
    }
    // Source: cc/layers/layer_impl.h:171-171.
    fn IsScrollbarLayer(&self, base: &LayerImpl<B>) -> bool {
        base.IsScrollbarLayer()
    }
    // Source: cc/layers/layer_impl.h:395-395.
    fn GetDamageRect(&self, base: &LayerImpl<B>) -> Rect {
        base.GetDamageRect()
    }
    // Source: cc/layers/layer_impl.h:403-403.
    fn GetDamageReasons(
        &self,
        base: &LayerImpl<B>,
        c: &LayerImplContext<'_, B>,
    ) -> DamageReasonSet {
        base.GetDamageReasons(self, c)
    }
    // Source: cc/layers/layer_impl.h:418-418.
    fn ResetChangeTracking(&mut self, base: &mut LayerImpl<B>) {
        base.ResetChangeTracking();
    }
    // Source: cc/layers/layer_impl.h:420-420.
    fn VisibleOpaqueRegion(&self, base: &LayerImpl<B>) -> SimpleEnclosedRegion {
        base.VisibleOpaqueRegion()
    }
    // Source: cc/layers/layer_impl.h:422-422.
    fn DidBecomeActive(&mut self, _base: &mut LayerImpl<B>, _c: &mut LayerImplContext<'_, B>) {}
    // Source: cc/layers/layer_impl.h:424-424.
    fn DidBeginTracing(&mut self, base: &mut LayerImpl<B>) {
        base.DidBeginTracing();
    }
    // Source: cc/layers/layer_impl.h:428-428.
    fn ReleaseResources(&mut self, base: &mut LayerImpl<B>, c: &mut LayerImplContext<'_, B>) {
        base.ReleaseResources(c);
    }
    // Source: cc/layers/layer_impl.h:433-433.
    fn OnPurgeMemory(&mut self, base: &mut LayerImpl<B>, c: &mut LayerImplContext<'_, B>) {
        base.OnPurgeMemory(self, c);
    }
    // Source: cc/layers/layer_impl.h:437-437.
    fn ReleaseTileResources(&mut self, base: &mut LayerImpl<B>, c: &mut LayerImplContext<'_, B>) {
        base.ReleaseTileResources(c);
    }
    // Source: cc/layers/layer_impl.h:441-441.
    fn RecreateTileResources(&mut self, base: &mut LayerImpl<B>, c: &mut LayerImplContext<'_, B>) {
        base.RecreateTileResources(c);
    }
    // Source: cc/layers/layer_impl.h:443-444.
    fn CreateLayerImpl(
        &self,
        base: &LayerImpl<B>,
        tree: LayerTreeId,
        c: &mut LayerImplContext<'_, B>,
    ) -> Box<B::Layer> {
        base.CreateLayerImpl(tree, c)
    }
    // Source: cc/layers/layer_impl.h:446-446.
    fn CopyPropertiesTo(
        &self,
        base: &LayerImpl<B>,
        target: &mut B::Layer,
        source: &crate::adapter::layer_impl::LayerImplReadContext<'_, B>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        base.CopyPropertiesTo(target.impl_base_mut(), source, c);
    }
    // Source: cc/layers/layer_impl.h:449-449.
    fn MovePropertiesToActiveLayer(
        &mut self,
        base: &mut LayerImpl<B>,
        target: &mut B::Layer,
        source: &crate::adapter::layer_impl::LayerImplReadContext<'_, B>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        base.MovePropertiesToActiveLayer(self, target, source, c);
    }
    // Source: cc/layers/layer_impl.h:454-454.
    fn IsSnappedToPixelGridInTarget(&mut self, base: &mut LayerImpl<B>) -> bool {
        base.IsSnappedToPixelGridInTarget()
    }
    // Source: cc/layers/layer_impl.h:456-457.
    fn GetAllPrioritizedTilesForTracing(
        &self,
        base: &LayerImpl<B>,
        tiles: &mut Vec<crate::cc::tiles::prioritized_tile::PrioritizedTile>,
    ) {
        base.GetAllPrioritizedTilesForTracing(tiles);
    }
    // Source: cc/layers/layer_impl.h:458-458.
    fn AsValueInto(
        &self,
        base: &LayerImpl<B>,
        value: &mut B::LayerTraceValue,
        c: &LayerImplContext<'_, B>,
    ) {
        base.AsValueInto(self, value, c);
    }
    // Source: cc/layers/layer_impl.h:461-461.
    fn GPUMemoryUsageInBytes(&self, base: &LayerImpl<B>) -> usize {
        base.GPUMemoryUsageInBytes()
    }
    // Source: cc/layers/layer_impl.h:475-475.
    fn RunMicroBenchmark(
        &mut self,
        base: &mut LayerImpl<B>,
        benchmark: &mut dyn LayerImplBenchmark<B>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        base.RunMicroBenchmark(benchmark, c);
    }
    // Source: cc/layers/layer_impl.h:494-494.
    fn GetInvalidationRegionForDebugging(
        &mut self,
        base: &mut LayerImpl<B>,
    ) -> Region<B::RegionBackend> {
        base.GetInvalidationRegionForDebugging()
    }
    // Source: cc/layers/layer_impl.h:506-506.
    fn GetEnclosingVisibleRectInTargetSpace(
        &self,
        base: &LayerImpl<B>,
        c: &LayerImplContext<'_, B>,
    ) -> Rect {
        base.GetEnclosingVisibleRectInTargetSpace(c)
    }
    // Source: cc/layers/layer_impl.h:550-550.
    fn is_surface_layer(&self, base: &LayerImpl<B>) -> bool {
        base.is_surface_layer()
    }
    // Source: cc/layers/layer_impl.h:554-554.
    fn GetContentColorUsage(
        &self,
        base: &LayerImpl<B>,
    ) -> crate::adapter::layer_tree_host_impl::ContentColorUsage {
        base.GetContentColorUsage()
    }
    // Source: cc/layers/layer_impl.h:556-558.
    fn NotifyKnownResourceIdsBeforeAppendQuads(
        &mut self,
        _base: &mut LayerImpl<B>,
        _known: &std::collections::BTreeSet<
            crate::adapter::property_types::ViewTransitionElementResourceId,
        >,
    ) {
    }
    // Source: cc/layers/layer_impl.h:562-562.
    fn SetInInvisibleLayerTree(&mut self, _base: &mut LayerImpl<B>) {}
    // Source: cc/layers/layer_impl.h:560-560.
    fn ViewTransitionResourceId(
        &self,
        base: &LayerImpl<B>,
    ) -> crate::adapter::property_types::ViewTransitionElementResourceId {
        base.ViewTransitionResourceId()
    }
    // Source: cc/layers/layer_impl.h:581-581.
    fn GetDebugBorderProperties(
        &self,
        base: &LayerImpl<B>,
        c: &LayerImplContext<'_, B>,
    ) -> (Color4f, f32) {
        base.GetDebugBorderProperties(c)
    }
}
/// The actual base-most-derived vtable has no sibling fields.
pub struct LayerImplBaseVirtual;
impl<B: LayerImplTypes> LayerImplVirtual<B> for LayerImplBaseVirtual {}
impl<B: LayerImplTypes> LayerImpl<B> {
    // Source: cc/layers/layer_impl.cc:504-506.
    pub fn MovePropertiesToActiveLayer(
        &mut self,
        receiver: &mut (impl LayerImplVirtual<B> + ?Sized),
        target: &mut B::Layer,
        source: &crate::adapter::layer_impl::LayerImplReadContext<'_, B>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        receiver.CopyPropertiesTo(self, target, source, c);
    }
    // Source: cc/layers/layer_impl.cc:633-637.
    pub fn IsScrollerOrScrollbar(
        &self,
        receiver: &(impl LayerImplVirtual<B> + ?Sized),
        c: &LayerImplContext<'_, B>,
    ) -> bool {
        debug_assert!(!c.tree.settings().enable_hit_test_opaqueness);
        self.IsScrollerOrScrollbarForPropertyTrees(
            receiver.IsScrollbarLayer(self),
            c.tree.property_trees(),
        )
    }
    // Source: cc/layers/layer_impl.cc:633-637; native query with live trees.
    pub fn IsScrollerOrScrollbarForPropertyTrees(
        &self,
        is_scrollbar: bool,
        trees: &PropertyTreesFor<B>,
    ) -> bool {
        is_scrollbar
            || trees
                .scroll_tree()
                .FindNodeFromElementId(self.element_id())
                .is_some()
    }
    // Source: cc/layers/layer_impl.cc:760-766.
    pub fn GetDamageReasons(
        &self,
        receiver: &(impl LayerImplVirtual<B> + ?Sized),
        c: &LayerImplContext<'_, B>,
    ) -> DamageReasonSet {
        self.GetDamageReasonsForPropertyTrees(c.tree.property_trees(), || {
            receiver.GetDamageRect(self)
        })
    }
    // Source: cc/layers/layer_impl.cc:760-766. The callback preserves native
    // lazy virtual dispatch on the right hand side of the short-circuit OR.
    pub fn GetDamageReasonsForPropertyTrees(
        &self,
        trees: &PropertyTreesFor<B>,
        damage_rect: impl FnOnce() -> Rect,
    ) -> DamageReasonSet {
        let mut reasons = self.GetDamageReasonsFromLayerPropertyChangeForPropertyTrees(trees);
        if !self.update_rect_.IsEmpty() || !damage_rect().IsEmpty() {
            reasons.Put(DamageReason::kUntracked);
        }
        reasons
    }
    // Source: cc/layers/layer_impl.cc:782,784,790,792. Native empty base bodies.
    // Source: cc/layers/layer_impl.cc:782-782.
    pub fn DidBeginTracing(&mut self) {}
    // Source: cc/layers/layer_impl.cc:784-784.
    pub fn ReleaseResources(&mut self, _c: &mut LayerImplContext<'_, B>) {}
    // Source: cc/layers/layer_impl.cc:790-790.
    pub fn ReleaseTileResources(&mut self, _c: &mut LayerImplContext<'_, B>) {}
    // Source: cc/layers/layer_impl.cc:792-792.
    pub fn RecreateTileResources(&mut self, _c: &mut LayerImplContext<'_, B>) {}
    // Source: cc/layers/layer_impl.cc:786-788.
    pub fn OnPurgeMemory(
        &mut self,
        receiver: &mut (impl LayerImplVirtual<B> + ?Sized),
        c: &mut LayerImplContext<'_, B>,
    ) {
        receiver.ReleaseResources(self, c);
    }
    // Source: cc/layers/layer_impl.cc:812-814.
    pub fn GetAllPrioritizedTilesForTracing(
        &self,
        _tiles: &mut Vec<crate::cc::tiles::prioritized_tile::PrioritizedTile>,
    ) {
    }
    // Source: cc/layers/layer_impl.cc:922.
    pub fn GPUMemoryUsageInBytes(&self) -> usize {
        0
    }
    // Source: cc/layers/layer_impl.cc:924-926.
    pub fn RunMicroBenchmark(
        &mut self,
        benchmark: &mut dyn LayerImplBenchmark<B>,
        c: &mut LayerImplContext<'_, B>,
    ) {
        benchmark.RunOnLayer(self, c);
    }
}
impl<B: LayerImplTypes> LayerImpl<B> {
    // Source: cc/layers/layer_impl.cc:816-914.
    pub fn AsValueInto(
        &self,
        receiver: &(impl LayerImplVirtual<B> + ?Sized),
        state: &mut B::LayerTraceValue,
        c: &LayerImplContext<'_, B>,
    ) {
        use crate::adapter::{
            geometry::{QuadF, RectF, TracedValue},
            property_types::PropertyTrace,
        };
        use crate::cc::base::math_util::{MathUtil, TransformTrace};
        state.MakeDictIntoImplicitSnapshotWithCategory(
            "disabled-by-default-cc.debug",
            "cc::LayerImpl",
            LayerTypeAsString(receiver.GetLayerType(self)),
            self.stable_id_for_shared_quad_state_,
        );
        state.SetInteger("layer_id", self.id());
        MathUtil::AddToTracedValue("bounds", &self.bounds_, state);
        state.SetDouble("opacity", self.Opacity(c) as f64);
        MathUtil::AddToTracedValue("position", &PointF::default(), state);
        state.SetInteger("transform_tree_index", self.transform_tree_index());
        state.SetInteger("clip_tree_index", self.clip_tree_index());
        state.SetInteger("effect_tree_index", self.effect_tree_index());
        state.SetInteger("scroll_tree_index", self.scroll_tree_index());
        state.SetInteger("sorting_context_id", self.GetSortingContextId(c));
        state.SetInteger("draws_content", i32::from(self.draws_content()));
        state.SetInteger(
            "gpu_memory_usage",
            receiver.GPUMemoryUsageInBytes(self).min(i32::MAX as usize) as i32,
        );
        if self.element_id_.IsValid() {
            state.ElementIdInto(self.element_id_);
        }
        if !self.ScreenSpaceTransform(c).IsIdentity() {
            MathUtil::AddToTracedValue(
                "screen_space_transform",
                &TransformTrace(&self.ScreenSpaceTransform(c)),
                state,
            );
        }
        let mut clipped = false;
        let bounds = self.bounds(c);
        let quad = MathUtil::MapQuad(
            &self.ScreenSpaceTransform(c),
            QuadF::FromRect(RectF::from(Rect::new(
                0,
                0,
                bounds.width(),
                bounds.height(),
            ))),
            &mut clipped,
        );
        MathUtil::AddToTracedValue("layer_quad", &quad, state);
        if !self.GetAllTouchActionRegions().IsEmpty() {
            state.BeginArray("all_touch_action_regions");
            self.GetAllTouchActionRegions().AsValueInto(state);
            state.EndArray();
        }
        state.BeginArray("wheel_event_handler_region");
        self.wheel_event_handler_region().AsValueInto(state);
        state.EndArray();
        state.BeginArray("non_fast_scrollable_region");
        self.main_thread_scroll_hit_test_region().AsValueInto(state);
        state.EndArray();
        state.SetBoolean("hit_testable", self.HitTestable(c));
        state.SetBoolean("opaque_to_hit_test", self.OpaqueToHitTest(c));
        state.SetBoolean("contents_opaque", self.contents_opaque());
        if let Some(info) = &self.debug_info_ {
            let info = info.read().unwrap();
            state.SetString("layer_name", &info.name);
            if info.owner_node_id.IsNonzero() {
                state.SetInteger("owner_node", info.owner_node_id.AsInteger());
            }
            if !info.compositing_reasons.is_empty() {
                state.BeginArray("compositing_reasons");
                for reason in &info.compositing_reasons {
                    state.AppendString(reason);
                }
                state.EndArray();
            }
            if !info.compositing_reason_ids.is_empty() {
                state.BeginArray("compositing_reason_ids");
                for reason in &info.compositing_reason_ids {
                    state.AppendString(reason);
                }
                state.EndArray();
            }
            if !info.invalidations.is_empty() {
                state.BeginArray("annotated_invalidation_rects");
                for invalidation in &info.invalidations {
                    state.BeginDictionaryUnnamed();
                    MathUtil::AddToTracedValue("geometry_rect", &invalidation.rect, state);
                    state.SetString("reason", invalidation.reason);
                    state.SetString("client", &invalidation.client);
                    state.EndDictionary();
                }
                state.EndArray();
            }
        }
    }
    // Source: cc/layers/layer_impl.cc:916-920.
    pub fn ToString(
        &self,
        receiver: &(impl LayerImplVirtual<B> + ?Sized),
        c: &LayerImplContext<'_, B>,
    ) -> String {
        let mut value = c.services.NewTracedValueJSON();
        receiver.AsValueInto(self, &mut value, c);
        value.ToFormattedJSON()
    }
    // Source: cc/layers/layer_impl.cc:1086-1088. This is the live typed tree
    // output; mutations use the unique mutable counterpart below.
    pub fn GetPropertyTrees<'a>(
        &self,
        c: &'a LayerImplContext<'_, B>,
    ) -> &'a crate::adapter::layer_tree_impl::PropertyTreesFor<B> {
        self.CheckTree(c);
        c.tree.property_trees()
    }
    pub fn GetPropertyTreesMutable<'a>(
        &self,
        c: &'a mut LayerImplContext<'_, B>,
    ) -> &'a mut crate::adapter::layer_tree_impl::PropertyTreesFor<B> {
        self.CheckTree(c);
        c.tree.property_trees_mut()
    }
    // Source: cc/layers/layer_impl.cc:1090-1092.
    pub fn GetClipTree<'a>(
        &self,
        c: &'a LayerImplContext<'_, B>,
    ) -> &'a crate::cc::trees::property_tree::ClipTree {
        self.GetPropertyTrees(c).clip_tree()
    }
    // Source: cc/layers/layer_impl.cc:1094-1096.
    pub fn GetEffectTree<'a>(
        &self,
        c: &'a LayerImplContext<'_, B>,
    ) -> &'a crate::cc::trees::property_tree::EffectTree<
        B::Filters,
        B::Path,
        B::RenderSurface,
        B::CopyOutputRequest,
    > {
        self.GetPropertyTrees(c).effect_tree()
    }
    // Source: cc/layers/layer_impl.cc:1098-1100.
    pub fn GetScrollTree<'a>(
        &self,
        c: &'a LayerImplContext<'_, B>,
    ) -> &'a crate::cc::trees::property_tree::ScrollTree {
        self.GetPropertyTrees(c).scroll_tree()
    }
    // Source: cc/layers/layer_impl.cc:1102-1104.
    pub fn GetTransformTree<'a>(
        &self,
        c: &'a LayerImplContext<'_, B>,
    ) -> &'a crate::cc::trees::property_tree::TransformTree<B::Transform> {
        self.GetPropertyTrees(c).transform_tree()
    }
}

impl<B: LayerImplTypes> LayerImpl<B> {
    // Source: cc/layers/layer_impl.h:125. C++ owner pointer is exposed only as
    // the actual synchronous live owner facet, never as a retained whole tree.
    // Source: cc/layers/layer_impl.h:125-125.
    pub fn layer_tree_impl<'a>(
        &self,
        c: &'a mut LayerImplContext<'_, B>,
    ) -> &'a mut dyn LayerImplTree<B> {
        self.CheckTree(c);
        c.tree
    }
}

// Source: cc/layers/layer_impl.h:682-685; cc/layers/layer_impl.cc:1130. Namespace/header closure.

/// Source field split only; no owning or copied LayerImpl state.
pub(crate) struct LayerImplTileClient<'a> {
    id: &'a i32,
    tree: &'a LayerTreeId,
    changed: &'a mut u8,
    needs_push: &'a mut bool,
    opaque: &'a bool,
    contributes: &'a bool,
    raster_even_if_not_drawn: &'a bool,
}
impl LayerImplTileClient<'_> {
    pub(crate) fn id(&self) -> i32 {
        *self.id
    }
    pub(crate) fn contents_opaque(&self) -> bool {
        *self.opaque
    }
    pub(crate) fn contributes_to_drawn_render_surface(&self) -> bool {
        *self.contributes
    }
    pub(crate) fn raster_even_if_not_drawn(&self) -> bool {
        *self.raster_even_if_not_drawn
    }
    // Source: cc/layers/layer_impl.cc:794-810. Single canonical native body.
    pub(crate) fn SetNeedsPushProperties<B: LayerImplTypes>(
        &mut self,
        changed: u8,
        c: &mut (impl crate::adapter::layer_impl::LayerImplPushContext<Types = B> + ?Sized),
    ) {
        debug_assert_eq!(*self.tree, c.id());
        if c.IsActiveTree() && !c.TreesInVizInClientProcess() {
            return;
        }
        if c.IsCCSlimmingEnabled() {
            *self.changed |= changed;
        } else {
            *self.changed = LayerImpl::<B>::kChangedAllProperties;
        }
        if !*self.needs_push {
            *self.needs_push = true;
            c.AddLayerShouldPushProperties(*self.id);
        }
    }
}

struct LayerImplCurrentReceiver<'a, 'g, B: LayerImplTypes> {
    base: &'a mut LayerImpl<B>,
    geometry: Option<&'a mut super::scrollbar_layer_impl_base::ScrollbarGeometryFields<'g>>,
}
impl<B: LayerImplTypes> crate::adapter::layer_tree_impl::LayerTreeCurrentReceiver<B>
    for LayerImplCurrentReceiver<'_, '_, B>
{
    fn id(&self) -> i32 {
        self.base.id()
    }
    fn WithScrollbarGeometry(
        &mut self,
        callback: &mut dyn FnMut(
            &mut dyn crate::adapter::layer_tree_impl::LayerTreeScrollbarGeometry<B>,
        ),
    ) {
        let geometry = self
            .geometry
            .as_deref_mut()
            .expect("native selected registered layer must have actual scrollbar subtype");
        let mut receiver = super::scrollbar_layer_impl_base::ScrollbarGeometryReceiver {
            base: self.base,
            geometry,
        };
        callback(&mut receiver)
    }
}
