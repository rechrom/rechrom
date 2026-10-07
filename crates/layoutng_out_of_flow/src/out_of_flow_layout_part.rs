#![allow(non_snake_case)]

use foundation::style_constants::EPositionTryOrder;
use foundation::{
    kIndefiniteSize, ClearCollectionScope, DynamicTo, EPosition, GCedHeapHashSet, HeapHashMap,
    HeapVector, IsParallelWritingMode, LayoutUnit, Member, PhysicalSize, RuntimeEnabledFeatures,
    TextDirection, To, Visitor, WritingDirectionMode, WritingMode,
};
use layoutng_assembly::block_break_token::BlockBreakToken;
use layoutng_assembly::box_fragment_builder::BoxFragmentBuilder;
use layoutng_assembly::fragment_builder::FragmentBuilder;
use layoutng_assembly::internal::anchor_map::AnchorMap;
use layoutng_assembly::internal::anchor_scope::ToAnchorScopedName;
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::break_appeal::BreakAppeal;
use layoutng_assembly::internal::constraint_space::{
    AutoSizeBehavior, ConstraintSpace, FragmentationType, LayoutResultCacheSlot,
};
use layoutng_assembly::internal::constraint_space_builder::ConstraintSpaceBuilder;
use layoutng_assembly::internal::css::out_of_flow_data::RememberedScrollOffsets;
use layoutng_assembly::internal::disable_layout_side_effects_scope::DisableLayoutSideEffectsScope;
use layoutng_assembly::internal::fragmentation_utils::{
    CalculateSpaceShortage, ClampedToValidFragmentainerCapacity,
    CreateConstraintSpaceForFragmentainer, FragmentainerSpaceLeft,
    InvolvedInBlockFragmentationForBuilder, IsBreakInside,
    SetupSpaceBuilderForFragmentationFromBuilder, SetupSpaceBuilderForFragmentationFromSpace,
    UpdateMinimalSpaceShortage,
};
use layoutng_assembly::internal::inline_containing_block_utils::{
    InlineContainingBlockGeometry, InlineContainingBlockMap, InlineContainingBlockUtils,
};
use layoutng_assembly::internal::layout_algorithm::LayoutAlgorithmParams;
use layoutng_assembly::internal::layout_box::LayoutBox;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::LayoutObject;
use layoutng_assembly::internal::layout_pass_scope::LayoutPassScope;
use layoutng_assembly::internal::layout_view::LayoutView;
use layoutng_assembly::internal::length_utils::{
    AddScrollbarFreeze, CalculateInitialFragmentGeometry, ColumnInlineProgression, ComputeBorders,
    ComputeBordersForInline, ComputePadding, ComputeReplacedSize, ComputeScrollbarsForNonAnonymous,
    ReplacedSizeMode, ShrinkLogicalSize,
};
use layoutng_assembly::internal::non_overflowing_scroll_range::NonOverflowingScrollRange;
use layoutng_assembly::internal::oof_positioned_node::{
    LogicalOofNodeForFragmentation, LogicalOofPositionedNode, MulticolWithPendingOofs,
    OofContainingBlock, OofInlineContainer,
};
use layoutng_assembly::internal::pagination_utils::{GetPageArea, GetPageBorderBox};
use layoutng_assembly::internal::scroll_layout_scope::FreezeScrollbarsRootScope;
use layoutng_assembly::layout_result::EStatus;
use layoutng_assembly::layout_result::LayoutResult;
use layoutng_assembly::logical_fragment::LogicalFragment;
use layoutng_assembly::logical_fragment_link::{LogicalFragmentLink, LogicalFragmentLinkVector};
use layoutng_assembly::physical_box_fragment::PhysicalBoxFragment;
use layoutng_assembly::physical_fragment::{BoxType, PhysicalFragment};
use layoutng_geometry::geometry::box_sides::{LogicalBoxSides, PhysicalBoxSides};
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_geometry::geometry::fragment_geometry::FragmentGeometry;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::logical_size::{LogicalSize, ToLogicalSize, ToPhysicalSize};
use layoutng_geometry::geometry::scroll_offset_range::LogicalScrollRange;
use layoutng_geometry::geometry::static_position::LogicalStaticPosition;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::{ItemPosition, PositionVisibility};
use layoutng_style::style::position_try_fallbacks::PositionTryFallbacks;
use layoutng_style::style::style_position_anchor::Type as StylePositionAnchorType;

use crate::absolute_utils::{
    ComputeAlignment, ComputeAnchorCenterPosition, ComputeIMCBForPositionFallback,
    ComputeInsetModifiedContainingBlock, ComputeOofBlockDimensions, ComputeOofInlineDimensions,
    ComputeOutOfFlowInsets,
};
use crate::anchor_evaluator_impl::AnchorEvaluatorImpl;
use crate::simplified_oof_layout_algorithm::SimplifiedOofLayoutAlgorithm;

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:183-315
struct OOFCandidateStyleIterator {
    element_: *mut Element,
    original_style_: *const ComputedStyle,
    style_: *const ComputedStyle,
    anchor_evaluator_: *mut AnchorEvaluatorImpl,
    position_try_fallbacks_: *const PositionTryFallbacks,
    try_fallback_index_: Option<usize>,
    container_writing_direction_: WritingDirectionMode,
}

