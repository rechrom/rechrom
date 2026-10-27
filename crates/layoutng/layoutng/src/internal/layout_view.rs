#![allow(non_snake_case)]

use foundation::graphics_types;
use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use foundation::{
    gfx, AtomicString, CompositingReasons, DynamicTo, EOverflow, GCedHeapHashSet, HeapVector,
    LayoutUnit, MakeGarbageCollected, Member, OverlayScrollbarClipBehavior, PhysicalOffset,
    PhysicalRect, PhysicalSize, RuntimeEnabledFeatures, ToFlooredSize, TransformState, Vector,
    Visitor, WeakHeapHashMap, WeakMember,
};
use graphics_types::graphics::visual_rect_flags::VisualRectFlags;
use layout_fragment::layout_result::LayoutResult;
use layoutng_geometry::geometry::logical_size::ToLogicalSize;
use layoutng_style::style::computed_style::ComputedStyle;

use super::block_node::BlockNode;
use super::constraint_space_builder::ConstraintSpaceBuilder;
use super::layout_block::LayoutBlock;
use super::layout_block_flow::LayoutBlockFlow;
use super::layout_box::LayoutBox;
use super::layout_box_layout::ShouldIncludeScrollbarGutter;
use super::layout_box_model_object::{LayoutBoxModelObject, PaintLayerType};
use super::layout_input_node::MinMaxSizesFloatInput;
use super::layout_invalidation_reason;
use super::layout_node_metadata::ContainerNode;
use super::layout_object::{
    BoxQuadType, HitTestLocation, HitTestResult, LayoutObject, MarkingBehavior, PositionedState,
};
use super::layout_text::LayoutText;
use super::length_utils::SizeType;
use super::map_coordinates_flags::MapCoordinatesFlags;
use super::scroll_types::{IncludeScrollbarsInRect, ScrollOffset};
use super::scrollbar_mode::mojom::blink::ScrollbarMode;
use super::variable_length_transform_result::VariableLengthTransformResult;

#[path = "../../../layout_invalidation/src/native_layout_view_destruction.rs"]
mod layout_invalidation_native_layout_view_destruction;
use layout_invalidation_native_layout_view_destruction::LayoutViewWillBeDestroyed;

#[path = "../../../layout_invalidation/src/native_layout_view_invalidation.rs"]
mod layout_invalidation_native_layout_view_invalidation;

#[path = "../../../layout_invalidation/src/native_layout_view_child_mutation.rs"]
mod layout_invalidation_native_layout_view_child_mutation;

#[path = "../../../layout_algo_block/src/native_layout_view_root_layout.rs"]
mod layout_algo_block_native_layout_view_root_layout;

#[path = "../../../layout_algo_block/src/native_layout_view_block_queries.rs"]
mod layout_algo_block_native_layout_view_block_queries;

#[path = "../../../layout_algo_svg/src/native_svg_text_descendant_index.rs"]
mod layout_algo_svg_native_svg_text_descendant_index;

#[path = "../../../layout_algo_inline/src/native_layout_view_variable_length_transform_index.rs"]
mod layout_algo_inline_native_layout_view_variable_length_transform_index;

#[path = "../../../layout_algo_list/src/native_layout_view_list_index.rs"]
mod layout_algo_list_native_layout_view_list_index;

#[path = "../../../layout_fragment/src/native_layout_view_geometry.rs"]
mod layout_fragment_native_layout_view_geometry;

#[path = "../../../layout_fragment/src/native_layout_view_fragment_queries.rs"]
mod layout_fragment_native_layout_view_fragment_queries;

#[path = "../../../layout_fragment/src/native_layout_view_paint_policy.rs"]
mod layout_fragment_native_layout_view_paint_policy;

#[path = "../../../layout_commit/src/native_layout_view_commit.rs"]
mod layout_commit_native_layout_view_commit;

// The corresponding C++ classes are only forward-declared in layout_view.h.
#[repr(C)]
pub struct HitTestCache {
    _opaque: [u8; 0],
}
#[repr(C)]
pub struct LayoutViewTransitionRoot {
    _opaque: [u8; 0],
}

// The source tree supplies declarations but no bodies for these view methods.
// The pagination factory is defined in //src/layoutng_paged, outside the
// selected packages; the other contracts belong to the external assembly.
unsafe extern "Rust" {
    fn LayoutViewCreateAnonymousPageLayoutObjectProvider(
        view: &mut LayoutView,
        style: &ComputedStyle,
    ) -> *mut LayoutBlockFlow;
    fn LayoutViewHitTestProvider(
        view: &mut LayoutView,
        location: &HitTestLocation,
        result: &mut HitTestResult,
    ) -> bool;
    fn LayoutViewHitTestNoLifecycleUpdateProvider(
        view: &mut LayoutView,
        location: &HitTestLocation,
        result: &mut HitTestResult,
    ) -> bool;
    fn LayoutViewClearHitTestCacheProvider(view: &mut LayoutView);
    fn LayoutViewMapToVisualRectInAncestorSpaceInternalProvider(
        view: &LayoutView,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: VisualRectFlags,
    ) -> bool;
    fn LayoutViewCommitPendingSelectionProvider(view: &mut LayoutView);
    fn LayoutViewQuadsInAncestorInternalProvider(
        view: &LayoutView,
        quads: &mut Vector<gfx::QuadF>,
        ancestor: *const LayoutBoxModelObject,
        flags: MapCoordinatesFlags,
        box_type: BoxQuadType,
    );
    fn LayoutViewUpdateHitTestResultProvider(
        view: &LayoutView,
        result: &mut HitTestResult,
        offset: &PhysicalOffset,
    );
    fn LayoutViewNamedPageAtIndexProvider(view: &LayoutView, page_index: usize) -> AtomicString;
    fn LayoutViewInvalidateLayoutForCounterStyleChangesProvider(view: &mut LayoutView);
    fn LayoutViewInvalidatePaintForViewAndDescendantsProvider(view: &mut LayoutView);
    fn LayoutViewMapAncestorToLocalProvider(
        view: &LayoutView,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: MapCoordinatesFlags,
    );
    fn LayoutViewMapLocalToAncestorProvider(
        view: &LayoutView,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: MapCoordinatesFlags,
    );
    fn LayoutViewGetViewTransitionRootProvider(view: &LayoutView) -> *mut LayoutViewTransitionRoot;
    fn LayoutViewCacheScrollDimensionsProvider(view: &mut LayoutView);
}

