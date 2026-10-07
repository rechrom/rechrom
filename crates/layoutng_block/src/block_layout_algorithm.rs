#![allow(non_snake_case)]

use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use font_engine::FontBaseline;
use foundation::{
    kIndefiniteSize, DynamicTo, EBlockEllipsis, EBreakBetween, EClear, EFloat, EOverflow,
    ETextAlign, IsHorizontalWritingMode, IsParallelWritingMode, IsRtl, LayoutUnit, Length,
    LogicalToLogical, MakeGarbageCollected, MarginStrut, RuntimeEnabledFeatures, TextDirection, To,
    UnsupportedLayout,
};
use layoutng::internal::algorithm_entry::NativeAlgorithm;
use layoutng::internal::algorithm_forward::{
    BlockLayoutAlgorithm as ForwardBlockLayoutAlgorithm, InlineChildLayoutContext,
    PreviousInflowPosition as ForwardPreviousInflowPosition,
};
use layoutng::internal::block_node::BlockNode;
use layoutng::internal::break_appeal::BreakAppeal;
use layoutng::internal::column_spanner_path::ColumnSpannerPath;
use layoutng::internal::constraint_space::{
    AdjoiningObjectTypeValue, AdjoiningObjectTypes, AutoSizeBehavior, BaselineAlgorithmType,
    ConstraintSpace,
};
use layoutng::internal::constraint_space_builder::ConstraintSpaceBuilder;
use layoutng::internal::constraint_space_builder_style::MinMaxConstraintSpaceBuilder;
use layoutng::internal::early_break::EarlyBreak;
use layoutng::internal::exclusions::exclusion_space::ExclusionSpace;
use layoutng::internal::form_control_sizing_service::ComputedStyleControlSizingExt;
use layoutng::internal::fragmentation_utils::IsEarlyBreakTarget;
use layoutng::internal::fragmentation_utils::{
    AdjustMarginsForFragmentation, AdjustedMarginAfterFinalChildFragment, AttemptSoftBreakDefault,
    BreakBeforeChild, BreakStatus, CalculateBreakAppealBeforeChild, CalculateBreakBetweenValue,
    EnterEarlyBreakInChild, FinishFragmentation, FinishFragmentationForFragmentainer,
    FollowColumnSpannerPath, FragmentainerOffsetAtBfcForBuilder,
    HasBreakOpportunityBeforeNextChild, InvolvedInBlockFragmentationForBuilder, IsBreakInside,
    IsForcedBreakValue, PropagateSpaceShortageDefault,
    SetupSpaceBuilderForFragmentationFromBuilder, ShouldIncludeBlockEndBorderPadding,
};
use layoutng::internal::inline_node::InlineNode;
use layoutng::internal::layout_algorithm::{
    LayoutAlgorithm, LayoutAlgorithmParams, RelayoutAlgorithm, RelayoutType,
};
use layoutng::internal::layout_block_flow::LayoutBlockFlow;
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_input_node::{LayoutInputNode, MinMaxSizesFloatInput};
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_pass_scope::LayoutPassScope;
use layoutng::internal::layout_utils::{BlockStaticPositionEdge, InlineStaticPositionEdge};
use layoutng::internal::length_utils::{
    CalculateChildPercentageSize, CalculateMinMaxSizesIgnoringChildren,
    CalculateReplacedChildPercentageSize, ClampIntrinsicBlockSize, ComputeBlockSizeForFragment,
    ComputeBorders, ComputeInitialBlockSizeForFragment, ComputeInitialMinMaxBlockSizes,
    ComputeInlineSizeForFragmentWithOverride, ComputeMarginsFor, ComputeMarginsForInlineSize,
    ComputeMarginsForSelf, ComputeMinAndMaxContentContribution, ComputePadding,
    ResolveInlineAutoMargins,
};
use layoutng::internal::line_clamp_data::{
    LineClampAncestorChain, LineClampData, State as LineClampState,
};
use layoutng::internal::min_max_sizes::{MinMaxSizes, MinMaxSizesResult};
use layoutng::internal::space_utils::{
    AdjustToClearance, SetOrthogonalFallbackInlineSize,
    SetTextBoxTrimOnChildSpaceBuilderWithKnownContent,
    ShouldBlockContainerChildStretchAutoInlineSize,
};
use layoutng::internal::style_variant::StyleVariant;
use layoutng::internal::unpositioned_float::UnpositionedFloat;
use layoutng::internal::unpositioned_list_marker::UnpositionedListMarker;
use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_fragment_tree::box_fragment_builder::BoxFragmentBuilder;
use layoutng_fragment_tree::break_token::BreakToken;
use layoutng_fragment_tree::fragment_builder::FragmentBuilder;
use layoutng_fragment_tree::inline_break_token::InlineBreakToken;
use layoutng_fragment_tree::layout_result::{EStatus, LayoutResult};
use layoutng_fragment_tree::logical_box_fragment::LogicalBoxFragment;
use layoutng_fragment_tree::logical_fragment::LogicalFragment;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_fragment_tree::physical_fragment::BoxType;
use layoutng_fragment_tree::physical_fragment::PhysicalFragment;
use layoutng_fragment_tree::physical_line_box_fragment::PhysicalLineBoxFragment;
use layoutng_geometry::geometry::bfc_offset::BfcOffset;
use layoutng_geometry::geometry::box_sides::LogicalBoxSides;
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_size::LogicalSize;
use layoutng_geometry::geometry::static_position::{InlineEdge, LogicalStaticPosition};
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::{
    EMarginTrim, ItemPosition, OverflowAlignment,
};
use layoutng_style::style::style_self_alignment_data::StyleSelfAlignmentData;

use crate::block_layout_utils::{AlignBlockContent, CalculateOutOfFlowStaticInlineLevelOffset};

// cpp: layoutng_block/block_layout_algorithm.cc:48-94
fn HasLineEvenIfEmpty(box_: *mut LayoutBox) -> bool {
    let block_flow = DynamicTo::<LayoutBlockFlow>(box_);
    if block_flow.is_null() {
        return false;
    }
    let block_flow = unsafe { &*block_flow };
    if block_flow.FirstChild().is_null() {
        return block_flow.HasLineIfEmpty();
    }
    if block_flow.ChildrenInline() {
        return block_flow.HasLineIfEmpty()
            && InlineNode::new(block_flow as *const _ as *mut LayoutBlockFlow).IsBlockLevel();
    }
    let fragmentation_context_root = if block_flow.IsMulticolContainer() {
        Some(block_flow)
    } else {
        None
    };
    if let Some(root) = fragmentation_context_root {
        debug_assert!(!root.ChildrenInline());
        let mut child = root.FirstChild();
        while !child.is_null() {
            let item = unsafe { &*child };
            if item.IsInline() {
                debug_assert!(item.IsLayoutOutsideListMarker());
                return false;
            }
            if !item.IsFloatingOrOutOfFlowPositioned() {
                return false;
            }
            child = item.NextSibling();
        }
        return block_flow.HasLineIfEmpty();
    }
    false
}

// cpp: layoutng_block/block_layout_algorithm.cc:96-104
fn IsLastInflowChild(box_: &LayoutBox) -> bool {
    let mut next = box_.NextSibling();
    while !next.is_null() {
        let object = unsafe { &*next };
        if !object.IsFloatingOrOutOfFlowPositioned() {
            return false;
        }
        next = object.NextSibling();
    }
    true
}

// cpp: layoutng_block/block_layout_algorithm.cc:106-119
fn LayoutBlockChild(
    space: &ConstraintSpace,
    break_token: *const BreakToken,
    early_break: *const EarlyBreak,
    column_spanner_path: *const ColumnSpannerPath,
    node: &mut BlockNode,
) -> *const LayoutResult {
    let mut early_break_in_child = std::ptr::null();
    if !early_break.is_null() {
        early_break_in_child = EnterEarlyBreakInChild(node, unsafe { &*early_break });
    }
    let column_spanner_path = FollowColumnSpannerPath(column_spanner_path, node);
    node.Layout(
        space,
        To::<BlockBreakToken>(break_token),
        early_break_in_child,
        column_spanner_path,
    )
}

// cpp: layoutng_block/block_layout_algorithm.cc:121-137
fn LayoutInflow(
    space: &ConstraintSpace,
    break_token: *const BreakToken,
    early_break: *const EarlyBreak,
    column_spanner_path: *const ColumnSpannerPath,
    node: &mut LayoutInputNode,
    context: *mut InlineChildLayoutContext,
) -> *const LayoutResult {
    if node.IsInline() {
        let inline_node = InlineNode::from(node.clone());
        let algorithms = LayoutPassScope::Algorithms();
        let callback = if algorithms.is_null() {
            None
        } else {
            unsafe { &*algorithms }.inline_support.layout
        };
        let callback = callback.unwrap_or_else(|| {
            std::panic::panic_any(UnsupportedLayout::new(
                "inline layout module is not installed",
            ))
        });
        return callback(
            &inline_node,
            space,
            break_token,
            column_spanner_path,
            context,
        );
    }
    let mut block_node = BlockNode::from(node.clone());
    LayoutBlockChild(
        space,
        break_token,
        early_break,
        column_spanner_path,
        &mut block_node,
    )
}

// cpp: layoutng_block/block_layout_algorithm.cc:139-152
fn ToAdjoiningObjectTypes(clear: EClear) -> AdjoiningObjectTypes {
    match clear {
        EClear::kNone => AdjoiningObjectTypeValue::kAdjoiningNone as AdjoiningObjectTypes,
        EClear::kLeft => AdjoiningObjectTypeValue::kAdjoiningFloatLeft as AdjoiningObjectTypes,
        EClear::kRight => AdjoiningObjectTypeValue::kAdjoiningFloatRight as AdjoiningObjectTypes,
        EClear::kBoth => AdjoiningObjectTypeValue::kAdjoiningFloatBoth as AdjoiningObjectTypes,
        _ => std::process::abort(),
    }
}

// cpp: layoutng_block/block_layout_algorithm.cc:154-166
fn HasClearancePastAdjoiningFloats(
    adjoining_object_types: AdjoiningObjectTypes,
    child_style: &ComputedStyle,
    cb_style: &ComputedStyle,
) -> bool {
    ToAdjoiningObjectTypes(child_style.ClearWithContainingStyle(cb_style)) & adjoining_object_types
        != 0
}

// cpp: layoutng_block/block_layout_algorithm.cc:168-196
fn ApplyClearance(space: &ConstraintSpace, bfc_block_offset: &mut LayoutUnit) -> bool {
    if space.HasClearanceOffset() && *bfc_block_offset < space.ClearanceOffset() {
        *bfc_block_offset = space.ClearanceOffset();
        return true;
    }
    false
}

// cpp: layoutng_block/block_layout_algorithm.cc:198-214
fn LogicalFromBfcLineOffset(
    child_bfc_line_offset: LayoutUnit,
    parent_bfc_line_offset: LayoutUnit,
    child_inline_size: LayoutUnit,
    parent_inline_size: LayoutUnit,
    direction: TextDirection,
) -> LayoutUnit {
    let relative_line_offset = child_bfc_line_offset - parent_bfc_line_offset;
    if direction == TextDirection::kLtr {
        relative_line_offset
    } else {
        parent_inline_size - relative_line_offset - child_inline_size
    }
}

// cpp: layoutng_block/block_layout_algorithm.cc:216-227
fn LogicalFromBfcOffsets(
    child_bfc_offset: &BfcOffset,
    parent_bfc_offset: &BfcOffset,
    child_inline_size: LayoutUnit,
    parent_inline_size: LayoutUnit,
    direction: TextDirection,
) -> LogicalOffset {
    let inline_offset = LogicalFromBfcLineOffset(
        child_bfc_offset.line_offset,
        parent_bfc_offset.line_offset,
        child_inline_size,
        parent_inline_size,
        direction,
    );
    LogicalOffset::new(
        inline_offset,
        child_bfc_offset.block_offset - parent_bfc_offset.block_offset,
    )
}

// cpp: layoutng_block/block_layout_algorithm.cc:229-241
fn WebkitTextToItemPosition(text_align: ETextAlign) -> ItemPosition {
    match text_align {
        ETextAlign::kWebkitLeft => ItemPosition::kLeft,
        ETextAlign::kWebkitCenter => ItemPosition::kCenter,
        ETextAlign::kWebkitRight => ItemPosition::kRight,
        _ => ItemPosition::kNormal,
    }
}

// cpp: layoutng_block/block_layout_algorithm.cc:243-299
fn WebkitTextAlignAndJustifySelfOffset(
    child_style: &ComputedStyle,
    style: &ComputedStyle,
    available_space: LayoutUnit,
    margins: &BoxStrut,
    child_inline_size: impl Fn() -> LayoutUnit,
) -> LayoutUnit {
    debug_assert!(!child_style.MarginInlineStartUsing(style).IsAuto());
    debug_assert!(!child_style.MarginInlineEndUsing(style).IsAuto());
    let default_alignment =
        StyleSelfAlignmentData::new_nonlegacy(ItemPosition::kNormal, OverflowAlignment::kDefault);
    let alignment_data = child_style.ResolvedJustifySelf(&default_alignment, style);
    let mut justify_self = alignment_data.GetPosition();
    let mut safe = OverflowAlignment::kSafe;
    if justify_self != ItemPosition::kNormal {
        safe = alignment_data.Overflow();
    } else {
        justify_self = WebkitTextToItemPosition(style.GetTextAlign());
    }
    let free_space = || {
        let free_space = available_space - child_inline_size() - margins.InlineSum();
        if safe == OverflowAlignment::kSafe {
            free_space.ClampNegativeToZero()
        } else {
            free_space
        }
    };
    let self_start_end_converter = || {
        let free_space = free_space();
        LogicalToLogical::<LayoutUnit>::new(
            child_style.GetWritingDirection(),
            style.GetWritingDirection(),
            LayoutUnit::default(),
            free_space,
            LayoutUnit::default(),
            free_space,
        )
    };
    let is_rtl = IsRtl(style.Direction());
    match justify_self {
        ItemPosition::kLeft => {
            if is_rtl {
                free_space()
            } else {
                LayoutUnit::default()
            }
        }
        ItemPosition::kCenter => free_space() / 2,
        ItemPosition::kRight => {
            if is_rtl {
                LayoutUnit::default()
            } else {
                free_space()
            }
        }
        ItemPosition::kFlexStart | ItemPosition::kStart => LayoutUnit::default(),
        ItemPosition::kFlexEnd | ItemPosition::kEnd => free_space(),
        ItemPosition::kSelfStart => self_start_end_converter().InlineStart(),
        ItemPosition::kSelfEnd => self_start_end_converter().InlineEnd(),
        _ => LayoutUnit::default(),
    }
}

// The child receives the preceding in-flow sibling's position and annotation
// state by value. This record's field order matches the C++ struct.
// cpp: layoutng_block/block_layout_algorithm.h:38-51
#[derive(Clone)]
pub struct PreviousInflowPosition {
    pub logical_block_offset: LayoutUnit,
    pub margin_strut: MarginStrut,
    pub block_end_annotation_space: LayoutUnit,
    pub previous_sibling_block_end_annotation_space: LayoutUnit,
    pub self_collapsing_child_had_clearance: bool,
}

// cpp: layoutng_block/block_layout_algorithm.h:53-71
#[derive(Clone)]
pub struct InflowChildData {
    pub bfc_offset_estimate: BfcOffset,
    pub margin_strut: MarginStrut,
    pub margins: BoxStrut,
    pub is_pushed_by_floats: bool,
}

impl InflowChildData {
    // cpp: layoutng_block/block_layout_algorithm.h:56-63
    pub fn new(
        bfc_offset_estimate: BfcOffset,
        margin_strut: &MarginStrut,
        margins: &BoxStrut,
        is_pushed_by_floats: bool,
    ) -> Self {
        Self {
            bfc_offset_estimate,
            margin_strut: margin_strut.clone(),
            margins: *margins,
            is_pushed_by_floats,
        }
    }
}

// STACK_ALLOCATED maps to a value type. C++ keeps the last object and ancestor
// chain as borrowed pointers, while the optional previous position is owned.
// cpp: layoutng_block/block_layout_algorithm.h:73-159
#[derive(Clone)]
pub struct BlockLineClampData {
    pub data: LineClampData,
    pub ignore_line_clamp: bool,
    pub initial_lines_until_clamp: i32,
    pub previous_inflow_position_when_clamped: Option<PreviousInflowPosition>,
    pub ignore_further_lines: bool,
    pub last_layout_object: *const LayoutObject,
    pub ancestor_chain: *const LineClampAncestorChain,
}

impl BlockLineClampData {
    // cpp: layoutng_block/block_layout_algorithm.h:77-82
    pub fn new(data: LineClampData) -> Self {
        let initial_lines_until_clamp = if data.IsClampByLines() {
            data.lines_until_clamp
        } else {
            0
        };
        Self {
            data,
            ignore_line_clamp: false,
            initial_lines_until_clamp,
            previous_inflow_position_when_clamped: None,
            ignore_further_lines: false,
            last_layout_object: std::ptr::null(),
            ancestor_chain: std::ptr::null(),
        }
    }

    // cpp: layoutng_block/block_layout_algorithm.h:84-98
    pub fn LinesUntilClamp(&self, show_measured_lines: bool) -> Option<i32> {
        self.data.LinesUntilClamp(show_measured_lines)
    }

    pub fn IsPastClampPoint(&self) -> bool {
        self.data.IsPastClampPoint()
    }

    pub fn ShouldHideForPaint(&self) -> bool {
        self.data.ShouldHideForPaint()
    }

    pub fn ShouldRelayoutWithNoForcedTruncate(&self) -> bool {
        if self.previous_inflow_position_when_clamped.is_none() {
            return false;
        }
        debug_assert!(self.data.IsClampByLines());
        self.data.lines_until_clamp == 0
    }

    // cpp: layoutng_block/block_layout_algorithm.h:108-126
    pub fn PropagateClampAfterLayoutObject(
        &self,
        layout_result: &LayoutResult,
    ) -> *const LayoutObject {
        let clamp_after = layout_result.LineClampAfterLayoutObject();
        if !clamp_after.is_null() {
            return clamp_after;
        }
        if layout_result.LinesUntilClamp() != self.data.lines_until_clamp {
            return std::ptr::null();
        }
        self.last_layout_object
    }

    // cpp: layoutng_block/block_layout_algorithm.h:100-106
    // cpp: layoutng_block/block_layout_algorithm.cc:3978-4066
    pub fn Setup(&mut self, node: &BlockNode, builder: &BoxFragmentBuilder) {
        let constraint_space = builder.GetConstraintSpace();
        let style = node.Style();
        if style.HasLineClamp() && !node.IsMulticolContainer() {
            if self.data.IsLineClampContext() {
                return;
            }
            debug_assert_eq!(self.data.state, LineClampState::kDisabled);
            if self.ignore_line_clamp {
                return;
            }
            let mut clamp_bfc_offset = kIndefiniteSize;
            if RuntimeEnabledFeatures::CSSLineClampEnabled() && style.MaxLines().HasAutoKeyword() {
                clamp_bfc_offset = builder.ChildAvailableSize().block_size;
                if clamp_bfc_offset == kIndefiniteSize {
                    let sizes = ComputeInitialMinMaxBlockSizes(
                        constraint_space,
                        node,
                        builder.BorderPadding(),
                        kIndefiniteSize,
                    );
                    if sizes.max_size != LayoutUnit::Max() {
                        clamp_bfc_offset = sizes.max_size;
                    }
                } else {
                    clamp_bfc_offset += builder.BorderScrollbarPadding().BlockSum();
                }
            }
            debug_assert!(style.LineClamp() >= 0);
            self.data.lines_until_clamp = style.LineClamp();
            if clamp_bfc_offset != kIndefiniteSize {
                self.data.state = if self.data.lines_until_clamp != 0 {
                    LineClampState::kClampByLinesWithBfcOffset
                } else {
                    LineClampState::kMeasureLinesUntilBfcOffset
                };
                self.data.clamp_bfc_offset = clamp_bfc_offset;
                self.ancestor_chain = MakeGarbageCollected(LineClampAncestorChain::new_root(
                    builder.BorderPadding().block_end,
                ));
            } else if self.data.lines_until_clamp != 0 {
                self.data.state = LineClampState::kClampByLines;
            } else {
                self.data.state = LineClampState::kDisabled;
                return;
            }
            if !RuntimeEnabledFeatures::CSSLineClampEnabled() {
                self.data.block_ellipsis = EBlockEllipsis::kEllipsis;
            } else if !RuntimeEnabledFeatures::CSSLineClampAsShorthandEnabled() {
                self.data.block_ellipsis = style.LineClampInternalBlockEllipsis();
            }
            return;
        }
        if self.data.IsMeasureUntilBfcOffset() {
            let block_min_max_sizes = ComputeInitialMinMaxBlockSizes(
                constraint_space,
                node,
                builder.BorderPadding(),
                kIndefiniteSize,
            );
            let is_fixed_block_size = builder.ChildAvailableSize().block_size != kIndefiniteSize
                || node.ShouldApplyBlockSizeContainment()
                || block_min_max_sizes.min_size == block_min_max_sizes.max_size;
            if is_fixed_block_size {
                self.data.state = LineClampState::kCountLines;
            } else {
                let parent = constraint_space.GetLineClampAncestorChain();
                debug_assert!(!parent.is_null());
                let end_margin = ComputeMarginsForSelf(constraint_space, style).block_end;
                self.ancestor_chain = MakeGarbageCollected(LineClampAncestorChain::new_child(
                    *builder.BfcBlockOffset(),
                    builder.BorderPadding().block_end,
                    end_margin,
                    block_min_max_sizes,
                    parent,
                ));
            }
        }
    }