impl OOFCandidateStyleIterator {
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:187-198
    fn new(
        object: &LayoutObject,
        anchor_evaluator: &mut AnchorEvaluatorImpl,
        container_writing_direction: WritingDirectionMode,
        remembered_scroll_offsets: *const RememberedScrollOffsets,
    ) -> Self {
        let original_style = object.StyleRef() as *const ComputedStyle;
        let mut iter = Self {
            element_: DynamicTo::<Element>(object.GetNode()),
            original_style_: original_style,
            style_: original_style,
            anchor_evaluator_: anchor_evaluator,
            position_try_fallbacks_: std::ptr::null(),
            try_fallback_index_: None,
            container_writing_direction_: container_writing_direction,
        };
        unsafe { &mut *iter.anchor_evaluator_ }
            .SetRememberedScrollOffsets(remembered_scroll_offsets);
        iter.Initialize();
        iter
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:200-202
    fn HasPositionTryFallbacks(&self) -> bool {
        !self.position_try_fallbacks_.is_null()
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:207-209
    fn TryFallbackIndex(&self) -> Option<usize> {
        self.try_fallback_index_
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:211-218
    fn GetStyle(&self) -> &ComputedStyle {
        unsafe { &*self.style_ }
    }
    fn GetBaseStyle(&self) -> &ComputedStyle {
        if self.HasPositionTryFallbacks() {
            unsafe { &*self.GetStyle().GetBaseComputedStyleOrThis() }
        } else {
            self.GetStyle()
        }
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:220-233
    fn ActivateBaseStyleForTryAttempt(&mut self) -> &ComputedStyle {
        if !self.HasPositionTryFallbacks() {
            return self.GetStyle();
        }
        let base_style = self.GetBaseStyle() as *const ComputedStyle;
        self.ActivateStyle(unsafe { &*base_style });
        unsafe { &*base_style }
    }
    fn ActivateStyleForChosenFallback(&mut self) -> &ComputedStyle {
        let style = self.GetStyle() as *const ComputedStyle;
        self.ActivateStyle(unsafe { &*style });
        unsafe { &*style }
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:235-241
    fn MoveToNextStyle(&mut self) -> bool {
        assert!(!self.HasPositionTryFallbacks());
        false
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:243-257
    fn MoveToLastSuccessfulOrStyleWithoutFallbacks(&mut self) {
        unsafe { &mut *self.anchor_evaluator_ }.ClearRememberedScrollOffsets();
        self.try_fallback_index_ = None;
        self.style_ = self.original_style_;
    }
    fn MoveToChosenTryFallbackIndex(&mut self, index: Option<usize>) {
        assert!(index.is_none());
        self.UpdateStyle(None, false);
    }
    fn Reset(&mut self) {
        unsafe { &mut *self.anchor_evaluator_ }.ClearRememberedScrollOffsets();
        self.try_fallback_index_ = None;
        self.Initialize();
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:260-263
    fn GetCurrentUsedScrollOffsets(&self) -> *const RememberedScrollOffsets {
        unsafe { &*self.anchor_evaluator_ }.LastUsedScrollOffsets()
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:266-287
    fn Initialize(&mut self) {
        self.style_ = self.original_style_;
        assert!(self.GetStyle().GetPositionTryFallbacks().Get().is_null());
    }
    fn UpdateStyle(&mut self, try_fallback_index: Option<usize>, _initial_update: bool) {
        assert!(try_fallback_index.is_none());
        self.try_fallback_index_ = None;
        self.style_ = self.original_style_;
        unsafe { &mut *self.anchor_evaluator_ }.ClearLastUsedScrollOffsets();
    }
    fn ActivateStyle(&mut self, new_style: &ComputedStyle) {
        assert!(std::ptr::eq(new_style, unsafe { &*self.original_style_ }));
    }
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:1856
const kMaxTryAttempts: usize = 6;

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:1864-1880
struct NonOverflowingCandidate {
    try_fallback_index: Option<usize>,
    offset_info: OffsetInfo,
    non_overflowing_range: NonOverflowingScrollRange,
}

impl NonOverflowingCandidate {
    fn Trace(&self, visitor: &mut Visitor) {
        self.offset_info.Trace(visitor);
        self.non_overflowing_range.Trace(visitor);
    }
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:1882-1897
fn ToPhysicalPositionTryOrder(
    position_try_order: EPositionTryOrder,
    writing_direction: WritingDirectionMode,
) -> EPositionTryOrder {
    match position_try_order {
        EPositionTryOrder::kNormal
        | EPositionTryOrder::kMostWidth
        | EPositionTryOrder::kMostHeight => position_try_order,
        EPositionTryOrder::kMostBlockSize => {
            if writing_direction.IsHorizontal() {
                EPositionTryOrder::kMostHeight
            } else {
                EPositionTryOrder::kMostWidth
            }
        }
        EPositionTryOrder::kMostInlineSize => {
            if writing_direction.IsHorizontal() {
                EPositionTryOrder::kMostWidth
            } else {
                EPositionTryOrder::kMostHeight
            }
        }
    }
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:1901-1935
fn SortNonOverflowingCandidates(
    position_try_order: EPositionTryOrder,
    writing_direction: WritingDirectionMode,
    candidates: &mut HeapVector<NonOverflowingCandidate, kMaxTryAttempts>,
) {
    let physical_position_try_order =
        ToPhysicalPositionTryOrder(position_try_order, writing_direction);
    if physical_position_try_order == EPositionTryOrder::kNormal {
        return;
    }
    candidates.sort_by(|a, b| match physical_position_try_order {
        EPositionTryOrder::kMostWidth => b
            .offset_info
            .imcb_size_for_try_order
            .expect("position-try candidate has no IMCB size")
            .width
            .cmp(
                &a.offset_info
                    .imcb_size_for_try_order
                    .expect("position-try candidate has no IMCB size")
                    .width,
            ),
        EPositionTryOrder::kMostHeight => b
            .offset_info
            .imcb_size_for_try_order
            .expect("position-try candidate has no IMCB size")
            .height
            .cmp(
                &a.offset_info
                    .imcb_size_for_try_order
                    .expect("position-try candidate has no IMCB size")
                    .height,
            ),
        EPositionTryOrder::kNormal
        | EPositionTryOrder::kMostBlockSize
        | EPositionTryOrder::kMostInlineSize => unreachable!("physical try order expected"),
    });
}
use crate::oof_dimensions::LogicalOofDimensions;

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:55-63
fn BoxInlineSize(box_: &LayoutBox) -> LayoutUnit {
    debug_assert!(box_.PhysicalFragmentCount() > 0);
    let fragment = unsafe { &*box_.GetPhysicalFragment(0) };
    ToLogicalSize(fragment.Size(), box_.StyleRef().GetWritingMode()).inline_size
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:66-91
fn BoxTotalBlockSize(box_: &LayoutBox) -> LayoutUnit {
    let mut num_fragments = box_.PhysicalFragmentCount();
    debug_assert!(num_fragments > 0);
    let mut total_block_size = LayoutUnit::default();
    while num_fragments > 0 {
        let fragment = unsafe { &*box_.GetPhysicalFragment(num_fragments - 1) };
        let block_size =
            ToLogicalSize(fragment.Size(), box_.StyleRef().GetWritingMode()).block_size;
        if block_size > LayoutUnit::default() {
            total_block_size += block_size;
            break;
        }
        num_fragments -= 1;
    }
    if num_fragments > 1 {
        let preceding = unsafe { &*box_.GetPhysicalFragment(num_fragments - 2) };
        total_block_size += unsafe { &*preceding.GetBreakToken() }.ConsumedBlockSize();
    }
    total_block_size
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:93-102
fn GetFragmentainerProgression(
    builder: &BoxFragmentBuilder,
    type_: FragmentationType,
) -> LogicalOffset {
    if type_ == FragmentationType::kFragmentColumn {
        let inline_progression =
            ColumnInlineProgression(builder.Style(), builder.ChildAvailableSize().inline_size);
        return LogicalOffset::new(inline_progression, LayoutUnit::default());
    }
    debug_assert_eq!(type_, FragmentationType::kFragmentPage);
    LogicalOffset::new(
        LayoutUnit::default(),
        builder.ChildAvailableSize().block_size,
    )
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:104-117
fn CreateContainerBuilderForMulticol(
    multicol: &BlockNode,
    space: &ConstraintSpace,
    fragment_geometry: &FragmentGeometry,
) -> BoxFragmentBuilder {
    let style = multicol.Style();
    let mut builder = BoxFragmentBuilder::new(
        multicol.clone().into(),
        style,
        space,
        style.GetWritingDirection(),
        std::ptr::null(),
    );
    builder.SetIsNewFormattingContext(true);
    builder.SetInitialFragmentGeometry(fragment_geometry);
    builder.SetIsBlockFragmentationContextRoot();
    builder
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:119-133
fn CreateConstraintSpaceForMulticol(multicol: &BlockNode) -> ConstraintSpace {
    let writing_direction = multicol.Style().GetWritingDirection();
    let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
        writing_direction.GetWritingMode(),
        writing_direction,
        true,
        true,
        false,
    );
    builder.SetAvailableSize(LogicalSize::default());
    builder.ToConstraintSpace()
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:137-177
fn CalculateNonOverflowingRangeInOneAxis(
    margin_box_start: LayoutUnit,
    margin_box_end: LayoutUnit,
    imcb_inset_start: LayoutUnit,
    imcb_inset_end: LayoutUnit,
    has_non_auto_inset_start: bool,
    has_non_auto_inset_end: bool,
    out_scroll_min: &mut Option<LayoutUnit>,
    out_scroll_max: &mut Option<LayoutUnit>,
) -> bool {
    let start_available_space = margin_box_start - imcb_inset_start;
    let end_available_space = imcb_inset_end - margin_box_end;
    if !has_non_auto_inset_start {
        *out_scroll_max = Some(start_available_space);
    }
    if !has_non_auto_inset_end {
        *out_scroll_min = Some(-end_available_space);
    }
    if start_available_space < LayoutUnit::default() {
        return true;
    }
    if end_available_space < LayoutUnit::default() {
        return true;
    }
    false
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:317-359
fn GetPositionAnchorElement(
    node: &BlockNode,
    style: &ComputedStyle,
    anchor_map: *const AnchorMap,
) -> *const Element {
    if anchor_map.is_null() {
        return std::ptr::null();
    }
    let default_anchor_data = style.GetDefaultAnchorData();
    match default_anchor_data.GetType() {
        StylePositionAnchorType::kNone => std::ptr::null(),
        StylePositionAnchorType::kAuto => {
            let element = DynamicTo::<Element>(node.GetDOMNode());
            if element.is_null() {
                std::ptr::null()
            } else {
                unsafe { &*element }.ImplicitAnchorElement()
            }
        }
        StylePositionAnchorType::kName => {
            let anchored_box = unsafe { &*node.GetLayoutBox() };
            let actual_containing_block = if anchored_box.MightBeInsideFragmentationContext() {
                anchored_box.Container()
            } else {
                std::ptr::null_mut()
            };
            let reference = unsafe { &*anchor_map }.AnchorReference(
                anchored_box,
                actual_containing_block,
                ToAnchorScopedName(default_anchor_data.GetName(), anchored_box),
            );
            if reference.is_null() {
                std::ptr::null()
            } else {
                debug_assert!(!unsafe { &*reference }.GetLayoutObject().is_null());
                unsafe { &*reference }.GetElement()
            }
        }
        StylePositionAnchorType::kNormal => unreachable!("normal default anchor data"),
    }
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:361-369
fn CalculateScrollDirection(
    node: &BlockNode,
    writing_direction: WritingDirectionMode,
) -> LogicalBoxSides {
    let has_top_overflow = node.HasTopOverflow();
    let has_left_overflow = node.HasLeftOverflow();
    PhysicalBoxSides::new(
        has_top_overflow,
        !has_left_overflow,
        !has_top_overflow,
        has_left_overflow,
    )
    .ToLogical(writing_direction)
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:371-402
fn CalculateScrollRect(
    scroll_direction: LogicalBoxSides,
    container_rect: &LogicalRect,
    padding: &BoxStrut,
    inflow_bounds: &LogicalRect,
) -> LogicalRect {
    let mut rect = *container_rect;
    if scroll_direction.inline_start {
        let offset = inflow_bounds.InlineStartOffset() - padding.inline_start;
        rect.ShiftInlineStartEdgeTo(offset.min(rect.InlineStartOffset()));
    } else {
        let offset = inflow_bounds.InlineEndOffset() + padding.inline_end;
        rect.ShiftInlineEndEdgeTo(offset.max(rect.InlineEndOffset()));
    }
    if scroll_direction.block_start {
        let offset = inflow_bounds.BlockStartOffset() - padding.block_start;
        rect.ShiftBlockStartEdgeTo(offset.min(rect.BlockStartOffset()));
    } else {
        let offset = inflow_bounds.BlockEndOffset() + padding.block_end;
        rect.ShiftBlockEndEdgeTo(offset.max(rect.BlockEndOffset()));
    }
    rect
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:48-87
pub struct ColumnBalancingInfo {
    pub out_of_flow_fragmentainer_descendants: HeapVector<LogicalOofNodeForFragmentation>,
    pub minimal_space_shortage: LayoutUnit,
    pub num_new_columns: u32,
    pub has_violating_break: bool,
}

impl Default for ColumnBalancingInfo {
    fn default() -> Self {
        Self {
            out_of_flow_fragmentainer_descendants: HeapVector::default(),
            minimal_space_shortage: kIndefiniteSize,
            num_new_columns: 0,
            has_violating_break: false,
        }
    }
}

impl ColumnBalancingInfo {
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:55-62
    pub fn HasOutOfFlowFragmentainerDescendants(&self) -> bool {
        !self.out_of_flow_fragmentainer_descendants.is_empty()
    }

    pub fn SwapOutOfFlowFragmentainerDescendants(
        &mut self,
        descendants: &mut HeapVector<LogicalOofNodeForFragmentation>,
    ) {
        debug_assert!(descendants.is_empty());
        std::mem::swap(&mut self.out_of_flow_fragmentainer_descendants, descendants);
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:64-64
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:3057-3060
    pub fn PropagateSpaceShortage(&mut self, space_shortage: LayoutUnit) {
        UpdateMinimalSpaceShortage(Some(space_shortage), &mut self.minimal_space_shortage);
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:84-86
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.out_of_flow_fragmentainer_descendants);
    }
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:137-163
#[derive(Clone)]
pub struct ContainingBlockInfo {
    pub writing_direction: WritingDirectionMode,
    pub is_scroll_container: bool,
    pub is_hidden_for_paint: bool,
    pub rect: LogicalRect,
    pub scroll_rect: Option<LogicalRect>,
    pub scroll_limit_rect: Option<LogicalRect>,
    pub scroll_direction: LogicalBoxSides,
    pub relative_offset: LogicalOffset,
}

impl Default for ContainingBlockInfo {
    fn default() -> Self {
        Self {
            writing_direction: WritingDirectionMode::new(
                WritingMode::kHorizontalTb,
                TextDirection::kLtr,
            ),
            is_scroll_container: false,
            is_hidden_for_paint: false,
            rect: LogicalRect::default(),
            scroll_rect: None,
            scroll_limit_rect: None,
            scroll_direction: LogicalBoxSides::with_value(false),
            relative_offset: LogicalOffset::default(),
        }
    }
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:171-183
pub struct MulticolChildInfo {
    pub parent_break_token: Member<BlockBreakToken>,
}

impl Default for MulticolChildInfo {
    fn default() -> Self {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        Self {
            parent_break_token: Member::default(),
        }
    }
}

impl MulticolChildInfo {
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:3062-3064
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.parent_break_token);
    }
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:186-218
#[derive(Clone)]
pub struct NodeInfo {
    pub node: BlockNode,
    pub static_position: LogicalStaticPosition,
    pub base_container_info: ContainingBlockInfo,
    pub default_writing_direction: WritingDirectionMode,
    pub containing_block: OofContainingBlock<LogicalOffset>,
    pub fixedpos_containing_block: OofContainingBlock<LogicalOffset>,
    pub fixedpos_inline_container: OofInlineContainer<LogicalOffset>,
    pub break_token: Member<BlockBreakToken>,
    pub requires_content_before_breaking: bool,
}

impl NodeInfo {
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:201-217
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        node: BlockNode,
        static_position: LogicalStaticPosition,
        base_container_info: ContainingBlockInfo,
        default_writing_direction: WritingDirectionMode,
        _is_fragmentainer_descendant: bool,
        containing_block: &OofContainingBlock<LogicalOffset>,
        fixedpos_containing_block: &OofContainingBlock<LogicalOffset>,
        fixedpos_inline_container: &OofInlineContainer<LogicalOffset>,
        break_token: *const BlockBreakToken,
        requires_content_before_breaking: bool,
    ) -> Self {
        Self {
            node,
            static_position,
            base_container_info,
            default_writing_direction,
            containing_block: containing_block.clone(),
            fixedpos_containing_block: fixedpos_containing_block.clone(),
            fixedpos_inline_container: fixedpos_inline_container.clone(),
            break_token: Member::from_ptr(break_token as *mut BlockBreakToken),
            requires_content_before_breaking,
        }
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:3066-3072
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.node);
        visitor.Trace(&self.containing_block);
        visitor.Trace(&self.fixedpos_containing_block);
        visitor.Trace(&self.fixedpos_inline_container);
        visitor.Trace(&self.break_token);
    }
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:223-274
#[derive(Clone)]
pub struct OffsetInfo {
    pub insets_for_get_computed_style: BoxStrut,
    pub offset: LogicalOffset,
    pub initial_layout_result: Member<LayoutResult>,
    pub container_content_size: LogicalSize,
    pub imcb_block_size: LayoutUnit,
    pub block_auto_size_behavior: AutoSizeBehavior,
    pub node_dimensions: LogicalOofDimensions,
    pub original_offset: LogicalOffset,
    pub non_overflowing_scroll_ranges: HeapVector<NonOverflowingScrollRange>,
    pub imcb_size_for_try_order: Option<PhysicalSize>,
    pub inline_size_depends_on_min_max_sizes: bool,
    pub needs_scroll_adjustment_in_x: bool,
    pub needs_scroll_adjustment_in_y: bool,
    pub overflows_containing_block: bool,
    pub display_locks_affected_by_anchors: Member<GCedHeapHashSet<Member<Element>>>,
}

impl OffsetInfo {
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:3074-3078
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.initial_layout_result);
        visitor.Trace(&self.non_overflowing_scroll_ranges);
        visitor.Trace(&self.display_locks_affected_by_anchors);
    }
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:276-290
#[derive(Clone)]
pub struct NodeToLayout {
    pub node_info: NodeInfo,
    pub offset_info: OffsetInfo,
    pub containing_block_fragment: Member<PhysicalFragment>,
}

impl NodeToLayout {
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:3080-3085
    pub fn Trace(&self, visitor: &mut Visitor) {
        self.node_info.Trace(visitor);
        self.offset_info.Trace(visitor);
        visitor.Trace(&self.containing_block_fragment);
    }
}

// cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:42-46,429-495
pub struct OutOfFlowLayoutPart {
    pub container_builder_: *mut BoxFragmentBuilder,
    pub default_containing_block_: ContainingBlockInfo,
    pub viewport_containing_block_: Option<ContainingBlockInfo>,
    pub containing_blocks_map_: HeapHashMap<Member<LayoutObject>, ContainingBlockInfo>,
    pub repeated_fixed_pos_boxes_: HeapVector<Member<LayoutBox>>,
    pub delayed_descendants_: HeapVector<LogicalOofNodeForFragmentation>,
    pub multicol_children_: *mut HeapVector<MulticolChildInfo>,
    pub column_balancing_info_: *mut ColumnBalancingInfo,
    pub child_fragment_storage_: *mut LogicalFragmentLinkVector,
    pub fragmentainer_consumed_block_size_: LayoutUnit,
    pub is_absolute_container_: bool,
    pub is_fixed_container_: bool,
    pub should_add_outer_fragmentainer_children_: bool,
    pub outer_context_has_fixedpos_container_: bool,
    pub needs_total_page_count_: bool,
    pub additional_pages_were_added_: bool,
}

impl OutOfFlowLayoutPart {
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:418-423
    fn FragmentationContextChildren(&self) -> &LogicalFragmentLinkVector {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        debug_assert!(unsafe { &*self.container_builder_ }.IsBlockFragmentationContextRoot());
        if self.child_fragment_storage_.is_null() {
            unsafe { &*self.container_builder_ }.Children()
        } else {
            unsafe { &*self.child_fragment_storage_ }
        }
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:433-435
    fn ChildCount(&self) -> usize {
        self.FragmentationContextChildren().len()
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:437-445
    fn AddFragmentainer(
        &mut self,
        fragmentainer: &PhysicalBoxFragment,
        fragmentainer_offset: LogicalOffset,
    ) {
        if !self.child_fragment_storage_.is_null() {
            unsafe { &mut *self.child_fragment_storage_ }.push(LogicalFragmentLink::new(
                fragmentainer,
                fragmentainer_offset,
            ));
        } else {
            unsafe { &mut *self.container_builder_ }.AddChild(
                fragmentainer,
                fragmentainer_offset,
                None,
                false,
                None,
                std::ptr::null(),
            );
        }
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:3034-3043
    fn GetChildFragment(&self, index: usize) -> &PhysicalBoxFragment {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        let link = &self.FragmentationContextChildren()[index];
        let fragment = link.get();
        debug_assert!(!fragment.is_null() && unsafe { &*fragment }.IsBox());
        let box_fragment = unsafe { &*fragment.cast::<PhysicalBoxFragment>() };
        if !unsafe { &*self.container_builder_ }
            .Node()
            .IsPaginatedRoot()
        {
            return box_fragment;
        }
        debug_assert_eq!(box_fragment.GetBoxType(), BoxType::kPageContainer);
        GetPageArea(GetPageBorderBox(box_fragment))
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:3045-3055
    fn PreviousFragmentainerBreakToken(&self, index: usize) -> *const BlockBreakToken {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        for i in (0..index).rev() {
            let previous_fragment = self.GetChildFragment(i);
            if previous_fragment.IsFragmentainerBox() {
                return previous_fragment.GetBreakToken();
            }
        }
        std::ptr::null()
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:342-345
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:1599-1697
    fn CreateAnchorEvaluator(
        &self,
        container_info: &ContainingBlockInfo,
        candidate: &BlockNode,
        is_inside_fragmentation_context: bool,
    ) -> AnchorEvaluatorImpl {
        let mut implicit_anchor: *const LayoutObject = std::ptr::null();
        let candidate_layout_box = unsafe { &*candidate.GetLayoutBox() };
        let element = DynamicTo::<Element>(candidate_layout_box.GetNode());
        if !element.is_null() {
            let implicit_anchor_element = unsafe { &*element }.ImplicitAnchorElement();
            if !implicit_anchor_element.is_null() {
                implicit_anchor = unsafe { &*implicit_anchor_element }.GetLayoutObject();
            }
        }

        let builder = unsafe { &*self.container_builder_ };
        let mut container_size = builder.SizeForAnchorQueries();
        let container_rect = container_info.rect;
        let mut scroll_rect = container_info.scroll_rect;

        let anchor_map: *const AnchorMap;
        let containing_block: *const LayoutObject;
        let mut actual_containing_block: *const LayoutObject = std::ptr::null();
        let mut grid_layout_data = std::ptr::null();
        if is_inside_fragmentation_context && !RuntimeEnabledFeatures::FragmentedOofInCbEnabled() {
            debug_assert!(builder.IsBlockFragmentationContextRoot());
            let child_count = self.ChildCount();
            let mut stitched_container_size = LogicalSize::default();
            for idx in 0..child_count {
                let fragment = self.GetChildFragment(idx);
                if !fragment.IsFragmentainerBox() {
                    continue;
                }
                let logical_fragment =
                    LogicalFragment::new(container_info.writing_direction, fragment);
                stitched_container_size.block_size += logical_fragment.BlockSize();
            }
            container_size = stitched_container_size;
            scroll_rect = None;

            let mut stitched_anchor_map: *mut AnchorMap = std::ptr::null_mut();
            let mut stitched_offset = LogicalOffset::default();
            for idx in 0..child_count {
                let fragment = self.GetChildFragment(idx);
                if !fragment.IsFragmentainerBox() {
                    continue;
                }
                let options = builder.AnchorOptionsForChild(fragment);
                let container_object = builder.GetLayoutObject();
                assert!(!container_object.is_null());
                let writing_direction = unsafe { &*container_object }
                    .StyleRef()
                    .GetWritingDirection();
                FragmentBuilder::PropagateChildAnchorsIntoMap(
                    fragment,
                    stitched_offset,
                    unsafe { &*container_object },
                    writing_direction,
                    stitched_container_size,
                    options,
                    &mut stitched_anchor_map,
                );
                let break_token = fragment.GetBreakToken();
                if !break_token.is_null() {
                    stitched_offset.block_offset = unsafe { &*break_token }.ConsumedBlockSize();
                }
            }
            anchor_map = stitched_anchor_map;
            actual_containing_block = candidate_layout_box.Container();
            containing_block = candidate_layout_box.Container();
        } else {
            anchor_map = builder.GetAnchorMap();
            containing_block = builder.Node().GetLayoutBox() as *const LayoutObject;
            grid_layout_data = builder.GetGridLayoutData();
        }

        AnchorEvaluatorImpl::new(
            candidate_layout_box,
            anchor_map,
            implicit_anchor,
            containing_block,
            actual_containing_block,
            grid_layout_data,
            container_info.writing_direction,
            container_size,
            container_rect,
            scroll_rect,
        )
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:359-360
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:1939-2129
    fn CalculateOffset(
        &mut self,
        node_info: &NodeInfo,
        is_inside_fragmentation_context: bool,
    ) -> OffsetInfo {
        let mut non_overflowing_scroll_ranges = HeapVector::<NonOverflowingScrollRange>::default();
        let mut anchor_evaluator = self.CreateAnchorEvaluator(
            &node_info.base_container_info,
            &node_info.node,
            is_inside_fragmentation_context,
        );

        let current_style = node_info.node.Style();
        let has_try_fallbacks = !current_style.GetPositionTryFallbacks().Get().is_null();
        let position_try_order = current_style.PositionTryOrder();
        let has_no_overflow_visibility =
            current_style.HasPositionVisibility(PositionVisibility::kNoOverflow);
        let try_fit_available_space = has_try_fallbacks || has_no_overflow_visibility;
        let mut non_overflowing_candidates =
            HeapVector::<NonOverflowingCandidate, kMaxTryAttempts>::default();

        let element = To::<Element>(node_info.node.GetDOMNode());
        let oof_data = if element.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*element }.GetOutOfFlowData()
        };
        let mut last_successful_index = None;
        let mut find_last_successful_option = false;
        if !oof_data.is_null() {
            let data = unsafe { &*oof_data };
            if data.HasLastSuccessfulPositionFallback()
                && !data.HasStaleFallbackData(unsafe { &*node_info.node.GetLayoutBox() })
            {
                last_successful_index = data.GetLastSuccessfulIndex();
                find_last_successful_option = true;
            }
        }

        let _overflowing_options = HeapVector::<Option<usize>, kMaxTryAttempts>::default();
        let mut iter = OOFCandidateStyleIterator::new(
            unsafe { &*node_info.node.GetLayoutBox() },
            &mut anchor_evaluator,
            node_info.base_container_info.writing_direction,
            if oof_data.is_null() {
                std::ptr::null()
            } else {
                unsafe { &*oof_data }.GetRememberedScrollOffsets()
            },
        );

        loop {
            let mut attempts_left = kMaxTryAttempts;
            loop {
                let mut non_overflowing_range = NonOverflowingScrollRange::default();
                let style = iter.ActivateBaseStyleForTryAttempt() as *const ComputedStyle;
                assert!(has_try_fallbacks || std::ptr::eq(unsafe { &*style }, iter.GetStyle()));
                let offset_info = self.TryCalculateOffset(
                    node_info,
                    unsafe { &*style },
                    &mut anchor_evaluator,
                    iter.TryFallbackIndex(),
                    try_fit_available_space,
                    &mut non_overflowing_range,
                );

                let offsets = iter.GetCurrentUsedScrollOffsets();
                if !offsets.is_null() {
                    let offset = unsafe { &*offsets }
                        .GetOffset(
                            non_overflowing_range.anchor_element.Get(),
                            layoutng_assembly::internal::css::out_of_flow_data::RememberedScrollOffsetType::kRangeAdjustment,
                            anchor_evaluator.NeedsScrollAdjustmentInX(),
                            anchor_evaluator.NeedsScrollAdjustmentInY(),
                        )
                        .unwrap_or_default();
                    non_overflowing_range.containing_block_range.Move(&offset);
                }

                if try_fit_available_space {
                    non_overflowing_scroll_ranges.push(non_overflowing_range.clone());
                }
                if let Some(offset_info) = offset_info {
                    let candidate = NonOverflowingCandidate {
                        try_fallback_index: iter.TryFallbackIndex(),
                        offset_info,
                        non_overflowing_range,
                    };
                    if find_last_successful_option
                        && iter.TryFallbackIndex() == last_successful_index
                    {
                        non_overflowing_candidates.clear();
                        non_overflowing_candidates.push(candidate);
                        find_last_successful_option = false;
                        break;
                    }
                    non_overflowing_candidates.push(candidate);
                }
                if !(non_overflowing_candidates.is_empty()
                    || find_last_successful_option
                    || position_try_order != EPositionTryOrder::kNormal)
                {
                    break;
                }
                attempts_left -= 1;
                if attempts_left == 0 || !has_try_fallbacks || !iter.MoveToNextStyle() {
                    break;
                }
            }
            if !find_last_successful_option {
                break;
            }
            find_last_successful_option = false;
            debug_assert!(!oof_data.is_null());
            non_overflowing_scroll_ranges.clear();
            non_overflowing_candidates.clear();
            iter.Reset();
        }

        SortNonOverflowingCandidates(
            position_try_order,
            node_info.base_container_info.writing_direction,
            &mut non_overflowing_candidates,
        );
        let mut offset_info = non_overflowing_candidates
            .first()
            .map(|candidate| candidate.offset_info.clone());

        if try_fit_available_space {
            let overflows_containing_block;
            if non_overflowing_candidates.is_empty() {
                iter.MoveToLastSuccessfulOrStyleWithoutFallbacks();
                overflows_containing_block = true;
            } else {
                iter.MoveToChosenTryFallbackIndex(non_overflowing_candidates[0].try_fallback_index);
                non_overflowing_scroll_ranges.clear();
                non_overflowing_scroll_ranges
                    .push(non_overflowing_candidates[0].non_overflowing_range.clone());
                overflows_containing_block = false;
            }
            let style = iter.ActivateStyleForChosenFallback() as *const ComputedStyle;
            let mut non_overflowing_range_unused = NonOverflowingScrollRange::default();
            offset_info = self.TryCalculateOffset(
                node_info,
                unsafe { &*style },
                &mut anchor_evaluator,
                iter.TryFallbackIndex(),
                false,
                &mut non_overflowing_range_unused,
            );
            offset_info
                .as_mut()
                .expect("chosen style must produce offset")
                .overflows_containing_block = overflows_containing_block;
        }
        if !element.is_null() {
            unsafe { &mut *element }
                .EnsureOutOfFlowData()
                .SetPendingRememberedScrollOffsets(iter.GetCurrentUsedScrollOffsets());
        }
        let mut offset_info = offset_info.expect("OOF offset must be calculated");

        if try_fit_available_space {
            offset_info.non_overflowing_scroll_ranges = non_overflowing_scroll_ranges;
        } else {
            debug_assert!(offset_info.non_overflowing_scroll_ranges.is_empty());
        }
        offset_info.display_locks_affected_by_anchors =
            Member::from_ptr(anchor_evaluator.GetDisplayLocksAffectedByAnchors());
        if anchor_evaluator.DidResolveAnchorWithRunningTransformAnimation() {
            unsafe { &mut *self.container_builder_ }.SetHasRunningAnchorTransformAnimation();
        }
        offset_info
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:363-370
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:2131-2452
    #[allow(clippy::too_many_arguments)]
    fn TryCalculateOffset(
        &mut self,
        node_info: &NodeInfo,
        candidate_style: &ComputedStyle,
        anchor_evaluator: &mut AnchorEvaluatorImpl,
        _option_index: Option<usize>,
        try_fit_available_space: bool,
        out_non_overflowing_range: &mut NonOverflowingScrollRange,
    ) -> Option<OffsetInfo> {
        debug_assert!(
            node_info.node.Style().GetWritingDirection() == candidate_style.GetWritingDirection()
        );
        debug_assert!(node_info.node.Style().PositionAnchor() == candidate_style.PositionAnchor());

        let has_default_anchor = !anchor_evaluator
            .DefaultAnchor(&candidate_style.GetDefaultAnchorData())
            .is_null();
        let container_info = &node_info.base_container_info;
        let base_rect = if has_default_anchor {
            container_info.scroll_rect.unwrap_or(container_info.rect)
        } else {
            container_info.rect
        };

        let candidate_writing_direction = candidate_style.GetWritingDirection();
        let container_writing_direction = container_info.writing_direction;
        let container_rect = anchor_evaluator
            .AdjustedContainingBlockRect(candidate_style.PositionAreaOffsets(), has_default_anchor);
        let container_physical_content_size = ToPhysicalSize(
            container_rect.size,
            node_info.default_writing_direction.GetWritingMode(),
        );

        let space = {
            let mut builder = ConstraintSpaceBuilder::new(
                self.GetConstraintSpace(),
                candidate_writing_direction,
                true,
            );
            builder.SetAvailableSize(container_rect.size);
            builder.SetPercentageResolutionSize(container_rect.size);
            builder.SetIsHiddenForPaint(container_info.is_hidden_for_paint);
            if self.GetConstraintSpace().IsInitialColumnBalancingPass() {
                SetupSpaceBuilderForFragmentationFromSpace(
                    self.GetConstraintSpace(),
                    &node_info.node,
                    LayoutUnit::default(),
                    self.GetConstraintSpace().FragmentainerBlockSize(),
                    false,
                    &mut builder,
                );
            }
            builder.ToConstraintSpace()
        };

        let alignment = ComputeAlignment(
            candidate_style,
            container_writing_direction,
            candidate_writing_direction,
        );
        let insets = ComputeOutOfFlowInsets(candidate_style, &space.AvailableSize(), &alignment);
        let mut static_position = node_info.static_position;
        static_position.offset += node_info.containing_block.Offset() - container_rect.offset;

        let mut previously_consumed_block_size = LayoutUnit::default();
        let container_break_token = unsafe { &*self.container_builder_ }.PreviousBreakToken();
        if !container_break_token.is_null() && RuntimeEnabledFeatures::FragmentedOofInCbEnabled() {
            previously_consumed_block_size = unsafe { &*container_break_token }.ConsumedBlockSize();
        }
        static_position.offset.block_offset += previously_consumed_block_size;
        static_position = static_position
            .ConvertToPhysical(&WritingModeConverter::new(
                node_info.default_writing_direction,
                container_physical_content_size,
            ))
            .ConvertToLogical(&WritingModeConverter::new(
                candidate_writing_direction,
                container_physical_content_size,
            ));

        let imcb = ComputeInsetModifiedContainingBlock(
            &node_info.node,
            &space.AvailableSize(),
            &alignment,
            &insets,
            &static_position,
            container_writing_direction,
            candidate_writing_direction,
        );
        let border_padding =
            ComputeBorders(&space, &node_info.node) + ComputePadding(&space, candidate_style);

        let auto_size_behavior = |position: ItemPosition, has_auto_inset: bool| {
            if has_auto_inset {
                return AutoSizeBehavior::kFitContent;
            }
            if position == ItemPosition::kStretch {
                return AutoSizeBehavior::kStretchExplicit;
            }
            if node_info.node.IsTable() || node_info.node.IsReplaced() {
                return AutoSizeBehavior::kFitContent;
            }
            if position == ItemPosition::kNormal {
                AutoSizeBehavior::kStretchImplicit
            } else {
                AutoSizeBehavior::kFitContent
            }
        };
        let inline_auto_size_behavior = auto_size_behavior(
            alignment.inline_alignment.GetPosition(),
            imcb.has_auto_inline_inset,
        );
        let block_auto_size_behavior = auto_size_behavior(
            alignment.block_alignment.GetPosition(),
            imcb.has_auto_block_inset,
        );

        let mut replaced_size = None;
        if node_info.node.IsReplaced() {
            let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
                candidate_style.GetWritingMode(),
                candidate_style.GetWritingDirection(),
                true,
                false,
                false,
            );
            builder.SetAvailableSize(imcb.Size());
            builder.SetPercentageResolutionSize(space.PercentageResolutionSize());
            builder.SetInlineAutoBehavior(inline_auto_size_behavior);
            builder.SetBlockAutoBehavior(block_auto_size_behavior);
            replaced_size = Some(ComputeReplacedSize(
                &node_info.node,
                &builder.ToConstraintSpace(),
                &border_padding,
                ReplacedSizeMode::kNormal,
            ));
        }

        let anchor_center_position =
            ComputeAnchorCenterPosition(candidate_style, &alignment, space.AvailableSize());
        let overflow_limit_insets = {
            let overflow_limit_rect = if has_default_anchor {
                container_info.scroll_limit_rect.unwrap_or(base_rect)
            } else {
                base_rect
            };
            let mut insets = BoxStrut::from_rects(&container_rect, &overflow_limit_rect);
            if container_info.scroll_direction.inline_start {
                insets.inline_start = LayoutUnit::Min();
            }
            if container_info.scroll_direction.inline_end {
                insets.inline_end = LayoutUnit::Min();
            }
            if container_info.scroll_direction.block_start {
                insets.block_start = LayoutUnit::Min();
            }
            if container_info.scroll_direction.block_end {
                insets.block_end = LayoutUnit::Min();
            }
            insets
                .ConvertToPhysical(node_info.default_writing_direction)
                .ConvertToLogical(candidate_writing_direction)
        };

        let mut node_dimensions = LogicalOofDimensions::default();
        let inline_size_depends_on_min_max_sizes = ComputeOofInlineDimensions(
            &node_info.node,
            node_info.break_token.Get(),
            candidate_style,
            &space,
            &imcb,
            &anchor_center_position,
            &alignment,
            &border_padding,
            replaced_size,
            &overflow_limit_insets,
            inline_auto_size_behavior,
            block_auto_size_behavior,
            container_writing_direction,
            &mut node_dimensions,
        );
        let initial_layout_result = ComputeOofBlockDimensions(
            &node_info.node,
            node_info.break_token.Get(),
            candidate_style,
            &space,
            &imcb,
            &anchor_center_position,
            &alignment,
            &border_padding,
            replaced_size,
            &overflow_limit_insets,
            block_auto_size_behavior,
            container_writing_direction,
            &mut node_dimensions,
        );
        let mut offset_info = OffsetInfo {
            insets_for_get_computed_style: BoxStrut::default(),
            offset: LogicalOffset::default(),
            initial_layout_result: Member::from_ptr(initial_layout_result as *mut LayoutResult),
            container_content_size: LogicalSize::default(),
            imcb_block_size: LayoutUnit::default(),
            block_auto_size_behavior,
            node_dimensions,
            original_offset: LogicalOffset::default(),
            non_overflowing_scroll_ranges: HeapVector::default(),
            imcb_size_for_try_order: None,
            inline_size_depends_on_min_max_sizes,
            needs_scroll_adjustment_in_x: false,
            needs_scroll_adjustment_in_y: false,
            overflows_containing_block: false,
            display_locks_affected_by_anchors: Member::default(),
        };

        if try_fit_available_space {
            let has_non_auto_inset = PhysicalBoxSides::new(
                candidate_style.IsTopInsetNonAuto(),
                candidate_style.IsRightInsetNonAuto(),
                candidate_style.IsBottomInsetNonAuto(),
                candidate_style.IsLeftInsetNonAuto(),
            )
            .ToLogical(candidate_writing_direction);
            let imcb_for_position_fallback = ComputeIMCBForPositionFallback(
                &space.AvailableSize(),
                &alignment,
                &insets,
                &static_position,
                candidate_style,
                container_writing_direction,
                candidate_writing_direction,
            );
            offset_info.imcb_size_for_try_order = Some(ToPhysicalSize(
                imcb_for_position_fallback.Size(),
                candidate_writing_direction.GetWritingMode(),
            ));
            let mut scroll_range = LogicalScrollRange::default();
            let mut overflows_imcb = CalculateNonOverflowingRangeInOneAxis(
                offset_info.node_dimensions.MarginBoxInlineStart(),
                offset_info.node_dimensions.MarginBoxInlineEnd(),
                imcb_for_position_fallback.inline_start,
                imcb_for_position_fallback.InlineEndOffset(),
                has_non_auto_inset.inline_start,
                has_non_auto_inset.inline_end,
                &mut scroll_range.inline_min,
                &mut scroll_range.inline_max,
            );
            overflows_imcb |= CalculateNonOverflowingRangeInOneAxis(
                offset_info.node_dimensions.MarginBoxBlockStart(),
                offset_info.node_dimensions.MarginBoxBlockEnd(),
                imcb_for_position_fallback.block_start,
                imcb_for_position_fallback.BlockEndOffset(),
                has_non_auto_inset.block_start,
                has_non_auto_inset.block_end,
                &mut scroll_range.block_min,
                &mut scroll_range.block_max,
            );
            out_non_overflowing_range.containing_block_range =
                scroll_range.ToPhysical(candidate_writing_direction);
            out_non_overflowing_range.anchor_element = Member::from_ptr(GetPositionAnchorElement(
                &node_info.node,
                candidate_style,
                anchor_evaluator.GetAnchorMap(),
            )
                as *mut Element);
            if overflows_imcb {
                return None;
            }
        }

        offset_info.container_content_size = ToLogicalSize(
            container_physical_content_size,
            candidate_writing_direction.GetWritingMode(),
        );
        offset_info.imcb_block_size = imcb.BlockSize();
        offset_info.block_auto_size_behavior = block_auto_size_behavior;

        let break_token = node_info.break_token.Get();
        if !break_token.is_null() {
            debug_assert!(RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
            offset_info.offset.inline_offset = unsafe { &*break_token }.OofInlineStartOffset();
            if unsafe { &*break_token }.IsRepeated() {
                offset_info.offset.block_offset = unsafe { &*break_token }.OofBlockStartOffset();
            } else {
                let mut monolithic_overflow = LayoutUnit::default();
                if !container_break_token.is_null() {
                    monolithic_overflow = unsafe { &*container_break_token }.MonolithicOverflow();
                }
                offset_info.offset.block_offset = std::cmp::max(
                    monolithic_overflow,
                    unsafe { &*break_token }.OofBlockStartOffset(),
                );
            }
        } else {
            let inset = offset_info
                .node_dimensions
                .inset
                .ConvertToPhysical(candidate_writing_direction)
                .ConvertToLogical(node_info.default_writing_direction);
            offset_info.offset = container_rect.offset;
            offset_info.offset.inline_offset += inset.inline_start;
            offset_info.offset.block_offset += inset.block_start;
            offset_info.offset.block_offset -= previously_consumed_block_size;

            let used_insets =
                offset_info.node_dimensions.inset - offset_info.node_dimensions.margins;
            let insets_to_store = BoxStrut::new(
                insets.inline_start.unwrap_or(used_insets.inline_start),
                insets.inline_end.unwrap_or(used_insets.inline_end),
                insets.block_start.unwrap_or(used_insets.block_start),
                insets.block_end.unwrap_or(used_insets.block_end),
            );
            offset_info.insets_for_get_computed_style = insets_to_store
                .ConvertToPhysical(candidate_writing_direction)
                .ConvertToLogical(node_info.default_writing_direction);
        }

        let mut anchor_center_x = anchor_center_position.inline_offset.is_some();
        let mut anchor_center_y = anchor_center_position.block_offset.is_some();
        if !candidate_writing_direction.IsHorizontal() {
            std::mem::swap(&mut anchor_center_x, &mut anchor_center_y);
        }
        offset_info.needs_scroll_adjustment_in_x =
            anchor_center_x || anchor_evaluator.NeedsScrollAdjustmentInX();
        offset_info.needs_scroll_adjustment_in_y =
            anchor_center_y || anchor_evaluator.NeedsScrollAdjustmentInY();
        Some(offset_info)
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:372-375
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:2454-2515
    fn Layout(
        &mut self,
        oof_node_to_layout: &NodeToLayout,
        fragmentainer_constraint_space: *const ConstraintSpace,
        is_last_fragmentainer_so_far: bool,
    ) -> *const LayoutResult {
        let offset_info = &oof_node_to_layout.offset_info;
        let mut layout_result = offset_info.initial_layout_result.Get() as *const LayoutResult;
        if !fragmentainer_constraint_space.is_null()
            || (RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
                && self.GetConstraintSpace().HasBlockFragmentation())
            || self.GetConstraintSpace().IsInsideRepeatableContent()
        {
            layout_result = std::ptr::null();
        }
        if layout_result.is_null() {
            layout_result = self.GenerateFragment(
                oof_node_to_layout,
                fragmentainer_constraint_space,
                is_last_fragmentainer_so_far,
            );
        }
        let result = unsafe { &*layout_result };
        debug_assert_eq!(result.Status(), EStatus::kSuccess);
        result
            .GetMutableForOutOfFlow()
            .SetOutOfFlowInsetsForGetComputedStyle(&offset_info.insets_for_get_computed_style);
        result
            .GetMutableForOutOfFlow()
            .SetOutOfFlowPositionedOffset(&offset_info.offset);
        result.GetMutableForOutOfFlow().SetNeedsScrollAdjustment(
            offset_info.needs_scroll_adjustment_in_x,
            offset_info.needs_scroll_adjustment_in_y,
        );
        result
            .GetMutableForOutOfFlow()
            .SetNonOverflowingScrollRanges(&offset_info.non_overflowing_scroll_ranges);
        result
            .GetMutableForOutOfFlow()
            .SetDisplayLocksAffectedByAnchors(offset_info.display_locks_affected_by_anchors.Get());

        let fragment = result.GetPhysicalFragment();
        debug_assert!(fragment.IsBox());
        let box_fragment = unsafe { &*(fragment as *const _ as *const PhysicalBoxFragment) };
        let break_token = box_fragment.GetBreakToken();
        if !break_token.is_null() && RuntimeEnabledFeatures::FragmentedOofInCbEnabled() {
            let mutator = unsafe { &*break_token }.GetMutableForOofFragmentation();
            mutator.SetInlineStartOffset(offset_info.offset.inline_offset);
            if unsafe { &*break_token }.IsRepeated() {
                mutator.SetBlockStartOffset(offset_info.offset.block_offset);
            }
        }
        layout_result
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:379-382
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:2559-2673
    fn GenerateFragment(
        &mut self,
        oof_node_to_layout: &NodeToLayout,
        fragmentainer_constraint_space: *const ConstraintSpace,
        is_last_fragmentainer_so_far: bool,
    ) -> *const LayoutResult {
        let node_info = &oof_node_to_layout.node_info;
        let offset_info = &oof_node_to_layout.offset_info;
        let break_token = node_info.break_token.Get();
        let node = &node_info.node;
        let style = node.Style();
        let is_replaced = node.IsReplaced();
        let block_offset = offset_info.offset.block_offset;
        let force_orthogonal_writing_mode_root = !IsParallelWritingMode(
            self.GetConstraintSpace().GetWritingMode(),
            style.GetWritingMode(),
        );

        let mut builder = ConstraintSpaceBuilder::new_with_parent_writing_mode(
            self.GetConstraintSpace(),
            style.GetWritingMode(),
            style.GetWritingDirection(),
            true,
            true,
            force_orthogonal_writing_mode_root,
        );
        builder.SetAvailableSize(LogicalSize::new(
            offset_info.node_dimensions.size.inline_size,
            if is_replaced {
                offset_info.node_dimensions.size.block_size
            } else {
                offset_info.imcb_block_size
            },
        ));
        builder.SetPercentageResolutionSize(offset_info.container_content_size);
        builder.SetIsFixedInlineSize(true);
        builder.SetIsHiddenForPaint(node_info.base_container_info.is_hidden_for_paint);
        if is_replaced {
            builder.SetIsFixedBlockSize(true);
        } else {
            builder.SetBlockAutoBehavior(offset_info.block_auto_size_behavior);
        }

        let is_in_block_fragmentation = (RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
            && self.GetConstraintSpace().HasBlockFragmentation())
            || !fragmentainer_constraint_space.is_null();
        let mut is_repeatable = false;
        if is_in_block_fragmentation {
            if unsafe { &*self.container_builder_ }
                .Node()
                .IsPaginatedRoot()
                && style.GetPosition() == EPosition::kFixed
                && oof_node_to_layout.containing_block_fragment.Get().is_null()
            {
                builder.SetShouldRepeat(true);
                builder.SetIsInsideRepeatableContent(true);
                builder.DisableMonolithicOverflowPropagation();
                is_repeatable = true;
            } else {
                if RuntimeEnabledFeatures::FragmentedOofInCbEnabled() {
                    debug_assert!(fragmentainer_constraint_space.is_null());
                    SetupSpaceBuilderForFragmentationFromBuilder(
                        unsafe { &*self.container_builder_ },
                        node,
                        block_offset,
                        &mut builder,
                    );
                } else {
                    debug_assert!(!fragmentainer_constraint_space.is_null());
                    let fragmentainer_space = unsafe { &*fragmentainer_constraint_space };
                    SetupSpaceBuilderForFragmentationFromSpace(
                        fragmentainer_space,
                        node,
                        fragmentainer_space.FragmentainerOffset() + block_offset,
                        fragmentainer_space.FragmentainerBlockSize(),
                        node_info.requires_content_before_breaking,
                        &mut builder,
                    );
                }
                if node_info
                    .containing_block
                    .IsFragmentedInsideClippedContainer()
                {
                    if is_last_fragmentainer_so_far {
                        builder.DisableFurtherFragmentation();
                    }
                    builder.DisableMonolithicOverflowPropagation();
                }
            }
        } else if self.GetConstraintSpace().IsInitialColumnBalancingPass() {
            SetupSpaceBuilderForFragmentationFromSpace(
                self.GetConstraintSpace(),
                node,
                self.GetConstraintSpace().FragmentainerOffset() + block_offset,
                self.GetConstraintSpace().FragmentainerBlockSize(),
                false,
                &mut builder,
            );
        }
        let space = builder.ToConstraintSpace();
        if is_repeatable {
            node.LayoutRepeatableRoot(&space, break_token)
        } else {
            node.Layout(&space, break_token, std::ptr::null(), std::ptr::null())
        }
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:349-352
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:1753-1846
    fn LayoutOOFNode(
        &mut self,
        oof_node_to_layout: &mut NodeToLayout,
        fragmentainer_constraint_space: *const ConstraintSpace,
        is_last_fragmentainer_so_far: bool,
    ) -> *const LayoutResult {
        let box_ = oof_node_to_layout.node_info.node.GetLayoutBox();
        let _past_display_lock_elements = if box_.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*box_ }.DisplayLocksAffectedByAnchors()
        };
        let node_info = &oof_node_to_layout.node_info;
        let mut scrollbars_before = ComputeScrollbarsForNonAnonymous(&node_info.node);
        let mut layout_result = self.Layout(
            oof_node_to_layout,
            fragmentainer_constraint_space,
            is_last_fragmentainer_so_far,
        );

        if unsafe { &*box_ }.IntrinsicLogicalWidthsDirty()
            && oof_node_to_layout
                .offset_info
                .inline_size_depends_on_min_max_sizes
        {
            let writing_mode_direction = node_info.node.Style().GetWritingDirection();
            let mut freeze_horizontal = false;
            let mut freeze_vertical = false;
            let mut scrollbars_after = ComputeScrollbarsForNonAnonymous(&node_info.node);
            let mut ignore_first_inline_freeze = scrollbars_after.InlineSum()
                != LayoutUnit::default()
                && scrollbars_after.BlockSum() != LayoutUnit::default();
            if self.GetConstraintSpace().CacheSlot() == LayoutResultCacheSlot::kMeasure {
                freeze_horizontal = true;
                freeze_vertical = true;
                ignore_first_inline_freeze = false;
            }
            loop {
                AddScrollbarFreeze(
                    &scrollbars_before,
                    &scrollbars_after,
                    writing_mode_direction,
                    &mut freeze_horizontal,
                    &mut freeze_vertical,
                );
                if ignore_first_inline_freeze {
                    ignore_first_inline_freeze = false;
                    if writing_mode_direction.IsHorizontal() {
                        freeze_horizontal = false;
                    } else {
                        freeze_vertical = false;
                    }
                }
                scrollbars_before = scrollbars_after;
                let _freezer = FreezeScrollbarsRootScope::from_box(
                    unsafe { &*box_ },
                    freeze_horizontal,
                    freeze_vertical,
                );
                if !IsBreakInside(node_info.break_token.Get()) {
                    let is_inside_fragmentation_context =
                        (RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
                            && self.GetConstraintSpace().HasBlockFragmentation())
                            || !fragmentainer_constraint_space.is_null();
                    oof_node_to_layout.offset_info =
                        self.CalculateOffset(node_info, is_inside_fragmentation_context);
                }
                layout_result = self.Layout(
                    oof_node_to_layout,
                    fragmentainer_constraint_space,
                    is_last_fragmentainer_so_far,
                );
                scrollbars_after = ComputeScrollbarsForNonAnonymous(&node_info.node);
                debug_assert!(
                    !freeze_horizontal || !freeze_vertical || scrollbars_after == scrollbars_before
                );
                if scrollbars_after == scrollbars_before {
                    break;
                }
            }
        }
        layout_result
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:405-405
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:2936-2961
    fn GetFragmentainerConstraintSpace(&self, index: usize) -> ConstraintSpace {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        debug_assert!(index < self.ChildCount());
        let fragment = self.GetChildFragment(index);
        debug_assert!(fragment.IsFragmentainerBox());
        let container_builder = unsafe { &*self.container_builder_ };
        let container_writing_mode = container_builder.Style().GetWritingMode();
        let fragmentainer_size = ToLogicalSize(fragment.Size(), container_writing_mode);
        let percentage_resolution_size = LogicalSize::new(
            fragmentainer_size.inline_size,
            container_builder.ChildAvailableSize().block_size,
        );
        let min_break_appeal = BreakAppeal::kBreakAppealLastResort;
        CreateConstraintSpaceForFragmentainer(
            self.GetConstraintSpace(),
            self.GetFragmentainerType(),
            fragmentainer_size,
            percentage_resolution_size,
            false,
            min_break_appeal,
            std::ptr::null(),
        )
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:406-414
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:2965-3032
    fn ComputeStartFragmentIndexAndRelativeOffset(
        &self,
        default_writing_mode: WritingMode,
        block_estimate: LayoutUnit,
        clipped_container_block_offset: Option<LayoutUnit>,
        start_index: &mut usize,
        offset: &mut LogicalOffset,
    ) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        let mut used_block_size = LayoutUnit::default();
        let mut current_max_block_size = LayoutUnit::default();
        let mut fragmentainer_block_size = LayoutUnit::default();
        let mut target_block_offset = offset.block_offset;
        if let Some(clipped_container_block_offset) = clipped_container_block_offset {
            if unsafe { &*self.container_builder_ }
                .Node()
                .IsPaginatedRoot()
            {
                target_block_offset =
                    std::cmp::max(target_block_offset, clipped_container_block_offset);
            }
        }
        let mut child_index = 0;
        while child_index < self.ChildCount() {
            let child_fragment = self.GetChildFragment(child_index);
            if child_fragment.IsFragmentainerBox() {
                fragmentainer_block_size =
                    ToLogicalSize(child_fragment.Size(), default_writing_mode).block_size;
                fragmentainer_block_size =
                    ClampedToValidFragmentainerCapacity(fragmentainer_block_size);
                current_max_block_size += fragmentainer_block_size;
                if target_block_offset < current_max_block_size
                    || (target_block_offset == current_max_block_size
                        && block_estimate == LayoutUnit::default())
                {
                    *start_index = child_index;
                    offset.block_offset -= used_block_size;
                    return;
                }
                used_block_size = current_max_block_size;
            }
            child_index += 1;
        }
        let remaining_block_offset = offset.block_offset - used_block_size;
        let additional_fragment_count = (remaining_block_offset.ToFloat()
            / fragmentainer_block_size.ToFloat())
        .floor() as usize;
        *start_index = child_index + additional_fragment_count;
        offset.block_offset =
            remaining_block_offset - fragmentainer_block_size * additional_fragment_count as i32;
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:565-581
    fn PropagateOOFsFromPageAreas(&mut self) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        debug_assert!(unsafe { &*self.container_builder_ }.IsPaginatedRoot());
        let mut offset_adjustment = LogicalOffset::default();
        for i in 0..self.ChildCount() {
            let fragmentainer = self.GetChildFragment(i) as *const PhysicalBoxFragment;
            let fragmentainer = unsafe { &*fragmentainer };
            if fragmentainer.NeedsOOFPositionedInfoPropagation() {
                unsafe { &mut *self.container_builder_ }.PropagateOOFPositionedInfo(
                    fragmentainer,
                    LogicalOffset::default(),
                    LogicalOffset::default(),
                    offset_adjustment,
                    std::ptr::null(),
                    LayoutUnit::default(),
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null(),
                    LogicalOffset::default(),
                );
            }
            let break_token = fragmentainer.GetBreakToken();
            if !break_token.is_null() {
                offset_adjustment.block_offset = unsafe { &*break_token }.ConsumedBlockSize();
            }
        }
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:583-656
    fn HandleFragmentation(&mut self) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        if DisableLayoutSideEffectsScope::IsDisabled() {
            return;
        }
        let builder_ptr = self.container_builder_;
        if self.column_balancing_info_.is_null()
            && (!unsafe { &*builder_ptr }.IsBlockFragmentationContextRoot()
                || self.should_add_outer_fragmentainer_children_)
        {
            return;
        }
        if unsafe { &*builder_ptr }.Node().IsPaginatedRoot() {
            let mut candidates = HeapVector::<LogicalOofPositionedNode>::default();
            let _scope = unsafe { ClearCollectionScope::new(&mut candidates as *mut _) };
            unsafe { &mut *builder_ptr }.SwapOutOfFlowPositionedCandidates(&mut candidates);
            for candidate in &candidates {
                unsafe { &mut *builder_ptr }.AddOutOfFlowFragmentainerDescendantFromOof(candidate);
            }
        }
        debug_assert!(
            self.child_fragment_storage_.is_null()
                || !unsafe { &*self.child_fragment_storage_ }.is_empty()
        );
        debug_assert!(
            self.column_balancing_info_.is_null()
                || !unsafe { &*self.column_balancing_info_ }
                    .out_of_flow_fragmentainer_descendants
                    .is_empty()
        );

        loop {
            let should_continue = if self.column_balancing_info_.is_null() {
                unsafe { &*builder_ptr }.HasOutOfFlowFragmentainerDescendants()
                    || unsafe { &*builder_ptr }.HasMulticolsWithPendingOOFs()
            } else {
                unsafe { &*self.column_balancing_info_ }.HasOutOfFlowFragmentainerDescendants()
            };
            if !should_continue {
                break;
            }
            let mut fragmentainer_descendants =
                HeapVector::<LogicalOofNodeForFragmentation>::default();
            let _scope =
                unsafe { ClearCollectionScope::new(&mut fragmentainer_descendants as *mut _) };
            if !self.column_balancing_info_.is_null() {
                unsafe { &mut *self.column_balancing_info_ }
                    .SwapOutOfFlowFragmentainerDescendants(&mut fragmentainer_descendants);
                debug_assert!(!fragmentainer_descendants.is_empty());
            } else {
                self.HandleMulticolsWithPendingOOFs(builder_ptr);
                if unsafe { &*builder_ptr }.HasOutOfFlowFragmentainerDescendants() {
                    unsafe { &mut *builder_ptr }
                        .SwapOutOfFlowFragmentainerDescendants(&mut fragmentainer_descendants);
                    debug_assert!(!fragmentainer_descendants.is_empty());
                }
            }
            if !fragmentainer_descendants.is_empty() {
                let fragmentainer_progression = GetFragmentainerProgression(
                    unsafe { &*builder_ptr },
                    self.GetFragmentainerType(),
                );
                self.LayoutFragmentainerDescendants(
                    &mut fragmentainer_descendants,
                    fragmentainer_progression,
                    false,
                    std::ptr::null_mut(),
                );
            }
        }
        if self.column_balancing_info_.is_null() {
            for descendant in &self.delayed_descendants_ {
                unsafe { &mut *builder_ptr }.AddOutOfFlowFragmentainerDescendant(descendant);
            }
        }
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:1106-1135
    fn HandleMulticolsWithPendingOOFs(&mut self, container_builder: *mut BoxFragmentBuilder) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        if !unsafe { &*container_builder }.HasMulticolsWithPendingOOFs() {
            return;
        }

        let mut multicols_handled = HeapHashMap::<
            Member<LayoutBox>,
            Member<MulticolWithPendingOofs<LogicalOffset>>,
        >::default();
        let mut multicols_with_pending_oofs = HeapHashMap::<
            Member<LayoutBox>,
            Member<MulticolWithPendingOofs<LogicalOffset>>,
        >::default();
        unsafe { &mut *container_builder }
            .SwapMulticolsWithPendingOOFs(&mut multicols_with_pending_oofs);
        debug_assert!(!multicols_with_pending_oofs.is_empty());

        while !multicols_with_pending_oofs.is_empty() {
            for (key, value) in multicols_with_pending_oofs.iter() {
                debug_assert!(!multicols_handled.Contains(key));
                self.LayoutOOFsInMulticol(&BlockNode::new(key.Get()), unsafe { &*value.Get() });
                multicols_handled.insert(*key, *value);
            }
            multicols_with_pending_oofs.clear();

            // Processing an outer multicol can discover additional inner multicols.
            let mut new_multicols = HeapHashMap::<
                Member<LayoutBox>,
                Member<MulticolWithPendingOofs<LogicalOffset>>,
            >::default();
            unsafe { &mut *container_builder }.SwapMulticolsWithPendingOOFs(&mut new_multicols);
            for (key, value) in new_multicols.iter() {
                if !multicols_handled.Contains(key) {
                    multicols_with_pending_oofs.insert(*key, *value);
                }
            }
        }
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:1137-1370
    fn LayoutOOFsInMulticol(
        &mut self,
        multicol: &BlockNode,
        multicol_info: &MulticolWithPendingOofs<LogicalOffset>,
    ) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        let mut oof_nodes_to_layout = HeapVector::<LogicalOofNodeForFragmentation>::default();
        let _oof_nodes_scope =
            unsafe { ClearCollectionScope::new(&mut oof_nodes_to_layout as *mut _) };
        let mut multicol_children = HeapVector::<MulticolChildInfo>::default();
        let _multicol_scope =
            unsafe { ClearCollectionScope::new(&mut multicol_children as *mut _) };

        let mut current_column_break_token: *const BlockBreakToken = std::ptr::null();
        let mut previous_multicol_break_token: *const BlockBreakToken = std::ptr::null();
        let mut column_inline_progression = kIndefiniteSize;
        let mut multicol_offset = multicol_info.multicol_offset;

        let limited_multicol_constraint_space = CreateConstraintSpaceForMulticol(multicol);
        let limited_fragment_geometry = CalculateInitialFragmentGeometry(
            &limited_multicol_constraint_space,
            multicol,
            std::ptr::null(),
            false,
        );
        let mut limited_multicol_container_builder = CreateContainerBuilderForMulticol(
            multicol,
            &limited_multicol_constraint_space,
            &limited_fragment_geometry,
        );
        limited_multicol_container_builder.SetFragmentsTotalBlockSize(LayoutUnit::default());

        let writing_direction = multicol.Style().GetWritingDirection();
        let mut last_fragment_with_fragmentainer: *const PhysicalBoxFragment = std::ptr::null();
        for multicol_box_fragment in unsafe { &*multicol.GetLayoutBox() }.PhysicalFragments() {
            let style = multicol_box_fragment.Style();
            let converter =
                WritingModeConverter::new(writing_direction, multicol_box_fragment.Size());
            let mut current_column_index: Option<usize> = None;
            if column_inline_progression == kIndefiniteSize {
                let border_padding = multicol_box_fragment
                    .Borders()
                    .ConvertToLogical(writing_direction)
                    + multicol_box_fragment
                        .Padding()
                        .ConvertToLogical(writing_direction);
                let available_inline_size = ToLogicalSize(
                    multicol_box_fragment.Size(),
                    writing_direction.GetWritingMode(),
                )
                .inline_size
                    - border_padding.InlineSum();
                column_inline_progression = ColumnInlineProgression(style, available_inline_size);
            }

            for child in multicol_box_fragment
                .GetMutableChildrenForOutOfFlow()
                .Children()
            {
                let fragment = unsafe { &*child.get() };
                let offset = converter.ToLogicalOffset(child.Offset(), fragment.Size());
                if fragment.IsFragmentainerBox() {
                    current_column_break_token = To::<BlockBreakToken>(fragment.GetBreakToken());
                    current_column_index = Some(multicol_children.len());
                    last_fragment_with_fragmentainer = multicol_box_fragment;
                }
                limited_multicol_container_builder.AddChild(
                    fragment,
                    offset,
                    None,
                    false,
                    None,
                    std::ptr::null(),
                );
                multicol_children.push(MulticolChildInfo::default());
            }

            let break_token = multicol_box_fragment.GetBreakToken();
            if let Some(current_column_index) = current_column_index {
                if !break_token.is_null() && !unsafe { &*break_token }.ChildBreakTokens().is_empty()
                {
                    let children = unsafe { &*break_token }.ChildBreakTokens();
                    let child_token = To::<BlockBreakToken>(children[children.len() - 1].Get());
                    if child_token as *const BlockBreakToken == current_column_break_token {
                        multicol_children[current_column_index].parent_break_token =
                            Member::from_ptr(break_token as *mut BlockBreakToken);
                    }
                }
            }

            let mut oof_fragmentainer_descendants =
                HeapVector::<LogicalOofNodeForFragmentation>::default();
            limited_multicol_container_builder
                .SwapOutOfFlowFragmentainerDescendants(&mut oof_fragmentainer_descendants);
            for descendant in &oof_fragmentainer_descendants {
                if oof_nodes_to_layout.is_empty()
                    && !multicol_info.fixedpos_containing_block.Fragment().is_null()
                    && !previous_multicol_break_token.is_null()
                {
                    multicol_offset.block_offset -=
                        unsafe { &*previous_multicol_break_token }.ConsumedBlockSize();
                }
                if descendant.containing_block.Fragment().is_null()
                    || descendant.containing_block.IsInsideColumnSpanner()
                {
                    continue;
                }
                oof_nodes_to_layout.push(descendant.clone());
            }
            previous_multicol_break_token = break_token;
        }
        if oof_nodes_to_layout.is_empty() {
            return;
        }
        debug_assert!(!limited_multicol_container_builder.HasOutOfFlowFragmentainerDescendants());
        limited_multicol_container_builder.ClearOutOfFlowPositionedCandidates();

        let old_fragment_count = limited_multicol_container_builder.Children().len();
        let fragmentainer_progression =
            LogicalOffset::new(column_inline_progression, LayoutUnit::default());
        let mut inner_part = OutOfFlowLayoutPart::new(&mut limited_multicol_container_builder);
        inner_part.LayoutFragmentainerDescendants(
            &mut oof_nodes_to_layout,
            fragmentainer_progression,
            !multicol_info.fixedpos_containing_block.Fragment().is_null(),
            &mut multicol_children,
        );
        let new_fragment_count = limited_multicol_container_builder.Children().len();

        if old_fragment_count != new_fragment_count {
            debug_assert!(new_fragment_count > old_fragment_count);
            debug_assert!(!last_fragment_with_fragmentainer.is_null());
            let box_ =
                unsafe { &mut *(&*last_fragment_with_fragmentainer).MutableOwnerLayoutBox() };
            let fragment_count = box_.PhysicalFragmentCount();
            debug_assert!(fragment_count >= 1);
            let mut fragment_idx = fragment_count - 1;
            let old_result = loop {
                let old_result = box_.GetLayoutResult(fragment_idx);
                if unsafe { &*old_result }.GetPhysicalFragment() as *const PhysicalFragment
                    == last_fragment_with_fragmentainer as *const PhysicalFragment
                {
                    break unsafe { &*old_result };
                }
                debug_assert!(fragment_idx > 0);
                fragment_idx -= 1;
            };
            let existing_fragment =
                unsafe { &*To::<PhysicalBoxFragment>(old_result.GetPhysicalFragment()) };
            let converter = WritingModeConverter::new(
                existing_fragment.Style().GetWritingDirection(),
                existing_fragment.Size(),
            );
            let mut additional_column_block_size = LayoutUnit::default();
            let fragment_mutator = existing_fragment.GetMutableForOofFragmentation();
            for i in old_fragment_count..new_fragment_count {
                let child = &limited_multicol_container_builder.Children()[i];
                let child_fragment = unsafe { &*To::<PhysicalBoxFragment>(child.get()) };
                fragment_mutator.AddChildFragmentainer(child_fragment, child.offset);
                additional_column_block_size += converter
                    .ToLogicalSize(unsafe { &*child.fragment.Get() }.Size())
                    .block_size;
            }
            fragment_mutator.UpdateOverflow();
        }

        debug_assert!(!limited_multicol_container_builder.HasOutOfFlowPositionedDescendants());
        debug_assert!(!limited_multicol_container_builder.HasOutOfFlowFragmentainerDescendants());
        limited_multicol_container_builder.TransferOutOfFlowCandidates(
            unsafe { &mut *self.container_builder_ },
            multicol_offset,
            Some(multicol_info),
        );

        let mut multicols_with_pending_oofs = HeapHashMap::<
            Member<LayoutBox>,
            Member<MulticolWithPendingOofs<LogicalOffset>>,
        >::default();
        limited_multicol_container_builder
            .SwapMulticolsWithPendingOOFs(&mut multicols_with_pending_oofs);
        for (key, value) in multicols_with_pending_oofs.iter() {
            unsafe { &mut *self.container_builder_ }
                .AddMulticolWithPendingOOFs(&BlockNode::new(key.Get()), value.Get());
        }
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:2810-2934
    #[allow(clippy::too_many_arguments)]
    fn AddOOFToFragmentainer(
        &mut self,
        descendant: &mut NodeToLayout,
        fragmentainer_space: &ConstraintSpace,
        fragmentainer_offset: LogicalOffset,
        index: usize,
        is_last_fragmentainer_so_far: bool,
        has_actual_break_inside: &mut bool,
        algorithm: &mut SimplifiedOofLayoutAlgorithm,
        fragmented_descendants: &mut HeapVector<NodeToLayout>,
    ) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        let result = self.LayoutOOFNode(
            descendant,
            fragmentainer_space,
            is_last_fragmentainer_so_far,
        );
        let result = unsafe { &*result };
        debug_assert_eq!(result.Status(), EStatus::kSuccess);

        let oof_offset = result.OutOfFlowPositionedOffset();
        let relative_offset = descendant.node_info.base_container_info.relative_offset;
        let adjusted_offset = oof_offset + relative_offset;
        let offset_adjustment = fragmentainer_offset;
        result
            .GetMutableForOutOfFlow()
            .SetOutOfFlowPositionedOffset(&adjusted_offset);

        let mut additional_fixedpos_offset = LogicalOffset::default();
        if !descendant
            .node_info
            .fixedpos_containing_block
            .Fragment()
            .is_null()
        {
            additional_fixedpos_offset = (descendant.offset_info.original_offset
                - descendant.node_info.fixedpos_containing_block.Offset())
            .into();
            let token = descendant.node_info.break_token.Get();
            if !token.is_null() {
                additional_fixedpos_offset.block_offset += unsafe { &*token }.ConsumedBlockSize();
            }
        } else if self.outer_context_has_fixedpos_container_ {
            debug_assert!(!self.multicol_children_.is_null());
            let multicol_children = unsafe { &*self.multicol_children_ };
            for i in (0..std::cmp::min(index, multicol_children.len())).rev() {
                let token = multicol_children[i].parent_break_token.Get();
                if !token.is_null() {
                    additional_fixedpos_offset.block_offset +=
                        unsafe { &*token }.ConsumedBlockSize();
                    break;
                }
            }
        }

        let physical_fragment = To::<PhysicalBoxFragment>(result.GetPhysicalFragment());
        let physical_fragment = unsafe { &*physical_fragment };
        let break_token = physical_fragment.GetBreakToken();
        if !break_token.is_null() {
            let mut fragmented_descendant = descendant.clone();
            fragmented_descendant.node_info.break_token =
                Member::from_ptr(break_token as *mut BlockBreakToken);
            if !unsafe { &*break_token }.IsRepeated() {
                fragmented_descendant.offset_info.offset.block_offset =
                    unsafe { &*break_token }.MonolithicOverflow();
                *has_actual_break_inside = true;
            }
            fragmented_descendants.push(fragmented_descendant);
        }

        if !self.column_balancing_info_.is_null() {
            let space_shortage = CalculateSpaceShortage(
                fragmentainer_space,
                result,
                oof_offset.block_offset,
                fragmentainer_space.FragmentainerBlockSize(),
                None,
            );
            let balancing = unsafe { &mut *self.column_balancing_info_ };
            balancing.PropagateSpaceShortage(space_shortage);
            if !balancing.has_violating_break
                && space_shortage > LayoutUnit::default()
                && physical_fragment.GetBreakToken().is_null()
            {
                balancing.has_violating_break = true;
            }
            return;
        }

        let builder = unsafe { &mut *self.container_builder_ };
        builder.PropagateChildAnchors(
            physical_fragment,
            oof_offset + relative_offset + offset_adjustment,
        );
        builder.PropagateStickyDescendants(physical_fragment);
        let containing_block_adjustment =
            builder.BlockOffsetAdjustmentForFragmentainer(self.fragmentainer_consumed_block_size_);
        if result
            .GetPhysicalFragment()
            .NeedsOOFPositionedInfoPropagation()
        {
            builder.PropagateOOFPositionedInfo(
                result.GetPhysicalFragment(),
                oof_offset,
                relative_offset,
                offset_adjustment,
                std::ptr::null(),
                containing_block_adjustment,
                &descendant.node_info.containing_block,
                &descendant.node_info.fixedpos_containing_block,
                &descendant.node_info.fixedpos_inline_container,
                additional_fixedpos_offset,
            );
        }
        algorithm.AppendOutOfFlowResult(result);
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:2675-2808
    #[allow(clippy::too_many_arguments)]
    fn LayoutOOFsInFragmentainer(
        &mut self,
        pending_descendants: &mut HeapVector<NodeToLayout>,
        index: usize,
        fragmentainer_progression: LogicalOffset,
        has_oofs_in_later_fragmentainer: bool,
        monolithic_overflow: &mut LayoutUnit,
        has_actual_break_inside: &mut bool,
        fragmented_descendants: &mut HeapVector<NodeToLayout>,
    ) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        let num_children = self.ChildCount();
        let is_new_fragment = index >= num_children;
        let is_last_fragmentainer_so_far = index + 1 >= num_children;
        let mut descendants_continued = HeapVector::<NodeToLayout>::default();
        let _scope = unsafe { ClearCollectionScope::new(&mut descendants_continued as *mut _) };
        std::mem::swap(fragmented_descendants, &mut descendants_continued);

        if pending_descendants.is_empty()
            && descendants_continued.is_empty()
            && *monolithic_overflow <= LayoutUnit::default()
            && !is_new_fragment
            && !is_last_fragmentainer_so_far
        {
            return;
        }

        let mut last_fragmentainer_index = index;
        while last_fragmentainer_index >= num_children
            || !self
                .GetChildFragment(last_fragmentainer_index)
                .IsFragmentainerBox()
        {
            debug_assert!(num_children > 0);
            last_fragmentainer_index = last_fragmentainer_index
                .checked_sub(1)
                .expect("no preceding fragmentainer");
        }
        let mut fragmentainer_offset =
            self.FragmentationContextChildren()[last_fragmentainer_index].offset;
        let node = unsafe { &*self.container_builder_ }.Node().clone();
        if is_new_fragment {
            let previous_fragmentainer =
                self.GetChildFragment(last_fragmentainer_index) as *const PhysicalBoxFragment;
            let algorithms = LayoutPassScope::Algorithms();
            assert!(
                !algorithms.is_null(),
                "LayoutNG algorithm set was not installed"
            );
            let support = unsafe { &*algorithms }.fragmentainer_support;
            let new_fragmentainer = if node.IsPaginatedRoot() {
                let empty_page = support
                    .empty_page
                    .expect("paged fragmentainer support was not assembled");
                let mut needs_total_page_count = false;
                let page = empty_page(
                    &node,
                    self.GetConstraintSpace(),
                    index
                        .try_into()
                        .expect("fragmentainer index exceeds 32 bits"),
                    unsafe { &*previous_fragmentainer },
                    &mut needs_total_page_count,
                );
                self.needs_total_page_count_ |= needs_total_page_count;
                self.additional_pages_were_added_ = true;
                page
            } else {
                let empty_column = support
                    .empty_column
                    .expect("multicol fragmentainer support was not assembled");
                empty_column(&node, self.GetConstraintSpace(), unsafe {
                    &*previous_fragmentainer
                })
            };
            fragmentainer_offset += fragmentainer_progression;
            self.AddFragmentainer(unsafe { &*new_fragmentainer }, fragmentainer_offset);
            debug_assert_eq!(index + 1, self.ChildCount());
        }

        let space = self.GetFragmentainerConstraintSpace(index);
        let fragmentainer = self.GetChildFragment(index) as *const PhysicalBoxFragment;
        let fragment_geometry =
            CalculateInitialFragmentGeometry(&space, &node, std::ptr::null(), false);
        let mut params = LayoutAlgorithmParams::new(node, &fragment_geometry, &space);
        params.break_token = self.PreviousFragmentainerBreakToken(index);
        let mut algorithm = SimplifiedOofLayoutAlgorithm::new(&params, unsafe { &*fragmentainer });
        if has_oofs_in_later_fragmentainer {
            algorithm.SetHasSubsequentChildren();
        }
        for descendant in &mut descendants_continued {
            self.AddOOFToFragmentainer(
                descendant,
                &space,
                fragmentainer_offset,
                index,
                is_last_fragmentainer_so_far,
                has_actual_break_inside,
                &mut algorithm,
                fragmented_descendants,
            );
        }
        for descendant in pending_descendants {
            self.AddOOFToFragmentainer(
                descendant,
                &space,
                fragmentainer_offset,
                index,
                is_last_fragmentainer_so_far,
                has_actual_break_inside,
                &mut algorithm,
                fragmented_descendants,
            );
        }

        if !self.column_balancing_info_.is_null() {
            return;
        }
        let fragmentainer_result = unsafe { &*algorithm.Layout() };
        let new_fragmentainer =
            To::<PhysicalBoxFragment>(fragmentainer_result.GetPhysicalFragment());
        unsafe { &*fragmentainer }
            .GetMutableForOofFragmentation()
            .Merge(unsafe { &*new_fragmentainer });
        let break_token = unsafe { &*fragmentainer }.GetBreakToken();
        *monolithic_overflow = if break_token.is_null() {
            LayoutUnit::default()
        } else {
            unsafe { &*break_token }.MonolithicOverflow()
        };
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:1372-1595
    fn LayoutFragmentainerDescendants(
        &mut self,
        descendants: &mut HeapVector<LogicalOofNodeForFragmentation>,
        fragmentainer_progression: LogicalOffset,
        outer_context_has_fixedpos_container: bool,
        multicol_children: *mut HeapVector<MulticolChildInfo>,
    ) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        self.multicol_children_ = multicol_children;
        self.outer_context_has_fixedpos_container_ = outer_context_has_fixedpos_container;
        debug_assert!(
            !self.multicol_children_.is_null() || !self.outer_context_has_fixedpos_container_
        );

        let mut descendants_to_layout = HeapVector::<HeapVector<NodeToLayout>>::default();
        let _descendants_scope =
            unsafe { ClearCollectionScope::new(&mut descendants_to_layout as *mut _) };
        let mut repeated_fixedpos_descendants = HeapVector::<NodeToLayout>::default();
        let _repeated_scope =
            unsafe { ClearCollectionScope::new(&mut repeated_fixedpos_descendants as *mut _) };
        let mut previous_repeated_fixedpos_resume_idx: Option<usize> = None;

        while !descendants.is_empty() {
            self.ComputeInlineContainingBlocksForFragmentainer(descendants);
            let mut span_start = 0;
            loop {
                let mut has_new_descendants_span = false;
                debug_assert!(span_start < descendants.len());
                for i in span_start..descendants.len() {
                    let descendant = &descendants[i];
                    if self.GetFragmentainerType() == FragmentationType::kFragmentColumn {
                        let containing_fragment = descendant.containing_block.Fragment();
                        let containing_block =
                            To::<LayoutBox>(unsafe { &*containing_fragment }.GetLayoutObject());
                        debug_assert!(!containing_block.is_null());
                        if !unsafe { &*containing_block }
                            .PhysicalFragments()
                            .back()
                            .GetBreakToken()
                            .is_null()
                        {
                            self.delayed_descendants_.push(descendant.clone());
                            continue;
                        }
                    }

                    if descendant.Node().MayContainAnchor() {
                        span_start = i + 1;
                        has_new_descendants_span = span_start < descendants.len();
                    }
                    let node_info = self.SetupNodeInfo(descendant);
                    let offset_info = self.CalculateOffset(&node_info, true);
                    let mut node_to_layout = NodeToLayout {
                        node_info,
                        offset_info,
                        containing_block_fragment: Member::from_ptr(
                            descendant.containing_block.Fragment() as *mut PhysicalFragment,
                        ),
                    };
                    node_to_layout.offset_info.original_offset = node_to_layout.offset_info.offset;

                    let mut start_index = 0;
                    self.ComputeStartFragmentIndexAndRelativeOffset(
                        node_to_layout
                            .node_info
                            .default_writing_direction
                            .GetWritingMode(),
                        node_to_layout.offset_info.node_dimensions.size.block_size,
                        node_to_layout
                            .node_info
                            .containing_block
                            .ClippedContainerBlockOffset(),
                        &mut start_index,
                        &mut node_to_layout.offset_info.offset,
                    );
                    if start_index >= descendants_to_layout.len() {
                        descendants_to_layout.resize_with(start_index + 1, HeapVector::default);
                    }
                    descendants_to_layout[start_index].push(node_to_layout);
                    if has_new_descendants_span {
                        break;
                    }
                }

                let mut fragmented_descendants = HeapVector::<NodeToLayout>::default();
                let _fragmented_scope =
                    unsafe { ClearCollectionScope::new(&mut fragmented_descendants as *mut _) };
                self.fragmentainer_consumed_block_size_ = LayoutUnit::default();
                let mut monolithic_overflow = LayoutUnit::default();
                let mut last_fragmentainer_has_break_inside = false;

                let mut index = 0;
                while index < descendants_to_layout.len() {
                    let mut fragment = if index < self.ChildCount() {
                        self.GetChildFragment(index) as *const PhysicalBoxFragment
                    } else {
                        if !self.column_balancing_info_.is_null() {
                            unsafe { &mut *self.column_balancing_info_ }.num_new_columns += 1;
                        }
                        std::ptr::null()
                    };
                    if fragment.is_null() || unsafe { &*fragment }.IsFragmentainerBox() {
                        if !repeated_fixedpos_descendants.is_empty()
                            && previous_repeated_fixedpos_resume_idx == Some(index)
                        {
                            fragmented_descendants.InsertVector(0, &repeated_fixedpos_descendants);
                            repeated_fixedpos_descendants.clear();
                        }
                        let has_oofs_in_later_fragmentainer =
                            index + 1 < descendants_to_layout.len();
                        last_fragmentainer_has_break_inside = false;
                        self.LayoutOOFsInFragmentainer(
                            &mut descendants_to_layout[index],
                            index,
                            fragmentainer_progression,
                            has_oofs_in_later_fragmentainer,
                            &mut monolithic_overflow,
                            &mut last_fragmentainer_has_break_inside,
                            &mut fragmented_descendants,
                        );
                        if self.column_balancing_info_.is_null() {
                            fragment = self.GetChildFragment(index);
                            self.fragmentainer_consumed_block_size_ += ToLogicalSize(
                                unsafe { &*fragment }.Size(),
                                unsafe { &*self.container_builder_ }
                                    .Style()
                                    .GetWritingMode(),
                            )
                            .block_size;
                        }
                    }
                    if index == descendants_to_layout.len() - 1
                        && (last_fragmentainer_has_break_inside
                            || monolithic_overflow > LayoutUnit::default()
                            || (!fragmented_descendants.is_empty()
                                && index + 1 < self.ChildCount()))
                    {
                        descendants_to_layout.resize_with(index + 2, HeapVector::default);
                    }
                    index += 1;
                }

                if !fragmented_descendants.is_empty() {
                    debug_assert!(unsafe { &*self.container_builder_ }
                        .Node()
                        .IsPaginatedRoot());
                    debug_assert!(previous_repeated_fixedpos_resume_idx
                        .is_none_or(|resume_idx| resume_idx <= descendants_to_layout.len()));
                    previous_repeated_fixedpos_resume_idx = Some(descendants_to_layout.len());
                    repeated_fixedpos_descendants.AppendVector(&fragmented_descendants);
                }
                descendants_to_layout.Shrink(0);
                if !has_new_descendants_span {
                    break;
                }
            }

            descendants.Shrink(0);
            if self.column_balancing_info_.is_null() {
                unsafe { &mut *self.container_builder_ }
                    .SwapOutOfFlowFragmentainerDescendants(descendants);
            }
        }

        if unsafe { &*self.container_builder_ }
            .Node()
            .IsPaginatedRoot()
        {
            for node_to_layout in &repeated_fixedpos_descendants {
                let node = &node_to_layout.node_info.node;
                debug_assert_eq!(node.Style().GetPosition(), EPosition::kFixed);
                node.FinishRepeatableRoot();
            }
        } else {
            debug_assert!(repeated_fixedpos_descendants.is_empty());
        }
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:100-106
    pub fn SetChildFragmentStorage(
        &mut self,
        child_fragment_storage: &mut LogicalFragmentLinkVector,
    ) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        self.child_fragment_storage_ = child_fragment_storage;
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:113-122
    pub fn SetColumnBalancingInfo(
        &mut self,
        column_balancing_info: &mut ColumnBalancingInfo,
        child_fragment_storage: &mut LogicalFragmentLinkVector,
    ) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        self.column_balancing_info_ = column_balancing_info;
        self.SetChildFragmentStorage(child_fragment_storage);
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:132-132
    pub fn NeedsTotalPageCount(&self) -> bool {
        self.needs_total_page_count_
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:134-137
    pub fn AdditionalPagesWereAdded(&self) -> bool {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        self.additional_pages_were_added_
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:298-302
    fn GetFragmentainerType(&self) -> FragmentationType {
        if unsafe { &*self.container_builder_ }
            .Node()
            .IsPaginatedRoot()
        {
            FragmentationType::kFragmentPage
        } else {
            FragmentationType::kFragmentColumn
        }
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:303-305
    fn GetConstraintSpace(&self) -> &ConstraintSpace {
        unsafe { &*self.container_builder_ }.GetConstraintSpace()
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:2517-2556
    fn IsContainingBlockForCandidate(&self, candidate: &LogicalOofPositionedNode) -> bool {
        let builder = unsafe { &*self.container_builder_ };
        if builder.GetBoxType() == BoxType::kPageArea
            && RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
        {
            return true;
        }
        if builder.IsFragmentainerBoxType() {
            return false;
        }
        let break_token = candidate.GetBreakToken();
        if !break_token.is_null() && !unsafe { &*break_token }.IsForcedBreak() {
            debug_assert!(RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
            return true;
        }
        let position = candidate.Node().Style().GetPosition();
        let inline_container = candidate.InlineContainer();
        if !inline_container.is_null() {
            debug_assert!(
                unsafe { &*inline_container }.CanContainOutOfFlowPositionedElement(position)
            );
            return builder.GetLayoutObject()
                == unsafe { &*candidate.Node().GetLayoutBox() }.ContainingBlock()
                    as *const LayoutObject;
        }
        (self.is_absolute_container_ && position == EPosition::kAbsolute)
            || (self.is_fixed_container_ && position == EPosition::kFixed)
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:297-297
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:666-733
    fn GetContainingBlockInfo(
        &mut self,
        candidate: &LogicalOofPositionedNode,
    ) -> ContainingBlockInfo {
        let inline_container = candidate.InlineContainer();
        if !inline_container.is_null() {
            let key = Member::from_ptr(inline_container as *mut LayoutObject);
            return self
                .containing_blocks_map_
                .get(&key)
                .expect("inline containing block geometry was not computed")
                .clone();
        }
        if candidate.IsForFragmentation() {
            debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
            let descendant = unsafe {
                &*(candidate as *const LogicalOofPositionedNode
                    as *const LogicalOofNodeForFragmentation)
            };
            let fragment = descendant.containing_block.Fragment();
            if !fragment.is_null() {
                debug_assert!(
                    unsafe { &*self.container_builder_ }.IsBlockFragmentationContextRoot()
                );
                let fragment = unsafe { &*fragment };
                let containing_block = fragment.GetLayoutObject();
                debug_assert!(!containing_block.is_null());
                let key = Member::from_ptr(containing_block as *mut LayoutObject);
                if let Some(info) = self.containing_blocks_map_.get(&key) {
                    return info.clone();
                }
                let writing_direction = unsafe { &*containing_block }
                    .StyleRef()
                    .GetWritingDirection();
                let mut padding_box_rect = LogicalRect::default();
                padding_box_rect.size =
                    ToLogicalSize(fragment.Size(), writing_direction.GetWritingMode());
                let containing_box = To::<LayoutBox>(containing_block);
                padding_box_rect.size.block_size = BoxTotalBlockSize(unsafe { &*containing_box });
                let box_fragment = To::<PhysicalBoxFragment>(fragment);
                padding_box_rect.Contract(
                    &unsafe { &*box_fragment }
                        .Borders()
                        .ConvertToLogical(writing_direction),
                );
                padding_box_rect.offset += descendant.containing_block.Offset();
                let info = ContainingBlockInfo {
                    writing_direction,
                    is_scroll_container: fragment.IsScrollContainer(),
                    is_hidden_for_paint: fragment.IsHiddenForPaint(),
                    rect: padding_box_rect,
                    scroll_rect: None,
                    scroll_limit_rect: None,
                    scroll_direction: LogicalBoxSides::with_value(false),
                    relative_offset: descendant.containing_block.RelativeOffset(),
                };
                self.containing_blocks_map_.insert(key, info.clone());
                return info;
            }
        }
        if candidate.Node().Style().GetPosition() == EPosition::kFixed {
            if let Some(viewport) = &self.viewport_containing_block_ {
                return viewport.clone();
            }
        }
        self.default_containing_block_.clone()
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:337-337
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:1703-1756
    fn SetupNodeInfo(&mut self, oof_node: &LogicalOofPositionedNode) -> NodeInfo {
        let node = oof_node.Node();
        let descendant = if oof_node.IsForFragmentation() {
            Some(unsafe {
                &*(oof_node as *const LogicalOofPositionedNode
                    as *const LogicalOofNodeForFragmentation)
            })
        } else {
            None
        };
        let containing_block_fragment =
            descendant.map_or(std::ptr::null(), |value| value.containing_block.Fragment());
        #[cfg(debug_assertions)]
        {
            let container = if containing_block_fragment.is_null() {
                unsafe { &*self.container_builder_ }.GetLayoutObject()
            } else {
                unsafe { &*containing_block_fragment }.GetLayoutObject()
            };
            let layout_box = unsafe { &*node.GetLayoutBox() };
            let actual_containing_block = layout_box.ContainingBlock();
            if !container.is_null() {
                debug_assert!(
                    container == actual_containing_block as *const LayoutObject
                        || unsafe { &*actual_containing_block }.IsTable()
                );
            } else {
                debug_assert_eq!(
                    unsafe { &*containing_block_fragment }.GetBoxType(),
                    BoxType::kPageArea
                );
                debug_assert_eq!(
                    actual_containing_block as *const LayoutObject,
                    layout_box.View() as *const LayoutObject
                );
            }
        }
        let base_container_info = self.GetContainingBlockInfo(oof_node);
        let containing_block = descendant
            .map(|value| value.containing_block.clone())
            .unwrap_or_default();
        let fixedpos_containing_block = descendant
            .map(|value| value.fixedpos_containing_block.clone())
            .unwrap_or_default();
        let fixedpos_inline_container = descendant
            .map(|value| value.fixedpos_inline_container.clone())
            .unwrap_or_default();
        NodeInfo::new(
            node,
            oof_node.StaticPosition(),
            base_container_info,
            self.GetConstraintSpace().GetWritingDirection(),
            !containing_block_fragment.is_null(),
            &containing_block,
            &fixedpos_containing_block,
            &fixedpos_inline_container,
            oof_node.GetBreakToken(),
            oof_node.RequiresContentBeforeBreaking(),
        )
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:307-308
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:734-758
    fn ComputeInlineContainingBlocks(&mut self, candidates: &HeapVector<LogicalOofPositionedNode>) {
        let mut inline_container_fragments = InlineContainingBlockMap::default();
        for candidate in candidates {
            let inline_container = candidate.InlineContainer();
            if !inline_container.is_null() {
                inline_container_fragments.insert(
                    Member::from_ptr(inline_container as *mut LayoutObject),
                    Some(InlineContainingBlockGeometry::default()),
                );
            }
        }
        InlineContainingBlockUtils::ComputeInlineContainerGeometry(
            &mut inline_container_fragments,
            self.container_builder_,
        );
        let builder_size = *unsafe { &*self.container_builder_ }.Size();
        let physical_size =
            ToPhysicalSize(builder_size, self.GetConstraintSpace().GetWritingMode());
        self.AddInlineContainingBlockInfo(
            &inline_container_fragments,
            self.default_containing_block_.writing_direction,
            physical_size,
            LogicalOffset::default(),
            LogicalOffset::default(),
            false,
        );
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:309-310
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:760-828
    fn ComputeInlineContainingBlocksForFragmentainer(
        &mut self,
        descendants: &HeapVector<LogicalOofNodeForFragmentation>,
    ) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        struct InlineContainingBlockInfo {
            map: InlineContainingBlockMap,
            relative_offset: LogicalOffset,
            offset_to_fragmentation_context: LogicalOffset,
        }
        impl InlineContainingBlockInfo {
            fn Trace(&self, visitor: &mut Visitor) {
                visitor.Trace(&self.map);
            }
        }
        let mut inline_containing_blocks =
            HeapHashMap::<Member<LayoutBox>, InlineContainingBlockInfo>::default();
        for descendant in descendants {
            let inline_container = descendant.InlineContainer();
            if inline_container.is_null() {
                continue;
            }
            let fragment = descendant.containing_block.Fragment();
            debug_assert!(!fragment.is_null());
            let containing_block = To::<LayoutBox>(unsafe { &*fragment }.GetLayoutObject());
            let mut inline_geometry = InlineContainingBlockGeometry::default();
            inline_geometry.relative_offset = descendant.InlineContainerInfo().RelativeOffset();
            let containing_key = Member::from_ptr(containing_block);
            let inline_key = Member::from_ptr(inline_container as *mut LayoutObject);
            if let Some(info) = inline_containing_blocks.get_mut(&containing_key) {
                info.map.insert(inline_key, Some(inline_geometry));
                continue;
            }
            let mut inline_map = InlineContainingBlockMap::default();
            inline_map.insert(inline_key, Some(inline_geometry));
            inline_containing_blocks.insert(
                containing_key,
                InlineContainingBlockInfo {
                    map: inline_map,
                    relative_offset: descendant.containing_block.RelativeOffset(),
                    offset_to_fragmentation_context: descendant.containing_block.Offset(),
                },
            );
        }
        let containing_blocks: Vec<_> = inline_containing_blocks
            .iter()
            .map(|(key, _)| *key)
            .collect();
        for containing_block in containing_blocks {
            let info = inline_containing_blocks
                .get_mut(&containing_block)
                .expect("collected inline containing block");
            let containing_block = containing_block.Get();
            let containing_box = unsafe { &*containing_block };
            let size = LogicalSize::new(
                BoxInlineSize(containing_box),
                BoxTotalBlockSize(containing_box),
            );
            let physical_size = ToPhysicalSize(size, containing_box.StyleRef().GetWritingMode());
            InlineContainingBlockUtils::ComputeInlineContainerGeometryForFragmentainer(
                containing_block,
                physical_size,
                &mut info.map,
            );
            self.AddInlineContainingBlockInfo(
                &info.map,
                containing_box.StyleRef().GetWritingDirection(),
                physical_size,
                info.relative_offset,
                info.offset_to_fragmentation_context,
                true,
            );
        }
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:315-318
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:830-991
    fn AddInlineContainingBlockInfo(
        &mut self,
        inline_container_fragments: &InlineContainingBlockMap,
        container_writing_direction: WritingDirectionMode,
        container_builder_size: PhysicalSize,
        containing_block_relative_offset: LogicalOffset,
        containing_block_offset: LogicalOffset,
        adjust_for_fragmentation: bool,
    ) {
        for (key, value) in inline_container_fragments.iter() {
            let geometry = value
                .as_ref()
                .expect("inline containing-block geometry missing");
            let inline_cb_style = unsafe { &*key.Get() }.StyleRef();
            let inline_writing_direction = inline_cb_style.GetWritingDirection();
            let inline_cb_borders = ComputeBordersForInline(inline_cb_style);
            debug_assert_eq!(
                container_writing_direction.GetWritingMode(),
                inline_writing_direction.GetWritingMode()
            );
            let is_same_direction = container_writing_direction == inline_writing_direction;

            let start_rect = &geometry.start_fragment_union_rect;
            let container_converter =
                WritingModeConverter::new(container_writing_direction, container_builder_size);
            let mut start_offset =
                container_converter.ToLogicalOffset(start_rect.offset, start_rect.size);
            start_offset.block_offset += inline_cb_borders.block_start;
            if is_same_direction {
                start_offset.inline_offset += inline_cb_borders.inline_start;
            }

            let end_rect = &geometry.end_fragment_union_rect;
            let mut end_offset =
                container_converter.ToLogicalOffset(end_rect.offset, end_rect.size);
            end_offset +=
                ToLogicalSize(end_rect.size, container_writing_direction.GetWritingMode());
            end_offset.block_offset -= inline_cb_borders.block_end;
            if is_same_direction {
                end_offset.inline_offset -= inline_cb_borders.inline_end;
            }
            end_offset.inline_offset = end_offset.inline_offset.max(start_offset.inline_offset);
            end_offset.block_offset = end_offset.block_offset.max(start_offset.block_offset);

            let inline_cb_size = LogicalSize::new(
                end_offset.inline_offset - start_offset.inline_offset,
                end_offset.block_offset - start_offset.block_offset,
            );
            debug_assert!(inline_cb_size.inline_size >= LayoutUnit::default());
            debug_assert!(inline_cb_size.block_size >= LayoutUnit::default());
            if adjust_for_fragmentation {
                let physical_size =
                    ToPhysicalSize(inline_cb_size, self.GetConstraintSpace().GetWritingMode());
                let physical_offset = start_offset.ConvertToPhysical(
                    container_writing_direction,
                    container_builder_size,
                    physical_size,
                );
                start_offset = WritingModeConverter::new(
                    self.GetConstraintSpace().GetWritingDirection(),
                    container_builder_size,
                )
                .ToLogicalOffset(physical_offset, physical_size);
            }
            debug_assert!(
                (geometry.relative_offset == LogicalOffset::default()
                    && containing_block_relative_offset == LogicalOffset::default()
                    && containing_block_offset == LogicalOffset::default())
                    || unsafe { &*self.container_builder_ }.IsBlockFragmentationContextRoot()
            );
            let mut container_offset: LogicalOffset =
                (start_offset - geometry.relative_offset).into();
            let total_relative_offset = containing_block_relative_offset + geometry.relative_offset;
            container_offset += containing_block_offset;
            self.containing_blocks_map_.insert(
                Member::from_ptr(key.Get()),
                ContainingBlockInfo {
                    writing_direction: inline_writing_direction,
                    is_scroll_container: false,
                    is_hidden_for_paint: geometry.is_hidden_for_paint,
                    rect: LogicalRect::new(container_offset, inline_cb_size),
                    scroll_rect: None,
                    scroll_limit_rect: None,
                    scroll_direction: LogicalBoxSides::with_value(false),
                    relative_offset: total_relative_offset,
                },
            );
        }
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:319-319
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:993-1105
    fn LayoutCandidates(&mut self, candidates: &HeapVector<LogicalOofPositionedNode>) {
        let builder_ptr = self.container_builder_;
        if !self.should_add_outer_fragmentainer_children_
            || self.GetConstraintSpace().IsInitialColumnBalancingPass()
        {
            self.ComputeInlineContainingBlocks(candidates);
        }
        for candidate in candidates {
            let layout_box = candidate.Node().GetLayoutBox();
            unsafe { &mut *layout_box }.SetStaticPositionForLayout(&candidate.StaticPosition());
            if self.IsContainingBlockForCandidate(candidate) {
                if self.should_add_outer_fragmentainer_children_ {
                    debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
                    unsafe { &mut *builder_ptr }.SetHasOutOfFlowInFragmentainerSubtree(true);
                    if !self.GetConstraintSpace().IsInitialColumnBalancingPass() {
                        let mut descendant = LogicalOofNodeForFragmentation::from_oof(candidate);
                        unsafe { &*builder_ptr }
                            .AdjustFragmentainerDescendant(&mut descendant, false);
                        unsafe { &mut *builder_ptr }
                            .AdjustFixedposContainingBlockForInnerMulticols();
                        unsafe { &mut *builder_ptr }
                            .AddOutOfFlowFragmentainerDescendant(&descendant);
                        continue;
                    }
                }
                let is_inside_fragmentation_context =
                    InvolvedInBlockFragmentationForBuilder(unsafe { &*builder_ptr })
                        && RuntimeEnabledFeatures::FragmentedOofInCbEnabled();
                let node_info = self.SetupNodeInfo(candidate);
                let offset_info = self.CalculateOffset(&node_info, is_inside_fragmentation_context);
                let mut node_to_layout = NodeToLayout {
                    node_info,
                    offset_info,
                    containing_block_fragment: Member::default(),
                };
                let break_token = node_to_layout.node_info.break_token.Get();
                if self.GetConstraintSpace().HasKnownFragmentainerBlockSize()
                    && !IsBreakInside(break_token)
                    && RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
                {
                    let space_left = FragmentainerSpaceLeft(unsafe { &*builder_ptr }, true);
                    let block_overflow =
                        node_to_layout.offset_info.offset.block_offset - space_left;
                    if block_overflow > LayoutUnit::default() {
                        let start_inset = LogicalOffset::new(
                            node_to_layout.offset_info.offset.inline_offset,
                            block_overflow,
                        );
                        unsafe { &mut *builder_ptr }.AddBreakBeforeChild(
                            node_to_layout.node_info.node.clone().into(),
                            None,
                            false,
                            start_inset,
                        );
                        continue;
                    }
                }
                let result = self.LayoutOOFNode(&mut node_to_layout, std::ptr::null(), false);
                let result = unsafe { &*result };
                let physical_margins = node_to_layout
                    .offset_info
                    .node_dimensions
                    .margins
                    .ConvertToPhysical(node_to_layout.node_info.node.Style().GetWritingDirection());
                let margins = physical_margins
                    .ConvertToLogical(unsafe { &*builder_ptr }.GetWritingDirection());
                unsafe { &mut *builder_ptr }.AddResult(
                    result,
                    result.OutOfFlowPositionedOffset(),
                    Some(margins),
                    None,
                    candidate.InlineContainerInfo(),
                );
                unsafe { &mut *builder_ptr }.SetHasOutOfFlowFragmentChild(true);
                if self.GetConstraintSpace().IsInitialColumnBalancingPass() {
                    unsafe { &mut *builder_ptr }
                        .PropagateTallestUnbreakableBlockSize(result.TallestUnbreakableBlockSize());
                }
                let fragment = To::<PhysicalBoxFragment>(result.GetPhysicalFragment());
                let fragment = unsafe { &*fragment };
                let outgoing_break_token = fragment.GetBreakToken();
                if !outgoing_break_token.is_null()
                    && unsafe { &*outgoing_break_token }.IsRepeated()
                    && RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
                {
                    debug_assert_eq!(unsafe { &*builder_ptr }.GetBoxType(), BoxType::kPageArea);
                    debug_assert!(fragment.IsFixedPositioned());
                    if !unsafe { &*builder_ptr }.HasInsertedChildBreak() {
                        self.repeated_fixed_pos_boxes_
                            .push(Member::from_ptr(fragment.MutableOwnerLayoutBox()));
                    }
                }
                let mut child_candidates = HeapVector::<LogicalOofPositionedNode>::default();
                unsafe { &mut *builder_ptr }
                    .SwapOutOfFlowPositionedCandidates(&mut child_candidates);
                if !child_candidates.is_empty() {
                    self.LayoutCandidates(&child_candidates);
                }
            } else {
                unsafe { &mut *builder_ptr }.AddOutOfFlowDescendant(candidate);
            }
        }
        debug_assert!(!unsafe { &*builder_ptr }.HasOutOfFlowPositionedCandidates());
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:292-293
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:407-416
    pub fn InitialContainingBlockFixedSize(container: BlockNode) -> Option<LogicalSize> {
        let box_ptr = container.GetLayoutBox();
        let box_ = unsafe { &*box_ptr };
        if !box_.IsLayoutView() || box_.IsPrintingForLayout() {
            return None;
        }
        let view = To::<LayoutView>(box_ptr);
        let layout_size = unsafe { &*view }.GetLayoutSize(
            layoutng_assembly::internal::scroll_types::IncludeScrollbarsInRect::kIncludeScrollbars,
        );
        Some(ToLogicalSize(
            PhysicalSize::new(
                LayoutUnit::from_signed(layout_size.width()),
                LayoutUnit::from_signed(layout_size.height()),
            ),
            container.Style().GetWritingMode(),
        ))
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:47-48
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:418-489
    pub fn new(container_builder: &mut BoxFragmentBuilder) -> Self {
        let mut part = Self {
            container_builder_: container_builder as *mut BoxFragmentBuilder,
            default_containing_block_: ContainingBlockInfo::default(),
            viewport_containing_block_: None,
            containing_blocks_map_: HeapHashMap::default(),
            repeated_fixed_pos_boxes_: HeapVector::default(),
            delayed_descendants_: HeapVector::default(),
            multicol_children_: std::ptr::null_mut(),
            column_balancing_info_: std::ptr::null_mut(),
            child_fragment_storage_: std::ptr::null_mut(),
            fragmentainer_consumed_block_size_: LayoutUnit::default(),
            is_absolute_container_: container_builder.Node().IsAbsoluteContainer(),
            is_fixed_container_: container_builder.Node().IsFixedContainer(),
            should_add_outer_fragmentainer_children_:
                !RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
                    && InvolvedInBlockFragmentationForBuilder(container_builder),
            outer_context_has_fixedpos_container_: false,
            needs_total_page_count_: false,
            additional_pages_were_added_: false,
        };
        if !container_builder.HasOutOfFlowPositionedCandidates()
            && !container_builder.HasOutOfFlowFragmentainerDescendants()
            && !container_builder.HasMulticolsWithPendingOOFs()
            && !container_builder.IsRoot()
        {
            return part;
        }

        let node = container_builder.Node();
        let space = container_builder.GetConstraintSpace();
        let writing_direction = space.GetWritingDirection();
        let is_scroll_container = node.IsScrollContainer();
        let is_hidden_for_paint = space.IsHiddenForPaint();
        let border_scrollbar = *container_builder.Borders() + *container_builder.Scrollbar();
        let padding = container_builder.Padding();
        let has_block_size = container_builder.HasBlockSize();
        let container_size = if has_block_size {
            ShrinkLogicalSize(*container_builder.Size(), &border_scrollbar)
        } else {
            LogicalSize::default()
        };
        let container_rect = LogicalRect::new(border_scrollbar.StartOffset(), container_size);
        let scroll_direction = if is_scroll_container {
            CalculateScrollDirection(&node, writing_direction)
        } else {
            LogicalBoxSides::with_value(false)
        };
        let mut scroll_rect = None;
        if is_scroll_container && has_block_size {
            if let Some(inflow_bounds) = container_builder.InflowBounds() {
                scroll_rect = Some(CalculateScrollRect(
                    scroll_direction,
                    &container_rect,
                    padding,
                    inflow_bounds,
                ));
            }
        }
        part.default_containing_block_ = ContainingBlockInfo {
            writing_direction,
            is_scroll_container,
            is_hidden_for_paint,
            rect: container_rect,
            scroll_rect,
            scroll_direction,
            ..ContainingBlockInfo::default()
        };
        if let Some(viewport_size) = Self::InitialContainingBlockFixedSize(node) {
            part.viewport_containing_block_ = Some(ContainingBlockInfo {
                writing_direction,
                is_scroll_container,
                is_hidden_for_paint,
                rect: LogicalRect::new(
                    container_rect.offset,
                    ShrinkLogicalSize(viewport_size, &border_scrollbar),
                ),
                scroll_rect: None,
                scroll_limit_rect: scroll_rect,
                scroll_direction: LogicalBoxSides::with_value(false),
                ..ContainingBlockInfo::default()
            });
        }
        part
    }

    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.h:48-48
    // cpp: layoutng_out_of_flow/out_of_flow_layout_part.cc:491-563
    pub fn Run(&mut self) {
        let builder_ptr = self.container_builder_;
        if !RuntimeEnabledFeatures::FragmentedOofInCbEnabled() {
            if unsafe { &*builder_ptr }.IsPaginatedRoot() {
                self.PropagateOOFsFromPageAreas();
            }
            self.HandleFragmentation();
        }
        let node = unsafe { &*builder_ptr }.Node();
        if node.ChildLayoutBlockedByDisplayLock() {
            return;
        }

        let mut candidates = HeapVector::<LogicalOofPositionedNode>::default();
        let _clear_scope = unsafe { ClearCollectionScope::new(&mut candidates as *mut _) };
        unsafe { &mut *builder_ptr }.SwapOutOfFlowPositionedCandidates(&mut candidates);
        if !candidates.is_empty() {
            self.LayoutCandidates(&candidates);
        } else if !RuntimeEnabledFeatures::FragmentedOofInCbEnabled() {
            unsafe { &mut *builder_ptr }.AdjustFixedposContainingBlockForFragmentainerDescendants();
            unsafe { &mut *builder_ptr }.AdjustFixedposContainingBlockForInnerMulticols();
        }

        if unsafe { &*builder_ptr }.IsRoot() {
            let mut child = node.FirstChild();
            while !child.IsNull() {
                if child.IsBlock() {
                    let block_child = BlockNode::from(child.clone());
                    if block_child.IsInTopOrViewTransitionLayer()
                        && block_child.IsOutOfFlowPositioned()
                    {
                        unsafe { &mut *builder_ptr }.AddOutOfFlowChildCandidate(
                            &block_child,
                            &LogicalStaticPosition::default(),
                            true,
                        );
                        if !RuntimeEnabledFeatures::FragmentedOofInCbEnabled() {
                            self.HandleFragmentation();
                        }
                        candidates.Shrink(0);
                        unsafe { &mut *builder_ptr }
                            .SwapOutOfFlowPositionedCandidates(&mut candidates);
                        self.LayoutCandidates(&candidates);
                    }
                }
                child = child.NextSibling();
            }
        }

        if !self.repeated_fixed_pos_boxes_.is_empty()
            && !unsafe { &*builder_ptr }.HasInsertedChildBreak()
        {
            debug_assert!(RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
            for box_ in &self.repeated_fixed_pos_boxes_ {
                BlockNode::new(box_.Get()).FinishRepeatableRoot();
            }
        }
    }
}