// cpp: layoutng/internal/layout_view.h:45-47
pub type SVGTextDescendantsMap =
    WeakHeapHashMap<LayoutBlock, Member<GCedHeapHashSet<Member<LayoutBox>>>>;

// cpp: layoutng/internal/layout_view.h:407-412
#[derive(Clone, Copy, Debug, Default)]
pub struct CachedScrollDimensions {
    pub width: LayoutUnit,
    pub height: LayoutUnit,
    pub origin: gfx::Point,
    pub offset: ScrollOffset,
}

// cpp: layoutng/internal/layout_view.h:50-68
// cpp: layoutng/internal/layout_view.h:349-434
#[repr(C)]
pub struct LayoutView {
    base_: LayoutBlockFlow,
    initial_containing_block_resize_handled_list_: Member<GCedHeapHashSet<Member<LayoutObject>>>,
    initial_containing_block_size_for_printing_: PhysicalSize,
    pagination_scale_factor_: f32,
    anonymous_page_objects_: HeapVector<Member<LayoutObject>>,
    layout_counter_count_: u32,
    layout_list_item_count_: u32,
    svg_text_descendants_: SVGTextDescendantsMap,
    text_to_variable_length_transform_result_:
        WeakHeapHashMap<LayoutText, VariableLengthTransformResult>,
    hit_test_count_: u32,
    hit_test_cache_hits_: u32,
    hit_test_cache_: Member<HitTestCache>,
    autosize_h_scrollbar_mode_: ScrollbarMode,
    autosize_v_scrollbar_mode_: ScrollbarMode,
    cached_scroll_dimensions_: Option<CachedScrollDimensions>,
    previous_background_rect_: Cell<PhysicalRect>,
    vertical_scrollbar_width_for_viewport_units_: i32,
    horizontal_scrollbar_height_for_viewport_units_: i32,
    contains_annotations_: bool,
    contains_non_scaling_stroke_: bool,
}

// cpp: layoutng/internal/layout_view.h:437-442
impl foundation::DowncastFrom<LayoutObject> for LayoutView {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsLayoutView()
    }
}

impl foundation::DowncastFrom<LayoutBox> for LayoutView {
    fn AllowFrom(box_: &LayoutBox) -> bool {
        box_.IsLayoutView()
    }
}

const _: () = assert!(std::mem::offset_of!(LayoutView, base_) == 0);

impl Deref for LayoutView {
    type Target = LayoutBlockFlow;
    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}

impl DerefMut for LayoutView {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_
    }
}

impl LayoutView {
    // cpp: layoutng/internal/layout_view_layout.cc:22-33
    pub fn new(input_root: *mut ContainerNode) -> Self {
        assert!(!input_root.is_null());
        let mut view = Self {
            base_: LayoutBlockFlow::new(input_root),
            initial_containing_block_resize_handled_list_: Member::default(),
            initial_containing_block_size_for_printing_: PhysicalSize::default(),
            pagination_scale_factor_: 1.0,
            anonymous_page_objects_: HeapVector::default(),
            layout_counter_count_: 0,
            layout_list_item_count_: 0,
            svg_text_descendants_: SVGTextDescendantsMap::default(),
            text_to_variable_length_transform_result_: WeakHeapHashMap::default(),
            hit_test_count_: 0,
            hit_test_cache_hits_: 0,
            hit_test_cache_: Member::default(),
            autosize_h_scrollbar_mode_: ScrollbarMode::kAuto,
            autosize_v_scrollbar_mode_: ScrollbarMode::kAuto,
            cached_scroll_dimensions_: None,
            previous_background_rect_: Cell::new(PhysicalRect::default()),
            vertical_scrollbar_width_for_viewport_units_: 0,
            horizontal_scrollbar_height_for_viewport_units_: 0,
            contains_annotations_: false,
            contains_non_scaling_stroke_: false,
        };
        view.SetRuntimeClass(super::layout_object::LayoutObjectClass::View);
        view.SetInline(false);
        view.SetIntrinsicLogicalWidthsDirty(MarkingBehavior::kMarkOnlyThis);
        view.SetPositionState(PositionedState::kIsOutOfFlowPositioned);
        view
    }

    // cpp: layoutng/internal/layout_view_layout.cc:36-42
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.anonymous_page_objects_);
        visitor.Trace(&self.svg_text_descendants_);
        visitor.Trace(&self.text_to_variable_length_transform_result_);
        visitor.Trace(&self.initial_containing_block_resize_handled_list_);
        self.base_.Trace(visitor);
    }

    // cpp: layoutng/internal/layout_view_layout.cc:194-202
    pub fn WillBeDestroyed(&mut self) {
        LayoutViewWillBeDestroyed(self)
    }

    // cpp: layoutng/internal/layout_view.h:102-115
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        "LayoutView"
    }
    pub fn IsLayoutView(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }
    // cpp: layoutng/internal/layout_view.h:437-442
    pub fn AllowFromLayoutObject(object: &LayoutObject) -> bool {
        object.IsLayoutView()
    }
}