    // cpp: layoutng_block/block_layout_algorithm.cc:4068-4170
    pub fn UpdateAfterLayout(
        &mut self,
        layout_result: &LayoutResult,
        previous: &PreviousInflowPosition,
        builder: &BoxFragmentBuilder,
    ) -> bool {
        let fragment = layout_result.GetPhysicalFragment();
        let mut old_lines_until_clamp = 0;
        if self.data.IsClampByLines() || self.data.IsCountLines() {
            old_lines_until_clamp = self.data.lines_until_clamp;
            if !fragment.IsFormattingContextRoot() && !self.ignore_further_lines {
                self.data.lines_until_clamp = layout_result.LinesUntilClamp();
            }
            if self.data.IsClampByLines()
                && !fragment.IsLineBox()
                && !self.ignore_further_lines
                && old_lines_until_clamp == 0
                && self.data.lines_until_clamp == 0
            {
                self.data.lines_until_clamp = -1;
            }
        }
        if self.data.IsMeasureUntilBfcOffset()
            && self.previous_inflow_position_when_clamped.is_none()
        {
            let Some(bfc_block_offset) = builder.BfcBlockOffset() else {
                debug_assert_eq!(old_lines_until_clamp, self.data.lines_until_clamp);
                debug_assert!(fragment.Size().IsEmpty());
                return true;
            };
            let mut padding_annotation_overflow = LayoutUnit::default();
            if previous.block_end_annotation_space < LayoutUnit::default() {
                padding_annotation_overflow = previous
                    .block_end_annotation_space
                    .max(-builder.Padding().block_end);
            }
            debug_assert!(!self.ancestor_chain.is_null());
            if !unsafe { &*self.ancestor_chain }.HasBfcOffset() {
                self.ancestor_chain =
                    unsafe { &*self.ancestor_chain }.WithResolvedBfcOffset(*bfc_block_offset);
            }
            let bfc_offset = unsafe { &*self.ancestor_chain }.FinalLineClampBlockSize(
                previous.logical_block_offset + padding_annotation_overflow,
                previous.margin_strut.clone(),
            );
            if bfc_offset > self.data.clamp_bfc_offset {
                self.data.lines_until_clamp = old_lines_until_clamp;
                return false;
            }
            if old_lines_until_clamp == self.data.lines_until_clamp
                || !layout_result.LineClampAfterLayoutObject().is_null()
            {
                if !fragment.IsLineBox() {
                    self.last_layout_object = fragment.GetLayoutObject();
                    debug_assert!(!self.last_layout_object.is_null());
                }
            } else {
                self.last_layout_object = std::ptr::null();
            }
        }
        if self.data.IsClampByLines() {
            if layout_result.WouldBeLastLineIfNotForEllipsis() {
                debug_assert!(fragment.IsLineBox());
                debug_assert_eq!(self.data.lines_until_clamp, 0);
                self.ignore_further_lines = true;
            }
            if self.IsPastClampPoint() && self.previous_inflow_position_when_clamped.is_none() {
                self.previous_inflow_position_when_clamped = Some(previous.clone());
            }
        }
        if self.data.state == LineClampState::kClampAfterLayoutObject
            && (layout_result.LinesUntilClamp() < 0
                || self.data.clamp_after_layout_object == fragment.GetLayoutObject())
        {
            debug_assert!(self.previous_inflow_position_when_clamped.is_none());
            self.data.state = LineClampState::kClampByLines;
            self.data.lines_until_clamp = -1;
            self.previous_inflow_position_when_clamped = Some(previous.clone());
        }
        true
    }
}

// A C++ derived LayoutAlgorithm has its base at offset zero. Rust retains
// that layout and uses Deref to share the translated base's accessors.
// cpp: layoutng_block/block_layout_algorithm.h:161-175
// cpp: layoutng_block/block_layout_algorithm.h:510-587
#[repr(C)]
pub struct BlockLayoutAlgorithm {
    pub(crate) base: LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>,
    pub(crate) child_percentage_size_: LogicalSize,
    pub(crate) replaced_child_percentage_size_: LogicalSize,
    pub(crate) previous_result_: *const LayoutResult,
    pub(crate) column_spanner_path_: *const ColumnSpannerPath,
    pub(crate) last_non_empty_inflow_child_: InlineNode,
    pub(crate) last_non_empty_break_token_: *const BreakToken,
    pub(crate) override_text_box_trim_end_child_: InlineNode,
    pub(crate) override_text_box_trim_end_break_token_: *const BreakToken,
    pub(crate) last_non_self_collapsing_child_: LayoutInputNode,
    pub(crate) pending_margin_end_trim_child_: LayoutInputNode,
    pub(crate) incoming_margin_strut_: MarginStrut,
    pub(crate) intrinsic_block_size_: LayoutUnit,
    pub(crate) line_clamp_data_: BlockLineClampData,
    pub(crate) first_overflowing_line_: i32,
    pub(crate) fit_all_lines_: bool,
    pub(crate) is_resuming_: bool,
    pub(crate) abort_when_bfc_block_offset_updated_: bool,
    pub(crate) has_break_opportunity_before_next_child_: bool,
    pub(crate) is_relayout_for_margin_end_trim_: bool,
    pub(crate) is_measuring_text_fit_: bool,
    pub(crate) is_last_non_self_collapsing_child_determined_: bool,
}

impl Deref for BlockLayoutAlgorithm {
    type Target = LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for BlockLayoutAlgorithm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

// C++ nests this only for the text-control placeholder result. Rust places
// it beside the algorithm because a struct cannot contain a named type.
// cpp: layoutng_block/block_layout_algorithm.h:482-486
pub struct PlaceholderLayoutResult {
    pub logical_block_offset: LayoutUnit,
    pub status: layoutng_fragment_tree::layout_result::EStatus,
}

impl BlockLayoutAlgorithm {
    // cpp: layoutng_block/block_layout_algorithm.h:166-168
    // cpp: layoutng_block/block_layout_algorithm.cc:303-336
    pub fn new(params: &LayoutAlgorithmParams) -> Self {
        let base = LayoutAlgorithm::from_params(params);
        let mut algorithm = Self {
            base,
            child_percentage_size_: LogicalSize::default(),
            replaced_child_percentage_size_: LogicalSize::default(),
            previous_result_: params.previous_result,
            column_spanner_path_: params.column_spanner_path,
            last_non_empty_inflow_child_: InlineNode::null(),
            last_non_empty_break_token_: std::ptr::null(),
            override_text_box_trim_end_child_: InlineNode::null(),
            override_text_box_trim_end_break_token_: std::ptr::null(),
            last_non_self_collapsing_child_: LayoutInputNode::null(),
            pending_margin_end_trim_child_: LayoutInputNode::null(),
            incoming_margin_strut_: MarginStrut::default(),
            intrinsic_block_size_: LayoutUnit::default(),
            line_clamp_data_: BlockLineClampData::new(params.space.GetLineClampData()),
            first_overflowing_line_: 0,
            fit_all_lines_: false,
            is_resuming_: IsBreakInside(params.break_token),
            abort_when_bfc_block_offset_updated_: false,
            has_break_opportunity_before_next_child_: false,
            is_relayout_for_margin_end_trim_: false,
            is_measuring_text_fit_: false,
            is_last_non_self_collapsing_child_determined_: false,
        };
        algorithm
            .base
            .container_builder_
            .SetExclusionSpace(params.space.GetExclusionSpace());
        algorithm.child_percentage_size_ = CalculateChildPercentageSize(
            algorithm.GetConstraintSpace(),
            algorithm.Node(),
            *algorithm.ChildAvailableSize(),
        );
        algorithm.replaced_child_percentage_size_ = CalculateReplacedChildPercentageSize(
            algorithm.GetConstraintSpace(),
            algorithm.Node(),
            *algorithm.ChildAvailableSize(),
            algorithm.BorderScrollbarPadding(),
            algorithm.BorderPadding(),
        );
        let marker_node = algorithm.Node().ListMarkerBlockNodeIfListItem();
        if marker_node.is_non_null()
            && algorithm.ShouldPlaceUnpositionedListMarker()
            && !marker_node.ListMarkerOccupiesWholeLine()
            && (algorithm.GetBreakToken().is_null()
                || unsafe { &*algorithm.GetBreakToken() }.HasUnpositionedListMarker())
        {
            algorithm
                .base
                .container_builder_
                .SetUnpositionedListMarker(&UnpositionedListMarker::new(&marker_node));
        }
        algorithm.base.container_builder_.SetInitialTextBoxTrim();
        algorithm
    }

    // cpp: layoutng_block/block_layout_algorithm.h:170
    // cpp: layoutng_block/block_layout_algorithm.cc:338-401
    pub fn SetupRelayoutData(&mut self, previous: &Self, relayout_type: RelayoutType) {
        self.base.SetupRelayoutData(&previous.base, relayout_type);
        self.column_spanner_path_ = previous.column_spanner_path_;

        if relayout_type == RelayoutType::kRelayoutIgnoringLineClamp {
            self.line_clamp_data_.data.state = LineClampState::kDisabled;
            self.line_clamp_data_.ignore_line_clamp = true;
        } else if relayout_type == RelayoutType::kRelayoutClampingByLines {
            self.line_clamp_data_.data.state = LineClampState::kClampByLines;
            self.line_clamp_data_.data.lines_until_clamp =
                previous.line_clamp_data_.data.lines_until_clamp;
            self.line_clamp_data_.initial_lines_until_clamp =
                self.line_clamp_data_.data.lines_until_clamp;
            self.line_clamp_data_.data.block_ellipsis =
                previous.line_clamp_data_.data.block_ellipsis;
        } else if relayout_type == RelayoutType::kRelayoutClampingAfterLayoutObject {
            self.line_clamp_data_.data.state = LineClampState::kClampAfterLayoutObject;
            self.line_clamp_data_.data.clamp_after_layout_object =
                previous.line_clamp_data_.last_layout_object;
            self.line_clamp_data_.data.block_ellipsis =
                previous.line_clamp_data_.data.block_ellipsis;
        } else if previous.line_clamp_data_.data.IsClampByLines() {
            self.line_clamp_data_.data.state = LineClampState::kClampByLines;
            self.line_clamp_data_.data.lines_until_clamp =
                previous.line_clamp_data_.initial_lines_until_clamp;
            self.line_clamp_data_.initial_lines_until_clamp =
                self.line_clamp_data_.data.lines_until_clamp;
            self.line_clamp_data_.data.block_ellipsis =
                previous.line_clamp_data_.data.block_ellipsis;
        }

        if relayout_type == RelayoutType::kRelayoutForTextBoxTrim {
            debug_assert!(previous.last_non_empty_inflow_child_.is_non_null());
            self.override_text_box_trim_end_child_ = previous.last_non_empty_inflow_child_.clone();
            self.override_text_box_trim_end_break_token_ = previous.last_non_empty_break_token_;
        } else {
            self.override_text_box_trim_end_child_ =
                if previous.override_text_box_trim_end_child_.is_non_null() {
                    previous.override_text_box_trim_end_child_.clone()
                } else {
                    InlineNode::null()
                };
            self.override_text_box_trim_end_break_token_ =
                previous.override_text_box_trim_end_break_token_;
            self.base.container_builder_.SetShouldTextBoxTrimNodeEnd(
                previous.base.container_builder_.ShouldTextBoxTrimNodeEnd(),
            );
            if self.base.relayout_mode_ & RelayoutType::kRelayoutForTextBoxTrim as i32 != 0 {
                self.override_text_box_trim_end_child_ =
                    if previous.override_text_box_trim_end_child_.is_non_null() {
                        previous.override_text_box_trim_end_child_.clone()
                    } else {
                        InlineNode::null()
                    };
                self.override_text_box_trim_end_break_token_ =
                    previous.override_text_box_trim_end_break_token_;
            }
        }

        self.last_non_self_collapsing_child_ = previous.last_non_self_collapsing_child_.clone();
        self.is_last_non_self_collapsing_child_determined_ = true;
        if relayout_type == RelayoutType::kRelayoutForMarginTrim {
            self.pending_margin_end_trim_child_ = self.last_non_self_collapsing_child_.clone();
            self.is_relayout_for_margin_end_trim_ = true;
        }
    }

    // cpp: layoutng_block/block_layout_algorithm.h:171-171
    // cpp: layoutng_block/block_layout_algorithm.cc:403-405
    pub fn SetBoxType(&mut self, type_: BoxType) {
        self.base.container_builder_.SetBoxType(type_);
    }

    // cpp: layoutng_block/block_layout_algorithm.h:173
    // cpp: layoutng_block/block_layout_algorithm.cc:407-588
    pub fn ComputeMinMaxSizes(&self, float_input: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        if let Some(result) =
            CalculateMinMaxSizesIgnoringChildren(self.Node(), self.BorderScrollbarPadding())
        {
            return result;
        }

        let mut sizes = MinMaxSizes::default();
        let mut depends_on_block_constraints = false;
        let direction = self.Style().Direction();
        let mut float_left_inline_size = float_input.float_left_inline_size;
        let mut float_right_inline_size = float_input.float_right_inline_size;

        let mut child = self.Node().FirstChild();
        while child.is_non_null() {
            if child.IsOutOfFlowPositioned()
                || (child.IsColumnSpanAll() && self.GetConstraintSpace().IsInColumnBfc())
            {
                child = child.NextSibling();
                continue;
            }
            if child.IsTextControlPlaceholder()
                && self.Style().ApplyControlFixedSize(self.Node().GetDOMNode())
            {
                child = child.NextSibling();
                continue;
            }

            let child_style = child.Style();
            let child_clear = child_style.ClearWithContainingStyle(self.Style());
            let child_is_new_fc = child.CreatesNewFormattingContext();
            if child.IsFloating() || child_is_new_fc {
                let float_inline_size = float_left_inline_size + float_right_inline_size;
                if child_clear != EClear::kNone {
                    sizes.max_size = sizes.max_size.max(float_inline_size);
                }
                if matches!(child_clear, EClear::kBoth | EClear::kLeft) {
                    float_left_inline_size = LayoutUnit::default();
                }
                if matches!(child_clear, EClear::kBoth | EClear::kRight) {
                    float_right_inline_size = LayoutUnit::default();
                }
            }

            let mut child_float_input = MinMaxSizesFloatInput::default();
            if child.IsInline() || child.IsAnonymousBlockFlow() {
                child_float_input.float_left_inline_size = float_left_inline_size;
                child_float_input.float_right_inline_size = float_right_inline_size;
            }
            let mut builder = MinMaxConstraintSpaceBuilder::new(
                self.GetConstraintSpace(),
                self.Style(),
                &child,
                child_is_new_fc,
            );
            builder.SetAvailableBlockSize(self.ChildAvailableSize().block_size);
            builder
                .SetPercentageResolutionBlockSize(self.PercentageSizeForChild(&child).block_size);
            if (child.IsAnonymousBlockFlow() || child.IsInline())
                && self.replaced_child_percentage_size_.block_size
                    != self.child_percentage_size_.block_size
            {
                builder.SetReplacedChildPercentageResolutionBlockSize(
                    self.replaced_child_percentage_size_.block_size,
                );
            }
            let space = builder.ToConstraintSpace();
            let child_result = if child.IsInline() {
                if child.Style().IsInShrinkToFitSubtree()
                    && self.GetConstraintSpace().AvailableSize().inline_size != kIndefiniteSize
                {
                    child_float_input.constrained_inline_size =
                        (self.GetConstraintSpace().AvailableSize().inline_size
                            - self.BorderScrollbarPadding().InlineSum())
                        .ClampNegativeToZero();
                }
                let algorithms = LayoutPassScope::Algorithms();
                let callback = if algorithms.is_null() {
                    None
                } else {
                    unsafe { &*algorithms }.inline_support.measure
                };
                let callback = callback.unwrap_or_else(|| {
                    std::panic::panic_any(UnsupportedLayout::new(
                        "inline layout module is not installed",
                    ))
                });
                callback(
                    &InlineNode::from(child.clone()),
                    self.Style().GetWritingMode(),
                    &space,
                    &child_float_input,
                )
            } else {
                ComputeMinAndMaxContentContribution(
                    self.Style(),
                    &BlockNode::from(child.clone()),
                    &space,
                    child_float_input,
                )
            };
            debug_assert!(child_result.sizes.min_size <= child_result.sizes.max_size);

            let margins = if child.IsInline() {
                BoxStrut::default()
            } else {
                ComputeMarginsFor(&space, child_style, self.GetConstraintSpace())
            };
            let max_inline_contribution = if child.IsFloating() {
                let float_inline_size = child_result.sizes.max_size + margins.InlineSum();
                if float_inline_size > LayoutUnit::default() {
                    if child_style.FloatingWithContainingStyle(self.Style()) == EFloat::kLeft {
                        float_left_inline_size += float_inline_size;
                    } else {
                        float_right_inline_size += float_inline_size;
                    }
                }
                float_left_inline_size + float_right_inline_size
            } else if child_is_new_fc {
                let margin_line_left = margins.LineLeft(direction);
                let margin_line_right = margins.LineRight(direction);
                let line_left_inset = if margin_line_left > LayoutUnit::default() {
                    float_left_inline_size.max(margin_line_left)
                } else {
                    float_left_inline_size + margin_line_left
                };
                let line_right_inset = if margin_line_right > LayoutUnit::default() {
                    float_right_inline_size.max(margin_line_right)
                } else {
                    float_right_inline_size + margin_line_right
                };
                child_result.sizes.max_size + (line_left_inset + line_right_inset)
            } else {
                child_result.sizes.max_size + margins.InlineSum()
            };
            sizes.max_size = sizes.max_size.max(max_inline_contribution);
            let min_inline_contribution = child_result.sizes.min_size + margins.InlineSum();
            sizes.min_size = sizes.min_size.max(min_inline_contribution);
            depends_on_block_constraints |= child_result.depends_on_block_constraints;
            if !child.IsFloating() {
                float_left_inline_size = LayoutUnit::default();
                float_right_inline_size = LayoutUnit::default();
            }
            child = child.NextSibling();
        }
        debug_assert!(sizes.min_size >= LayoutUnit::default());
        debug_assert!(sizes.min_size <= sizes.max_size);
        sizes += self.BorderScrollbarPadding().InlineSum();
        MinMaxSizesResult::new(sizes, depends_on_block_constraints)
    }

    // cpp: layoutng_block/block_layout_algorithm.h:197
    // cpp: layoutng_block/block_layout_algorithm.cc:686-689
    pub fn RelayoutIgnoringLineClamp(&self) -> *const LayoutResult {
        debug_assert!(self.line_clamp_data_.data.IsClampByLines());
        self.base
            .RelayoutDefault::<Self>(RelayoutType::kRelayoutIgnoringLineClamp)
    }

    // cpp: layoutng_block/block_layout_algorithm.h:198
    // cpp: layoutng_block/block_layout_algorithm.cc:691-696
    pub fn RelayoutClampingByLines(&mut self, lines_until_clamp: i32) -> *const LayoutResult {
        debug_assert!(self.line_clamp_data_.data.IsMeasureUntilBfcOffset());
        self.line_clamp_data_.data.lines_until_clamp = lines_until_clamp.max(0);
        self.base
            .RelayoutDefault::<Self>(RelayoutType::kRelayoutClampingByLines)
    }

    // cpp: layoutng_block/block_layout_algorithm.h:199-200
    // cpp: layoutng_block/block_layout_algorithm.cc:698-705
    pub fn RelayoutClampingAfterLayoutObject(
        &mut self,
        layout_object: *const LayoutObject,
    ) -> *const LayoutResult {
        debug_assert!(self.line_clamp_data_.data.IsMeasureUntilBfcOffset());
        debug_assert!(!layout_object.is_null());
        self.line_clamp_data_.last_layout_object = layout_object;
        self.base
            .RelayoutDefault::<Self>(RelayoutType::kRelayoutClampingAfterLayoutObject)
    }

