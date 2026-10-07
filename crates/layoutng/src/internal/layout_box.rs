#![allow(non_snake_case)]

use foundation::graphics_types;
use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use super::algorithm_forward::CustomLayoutChild;
use foundation::{
    gfx, kIndefiniteSize, DynamicTo, EClear, EColumnSpan, EOverflow, EPosition, EReadingFlow,
    GCedHeapHashSet, GCedHeapVector, HeapVector, InfiniteIntRect, LayoutUnit, LogicalToPhysical,
    MakeGarbageCollected, Member, OverlayScrollbarClipBehavior, PhysicalOffset, PhysicalRect,
    PhysicalSize, ScopedCSSName, TextDirection, To, TransformAccumulation, TransformState,
    ValueForLength, Vector, WritingDirectionMode, WritingMode,
};
use graphics_types::graphics::paint::display_item_client_types::RasterEffectOutset;
use graphics_types::graphics::visual_rect_flags::{VisualRectFlag, VisualRectFlags};
use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_fragment_tree::fragment_data::FragmentData;
use layoutng_fragment_tree::fragment_items::FragmentItems;
use layoutng_fragment_tree::inline_cursor::InlineCursor;
use layoutng_fragment_tree::layout_result::{EStatus, LayoutResult};
use layoutng_fragment_tree::logical_fragment::LogicalFragment;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::box_sides::PhysicalBoxSides;
use layoutng_geometry::geometry::box_strut::{BoxStrut, PhysicalBoxStrut};
use layoutng_geometry::geometry::fragment_geometry::FragmentGeometry;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::logical_size::{LogicalSize, ToLogicalSize};
use layoutng_geometry::geometry::overflow_clip_axes::kOverflowClipBothAxis;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_geometry::geometry::static_position::LogicalStaticPosition;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::style_overflow_clip_margin::ReferenceBox;

use super::anchor_position_scroll_data::AnchorPositionScrollData;
use super::block_node::BlockNode;
use super::break_appeal::BreakAppeal;
use super::caret_rect::CaretShape;
use super::column_spanner_path::ColumnSpannerPath;
use super::constraint_space::ConstraintSpace;
use super::constraint_space::LayoutResultCacheSlot;
use super::disable_layout_side_effects_scope::DisableLayoutSideEffectsScope;
use super::early_break::EarlyBreak;
use super::editing::forward::PositionWithAffinity;
use super::form_node_metadata::{HTMLFieldSetElement, HTMLImageElement};
use super::fragmentation_utils::FragmentIndex;
use super::fragmentation_utils::{FragmentainerOffsetAtBfc, IsBreakInside};
use super::gap::gap_geometry::GapGeometry;
use super::grid_layout_data::GridLayoutData;
use super::hit_test_phase::HitTestPhase;
use super::layout_block::LayoutBlock;
use super::layout_block_flow::LayoutBlockFlow;
use super::layout_box_core_services::{
    LayoutBoxAcceptableImplicitAnchor, LayoutBoxDefaultIntrinsicContentBlockSize,
    LayoutBoxDefaultIntrinsicContentInlineSize, LayoutBoxFindTargetAnchor,
    LayoutBoxGetScrollMarkerGroup, LayoutBoxRecalcChildScrollableOverflowNG,
    LayoutBoxRecalcScrollableOverflowNG,
};
use super::layout_box_layout::{ShouldClampToContentBox, ShouldIncludeScrollbarGutter};
use super::layout_box_model_object::LayoutBoxModelObject;
use super::layout_inline::LayoutInline;
use super::layout_node_data::LayoutBoxRareData;
use super::layout_node_metadata::{ContainerNode, Element, Node};
use super::layout_object::{
    BackgroundPaintLocation, BoxQuadType, HitTestLocation, HitTestResult, IncludeDescendants,
    LayoutObject, LayoutObjectClass, LocalRectForBoxQuad,
    MutableForPainting as LayoutObjectMutableForPainting, PaintInfo, PaintInvalidatorContext,
    PositionedState, RecalcScrollableOverflowResult,
};
use super::layout_pass_scope::LayoutObjectFactoryScope;
use super::layout_replaced::LayoutReplaced;
use super::layout_utils::{
    CalculateSizeBasedLayoutCacheStatus, LayoutCacheStatus,
    MaySkipLayoutWithinBlockFormattingContext,
};
use super::layout_view::LayoutView;
use super::loader::fetch::resource_priority::ResourcePriority;
use super::loader::resource::image_resource_observer::{CanDeferInvalidation, WrappedImagePtr};
use super::map_coordinates_flags::MapCoordinatesFlags;
use super::measure_cache::MeasureCache;
use super::min_max_sizes::{MinMaxSizes, MinMaxSizesResult};
use super::min_max_sizes_cache::MinMaxSizesCache;
use super::non_overflowing_scroll_range::NonOverflowingScrollRange;
use super::outline_info::LayoutOutlineInfo;
use super::outline_rect_collector::OutlineRectCollector;
use super::overflow_model::{BoxOverflowModel, BoxScrollableOverflowModel};
use super::scroll_layout_scope::ScrollbarFreezeState;
use super::shapes::shape_outside_info::ShapeOutsideInfo;
use layoutng_style::style::outline_type::OutlineType;

// The base-first Rust representation cannot use C++'s LayoutBox virtual
// table. Keep the flex overflow conversion at this call boundary so callers
// through BlockNode and LayoutBox observe LayoutFlexibleBox's override.
// cpp: layoutng_flex/layout_flexible_box.cc:20-47
fn FlexOverflowConverter(style: &ComputedStyle) -> LogicalToPhysical<bool> {
    let is_wrap_reverse = style.ResolvedIsFlexWrapReverse();
    let is_direction_reverse = style.ResolvedIsReverseFlexDirection();
    let (mut inline_start, mut inline_end, mut block_start, mut block_end) =
        (false, true, false, true);
    if style.ResolvedIsColumnFlexDirection() {
        if is_direction_reverse {
            std::mem::swap(&mut block_start, &mut block_end);
        }
        if is_wrap_reverse {
            std::mem::swap(&mut inline_start, &mut inline_end);
        }
    } else {
        if is_direction_reverse {
            std::mem::swap(&mut inline_start, &mut inline_end);
        }
        if is_wrap_reverse {
            std::mem::swap(&mut block_start, &mut block_end);
        }
    }
    LogicalToPhysical::new(
        style.GetWritingDirection(),
        inline_start,
        inline_end,
        block_start,
        block_end,
    )
}

// C++ stores traced const results. The Rust member retains the same object
// identity; readers borrow it immutably at use sites.
// cpp: layoutng/internal/layout_box.h:535-535
pub type LayoutResultList = HeapVector<Member<LayoutResult>, 1>;

// cpp: layoutng/internal/layout_box.h:70-73
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundRectType {
    kBackgroundPaintedExtent,
    kBackgroundKnownOpaqueRect,
}

unsafe extern "Rust" {
    fn LayoutBoxStitchedSizeProvider(box_: &LayoutBox) -> PhysicalSize;
    fn LayoutBoxPhysicalLocationProvider(box_: &LayoutBox) -> PhysicalOffset;
    fn LayoutBoxOverflowClipRectProvider(
        box_: &LayoutBox,
        behavior: OverlayScrollbarClipBehavior,
    ) -> PhysicalRect;

    // The source LayoutBox default is empty; derived table cells may override it.
    fn InvalidateLayoutResultCacheAfterMeasureProvider(box_: &LayoutBox);
    // Flex/grid identities belong to algorithm owners outside the selected package.
    fn LayoutBoxIsFlexibleBoxProvider(box_: &LayoutBox) -> bool;
    fn LayoutBoxIsLayoutGridProvider(box_: &LayoutBox) -> bool;
    // LayoutView and its viewport geometry belong to a pending owner in this package.
    fn LayoutBoxEffectiveRootScrollerViewportSizeProvider(box_: &LayoutBox) -> PhysicalSize;
    fn LayoutBoxCreatesNewFormattingContextProvider(box_: &LayoutBox) -> bool;
    fn LayoutBoxIsFragmentationContextRootProvider(box_: &LayoutBox) -> bool;
    fn LayoutBoxShouldPlaceBlockDirectionScrollbarOnLogicalLeftProvider(box_: &LayoutBox) -> bool;
    fn LayoutBoxMutableSavePreviousOverflowDataProvider(box_: &mut LayoutBox);
    fn LayoutBoxMutableSavePreviousGapGeometriesProvider(box_: &mut LayoutBox);
    fn LayoutBoxMutableClearPreviousGapGeometriesProvider(box_: &mut LayoutBox);
    fn LayoutBoxMutableSetPreviousGeometryForLayoutShiftTrackingProvider(
        box_: &mut LayoutBox,
        paint_offset: &PhysicalOffset,
        size: &PhysicalSize,
        visual_overflow_rect: &PhysicalRect,
    );
    fn LayoutBoxMutableUpdateBackgroundPaintLocationProvider(
        box_: &mut LayoutBox,
        needs_root_element_group: bool,
    );
    // These declarations have no definitions in the supplied source packages.
    // Their typed providers preserve the source interface for the owner layer.
    fn LayoutBoxPhysicalBackgroundRectProvider(
        box_: &LayoutBox,
        kind: BackgroundRectType,
    ) -> PhysicalRect;
    fn LayoutBoxScrollerFromScrollMarkerGroupProvider(box_: &LayoutBox) -> *mut LayoutBlock;
    fn LayoutBoxLayoutSubtreeRootProvider(box_: &mut LayoutBox);
    fn LayoutBoxHitTestAllPhasesProvider(
        box_: &mut LayoutBox,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        offset: &PhysicalOffset,
    ) -> bool;
    fn LayoutBoxMayIntersectProvider(
        box_: &LayoutBox,
        result: &HitTestResult,
        location: &HitTestLocation,
        offset: &PhysicalOffset,
    ) -> bool;
    fn LayoutBoxHasHitTestableOverflowProvider(box_: &LayoutBox) -> bool;
    fn LayoutBoxInvalidateItemsProvider(result: &LayoutResult);
    fn LayoutBoxFragmentDataFromPhysicalFragmentProvider(
        box_: &LayoutBox,
        fragment: &PhysicalBoxFragment,
    ) -> *const FragmentData;
    fn LayoutBoxAutoscrollProvider(box_: &mut LayoutBox, offset: &PhysicalOffset) -> bool;
    fn LayoutBoxCalculateAutoscrollDirectionProvider(
        box_: &LayoutBox,
        point: &gfx::PointF,
    ) -> PhysicalOffset;
    fn LayoutBoxFindAutoscrollableProvider(
        object: *mut LayoutObject,
        middle_click: bool,
    ) -> *mut LayoutBox;
    fn LayoutBoxHasHorizontallyScrollableAncestorProvider(object: *mut LayoutObject) -> bool;
    fn LayoutBoxPositionForPointInFragmentsProvider(
        box_: &LayoutBox,
        offset: &PhysicalOffset,
    ) -> PositionWithAffinity;
    fn LayoutBoxReadingFlowNodesProvider(box_: &LayoutBox) -> *const GCedHeapVector<Member<Node>>;
    fn LayoutBoxHitTestOverflowControlProvider(
        box_: &LayoutBox,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        offset: &PhysicalOffset,
    ) -> bool;
    fn LayoutBoxIntersectsVisibleViewportProvider(box_: &LayoutBox) -> bool;
    fn LayoutBoxEnsureIsReadyForPaintInvalidationProvider(box_: &mut LayoutBox);
    fn LayoutBoxClearPaintFlagsProvider(box_: &mut LayoutBox);
    fn LayoutBoxOverrideTickmarksProvider(box_: &mut LayoutBox, tickmarks: Vector<gfx::Rect>);
    fn LayoutBoxInvalidatePaintForTickmarksProvider(box_: &mut LayoutBox);
    fn LayoutBoxNonOverflowingScrollRangesProvider(
        box_: &LayoutBox,
    ) -> *const GCedHeapVector<NonOverflowingScrollRange>;
    fn LayoutBoxDisplayLocksAffectedByAnchorsProvider(
        box_: &LayoutBox,
    ) -> *const GCedHeapHashSet<Member<Element>>;
    fn LayoutBoxNotifyContainingDisplayLocksForAnchorPositioningProvider(
        box_: &LayoutBox,
        past: *const GCedHeapHashSet<Member<Element>>,
        current: *const GCedHeapHashSet<Member<Element>>,
    );
    fn LayoutBoxBackgroundPaintedExtentProvider(box_: &LayoutBox) -> PhysicalRect;
    fn LayoutBoxForegroundIsKnownToBeOpaqueInRectProvider(
        box_: &LayoutBox,
        rect: &PhysicalRect,
        max_depth: u32,
    ) -> bool;
    fn LayoutBoxComputeBackgroundIsKnownToBeObscuredProvider(box_: &LayoutBox) -> bool;
    fn LayoutBoxInflateVisualRectForFilterProvider(box_: &LayoutBox, state: &mut TransformState);
    fn LayoutBoxBackgroundClipBorderBoxIsEquivalentToPaddingBoxProvider(box_: &LayoutBox) -> bool;
    fn LayoutBoxComputeBackgroundPaintLocationProvider(
        box_: &LayoutBox,
        needs_root_element_group: bool,
    ) -> BackgroundPaintLocation;
    fn LayoutBoxMapContentsRectToBoxSpaceProvider(
        box_: &LayoutBox,
        state: &mut TransformState,
        accumulation: TransformAccumulation,
        contents: &LayoutObject,
        flags: VisualRectFlags,
    ) -> bool;
    fn LayoutBoxMapVisualRectToContainerProvider(
        box_: &LayoutBox,
        container: *const LayoutObject,
        offset: &PhysicalOffset,
        ancestor: *const LayoutObject,
        flags: VisualRectFlags,
        state: &mut TransformState,
    ) -> bool;
    fn LayoutBoxIntrinsicLogicalWidthsBorderSizesProvider(box_: &LayoutBox) -> *const BoxStrut;
    fn LayoutBoxSetIntrinsicLogicalWidthsBorderSizesProvider(
        box_: &mut LayoutBox,
        borders: &BoxStrut,
    );
    fn LayoutBoxContentLayoutBoxProvider(box_: &mut LayoutBox) -> *mut LayoutBox;
    fn LayoutBoxScrollWidthProvider(box_: &LayoutBox) -> LayoutUnit;
    fn LayoutBoxScrollHeightProvider(box_: &LayoutBox) -> LayoutUnit;
    fn LayoutBoxUpdateScrollSnapMappingAfterStyleChangeProvider(
        box_: &mut LayoutBox,
        old_style: &ComputedStyle,
    );
    fn LayoutBoxInflateVisualRectForFilterUnderContainerProvider(
        box_: &LayoutBox,
        state: &mut TransformState,
        container: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
    );
    fn LayoutBoxPaintProvider(box_: &LayoutBox, info: &PaintInfo);
    fn LayoutBoxLocalCaretRectProvider(
        box_: &LayoutBox,
        offset: i32,
        shape: CaretShape,
    ) -> PhysicalRect;
    fn LayoutBoxComputeResourcePriorityProvider(box_: &LayoutBox) -> ResourcePriority;
    fn LayoutBoxInvalidatePaintProvider(box_: &LayoutBox, context: &PaintInvalidatorContext);
    fn LayoutBoxGetCustomLayoutChildProvider(box_: &LayoutBox) -> *mut CustomLayoutChild;
    fn LayoutBoxMapToVisualRectInAncestorSpaceInternalProvider(
        box_: &LayoutBox,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: VisualRectFlags,
    ) -> bool;
    fn LayoutBoxBackgroundIsKnownToBeOpaqueInRectProvider(
        box_: &LayoutBox,
        rect: &PhysicalRect,
    ) -> bool;
    fn LayoutBoxBackgroundShouldAlwaysBeClippedProvider(box_: &LayoutBox) -> bool;
    fn LayoutBoxImageChangedProvider(
        box_: &mut LayoutBox,
        image: WrappedImagePtr,
        defer: CanDeferInvalidation,
    );
    fn LayoutBoxCreateAnonymousBoxWithSameTypeAsProvider(
        box_: &LayoutBox,
        object: *const LayoutObject,
    ) -> *mut LayoutBox;
    fn LayoutBoxIsEligibleForPaintOrLayoutContainmentProvider(box_: &LayoutBox) -> bool;
    fn LayoutBoxIsEligibleForSizeContainmentProvider(box_: &LayoutBox) -> bool;
    fn LayoutBoxComputeCanCompositeBackgroundAttachmentFixedProvider(box_: &LayoutBox) -> bool;
}

