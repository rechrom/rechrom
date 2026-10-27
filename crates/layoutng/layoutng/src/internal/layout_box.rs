#![allow(non_snake_case)]

use foundation::graphics_types;
use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use super::algorithm_forward::CustomLayoutChild;
use foundation::{
    gfx, kIndefiniteSize, DynamicTo, EColumnSpan, EOverflow, EPosition, EReadingFlow,
    GCedHeapHashSet, GCedHeapVector, HeapVector, InfiniteIntRect, LayoutUnit, LogicalToPhysical,
    MakeGarbageCollected, Member, OverlayScrollbarClipBehavior, PhysicalOffset, PhysicalRect,
    PhysicalSize, ScopedCSSName, TextDirection, To, TransformAccumulation, TransformState,
    ValueForLength, Vector, WritingDirectionMode, WritingMode,
};
use graphics_types::graphics::paint::display_item_client_types::RasterEffectOutset;
use graphics_types::graphics::visual_rect_flags::{VisualRectFlag, VisualRectFlags};
use layout_fragment::block_break_token::BlockBreakToken;
use layout_fragment::fragment_data::FragmentData;
use layout_fragment::fragment_items::FragmentItems;
use layout_fragment::inline_cursor::InlineCursor;
use layout_fragment::layout_result::{EStatus, LayoutResult};
use layout_fragment::logical_fragment::LogicalFragment;
use layout_fragment::physical_box_fragment::PhysicalBoxFragment;
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
use super::caret_rect::CaretShape;
use super::column_spanner_path::ColumnSpannerPath;
use super::constraint_space::ConstraintSpace;
use super::constraint_space::LayoutResultCacheSlot;
use super::disable_layout_side_effects_scope::DisableLayoutSideEffectsScope;
use super::early_break::EarlyBreak;
use super::editing::forward::PositionWithAffinity;
use super::form_node_metadata::{HTMLFieldSetElement, HTMLImageElement};
use super::fragmentation_utils::{BoxFragmentIndex, FragmentIndex};
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
use super::layout_utils::LayoutCacheStatus;
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

#[path = "../../../layout_fragment/src/native_layout_box_fragment_geometry.rs"]
mod layout_fragment_native_layout_box_fragment_geometry;
use layout_fragment_native_layout_box_fragment_geometry::{
    LayoutBoxBoundingBoxRelativeToFirstFragment, LayoutBoxComputeSize,
    LayoutBoxFindPreviousBreakToken, LayoutBoxLocationContainer,
    LayoutBoxPhysicalContractedBoxRect, LayoutBoxQuadsInAncestorInternal,
    LayoutBoxResolvedDirection, LayoutBoxStitchedBlockSize,
};

#[path = "../../../layout_fragment/src/native_layout_box_fragment_access.rs"]
mod layout_fragment_native_layout_box_fragment_access;

#[path = "../../../layout_fragment/src/native_layout_result_overflow.rs"]
mod layout_fragment_native_layout_result_overflow;
use layout_fragment_native_layout_result_overflow::LayoutBoxSetScrollableOverflowFromLayoutResults;

#[path = "../../../layout_fragment/src/native_physical_fragment_list.rs"]
mod layout_fragment_native_physical_fragment_list;
pub use layout_fragment_native_physical_fragment_list::{
    PhysicalFragmentIterator, PhysicalFragmentList,
};

#[path = "../../../layout_invalidation/src/native_transform_invalidation.rs"]
mod layout_invalidation_native_transform_invalidation;
use layout_invalidation_native_transform_invalidation::LayoutBoxTransformsChangeMayRequireLayout;

#[path = "../../../layout_invalidation/src/native_layout_box_fragment_retirement.rs"]
mod layout_invalidation_native_layout_box_fragment_retirement;
use layout_invalidation_native_layout_box_fragment_retirement::LayoutBoxDisassociatePhysicalFragments;

#[path = "../../../layout_reuse/src/native_layout_box_result_cache.rs"]
mod layout_reuse_native_layout_box_result_cache;
use layout_reuse_native_layout_box_result_cache::{
    LayoutBoxAddMeasureLayoutResult, LayoutBoxGetCachedLayoutResult,
    LayoutBoxGetCachedMeasureResult, LayoutBoxGetSingleCachedLayoutResult,
    LayoutBoxGetSingleCachedMeasureResultForTesting, LayoutBoxSetCachedLayoutResult,
};

#[path = "../../../layout_commit/src/native_layout_box_result_commit.rs"]
mod layout_commit_native_layout_box_result_commit;
use layout_commit_native_layout_box_result_commit::{
    LayoutBoxAppendLayoutResult, LayoutBoxFinalizeLayoutResults, LayoutBoxRebuildFragmentTreeSpine,
    LayoutBoxReplaceLayoutResult, LayoutBoxSetLayoutResult, LayoutBoxShrinkLayoutResults,
};

#[path = "../../../layout_algo_block/src/native_column_spanner_admission.rs"]
mod layout_algo_block_native_column_spanner_admission;
use layout_algo_block_native_column_spanner_admission::{
    LayoutBoxDoesAncestryAllowColumnSpanner, LayoutBoxIsSelfValidColumnSpanner,
    LayoutBoxIsValidColumnSpannerInTree, LayoutBoxShouldPreventColumnSpannerDescendants,
};

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