    // cpp: layoutng_block/block_layout_algorithm.h:201
    // cpp: layoutng_block/block_layout_algorithm.cc:707-710
    pub fn RelayoutForTextBoxTrimEnd(&self) -> *const LayoutResult {
        debug_assert!(self.last_non_empty_inflow_child_.is_non_null());
        self.base
            .RelayoutDefault::<Self>(RelayoutType::kRelayoutForTextBoxTrim)
    }

    // cpp: layoutng_block/block_layout_algorithm.h:202
    // cpp: layoutng_block/block_layout_algorithm.cc:712-714
    pub fn RelayoutForMarginTrimEnd(&self) -> *const LayoutResult {
        self.base
            .RelayoutDefault::<Self>(RelayoutType::kRelayoutForMarginTrim)
    }

    // cpp: layoutng_block/block_layout_algorithm.h:174
    // cpp: layoutng_block/block_layout_algorithm.cc:613-635
    pub fn Layout(&mut self) -> *const LayoutResult {
        let mut inline_child = InlineNode::null();
        let result = if self.Node().IsInlineFormattingContextRoot(&mut inline_child) {
            let algorithms = LayoutPassScope::Algorithms();
            let callback = if algorithms.is_null() {
                None
            } else {
                unsafe { &*algorithms }.inline_support.layout_block_child
            };
            let callback = callback.unwrap_or_else(|| {
                std::panic::panic_any(UnsupportedLayout::new(
                    "inline layout module is not installed",
                ))
            });
            callback(
                self as *mut Self as *mut ForwardBlockLayoutAlgorithm,
                &inline_child,
            )
        } else {
            self.LayoutWithInlineChildContext(std::ptr::null_mut())
        };
        if unsafe { &*result }.Status() == EStatus::kSuccess {
            return result;
        }
        self.HandleNonsuccessfulLayoutResult(result)
    }

    // C++ overloads Layout; Rust names the inline-context variant explicitly.
    // cpp: layoutng_block/block_layout_algorithm.h:204-205
    // cpp: layoutng_block/block_layout_algorithm.cc:716-1074
    pub fn LayoutWithInlineChildContext(
        &mut self,
        inline_child_layout_context: *mut InlineChildLayoutContext,
    ) -> *const LayoutResult {
        debug_assert_eq!(
            !inline_child_layout_context.is_null(),
            self.Node()
                .IsInlineFormattingContextRoot(std::ptr::null_mut())
        );
        self.base
            .container_builder_
            .SetIsInlineFormattingContext(!inline_child_layout_context.is_null());
        if !self.column_spanner_path_.is_null() {
            self.base
                .container_builder_
                .SetShouldForceSameFragmentationFlow();
        }

        // ConstraintSpace is owned outside this algorithm. Keep its C++ shared
        // reference live while the builder and other algorithm fields mutate.
        let constraint_space = self.GetConstraintSpace() as *const ConstraintSpace;
        let constraint_space = unsafe { &*constraint_space };
        self.base
            .container_builder_
            .SetBfcLineOffset(constraint_space.GetBfcOffset().line_offset);
        let adjoining_object_types = constraint_space.GetAdjoiningObjectTypes();
        if adjoining_object_types != 0 {
            debug_assert!(!constraint_space.IsNewFormattingContext());
            debug_assert!(self.base.container_builder_.BfcBlockOffset().is_none());
            self.abort_when_bfc_block_offset_updated_ = true;
            self.base
                .container_builder_
                .SetAdjoiningObjectTypes(adjoining_object_types);
        } else if constraint_space.HasBlockFragmentation() {
            self.abort_when_bfc_block_offset_updated_ = true;
        }

        self.line_clamp_data_
            .Setup(&self.base.node_, &self.base.container_builder_);
        let content_edge = self.BorderScrollbarPadding().block_start;
        let mut previous_inflow_position = PreviousInflowPosition {
            logical_block_offset: LayoutUnit::default(),
            margin_strut: constraint_space.GetMarginStrut(),
            block_end_annotation_space: if self.is_resuming_ {
                LayoutUnit::default()
            } else {
                self.ComputeInitialBlockStartAnnotationSpace()
            },
            previous_sibling_block_end_annotation_space: LayoutUnit::default(),
            self_collapsing_child_had_clearance: false,
        };
        let break_token = self.GetBreakToken();
        if !break_token.is_null() {
            let token = unsafe { &*break_token };
            if IsBreakInside(break_token)
                && !token.IsForcedBreak()
                && !token.IsCausedByColumnSpanner()
            {
                previous_inflow_position.margin_strut.discard_margins = true;
            }
            if token.MonolithicOverflow() != LayoutUnit::default() {
                self.has_break_opportunity_before_next_child_ = true;
            }
        }

        if content_edge != LayoutUnit::default()
            || self.is_resuming_
            || constraint_space.IsNewFormattingContext()
        {
            let discard_subsequent_margins = previous_inflow_position.margin_strut.discard_margins
                && content_edge == LayoutUnit::default();
            if !self.ResolveBfcBlockOffset(&mut previous_inflow_position) {
                debug_assert!(!constraint_space.IsNewFormattingContext());
                debug_assert!(!self.is_resuming_);
                return self
                    .base
                    .container_builder_
                    .Abort(EStatus::kBfcBlockOffsetResolved);
            }
            previous_inflow_position.logical_block_offset = content_edge;
            if discard_subsequent_margins {
                previous_inflow_position.margin_strut.discard_margins = true;
            }
        }

        let margin_trim = self.Style().MarginTrim();
        if margin_trim & EMarginTrim::kMarginTrimBlock.value() as u32 != 0 {
            self.incoming_margin_strut_ = previous_inflow_position.margin_strut.clone();
            if margin_trim & EMarginTrim::kMarginTrimBlockStart.value() as u32 != 0 {
                previous_inflow_position.margin_strut.trim_leading_margins = true;
            }
        }

        if constraint_space.IsNewFormattingContext() {
            debug_assert_eq!(
                *self
                    .base
                    .container_builder_
                    .BfcBlockOffset()
                    .as_ref()
                    .unwrap(),
                LayoutUnit::default()
            );
        }
        if constraint_space.IsNewFormattingContext() || self.is_resuming_ {
            debug_assert!(constraint_space.GetMarginStrut().IsEmpty());
        }
        if self.base.container_builder_.BfcBlockOffset().is_none() {
            debug_assert!(!previous_inflow_position.self_collapsing_child_had_clearance);
            debug_assert!(!constraint_space.IsNewFormattingContext());
        }

        if constraint_space.IsNewFormattingContext() && self.line_clamp_data_.IsPastClampPoint() {
            self.line_clamp_data_.previous_inflow_position_when_clamped =
                Some(previous_inflow_position.clone());
        }
        if self.Node().IsQuirkyContainer() {
            previous_inflow_position
                .margin_strut
                .is_quirky_container_start = true;
        }

        let mut previous_inline_break_token: *const InlineBreakToken = std::ptr::null();
        let first_child = self.Node().FirstChild();
        let mut child_iterator = crate::block_child_iterator::BlockChildIterator::new(
            first_child.clone(),
            break_token,
            false,
        );
        if self.Node().ChildLayoutBlockedByDisplayLock() {
            child_iterator = crate::block_child_iterator::BlockChildIterator::new(
                LayoutInputNode::null(),
                std::ptr::null(),
                false,
            );
        }
        let mut placeholder_child = BlockNode::null();
        let mut entry = child_iterator.NextChild(std::ptr::null());
        while !entry.AtEnd() {
            let child_break_token = entry.token;
            debug_assert!(
                entry.block_node.is_non_null()
                    || child_break_token.is_null()
                    || unsafe { &*child_break_token }.IsInlineType()
            );
            debug_assert!(entry.block_node.is_non_null() || first_child.IsInline());
            let child = if entry.block_node.is_non_null() {
                entry.block_node.base
            } else {
                first_child.clone()
            };

            if child.IsOutOfFlowPositioned() {
                self.HandleOutOfFlowPositioned(
                    &previous_inflow_position,
                    &BlockNode::from(child.clone()),
                    To::<BlockBreakToken>(child_break_token),
                );
            } else if child.IsFloating() {
                self.HandleFloat(
                    &previous_inflow_position,
                    BlockNode::from(child.clone()),
                    To::<BlockBreakToken>(child_break_token),
                );
            } else if child.IsListMarker() && !child.ListMarkerOccupiesWholeLine() {
                // The outside marker was already recorded by the constructor.
            } else if child.IsColumnSpanAll()
                && constraint_space.IsInColumnBfc()
                && constraint_space.HasBlockFragmentation()
            {
                debug_assert!(!self.base.container_builder_.DidBreakSelf());
                debug_assert!(!self.base.container_builder_.FoundColumnSpanner());
                debug_assert!(!IsBreakInside(To::<BlockBreakToken>(child_break_token)));
                if constraint_space.IsPastBreak()
                    || self.base.container_builder_.HasInsertedChildBreak()
                {
                    self.base.container_builder_.AddBreakBeforeChild(
                        child,
                        Some(BreakAppeal::kBreakAppealPerfect),
                        true,
                        LogicalOffset::default(),
                    );
                    self.base.container_builder_.SetHasColumnSpanner();
                    break;
                }
                let child_spanner_path =
                    MakeGarbageCollected(ColumnSpannerPath::new(BlockNode::from(child.clone())));
                let container_spanner_path = MakeGarbageCollected(ColumnSpannerPath::with_child(
                    self.Node().clone(),
                    child_spanner_path,
                ));
                self.base
                    .container_builder_
                    .SetColumnSpannerPath(unsafe { &*container_spanner_path });
                self.base.container_builder_.SetIsEmptySpannerParent(
                    self.base.container_builder_.Children().is_empty() && self.is_resuming_,
                );
                loop {
                    entry = child_iterator.NextChild(std::ptr::null());
                    if !entry.block_node.is_non_null() {
                        break;
                    }
                    debug_assert!(entry.token.is_null());
                    let sibling = entry.block_node.base;
                    if sibling.IsColumnSpanAll() {
                        continue;
                    }
                    self.base.container_builder_.AddBreakBeforeChild(
                        sibling,
                        Some(BreakAppeal::kBreakAppealPerfect),
                        true,
                        LogicalOffset::default(),
                    );
                    break;
                }
                break;
            } else if child.IsTextControlPlaceholder() {
                placeholder_child = BlockNode::from(child.clone());
            } else {
                if !self.base.early_break_.is_null()
                    && IsEarlyBreakTarget(
                        unsafe { &*self.base.early_break_ },
                        &self.base.container_builder_,
                        &child,
                    )
                {
                    if !self.ResolveBfcBlockOffset(&mut previous_inflow_position) {
                        return self
                            .base
                            .container_builder_
                            .Abort(EStatus::kBfcBlockOffsetResolved);
                    }
                    self.base.container_builder_.AddBreakBeforeChild(
                        child,
                        Some(BreakAppeal::kBreakAppealPerfect),
                        false,
                        LogicalOffset::default(),
                    );
                    self.ConsumeRemainingFragmentainerSpace(&mut previous_inflow_position);
                    break;
                }
                let status = if child.CreatesNewFormattingContext() {
                    let status = self.HandleNewFormattingContext(
                        child,
                        To::<BlockBreakToken>(child_break_token),
                        &mut previous_inflow_position,
                    );
                    previous_inline_break_token = std::ptr::null();
                    status
                } else {
                    self.HandleInflow(
                        child,
                        child_break_token,
                        &mut previous_inflow_position,
                        inline_child_layout_context,
                        &mut previous_inline_break_token,
                    )
                };
                if status != EStatus::kSuccess {
                    return self.base.container_builder_.Abort(status);
                }
                if constraint_space.HasBlockFragmentation()
                    && self.base.container_builder_.HasInflowChildBreakInside()
                {
                    break;
                }
            }
            entry = child_iterator.NextChild(previous_inline_break_token);
        }

        if !self.base.container_builder_.FoundColumnSpanner()
            && !self
                .base
                .container_builder_
                .ShouldForceSameFragmentationFlow()
        {
            let inline_token = DynamicTo::<InlineBreakToken>(entry.token);
            if !inline_token.is_null() {
                debug_assert!(!unsafe { &*inline_token }.IsInParallelBlockFlow());
            } else {
                let block_token = DynamicTo::<BlockBreakToken>(entry.token);
                if !block_token.is_null() {
                    debug_assert!(!unsafe { &*block_token }.IsAtBlockEnd());
                }
            }
        }
        if placeholder_child.is_non_null() {
            let offset_and_status =
                self.HandleTextControlPlaceholder(placeholder_child, &previous_inflow_position);
            if offset_and_status.status != EStatus::kSuccess {
                return self.base.container_builder_.Abort(offset_and_status.status);
            }
            previous_inflow_position.logical_block_offset = offset_and_status.logical_block_offset;
        }
        if child_iterator
            .NextChild(previous_inline_break_token)
            .AtEnd()
        {
            self.base.container_builder_.SetHasSeenAllChildren();
        }
        self.intrinsic_block_size_ = content_edge;
        self.FinishLayout(&mut previous_inflow_position, inline_child_layout_context)
    }

    // cpp: layoutng_block/block_layout_algorithm.h:207-208
    // cpp: layoutng_block/block_layout_algorithm.cc:1076-1456
    pub fn FinishLayout(
        &mut self,
        previous_inflow_position: &mut PreviousInflowPosition,
        _inline_child_layout_context: *mut InlineChildLayoutContext,
    ) -> *const LayoutResult {
        let constraint_space = self.GetConstraintSpace() as *const ConstraintSpace;
        let constraint_space = unsafe { &*constraint_space };
        if constraint_space.IsNewFormattingContext()
            && self.line_clamp_data_.ShouldRelayoutWithNoForcedTruncate()
        {
            return self
                .base
                .container_builder_
                .Abort(EStatus::kNeedsLineClampRelayout);
        }
        if self.base.container_builder_.ShouldTextBoxTrimEnd()
            && self.last_non_empty_inflow_child_.is_non_null()
            && self
                .line_clamp_data_
                .previous_inflow_position_when_clamped
                .is_none()
        {
            return self
                .base
                .container_builder_
                .Abort(EStatus::kTextBoxTrimEndDidNotApply);
        }

        // C++ rebinds this pointer to the saved clamp point; later mutations
        // must update that same record rather than a copy of it.
        let position = if RuntimeEnabledFeatures::CSSLineClampEnabled()
            && self
                .line_clamp_data_
                .previous_inflow_position_when_clamped
                .is_some()
        {
            self.line_clamp_data_
                .previous_inflow_position_when_clamped
                .as_mut()
                .unwrap() as *mut PreviousInflowPosition
        } else {
            previous_inflow_position as *mut PreviousInflowPosition
        };
        let previous = unsafe { &mut *position };
        let mut border_box_size = *self.base.container_builder_.InitialBorderBoxSize();
        let mut end_margin_strut = previous.margin_strut.clone();

        if self.base.container_builder_.HasSeenAllChildren()
            && HasLineEvenIfEmpty(self.Node().GetLayoutBox())
        {
            self.intrinsic_block_size_ = self.intrinsic_block_size_.max(
                self.BorderScrollbarPadding().block_start
                    + self.Node().EmptyLineBlockSize(self.GetBreakToken()),
            );
            if constraint_space.IsInitialColumnBalancingPass() {
                self.base
                    .container_builder_
                    .PropagateTallestUnbreakableBlockSize(self.intrinsic_block_size_);
            }
            if let Some(baseline_offset) = self.BaselineForEmptyLine() {
                self.base.container_builder_.SetBaselines(baseline_offset);
            }
        }
        if previous.block_end_annotation_space < LayoutUnit::default() {
            let overflow = -previous.block_end_annotation_space;
            previous.logical_block_offset -= self
                .base
                .container_builder_
                .Padding()
                .block_end
                .min(overflow);
        }

        let margin_trim = self.Style().MarginTrim();
        if margin_trim & EMarginTrim::kMarginTrimBlock.value() as u32 != 0 {
            let is_self_collapsing = !self.last_non_self_collapsing_child_.is_non_null();
            let mut trimmed = MarginStrut::default();
            if is_self_collapsing {
                trimmed = self.incoming_margin_strut_.clone();
            }
            if margin_trim & EMarginTrim::kMarginTrimBlockEnd.value() as u32 != 0 {
                if !self.is_relayout_for_margin_end_trim_ && !end_margin_strut.IsEmpty() {
                    return self
                        .base
                        .container_builder_
                        .Abort(EStatus::kMarginTrimEndDidNotApply);
                }
                end_margin_strut = trimmed;
            } else if is_self_collapsing {
                debug_assert!(margin_trim & EMarginTrim::kMarginTrimBlockStart.value() as u32 != 0);
                end_margin_strut = trimmed;
            }
        }

        if !RuntimeEnabledFeatures::CSSLineClampEnabled()
            && self
                .line_clamp_data_
                .previous_inflow_position_when_clamped
                .is_some()
        {
            debug_assert!(self.base.container_builder_.BfcBlockOffset().is_some());
            self.intrinsic_block_size_ = self
                .line_clamp_data_
                .previous_inflow_position_when_clamped
                .as_ref()
                .unwrap()
                .logical_block_offset
                + self.BorderScrollbarPadding().block_end;
            end_margin_strut = MarginStrut::default();
        } else if self.BorderScrollbarPadding().block_end != LayoutUnit::default()
            || previous.self_collapsing_child_had_clearance
            || constraint_space.IsNewFormattingContext()
        {
            if constraint_space.IsNewFormattingContext()
                && self
                    .line_clamp_data_
                    .previous_inflow_position_when_clamped
                    .is_none()
            {
                let clearance = self
                    .base
                    .container_builder_
                    .GetExclusionSpace()
                    .ClearanceOffsetIncludingInitialLetter(EClear::kBoth);
                self.intrinsic_block_size_ = self.intrinsic_block_size_.max(clearance);
            }
            if self.base.container_builder_.BfcBlockOffset().is_none() {
                debug_assert!(!constraint_space.IsNewFormattingContext());
                if !self.ResolveBfcBlockOffset(previous) {
                    return self
                        .base
                        .container_builder_
                        .Abort(EStatus::kBfcBlockOffsetResolved);
                }
                debug_assert!(self.base.container_builder_.BfcBlockOffset().is_some());
            } else {
                let mut margin_sum = if self.Node().IsQuirkyContainer() {
                    end_margin_strut.QuirkyContainerSum()
                } else {
                    end_margin_strut.Sum()
                };
                if constraint_space.HasKnownFragmentainerBlockSize() {
                    let adjusted = AdjustedMarginAfterFinalChildFragment(
                        &self.base.container_builder_,
                        previous.logical_block_offset,
                        margin_sum,
                    );
                    if adjusted != margin_sum {
                        self.base
                            .container_builder_
                            .SetIsTruncatedByFragmentationLine();
                        margin_sum = adjusted;
                    }
                }
                self.intrinsic_block_size_ = self
                    .intrinsic_block_size_
                    .max(previous.logical_block_offset + margin_sum);
            }
            if !ShouldIncludeBlockEndBorderPadding(&self.base.container_builder_) {
                self.base
                    .container_builder_
                    .ClearBorderScrollbarPaddingBlockEnd();
            }
            self.intrinsic_block_size_ += self.BorderScrollbarPadding().block_end;
            end_margin_strut = MarginStrut::default();
        } else {
            self.intrinsic_block_size_ = self
                .intrinsic_block_size_
                .max(previous.logical_block_offset);
            if constraint_space.ShouldForceMarginTrimEnd()
                && !self.is_relayout_for_margin_end_trim_
                && !end_margin_strut.IsEmpty()
            {
                return self
                    .base
                    .container_builder_
                    .Abort(EStatus::kMarginTrimEndDidNotApply);
            }
        }

        let unconstrained_intrinsic_block_size = self.intrinsic_block_size_;
        self.intrinsic_block_size_ = ClampIntrinsicBlockSize(
            constraint_space,
            self.Node(),
            self.GetBreakToken(),
            self.BorderScrollbarPadding(),
            self.intrinsic_block_size_,
            self.CalculateQuirkyBodyMarginBlockSum(&end_margin_strut),
        );
        let mut previously_consumed_block_size = LayoutUnit::default();
        if !self.GetBreakToken().is_null() && !self.base.container_builder_.IsFragmentainerBoxType()
        {
            previously_consumed_block_size = unsafe { &*self.GetBreakToken() }.ConsumedBlockSize();
        }
        border_box_size.block_size = ComputeBlockSizeForFragment(
            constraint_space,
            self.Node(),
            self.BorderPadding(),
            previously_consumed_block_size + self.intrinsic_block_size_,
            border_box_size.inline_size,
            kIndefiniteSize,
        );
        self.base
            .container_builder_
            .SetFragmentsTotalBlockSize(border_box_size.block_size);

        if RuntimeEnabledFeatures::AnnotationSpaceOnStartEnabled()
            && constraint_space.ContainsAnnotations()
            && !constraint_space.IsNewFormattingContext()
            && previous.block_end_annotation_space > LayoutUnit::default()
            && self.Borders().block_end == LayoutUnit::default()
        {
            let content_end_offset =
                self.BorderScrollbarPadding().block_start + previous.logical_block_offset;
            let annotation_start = content_end_offset - previous.block_end_annotation_space;
            let container_end =
                border_box_size.block_size - self.Borders().block_end - self.Scrollbar().block_end;
            self.base.container_builder_.SetBlockEndAnnotationSpace(
                (container_end - annotation_start).max(LayoutUnit::default()),
            );
        }
        if self.base.container_builder_.BfcBlockOffset().is_none()
            && (border_box_size.block_size != LayoutUnit::default()
                || !self.GetBreakToken().is_null()
                || self.base.container_builder_.FoundColumnSpanner())
        {
            if !self.ResolveBfcBlockOffset(previous) {
                return self
                    .base
                    .container_builder_
                    .Abort(EStatus::kBfcBlockOffsetResolved);
            }
            debug_assert!(self.base.container_builder_.BfcBlockOffset().is_some());
        }
        if self.base.container_builder_.BfcBlockOffset().is_some() {
            let stitched_intrinsic_block_size =
                previously_consumed_block_size + self.intrinsic_block_size_;
            if border_box_size.block_size != stitched_intrinsic_block_size
                || ComputeInitialBlockSizeForFragment(
                    constraint_space,
                    self.Node(),
                    self.BorderPadding(),
                    kIndefiniteSize,
                    border_box_size.inline_size,
                    kIndefiniteSize,
                ) != kIndefiniteSize
            {
                end_margin_strut = MarginStrut::default();
            }
        }
        if self
            .base
            .container_builder_
            .GetUnpositionedListMarker()
            .is_present()
            && self.ShouldPlaceUnpositionedListMarker()
            && !self.base.container_builder_.HasInflowChildBreakInside()
            && !self.PositionListMarkerWithoutLineBoxes(previous)
        {
            return self
                .base
                .container_builder_
                .Abort(EStatus::kBfcBlockOffsetResolved);
        }
        self.base
            .container_builder_
            .SetEndMarginStrut(&end_margin_strut);
        self.base
            .container_builder_
            .SetIntrinsicBlockSize(self.intrinsic_block_size_);
        if self.base.container_builder_.BfcBlockOffset().is_some() {
            self.base.container_builder_.ResetAdjoiningObjectTypes();
        } else {
            self.base.container_builder_.SetIsSelfCollapsing();
            if let Some(forced_offset) = constraint_space.ForcedBfcBlockOffset() {
                self.base
                    .container_builder_
                    .SetBfcBlockOffset(forced_offset);
                if constraint_space.IsPushedByFloats() {
                    self.base.container_builder_.SetIsPushedByFloats();
                }
            }
        }

        if InvolvedInBlockFragmentationForBuilder(&self.base.container_builder_) {
            let status = self.FinalizeForFragmentation();
            if status != BreakStatus::kContinue {
                if status == BreakStatus::kNeedsEarlierBreak {
                    return self
                        .base
                        .container_builder_
                        .Abort(EStatus::kNeedsEarlierBreak);
                }
                debug_assert_eq!(status, BreakStatus::kDisableFragmentation);
                return self
                    .base
                    .container_builder_
                    .Abort(EStatus::kDisableFragmentation);
            }
            self.intrinsic_block_size_ = self.base.container_builder_.IntrinsicBlockSize();
        } else {
            #[cfg(debug_assertions)]
            self.base.container_builder_.CheckNoBlockFragmentation();
        }
        if constraint_space.IsTableCell() {
            let algorithms = LayoutPassScope::Algorithms();
            let callback = if algorithms.is_null() {
                None
            } else {
                unsafe { &*algorithms }.table_support.finalize_cell
            };
            let callback = callback.unwrap_or_else(|| {
                std::panic::panic_any(UnsupportedLayout::new(
                    "table cell finalization module is not installed",
                ))
            });
            callback(
                self.intrinsic_block_size_,
                &mut self.base.container_builder_,
            );
        } else {
            let style = self.Style() as *const ComputedStyle;
            AlignBlockContent(
                unsafe { &*style },
                self.GetBreakToken(),
                unconstrained_intrinsic_block_size,
                &mut self.base.container_builder_,
            );
        }
        self.base
            .container_builder_
            .HandleOofsAndSpecialDescendants();
        if constraint_space.GetBaselineAlgorithmType() == BaselineAlgorithmType::kInlineBlock {
            self.base
                .container_builder_
                .SetUseLastBaselineForInlineBaseline();
        }
        if constraint_space.IsNewFormattingContext() {
            self.base
                .container_builder_
                .SetExclusionSpace(&ExclusionSpace::default());
        } else {
            self.base
                .container_builder_
                .SetLinesUntilClamp(self.line_clamp_data_.data.LinesUntilClamp(true));
            self.base
                .container_builder_
                .SetLineClampAfterLayoutObject(self.line_clamp_data_.last_layout_object);
        }
        if constraint_space.UseFirstLineStyle() {
            self.base
                .container_builder_
                .SetStyleVariant(StyleVariant::kFirstLine);
        }
        self.base.container_builder_.ToBoxFragment()
    }