// C++ derives this short-lived mutator from LayoutObject::MutableForPainting.
// The base mutator retains the same LayoutObject while the raw pointer
// addresses LayoutBox's additional previous-geometry fields.
// cpp: layoutng/internal/layout_box.h:959-1005
pub struct LayoutBoxMutableForPainting {
    base_: LayoutObjectMutableForPainting,
    box_: *mut LayoutBox,
}

impl LayoutBoxMutableForPainting {
    pub fn GetLayoutBox(&mut self) -> &mut LayoutBox {
        unsafe { &mut *self.box_ }
    }

    // cpp: layoutng/internal/layout_box.h:961-966
    pub fn SavePreviousSize(&mut self) {
        let size = unsafe { &*self.box_ }.StitchedSize();
        self.GetLayoutBox().previous_size_ = size;
    }

    pub fn ClearPreviousSize(&mut self) {
        self.GetLayoutBox().previous_size_ = PhysicalSize::default();
    }

    pub fn SavePreviousOverflowData(&mut self) {
        unsafe { LayoutBoxMutableSavePreviousOverflowDataProvider(self.GetLayoutBox()) }
    }

    // cpp: layoutng/internal/layout_box.h:966-980
    pub fn ClearPreviousOverflowData(&mut self) {
        let box_ = self.GetLayoutBox();
        debug_assert!(!box_.HasVisualOverflow());
        debug_assert!(!box_.HasScrollableOverflow());
        box_.overflow_ = Member::default();
    }

    pub fn SavePreviousContentBoxRect(&mut self) {
        let rect = unsafe { &*self.box_ }.PhysicalContentBoxRect();
        let rare = self.GetLayoutBox().EnsureRareData();
        rare.has_previous_content_box_rect_ = true;
        rare.previous_physical_content_box_rect_ = rect;
    }

    pub fn ClearPreviousContentBoxRect(&mut self) {
        let rare = self.GetLayoutBox().rare_data_.Get();
        if !rare.is_null() {
            unsafe { &mut *rare }.has_previous_content_box_rect_ = false;
        }
    }

    // cpp: layoutng/internal/layout_box.h:981-993
    pub fn SavePreviousGapGeometries(&mut self) {
        unsafe { LayoutBoxMutableSavePreviousGapGeometriesProvider(self.GetLayoutBox()) }
    }

    pub fn ClearPreviousGapGeometries(&mut self) {
        unsafe { LayoutBoxMutableClearPreviousGapGeometriesProvider(self.GetLayoutBox()) }
    }

    pub fn SetPreviousGeometryForLayoutShiftTracking(
        &mut self,
        paint_offset: &PhysicalOffset,
        size: &PhysicalSize,
        visual_overflow_rect: &PhysicalRect,
    ) {
        unsafe {
            LayoutBoxMutableSetPreviousGeometryForLayoutShiftTrackingProvider(
                self.GetLayoutBox(),
                paint_offset,
                size,
                visual_overflow_rect,
            )
        }
    }

    pub fn UpdateBackgroundPaintLocation(&mut self, needs_root_element_group: bool) {
        unsafe {
            LayoutBoxMutableUpdateBackgroundPaintLocationProvider(
                self.GetLayoutBox(),
                needs_root_element_group,
            )
        }
    }
}

// cpp: layoutng/internal/layout_box.h:536-600
// cpp: layoutng/internal/layout_box_fragment_data.cc:157-184,468-479
pub struct PhysicalFragmentList<'a> {
    layout_results_: &'a LayoutResultList,
}

pub struct PhysicalFragmentIterator<'a> {
    list: PhysicalFragmentList<'a>,
    index: usize,
}

impl<'a> PhysicalFragmentList<'a> {
    pub fn new(layout_results: &'a LayoutResultList) -> Self {
        Self {
            layout_results_: layout_results,
        }
    }

    pub fn Size(&self) -> usize {
        self.layout_results_.len()
    }

    pub fn IsEmpty(&self) -> bool {
        self.layout_results_.is_empty()
    }

    pub fn MayHaveFragmentItems(&self) -> bool {
        !self.IsEmpty() && self.front().IsInlineFormattingContext()
    }

    pub fn HasFragmentItems(&self) -> bool {
        self.MayHaveFragmentItems() && self.SlowHasFragmentItems()
    }

    pub fn SlowHasFragmentItems(&self) -> bool {
        self.into_iter().any(PhysicalBoxFragment::HasItems)
    }

    pub fn IndexOf(&self, fragment: &PhysicalBoxFragment) -> usize {
        for (index, result) in self.layout_results_.iter().enumerate() {
            if unsafe { &*result.Get() }.GetPhysicalFragment() as *const _
                == fragment as *const PhysicalBoxFragment as *const _
            {
                return index;
            }
        }
        usize::MAX
    }

    pub fn Contains(&self, fragment: &PhysicalBoxFragment) -> bool {
        self.IndexOf(fragment) != usize::MAX
    }

    pub fn front(&self) -> &PhysicalBoxFragment {
        self.at(0)
    }

    pub fn back(&self) -> &PhysicalBoxFragment {
        self.at(self.Size() - 1)
    }

    fn at(&self, index: usize) -> &PhysicalBoxFragment {
        let physical = unsafe { &*self.layout_results_[index].Get() }.GetPhysicalFragment();
        let box_fragment = To::<PhysicalBoxFragment>(physical as *const _);
        unsafe { &*box_fragment }
    }
}

impl<'a> IntoIterator for PhysicalFragmentList<'a> {
    type Item = &'a PhysicalBoxFragment;
    type IntoIter = PhysicalFragmentIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PhysicalFragmentIterator {
            list: self,
            index: 0,
        }
    }
}

impl<'a> IntoIterator for &PhysicalFragmentList<'a> {
    type Item = &'a PhysicalBoxFragment;
    type IntoIter = PhysicalFragmentIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PhysicalFragmentIterator {
            list: PhysicalFragmentList::new(self.layout_results_),
            index: 0,
        }
    }
}

impl<'a> Iterator for PhysicalFragmentIterator<'a> {
    type Item = &'a PhysicalBoxFragment;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.list.Size() {
            return None;
        }
        let index = self.index;
        self.index += 1;
        let physical = unsafe { &*self.list.layout_results_[index].Get() }.GetPhysicalFragment();
        let box_fragment = To::<PhysicalBoxFragment>(physical as *const _);
        Some(unsafe { &*box_fragment })
    }
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:42-66
fn CheckDidAddFragment(
    box_: &LayoutBox,
    new_fragment: &PhysicalBoxFragment,
    new_fragment_index: usize,
) {
    #[cfg(debug_assertions)]
    {
        if new_fragment.HasItems() {
            debug_assert!(box_.ChildrenInline());
        }
        for (index, fragment) in box_.PhysicalFragments().into_iter().enumerate() {
            debug_assert_eq!(fragment.IsFirstForNode(), index == 0);
            let items = fragment.Items();
            if !items.is_null() {
                unsafe { &*items }.CheckAllItemsAreValid();
            }
            if index == new_fragment_index {
                break;
            }
        }
    }
    #[cfg(not(debug_assertions))]
    let _ = (box_, new_fragment, new_fragment_index);
}

// cpp: layoutng/internal/layout_box.h:299-302
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContractionEdge {
    kContractToPaddingEdge,
    kContractToContentEdge,
}