// cpp: layoutng/internal/layout_box.h:299-302
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContractionEdge {
    kContractToPaddingEdge,
    kContractToContentEdge,
}

// cpp: layoutng/internal/fragmentation_utils.cc:1404-1434
pub(crate) fn FindPreviousBreakToken(fragment: &PhysicalBoxFragment) -> *const BlockBreakToken {
    LayoutBoxFindPreviousBreakToken(fragment)
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
        // cpp: layoutng/internal/layout_box.cc
        // LayoutBox::FragmentDataFromPhysicalFragment
        self.FragmentList().at(BoxFragmentIndex(fragment) as u32)
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

    pub(crate) fn HasCachedLayoutResults(&self) -> bool {
        !self.layout_results_.is_empty()
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
        crate::layout_reuse_package::cached_layout_result(
            self,
            new_space,
            break_token,
            early_break,
            column_spanner_path,
            initial_fragment_geometry,
            out_cache_status,
        )
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
        LayoutBoxBoundingBoxRelativeToFirstFragment(self)
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:647-650
    pub fn DebugRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        PhysicalRect::new(self.PhysicalLocation(), self.StitchedSize())
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:772-779
    pub fn TransformsChangeMayRequireLayout(&self) -> bool {
        LayoutBoxTransformsChangeMayRequireLayout(self)
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
        LayoutBoxIsValidColumnSpannerInTree(self, style)
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:416-433
    pub fn IsSelfValidColumnSpanner(&self, style: &ComputedStyle) -> bool {
        LayoutBoxIsSelfValidColumnSpanner(self, style)
    }

    // cpp: layoutng/internal/layout_box.h:638-641
    pub fn IsSelfValidColumnSpannerCurrentStyle(&self) -> bool {
        self.IsSelfValidColumnSpanner(self.StyleRef())
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:435-449
    pub fn DoesAncestryAllowColumnSpanner(&self, _style: &ComputedStyle) -> bool {
        LayoutBoxDoesAncestryAllowColumnSpanner(self)
    }

    // cpp: layoutng/internal/layout_box.h:650-653
    pub fn DoesAncestryAllowColumnSpannerCurrentStyle(&self) -> bool {
        self.DoesAncestryAllowColumnSpanner(self.StyleRef())
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:451-478
    pub fn ShouldPreventColumnSpannerDescendants(&self) -> bool {
        LayoutBoxShouldPreventColumnSpannerDescendants(self)
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
        LayoutBoxQuadsInAncestorInternal(self, quads, ancestor, mode, box_type)
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
        LayoutBoxResolvedDirection(self)
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
        LayoutBoxStitchedBlockSize(self)
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
        LayoutBoxPhysicalContractedBoxRect(self, edge)
    }

    // cpp: layoutng/internal/layout_box.h:1314-1315
    // cpp: layoutng/internal/layout_box_fragment_data.cc:533-576
    pub fn ComputeSize(&self) -> PhysicalSize {
        LayoutBoxComputeSize(self)
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
        LayoutBoxDisassociatePhysicalFragments(self)
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:186-207
    pub fn AddMeasureLayoutResult(&mut self, result: *const LayoutResult) {
        LayoutBoxAddMeasureLayoutResult(self, result)
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:209-243
    pub fn SetCachedLayoutResult(&mut self, result: *const LayoutResult, index: usize) {
        LayoutBoxSetCachedLayoutResult(self, result, index)
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:245-290
    pub fn SetLayoutResult(&mut self, result: *const LayoutResult, index: usize) {
        LayoutBoxSetLayoutResult(self, result, index)
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:292-300
    pub fn AppendLayoutResult(&mut self, result: *const LayoutResult) {
        LayoutBoxAppendLayoutResult(self, result)
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:302-341
    pub fn ReplaceLayoutResult(&mut self, result: *const LayoutResult, index: usize) {
        LayoutBoxReplaceLayoutResult(self, result, index)
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:343-356
    pub fn FinalizeLayoutResults(&mut self) {
        LayoutBoxFinalizeLayoutResults(self)
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:358-387
    pub fn RebuildFragmentTreeSpine(&mut self) {
        LayoutBoxRebuildFragmentTreeSpine(self)
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:389-397
    pub fn ShrinkLayoutResults(&mut self, results_to_keep: usize) {
        LayoutBoxShrinkLayoutResults(self, results_to_keep)
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:412-422
    pub fn GetCachedLayoutResult(
        &self,
        break_token: *const BlockBreakToken,
    ) -> *const LayoutResult {
        LayoutBoxGetCachedLayoutResult(self, break_token)
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:424-450
    pub fn GetCachedMeasureResult(
        &self,
        space: &ConstraintSpace,
        fragment_geometry: *mut Option<FragmentGeometry>,
    ) -> *const LayoutResult {
        LayoutBoxGetCachedMeasureResult(self, space, fragment_geometry)
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:452-455
    pub fn GetSingleCachedLayoutResult(&self) -> *const LayoutResult {
        LayoutBoxGetSingleCachedLayoutResult(self)
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
        LayoutBoxSetScrollableOverflowFromLayoutResults(self)
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:578-592
    pub fn LocationContainer(&self) -> *mut LayoutBox {
        LayoutBoxLocationContainer(self)
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:457-460
    pub fn GetSingleCachedMeasureResultForTesting(&self) -> *const LayoutResult {
        LayoutBoxGetSingleCachedMeasureResultForTesting(self)
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