    // cpp: layoutng_block/block_layout_algorithm.h:291-293
    // cpp: layoutng_block/block_layout_algorithm.cc:1458-1565
    pub fn HandleOutOfFlowPositioned(
        &mut self,
        previous: &PreviousInflowPosition,
        child: &BlockNode,
        child_break_token: *const BlockBreakToken,
    ) {
        if !RuntimeEnabledFeatures::FragmentedOofInCbEnabled() {
            debug_assert!(
                child_break_token.is_null() || unsafe { &*child_break_token }.IsBreakBefore()
            );
        }
        if self.GetConstraintSpace().HasBlockFragmentation() {
            let break_between = self
                .base
                .container_builder_
                .JoinedBreakBetweenValue(EBreakBetween::kAuto);
            if IsForcedBreakValue(self.GetConstraintSpace(), break_between) {
                self.base.container_builder_.AddBreakBeforeChild(
                    child.base.clone(),
                    Some(BreakAppeal::kBreakAppealPerfect),
                    true,
                    LogicalOffset::default(),
                );
                return;
            }
        }
        if !child_break_token.is_null()
            && !unsafe { &*child_break_token }.IsForcedBreak()
            && RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
        {
            self.base
                .container_builder_
                .AddOutOfFlowChildCandidateWithBreakToken(child, unsafe { &*child_break_token });
            return;
        }
        debug_assert!(child.IsOutOfFlowPositioned());
        let mut static_pos = LogicalStaticPosition::from_offset(LogicalOffset::new(
            self.BorderScrollbarPadding().inline_start,
            previous.logical_block_offset,
        ));
        if self.base.container_builder_.BfcBlockOffset().is_some() {
            static_pos.offset.block_offset += previous.margin_strut.Sum();
        }
        if child.Style().IsOriginalDisplayInlineType() {
            if self.base.container_builder_.BfcBlockOffset().is_none() {
                self.base.container_builder_.AddAdjoiningObjectTypes(
                    AdjoiningObjectTypeValue::kAdjoiningInlineOutOfFlow as i32,
                );
                self.abort_when_bfc_block_offset_updated_ = true;
            }
            let origin_bfc_block_offset = (*self.base.container_builder_.BfcBlockOffset())
                .unwrap_or(self.GetConstraintSpace().ExpectedBfcBlockOffset())
                + static_pos.offset.block_offset;
            let origin_bfc_offset = BfcOffset::new(
                self.GetConstraintSpace().GetBfcOffset().line_offset
                    + self
                        .BorderScrollbarPadding()
                        .LineLeft(self.Style().Direction()),
                origin_bfc_block_offset,
            );
            let style = self.Style() as *const ComputedStyle;
            let child_inline_size = self.ChildAvailableSize().inline_size;
            static_pos.offset.inline_offset += CalculateOutOfFlowStaticInlineLevelOffset(
                unsafe { &*style },
                &origin_bfc_offset,
                self.base.container_builder_.GetExclusionSpace(),
                child_inline_size,
            );
        } else {
            let parent_writing_direction = self.GetConstraintSpace().GetWritingDirection();
            static_pos.inline_edge = InlineStaticPositionEdge(
                child,
                self.Style() as *const ComputedStyle,
                parent_writing_direction,
                false,
            );
            static_pos.block_edge =
                BlockStaticPositionEdge(child, std::ptr::null(), parent_writing_direction);
            let available_inline_size = self.ChildAvailableSize().inline_size;
            match static_pos.inline_edge {
                InlineEdge::kInlineCenter => {
                    static_pos.offset.inline_offset += available_inline_size / 2;
                }
                InlineEdge::kInlineEnd => {
                    static_pos.offset.inline_offset += available_inline_size;
                }
                InlineEdge::kInlineStart => {}
            }
        }
        self.base
            .container_builder_
            .AddOutOfFlowChildCandidateDefault(child, &static_pos);
    }

    // cpp: layoutng_block/block_layout_algorithm.h:294-296
    // cpp: layoutng_block/block_layout_algorithm.cc:1567-1668
    pub fn HandleFloat(
        &mut self,
        previous: &PreviousInflowPosition,
        child: BlockNode,
        child_break_token: *const BlockBreakToken,
    ) {
        debug_assert!(
            !IsBreakInside(child_break_token)
                || self.base.container_builder_.BfcBlockOffset().is_some()
        );
        let constraint_space = self.GetConstraintSpace() as *const ConstraintSpace;
        let constraint_space = unsafe { &*constraint_space };
        let mut origin_bfc_offset = BfcOffset::new(
            constraint_space.GetBfcOffset().line_offset
                + self
                    .BorderScrollbarPadding()
                    .LineLeft(constraint_space.Direction()),
            if self.base.container_builder_.BfcBlockOffset().is_some() {
                self.NextBorderEdge(previous)
            } else {
                constraint_space.ExpectedBfcBlockOffset()
            },
        );
        if !child_break_token.is_null() {
            origin_bfc_offset.block_offset += unsafe { &*child_break_token }.MonolithicOverflow()
                - unsafe { &*self.GetBreakToken() }.MonolithicOverflow();
        }
        if constraint_space.HasBlockFragmentation() {
            let break_between = self
                .base
                .container_builder_
                .JoinedBreakBetweenValue(EBreakBetween::kAuto);
            if IsForcedBreakValue(constraint_space, break_between) {
                self.base.container_builder_.AddBreakBeforeChild(
                    child.base,
                    Some(BreakAppeal::kBreakAppealPerfect),
                    true,
                    LogicalOffset::default(),
                );
                return;
            }
        }
        let percentage_size = self.PercentageSizeForChild(&child.base);
        let style = self.Style() as *const ComputedStyle;
        let mut unpositioned_float = UnpositionedFloat::new(
            child,
            child_break_token,
            *self.ChildAvailableSize(),
            percentage_size,
            &origin_bfc_offset,
            constraint_space,
            unsafe { &*style },
            self.FragmentainerCapacityForChildren(),
            self.FragmentainerOffsetForChildren(),
            self.line_clamp_data_.ShouldHideForPaint(),
        );
        if self.base.container_builder_.BfcBlockOffset().is_none() {
            self.base.container_builder_.AddAdjoiningObjectTypes(
                if unpositioned_float.IsLineLeft(constraint_space.Direction()) {
                    AdjoiningObjectTypeValue::kAdjoiningFloatLeft as i32
                } else {
                    AdjoiningObjectTypeValue::kAdjoiningFloatRight as i32
                },
            );
            if constraint_space.ForcedBfcBlockOffset().is_none() {
                self.abort_when_bfc_block_offset_updated_ = true;
            }
        }
        let float_support = LayoutPassScope::RequireFloatSupport();
        let position = unsafe { &*float_support }
            .position
            .expect("float position callback");
        let positioned_float = position(
            &mut unpositioned_float,
            self.base.container_builder_.GetExclusionSpace(),
        );
        if positioned_float.minimum_space_shortage > LayoutUnit::default() {
            self.base
                .container_builder_
                .PropagateSpaceShortage(Some(positioned_float.minimum_space_shortage));
            debug_assert_eq!(
                positioned_float.tallest_unbreakable_block_size,
                LayoutUnit::default()
            );
        } else if positioned_float.tallest_unbreakable_block_size != LayoutUnit::default() {
            self.base
                .container_builder_
                .PropagateTallestUnbreakableBlockSize(
                    positioned_float.tallest_unbreakable_block_size,
                );
        }
        if !positioned_float.break_before_token.Get().is_null() {
            debug_assert!(constraint_space.HasBlockFragmentation());
            self.base.container_builder_.AddBreakToken(
                positioned_float.break_before_token.Get() as *const BreakToken,
                true,
            );
            return;
        }
        let layout_result = unsafe { &*positioned_float.layout_result.Get() };
        debug_assert_eq!(layout_result.Status(), EStatus::kSuccess);
        let physical_fragment = layout_result.GetPhysicalFragment();
        let float_inline_size =
            LogicalFragment::new(constraint_space.GetWritingDirection(), physical_fragment)
                .InlineSize();
        let bfc_offset = BfcOffset::new(
            constraint_space.GetBfcOffset().line_offset,
            (*self.base.container_builder_.BfcBlockOffset())
                .unwrap_or(constraint_space.ExpectedBfcBlockOffset()),
        );
        let logical_offset = LogicalFromBfcOffsets(
            &positioned_float.bfc_offset,
            &bfc_offset,
            float_inline_size,
            self.base.container_builder_.InlineSize(),
            constraint_space.Direction(),
        );
        self.base.container_builder_.AddResult(
            layout_result,
            logical_offset,
            None,
            None,
            std::ptr::null(),
        );
    }

    // cpp: layoutng_block/block_layout_algorithm.h:310-314
    // cpp: layoutng_block/block_layout_algorithm.cc:1670-1907
    pub fn HandleNewFormattingContext(
        &mut self,
        child: LayoutInputNode,
        child_break_token: *const BlockBreakToken,
        previous: &mut PreviousInflowPosition,
    ) -> EStatus {
        debug_assert!(child.is_non_null());
        debug_assert!(!child.IsFloating());
        debug_assert!(!child.IsOutOfFlowPositioned());
        debug_assert!(child.CreatesNewFormattingContext());
        debug_assert!(child.IsBlock());

        let constraint_space = self.GetConstraintSpace() as *const ConstraintSpace;
        let constraint_space = unsafe { &*constraint_space };
        let child_style = child.Style();
        let direction = constraint_space.Direction();
        let child_data = self.ComputeChildData(
            previous,
            child.clone(),
            child_break_token as *const BreakToken,
            true,
        );
        let child_origin_line_offset = constraint_space.GetBfcOffset().line_offset
            + self.BorderScrollbarPadding().LineLeft(direction);
        let adjoining_margin_strut = child_data.margin_strut.clone();
        let adjoining_bfc_offset_estimate =
            child_data.bfc_offset_estimate.block_offset + adjoining_margin_strut.Sum();
        let non_adjoining_bfc_offset_estimate =
            child_data.bfc_offset_estimate.block_offset + previous.margin_strut.Sum();
        let mut child_bfc_offset_estimate = adjoining_bfc_offset_estimate;
        let mut bfc_offset_already_resolved = false;
        let mut child_determined_bfc_offset = false;
        let mut child_margin_got_separated = false;
        let mut has_adjoining_floats = false;

        if self.base.container_builder_.BfcBlockOffset().is_none() {
            has_adjoining_floats = self.base.container_builder_.GetAdjoiningObjectTypes()
                & AdjoiningObjectTypeValue::kAdjoiningFloatBoth as i32
                != 0;
            let has_clearance_past_adjoining_floats = constraint_space
                .AncestorHasClearancePastAdjoiningFloats()
                || HasClearancePastAdjoiningFloats(
                    self.base.container_builder_.GetAdjoiningObjectTypes(),
                    child_style,
                    self.Style(),
                );
            if has_clearance_past_adjoining_floats {
                child_bfc_offset_estimate = self.NextBorderEdge(previous);
                child_margin_got_separated = true;
            } else if let Some(forced) = constraint_space.ForcedBfcBlockOffset() {
                bfc_offset_already_resolved = true;
                child_bfc_offset_estimate = forced;
                debug_assert!(
                    child_bfc_offset_estimate == adjoining_bfc_offset_estimate
                        || child_bfc_offset_estimate == non_adjoining_bfc_offset_estimate
                        || child_bfc_offset_estimate == constraint_space.ClearanceOffset()
                );
                child_margin_got_separated =
                    child_bfc_offset_estimate != adjoining_bfc_offset_estimate;
            }
            child_determined_bfc_offset = true;
            if !child_margin_got_separated {
                let child_margin_is_zero = child_style.MarginBlockStartUsing(self.Style()).IsZero();
                if !child_margin_is_zero {
                    self.SetSubtreeModifiedMarginStrutIfNeeded(None);
                }
            }
            if !self.ResolveBfcBlockOffsetAt(previous, child_bfc_offset_estimate) {
                debug_assert!(!bfc_offset_already_resolved);
                return EStatus::kBfcBlockOffsetResolved;
            }
            child_bfc_offset_estimate = self.ContainerBfcOffset().block_offset;
        }

        let abort_if_cleared = child_data.margins.block_start != LayoutUnit::default()
            && !child_margin_got_separated
            && child_determined_bfc_offset;
        let mut child_bfc_offset = BfcOffset::default();
        let mut resolved_margins = BoxStrut::default();
        let mut layout_result = self.LayoutNewFormattingContext(
            child.clone(),
            child_break_token,
            &child_data,
            BfcOffset::new(child_origin_line_offset, child_bfc_offset_estimate),
            abort_if_cleared,
            &mut child_bfc_offset,
            &mut resolved_margins,
        );
        if layout_result.is_null() {
            debug_assert!(abort_if_cleared);
            if child_determined_bfc_offset {
                let old_offset = *self
                    .base
                    .container_builder_
                    .BfcBlockOffset()
                    .as_ref()
                    .unwrap();
                self.base.container_builder_.ResetBfcBlockOffset();
                debug_assert!(!constraint_space.AncestorHasClearancePastAdjoiningFloats());
                self.ResolveBfcBlockOffsetWithForced(
                    previous,
                    non_adjoining_bfc_offset_estimate,
                    None,
                );
                if (bfc_offset_already_resolved || has_adjoining_floats)
                    && old_offset
                        != *self
                            .base
                            .container_builder_
                            .BfcBlockOffset()
                            .as_ref()
                            .unwrap()
                {
                    debug_assert_eq!(old_offset, adjoining_bfc_offset_estimate);
                    return EStatus::kBfcBlockOffsetResolved;
                }
            }
            child_bfc_offset_estimate = non_adjoining_bfc_offset_estimate;
            child_margin_got_separated = true;
            layout_result = self.LayoutNewFormattingContext(
                child.clone(),
                child_break_token,
                &child_data,
                BfcOffset::new(child_origin_line_offset, child_bfc_offset_estimate),
                false,
                &mut child_bfc_offset,
                &mut resolved_margins,
            );
        }
        let layout_result = unsafe { &*layout_result };
        if constraint_space.HasBlockFragmentation() {
            let has_container_separation = self.has_break_opportunity_before_next_child_
                || child_bfc_offset.block_offset > child_bfc_offset_estimate
                || layout_result.IsPushedByFloats();
            let break_status = self.BreakBeforeChildIfNeeded(
                child.clone(),
                layout_result,
                previous,
                child_bfc_offset.block_offset,
                has_container_separation,
            );
            if break_status == BreakStatus::kBrokeBefore {
                return EStatus::kSuccess;
            }
            if break_status == BreakStatus::kNeedsEarlierBreak {
                return EStatus::kNeedsEarlierBreak;
            }
            debug_assert_eq!(layout_result.Status(), EStatus::kSuccess);
        }
        let physical_fragment = layout_result.GetPhysicalFragment();
        let fragment =
            LogicalFragment::new(constraint_space.GetWritingDirection(), physical_fragment);
        let mut logical_offset = LogicalFromBfcOffsets(
            &child_bfc_offset,
            &self.ContainerBfcOffset(),
            fragment.InlineSize(),
            self.base.container_builder_.InlineSize(),
            direction,
        );
        if !self.PositionOrPropagateListMarker(layout_result, &mut logical_offset, previous) {
            return EStatus::kBfcBlockOffsetResolved;
        }
        self.PropagateBaselineFromBlockChild(
            physical_fragment,
            &resolved_margins,
            logical_offset.block_offset,
        );
        self.base.container_builder_.AddResult(
            layout_result,
            logical_offset,
            Some(resolved_margins),
            None,
            std::ptr::null(),
        );
        if child_break_token.is_null() || !unsafe { &*child_break_token }.IsInParallelFlow() {
            *previous = self.ComputeInflowPosition(
                previous,
                child.clone(),
                &child_data,
                Some(child_bfc_offset.block_offset),
                &logical_offset,
                layout_result,
                &fragment,
                false,
            );
        }
        if !self.line_clamp_data_.UpdateAfterLayout(
            layout_result,
            previous,
            &self.base.container_builder_,
        ) {
            self.base
                .container_builder_
                .SetLinesUntilClamp(self.line_clamp_data_.LinesUntilClamp(true));
            self.base
                .container_builder_
                .SetLineClampAfterLayoutObject(self.line_clamp_data_.last_layout_object);
            return EStatus::kNeedsLineClampRelayout;
        }
        if self.base.container_builder_.ShouldTextBoxTrim() {
            self.UpdateTextBoxTrim(
                child,
                child_break_token as *const BreakToken,
                std::ptr::null(),
                layout_result,
                previous,
            );
        }
        if constraint_space.HasBlockFragmentation()
            && !self.has_break_opportunity_before_next_child_
        {
            self.has_break_opportunity_before_next_child_ = HasBreakOpportunityBeforeNextChild(
                physical_fragment,
                child_break_token as *const BreakToken,
            );
        }
        EStatus::kSuccess
    }