// cpp: layoutng/internal/fragmentation_utils.cc:1404-1434
pub(crate) fn FindPreviousBreakToken(fragment: &PhysicalBoxFragment) -> *const BlockBreakToken {
    let box_ = To::<LayoutBox>(fragment.GetLayoutObject());
    debug_assert!(!box_.is_null());
    let box_ = unsafe { &*box_ };
    debug_assert!(box_.PhysicalFragmentCount() >= 1);
    if fragment.IsFirstForNode() {
        return std::ptr::null();
    }
    debug_assert!(box_.PhysicalFragmentCount() > 1);
    let break_token = fragment.GetBreakToken();
    let previous_fragment = if !break_token.is_null() {
        let sequence_number = unsafe { &*break_token }.SequenceNumber() as usize;
        debug_assert!(sequence_number >= 1);
        box_.GetPhysicalFragment(sequence_number - 1)
    } else {
        box_.GetPhysicalFragment(box_.PhysicalFragmentCount() - 2)
    };
    unsafe { &*previous_fragment }.GetBreakToken()
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:69-90
fn FragmentForEdge(box_: &LayoutBox, edges: PhysicalBoxSides) -> *const PhysicalBoxFragment {
    debug_assert!(box_.PhysicalFragmentCount() > 1);
    debug_assert_eq!(
        u8::from(edges.top) + u8::from(edges.right) + u8::from(edges.bottom) + u8::from(edges.left),
        1
    );
    let logical_edges = edges.ToLogical(box_.StyleRef().GetWritingDirection());
    if logical_edges.block_end {
        for idx in (1..box_.PhysicalFragmentCount()).rev() {
            let fragment = box_.GetPhysicalFragment(idx);
            let previous_token = FindPreviousBreakToken(unsafe { &*fragment });
            if !unsafe { &*previous_token }.IsAtBlockEnd() {
                return fragment;
            }
        }
    }
    box_.GetPhysicalFragment(0)
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:93-97
fn FragmentForLeftEdge(box_: &LayoutBox) -> *const PhysicalBoxFragment {
    FragmentForEdge(box_, PhysicalBoxSides::new(false, false, false, true))
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:99-103
fn FragmentForRightEdge(box_: &LayoutBox) -> *const PhysicalBoxFragment {
    FragmentForEdge(box_, PhysicalBoxSides::new(false, true, false, false))
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:105-109
fn FragmentForTopEdge(box_: &LayoutBox) -> *const PhysicalBoxFragment {
    FragmentForEdge(box_, PhysicalBoxSides::new(true, false, false, false))
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:111-115
fn FragmentForBottomEdge(box_: &LayoutBox) -> *const PhysicalBoxFragment {
    FragmentForEdge(box_, PhysicalBoxSides::new(false, false, true, false))
}

// LayoutBox owns exactly one LayoutBoxModelObject base, kept first so the
// explicit non-virtual base calls in translated methods reach that subobject.
// cpp: layoutng/internal/layout_box.h:205-206
// cpp: layoutng/internal/layout_box.h:1252-1253
// cpp: layoutng/internal/layout_box.h:1328-1355
#[repr(C)]
pub struct LayoutBox {
    model_object_: LayoutBoxModelObject,
    pub(crate) scrollbar_freeze_state_: ScrollbarFreezeState,
    pub(crate) static_position_for_layout_: LogicalStaticPosition,
    pub(crate) frame_location_: PhysicalOffset,
    pub(crate) frame_size_: Cell<PhysicalSize>,
    pub(crate) previous_size_: PhysicalSize,
    pub(crate) intrinsic_logical_widths_: MinMaxSizes,
    pub(crate) min_max_sizes_cache_: Member<MinMaxSizesCache>,
    pub(crate) measure_cache_: Member<MeasureCache>,
    pub(crate) layout_results_: LayoutResultList,
    pub(crate) first_fragment_item_index_: usize,
    pub(crate) overflow_: Member<BoxOverflowModel>,
    pub(crate) rare_data_: Member<LayoutBoxRareData>,
}

// The C++ DowncastTraits<LayoutBox> accepts only LayoutObject::IsBox().
// LayoutBox -> LayoutBoxModelObject -> LayoutObject are all first fields.
// cpp: layoutng/internal/layout_box.h:1358-1361
impl foundation::DowncastFrom<LayoutObject> for LayoutBox {
    fn AllowFrom(object: &LayoutObject) -> bool {
        Self::AllowFrom(object)
    }
}

// cpp: layoutng/internal/layout_box.h:1358-1361
impl foundation::DowncastFrom<LayoutBoxModelObject> for LayoutBox {
    fn AllowFrom(object: &LayoutBoxModelObject) -> bool {
        object.IsBox()
    }
}

const _: () = assert!(std::mem::offset_of!(LayoutBox, model_object_) == 0);

impl LayoutBox {
    // cpp: layoutng/internal/layout_box.h:1358-1361
    pub fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsBox()
    }

    // cpp: layoutng/internal/layout_box.h:220-226,768-768,901-905
    pub fn BackgroundIsKnownToBeOpaqueInRect(&self, rect: &PhysicalRect) -> bool {
        unsafe { LayoutBoxBackgroundIsKnownToBeOpaqueInRectProvider(self, rect) }
    }

    pub fn BackgroundShouldAlwaysBeClipped(&self) -> bool {
        unsafe { LayoutBoxBackgroundShouldAlwaysBeClippedProvider(self) }
    }

    pub fn ImageChanged(&mut self, image: WrappedImagePtr, defer: CanDeferInvalidation) {
        unsafe { LayoutBoxImageChangedProvider(self, image, defer) }
    }

    pub fn CreateAnonymousBoxWithSameTypeAs(&self, object: *const LayoutObject) -> *mut LayoutBox {
        unsafe { LayoutBoxCreateAnonymousBoxWithSameTypeAsProvider(self, object) }
    }

    // cpp: layoutng/internal/layout_box.h:1181-1184,1195-1198,1229-1229,1286-1289
    pub fn IsEligibleForPaintOrLayoutContainment(&self) -> bool {
        unsafe { LayoutBoxIsEligibleForPaintOrLayoutContainmentProvider(self) }
    }

    pub fn IsEligibleForSizeContainment(&self) -> bool {
        unsafe { LayoutBoxIsEligibleForSizeContainmentProvider(self) }
    }

    pub fn ComputeCanCompositeBackgroundAttachmentFixed(&self) -> bool {
        unsafe { LayoutBoxComputeCanCompositeBackgroundAttachmentFixedProvider(self) }
    }

    pub fn IsBox(&self) -> bool {
        self.IsBoxBase()
    }

    // cpp: layoutng/internal/layout_box.h:660-663
    pub fn MapToVisualRectInAncestorSpaceInternal(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: VisualRectFlags,
    ) -> bool {
        unsafe {
            LayoutBoxMapToVisualRectInAncestorSpaceInternalProvider(self, ancestor, state, flags)
        }
    }

    // cpp: layoutng/internal/layout_box.h:936-936
    pub fn GetCustomLayoutChild(&self) -> *mut CustomLayoutChild {
        unsafe { LayoutBoxGetCustomLayoutChildProvider(self) }
    }

    // cpp: layoutng/internal/layout_box.h:435-435,748-748,769-769,1231-1231
    pub fn Paint(&self, info: &PaintInfo) {
        unsafe { LayoutBoxPaintProvider(self, info) }
    }

    pub fn LocalCaretRect(&self, offset: i32, shape: CaretShape) -> PhysicalRect {
        unsafe { LayoutBoxLocalCaretRectProvider(self, offset, shape) }
    }

    pub fn ComputeResourcePriority(&self) -> ResourcePriority {
        unsafe { LayoutBoxComputeResourcePriorityProvider(self) }
    }

    pub fn InvalidatePaint(&self, context: &PaintInvalidatorContext) {
        unsafe { LayoutBoxInvalidatePaintProvider(self, context) }
    }

    // cpp: layoutng/internal/layout_box.h:872-895,909-928
    pub fn MapContentsRectToBoxSpace(
        &self,
        state: &mut TransformState,
        accumulation: TransformAccumulation,
        contents: &LayoutObject,
        flags: VisualRectFlags,
    ) -> bool {
        unsafe {
            LayoutBoxMapContentsRectToBoxSpaceProvider(self, state, accumulation, contents, flags)
        }
    }

    pub fn MapVisualRectToContainer(
        &self,
        container: *const LayoutObject,
        offset: &PhysicalOffset,
        ancestor: *const LayoutObject,
        flags: VisualRectFlags,
        state: &mut TransformState,
    ) -> bool {
        unsafe {
            LayoutBoxMapVisualRectToContainerProvider(
                self, container, offset, ancestor, flags, state,
            )
        }
    }

    pub fn InvalidateLayoutResultCacheAfterMeasure(&self) {
        unsafe { InvalidateLayoutResultCacheAfterMeasureProvider(self) }
    }

    pub fn IntrinsicLogicalWidthsBorderSizes(&self) -> &BoxStrut {
        unsafe { &*LayoutBoxIntrinsicLogicalWidthsBorderSizesProvider(self) }
    }

    pub fn SetIntrinsicLogicalWidthsBorderSizes(&mut self, borders: &BoxStrut) {
        unsafe { LayoutBoxSetIntrinsicLogicalWidthsBorderSizesProvider(self, borders) }
    }

    pub fn ContentLayoutBox(&mut self) -> *mut LayoutBox {
        unsafe { LayoutBoxContentLayoutBoxProvider(self) }
    }

    // cpp: layoutng/internal/layout_box.h:1277-1277,1294-1297
    pub fn UpdateScrollSnapMappingAfterStyleChange(&mut self, old_style: &ComputedStyle) {
        unsafe { LayoutBoxUpdateScrollSnapMappingAfterStyleChangeProvider(self, old_style) }
    }

    pub fn InflateVisualRectForFilterUnderContainer(
        &self,
        state: &mut TransformState,
        container: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
    ) {
        unsafe {
            LayoutBoxInflateVisualRectForFilterUnderContainerProvider(
                self, state, container, ancestor,
            )
        }
    }

    // cpp: layoutng/internal/layout_box.h:327-327,424-424,433-433,442-450
    pub fn PhysicalBackgroundRect(&self, kind: BackgroundRectType) -> PhysicalRect {
        unsafe { LayoutBoxPhysicalBackgroundRectProvider(self, kind) }
    }

    pub fn ScrollerFromScrollMarkerGroup(&self) -> *mut LayoutBlock {
        unsafe { LayoutBoxScrollerFromScrollMarkerGroupProvider(self) }
    }

    pub fn LayoutSubtreeRoot(&mut self) {
        unsafe { LayoutBoxLayoutSubtreeRootProvider(self) }
    }

    pub fn HitTestAllPhases(
        &mut self,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        offset: &PhysicalOffset,
    ) -> bool {
        unsafe { LayoutBoxHitTestAllPhasesProvider(self, result, location, offset) }
    }

    pub fn MayIntersect(
        &self,
        result: &HitTestResult,
        location: &HitTestLocation,
        offset: &PhysicalOffset,
    ) -> bool {
        unsafe { LayoutBoxMayIntersectProvider(self, result, location, offset) }
    }

    pub fn HasHitTestableOverflow(&self) -> bool {
        unsafe { LayoutBoxHasHitTestableOverflowProvider(self) }
    }

    // cpp: layoutng/internal/layout_box.h:461-461,612-613,690-695
    pub fn InvalidateItems(result: &LayoutResult) {
        unsafe { LayoutBoxInvalidateItemsProvider(result) }
    }

    pub fn FragmentDataFromPhysicalFragment(
        &self,
        fragment: &PhysicalBoxFragment,
    ) -> *const FragmentData {
        unsafe { LayoutBoxFragmentDataFromPhysicalFragmentProvider(self, fragment) }
    }

    pub fn Autoscroll(&mut self, offset: &PhysicalOffset) -> bool {
        unsafe { LayoutBoxAutoscrollProvider(self, offset) }
    }

    pub fn CalculateAutoscrollDirection(&self, point: &gfx::PointF) -> PhysicalOffset {
        unsafe { LayoutBoxCalculateAutoscrollDirectionProvider(self, point) }
    }

    pub fn FindAutoscrollable(object: *mut LayoutObject, middle_click: bool) -> *mut LayoutBox {
        unsafe { LayoutBoxFindAutoscrollableProvider(object, middle_click) }
    }

    pub fn HasHorizontallyScrollableAncestor(object: *mut LayoutObject) -> bool {
        unsafe { LayoutBoxHasHorizontallyScrollableAncestorProvider(object) }
    }

    // cpp: layoutng/internal/layout_box.h:771-771,849-849,948-956
    pub fn PositionForPointInFragments(&self, offset: &PhysicalOffset) -> PositionWithAffinity {
        unsafe { LayoutBoxPositionForPointInFragmentsProvider(self, offset) }
    }

    pub fn ReadingFlowNodes(&self) -> &GCedHeapVector<Member<Node>> {
        unsafe { &*LayoutBoxReadingFlowNodesProvider(self) }
    }

    pub fn HitTestOverflowControl(
        &self,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        offset: &PhysicalOffset,
    ) -> bool {
        unsafe { LayoutBoxHitTestOverflowControlProvider(self, result, location, offset) }
    }

    pub fn IntersectsVisibleViewport(&self) -> bool {
        unsafe { LayoutBoxIntersectsVisibleViewportProvider(self) }
    }

    pub fn EnsureIsReadyForPaintInvalidation(&mut self) {
        unsafe { LayoutBoxEnsureIsReadyForPaintInvalidationProvider(self) }
    }

    pub fn ClearPaintFlags(&mut self) {
        unsafe { LayoutBoxClearPaintFlagsProvider(self) }
    }

    // cpp: layoutng/internal/layout_box.h:1111-1115,1162-1172
    pub fn OverrideTickmarks(&mut self, tickmarks: Vector<gfx::Rect>) {
        unsafe { LayoutBoxOverrideTickmarksProvider(self, tickmarks) }
    }

    pub fn InvalidatePaintForTickmarks(&mut self) {
        unsafe { LayoutBoxInvalidatePaintForTickmarksProvider(self) }
    }

    pub fn NonOverflowingScrollRanges(&self) -> *const GCedHeapVector<NonOverflowingScrollRange> {
        unsafe { LayoutBoxNonOverflowingScrollRangesProvider(self) }
    }

    pub fn DisplayLocksAffectedByAnchors(&self) -> *const GCedHeapHashSet<Member<Element>> {
        unsafe { LayoutBoxDisplayLocksAffectedByAnchorsProvider(self) }
    }

    pub fn NotifyContainingDisplayLocksForAnchorPositioning(
        &self,
        past: *const GCedHeapHashSet<Member<Element>>,
        current: *const GCedHeapHashSet<Member<Element>>,
    ) {
        unsafe {
            LayoutBoxNotifyContainingDisplayLocksForAnchorPositioningProvider(self, past, current)
        }
    }

    // cpp: layoutng/internal/layout_box.h:1224-1229,1293-1294,1310-1312
    pub fn BackgroundPaintedExtent(&self) -> PhysicalRect {
        unsafe { LayoutBoxBackgroundPaintedExtentProvider(self) }
    }

    pub fn ForegroundIsKnownToBeOpaqueInRect(&self, rect: &PhysicalRect, max_depth: u32) -> bool {
        unsafe { LayoutBoxForegroundIsKnownToBeOpaqueInRectProvider(self, rect, max_depth) }
    }

    pub fn ComputeBackgroundIsKnownToBeObscured(&self) -> bool {
        unsafe { LayoutBoxComputeBackgroundIsKnownToBeObscuredProvider(self) }
    }

    pub fn InflateVisualRectForFilter(&self, state: &mut TransformState) {
        unsafe { LayoutBoxInflateVisualRectForFilterProvider(self, state) }
    }

    pub fn BackgroundClipBorderBoxIsEquivalentToPaddingBox(&self) -> bool {
        unsafe { LayoutBoxBackgroundClipBorderBoxIsEquivalentToPaddingBoxProvider(self) }
    }

    pub fn ComputeBackgroundPaintLocation(
        &self,
        needs_root_element_group: bool,
    ) -> BackgroundPaintLocation {
        unsafe { LayoutBoxComputeBackgroundPaintLocationProvider(self, needs_root_element_group) }
    }

    // cpp: layoutng/internal/layout_box.h:398-399,421-421
    pub fn DefaultIntrinsicContentInlineSize(&self) -> LayoutUnit {
        LayoutBoxDefaultIntrinsicContentInlineSize(self)
    }

    pub fn DefaultIntrinsicContentBlockSize(&self, children_have_geometry: bool) -> LayoutUnit {
        LayoutBoxDefaultIntrinsicContentBlockSize(self, children_have_geometry)
    }

    pub fn GetScrollMarkerGroup(&mut self) -> *mut LayoutBlock {
        LayoutBoxGetScrollMarkerGroup(self)
    }

    // cpp: layoutng/internal/layout_box.h:1155-1160,1248-1249
    pub fn FindTargetAnchor(&self, name: &ScopedCSSName) -> *const LayoutObject {
        LayoutBoxFindTargetAnchor(self, name)
    }

    pub fn AcceptableImplicitAnchor(&self) -> *const LayoutObject {
        LayoutBoxAcceptableImplicitAnchor(self)
    }

    pub fn RecalcScrollableOverflowNG(&mut self) -> RecalcScrollableOverflowResult {
        LayoutBoxRecalcScrollableOverflowNG(self)
    }

    pub fn RecalcChildScrollableOverflowNG(&mut self) -> RecalcScrollableOverflowResult {
        LayoutBoxRecalcChildScrollableOverflowNG(self)
    }

    // cpp: layoutng/internal/layout_box.h:1004-1007
    pub fn GetMutableForPainting(&self) -> LayoutBoxMutableForPainting {
        self.CheckIsNotDestroyed();
        let object = unsafe { &*(self as *const LayoutBox as *const LayoutObject) };
        LayoutBoxMutableForPainting {
            base_: object.GetMutableForPainting(),
            box_: self as *const LayoutBox as *mut LayoutBox,
        }
    }

    // cpp: layoutng/internal/layout_box.h:901-905
    pub fn CreateAnonymousBoxWithSameTypeAsBase(
        &self,
        _object: *const LayoutObject,
    ) -> *mut LayoutBox {
        self.CheckIsNotDestroyed();
        panic!("LayoutBox::CreateAnonymousBoxWithSameTypeAs is unreachable for this type")
    }

    // cpp: layoutng/internal/layout_box.h:909-909
    pub fn InvalidateLayoutResultCacheAfterMeasureBase(&self) {}

    // cpp: layoutng/internal/layout_box.h:913-921
    pub fn IntrinsicLogicalWidthsBorderSizesBase(&self) -> &BoxStrut {
        self.CheckIsNotDestroyed();
        panic!("LayoutBox::IntrinsicLogicalWidthsBorderSizes requires a derived table cell")
    }

    pub fn SetIntrinsicLogicalWidthsBorderSizesBase(&mut self, _borders: &BoxStrut) {
        self.CheckIsNotDestroyed();
        panic!("LayoutBox::SetIntrinsicLogicalWidthsBorderSizes requires a derived table cell")
    }

    // cpp: layoutng/internal/layout_box.h:926-930
    pub fn ContentLayoutBoxBase(&mut self) -> *mut LayoutBox {
        self.CheckIsNotDestroyed();
        self
    }

    // cpp: layoutng/internal/layout_box.h:937-944
    pub fn SetCustomLayoutChild(
        &mut self,
        child: *mut CustomLayoutChild,
        clear_child: Option<fn(*mut CustomLayoutChild)>,
    ) {
        assert!(!child.is_null());
        assert!(clear_child.is_some());
        let data = self.EnsureRareData();
        data.layout_child_ = Member::from_ptr(child);
        data.clear_layout_child_ = clear_child;
    }

    // cpp: layoutng/internal/layout_box.h:437-440
    pub fn IsInSelfHitTestingPhase(&self, phase: HitTestPhase) -> bool {
        self.CheckIsNotDestroyed();
        phase == HitTestPhase::kForeground
    }

    // cpp: layoutng/internal/layout_box.h:619-622
    pub fn IsFragmentLessBox(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.PhysicalFragmentCount() == 0
    }

    // cpp: layoutng/internal/layout_box.h:624-627
    pub fn IsValidColumnSpannerInTreeWithCurrentStyle(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsValidColumnSpannerInTree(self.StyleRef())
    }

    // cpp: layoutng/internal/layout_box.h:797-816
    pub fn IsFlexItem(&self) -> bool {
        self.CheckIsNotDestroyed();
        let parent = self.Parent();
        !self.IsInline()
            && !self.IsOutOfFlowPositioned()
            && !parent.is_null()
            && unsafe { &*parent }.IsFlexibleBox()
    }

    pub fn IsGridItem(&self) -> bool {
        self.CheckIsNotDestroyed();
        let parent = self.Parent();
        !parent.is_null() && unsafe { &*parent }.IsLayoutGrid()
    }

    pub fn IsGridLanesItem(&self) -> bool {
        self.CheckIsNotDestroyed();
        let parent = self.Parent();
        !parent.is_null() && unsafe { &*parent }.IsLayoutGridLanes()
    }

    pub fn IsMathItem(&self) -> bool {
        self.CheckIsNotDestroyed();
        let parent = self.Parent();
        !parent.is_null() && unsafe { &*parent }.IsMathML()
    }

    // cpp: layoutng/internal/layout_box.h:1181-1198
    pub fn IsEligibleForPaintOrLayoutContainmentBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    pub fn SetStaticPositionForLayout(&mut self, position: &LogicalStaticPosition) {
        self.static_position_for_layout_ = *position;
    }

    pub fn StaticPositionForLayout(&self) -> &LogicalStaticPosition {
        &self.static_position_for_layout_
    }

    pub fn IsEligibleForSizeContainmentBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng/internal/layout_box.h:1216-1219
    pub fn ShouldBeHandledAsFloatingCurrentStyle(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.ShouldBeHandledAsFloating(self.StyleRef())
    }

    // cpp: layoutng/internal/layout_box.h:1286-1289
    pub fn IsBoxBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng/internal/layout_box.h:356-367
    pub fn SelfVisualOverflowRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let overflow = self.overflow_.Get();
        if !overflow.is_null() {
            if let Some(visual) = unsafe { &*overflow }.visual_overflow.as_ref() {
                return *visual.SelfVisualOverflowRect();
            }
        }
        self.PhysicalBorderBoxRect()
    }

    pub fn ContentsVisualOverflowRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let overflow = self.overflow_.Get();
        if !overflow.is_null() {
            if let Some(visual) = unsafe { &*overflow }.visual_overflow.as_ref() {
                return *visual.ContentsVisualOverflowRect();
            }
        }
        PhysicalRect::default()
    }

    // cpp: layoutng/internal/layout_box.h:705-721
    pub fn ScrollsOverflow(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.HasNonVisibleOverflow() && self.StyleRef().ScrollsOverflow()
    }

    pub fn ShouldPlaceVerticalScrollbarOnLeft(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.ShouldPlaceBlockDirectionScrollbarOnLogicalLeft()
    }

    pub fn ShouldPlaceBlockDirectionScrollbarOnLogicalLeftBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef()
            .ShouldPlaceBlockDirectionScrollbarOnLogicalLeft()
    }

    pub fn ShouldPlaceBlockDirectionScrollbarOnLogicalLeft(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBoxShouldPlaceBlockDirectionScrollbarOnLogicalLeftProvider(self) }
    }

    // cpp: layoutng/internal/layout_box.h:773-776
    pub fn CreatesNewFormattingContextBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    pub fn CreatesNewFormattingContext(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBoxCreatesNewFormattingContextProvider(self) }
    }

    // cpp: layoutng/internal/layout_box.h:828-837
    pub fn HasSelfVisualOverflow(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.VisualOverflowIsSet()
            && !self
                .PhysicalBorderBoxRect()
                .ContainsRect(&self.SelfVisualOverflowRect())
    }

    pub fn HasVisualOverflow(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.VisualOverflowIsSet()
    }

    // cpp: layoutng/internal/layout_box.h:223-226
    pub fn BackgroundShouldAlwaysBeClippedBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_box.h:281-287
    pub fn SetLocation(&mut self, location: PhysicalOffset) {
        self.CheckIsNotDestroyed();
        if location == self.frame_location_ {
            return;
        }
        self.frame_location_ = location;
    }

    // cpp: layoutng/internal/layout_box.h:1009-1044
    pub fn PreviousSize(&self) -> PhysicalSize {
        self.CheckIsNotDestroyed();
        self.previous_size_
    }

    pub fn PreviousPhysicalContentBoxRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let rare = self.rare_data_.Get();
        if !rare.is_null() && unsafe { &*rare }.has_previous_content_box_rect_ {
            unsafe { &*rare }.previous_physical_content_box_rect_
        } else {
            PhysicalRect::new(PhysicalOffset::default(), self.PreviousSize())
        }
    }

    pub fn PreviousVisualOverflowRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let overflow = self.overflow_.Get();
        if !overflow.is_null() {
            if let Some(previous) = unsafe { &*overflow }.previous_overflow_data.as_ref() {
                return previous.previous_visual_overflow_rect;
            }
        }
        PhysicalRect::new(PhysicalOffset::default(), self.PreviousSize())
    }

    pub fn PreviousScrollableOverflowRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let overflow = self.overflow_.Get();
        if !overflow.is_null() {
            if let Some(previous) = unsafe { &*overflow }.previous_overflow_data.as_ref() {
                return previous.previous_scrollable_overflow_rect;
            }
        }
        PhysicalRect::new(PhysicalOffset::default(), self.PreviousSize())
    }

    pub fn PreviousSelfVisualOverflowRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let overflow = self.overflow_.Get();
        if !overflow.is_null() {
            if let Some(previous) = unsafe { &*overflow }.previous_overflow_data.as_ref() {
                return previous.previous_self_visual_overflow_rect;
            }
        }
        PhysicalRect::new(PhysicalOffset::default(), self.PreviousSize())
    }

    pub fn PreviousGapGeometries(&self) -> *const GCedHeapVector<Member<GapGeometry>> {
        self.CheckIsNotDestroyed();
        let rare = self.rare_data_.Get();
        if rare.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*rare }.previous_gap_geometries_.Get()
        }
    }

    // cpp: layoutng/internal/layout_box.h:1047-1077
    pub fn CachedIndefiniteIntrinsicLogicalWidths(&self) -> MinMaxSizesResult {
        self.CheckIsNotDestroyed();
        debug_assert!(!self.IntrinsicLogicalWidthsDirty());
        MinMaxSizesResult::new(
            self.intrinsic_logical_widths_,
            self.IntrinsicLogicalWidthsDependsOnBlockConstraints(),
        )
    }

    pub fn CachedIntrinsicLogicalWidths(
        &self,
        initial_block_size: LayoutUnit,
    ) -> Option<MinMaxSizesResult> {
        self.CheckIsNotDestroyed();
        debug_assert!(!self.IntrinsicLogicalWidthsDirty());
        if initial_block_size == kIndefiniteSize {
            if self.IndefiniteIntrinsicLogicalWidthsDirty() {
                return None;
            }
            return Some(MinMaxSizesResult::new(
                self.intrinsic_logical_widths_,
                self.IntrinsicLogicalWidthsDependsOnBlockConstraints(),
            ));
        }
        let cache = self.min_max_sizes_cache_.Get();
        if !cache.is_null() {
            if self.DefiniteIntrinsicLogicalWidthsDirty() {
                return None;
            }
            return unsafe { &mut *cache }.Find(initial_block_size);
        }
        None
    }

    // cpp: layoutng/internal/layout_box.h:1079-1107
    pub fn SetIntrinsicLogicalWidths(
        &mut self,
        initial_block_size: LayoutUnit,
        result: &MinMaxSizesResult,
    ) {
        self.CheckIsNotDestroyed();
        if initial_block_size == kIndefiniteSize || !result.depends_on_block_constraints {
            self.intrinsic_logical_widths_ = result.sizes;
            self.SetIntrinsicLogicalWidthsDependsOnBlockConstraints(
                result.depends_on_block_constraints,
            );
            self.SetIndefiniteIntrinsicLogicalWidthsDirty(false);
        } else {
            let mut cache = self.min_max_sizes_cache_.Get();
            if cache.is_null() {
                cache = MakeGarbageCollected(MinMaxSizesCache::default());
                self.min_max_sizes_cache_ = Member::from_ptr(cache);
            } else if self.DefiniteIntrinsicLogicalWidthsDirty() {
                unsafe { &mut *cache }.Clear();
            }
            unsafe { &mut *cache }.Add(
                &result.sizes,
                initial_block_size,
                result.depends_on_block_constraints,
            );
            self.SetDefiniteIntrinsicLogicalWidthsDirty(false);
        }
        self.ClearIntrinsicLogicalWidthsDirty();
    }

    // cpp: layoutng/internal/layout_box.h:1264-1268
    pub fn VisualOverflowIsSet(&self) -> bool {
        self.CheckIsNotDestroyed();
        #[cfg(debug_assertions)]
        self.CheckIsVisualOverflowComputed();
        let overflow = self.overflow_.Get();
        !overflow.is_null() && unsafe { &*overflow }.visual_overflow.is_some()
    }

    // cpp: layoutng/internal/layout_box.h:723-739
    pub fn ScrollsOverflowX(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.HasNonVisibleOverflow() && self.StyleRef().ScrollsOverflowX()
    }

    pub fn ScrollsOverflowY(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.HasNonVisibleOverflow() && self.StyleRef().ScrollsOverflowY()
    }

    pub fn HasScrollableOverflowX(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.ScrollsOverflowX() && self.ScrollWidth() != self.PhysicalPaddingBoxRect().Width()
    }

    pub fn HasScrollableOverflowY(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.ScrollsOverflowY() && self.ScrollHeight() != self.PhysicalPaddingBoxRect().Height()
    }

    // cpp: layoutng/internal/layout_box.h:839-842
    pub fn HasScrollableOverflow(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.ScrollableOverflowIsSet()
    }

    // cpp: layoutng/internal/layout_box_hot.cc:18-21
    pub fn IsUserScrollable(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.HasScrollableOverflowX() || self.HasScrollableOverflowY()
    }

    // cpp: layoutng/internal/layout_box.h:784-787
    pub fn IsFragmentationContextRootBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsFragmentationContextRoot(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBoxIsFragmentationContextRootProvider(self) }
    }

    // cpp: layoutng/internal/layout_box_hot.cc:23-462
    pub fn CachedLayoutResult(
        &mut self,
        new_space: &ConstraintSpace,
        break_token: *const BlockBreakToken,
        early_break: *const EarlyBreak,
        column_spanner_path: *const ColumnSpannerPath,
        initial_fragment_geometry: *mut Option<FragmentGeometry>,
        out_cache_status: &mut LayoutCacheStatus,
    ) -> *const LayoutResult {
        self.CheckIsNotDestroyed();
        *out_cache_status = LayoutCacheStatus::kNeedsLayout;
        if self.SelfNeedsFullLayout() || self.ShouldSkipLayoutCache() || !early_break.is_null() {
            return std::ptr::null();
        }

        let use_layout_cache_slot = new_space.CacheSlot() == LayoutResultCacheSlot::kLayout
            && !self.layout_results_.is_empty();
        let mut cached_layout_result = if use_layout_cache_slot {
            self.GetCachedLayoutResult(break_token)
        } else {
            self.GetCachedMeasureResult(new_space, initial_fragment_geometry)
        };
        if cached_layout_result.is_null() {
            return std::ptr::null();
        }
        let cached = unsafe { &*cached_layout_result };
        debug_assert_eq!(cached.Status(), EStatus::kSuccess);
        let mut cache_status = LayoutCacheStatus::kHit;
        let physical_fragment =
            unsafe { &*To::<PhysicalBoxFragment>(cached.GetPhysicalFragment() as *const _) };
        let fragment_break_token = physical_fragment.GetBreakToken();
        if (!fragment_break_token.is_null() && unsafe { &*fragment_break_token }.IsRepeated())
            || (!break_token.is_null() && unsafe { &*break_token }.IsRepeated())
        {
            return std::ptr::null();
        }

        let is_blocked_by_display_lock = self.ChildLayoutBlockedByDisplayLock();
        let child_needs_layout = !is_blocked_by_display_lock && self.ChildNeedsFullLayout();
        if self.NeedsSimplifiedLayoutOnly() {
            cache_status = LayoutCacheStatus::kNeedsSimplifiedLayout;
        } else if child_needs_layout {
            if !self.ChildrenInline() || !physical_fragment.HasItems() || !use_layout_cache_slot {
                return std::ptr::null();
            }
            if physical_fragment.NeedsOOFPositionedInfoPropagation()
                || !cached.GetExclusionSpace().IsEmpty()
                || new_space.HasFloats()
                || physical_fragment.HasMovedChildren()
            {
                return std::ptr::null();
            }
            cache_status = LayoutCacheStatus::kCanReuseLines;
        }

        let node = BlockNode::new(self as *mut LayoutBox);
        let mut size_cache_status = LayoutCacheStatus::kHit;
        if use_layout_cache_slot {
            size_cache_status = CalculateSizeBasedLayoutCacheStatus(
                &node,
                break_token,
                cached,
                new_space,
                initial_fragment_geometry,
            );
        }
        if size_cache_status == LayoutCacheStatus::kNeedsLayout {
            return std::ptr::null();
        }
        if cached.HasOrthogonalFallbackSizeDescendant()
            && unsafe { &mut *self.View() }.AffectedByResizedInitialContainingBlock(cached)
        {
            return std::ptr::null();
        }
        if !physical_fragment.ChildrenValid()
            && (size_cache_status == LayoutCacheStatus::kNeedsSimplifiedLayout
                || cache_status == LayoutCacheStatus::kNeedsSimplifiedLayout)
        {
            return std::ptr::null();
        }
        if size_cache_status == LayoutCacheStatus::kNeedsSimplifiedLayout
            && cache_status == LayoutCacheStatus::kHit
        {
            cache_status = LayoutCacheStatus::kNeedsSimplifiedLayout;
        }
        if cache_status == LayoutCacheStatus::kNeedsSimplifiedLayout {
            if self.IsLayoutReplaced()
                || (!use_layout_cache_slot && self.GetCachedLayoutResult(break_token).is_null())
            {
                return std::ptr::null();
            }
        }
        if !break_token.is_null()
            && unsafe { &*break_token }.IsBreakBefore()
            && cached.GetBreakAppeal() < BreakAppeal::kBreakAppealPerfect
        {
            return std::ptr::null();
        }

        let bfc_line_offset = new_space.GetBfcOffset().line_offset;
        let mut bfc_block_offset = cached.BfcBlockOffset();
        let mut block_offset_delta = LayoutUnit::default();
        let mut end_margin_strut = cached.EndMarginStrut();
        let is_fragmented = IsBreakInside(break_token)
            || !physical_fragment.GetBreakToken().is_null()
            || self.PhysicalFragmentCount() > 1;
        let old_space = cached.GetConstraintSpaceForCaching();
        let are_bfc_offsets_equal = new_space.GetBfcOffset() == old_space.GetBfcOffset()
            && new_space.ExpectedBfcBlockOffset() == old_space.ExpectedBfcBlockOffset()
            && new_space.ForcedBfcBlockOffset() == old_space.ForcedBfcBlockOffset();
        let is_margin_strut_equal = new_space.GetMarginStrut() == old_space.GetMarginStrut();
        let is_exclusion_space_equal =
            new_space.GetExclusionSpace() == old_space.GetExclusionSpace();
        let is_clearance_offset_equal = new_space.ClearanceOffset() == old_space.ClearanceOffset();
        let is_new_formatting_context = physical_fragment.IsFormattingContextRoot();

        if !is_new_formatting_context
            && (!are_bfc_offsets_equal
                || !is_exclusion_space_equal
                || !is_margin_strut_equal
                || !is_clearance_offset_equal)
        {
            debug_assert!(!unsafe { LayoutBoxCreatesNewFormattingContextProvider(self) });
            if cache_status == LayoutCacheStatus::kNeedsSimplifiedLayout
                || cache_status == LayoutCacheStatus::kCanReuseLines
            {
                return std::ptr::null();
            }
            debug_assert_eq!(cache_status, LayoutCacheStatus::kHit);
            if !MaySkipLayoutWithinBlockFormattingContext(
                cached,
                new_space,
                &mut bfc_block_offset,
                &mut block_offset_delta,
                &mut end_margin_strut,
            ) {
                return std::ptr::null();
            }
        }

        if new_space.HasBlockFragmentation() {
            debug_assert!(old_space.HasBlockFragmentation());
            if cache_status != LayoutCacheStatus::kHit
                || physical_fragment.HasNestedMulticolsWithOOFs()
                || physical_fragment.HasOutOfFlowFragmentChild()
                || !column_spanner_path.is_null()
                || !cached.GetColumnSpannerPath().is_null()
            {
                return std::ptr::null();
            }
            if new_space.FragmentainerBlockSize() != old_space.FragmentainerBlockSize()
                || new_space.FragmentainerOffset() != old_space.FragmentainerOffset()
            {
                if is_fragmented
                    || cached.MinimalSpaceShortage().is_some()
                    || physical_fragment.IsFragmentationContextRoot()
                    || cached.IsBlockSizeForFragmentationClamped()
                    || cached.IsTruncatedByFragmentationLine()
                {
                    return std::ptr::null();
                }

                let do_floats_cross_fragmentation_line = || {
                    let result_exclusion_space = cached.GetExclusionSpace();
                    if result_exclusion_space != old_space.GetExclusionSpace() {
                        let block_end_offset = FragmentainerOffsetAtBfc(new_space)
                            + result_exclusion_space.ClearanceOffset(EClear::kBoth);
                        if block_end_offset > new_space.FragmentainerBlockSize() {
                            return true;
                        }
                    }
                    false
                };

                if bfc_block_offset.is_none() && cached.IsSelfCollapsing() {
                    if old_space.IsInitialColumnBalancingPass()
                        || do_floats_cross_fragmentation_line()
                    {
                        return std::ptr::null();
                    }
                } else {
                    if physical_fragment.IsInlineFormattingContext()
                        && !is_new_formatting_context
                        && do_floats_cross_fragmentation_line()
                    {
                        return std::ptr::null();
                    }
                    let block_size_for_fragmentation = cached.BlockSizeForFragmentation();
                    let block_end_offset = FragmentainerOffsetAtBfc(new_space)
                        + bfc_block_offset.unwrap_or_default()
                        + block_size_for_fragmentation;
                    if block_end_offset > new_space.FragmentainerBlockSize() {
                        return std::ptr::null();
                    }
                }
                if old_space.IsInitialColumnBalancingPass() {
                    if physical_fragment.HasOutOfFlowInFragmentainerSubtree() {
                        return std::ptr::null();
                    }
                    let block = DynamicTo::<super::layout_block::LayoutBlock>(
                        self as *mut LayoutBox as *mut LayoutObject,
                    );
                    if !block.is_null() && unsafe { &*block }.IsFragmentationContextRoot() {
                        return std::ptr::null();
                    }
                }
            }
        }

        if is_fragmented {
            if cached.GetExclusionSpace().HasFragmentainerBreak()
                || cache_status == LayoutCacheStatus::kNeedsSimplifiedLayout
            {
                return std::ptr::null();
            }
        }
        *out_cache_status = cache_status;
        if cache_status == LayoutCacheStatus::kNeedsSimplifiedLayout
            || cache_status == LayoutCacheStatus::kCanReuseLines
        {
            return cached_layout_result;
        }
        physical_fragment.CheckType();
        debug_assert_eq!(*out_cache_status, LayoutCacheStatus::kHit);
        if use_layout_cache_slot
            && !is_blocked_by_display_lock
            && self.NeedsScrollableOverflowRecalc()
        {
            #[cfg(debug_assertions)]
            let cloned_cached_layout_result = LayoutResult::CloneWithPostLayoutFragments(cached);
            if !DisableLayoutSideEffectsScope::IsDisabled() {
                self.RecalcScrollableOverflow();
            }
            cached_layout_result = self.GetCachedLayoutResult(break_token);
            #[cfg(debug_assertions)]
            unsafe { &*cloned_cached_layout_result }
                .CheckSameForSimplifiedLayout(unsafe { &*cached_layout_result }, false);
        }

        let cached = unsafe { &*cached_layout_result };
        if self.IsTableRow() {
            // Match the source's const_cast through LayoutResult's
            // explicitly interior-mutable cached space.
            cached.ReplaceTableRowDataForCaching(
                new_space.CloneTableDataHandle(),
                new_space.TableRowIndex(),
            );
        }
        if are_bfc_offsets_equal && is_exclusion_space_equal && is_margin_strut_equal {
            cached
                .GetExclusionSpace()
                .MoveAndUpdateDerivedGeometry(new_space.GetExclusionSpace());
            return cached_layout_result;
        }
        MakeGarbageCollected(LayoutResult::copy_with_new_space(
            cached,
            new_space,
            &end_margin_strut,
            bfc_line_offset,
            bfc_block_offset,
            block_offset_delta,
        ))
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:58-59
    pub fn new(node: *mut ContainerNode) -> Self {
        let box_ = Self {
            model_object_: LayoutBoxModelObject::new(node),
            scrollbar_freeze_state_: ScrollbarFreezeState::default(),
            static_position_for_layout_: LogicalStaticPosition::from_offset(
                LogicalOffset::default(),
            ),
            frame_location_: PhysicalOffset::default(),
            frame_size_: Cell::new(PhysicalSize::default()),
            previous_size_: PhysicalSize::default(),
            intrinsic_logical_widths_: MinMaxSizes::default(),
            min_max_sizes_cache_: Member::default(),
            measure_cache_: Member::default(),
            layout_results_: LayoutResultList::default(),
            first_fragment_item_index_: 0,
            overflow_: Member::default(),
            rare_data_: Member::default(),
        };
        box_.SetRuntimeClass(super::layout_object::LayoutObjectClass::Box);
        box_
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:61-67
    pub fn AdjustOutOfFlowContainingBlockForAnchor(
        &self,
        grid_layout_data: *const GridLayoutData,
        style: &ComputedStyle,
        padding_box_rect: &LogicalRect,
        box_: &LayoutBox,
    ) -> LogicalRect {
        unsafe {
            LayoutBoxAdjustOutOfFlowContainingBlockForAnchorProvider(
                self,
                grid_layout_data,
                style,
                padding_box_rect,
                box_,
            )
        }
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:69-88
    pub fn ScrollWidth(&self) -> LayoutUnit {
        unsafe { LayoutBoxScrollWidthProvider(self) }
    }

    pub fn ScrollWidthBase(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        if self.IsScrollContainer() {
            return unsafe { &*self.GetScrollableArea() }.ScrollWidth();
        }
        if self.StyleRef().IsScrollbarGutterStable()
            && self.StyleRef().OverflowBlockDirection() == EOverflow::kHidden
        {
            let scrollable_area = self.GetScrollableArea();
            if !scrollable_area.is_null() {
                return unsafe { &*scrollable_area }.ScrollWidth();
            }
            return self.ScrollableOverflowRect().Width();
        }
        let overflow_rect = self.ScrollableOverflowRect();
        if !self.StyleRef().GetWritingDirection().IsFlippedX() {
            return std::cmp::max(
                self.PhysicalPaddingBoxRect().Width(),
                overflow_rect.Right() - self.BorderOutsets().left,
            );
        }
        self.PhysicalPaddingBoxRect().Width()
            - std::cmp::min(
                LayoutUnit::default(),
                overflow_rect.X() - self.BorderOutsets().left,
            )
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:90-105
    pub fn ScrollHeight(&self) -> LayoutUnit {
        unsafe { LayoutBoxScrollHeightProvider(self) }
    }

    pub fn ScrollHeightBase(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        if self.IsScrollContainer() {
            return unsafe { &*self.GetScrollableArea() }.ScrollHeight();
        }
        if self.StyleRef().IsScrollbarGutterStable()
            && self.StyleRef().OverflowBlockDirection() == EOverflow::kHidden
        {
            let scrollable_area = self.GetScrollableArea();
            if !scrollable_area.is_null() {
                return unsafe { &*scrollable_area }.ScrollHeight();
            }
            return self.ScrollableOverflowRect().Height();
        }
        std::cmp::max(
            self.PhysicalPaddingBoxRect().Height(),
            self.ScrollableOverflowRect().Bottom() - self.BorderOutsets().top,
        )
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:107-116
    pub fn MarginOutsets(&self) -> PhysicalBoxStrut {
        self.CheckIsNotDestroyed();
        if self.PhysicalFragmentCount() != 0 {
            return unsafe { &*self.GetPhysicalFragment(0) }.Margins();
        }
        PhysicalBoxStrut::default()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:118-122
    pub fn LogicalRectInContainer(&self) -> LogicalRect {
        self.CheckIsNotDestroyed();
        unsafe { &*self.LocationContainer() }
            .CreateWritingModeConverter()
            .ToLogicalRect(PhysicalRect::new(
                self.PhysicalLocation(),
                self.StitchedSize(),
            ))
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:565-569
    pub fn CreateWritingModeConverter(&self) -> WritingModeConverter {
        self.CheckIsNotDestroyed();
        WritingModeConverter::new(
            WritingDirectionMode::new(self.StyleRef().GetWritingMode(), TextDirection::kLtr),
            self.StitchedSize(),
        )
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:576-601
    pub fn BoundingBoxRelativeToFirstFragment(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let mut bounding_rect = PhysicalRect::default();
        let mut first_fragment: *const PhysicalBoxFragment = std::ptr::null();
        for fragment in self.PhysicalFragments() {
            let offset = if first_fragment.is_null() {
                first_fragment = fragment;
                PhysicalOffset::default()
            } else {
                fragment.OffsetFromRootFragmentationContext()
                    - unsafe { &*first_fragment }.OffsetFromRootFragmentationContext()
            };
            let fragment_rect = PhysicalRect::new(offset, fragment.Size());
            bounding_rect.UniteEvenIfEmpty(&fragment_rect);
            let break_token = fragment.GetBreakToken();
            if !break_token.is_null() && unsafe { &*break_token }.IsAtBlockEnd() {
                break;
            }
        }
        bounding_rect
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:647-650
    pub fn DebugRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        PhysicalRect::new(self.PhysicalLocation(), self.StitchedSize())
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:772-779
    pub fn TransformsChangeMayRequireLayout(&self) -> bool {
        for fragment in self.PhysicalFragments() {
            if fragment.HasAnchorsToPropagate() {
                return true;
            }
        }
        false
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:781-784
    pub fn GetShapeOutsideInfo(&self) -> *mut ShapeOutsideInfo {
        self.CheckIsNotDestroyed();
        ShapeOutsideInfo::Info(self)
    }

    // cpp: layoutng/internal/layout_box.h:789-793
    pub fn IsWritingModeRoot(&self) -> bool {
        self.CheckIsNotDestroyed();
        let parent = self.Parent();
        parent.is_null()
            || unsafe { &*parent }.StyleRef().GetWritingMode() != self.StyleRef().GetWritingMode()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:480-489
    pub fn IsSemiReplaced(&self) -> bool {
        let element = DynamicTo::<Element>(self.GetNode());
        if !element.is_null() {
            return !DynamicTo::<HTMLImageElement>(self.GetNode()).is_null()
                || (unsafe { &*element }.IsFormControlElement()
                    && DynamicTo::<HTMLFieldSetElement>(self.GetNode()).is_null());
        }
        false
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:509-524
    pub fn IsMonolithic(&self) -> bool {
        self.CheckIsNotDestroyed();
        if self.IsInline()
            || self.IsSemiReplaced()
            || self.HasUnsplittableScrollingOverflow()
            || self.IsOverscrollContainer()
            || (!self.Parent().is_null() && self.IsWritingModeRoot())
            || (self.IsFixedPositioned()
                && self.IsPrintingForLayout()
                && !DynamicTo::<LayoutView>(self.Container()).is_null())
            || self.ShouldApplySizeContainment()
            || self.IsFrameSet()
            || self.StyleRef().HasLineClamp()
            || self.IsScrollMarkerGroup()
        {
            return true;
        }
        false
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:404-414
    pub fn IsValidColumnSpannerInTree(&self, style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        if self.Parent().is_null()
            || !self.IsInsideMulticol()
            || !self.IsSelfValidColumnSpanner(style)
        {
            return false;
        }
        self.DoesAncestryAllowColumnSpanner(style)
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:416-433
    pub fn IsSelfValidColumnSpanner(&self, style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        if style.GetColumnSpan() != EColumnSpan::kAll {
            return false;
        }
        if self.ShouldBeHandledAsInline(style)
            || self.ShouldBeHandledAsFloating(style)
            || self.ToPositionedState(style.GetPosition())
                == PositionedState::kIsOutOfFlowPositioned
        {
            return false;
        }
        true
    }

    // cpp: layoutng/internal/layout_box.h:638-641
    pub fn IsSelfValidColumnSpannerCurrentStyle(&self) -> bool {
        self.IsSelfValidColumnSpanner(self.StyleRef())
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:435-449
    pub fn DoesAncestryAllowColumnSpanner(&self, _style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        debug_assert!(self.IsInsideMulticol());
        let parent = self.Parent();
        let mut ancestor = if parent.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*parent }.EnclosingBox()
        };
        while !ancestor.is_null() {
            let ancestor_ref = unsafe { &*ancestor };
            if ancestor_ref.IsMulticolContainer() {
                return true;
            }
            if ancestor_ref.ShouldPreventColumnSpannerDescendants() {
                return false;
            }
            ancestor = ancestor_ref.ContainingBlock() as *mut LayoutBox;
        }
        false
    }

    // cpp: layoutng/internal/layout_box.h:650-653
    pub fn DoesAncestryAllowColumnSpannerCurrentStyle(&self) -> bool {
        self.DoesAncestryAllowColumnSpanner(self.StyleRef())
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:451-478
    pub fn ShouldPreventColumnSpannerDescendants(&self) -> bool {
        self.CheckIsNotDestroyed();
        if self.IsSelfValidColumnSpannerCurrentStyle() {
            return true;
        }
        let block_flow =
            DynamicTo::<LayoutBlockFlow>(self as *const LayoutBox as *const LayoutObject);
        if block_flow.is_null() {
            return true;
        }
        let block_flow = unsafe { &*block_flow };
        if block_flow.IsMonolithic()
            || block_flow.CreatesNewFormattingContext()
            || block_flow.CanContainFixedPositionObjects()
        {
            return true;
        }
        debug_assert_ne!(self.StyleRef().GetColumnSpan(), EColumnSpan::kAll);
        false
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:348-374
    pub fn ContainingBlockLogicalHeightForRelPositioned(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        debug_assert!(self.IsRelPositioned());
        let container = self.Container();
        let box_ = DynamicTo::<LayoutBox>(container);
        if !box_.is_null() {
            let box_ = unsafe { &*box_ };
            let size = box_.PhysicalContentBoxRect().size;
            return if box_.StyleRef().IsHorizontalWritingMode() {
                size.height
            } else {
                size.width
            };
        }
        let layout_inline = DynamicTo::<LayoutInline>(container);
        debug_assert!(!layout_inline.is_null());
        let layout_inline = unsafe { &*layout_inline };
        if !layout_inline.HasInlineFragments() {
            return LayoutUnit::default();
        }
        let block_size = ToLogicalSize(
            layout_inline.PhysicalLinesBoundingBox().size,
            layout_inline.StyleRef().GetWritingMode(),
        )
        .block_size;
        (block_size - layout_inline.BorderPaddingBlockSize()).ClampNegativeToZero()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:376-387
    pub fn ContainingBlockLogicalWidthForContent(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        if self.HasOverrideContainingBlockContentLogicalWidth() {
            return self.OverrideContainingBlockContentLogicalWidth();
        }
        let cb = unsafe { &*self.ContainingBlock() };
        if self.IsOutOfFlowPositioned() {
            let size = cb.PhysicalPaddingBoxRect().size;
            return if cb.StyleRef().IsHorizontalWritingMode() {
                size.width
            } else {
                size.height
            };
        }
        cb.ContentLogicalWidth()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:389-402
    pub fn OffsetFromContainerInternal(
        &self,
        container: *const LayoutObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        debug_assert_eq!(container, self.Container() as *const LayoutObject);
        let mut offset = self.PhysicalLocation();
        if self.NeedsAnchorPositionScrollAdjustment() {
            offset += self.AnchorPositionScrollTranslationOffset();
        }
        offset
            + self
                .model_object_
                .OffsetFromContainerInternal(container, mode)
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:168-179
    pub fn ClippingRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let infinite = InfiniteIntRect();
        let mut result = PhysicalRect::new(
            PhysicalOffset::new(
                LayoutUnit::from_signed(infinite.x()),
                LayoutUnit::from_signed(infinite.y()),
            ),
            PhysicalSize::new(
                LayoutUnit::from_signed(infinite.width()),
                LayoutUnit::from_signed(infinite.height()),
            ),
        );
        if self.ShouldClipOverflowAlongEitherAxis() {
            result = self.OverflowClipRect();
        }
        if self.HasCSSClip() {
            result.Intersect(&self.CSSClipRect());
        }
        result
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:190-216
    pub fn ApplyBoxClips(
        &self,
        transform_state: &mut TransformState,
        _accumulation: TransformAccumulation,
        visual_rect_flags: VisualRectFlags,
    ) -> bool {
        self.CheckIsNotDestroyed();
        if visual_rect_flags.Has(VisualRectFlag::kSkipAncestorAndViewportClips) {
            return true;
        }
        transform_state.Flatten();
        let mut rect = PhysicalRect::EnclosingRect(&transform_state.LastPlanarQuad().BoundingBox());
        let clip_rect = self.ClippingRect();
        let does_intersect = if visual_rect_flags.Has(VisualRectFlag::kEdgeInclusive) {
            rect.InclusiveIntersect(&clip_rect)
        } else {
            rect.Intersect(&clip_rect);
            !rect.IsEmpty()
        };
        transform_state.SetQuad(gfx::QuadF::from(gfx::RectF::from(rect)));
        does_intersect
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:124-128
    pub fn AbsoluteContentQuad(&self, flags: MapCoordinatesFlags) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        let rect = self.PhysicalContentBoxRect();
        self.LocalRectToAbsoluteQuad(&rect, flags)
    }

    // cpp: layoutng/internal/layout_box.h:752-756
    pub fn OverflowClipRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.OverflowClipRectWithBehavior(OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize)
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:239-292
    pub fn OverflowClipRectWithBehavior(
        &self,
        behavior: OverlayScrollbarClipBehavior,
    ) -> PhysicalRect {
        unsafe { LayoutBoxOverflowClipRectProvider(self, behavior) }
    }
    pub fn OverflowClipRectWithBehaviorBase(
        &self,
        overlay_scrollbar_clip_behavior: OverlayScrollbarClipBehavior,
    ) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let mut clip_rect = if self.IsEffectiveRootScroller() {
            PhysicalRect::new(PhysicalOffset::default(), unsafe {
                LayoutBoxEffectiveRootScrollerViewportSizeProvider(self)
            })
        } else {
            let mut rect = self.PhysicalBorderBoxRect();
            rect.Contract(&self.BorderOutsets());
            if self.IsVideo() || self.IsLayoutEmbeddedContent() {
                rect = LayoutReplaced::PreSnappedRectForPersistentSizing(&rect);
            }
            if self.HasNonVisibleOverflow() {
                let overflow_clip = self.GetOverflowClipAxes();
                if overflow_clip != kOverflowClipBothAxis {
                    super::layout_box_core_services::ApplyVisibleOverflowToClipRect(
                        overflow_clip,
                        &mut rect,
                    );
                } else if self.ShouldApplyOverflowClipMargin() {
                    let margin = self
                        .StyleRef()
                        .OverflowClipMargin()
                        .as_ref()
                        .expect("overflow clip margin exists");
                    match margin.GetReferenceBox() {
                        ReferenceBox::kBorderBox => rect.Expand(&self.BorderOutsets()),
                        ReferenceBox::kPaddingBox => {}
                        ReferenceBox::kContentBox => rect.Contract(&self.PaddingOutsets()),
                    }
                    rect.Expand(&PhysicalBoxStrut::with_value(margin.GetMargin()));
                }
            }
            rect
        };
        if self.IsScrollContainer() {
            self.ExcludeScrollbars(
                &mut clip_rect,
                overlay_scrollbar_clip_behavior,
                ShouldIncludeScrollbarGutter::kExcludeScrollbarGutter,
            );
        }
        clip_rect
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:294-296
    pub fn OverflowClipRectForScrollNode(&self) -> PhysicalRect {
        self.OverflowClipRect()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:625-630
    pub fn LocalBoundingBoxRectForAccessibility(
        &self,
        _include_descendants: IncludeDescendants,
    ) -> gfx::RectF {
        self.CheckIsNotDestroyed();
        let size = self.StitchedSize();
        gfx::RectF::new(
            gfx::PointF::new(0.0, 0.0),
            gfx::SizeF::new(size.width.ToFloat(), size.height.ToFloat()),
        )
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:632-640
    pub fn AddOutlineRects(
        &self,
        collector: &mut dyn OutlineRectCollector,
        info: *mut LayoutOutlineInfo,
        additional_offset: &PhysicalOffset,
        _outline_type: OutlineType,
    ) {
        self.CheckIsNotDestroyed();
        collector.AddRect(&PhysicalRect::new(*additional_offset, self.StitchedSize()));
        if !info.is_null() {
            unsafe { *info = LayoutOutlineInfo::GetFromStyle(self.StyleRef()) };
        }
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:642-645
    pub fn OffsetPoint(&self, parent: *const Element) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        self.AdjustedPositionRelativeTo(&self.PhysicalLocation(), parent)
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:603-623
    pub fn QuadsInAncestorInternal(
        &self,
        quads: &mut Vector<gfx::QuadF>,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
        box_type: BoxQuadType,
    ) {
        self.CheckIsNotDestroyed();
        let mut first_fragment: Option<&PhysicalBoxFragment> = None;
        for fragment in self.PhysicalFragments() {
            let offset = if let Some(first) = first_fragment {
                fragment.OffsetFromRootFragmentationContext()
                    - first.OffsetFromRootFragmentationContext()
            } else {
                first_fragment = Some(fragment);
                PhysicalOffset::default()
            };
            let mut rect = LocalRectForBoxQuad(fragment, box_type);
            rect.offset += offset;
            quads.push(self.LocalRectToAncestorQuad(&rect, ancestor, mode));
        }
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:540-549
    pub fn AddCustomLayoutChildIfNeeded(&mut self) {
        self.CheckIsNotDestroyed();
        if !self.IsCustomItem() {
            return;
        }
        let factories = LayoutObjectFactoryScope::Objects();
        assert!(
            !factories.is_null(),
            "custom layout factory set is required"
        );
        let create = unsafe { &*factories }
            .create_custom_child
            .expect("custom layout child factory is required");
        create(self);
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:551-563
    pub fn ClearCustomLayoutChild(&mut self) {
        self.CheckIsNotDestroyed();
        let rare_data = self.rare_data_.Get();
        if rare_data.is_null() {
            return;
        }
        let layout_child = unsafe { &*rare_data }.layout_child_.Get();
        if !layout_child.is_null() {
            let clear = unsafe { &*rare_data }
                .clear_layout_child_
                .expect("custom layout child clearer is required");
            clear(layout_child);
        }
        unsafe { &mut *rare_data }.layout_child_ = Member::default();
        unsafe { &mut *rare_data }.clear_layout_child_ = None;
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:665-674
    pub fn ResolvedDirection(&self) -> TextDirection {
        self.CheckIsNotDestroyed();
        if self.IsInLayoutNGInlineFormattingContext() && self.IsInline() {
            let mut cursor = InlineCursor::default();
            cursor.MoveToLayoutObject(self);
            if !cursor.IsNull() {
                return cursor.Current().ResolvedDirection();
            }
        }
        self.StyleRef().Direction()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:652-662
    pub fn VisualRectOutsetForRasterEffects(&self) -> RasterEffectOutset {
        self.CheckIsNotDestroyed();
        if self.VisualOverflowIsSet()
            && unsafe { &*self.overflow_.Get() }
                .visual_overflow
                .as_ref()
                .expect("visual overflow set")
                .HasSubpixelVisualEffectOutsets()
        {
            RasterEffectOutset::kWholePixel
        } else {
            RasterEffectOutset::kNone
        }
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:298-314
    pub fn ExcludeScrollbars(
        &self,
        rect: &mut PhysicalRect,
        overlay_scrollbar_clip_behavior: OverlayScrollbarClipBehavior,
        include_scrollbar_gutter: ShouldIncludeScrollbarGutter,
    ) {
        self.CheckIsNotDestroyed();
        if self.CanSkipComputeScrollbars() {
            return;
        }
        let scrollbars = self.ComputeScrollbarsInternal(
            ShouldClampToContentBox::kDoNotClampToContentBox,
            overlay_scrollbar_clip_behavior,
            include_scrollbar_gutter,
        );
        rect.offset.top += scrollbars.top;
        rect.offset.left += scrollbars.left;
        rect.size.width -= scrollbars.HorizontalSum();
        rect.size.height -= scrollbars.VerticalSum();
        rect.size.ClampNegativeToZero();
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:316-346
    pub fn CSSClipRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let stitched_size = self.StitchedSize();
        let mut clip_rect = PhysicalRect::new(PhysicalOffset::default(), stitched_size);
        let width = stitched_size.width;
        let height = stitched_size.height;
        if !self.StyleRef().ClipLeft().IsAuto() {
            let c = ValueForLength(self.StyleRef().ClipLeft(), width);
            clip_rect.offset.left += c;
            clip_rect.size.width -= c;
        }
        if !self.StyleRef().ClipRight().IsAuto() {
            clip_rect.size.width -= width - ValueForLength(self.StyleRef().ClipRight(), width);
        }
        if !self.StyleRef().ClipTop().IsAuto() {
            let c = ValueForLength(self.StyleRef().ClipTop(), height);
            clip_rect.offset.top += c;
            clip_rect.size.height -= c;
        }
        if !self.StyleRef().ClipBottom().IsAuto() {
            clip_rect.size.height -= height - ValueForLength(self.StyleRef().ClipBottom(), height);
        }
        clip_rect
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:914-932
    pub fn IsReadingFlowContainer(&self) -> bool {
        self.CheckIsNotDestroyed();
        match self.StyleRef().ReadingFlow() {
            EReadingFlow::kNormal => false,
            EReadingFlow::kFlexVisual | EReadingFlow::kFlexFlow => unsafe {
                LayoutBoxIsFlexibleBoxProvider(self)
            },
            EReadingFlow::kGridRows | EReadingFlow::kGridColumns | EReadingFlow::kGridOrder => unsafe {
                LayoutBoxIsLayoutGridProvider(self)
            },
            EReadingFlow::kSourceOrder => {
                self.IsLayoutBlock()
                    || unsafe { LayoutBoxIsFlexibleBoxProvider(self) }
                    || self.IsLayoutGridOrGridLanes()
            }
            _ => false,
        }
    }

    // Cell preserves the source's const cache fill without manufacturing an
    // exclusive reference from &self. It has the same value layout as the
    // cached PhysicalSize field in the base-first LayoutBox record.
    // cpp: layoutng/internal/layout_box.h:279-279
    // cpp: layoutng/internal/layout_box_fragment_data.cc:523-531
    pub fn StitchedSize(&self) -> PhysicalSize {
        unsafe { LayoutBoxStitchedSizeProvider(self) }
    }
    pub fn StitchedSizeBase(&self) -> PhysicalSize {
        self.CheckIsNotDestroyed();
        if !self.HasValidCachedGeometry() {
            self.SetHasValidCachedGeometry(true);
            self.frame_size_.set(self.ComputeSize());
        }
        self.frame_size_.get()
    }

    // cpp: layoutng/internal/layout_box.h:263-263
    // cpp: layoutng/internal/layout_box_fragment_data.cc:482-521
    pub fn StitchedBlockSize(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        if self.PhysicalFragmentCount() == 0 {
            return LayoutUnit::default();
        }
        let writing_direction = self.StyleRef().GetWritingDirection();
        let first_fragment = unsafe { &*self.GetPhysicalFragment(0) };
        let first_break_token = first_fragment.GetBreakToken();
        if first_fragment.IsOnlyForNode()
            || (!first_break_token.is_null() && unsafe { &*first_break_token }.IsRepeated())
        {
            return LogicalFragment::new(writing_direction, first_fragment).BlockSize();
        }
        let mut idx = self.PhysicalFragmentCount() - 1;
        debug_assert!(idx >= 1);
        let mut last_content_fragment_size = unsafe { &*self.GetPhysicalFragment(idx) }.Size();
        let mut previously_consumed_block_size = LayoutUnit::default();
        while idx != 0 {
            idx -= 1;
            let break_token = unsafe { &*self.GetPhysicalFragment(idx) }.GetBreakToken();
            if !unsafe { &*break_token }.IsAtBlockEnd() {
                previously_consumed_block_size = unsafe { &*break_token }.ConsumedBlockSize();
                break;
            }
            last_content_fragment_size = unsafe { &*self.GetPhysicalFragment(idx) }.Size();
        }
        let logical_size =
            ToLogicalSize(last_content_fragment_size, self.StyleRef().GetWritingMode());
        previously_consumed_block_size + logical_size.block_size
    }

    // cpp: layoutng/internal/layout_box.h:250-254
    pub fn LogicalWidth(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        let size = self.StitchedSize();
        if self.StyleRef().IsHorizontalWritingMode() {
            size.width
        } else {
            size.height
        }
    }

    // cpp: layoutng/internal/layout_box.h:257-261
    pub fn LogicalHeight(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        let size = self.StitchedSize();
        if self.StyleRef().IsHorizontalWritingMode() {
            size.height
        } else {
            size.width
        }
    }

    // cpp: layoutng/internal/layout_box.h:291-296
    pub fn PhysicalBorderBoxRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        PhysicalRect::new(PhysicalOffset::default(), self.StitchedSize())
    }

    // cpp: layoutng/internal/layout_box.h:305-309
    pub fn PhysicalPaddingBoxRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.PhysicalContractedBoxRect(ContractionEdge::kContractToPaddingEdge)
    }

    // cpp: layoutng/internal/layout_box.h:310-314
    pub fn PhysicalContentBoxRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.PhysicalContractedBoxRect(ContractionEdge::kContractToContentEdge)
    }

    // cpp: layoutng/internal/layout_box.h:316-320
    pub fn ContentLogicalWidth(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        let size = self.PhysicalContentBoxRect().size;
        if self.IsHorizontalWritingMode() {
            size.width
        } else {
            size.height
        }
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:593-640
    pub fn PhysicalContractedBoxRect(&self, edge: ContractionEdge) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let mut rect = PhysicalRect::new(PhysicalOffset::default(), self.StitchedSize());
        let mut inset = PhysicalBoxStrut::default();
        if self.PhysicalFragmentCount() == 1 {
            let fragment = unsafe { &*self.GetPhysicalFragment(0) };
            if fragment.HasBorders() {
                inset += fragment.Borders();
            }
            if fragment.HasScrollbar() {
                inset += fragment.Scrollbar();
            }
            if edge == ContractionEdge::kContractToContentEdge && fragment.HasPadding() {
                inset += fragment.Padding();
            }
        } else if self.PhysicalFragmentCount() != 0 {
            let top_fragment = unsafe { &*FragmentForTopEdge(self) };
            let right_fragment = unsafe { &*FragmentForRightEdge(self) };
            let bottom_fragment = unsafe { &*FragmentForBottomEdge(self) };
            let left_fragment = unsafe { &*FragmentForLeftEdge(self) };
            inset.top += top_fragment.Borders().top + top_fragment.Scrollbar().top;
            inset.right += right_fragment.Borders().right + right_fragment.Scrollbar().right;
            inset.bottom += bottom_fragment.Borders().bottom + bottom_fragment.Scrollbar().bottom;
            inset.left += left_fragment.Borders().left + left_fragment.Scrollbar().left;
            if edge == ContractionEdge::kContractToContentEdge {
                inset.top += top_fragment.Padding().top;
                inset.right += right_fragment.Padding().right;
                inset.bottom += bottom_fragment.Padding().bottom;
                inset.left += left_fragment.Padding().left;
            }
        }
        rect.Contract(&inset);
        rect.size.width = rect.size.width.ClampNegativeToZero();
        rect.size.height = rect.size.height.ClampNegativeToZero();
        rect
    }

    // cpp: layoutng/internal/layout_box.h:1314-1315
    // cpp: layoutng/internal/layout_box_fragment_data.cc:533-576
    pub fn ComputeSize(&self) -> PhysicalSize {
        self.CheckIsNotDestroyed();
        let results = self.GetLayoutResults();
        let Some(first_result) = results.iter().next() else {
            return PhysicalSize::default();
        };
        let first_fragment = unsafe { &*first_result.Get() }.GetPhysicalFragment();
        if results.len() == 1 {
            return first_fragment.Size();
        }
        let converter =
            WritingModeConverter::without_outer_size(first_fragment.Style().GetWritingDirection());
        let mut previous_break_token: *const BlockBreakToken = std::ptr::null();
        let mut size = LogicalSize::default();
        for result in results {
            let fragment = unsafe { &*result.Get() }.GetPhysicalFragment();
            debug_assert!(fragment.IsBox());
            let physical_fragment =
                unsafe { &*(fragment as *const _ as *const PhysicalBoxFragment) };
            let fragment_logical_size = converter.ToLogicalSize(physical_fragment.Size());
            if physical_fragment.IsFirstForNode() {
                size = fragment_logical_size;
            } else {
                debug_assert!(!previous_break_token.is_null());
                size.block_size = fragment_logical_size.block_size
                    + unsafe { &*previous_break_token }.ConsumedBlockSize();
            }
            previous_break_token = physical_fragment.GetBreakToken();
            if previous_break_token.is_null()
                || unsafe { &*previous_break_token }.IsRepeated()
                || unsafe { &*previous_break_token }.IsAtBlockEnd()
            {
                break;
            }
        }
        converter.ToPhysicalSize(size)
    }

    // cpp: layoutng/internal/layout_box.h:670-676
    pub fn ComputeScrollbars(&self) -> PhysicalBoxStrut {
        self.CheckIsNotDestroyed();
        if self.CanSkipComputeScrollbars() {
            PhysicalBoxStrut::default()
        } else {
            self.ComputeScrollbarsInternal(
                ShouldClampToContentBox::kDoNotClampToContentBox,
                OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
                ShouldIncludeScrollbarGutter::kIncludeScrollbarGutter,
            )
        }
    }

    // cpp: layoutng/internal/layout_box.h:677-684
    pub fn ComputeLogicalScrollbars(&self) -> BoxStrut {
        self.CheckIsNotDestroyed();
        if self.CanSkipComputeScrollbars() {
            BoxStrut::default()
        } else {
            self.ComputeScrollbarsInternal(
                ShouldClampToContentBox::kDoNotClampToContentBox,
                OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
                ShouldIncludeScrollbarGutter::kIncludeScrollbarGutter,
            )
            .ConvertToLogical(self.StyleRef().GetWritingDirection())
        }
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:138-145
    pub fn OriginAdjustmentForScrollbars(&self) -> gfx::Vector2d {
        self.CheckIsNotDestroyed();
        if self.CanSkipComputeScrollbars() {
            return gfx::Vector2d::default();
        }
        let scrollbars = self.ComputeScrollbarsInternal(
            ShouldClampToContentBox::kClampToContentBox,
            OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
            ShouldIncludeScrollbarGutter::kIncludeScrollbarGutter,
        );
        gfx::Vector2d::new(scrollbars.left.ToInt(), scrollbars.top.ToInt())
    }

    // cpp: layoutng/internal/layout_box.h:1255-1258
    pub fn ScrollableOverflowIsSet(&self) -> bool {
        self.CheckIsNotDestroyed();
        let overflow = self.overflow_.Get();
        !overflow.is_null() && unsafe { &*overflow }.scrollable_overflow.is_some()
    }

    // cpp: layoutng/internal/layout_box.h:336-342
    pub fn ScrollableOverflowRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        if self.ScrollableOverflowIsSet() {
            *unsafe { &*self.overflow_.Get() }
                .scrollable_overflow
                .as_ref()
                .expect("scrollable overflow is set")
                .ScrollableOverflowRect()
        } else {
            self.PhysicalPaddingBoxRect()
        }
    }

    // cpp: layoutng/internal/layout_box.h:697-704
    pub fn HasAutoVerticalScrollbar(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.HasNonVisibleOverflow() && self.StyleRef().HasAutoVerticalScroll()
    }

    pub fn HasAutoHorizontalScrollbar(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.HasNonVisibleOverflow() && self.StyleRef().HasAutoHorizontalScroll()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:130-136
    pub fn CanResize(&self) -> bool {
        self.CheckIsNotDestroyed();
        (self.IsScrollContainer() || self.IsLayoutIFrame()) && self.StyleRef().HasResize()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:147-151
    pub fn ScrollOrigin(&self) -> gfx::Point {
        self.CheckIsNotDestroyed();
        let scrollable_area = self.GetScrollableArea();
        if scrollable_area.is_null() {
            gfx::Point::default()
        } else {
            unsafe { &*scrollable_area }.ScrollOrigin()
        }
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:153-159
    pub fn ScrolledContentOffset(&self) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        debug_assert!(self.IsScrollContainer());
        let scrollable_area = self.GetScrollableArea();
        debug_assert!(!scrollable_area.is_null());
        PhysicalOffset::FromVector2dFFloor(&unsafe { &*scrollable_area }.GetScrollOffset())
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:161-166
    pub fn PixelSnappedScrolledContentOffset(&self) -> gfx::Vector2d {
        self.CheckIsNotDestroyed();
        debug_assert!(self.IsScrollContainer());
        let scrollable_area = self.GetScrollableArea();
        debug_assert!(!scrollable_area.is_null());
        unsafe { &*scrollable_area }.ScrollOffsetInt()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:181-188
    pub fn ContainedContentsScroll(&self, contents: &LayoutObject) -> bool {
        self.CheckIsNotDestroyed();
        if self.IsLayoutView() && contents.StyleRef().GetPosition() == EPosition::kFixed {
            return false;
        }
        self.IsScrollContainer()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:218-222
    pub fn OverrideContainingBlockContentLogicalWidth(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        debug_assert!(self.HasOverrideContainingBlockContentLogicalWidth());
        unsafe { &*self.rare_data_.Get() }.override_containing_block_content_logical_width_
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:224-228
    pub fn HasOverrideContainingBlockContentLogicalWidth(&self) -> bool {
        self.CheckIsNotDestroyed();
        let rare_data = self.rare_data_.Get();
        !rare_data.is_null()
            && unsafe { &*rare_data }.has_override_containing_block_content_logical_width_
    }

    // cpp: layoutng/internal/layout_box.h:1279-1284
    pub fn EnsureRareData(&mut self) -> &mut LayoutBoxRareData {
        self.CheckIsNotDestroyed();
        if self.rare_data_.Get().is_null() {
            self.rare_data_ = Member::from_ptr(MakeGarbageCollected(LayoutBoxRareData::new()));
        }
        unsafe { &mut *self.rare_data_.Get() }
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:230-237
    pub fn SetOverrideContainingBlockContentLogicalWidth(&mut self, logical_width: LayoutUnit) {
        self.CheckIsNotDestroyed();
        debug_assert!(logical_width >= LayoutUnit::from_signed(-1));
        self.EnsureRareData()
            .override_containing_block_content_logical_width_ = logical_width;
        self.EnsureRareData()
            .has_override_containing_block_content_logical_width_ = true;
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:491-495
    pub fn IsCustomItem(&self) -> bool {
        self.CheckIsNotDestroyed();
        let parent = self.Parent();
        !parent.is_null() && unsafe { &*parent }.IsCustomLayoutLoaded()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:497-507
    pub fn HasUnsplittableScrollingOverflow(&self) -> bool {
        self.CheckIsNotDestroyed();
        if self.IsPrintingForLayout() {
            return false;
        }
        self.IsScrollContainer()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:526-538
    pub fn IsFixedToView(&self, container_for_fixed_position: *const LayoutObject) -> bool {
        self.CheckIsNotDestroyed();
        if !self.IsFixedPositioned() {
            return false;
        }
        let container = if container_for_fixed_position.is_null() {
            self.Container() as *const LayoutObject
        } else {
            debug_assert_eq!(
                container_for_fixed_position,
                self.Container() as *const LayoutObject
            );
            container_for_fixed_position
        };
        unsafe { &*container }.IsLayoutView()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:571-574
    pub fn PhysicalLocation(&self) -> PhysicalOffset {
        unsafe { LayoutBoxPhysicalLocationProvider(self) }
    }
    pub fn PhysicalLocationBase(&self) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        self.frame_location_
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:706-712
    pub fn GetAnchorPositionScrollData(&self) -> *mut AnchorPositionScrollData {
        self.CheckIsNotDestroyed();
        let element = DynamicTo::<Element>(self.GetNode());
        if !element.is_null() {
            return unsafe { &*element }.GetAnchorPositionScrollData();
        }
        std::ptr::null_mut()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:714-720
    pub fn NeedsAnchorPositionScrollAdjustment(&self) -> bool {
        self.CheckIsNotDestroyed();
        let data = self.GetAnchorPositionScrollData();
        if !data.is_null() {
            return unsafe { &*data }.NeedsScrollAdjustment();
        }
        false
    }

    // Preserve the source spelling for this public C++ method.
    // cpp: layoutng/internal/layout_box_geometry.cc:722-730
    pub fn AnchorPositionScrollAdjustmentAfectedByViewportScrolling(&self) -> bool {
        self.CheckIsNotDestroyed();
        let data = self.GetAnchorPositionScrollData();
        if !data.is_null() {
            return unsafe { &*data }.NeedsScrollAdjustment()
                && unsafe { &*data }.IsAffectedByViewportScrolling();
        }
        false
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:732-738
    pub fn AnchorPositionScrollTranslationOffset(&self) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        let data = self.GetAnchorPositionScrollData();
        if !data.is_null() {
            return unsafe { &*data }.TranslationAsPhysicalOffset();
        }
        PhysicalOffset::default()
    }

    // cpp: layoutng/internal/layout_box.h:607-610
    pub fn GetLayoutResults(&self) -> &LayoutResultList {
        self.CheckIsNotDestroyed();
        &self.layout_results_
    }

    // cpp: layoutng/internal/layout_box.h:602-605
    pub fn PhysicalFragments(&self) -> PhysicalFragmentList<'_> {
        self.CheckIsNotDestroyed();
        PhysicalFragmentList::new(&self.layout_results_)
    }

    // cpp: layoutng/internal/layout_box.h:1117-1129
    pub fn MayHaveFragmentItems(&self) -> bool {
        self.CheckIsNotDestroyed();
        (self.ChildrenInline() || self.NeedsLayout())
            && self.PhysicalFragments().MayHaveFragmentItems()
    }

    pub fn HasFragmentItems(&self) -> bool {
        self.CheckIsNotDestroyed();
        (self.ChildrenInline() || self.NeedsLayout()) && self.PhysicalFragments().HasFragmentItems()
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:119-131
    pub fn DisassociatePhysicalFragments(&mut self) {
        self.CheckIsNotDestroyed();
        if self.FirstInlineFragmentItemIndex() != 0 {
            FragmentItems::LayoutObjectWillBeDestroyed(self);
            self.ClearFirstInlineFragmentItemIndex();
        }
        let measure_cache = self.measure_cache_.Get();
        if !measure_cache.is_null() {
            unsafe { &*measure_cache }.LayoutObjectWillBeDestroyed();
        }
        for result in self.layout_results_.iter() {
            unsafe { &*result.Get() }
                .GetPhysicalFragment()
                .LayoutObjectWillBeDestroyed();
        }
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:186-207
    pub fn AddMeasureLayoutResult(&mut self, result: *const LayoutResult) {
        self.CheckIsNotDestroyed();
        let result_ref = unsafe { &*result };
        if result_ref.Status() != EStatus::kSuccess {
            return;
        }
        if result_ref.GetConstraintSpaceForCaching().CacheSlot() != LayoutResultCacheSlot::kMeasure
        {
            return;
        }
        let box_fragment = To::<PhysicalBoxFragment>(result_ref.GetPhysicalFragment() as *const _);
        debug_assert!(unsafe { &*box_fragment }.IsOnlyForNode());
        if self.measure_cache_.Get().is_null() {
            self.measure_cache_ = Member::from_ptr(MakeGarbageCollected(MeasureCache::default()));
        }
        let measure_cache = unsafe { &mut *self.measure_cache_.Get() };
        if self.NeedsLayout() && !self.NeedsSimplifiedLayoutOnly() {
            measure_cache.Clear();
        }
        measure_cache.Add(result);
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:209-243
    pub fn SetCachedLayoutResult(&mut self, result: *const LayoutResult, index: usize) {
        self.CheckIsNotDestroyed();
        let result_ref = unsafe { &*result };
        if result_ref.GetConstraintSpaceForCaching().CacheSlot() == LayoutResultCacheSlot::kMeasure
        {
            debug_assert!(result_ref.GetPhysicalFragment().GetBreakToken().is_null());
            let box_fragment =
                To::<PhysicalBoxFragment>(result_ref.GetPhysicalFragment() as *const _);
            debug_assert!(unsafe { &*box_fragment }.IsOnlyForNode());
            debug_assert_eq!(index, 0);
            self.AddMeasureLayoutResult(result);
            if self.IsTableCell() {
                unsafe { InvalidateLayoutResultCacheAfterMeasureProvider(self) };
            }
        } else if self.NeedsLayout() && !self.NeedsSimplifiedLayoutOnly() {
            let measure_cache = self.measure_cache_.Get();
            if !measure_cache.is_null() {
                unsafe { &mut *measure_cache }.Clear();
            }
        }
        let measure_cache = self.measure_cache_.Get();
        if !measure_cache.is_null() {
            unsafe { &*measure_cache }.SetFragmentChildrenInvalid(result);
        }
        self.SetLayoutResult(result, index);
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:245-290
    pub fn SetLayoutResult(&mut self, result: *const LayoutResult, index: usize) {
        self.CheckIsNotDestroyed();
        let result_ref = unsafe { &*result };
        debug_assert_eq!(result_ref.Status(), EStatus::kSuccess);
        let box_fragment = To::<PhysicalBoxFragment>(result_ref.GetPhysicalFragment() as *const _);
        let box_fragment = unsafe { &*box_fragment };
        if index != usize::MAX && self.layout_results_.len() > index {
            if self.layout_results_.len() > index + 1 {
                let break_token = box_fragment.GetBreakToken();
                if break_token.is_null()
                    || unsafe { &*break_token }.IsCausedByColumnSpanner()
                    || box_fragment.IsFragmentationContextRoot()
                {
                    if box_fragment.IsInlineFormattingContext() {
                        FragmentItems::ClearAssociatedFragments(
                            self as *mut LayoutBox as *mut LayoutObject,
                        );
                    }
                    self.ShrinkLayoutResults(index + 1);
                }
            }
            self.ReplaceLayoutResult(result, index);
            return;
        }
        debug_assert!(index == self.layout_results_.len() || index == usize::MAX);
        self.AppendLayoutResult(result);
        if box_fragment.GetBreakToken().is_null() {
            self.FinalizeLayoutResults();
        }
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:292-300
    pub fn AppendLayoutResult(&mut self, result: *const LayoutResult) {
        self.CheckIsNotDestroyed();
        let fragment =
            To::<PhysicalBoxFragment>(unsafe { &*result }.GetPhysicalFragment() as *const _);
        debug_assert!(!DisableLayoutSideEffectsScope::IsDisabled());
        self.layout_results_
            .push(Member::from_ptr(result as *mut LayoutResult));
        self.InvalidateCachedGeometry();
        CheckDidAddFragment(self, unsafe { &*fragment }, usize::MAX);
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:302-341
    pub fn ReplaceLayoutResult(&mut self, result: *const LayoutResult, index: usize) {
        self.CheckIsNotDestroyed();
        debug_assert!(index <= self.layout_results_.len());
        let old_result = self.layout_results_[index].Get();
        if old_result == result as *mut LayoutResult {
            return;
        }
        let fragment =
            To::<PhysicalBoxFragment>(unsafe { &*result }.GetPhysicalFragment() as *const _);
        let fragment = unsafe { &*fragment };
        let old_fragment = unsafe { &*old_result }.GetPhysicalFragment();
        let got_new_fragment = old_fragment as *const _ != fragment as *const _ as *const _;
        if got_new_fragment {
            if self.HasFragmentItems() {
                FragmentItems::ClearAssociatedFragments(
                    self as *mut LayoutBox as *mut LayoutObject,
                );
            }
            if self.layout_results_.len() > 1 && fragment.Size() != old_fragment.Size() {
                self.SetShouldDoFullPaintInvalidation();
            }
        }
        debug_assert!(!DisableLayoutSideEffectsScope::IsDisabled());
        self.layout_results_[index] = Member::from_ptr(result as *mut LayoutResult);
        self.InvalidateCachedGeometry();
        CheckDidAddFragment(self, fragment, index);
        if got_new_fragment && fragment.GetBreakToken().is_null() {
            debug_assert_eq!(index, self.layout_results_.len() - 1);
            self.FinalizeLayoutResults();
        }
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:343-356
    pub fn FinalizeLayoutResults(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(!self.layout_results_.is_empty());
        let last = self
            .layout_results_
            .last()
            .expect("layout result required")
            .Get();
        debug_assert!(unsafe { &*last }
            .GetPhysicalFragment()
            .GetBreakToken()
            .is_null());
        #[cfg(feature = "expensive_dchecks")]
        self.CheckMayHaveFragmentItems();
        if self.HasFragmentItems() {
            let results = &self.layout_results_ as *const LayoutResultList;
            let block_flow =
                DynamicTo::<LayoutBlockFlow>(self as *mut LayoutBox as *mut LayoutObject);
            debug_assert!(!block_flow.is_null());
            FragmentItems::FinalizeAfterLayout(unsafe { &*results }, unsafe { &mut *block_flow });
        }
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:358-387
    pub fn RebuildFragmentTreeSpine(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.PhysicalFragmentCount() != 0);
        let mut container = self as *mut LayoutBox;
        while !container.is_null()
            && unsafe { &*container }.PhysicalFragmentCount() != 0
            && !unsafe { &*container }.NeedsLayout()
        {
            let current = unsafe { &mut *container };
            for result in current.layout_results_.iter_mut() {
                let cloned = LayoutResult::CloneWithPostLayoutFragments(unsafe { &*result.Get() });
                *result = Member::from_ptr(cloned as *mut LayoutResult);
            }
            let measure_cache = current.measure_cache_.Get();
            if !measure_cache.is_null() {
                unsafe { &mut *measure_cache }.Clear();
            }
            container = current.ContainingNGBox();
        }
        if !container.is_null() && unsafe { &*container }.NeedsLayout() {
            unsafe { &mut *container }.SetHasBrokenSpine();
        }
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:389-397
    pub fn ShrinkLayoutResults(&mut self, results_to_keep: usize) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.layout_results_.len() >= results_to_keep);
        debug_assert!(!DisableLayoutSideEffectsScope::IsDisabled());
        self.layout_results_.truncate(results_to_keep);
        self.InvalidateCachedGeometry();
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:412-422
    pub fn GetCachedLayoutResult(
        &self,
        break_token: *const BlockBreakToken,
    ) -> *const LayoutResult {
        self.CheckIsNotDestroyed();
        let index = FragmentIndex(break_token);
        if index >= self.layout_results_.len() {
            return std::ptr::null();
        }
        let result = self.layout_results_[index].Get();
        debug_assert!(
            !unsafe { &*result }
                .GetPhysicalFragment()
                .IsLayoutObjectDestroyedOrMoved()
                || self.BeingDestroyed()
        );
        result
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:424-450
    pub fn GetCachedMeasureResult(
        &self,
        space: &ConstraintSpace,
        fragment_geometry: *mut Option<FragmentGeometry>,
    ) -> *const LayoutResult {
        self.CheckIsNotDestroyed();
        let measure_cache = self.measure_cache_.Get();
        if measure_cache.is_null() {
            return std::ptr::null();
        }
        if !self.layout_results_.is_empty() {
            let first_fragment = unsafe { &*self.GetPhysicalFragment(0) };
            if !first_fragment.GetBreakToken().is_null() {
                return std::ptr::null();
            }
        }
        debug_assert!(!fragment_geometry.is_null());
        unsafe { &mut *measure_cache }.Find(
            &BlockNode::new(self as *const LayoutBox as *mut LayoutBox),
            space,
            unsafe { &mut *fragment_geometry },
        )
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:452-455
    pub fn GetSingleCachedLayoutResult(&self) -> *const LayoutResult {
        debug_assert!(self.layout_results_.len() <= 1);
        self.GetCachedLayoutResult(std::ptr::null())
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:642-658
    pub fn HasTopOverflow(&self) -> bool {
        self.CheckIsNotDestroyed();
        // C++ virtual dispatch selects LayoutFlexibleBox::HasTopOverflow.
        // cpp: layoutng_flex/layout_flexible_box.cc:76-78
        if self.RuntimeClass() == LayoutObjectClass::FlexibleBox {
            return FlexOverflowConverter(self.StyleRef()).Top();
        }
        if self.IsHorizontalWritingMode() {
            return false;
        }
        match self.StyleRef().GetWritingMode() {
            WritingMode::kHorizontalTb => false,
            WritingMode::kSidewaysLr => self.StyleRef().IsLeftToRightDirection(),
            WritingMode::kVerticalLr | WritingMode::kVerticalRl | WritingMode::kSidewaysRl => {
                !self.StyleRef().IsLeftToRightDirection()
            }
        }
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:660-676
    pub fn HasLeftOverflow(&self) -> bool {
        self.CheckIsNotDestroyed();
        // cpp: layoutng_flex/layout_flexible_box.cc:80-82
        if self.RuntimeClass() == LayoutObjectClass::FlexibleBox {
            return FlexOverflowConverter(self.StyleRef()).Left();
        }
        if self.IsHorizontalWritingMode() {
            return !self.StyleRef().IsLeftToRightDirection();
        }
        match self.StyleRef().GetWritingMode() {
            WritingMode::kHorizontalTb => !self.StyleRef().IsLeftToRightDirection(),
            WritingMode::kVerticalLr | WritingMode::kSidewaysLr => false,
            WritingMode::kVerticalRl | WritingMode::kSidewaysRl => true,
        }
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:678-763
    pub fn SetScrollableOverflowFromLayoutResults(&mut self) {
        self.CheckIsNotDestroyed();
        self.ClearSelfNeedsScrollableOverflowRecalc();
        self.ClearChildNeedsScrollableOverflowRecalc();
        let overflow = self.overflow_.Get();
        if !overflow.is_null() {
            unsafe { &mut *overflow }.scrollable_overflow = None;
        }
        if self.IsLayoutReplaced() {
            return;
        }
        let writing_mode = self.StyleRef().GetWritingMode();
        let mut scrollable_overflow: Option<PhysicalRect> = None;
        let mut consumed_block_size = LayoutUnit::default();
        let mut fragment_width_sum = LayoutUnit::default();
        for layout_result in self.layout_results_.iter() {
            let fragment = To::<PhysicalBoxFragment>(
                unsafe { &*layout_result.Get() }.GetPhysicalFragment() as *const _,
            );
            let fragment = unsafe { &*fragment };
            let offset_adjust = match writing_mode {
                WritingMode::kHorizontalTb => {
                    PhysicalOffset::new(LayoutUnit::default(), consumed_block_size)
                }
                WritingMode::kVerticalRl | WritingMode::kSidewaysRl => {
                    fragment_width_sum += fragment.Size().width;
                    PhysicalOffset::new(
                        -fragment.Size().width - consumed_block_size,
                        LayoutUnit::default(),
                    )
                }
                WritingMode::kVerticalLr | WritingMode::kSidewaysLr => {
                    PhysicalOffset::new(consumed_block_size, LayoutUnit::default())
                }
            };
            let mut fragment_scrollable_overflow = fragment.ScrollableOverflow();
            fragment_scrollable_overflow.offset += offset_adjust;
            if let Some(overflow) = scrollable_overflow.as_mut() {
                overflow.UniteEvenIfEmpty(&fragment_scrollable_overflow);
            } else {
                scrollable_overflow = Some(fragment_scrollable_overflow);
            }
            let break_token = fragment.GetBreakToken();
            if !break_token.is_null() {
                if unsafe { &*break_token }.IsRepeated() {
                    break;
                }
                consumed_block_size = unsafe { &*break_token }.ConsumedBlockSize();
            }
        }
        let Some(mut scrollable_overflow) = scrollable_overflow else {
            return;
        };
        if self.StyleRef().IsFlippedBlocksWritingMode() {
            scrollable_overflow.offset.left += fragment_width_sum;
        }
        if scrollable_overflow.IsEmpty()
            || self
                .PhysicalPaddingBoxRect()
                .ContainsRect(&scrollable_overflow)
        {
            return;
        }
        debug_assert!(!self.ScrollableOverflowIsSet());
        if self.overflow_.Get().is_null() {
            self.overflow_ = Member::from_ptr(MakeGarbageCollected(BoxOverflowModel::default()));
        }
        unsafe { &mut *self.overflow_.Get() }.scrollable_overflow =
            Some(BoxScrollableOverflowModel::new(&scrollable_overflow));
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:578-592
    pub fn LocationContainer(&self) -> *mut LayoutBox {
        self.CheckIsNotDestroyed();
        if self.IsSVGChild() {
            return std::ptr::null_mut();
        }
        let mut container = self.Container();
        while !container.is_null() && !unsafe { &*container }.IsBox() {
            container = unsafe { &*container }.Container();
        }
        To::<LayoutBox>(container)
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:399-405
    #[cfg(feature = "expensive_dchecks")]
    pub fn CheckMayHaveFragmentItems(&self) {
        self.CheckIsNotDestroyed();
        if !self.MayHaveFragmentItems() {
            debug_assert!(!self.PhysicalFragments().SlowHasFragmentItems());
        }
    }

    // cpp: layoutng/internal/layout_box.h:606-606
    // cpp: layoutng/internal/layout_box_fragment_data.cc:462-465
    pub fn GetLayoutResult(&self, i: usize) -> *const LayoutResult {
        self.CheckIsNotDestroyed();
        self.layout_results_[i].Get()
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:407-410
    pub fn InvalidateCachedGeometry(&self) {
        self.CheckIsNotDestroyed();
        self.SetHasValidCachedGeometry(false);
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:457-460
    pub fn GetSingleCachedMeasureResultForTesting(&self) -> *const LayoutResult {
        self.CheckIsNotDestroyed();
        let measure_cache = self.measure_cache_.Get();
        if measure_cache.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*measure_cache }.GetLastForTesting()
        }
    }

    // cpp: layoutng/internal/layout_box.h:607-611
    // cpp: layoutng/internal/layout_box_hot.cc:464-467
    pub fn GetPhysicalFragment(&self, i: usize) -> *const PhysicalBoxFragment {
        self.CheckIsNotDestroyed();
        let result = unsafe { &*self.layout_results_[i].Get() };
        result.GetPhysicalFragment() as *const _ as *const PhysicalBoxFragment
    }

    // cpp: layoutng/internal/layout_box.h:614-617
    pub fn PhysicalFragmentCount(&self) -> usize {
        self.CheckIsNotDestroyed();
        self.layout_results_.len()
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:133-136
    pub fn HasInlineFragments(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.first_fragment_item_index_ != 0
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:138-142
    pub fn ClearFirstInlineFragmentItemIndex(&mut self) {
        self.CheckIsNotDestroyed();
        assert!(self.IsInLayoutNGInlineFormattingContext());
        self.first_fragment_item_index_ = 0;
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:144-149
    pub fn SetFirstInlineFragmentItemIndex(&mut self, index: usize) {
        self.CheckIsNotDestroyed();
        assert!(self.IsInLayoutNGInlineFormattingContext());
        debug_assert_ne!(index, 0);
        self.first_fragment_item_index_ = index;
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:151-155
    pub fn InLayoutNGInlineFormattingContextWillChange(&mut self, _new_value: bool) {
        self.CheckIsNotDestroyed();
        if self.IsInLayoutNGInlineFormattingContext() {
            self.ClearFirstInlineFragmentItemIndex();
        }
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:676-689
    // cpp: layoutng/internal/layout_box_geometry.cc:691-704
    pub fn OutOfFlowInsetsForGetComputedStyle(&self) -> &BoxStrut {
        self.CheckIsNotDestroyed();
        let layout_results = self.GetLayoutResults();
        let first = layout_results
            .iter()
            .next()
            .expect("out-of-flow insets require a completed layout result");
        #[cfg(feature = "expensive_dchecks")]
        {
            let mut previous = unsafe { &*first.Get() };
            for result in layout_results.iter().skip(1) {
                let current = unsafe { &*result.Get() };
                debug_assert_eq!(
                    current.OutOfFlowInsetsForGetComputedStyle(),
                    previous.OutOfFlowInsetsForGetComputedStyle(),
                );
                previous = current;
            }
        }
        unsafe { &*first.Get() }.OutOfFlowInsetsForGetComputedStyle()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:676-689
    // cpp: layoutng/internal/layout_box_geometry.cc:740-754
    pub fn NeedsAnchorPositionScrollAdjustmentInX(&self) -> bool {
        self.CheckIsNotDestroyed();
        let layout_results = self.GetLayoutResults();
        let Some(first) = layout_results.iter().next() else {
            return false;
        };
        #[cfg(feature = "expensive_dchecks")]
        {
            let mut previous = unsafe { &*first.Get() };
            for result in layout_results.iter().skip(1) {
                let current = unsafe { &*result.Get() };
                debug_assert_eq!(
                    current.NeedsAnchorPositionScrollAdjustmentInX(),
                    previous.NeedsAnchorPositionScrollAdjustmentInX(),
                );
                previous = current;
            }
        }
        unsafe { &*first.Get() }.NeedsAnchorPositionScrollAdjustmentInX()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:676-689
    // cpp: layoutng/internal/layout_box_geometry.cc:756-770
    pub fn NeedsAnchorPositionScrollAdjustmentInY(&self) -> bool {
        self.CheckIsNotDestroyed();
        let layout_results = self.GetLayoutResults();
        let Some(first) = layout_results.iter().next() else {
            return false;
        };
        #[cfg(feature = "expensive_dchecks")]
        {
            let mut previous = unsafe { &*first.Get() };
            for result in layout_results.iter().skip(1) {
                let current = unsafe { &*result.Get() };
                debug_assert_eq!(
                    current.NeedsAnchorPositionScrollAdjustmentInY(),
                    previous.NeedsAnchorPositionScrollAdjustmentInY(),
                );
                previous = current;
            }
        }
        unsafe { &*first.Get() }.NeedsAnchorPositionScrollAdjustmentInY()
    }

    // cpp: layoutng/internal/layout_box.h:1363-1368
    pub fn FirstInlineFragmentItemIndex(&self) -> usize {
        self.CheckIsNotDestroyed();
        if !self.IsInLayoutNGInlineFormattingContext() {
            return 0;
        }
        self.first_fragment_item_index_
    }
}

impl Deref for LayoutBox {
    type Target = LayoutBoxModelObject;

    fn deref(&self) -> &Self::Target {
        &self.model_object_
    }
}

impl DerefMut for LayoutBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.model_object_
    }
}

unsafe extern "Rust" {
    fn LayoutBoxAdjustOutOfFlowContainingBlockForAnchorProvider(
        box_: &LayoutBox,
        data: *const GridLayoutData,
        style: &ComputedStyle,
        padding: &LogicalRect,
        query: &LayoutBox,
    ) -> LogicalRect;
}