    // cpp: layoutng_block/block_layout_algorithm.h:315-325
    // cpp: layoutng_block/block_layout_algorithm.cc:1909-2130
    pub fn LayoutNewFormattingContext(
        &mut self,
        child: LayoutInputNode,
        child_break_token: *const BlockBreakToken,
        child_data: &InflowChildData,
        mut origin_offset: BfcOffset,
        abort_if_cleared: bool,
        out_child_bfc_offset: &mut BfcOffset,
        out_resolved_margins: &mut BoxStrut,
    ) -> *const LayoutResult {
        let style = self.Style() as *const ComputedStyle;
        let style = unsafe { &*style };
        let child_style = child.Style();
        let direction = self.GetConstraintSpace().Direction();
        let writing_direction = self.GetConstraintSpace().GetWritingDirection();

        if !IsBreakInside(child_break_token) {
            AdjustToClearance(
                self.GetExclusionSpace()
                    .ClearanceOffsetIncludingInitialLetter(
                        child_style.ClearWithContainingStyle(style),
                    ),
                &mut origin_offset,
            );
        }
        debug_assert!(self.base.container_builder_.BfcBlockOffset().is_some());

        let available_inline_size = self.ChildAvailableSize().inline_size;
        let opportunities = self.GetExclusionSpace().AllLayoutOpportunities(
            &origin_offset,
            available_inline_size,
            direction,
        );
        debug_assert!(!opportunities.is_empty());

        for opportunity in &opportunities {
            if abort_if_cleared && origin_offset.block_offset < opportunity.rect.BlockStartOffset()
            {
                return std::ptr::null();
            }

            let has_floats_on_line_left =
                opportunity.rect.LineStartOffset() != origin_offset.line_offset;
            let has_floats_on_line_right = opportunity.rect.LineEndOffset()
                != origin_offset.line_offset + self.ChildAvailableSize().inline_size;
            let can_expand_outside_opportunity =
                !has_floats_on_line_left && !has_floats_on_line_right;
            let line_left_margin = child_data.margins.LineLeft(direction);
            let line_right_margin = child_data.margins.LineRight(direction);
            let mut line_left_offset = opportunity.rect.LineStartOffset();
            let mut line_right_offset = opportunity.rect.LineEndOffset();
            if can_expand_outside_opportunity {
                debug_assert_eq!(line_left_offset, origin_offset.line_offset);
                debug_assert_eq!(
                    line_right_offset,
                    origin_offset.line_offset + self.ChildAvailableSize().inline_size
                );
                line_left_offset += line_left_margin;
                line_right_offset -= line_right_margin;
            } else {
                line_left_offset = std::cmp::max(
                    line_left_offset,
                    origin_offset.line_offset + line_left_margin.ClampNegativeToZero(),
                );
                line_right_offset = std::cmp::min(
                    line_right_offset,
                    origin_offset.line_offset + self.ChildAvailableSize().inline_size
                        - line_right_margin.ClampNegativeToZero(),
                );
            }
            let opportunity_size = (line_right_offset - line_left_offset).ClampNegativeToZero();
            let child_available_inline_size =
                (opportunity_size + child_data.margins.InlineSum()).ClampNegativeToZero();
            let child_space = self.CreateConstraintSpaceForChild(
                child.clone(),
                child_break_token as *const BreakToken,
                child_data,
                LogicalSize::new(
                    child_available_inline_size,
                    self.ChildAvailableSize().block_size,
                ),
                true,
                Some(opportunity.rect.start_offset.block_offset),
                false,
                LayoutUnit::default(),
                LayoutUnit::default(),
            );
            debug_assert!(child_space.GetExclusionSpace().IsEmpty());
            let mut block_child = BlockNode::from(child.clone());
            let layout_result = LayoutBlockChild(
                &child_space,
                child_break_token as *const BreakToken,
                self.base.early_break_,
                std::ptr::null(),
                &mut block_child,
            );
            let layout_result_ref = unsafe { &*layout_result };
            debug_assert!(layout_result_ref.GetExclusionSpace().IsEmpty());
            debug_assert_eq!(layout_result_ref.Status(), EStatus::kSuccess);
            let fragment =
                LogicalFragment::new(writing_direction, layout_result_ref.GetPhysicalFragment());
            if fragment.BlockSize() > opportunity.rect.BlockSize() {
                continue;
            }

            let mut auto_margins = child_data.margins;
            let mut text_align_offset = LayoutUnit::default();
            let mut has_auto_margins = false;
            if child.IsListMarker() {
                debug_assert!(child.ListMarkerOccupiesWholeLine());
                let marker_fragment = unsafe {
                    &*To::<PhysicalBoxFragment>(
                        layout_result_ref.GetPhysicalFragment() as *const PhysicalFragment
                    )
                };
                let marker_inline_size = marker_fragment
                    .Children()
                    .first()
                    .map(|first| LogicalFragment::new(writing_direction, first).InlineSize())
                    .unwrap_or_default();
                auto_margins.inline_start =
                    UnpositionedListMarker::new(&block_child).InlineOffset(marker_inline_size);
                auto_margins.inline_end = opportunity.rect.InlineSize()
                    - fragment.InlineSize()
                    - auto_margins.inline_start;
            } else if child_style.MarginInlineStartUsing(style).IsAuto()
                || child_style.MarginInlineEndUsing(style).IsAuto()
            {
                has_auto_margins = true;
                ResolveInlineAutoMargins(
                    child_style,
                    style,
                    child_available_inline_size,
                    fragment.InlineSize(),
                    &mut auto_margins,
                );
            } else {
                text_align_offset = WebkitTextAlignAndJustifySelfOffset(
                    child_style,
                    style,
                    opportunity.rect.InlineSize(),
                    &child_data.margins,
                    || fragment.InlineSize(),
                );
            }

            let mut child_bfc_offset =
                BfcOffset::new(LayoutUnit::default(), opportunity.rect.BlockStartOffset());
            if direction == TextDirection::kLtr {
                let auto_margin_line_left = auto_margins.LineLeft(direction) - line_left_margin;
                child_bfc_offset.line_offset =
                    line_left_offset + auto_margin_line_left + text_align_offset;
            } else {
                let auto_margin_line_right = auto_margins.LineRight(direction) - line_right_margin;
                child_bfc_offset.line_offset = line_right_offset
                    - text_align_offset
                    - auto_margin_line_right
                    - fragment.InlineSize();
            }
            if has_floats_on_line_left
                && child_bfc_offset.line_offset < opportunity.rect.LineStartOffset()
            {
                continue;
            }
            if has_floats_on_line_right
                && child_bfc_offset.line_offset + fragment.InlineSize()
                    > opportunity.rect.LineEndOffset()
            {
                continue;
            }
            if !can_expand_outside_opportunity
                && fragment.InlineSize() > opportunity.rect.InlineSize()
            {
                continue;
            }
            let mut resolved_margins = child_data.margins;
            if has_auto_margins {
                let inline_offset = LogicalFromBfcLineOffset(
                    child_bfc_offset.line_offset,
                    self.base.container_builder_.BfcLineOffset(),
                    fragment.InlineSize(),
                    self.base.container_builder_.InlineSize(),
                    direction,
                ) - self.BorderScrollbarPadding().inline_start;
                if child_style.MarginInlineStartUsing(style).IsAuto() {
                    resolved_margins.inline_start = inline_offset;
                }
                if child_style.MarginInlineEndUsing(style).IsAuto() {
                    resolved_margins.inline_end = self.ChildAvailableSize().inline_size
                        - inline_offset
                        - fragment.InlineSize();
                }
            }
            *out_child_bfc_offset = child_bfc_offset;
            *out_resolved_margins = resolved_margins;
            return layout_result;
        }
        unreachable!("a formatting-context child must fit a layout opportunity")
    }

    // cpp: layoutng_block/block_layout_algorithm.h:350-355
    // cpp: layoutng_block/block_layout_algorithm.cc:2635-2671
    pub fn UpdateTextBoxTrim(
        &mut self,
        child: LayoutInputNode,
        incoming_child_break_token: *const BreakToken,
        outgoing_inline_break_token: *const InlineBreakToken,
        layout_result: &LayoutResult,
        previous: &mut PreviousInflowPosition,
    ) {
        self.base
            .container_builder_
            .ClearShouldTextBoxTrimFragmentainerStart();
        if self.base.container_builder_.ShouldTextBoxTrimNodeStart()
            && (!child.IsInline()
                || outgoing_inline_break_token.is_null()
                || unsafe { &*outgoing_inline_break_token }.IsPastFirstFormattedLine())
        {
            self.base
                .container_builder_
                .ClearShouldTextBoxTrimNodeStart();
        }
        if self.base.container_builder_.ShouldTextBoxTrimNodeEnd() {
            if self.line_clamp_data_.data.IsMeasureUntilBfcOffset()
                && layout_result.TrimBlockEndBy().is_some()
                && !layout_result
                    .GetPhysicalFragment()
                    .GetBreakToken()
                    .is_null()
            {
                previous.logical_block_offset += layout_result.TrimBlockEndBy().unwrap();
            } else if layout_result.IsBlockEndTrimmableLine()
                || (child.IsBlock() && IsLastInflowChild(unsafe { &*child.GetLayoutBox() }))
            {
                self.base.container_builder_.ClearShouldTextBoxTrimEnd();
            } else if !layout_result.IsSelfCollapsing()
                && child.IsInline()
                && !self.override_text_box_trim_end_child_.is_non_null()
            {
                self.last_non_empty_inflow_child_ = InlineNode::from(child.clone());
                self.last_non_empty_break_token_ = incoming_child_break_token;
            }
        }
    }

    // cpp: layoutng_block/block_layout_algorithm.h:452-456
    // cpp: layoutng_block/block_layout_algorithm.cc:2931-2948
    pub fn PositionSelfCollapsingChildWithParentBfc(
        &self,
        _child: &LayoutInputNode,
        child_space: &ConstraintSpace,
        child_data: &InflowChildData,
        layout_result: &LayoutResult,
    ) -> LayoutUnit {
        debug_assert!(layout_result.IsSelfCollapsing());
        let mut child_bfc_block_offset =
            child_data.bfc_offset_estimate.block_offset + layout_result.EndMarginStrut().Sum();
        ApplyClearance(child_space, &mut child_bfc_block_offset);
        child_bfc_block_offset
    }

    // cpp: layoutng_block/block_layout_algorithm.h:357-363
    // cpp: layoutng_block/block_layout_algorithm.cc:2949-2961
    pub fn ConsumeRemainingFragmentainerSpace(&self, previous: &mut PreviousInflowPosition) {
        if self.GetConstraintSpace().HasKnownFragmentainerBlockSize() {
            previous.logical_block_offset = std::cmp::max(
                previous.logical_block_offset,
                self.FragmentainerSpaceLeftForChildren(),
            );
        }
    }

    // cpp: layoutng_block/block_layout_algorithm.h:335-340
    // cpp: layoutng_block/block_layout_algorithm.cc:2132-2213
    pub fn HandleInflow(
        &mut self,
        mut child: LayoutInputNode,
        child_break_token: *const BreakToken,
        previous: &mut PreviousInflowPosition,
        inline_child_layout_context: *mut InlineChildLayoutContext,
        previous_inline_break_token: &mut *const InlineBreakToken,
    ) -> EStatus {
        debug_assert!(child.is_non_null());
        debug_assert!(!child.IsFloating());
        debug_assert!(!child.IsOutOfFlowPositioned());
        debug_assert!(!child.CreatesNewFormattingContext());

        if child.IsInline()
            && !self.abort_when_bfc_block_offset_updated_
            && child_break_token.is_null()
            && !self.previous_result_.is_null()
        {
            debug_assert!(previous_inline_break_token.is_null());
            let algorithms = LayoutPassScope::Algorithms();
            let callback = if algorithms.is_null() {
                None
            } else {
                unsafe { &*algorithms }.inline_support.reuse_fragments
            };
            let callback = callback.unwrap_or_else(|| {
                std::panic::panic_any(UnsupportedLayout::new(
                    "inline layout module is not installed",
                ))
            });
            if callback(
                self as *mut Self as *mut ForwardBlockLayoutAlgorithm,
                InlineNode::from(child.clone()),
                previous as *mut PreviousInflowPosition as *mut ForwardPreviousInflowPosition,
                previous_inline_break_token,
            ) {
                return EStatus::kSuccess;
            }
        }

        let has_clearance_past_adjoining_floats =
            self.base.container_builder_.BfcBlockOffset().is_none()
                && child.IsBlock()
                && HasClearancePastAdjoiningFloats(
                    self.base.container_builder_.GetAdjoiningObjectTypes(),
                    child.Style(),
                    self.Style(),
                );
        let mut forced_bfc_block_offset = None;
        let mut is_pushed_by_floats = false;
        if has_clearance_past_adjoining_floats {
            if !self.ResolveBfcBlockOffset(previous) {
                return EStatus::kBfcBlockOffsetResolved;
            }
            let clear = child.Style().ClearWithContainingStyle(self.Style());
            forced_bfc_block_offset = Some(self.GetExclusionSpace().ClearanceOffset(clear));
            is_pushed_by_floats = true;
        }

        let mut child_data =
            self.ComputeChildData(previous, child.clone(), child_break_token, false);
        child_data.is_pushed_by_floats = is_pushed_by_floats;
        let child_available_size = *self.ChildAvailableSize();
        let child_space = self.CreateConstraintSpaceForChild(
            child.clone(),
            child_break_token,
            &child_data,
            child_available_size,
            false,
            forced_bfc_block_offset,
            has_clearance_past_adjoining_floats,
            previous.block_end_annotation_space,
            previous.previous_sibling_block_end_annotation_space,
        );
        let layout_result = LayoutInflow(
            &child_space,
            child_break_token,
            self.base.early_break_,
            self.column_spanner_path_,
            &mut child,
            inline_child_layout_context,
        );
        self.FinishInflow(
            child,
            child_break_token,
            &child_space,
            has_clearance_past_adjoining_floats,
            layout_result,
            &mut child_data,
            previous,
            inline_child_layout_context,
            previous_inline_break_token,
        )
    }

    // cpp: layoutng_block/block_layout_algorithm.h:342-351
    // cpp: layoutng_block/block_layout_algorithm.cc:2214-2634
    pub fn FinishInflow(
        &mut self,
        mut child: LayoutInputNode,
        child_break_token: *const BreakToken,
        child_space: &ConstraintSpace,
        has_clearance_past_adjoining_floats: bool,
        mut layout_result: *const LayoutResult,
        child_data: &mut InflowChildData,
        previous: &mut PreviousInflowPosition,
        inline_child_layout_context: *mut InlineChildLayoutContext,
        previous_inline_break_token: &mut *const InlineBreakToken,
    ) -> EStatus {
        if unsafe { &*layout_result }.Status() == EStatus::kNeedsLineClampRelayout {
            debug_assert!(self.line_clamp_data_.data.IsMeasureUntilBfcOffset());
            self.base
                .container_builder_
                .SetLinesUntilClamp(Some(unsafe { &*layout_result }.LinesUntilClamp()));
            self.base.container_builder_.SetLineClampAfterLayoutObject(
                self.line_clamp_data_
                    .PropagateClampAfterLayoutObject(unsafe { &*layout_result }),
            );
            return EStatus::kNeedsLineClampRelayout;
        }
        let mut child_bfc_block_offset = unsafe { &*layout_result }.BfcBlockOffset();
        let is_self_collapsing = unsafe { &*layout_result }.IsSelfCollapsing();
        let normal_child_had_clearance =
            unsafe { &*layout_result }.IsPushedByFloats() && !is_self_collapsing;
        if unsafe { &*layout_result }.Status() == EStatus::kBfcBlockOffsetResolved
            && self.base.container_builder_.BfcBlockOffset().is_none()
        {
            debug_assert!(child_bfc_block_offset.is_some());
            self.abort_when_bfc_block_offset_updated_ = true;
            let mut bfc_block_offset = child_bfc_block_offset.unwrap();
            if normal_child_had_clearance {
                if self.GetConstraintSpace().ClearanceOffset() == child_space.ClearanceOffset() {
                    self.base.container_builder_.SetIsPushedByFloats();
                } else {
                    bfc_block_offset = self.NextBorderEdge(previous);
                }
            }
            debug_assert!(!self
                .GetConstraintSpace()
                .AncestorHasClearancePastAdjoiningFloats());
            if !self.ResolveBfcBlockOffsetWithForced(previous, bfc_block_offset, None) {
                return EStatus::kBfcBlockOffsetResolved;
            }
        }
        let mut self_collapsing_child_had_clearance =
            is_self_collapsing && has_clearance_past_adjoining_floats;
        if child_bfc_block_offset.is_none() {
            debug_assert!(is_self_collapsing);
            if child_space.HasClearanceOffset() && child.Style().HasClear() {
                let child_block_offset_estimate =
                    self.BfcBlockOffset() + unsafe { &*layout_result }.EndMarginStrut().Sum();
                if child_block_offset_estimate < child_space.ClearanceOffset() {
                    self_collapsing_child_had_clearance = true;
                }
            }
        }
        let child_had_clearance = self_collapsing_child_had_clearance || normal_child_had_clearance;
        if child_had_clearance {
            if !self.ResolveBfcBlockOffset(previous) {
                return EStatus::kBfcBlockOffsetResolved;
            }
        } else if unsafe { &*layout_result }.SubtreeModifiedMarginStrut() {
            self.SetSubtreeModifiedMarginStrutIfNeeded(None);
        }
        let mut self_collapsing_child_needs_relayout = false;
        if child_bfc_block_offset.is_none() {
            debug_assert!(is_self_collapsing);
            if self.base.container_builder_.BfcBlockOffset().is_some()
                && unsafe { &*layout_result }.Status() == EStatus::kSuccess
            {
                child_bfc_block_offset = Some(self.PositionSelfCollapsingChildWithParentBfc(
                    &child,
                    child_space,
                    child_data,
                    unsafe { &*layout_result },
                ));
                if unsafe { &*layout_result }
                    .GetPhysicalFragment()
                    .HasAdjoiningObjectDescendants()
                    && child_bfc_block_offset.unwrap() != child_space.ExpectedBfcBlockOffset()
                {
                    self_collapsing_child_needs_relayout = true;
                }
            }
        } else if !child_had_clearance && !is_self_collapsing {
            if !self.ResolveBfcBlockOffsetAt(previous, child_bfc_block_offset.unwrap()) {
                return EStatus::kBfcBlockOffsetResolved;
            }
        }
        if self_collapsing_child_had_clearance {
            let mut margin_strut = MarginStrut::default();
            margin_strut.Append(
                &child_data.margins.block_start,
                child.Style().HasMarginBlockStartQuirk(),
            );
            if child_data.margin_strut != margin_strut {
                child_data.margin_strut = margin_strut;
                self_collapsing_child_needs_relayout = true;
            }
        }
        if (unsafe { &*layout_result }.Status() == EStatus::kBfcBlockOffsetResolved
            || self_collapsing_child_needs_relayout)
            && child_bfc_block_offset.is_some()
        {
            debug_assert!(
                !child_data.is_pushed_by_floats || unsafe { &*layout_result }.IsPushedByFloats()
            );
            child_data.is_pushed_by_floats = unsafe { &*layout_result }.IsPushedByFloats();
            let child_available_size = *self.ChildAvailableSize();
            let new_child_space = self.CreateConstraintSpaceForChild(
                child.clone(),
                child_break_token,
                child_data,
                child_available_size,
                false,
                child_bfc_block_offset,
                false,
                LayoutUnit::default(),
                previous.previous_sibling_block_end_annotation_space,
            );
            layout_result = LayoutInflow(
                &new_child_space,
                child_break_token,
                self.base.early_break_,
                self.column_spanner_path_,
                &mut child,
                inline_child_layout_context,
            );
            if unsafe { &*layout_result }.Status() == EStatus::kBfcBlockOffsetResolved {
                child_bfc_block_offset = unsafe { &*layout_result }.BfcBlockOffset();
                debug_assert!(child_bfc_block_offset.is_some());
                debug_assert!(
                    child_data.is_pushed_by_floats
                        || !unsafe { &*layout_result }.IsPushedByFloats()
                );
                let child_available_size = *self.ChildAvailableSize();
                let final_child_space = self.CreateConstraintSpaceForChild(
                    child.clone(),
                    child_break_token,
                    child_data,
                    child_available_size,
                    false,
                    child_bfc_block_offset,
                    false,
                    LayoutUnit::default(),
                    previous.previous_sibling_block_end_annotation_space,
                );
                layout_result = LayoutInflow(
                    &final_child_space,
                    child_break_token,
                    self.base.early_break_,
                    self.column_spanner_path_,
                    &mut child,
                    inline_child_layout_context,
                );
            }
            if unsafe { &*layout_result }.Status() == EStatus::kNeedsLineClampRelayout {
                debug_assert!(self.line_clamp_data_.data.IsMeasureUntilBfcOffset());
                self.base
                    .container_builder_
                    .SetLinesUntilClamp(Some(unsafe { &*layout_result }.LinesUntilClamp()));
                self.base.container_builder_.SetLineClampAfterLayoutObject(
                    self.line_clamp_data_
                        .PropagateClampAfterLayoutObject(unsafe { &*layout_result }),
                );
                return EStatus::kNeedsLineClampRelayout;
            }
            debug_assert_eq!(unsafe { &*layout_result }.Status(), EStatus::kSuccess);
            debug_assert_eq!(
                unsafe { &*layout_result }.IsSelfCollapsing(),
                is_self_collapsing
            );
        }
        let layout_result = unsafe { &*layout_result };
        let line_box_bfc_block_offset = layout_result.LineBoxBfcBlockOffset();
        if self.GetConstraintSpace().HasBlockFragmentation() {
            let consider_breaking_before = self.base.container_builder_.BfcBlockOffset().is_some()
                && child_bfc_block_offset.is_some()
                && (!child.IsInline()
                    || child_break_token.is_null()
                    || !unsafe { &*To::<InlineBreakToken>(child_break_token) }
                        .IsInParallelBlockFlow());
            if consider_breaking_before {
                let is_line_box_pushed_by_floats = line_box_bfc_block_offset
                    .is_some_and(|line| line > child_bfc_block_offset.unwrap());
                let has_container_separation = self.has_break_opportunity_before_next_child_
                    || (!self.base.container_builder_.IsPushedByFloats()
                        && (layout_result.IsPushedByFloats() || is_line_box_pushed_by_floats));
                let layout_result_to_use = self
                    .base
                    .container_builder_
                    .LayoutResultForPropagation(layout_result)
                    as *const LayoutResult;
                let break_status = self.BreakBeforeChildIfNeeded(
                    child.clone(),
                    unsafe { &*layout_result_to_use },
                    previous,
                    line_box_bfc_block_offset.unwrap_or(child_bfc_block_offset.unwrap()),
                    has_container_separation,
                );
                if child_space.ShouldForceTextBoxTrimEnd() {
                    self.base.container_builder_.ClearShouldTextBoxTrimEnd();
                }
                if break_status == BreakStatus::kBrokeBefore {
                    if self
                        .base
                        .container_builder_
                        .ShouldTextBoxTrimFragmentainerEnd()
                        && child.IsInline()
                        && !child_space.ShouldForceTextBoxTrimEnd()
                    {
                        self.last_non_empty_inflow_child_ = InlineNode::from(child.clone());
                        self.last_non_empty_break_token_ = child_break_token;
                        return EStatus::kTextBoxTrimEndDidNotApply;
                    }
                    self.base.container_builder_.ClearShouldTextBoxTrimEnd();
                    return EStatus::kSuccess;
                }
                if break_status == BreakStatus::kNeedsEarlierBreak {
                    return EStatus::kNeedsEarlierBreak;
                }
            }
            if !inline_child_layout_context.is_null() {
                for token in unsafe { &*inline_child_layout_context }.ParallelFlowBreakTokens() {
                    self.base
                        .container_builder_
                        .AddBreakToken(token.Get(), true);
                }
            }
        }
        self.base
            .container_builder_
            .SetExclusionSpace(layout_result.GetExclusionSpace());
        debug_assert!(layout_result.GetAdjoiningObjectTypes() == 0 || is_self_collapsing);
        self.base
            .container_builder_
            .SetAdjoiningObjectTypes(layout_result.GetAdjoiningObjectTypes());
        if self.base.container_builder_.BfcBlockOffset().is_none() {
            self.abort_when_bfc_block_offset_updated_ |=
                layout_result.GetAdjoiningObjectTypes() != 0;
            if layout_result.IsPushedByFloats() {
                self.base.container_builder_.SetIsPushedByFloats();
            }
        }
        let physical_fragment = layout_result.GetPhysicalFragment();
        let fragment = LogicalFragment::new(
            self.GetConstraintSpace().GetWritingDirection(),
            physical_fragment,
        );
        if line_box_bfc_block_offset.is_some() {
            child_bfc_block_offset = line_box_bfc_block_offset;
        }
        let mut logical_offset = self.CalculateLogicalOffset(
            &fragment,
            layout_result.BfcLineOffset(),
            child_bfc_block_offset,
        );
        if child.IsSliderThumb() {
            logical_offset = self.AdjustSliderThumbInlineOffset(&fragment, logical_offset);
        }
        if !self.PositionOrPropagateListMarker(layout_result, &mut logical_offset, previous) {
            return EStatus::kBfcBlockOffsetResolved;
        }
        if physical_fragment.IsLineBox() {
            self.PropagateBaselineFromLineBox(physical_fragment, logical_offset.block_offset);
        } else {
            self.PropagateBaselineFromBlockChild(
                physical_fragment,
                &child_data.margins,
                logical_offset.block_offset,
            );
        }
        let margins = if child.IsBlock() {
            Some(child_data.margins)
        } else {
            None
        };
        self.base.container_builder_.AddResult(
            layout_result,
            logical_offset,
            margins,
            None,
            std::ptr::null(),
        );
        if child_break_token.is_null() || !unsafe { &*child_break_token }.IsInParallelFlow() {
            *previous = self.ComputeInflowPosition(
                previous,
                child.clone(),
                child_data,
                child_bfc_block_offset,
                &logical_offset,
                layout_result,
                &fragment,
                self_collapsing_child_had_clearance,
            );
        }
        let outgoing_inline_break_token = if child.IsInline() {
            To::<InlineBreakToken>(physical_fragment.GetBreakToken())
        } else {
            std::ptr::null()
        };
        *previous_inline_break_token = outgoing_inline_break_token;
        if !self.line_clamp_data_.UpdateAfterLayout(
            layout_result,
            previous,
            &self.base.container_builder_,
        ) {
            self.base
                .container_builder_
                .SetLinesUntilClamp(self.line_clamp_data_.LinesUntilClamp(true));
            self.base
                .container_builder_
                .SetLineClampAfterLayoutObject(self.line_clamp_data_.last_layout_object);
            return EStatus::kNeedsLineClampRelayout;
        }
        if self.base.container_builder_.ShouldTextBoxTrim() {
            self.UpdateTextBoxTrim(
                child,
                child_break_token,
                outgoing_inline_break_token,
                layout_result,
                previous,
            );
        }
        if self.GetConstraintSpace().HasBlockFragmentation()
            && !self.has_break_opportunity_before_next_child_
        {
            self.has_break_opportunity_before_next_child_ =
                HasBreakOpportunityBeforeNextChild(physical_fragment, child_break_token);
        }
        EStatus::kSuccess
    }

    // cpp: layoutng_block/block_layout_algorithm.h:238-248
    // cpp: layoutng_block/block_layout_algorithm.cc:3270-3555
    pub fn CreateConstraintSpaceForChild(
        &mut self,
        child: LayoutInputNode,
        child_break_token: *const BreakToken,
        child_data: &InflowChildData,
        child_available_size: LogicalSize,
        is_new_fc: bool,
        child_bfc_block_offset: Option<LayoutUnit>,
        has_clearance_past_adjoining_floats: bool,
        block_start_annotation_space: LayoutUnit,
        previous_sibling_block_end_annotation_space: LayoutUnit,
    ) -> ConstraintSpace {
        let child_style = child.Style();
        let child_writing_direction = child_style.GetWritingDirection();
        let constraint_space = self.GetConstraintSpace() as *const ConstraintSpace;
        let constraint_space = unsafe { &*constraint_space };
        let style = self.Style() as *const ComputedStyle;
        let style = unsafe { &*style };
        let mut builder =
            ConstraintSpaceBuilder::new(constraint_space, child_writing_direction, is_new_fc);
        let is_in_parallel_flow = IsParallelWritingMode(
            constraint_space.GetWritingMode(),
            child_writing_direction.GetWritingMode(),
        );
        if !is_in_parallel_flow {
            SetOrthogonalFallbackInlineSize(style, child.clone(), &mut builder);
        }
        if child.IsInline() {
            if is_in_parallel_flow {
                builder.SetInlineAutoBehavior(AutoSizeBehavior::kStretchImplicit);
            }
        } else {
            let default_alignment = StyleSelfAlignmentData::new_nonlegacy(
                ItemPosition::kNormal,
                OverflowAlignment::kDefault,
            );
            let justify_self = child_style
                .ResolvedJustifySelf(&default_alignment, style)
                .GetPosition();
            if child.IsAnonymousBlockFlow() {
                builder.SetInlineAutoBehavior(AutoSizeBehavior::kStretchImplicit);
            } else if justify_self == ItemPosition::kStretch {
                builder.SetInlineAutoBehavior(AutoSizeBehavior::kStretchExplicit);
            } else if justify_self != ItemPosition::kNormal {
                builder.SetInlineAutoBehavior(AutoSizeBehavior::kFitContent);
            } else if is_in_parallel_flow
                && ShouldBlockContainerChildStretchAutoInlineSize(&BlockNode::from(child.clone()))
            {
                builder.SetInlineAutoBehavior(AutoSizeBehavior::kStretchImplicit);
            }
        }
        if self.line_clamp_data_.ShouldHideForPaint() {
            builder.SetIsHiddenForPaint(true);
        }
        builder.SetAvailableSize(child_available_size);
        builder.SetPercentageResolutionSize(self.PercentageSizeForChild(&child));
        if (child.IsAnonymousBlockFlow() || child.IsInline())
            && self.replaced_child_percentage_size_ != self.child_percentage_size_
        {
            builder.SetReplacedChildPercentageResolutionSize(self.replaced_child_percentage_size_);
        }
        if constraint_space.IsTableCell() {
            builder.SetIsTableCellChild(true);
            if self.Node().IsMathMLTableCell() {
                builder.SetInlineAutoBehavior(AutoSizeBehavior::kFitContent);
            }
            if constraint_space.IsRestrictedBlockSizeTableCell()
                && self.child_percentage_size_.block_size == kIndefiniteSize
                && !child.IsSemiReplaced()
                && child_style.LogicalHeight().HasPercent()
                && matches!(
                    child_style.OverflowBlockDirection(),
                    EOverflow::kAuto | EOverflow::kScroll
                )
            {
                builder.SetIsRestrictedBlockSizeTableCellChild();
            }
        }
        let has_bfc_block_offset = self.base.container_builder_.BfcBlockOffset().is_some();
        if !has_bfc_block_offset {
            if let Some(forced) = constraint_space.ForcedBfcBlockOffset() {
                builder.SetForcedBfcBlockOffset(forced);
            }
        }
        if !is_new_fc {
            if let Some(forced) = child_bfc_block_offset {
                builder.SetForcedBfcBlockOffset(forced);
            }
        }
        if has_bfc_block_offset {
            if child.IsBlock() {
                let cached_result =
                    unsafe { &*child.GetLayoutBox() }
                        .GetCachedLayoutResult(To::<BlockBreakToken>(child_break_token));
                if !cached_result.is_null() {
                    let prev_space = unsafe { &*cached_result }.GetConstraintSpaceForCaching();
                    let bfc_block_delta = child_data.bfc_offset_estimate.block_offset
                        - prev_space.GetBfcOffset().block_offset;
                    if let Some(forced) = prev_space.ForcedBfcBlockOffset() {
                        builder.SetOptimisticBfcBlockOffset(forced + bfc_block_delta);
                    } else if let Some(optimistic) = prev_space.OptimisticBfcBlockOffset() {
                        builder.SetOptimisticBfcBlockOffset(optimistic + bfc_block_delta);
                    }
                }
            }
        } else if let Some(optimistic) = constraint_space.OptimisticBfcBlockOffset() {
            builder.SetOptimisticBfcBlockOffset(optimistic);
        }
        if (!has_bfc_block_offset && constraint_space.AncestorHasClearancePastAdjoiningFloats())
            || has_clearance_past_adjoining_floats
        {
            builder.SetAncestorHasClearancePastAdjoiningFloats();
        }
        let mut clearance_offset = LayoutUnit::Min();
        if !IsBreakInside(DynamicTo::<BlockBreakToken>(child_break_token)) {
            if !constraint_space.IsNewFormattingContext() {
                clearance_offset = constraint_space.ClearanceOffset();
            }
            if child.IsBlock() {
                let child_clearance = self
                    .GetExclusionSpace()
                    .ClearanceOffset(child_style.ClearWithContainingStyle(style));
                clearance_offset = std::cmp::max(clearance_offset, child_clearance);
            }
        }
        builder.SetClearanceOffset(clearance_offset);
        builder.SetBaselineAlgorithmType(constraint_space.GetBaselineAlgorithmType());
        if child_data.is_pushed_by_floats {
            builder.SetIsPushedByFloats();
        }
        if !is_new_fc {
            builder.SetMarginStrut(&child_data.margin_strut);
            builder.SetBfcOffset(child_data.bfc_offset_estimate);
            builder.SetExclusionSpace(self.GetExclusionSpace());
            if !has_bfc_block_offset {
                builder.SetAdjoiningObjectTypes(
                    self.base.container_builder_.GetAdjoiningObjectTypes(),
                );
            }
            builder.SetLineClampData(self.line_clamp_data_.data.clone());
            if let Some(block_offset) = self.base.container_builder_.BfcBlockOffset() {
                if !self.line_clamp_data_.ancestor_chain.is_null()
                    && !unsafe { &*self.line_clamp_data_.ancestor_chain }.HasBfcOffset()
                {
                    self.line_clamp_data_.ancestor_chain =
                        unsafe { &*self.line_clamp_data_.ancestor_chain }
                            .WithResolvedBfcOffset(*block_offset);
                }
            }
            builder.SetLineClampAncestorChain(self.line_clamp_data_.ancestor_chain);
            builder.SetShouldTextBoxTrimInsideWhenLineClamp(
                self.line_clamp_data_.data.IsLineClampContext()
                    && (constraint_space.ShouldTextBoxTrimInsideWhenLineClamp()
                        || self.base.container_builder_.ShouldTextBoxTrimNodeEnd()),
            );
            if RuntimeEnabledFeatures::AnnotationSpaceOnStartEnabled()
                && constraint_space.ContainsAnnotations()
                && !constraint_space.IsInsideBalancedColumns()
                && previous_sibling_block_end_annotation_space > LayoutUnit::default()
            {
                builder.SetPreviousSiblingBlockEndAnnotationSpace(
                    previous_sibling_block_end_annotation_space,
                );
            }
        }
        builder.SetBlockStartAnnotationSpace(block_start_annotation_space);
        if self.base.container_builder_.ShouldTextBoxTrim()
            && !child.IsFloatingOrOutOfFlowPositioned()
        {
            let known_to_have_successive_content =
                !child.IsInline() && !IsLastInflowChild(unsafe { &*child.GetLayoutBox() });
            SetTextBoxTrimOnChildSpaceBuilderWithKnownContent(
                &self.base.container_builder_,
                known_to_have_successive_content,
                &mut builder,
            );
            if self.base.container_builder_.ShouldTextBoxTrimEnd()
                && child.IsInline()
                && &child == self.override_text_box_trim_end_child_.AsLayoutInputNode()
                && InlineBreakToken::IsStartEqual(
                    To::<InlineBreakToken>(self.override_text_box_trim_end_break_token_),
                    To::<InlineBreakToken>(child_break_token),
                )
            {
                builder.SetShouldForceTextBoxTrimEnd();
            }
        }
        if self.is_relayout_for_margin_end_trim_
            && (!self.pending_margin_end_trim_child_.is_non_null()
                || child == self.pending_margin_end_trim_child_)
        {
            builder.SetShouldForceMarginTrimEnd();
        }
        if constraint_space.HasBlockFragmentation() {
            let fragmentainer_offset_delta = if is_new_fc {
                child_bfc_block_offset.expect("new formatting context has a BFC offset")
                    - constraint_space.ExpectedBfcBlockOffset()
            } else {
                builder.ExpectedBfcBlockOffset() - constraint_space.ExpectedBfcBlockOffset()
            };
            SetupSpaceBuilderForFragmentationFromBuilder(
                &self.base.container_builder_,
                &child,
                fragmentainer_offset_delta,
                &mut builder,
            );
            if !is_new_fc && constraint_space.IsInColumnBfc() {
                builder.SetIsInColumnBfc();
            }
            if constraint_space.IsPastBreak()
                || self.base.container_builder_.HasInsertedChildBreak()
            {
                builder.SetIsPastBreak();
            }
        }
        if !constraint_space.IsNewFormattingContext() {
            if self.Node().IsAnonymousBlockFlow() {
                builder.SetIgnoreMarginsForStretch(
                    constraint_space.GetWritingDirection(),
                    constraint_space.IgnoreMarginsForStretch(),
                );
            } else {
                let has_stretch = if IsHorizontalWritingMode(constraint_space.GetWritingMode()) {
                    child_style.Height().HasStretch()
                        || child_style.MinHeight().HasStretch()
                        || child_style.MaxHeight().HasStretch()
                } else {
                    child_style.Width().HasStretch()
                        || child_style.MinWidth().HasStretch()
                        || child_style.MaxWidth().HasStretch()
                };
                if has_stretch || child.IsAnonymousBlockFlow() || child.IsInline() {
                    builder.SetIgnoreMarginsForStretch(
                        constraint_space.GetWritingDirection(),
                        LogicalBoxSides::new(
                            false,
                            false,
                            self.BorderPadding().block_start == LayoutUnit::default(),
                            self.BorderPadding().block_end == LayoutUnit::default(),
                        ),
                    );
                }
            }
        }
        builder.ToConstraintSpace()
    }

    // cpp: layoutng_block/block_layout_algorithm.h:254-263
    // cpp: layoutng_block/block_layout_algorithm.cc:2737-2930
    pub fn ComputeInflowPosition(
        &mut self,
        previous: &PreviousInflowPosition,
        child: LayoutInputNode,
        child_data: &InflowChildData,
        child_bfc_block_offset: Option<LayoutUnit>,
        logical_offset: &LogicalOffset,
        layout_result: &LayoutResult,
        fragment: &LogicalFragment<'_>,
        self_collapsing_child_had_clearance: bool,
    ) -> PreviousInflowPosition {
        let mut logical_block_offset;
        let mut clearance_after_line = None;
        let mut trim_block_end_by = None;
        let is_self_collapsing = layout_result.IsSelfCollapsing();
        if is_self_collapsing {
            logical_block_offset = previous.logical_block_offset;
            if self_collapsing_child_had_clearance {
                debug_assert!(self.base.container_builder_.BfcBlockOffset().is_some());
                let child_block = child_bfc_block_offset.expect("cleared child BFC offset");
                logical_block_offset += previous.margin_strut.Sum();
                let clearance = child_block
                    - layout_result.EndMarginStrut().Sum()
                    - self.NextBorderEdge(previous);
                logical_block_offset += clearance;
            }
            if self.base.container_builder_.BfcBlockOffset().is_none() {
                debug_assert_eq!(logical_block_offset, LayoutUnit::default());
            }
        } else {
            logical_block_offset = logical_offset.block_offset + fragment.BlockSize();
            clearance_after_line = layout_result.ClearanceAfterLine();
            trim_block_end_by = layout_result.TrimBlockEndBy();
            if let Some(trim) = trim_block_end_by {
                logical_block_offset -= trim;
                if let Some(clearance) = clearance_after_line {
                    logical_block_offset += clearance;
                }
            } else {
                logical_block_offset += std::cmp::max(
                    layout_result.AnnotationOverflow(),
                    clearance_after_line.unwrap_or_default(),
                );
            }
        }

        let mut margin_strut = layout_result.EndMarginStrut();
        let is_quirky = (is_self_collapsing && child.Style().HasMarginBlockStartQuirk())
            || child.Style().HasMarginBlockEndQuirk();
        margin_strut.Append(&child_data.margins.block_end, is_quirky);
        if child.IsBlock() {
            self.SetSubtreeModifiedMarginStrutIfNeeded(Some(&child.Style().MarginBlockEnd()));
        }
        if self.is_relayout_for_margin_end_trim_ {
            if !self.pending_margin_end_trim_child_.is_non_null()
                || (child == self.pending_margin_end_trim_child_ && !is_self_collapsing)
            {
                margin_strut = MarginStrut::default();
                self.pending_margin_end_trim_child_ = LayoutInputNode::null();
            }
        } else if !is_self_collapsing && !self.is_last_non_self_collapsing_child_determined_ {
            self.last_non_self_collapsing_child_ = child.clone();
        }
        if self.GetConstraintSpace().HasBlockFragmentation() {
            let physical_fragment =
                DynamicTo::<PhysicalBoxFragment>(layout_result.GetPhysicalFragment() as *const _);
            if !physical_fragment.is_null() {
                let token = unsafe { &*physical_fragment }.GetBreakToken();
                if !token.is_null() && !unsafe { &*token }.IsAtBlockEnd() {
                    margin_strut = MarginStrut::default();
                }
            }
        }

        let self_or_sibling_self_collapsing_child_had_clearance =
            self_collapsing_child_had_clearance
                || (previous.self_collapsing_child_had_clearance && is_self_collapsing);
        let mut annotation_space = LayoutUnit::default();
        if !is_self_collapsing && trim_block_end_by.is_none() {
            annotation_space = layout_result.BlockEndAnnotationSpace();
            if layout_result.AnnotationOverflow() > LayoutUnit::default() {
                debug_assert_eq!(annotation_space, LayoutUnit::default());
                annotation_space = -std::cmp::max(
                    LayoutUnit::default(),
                    layout_result.AnnotationOverflow() - clearance_after_line.unwrap_or_default(),
                );
            }
        }
        let mut previous_sibling_block_end_annotation_space = LayoutUnit::default();
        if RuntimeEnabledFeatures::AnnotationSpaceOnStartEnabled()
            && self.GetConstraintSpace().ContainsAnnotations()
            && child.IsBlock()
            && annotation_space > LayoutUnit::default()
        {
            previous_sibling_block_end_annotation_space = annotation_space;
        }
        PreviousInflowPosition {
            logical_block_offset,
            margin_strut,
            block_end_annotation_space: annotation_space,
            previous_sibling_block_end_annotation_space,
            self_collapsing_child_had_clearance:
                self_or_sibling_self_collapsing_child_had_clearance,
        }
    }

    // cpp: layoutng_block/block_layout_algorithm.h:371-372
    // cpp: layoutng_block/block_layout_algorithm.cc:3009-3154
    pub fn BreakBeforeChildIfNeeded(
        &mut self,
        child: LayoutInputNode,
        layout_result: &LayoutResult,
        previous: &mut PreviousInflowPosition,
        bfc_block_offset: LayoutUnit,
        has_container_separation: bool,
    ) -> BreakStatus {
        debug_assert!(self.GetConstraintSpace().HasBlockFragmentation());
        debug_assert!(self.base.container_builder_.BfcBlockOffset().is_some());
        let fragmentainer_block_offset =
            FragmentainerOffsetAtBfcForBuilder(&self.base.container_builder_) + bfc_block_offset
                - layout_result.AnnotationBlockOffsetAdjustment();
        if has_container_separation {
            let break_between = CalculateBreakBetweenValue(
                child.clone(),
                layout_result,
                &self.base.container_builder_,
            );
            if IsForcedBreakValue(self.GetConstraintSpace(), break_between) {
                let capacity = self.FragmentainerCapacityForChildren();
                BreakBeforeChild(
                    child,
                    layout_result,
                    fragmentainer_block_offset,
                    capacity,
                    Some(BreakAppeal::kBreakAppealPerfect),
                    true,
                    &mut self.base.container_builder_,
                    None,
                );
                self.ConsumeRemainingFragmentainerSpace(previous);
                return BreakStatus::kBrokeBefore;
            }
        }
        let mut appeal_before = CalculateBreakAppealBeforeChild(
            self.GetConstraintSpace(),
            child.clone(),
            layout_result,
            &self.base.container_builder_,
            has_container_separation,
        );
        if self.base.MovePastBreakpoint(
            child.clone(),
            layout_result,
            fragmentainer_block_offset,
            appeal_before,
        ) {
            return BreakStatus::kContinue;
        }
        if child.IsInline() && layout_result.Status() == EStatus::kSuccess {
            if self.first_overflowing_line_ == 0 {
                let capacity = self.FragmentainerCapacityForChildren();
                let builder: *mut FragmentBuilder =
                    std::ops::DerefMut::deref_mut(&mut self.base.container_builder_);
                PropagateSpaceShortageDefault(
                    layout_result,
                    fragmentainer_block_offset,
                    capacity,
                    builder,
                );
            }
            let line_count = self.base.container_builder_.LineCount();
            if line_count != 0 {
                if self.first_overflowing_line_ == 0 {
                    self.first_overflowing_line_ = line_count;
                }
                let mut minimum_line_count = self.Style().Orphans() as i32;
                if self.GetBreakToken().is_null() == false {
                    minimum_line_count =
                        std::cmp::max(minimum_line_count, self.Style().Widows() as i32);
                }
                if line_count < minimum_line_count {
                    if appeal_before > BreakAppeal::kBreakAppealViolatingOrphansAndWidows {
                        appeal_before = BreakAppeal::kBreakAppealViolatingOrphansAndWidows;
                    }
                } else {
                    debug_assert!(line_count >= self.first_overflowing_line_);
                    let line_box = DynamicTo::<PhysicalLineBoxFragment>(
                        layout_result.GetPhysicalFragment() as *const _,
                    );
                    let widows_found = line_count - self.first_overflowing_line_ + 1;
                    if widows_found < self.Style().Widows() as i32
                        || (!line_box.is_null() && unsafe { &*line_box }.IsEmptyLineBox())
                    {
                        if !self
                            .base
                            .container_builder_
                            .ShouldTextBoxTrimFragmentainerEnd()
                            || self.override_text_box_trim_end_child_.is_non_null()
                        {
                            return BreakStatus::kContinue;
                        }
                    }
                }
                self.fit_all_lines_ = true;
            }
        }
        let capacity = self.FragmentainerCapacityForChildren();
        if !AttemptSoftBreakDefault(
            child,
            layout_result,
            fragmentainer_block_offset,
            capacity,
            appeal_before,
            &mut self.base.container_builder_,
        ) {
            return BreakStatus::kNeedsEarlierBreak;
        }
        self.ConsumeRemainingFragmentainerSpace(previous);
        BreakStatus::kBrokeBefore
    }

    // cpp: layoutng_block/block_layout_algorithm.h:373-374
    // cpp: layoutng_block/block_layout_algorithm.cc:3156-3190
    pub fn UpdateEarlyBreakBetweenLines(&mut self) {
        debug_assert!(self.base.early_break_.is_null());
        debug_assert!(!self.base.container_builder_.HasInflowChildBreakInside());
        let line_count = self.base.container_builder_.LineCount();
        if line_count < 2 {
            return;
        }
        let mut line_number = std::cmp::max(
            line_count - self.Style().Widows() as i32,
            std::cmp::min(line_count - 1, self.Style().Orphans() as i32),
        );
        let mut appeal = BreakAppeal::kBreakAppealPerfect;
        if line_number < self.Style().Orphans() as i32
            || line_count - line_number < self.Style().Widows() as i32
        {
            line_number = line_count - 1;
            appeal = BreakAppeal::kBreakAppealViolatingOrphansAndWidows;
        }
        if self.base.container_builder_.HasEarlyBreak()
            && self
                .base
                .container_builder_
                .GetEarlyBreak()
                .GetBreakAppeal()
                > appeal
        {
            return;
        }
        let breakpoint = MakeGarbageCollected(EarlyBreak::from_line(line_number, appeal));
        self.base.container_builder_.SetEarlyBreak(breakpoint);
    }

    // cpp: layoutng_block/block_layout_algorithm.h:367-371
    // cpp: layoutng_block/block_layout_algorithm.cc:2962-3008
    pub fn FinalizeForFragmentation(&mut self) -> BreakStatus {
        if self.Node().IsTableCell() {
            self.base
                .container_builder_
                .SetShouldPreventBreakBeforeBlockEndDecorations(true);
        }
        if self
            .Node()
            .IsInlineFormattingContextRoot(std::ptr::null_mut())
            && self.base.early_break_.is_null()
            && self.GetConstraintSpace().HasBlockFragmentation()
        {
            if self.base.container_builder_.HasInflowChildBreakInside()
                || self.first_overflowing_line_ != 0
            {
                if self.first_overflowing_line_ != 0
                    && self.first_overflowing_line_ < self.base.container_builder_.LineCount()
                {
                    let line_number = if self.fit_all_lines_ {
                        self.first_overflowing_line_
                    } else {
                        let line_count = self.base.container_builder_.LineCount();
                        std::cmp::max(
                            line_count - self.Style().Widows() as i32,
                            std::cmp::min(line_count, self.Style().Orphans() as i32),
                        )
                    };
                    let breakpoint = MakeGarbageCollected(EarlyBreak::from_line(
                        line_number,
                        BreakAppeal::kBreakAppealPerfect,
                    ));
                    self.base.container_builder_.SetEarlyBreak(breakpoint);
                    return BreakStatus::kNeedsEarlierBreak;
                }
            } else {
                self.UpdateEarlyBreakBetweenLines();
            }
        }
        if self.base.container_builder_.IsFragmentainerBoxType() {
            FinishFragmentationForFragmentainer(&mut self.base.container_builder_)
        } else {
            FinishFragmentation(&mut self.base.container_builder_)
        }
    }

    // cpp: layoutng_block/block_layout_algorithm.h:481-484
    // cpp: layoutng_block/block_layout_algorithm.cc:3761-3813
    pub fn PositionOrPropagateListMarker(
        &mut self,
        layout_result: &LayoutResult,
        content_offset: &mut LogicalOffset,
        previous: &mut PreviousInflowPosition,
    ) -> bool {
        if !self.ShouldPlaceUnpositionedListMarker() {
            return true;
        }
        let list_marker = self
            .base
            .container_builder_
            .GetUnpositionedListMarker()
            .clone();
        if !list_marker.is_present() {
            return true;
        }
        self.base.container_builder_.ClearUnpositionedListMarker();
        let space = self.GetConstraintSpace() as *const ConstraintSpace;
        let space = unsafe { &*space };
        let content = layout_result.GetPhysicalFragment();
        let baseline_type: FontBaseline = self.Style().GetFontBaseline();
        if let Some(content_baseline) =
            list_marker.ContentAlignmentBaseline(space, baseline_type, content)
        {
            let marker_layout_result =
                list_marker.Layout(space, self.base.container_builder_.Style(), baseline_type);
            debug_assert!(!marker_layout_result.is_null());
            let marker_layout_result = unsafe { &*marker_layout_result };
            if self.base.container_builder_.BfcBlockOffset().is_none()
                && marker_layout_result.BfcBlockOffset().is_some()
            {
                #[cfg(debug_assertions)]
                list_marker.CheckMargin();
                if !self.ResolveBfcBlockOffset(previous) {
                    return false;
                }
            }
            let border_scrollbar_padding = *self.BorderScrollbarPadding();
            list_marker.AddToBox(
                space,
                baseline_type,
                content,
                &border_scrollbar_padding,
                marker_layout_result,
                content_baseline,
                &mut content_offset.block_offset,
                &mut self.base.container_builder_,
            );
            return true;
        }
        self.base
            .container_builder_
            .SetUnpositionedListMarker(&list_marker);
        true
    }

    // cpp: layoutng_block/block_layout_algorithm.h:485-486
    // cpp: layoutng_block/block_layout_algorithm.cc:3815-3845
    pub fn PositionListMarkerWithoutLineBoxes(
        &mut self,
        previous: &mut PreviousInflowPosition,
    ) -> bool {
        debug_assert!(self.ShouldPlaceUnpositionedListMarker());
        let list_marker = self
            .base
            .container_builder_
            .GetUnpositionedListMarker()
            .clone();
        debug_assert!(list_marker.is_present());
        let space = self.GetConstraintSpace() as *const ConstraintSpace;
        let space = unsafe { &*space };
        let baseline_type = self.Style().GetFontBaseline();
        let marker_layout_result =
            list_marker.Layout(space, self.base.container_builder_.Style(), baseline_type);
        debug_assert!(!marker_layout_result.is_null());
        let marker_layout_result = unsafe { &*marker_layout_result };
        if self.base.container_builder_.BfcBlockOffset().is_none()
            && marker_layout_result.BfcBlockOffset().is_some()
        {
            #[cfg(debug_assertions)]
            list_marker.CheckMargin();
            if !self.ResolveBfcBlockOffset(previous) {
                return false;
            }
        }
        list_marker.AddToBoxWithoutLineBoxes(
            space,
            baseline_type,
            marker_layout_result,
            &mut self.base.container_builder_,
            &mut self.intrinsic_block_size_,
        );
        self.base.container_builder_.ClearUnpositionedListMarker();
        true
    }

    // cpp: layoutng_block/block_layout_algorithm.h:499-500
    // cpp: layoutng_block/block_layout_algorithm.cc:3949-3956
    pub fn AdjustSliderThumbInlineOffset(
        &self,
        fragment: &LogicalFragment<'_>,
        logical_offset: LogicalOffset,
    ) -> LogicalOffset {
        let available_extent = self.ChildAvailableSize().inline_size - fragment.InlineSize();
        let ratio = self.Node().SliderThumbValueRatio();
        let offset = LayoutUnit::from_f64(ratio * available_extent);
        LogicalOffset::new(
            logical_offset.inline_offset + offset,
            logical_offset.block_offset,
        )
    }

    // cpp: layoutng_block/block_layout_algorithm.h:488-494
    // cpp: layoutng_block/block_layout_algorithm.cc:3847-3931
    #[inline(never)]
    pub fn HandleTextControlPlaceholder(
        &mut self,
        placeholder: BlockNode,
        previous: &PreviousInflowPosition,
    ) -> PlaceholderLayoutResult {
        debug_assert!(self.Node().IsTextControl());
        const TEXT_BLOCK_INDEX: usize = 0;
        let mut available_size = *self.ChildAvailableSize();
        let apply_fixed_size = self.Style().ApplyControlFixedSize(self.Node().GetDOMNode());
        if !self.base.container_builder_.Children().is_empty() && apply_fixed_size {
            let child_link = &self.base.container_builder_.Children()[TEXT_BLOCK_INDEX];
            let child = unsafe { &*child_link.fragment.Get() };
            if child.IsTextControlContainer() {
                let box_child = unsafe { &*To::<PhysicalBoxFragment>(child as *const _) };
                if let Some(grand_child) = box_child.PostLayoutChildren().iter().next() {
                    let grand_child = unsafe { &*grand_child.fragment.Get() };
                    let logical = LogicalFragment::new(
                        self.GetConstraintSpace().GetWritingDirection(),
                        grand_child,
                    );
                    available_size.inline_size = logical.InlineSize();
                }
            }
        }
        let is_new_fc = placeholder.CreatesNewFormattingContext();
        let child_data = self.ComputeChildData(
            previous,
            placeholder.base.clone(),
            std::ptr::null(),
            is_new_fc,
        );
        let space = self.CreateConstraintSpaceForChild(
            placeholder.base.clone(),
            std::ptr::null(),
            &child_data,
            available_size,
            is_new_fc,
            None,
            false,
            LayoutUnit::default(),
            LayoutUnit::default(),
        );
        let result =
            placeholder.Layout(&space, std::ptr::null(), std::ptr::null(), std::ptr::null());
        let result = unsafe { &*result };
        if result.Status() != EStatus::kSuccess {
            return PlaceholderLayoutResult {
                logical_block_offset: previous.logical_block_offset,
                status: result.Status(),
            };
        }
        let mut offset = self.BorderScrollbarPadding().StartOffset();
        if self.Node().IsTextArea() || self.base.container_builder_.FirstBaseline().is_none() {
            return PlaceholderLayoutResult {
                logical_block_offset: self.FinishTextControlPlaceholder(
                    result,
                    &offset,
                    apply_fixed_size,
                    previous,
                ),
                status: result.Status(),
            };
        }
        let box_fragment =
            unsafe { &*To::<PhysicalBoxFragment>(result.GetPhysicalFragment() as *const _) };
        let fragment = LogicalBoxFragment::new(
            self.GetConstraintSpace().GetWritingDirection(),
            box_fragment,
        );
        if let Some(placeholder_baseline) = fragment.FirstBaseline() {
            let first_baseline = self.base.container_builder_.FirstBaseline().unwrap();
            let border_padding_block_start = self.BorderScrollbarPadding().block_start;
            offset.block_offset = first_baseline - placeholder_baseline;
            if !apply_fixed_size && offset.block_offset < border_padding_block_start {
                let new_baseline = placeholder_baseline + border_padding_block_start;
                self.base.container_builder_.SetFirstBaseline(new_baseline);
                self.base.container_builder_.SetLastBaseline(new_baseline);
                let first_child = &self.base.container_builder_.Children()[TEXT_BLOCK_INDEX];
                let first_child_fragment = first_child.fragment.Get();
                let mut first_child_offset = first_child.offset;
                first_child_offset.block_offset += new_baseline - first_baseline;
                self.base.container_builder_.ReplaceChild(
                    TEXT_BLOCK_INDEX,
                    unsafe { &*first_child_fragment },
                    first_child_offset,
                );
                offset.block_offset = border_padding_block_start;
            }
        }
        PlaceholderLayoutResult {
            logical_block_offset: self.FinishTextControlPlaceholder(
                result,
                &offset,
                apply_fixed_size,
                previous,
            ),
            status: result.Status(),
        }
    }

    // cpp: layoutng_block/block_layout_algorithm.h:495-498
    // cpp: layoutng_block/block_layout_algorithm.cc:3933-3947
    pub fn FinishTextControlPlaceholder(
        &mut self,
        result: &LayoutResult,
        offset: &LogicalOffset,
        apply_fixed_size: bool,
        previous: &PreviousInflowPosition,
    ) -> LayoutUnit {
        self.base
            .container_builder_
            .AddResult(result, *offset, None, None, std::ptr::null());
        let block_offset = previous.logical_block_offset;
        if apply_fixed_size {
            return block_offset;
        }
        let box_fragment =
            unsafe { &*To::<PhysicalBoxFragment>(result.GetPhysicalFragment() as *const _) };
        let fragment = LogicalBoxFragment::new(
            self.GetConstraintSpace().GetWritingDirection(),
            box_fragment,
        );
        std::cmp::max(block_offset, offset.block_offset + fragment.BlockSize())
    }

    // cpp: layoutng_block/block_layout_algorithm.h:179-180
    // cpp: layoutng_block/block_layout_algorithm.cc:637-684
    #[inline(never)]
    pub fn HandleNonsuccessfulLayoutResult(
        &mut self,
        result: *const LayoutResult,
    ) -> *const LayoutResult {
        let result_ref = unsafe { &*result };
        debug_assert_ne!(result_ref.Status(), EStatus::kSuccess);
        match result_ref.Status() {
            EStatus::kNeedsEarlierBreak => {
                let early_break = result_ref.GetEarlyBreak();
                debug_assert!(!early_break.is_null());
                self.base
                    .RelayoutAndBreakEarlierDefault::<Self>(unsafe { &*early_break })
            }
            EStatus::kNeedsLineClampRelayout => {
                if !self.line_clamp_data_.data.IsMeasureUntilBfcOffset() {
                    debug_assert!(self.line_clamp_data_.data.IsClampByLines());
                    debug_assert_eq!(result_ref.LinesUntilClamp(), 0);
                    return self.RelayoutIgnoringLineClamp();
                }
                if self.GetConstraintSpace().IsNewFormattingContext() {
                    let after = result_ref.LineClampAfterLayoutObject();
                    if !after.is_null() {
                        return self.RelayoutClampingAfterLayoutObject(after);
                    }
                    let lines_to_relayout = if !self.line_clamp_data_.data.IsClampByLines() {
                        result_ref.LinesUntilClamp()
                    } else {
                        debug_assert!(self.Style().LineClamp() > 0);
                        self.Style().LineClamp() - result_ref.LinesUntilClamp()
                    };
                    return self.RelayoutClampingByLines(lines_to_relayout);
                }
                result
            }
            EStatus::kDisableFragmentation => {
                debug_assert!(self.GetConstraintSpace().HasBlockFragmentation());
                self.base.RelayoutWithoutFragmentation::<Self>()
            }
            EStatus::kTextBoxTrimEndDidNotApply => self.RelayoutForTextBoxTrimEnd(),
            EStatus::kMarginTrimEndDidNotApply => self.RelayoutForMarginTrimEnd(),
            _ => result,
        }
    }

    // cpp: layoutng_block/block_layout_algorithm.h:172-174
    // cpp: layoutng_block/block_layout_algorithm.cc:590-611
    pub fn CalculateLogicalOffset(
        &self,
        fragment: &LogicalFragment<'_>,
        child_bfc_line_offset: LayoutUnit,
        child_bfc_block_offset: Option<LayoutUnit>,
    ) -> LogicalOffset {
        let inline_size = self.base.container_builder_.InlineSize();
        let direction = self.GetConstraintSpace().Direction();
        if let (Some(child_block_offset), Some(_)) = (
            child_bfc_block_offset,
            self.base.container_builder_.BfcBlockOffset(),
        ) {
            return LogicalFromBfcOffsets(
                &BfcOffset::new(child_bfc_line_offset, child_block_offset),
                &self.ContainerBfcOffset(),
                fragment.InlineSize(),
                inline_size,
                direction,
            );
        }
        let inline_offset = LogicalFromBfcLineOffset(
            child_bfc_line_offset,
            self.base.container_builder_.BfcLineOffset(),
            fragment.InlineSize(),
            inline_size,
            direction,
        );
        LogicalOffset::new(inline_offset, LayoutUnit::default())
    }

    // cpp: layoutng_block/block_layout_algorithm.h:175-175
    // cpp: layoutng_block/block_layout_algorithm.cc:3958-3976
    pub fn ComputeInitialBlockStartAnnotationSpace(&self) -> LayoutUnit {
        let padding_start = self.base.container_builder_.Padding().block_start;
        if RuntimeEnabledFeatures::AnnotationSpaceOnStartEnabled()
            && self.GetConstraintSpace().ContainsAnnotations()
            && !self.GetConstraintSpace().IsNewFormattingContext()
            && !self.GetConstraintSpace().IsInsideBalancedColumns()
            && self.Borders().block_start == LayoutUnit::default()
        {
            let mut margin_strut = self.GetConstraintSpace().GetMarginStrut();
            margin_strut.Append(
                &ComputeMarginsForSelf(self.GetConstraintSpace(), self.Style()).block_start,
                self.Style().HasMarginBlockStartQuirk(),
            );
            return margin_strut.Sum()
                + padding_start
                + self
                    .GetConstraintSpace()
                    .PreviousSiblingBlockEndAnnotationSpace();
        }
        padding_start
    }

    // cpp: layoutng_block/block_layout_algorithm.h:210-218
    pub fn BfcBlockOffset(&self) -> LayoutUnit {
        if let Some(offset) = self.base.container_builder_.BfcBlockOffset() {
            return *offset;
        }
        self.GetConstraintSpace().GetBfcOffset().block_offset
    }

    // cpp: layoutng_block/block_layout_algorithm.h:413-416
    // cpp: layoutng_block/block_layout_algorithm.cc:3665-3720
    pub fn ResolveBfcBlockOffsetWithForced(
        &mut self,
        previous: &mut PreviousInflowPosition,
        mut bfc_block_offset: LayoutUnit,
        forced_bfc_block_offset: Option<LayoutUnit>,
    ) -> bool {
        if self.GetConstraintSpace().IsPushedByFloats() {
            self.base.container_builder_.SetIsPushedByFloats();
        }
        if self.base.container_builder_.BfcBlockOffset().is_some() {
            return true;
        }
        bfc_block_offset = forced_bfc_block_offset.unwrap_or(bfc_block_offset);
        if ApplyClearance(self.GetConstraintSpace(), &mut bfc_block_offset) {
            self.base.container_builder_.SetIsPushedByFloats();
        }
        self.base
            .container_builder_
            .SetBfcBlockOffset(bfc_block_offset);
        if self.NeedsAbortOnBfcBlockOffsetChange() {
            debug_assert!(!self.GetConstraintSpace().IsNewFormattingContext());
            return false;
        }
        previous.logical_block_offset = LayoutUnit::default();
        if !self.is_resuming_ {
            previous.margin_strut = MarginStrut::default();
        } else {
            debug_assert!(previous.margin_strut.IsEmpty());
        }
        true
    }

    // cpp: layoutng_block/block_layout_algorithm.h:420-424
    pub fn ResolveBfcBlockOffsetAt(
        &mut self,
        previous: &mut PreviousInflowPosition,
        bfc_block_offset: LayoutUnit,
    ) -> bool {
        self.ResolveBfcBlockOffsetWithForced(
            previous,
            bfc_block_offset,
            self.GetConstraintSpace().ForcedBfcBlockOffset(),
        )
    }

    // cpp: layoutng_block/block_layout_algorithm.h:428-431
    pub fn ResolveBfcBlockOffset(&mut self, previous: &mut PreviousInflowPosition) -> bool {
        let bfc_block_offset = self.NextBorderEdge(previous);
        self.ResolveBfcBlockOffsetAt(previous, bfc_block_offset)
    }

    // cpp: layoutng_block/block_layout_algorithm.h:445-445
    // cpp: layoutng_block/block_layout_algorithm.cc:3722-3730
    pub fn NeedsAbortOnBfcBlockOffsetChange(&self) -> bool {
        debug_assert!(self.base.container_builder_.BfcBlockOffset().is_some());
        if !self.abort_when_bfc_block_offset_updated_ {
            return false;
        }
        self.base.container_builder_.BfcBlockOffset().unwrap()
            != self.GetConstraintSpace().ExpectedBfcBlockOffset()
    }

    // cpp: layoutng_block/block_layout_algorithm.h:451-451
    // cpp: layoutng_block/block_layout_algorithm.cc:3732-3759
    pub fn CalculateQuirkyBodyMarginBlockSum(
        &self,
        end_margin_strut: &MarginStrut,
    ) -> Option<LayoutUnit> {
        if !self.Node().IsQuirkyAndFillsViewport() {
            return None;
        }
        if !self.Style().LogicalHeight().IsAuto() {
            return None;
        }
        if self.GetConstraintSpace().IsNewFormattingContext() {
            return None;
        }
        debug_assert!(self.Node().IsBody());
        let block_end_margin =
            ComputeMarginsForSelf(self.GetConstraintSpace(), self.Style()).block_end;
        if self.base.container_builder_.BfcBlockOffset().is_none() {
            return Some(end_margin_strut.Sum() + block_end_margin);
        }
        let mut body_strut = end_margin_strut.clone();
        body_strut.Append(&block_end_margin, self.Style().HasMarginBlockEndQuirk());
        Some(
            self.base.container_builder_.BfcBlockOffset().unwrap()
                - self.GetConstraintSpace().GetBfcOffset().block_offset
                + body_strut.Sum(),
        )
    }

    // cpp: layoutng_block/block_layout_algorithm.h:388-389
    // cpp: layoutng_block/block_layout_algorithm.cc:3577-3613
    pub fn PropagateBaselineFromLineBox(
        &mut self,
        child: &PhysicalFragment,
        block_offset: LayoutUnit,
    ) {
        let line_box = unsafe { &*To::<PhysicalLineBoxFragment>(child as *const _) };
        if line_box.IsEmptyLineBox() {
            return;
        }
        if self.line_clamp_data_.IsPastClampPoint() {
            return;
        }
        if line_box.IsBlockInInline() {
            let items_builder = self.base.container_builder_.ItemsBuilder();
            debug_assert!(!items_builder.is_null());
            let items = unsafe { &*items_builder }.GetLogicalLineItems(line_box);
            let result = items.BlockInInlineLayoutResult();
            debug_assert!(!result.is_null());
            self.PropagateBaselineFromBlockChild(
                unsafe { &*result }.GetPhysicalFragment(),
                &BoxStrut::default(),
                block_offset,
            );
            return;
        }
        let metrics = line_box.BaselineMetrics();
        debug_assert!(!metrics.IsEmpty());
        let baseline = block_offset
            + if self.Style().IsFlippedLinesWritingMode() {
                metrics.descent
            } else {
                metrics.ascent
            };
        if self.base.container_builder_.FirstBaseline().is_none() {
            self.base.container_builder_.SetFirstBaseline(baseline);
        }
        self.base.container_builder_.SetLastBaseline(baseline);
    }

    // cpp: layoutng_block/block_layout_algorithm.h:390-392
    // cpp: layoutng_block/block_layout_algorithm.cc:3615-3663
    pub fn PropagateBaselineFromBlockChild(
        &mut self,
        child: &PhysicalFragment,
        margins: &BoxStrut,
        block_offset: LayoutUnit,
    ) {
        debug_assert!(child.IsBox());
        let baseline_algorithm = self.GetConstraintSpace().GetBaselineAlgorithmType();
        if child.IsTable() && baseline_algorithm == BaselineAlgorithmType::kInlineBlock {
            return;
        }
        if self.line_clamp_data_.IsPastClampPoint() {
            return;
        }
        let physical_fragment = unsafe { &*To::<PhysicalBoxFragment>(child as *const _) };
        let fragment = LogicalBoxFragment::new(
            self.GetConstraintSpace().GetWritingDirection(),
            physical_fragment,
        );
        if self.base.container_builder_.FirstBaseline().is_none() {
            if let Some(first_baseline) = fragment.FirstBaseline() {
                self.base
                    .container_builder_
                    .SetFirstBaseline(block_offset + first_baseline);
            }
        }
        let use_last_baseline = baseline_algorithm == BaselineAlgorithmType::kDefault
            || physical_fragment.UseLastBaselineForInlineBaseline();
        let mut last_baseline = if use_last_baseline {
            fragment.LastBaseline()
        } else {
            fragment.FirstBaseline()
        };
        if baseline_algorithm == BaselineAlgorithmType::kInlineBlock
            && physical_fragment.ForceInlineBaselineSynthesis()
            && fragment.IsWritingModeEqual()
        {
            last_baseline = Some(fragment.BlockSize() + margins.block_end);
        }
        if let Some(last_baseline) = last_baseline {
            self.base
                .container_builder_
                .SetLastBaseline(block_offset + last_baseline);
        }
    }

    // cpp: layoutng_block/block_layout_algorithm.h:385-385
    // cpp: layoutng_block/block_layout_algorithm.cc:3557-3575
    pub fn BaselineForEmptyLine(&self) -> Option<LayoutUnit> {
        let first_line_style = self.Node().FirstLineStyle();
        let font_data = unsafe { &*first_line_style.GetFont() }.PrimaryFont();
        if font_data.is_null() {
            return None;
        }
        let font_metrics = unsafe { &*font_data }.GetFontMetrics();
        let baseline_type = first_line_style.GetFontBaseline();
        let line_height = first_line_style.ComputedLineHeightAsFixed();
        let offset = if first_line_style.IsFlippedLinesWritingMode() {
            font_metrics.DescentFor(baseline_type)
        } else {
            font_metrics.AscentFor(baseline_type)
        };
        let baseline: LayoutUnit = self.Borders().block_start
            + self.Padding().block_start
            + LayoutUnit::from_signed(offset)
            + (line_height - LayoutUnit::from_signed(font_metrics.Height())) / 2;
        Some(LayoutUnit::from_signed(baseline.ToInt()))
    }

    // cpp: layoutng_block/block_layout_algorithm.h:220-226
    pub fn NextBorderEdge(&self, previous: &PreviousInflowPosition) -> LayoutUnit {
        self.BfcBlockOffset() + previous.logical_block_offset + previous.margin_strut.Sum()
    }

    // cpp: layoutng_block/block_layout_algorithm.h:228-231
    pub fn PercentageSizeForChild(&self, child: &LayoutInputNode) -> LogicalSize {
        if child.IsReplaced() {
            self.replaced_child_percentage_size_
        } else {
            self.child_percentage_size_
        }
    }

    // cpp: layoutng_block/block_layout_algorithm.h:232-234
    // cpp: layoutng_block/block_layout_algorithm.cc:3191-3268
    pub fn CalculateMargins(
        &self,
        child: LayoutInputNode,
        is_new_fc: bool,
        additional_line_offset: &mut LayoutUnit,
    ) -> BoxStrut {
        debug_assert!(child.is_non_null());
        if child.IsInline() {
            return BoxStrut::default();
        }
        let child_style = child.Style();
        let mut margins = ComputeMarginsForInlineSize(
            child_style,
            self.child_percentage_size_.inline_size,
            self.GetConstraintSpace().GetWritingDirection(),
        );
        if is_new_fc {
            return margins;
        }

        let child_inline_size = Cell::new(None::<LayoutUnit>);
        let ChildInlineSize = || -> LayoutUnit {
            if let Some(size) = child_inline_size.get() {
                return size;
            }
            let mut builder = ConstraintSpaceBuilder::new(
                self.GetConstraintSpace(),
                child_style.GetWritingDirection(),
                false,
            );
            builder.SetAvailableSize(*self.ChildAvailableSize());
            builder.SetPercentageResolutionSize(self.child_percentage_size_);
            let default_alignment = StyleSelfAlignmentData::new_nonlegacy(
                ItemPosition::kNormal,
                OverflowAlignment::kDefault,
            );
            let justify_self = child_style
                .ResolvedJustifySelf(&default_alignment, self.Style())
                .GetPosition();
            if child.IsAnonymousBlockFlow() {
                builder.SetInlineAutoBehavior(AutoSizeBehavior::kStretchImplicit);
            } else if justify_self == ItemPosition::kStretch {
                builder.SetInlineAutoBehavior(AutoSizeBehavior::kStretchExplicit);
            } else if justify_self != ItemPosition::kNormal {
                builder.SetInlineAutoBehavior(AutoSizeBehavior::kFitContent);
            } else {
                builder.SetInlineAutoBehavior(AutoSizeBehavior::kStretchImplicit);
            }
            let space = builder.ToConstraintSpace();
            let block_child = BlockNode::from(child.clone());
            let child_border_padding =
                ComputeBorders(&space, &block_child) + ComputePadding(&space, child_style);
            let size = ComputeInlineSizeForFragmentWithOverride(
                &space,
                &block_child,
                &child_border_padding,
                None,
            );
            child_inline_size.set(Some(size));
            size
        };

        let style = self.Style();
        let is_rtl = IsRtl(style.Direction());
        let available_space = self.ChildAvailableSize().inline_size;
        let mut text_align_offset = LayoutUnit::default();
        if child_style.MarginInlineStartUsing(style).IsAuto()
            || child_style.MarginInlineEndUsing(style).IsAuto()
        {
            ResolveInlineAutoMargins(
                child_style,
                style,
                available_space,
                ChildInlineSize(),
                &mut margins,
            );
        } else {
            text_align_offset = WebkitTextAlignAndJustifySelfOffset(
                child_style,
                style,
                available_space,
                &margins,
                ChildInlineSize,
            );
        }
        *additional_line_offset = if is_rtl {
            self.ChildAvailableSize().inline_size
                - text_align_offset
                - ChildInlineSize()
                - margins.InlineSum()
        } else {
            text_align_offset
        };
        margins
    }

    // cpp: layoutng_block/block_layout_algorithm.h:251-254
    // cpp: layoutng_block/block_layout_algorithm.cc:2672-2735
    pub fn ComputeChildData(
        &mut self,
        previous: &PreviousInflowPosition,
        child: LayoutInputNode,
        child_break_token: *const BreakToken,
        is_new_fc: bool,
    ) -> InflowChildData {
        debug_assert!(child.is_non_null());
        debug_assert!(!child.IsFloating());
        debug_assert_eq!(is_new_fc, child.CreatesNewFormattingContext());
        let mut additional_line_offset = LayoutUnit::default();
        let mut margins =
            self.CalculateMargins(child.clone(), is_new_fc, &mut additional_line_offset);
        let mut margin_strut = previous.margin_strut.clone();
        let mut logical_block_offset = previous.logical_block_offset;
        let child_block_break_token = DynamicTo::<BlockBreakToken>(child_break_token);
        if !child_block_break_token.is_null() {
            AdjustMarginsForFragmentation(child_block_break_token, &mut margins);
            let token = unsafe { &*child_block_break_token };
            if token.IsForcedBreak() {
                margin_strut = MarginStrut::default();
            }
            if token.MonolithicOverflow() != LayoutUnit::default()
                && (self.Node().IsPaginatedRoot()
                    || unsafe { &*self.GetBreakToken() }.MonolithicOverflow()
                        == LayoutUnit::default())
            {
                logical_block_offset += token.MonolithicOverflow();
            }
        }
        margin_strut.Append(
            &margins.block_start,
            child.Style().HasMarginBlockStartQuirk(),
        );
        if self.is_relayout_for_margin_end_trim_
            && !self.pending_margin_end_trim_child_.is_non_null()
        {
            margin_strut = MarginStrut::default();
        }
        if child.IsBlock() {
            self.SetSubtreeModifiedMarginStrutIfNeeded(Some(child.Style().MarginBlockStart()));
        }
        let direction = self.GetConstraintSpace().Direction();
        let child_bfc_offset = BfcOffset::new(
            self.GetConstraintSpace().GetBfcOffset().line_offset
                + self.BorderScrollbarPadding().LineLeft(direction)
                + additional_line_offset
                + margins.LineLeft(direction),
            self.BfcBlockOffset() + logical_block_offset,
        );
        InflowChildData::new(child_bfc_offset, &margin_strut, &margins, false)
    }

    // cpp: layoutng_block/block_layout_algorithm.h:433-443
    pub fn SetSubtreeModifiedMarginStrutIfNeeded(&mut self, margin: Option<&Length>) {
        if self.base.container_builder_.BfcBlockOffset().is_some() {
            return;
        }
        if margin.is_some_and(Length::IsZero) {
            return;
        }
        self.base.container_builder_.SetSubtreeModifiedMarginStrut();
    }

    // cpp: layoutng_block/block_layout_algorithm.h:479-480
    // cpp: layoutng_block/block_layout_algorithm_node_access.cc:10-22
    pub fn ShouldPlaceUnpositionedListMarker(&self) -> bool {
        if !self.base.node_.IsListItem() {
            return false;
        }
        if !self.GetConstraintSpace().IsAnonymous() {
            return true;
        }
        debug_assert!(unsafe { &*self.base.node_.GetLayoutBox() }.IsMulticolContainer());
        false
    }
}

// The C++ registration template constructs a fresh block algorithm for each
// layout or measurement call; this is the Rust entry used by assembly.cc.
impl NativeAlgorithm for BlockLayoutAlgorithm {
    fn new(params: &LayoutAlgorithmParams) -> Self {
        BlockLayoutAlgorithm::new(params)
    }

    fn layout(&mut self) -> *const LayoutResult {
        self.Layout()
    }

    fn compute_min_max_sizes(&mut self, input: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        self.ComputeMinMaxSizes(input)
    }
}

// cpp: layoutng_block/block_layout_algorithm.h:161-175
impl RelayoutAlgorithm<BlockNode> for BlockLayoutAlgorithm {
    // cpp: layoutng_block/block_layout_algorithm.cc:338-401
    // Relayout invokes the concrete algorithm's setup, including line-clamp
    // state. The base-only default leaves the original clamp active forever.
    fn setup_relayout_data(&mut self, previous: &Self, relayout_type: RelayoutType) {
        self.SetupRelayoutData(previous, relayout_type);
    }

    fn base(&self) -> &LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken> {
        &self.base
    }

    fn base_mut(&mut self) -> &mut LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken> {
        &mut self.base
    }

    unsafe fn from_base<'a>(
        base: &'a LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>,
    ) -> &'a Self {
        &*(base as *const _ as *const Self)
    }
}
