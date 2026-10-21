#![allow(non_snake_case)]

use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use foundation::evaluation_input::CalcSizeKeywordBehavior;
use foundation::{
    kIndefiniteSize, kNotFound, DynamicTo, EBoxAlignment, EBoxDecorationBreak, EBoxOrient,
    EBoxSizing, EBreakBetween, EReadingFlow, HashMap, HeapVector, IsHorizontalWritingMode,
    LayoutUnit, Length, MakeGarbageCollected, Member, PhysicalRect, To, WtfSizeT,
};
use layoutng_assembly::block_break_token::BlockBreakToken;
use layoutng_assembly::box_fragment_builder::BoxFragmentBuilder;
use layoutng_assembly::internal::algorithm_entry::NativeAlgorithm;
use layoutng_assembly::internal::baseline_utils::BaselineGroup;
use layoutng_assembly::internal::baseline_utils::{
    DetermineBaselineGroup, DetermineBaselineWritingMode,
};
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::break_appeal::BreakAppeal;
use layoutng_assembly::internal::constraint_space::{
    AutoSizeBehavior, ConstraintSpace, LayoutResultCacheSlot,
};
use layoutng_assembly::internal::constraint_space_builder::ConstraintSpaceBuilder;
use layoutng_assembly::internal::constraint_space_builder_style::MinMaxConstraintSpaceBuilder;
use layoutng_assembly::internal::devtools_flex_info::{
    DevtoolsFlexInfo, DevtoolsFlexInfoItem, DevtoolsFlexInfoLine,
};
use layoutng_assembly::internal::disable_layout_side_effects_scope::DisableLayoutSideEffectsScope;
use layoutng_assembly::internal::early_break::{BreakType, EarlyBreak};
use layoutng_assembly::internal::fragmentation_utils::{
    AttemptSoftBreak, BreakBeforeChild, BreakBeforeChildIfNeededFull, BreakStatus,
    CalculateBreakAppealBeforeStatus, ClonedBlockStartDecoration, EnterEarlyBreakInChild,
    FinishFragmentation, FlexColumnBreakInfo, InvolvedInBlockFragmentationForBuilder,
    IsBreakInside, IsBreakableAtStartOfResumedContainer, IsEarlyBreakTarget, IsForcedBreakValue,
    JoinFragmentainerBreakValues, SetupSpaceBuilderForFragmentationFromBuilder,
};
use layoutng_assembly::internal::gap::gap_geometry::FlexGapPlacementReversal;
use layoutng_assembly::internal::layout_algorithm::{
    LayoutAlgorithm, LayoutAlgorithmParams, RelayoutAlgorithm, RelayoutType,
};
use layoutng_assembly::internal::layout_input_node::{LayoutInputNode, MinMaxSizesFloatInput};
use layoutng_assembly::internal::layout_node_metadata::Node;
use layoutng_assembly::internal::layout_pass_scope::LayoutPassScope;
use layoutng_assembly::internal::length_utils::{
    BlockSizeFromAspectRatio, CalculateChildPercentageSize, CalculateInitialFragmentGeometry,
    CalculateIntrinsicBlockSizeIgnoringChildren, CalculateMinMaxSizesIgnoringChildren,
    ClampIntrinsicBlockSize, ComputeBlockSizeForFragment, ComputeBorders,
    ComputeInitialMinMaxBlockSizes, ComputeInlineSizeForFragment, ComputeMarginsFor,
    ComputeMinAndMaxContentContribution, ComputeMinMaxBlockSizes, ComputeMinMaxInlineSizes,
    ComputePadding, ComputePhysicalMarginsForSpace, ComputeReplacedSize,
    ComputeScrollbarsForNonAnonymous, ComputeTransferredMinMaxBlockSizes, FitContentMode,
    ReplacedSizeMode, ResolveMainBlockLength, ResolveMainInlineLength, ShrinkLogicalSize, SizeType,
    TransferredSizesMode,
};
use layoutng_assembly::internal::min_max_sizes::{MinMaxSizes, MinMaxSizesResult};
use layoutng_assembly::internal::scroll_layout_scope::{
    DelayScrollOffsetClampScope, FreezeScrollbarsScope,
};
use layoutng_assembly::internal::space_utils::SetOrthogonalFallbackInlineSizeIfNeeded;
use layoutng_assembly::internal::table_node::TableNode;
use layoutng_assembly::layout_result::{EStatus, LayoutResult};
use layoutng_assembly::logical_box_fragment::LogicalBoxFragment;
use layoutng_assembly::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::logical_size::{LogicalSize, ToPhysicalSize};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_geometry::geometry::static_position::{
    BlockEdge, InlineEdge, LogicalAlignmentDirection, LogicalStaticPosition,
};
use layoutng_style::style::computed_style_constants::ItemPosition;
use layoutng_style::style::computed_style_constants::{
    ContentDistributionType, ContentPosition, OverflowAlignment,
};
use layoutng_style::style::style_self_alignment_data::StyleSelfAlignmentData;

use crate::flex_break_token_data::{FlexBreakBeforeRow, FlexBreakTokenData, FlexGapBreakTokenData};
use crate::flex_child_iterator::FlexChildIterator;
use crate::flex_gap_accumulator::FlexGapAccumulator;
use crate::flex_item::FlexItem;
use crate::flex_item_iterator::FlexItemIterator;
use crate::flex_line::{FlexItemData, FlexLine, FlexLineVector};
use crate::flex_line_breaker::{BreakFlexItemsIntoLines, FlexLineBreakerResult, InitialFlexLine};
use crate::line_flexer::LineFlexer;
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_geometry::geometry::layout_unit_diffuser::LayoutUnitDiffuser;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;

// cpp: layoutng_flex/flex_layout_algorithm.cc:612-628
fn AppendReadingFlowLine(
    nodes: &mut HeapVector<Member<Node>>,
    items: &HeapVector<FlexItem, 4>,
    line: &FlexLine,
    reverse_items: bool,
) {
    let mut append_item = |index: WtfSizeT| {
        let node = items[index as usize].block_node.GetDOMNode();
        if !node.is_null() {
            nodes.push(Member::from_ptr(node));
        }
    };
    if reverse_items {
        for &index in line.item_indices.iter().rev() {
            append_item(index);
        }
    } else {
        for &index in &line.item_indices {
            append_item(index);
        }
    }
}
use crate::flex_layout_algorithm_support::{
    AxisEdge, BaselineAccumulator, ColumnGap, ContentDistributionSpace,
    CrossAxisStaticPositionEdge, InitialContentPositionOffset, MainAxisStaticPositionEdge,
    PhysicalToFlex, ResolvedAlignSelf, ResolvedJustifyContent, RowGap,
};

#[derive(Clone, Copy)]
enum PhysicalMarginSide {
    Top,
    Right,
    Bottom,
    Left,
}

impl PhysicalMarginSide {
    fn get(self, margins: &PhysicalBoxStrut) -> LayoutUnit {
        match self {
            Self::Top => margins.top,
            Self::Right => margins.right,
            Self::Bottom => margins.bottom,
            Self::Left => margins.left,
        }
    }
    fn set(self, margins: &mut PhysicalBoxStrut, value: LayoutUnit) {
        match self {
            Self::Top => margins.top = value,
            Self::Right => margins.right = value,
            Self::Bottom => margins.bottom = value,
            Self::Left => margins.left = value,
        }
    }
}

// cpp: layoutng_flex/flex_layout_algorithm.h:20-273
#[repr(C)]
pub struct FlexLayoutAlgorithm {
    base: LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>,
    flex_items_: HeapVector<FlexItem, 4>,
    largest_min_content_contribution_: LayoutUnit,
    is_webkit_box_: bool,
    is_column_: bool,
    is_wrap_reverse_: bool,
    is_reverse_direction_: bool,
    is_multi_line_: bool,
    is_horizontal_flow_: bool,
    is_cross_size_definite_: bool,
    balance_min_line_count_: Option<WtfSizeT>,
    child_percentage_size_: LogicalSize,
    gap_between_items_: LayoutUnit,
    gap_between_lines_: LayoutUnit,
    has_column_percent_flex_basis_: bool,
    ignore_child_scrollbar_changes_: bool,
    last_line_idx_to_process_first_child_: WtfSizeT,
    has_processed_first_line_: bool,
    layout_info_for_devtools_: *mut DevtoolsFlexInfo,
    total_block_size_: LayoutUnit,
    intrinsic_block_size_: LayoutUnit,
    column_early_breaks_: HeapVector<Member<EarlyBreak>>,
    row_cross_size_updates_: HashMap<WtfSizeT, LayoutUnit>,
    cross_size_adjustments_: *const HashMap<WtfSizeT, LayoutUnit>,
}

// cpp: layoutng_flex/flex_layout_algorithm.h:35-35
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    kLayout,
    kRowIntrinsicSize,
    kColumnWrapIntrinsicSize,
}

impl Deref for FlexLayoutAlgorithm {
    type Target = LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for FlexLayoutAlgorithm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl FlexLayoutAlgorithm {
    // cpp: layoutng_flex/flex_layout_algorithm.cc:172-201
    pub fn new(params: &LayoutAlgorithmParams) -> Self {
        Self::new_with_cross_size_adjustments(params, std::ptr::null())
    }

    pub fn new_with_cross_size_adjustments(
        params: &LayoutAlgorithmParams,
        cross_size_adjustments: *const HashMap<WtfSizeT, LayoutUnit>,
    ) -> Self {
        let base = LayoutAlgorithm::from_params(params);
        let style = base.Style();
        let is_column = style.ResolvedIsColumnFlexDirection();
        let child_percentage_size = CalculateChildPercentageSize(
            base.GetConstraintSpace(),
            base.Node(),
            *base.ChildAvailableSize(),
        );
        let gap_between_items = if is_column {
            RowGap(style, child_percentage_size)
        } else {
            ColumnGap(style, child_percentage_size)
        };
        let gap_between_lines = if is_column {
            ColumnGap(style, child_percentage_size)
        } else {
            RowGap(style, child_percentage_size)
        };
        let layout_info_for_devtools = if unsafe { &*base.Node().GetLayoutBox() }
            .NeedsDevtoolsInfo()
            && !InvolvedInBlockFragmentationForBuilder(&base.container_builder_)
        {
            MakeGarbageCollected(DevtoolsFlexInfo { lines: Vec::new() })
        } else {
            std::ptr::null_mut()
        };
        Self {
            flex_items_: HeapVector::default(),
            largest_min_content_contribution_: LayoutUnit::default(),
            is_webkit_box_: style.IsDeprecatedFlexbox(),
            is_column_: is_column,
            is_wrap_reverse_: style.ResolvedIsFlexWrapReverse(),
            is_reverse_direction_: style.ResolvedIsReverseFlexDirection(),
            is_multi_line_: !style.ResolvedIsFlexNowrap(),
            is_horizontal_flow_: if style.IsHorizontalWritingMode() {
                !is_column
            } else {
                is_column
            },
            is_cross_size_definite_: IsContainerCrossSizeDefinite(&base, is_column),
            balance_min_line_count_: style.ResolvedFlexLineCount().map(|count| {
                WtfSizeT::try_from(count).expect("flex line count exceeds wtf_size_t")
            }),
            child_percentage_size_: child_percentage_size,
            gap_between_items_: gap_between_items,
            gap_between_lines_: gap_between_lines,
            has_column_percent_flex_basis_: false,
            ignore_child_scrollbar_changes_: false,
            last_line_idx_to_process_first_child_: kNotFound,
            has_processed_first_line_: false,
            layout_info_for_devtools_: layout_info_for_devtools,
            total_block_size_: LayoutUnit::default(),
            intrinsic_block_size_: LayoutUnit::default(),
            column_early_breaks_: HeapVector::default(),
            row_cross_size_updates_: HashMap::default(),
            cross_size_adjustments_: cross_size_adjustments,
            base,
        }
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:203-212
    pub fn SetupRelayoutData(&mut self, previous: &Self, relayout_type: RelayoutType) {
        self.base.SetupRelayoutData(&previous.base, relayout_type);
        if relayout_type == RelayoutType::kRelayoutIgnoringChildScrollbarChanges {
            self.ignore_child_scrollbar_changes_ = true;
        } else {
            self.ignore_child_scrollbar_changes_ = previous.ignore_child_scrollbar_changes_;
        }
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:1220-1241
    pub fn Layout(&mut self) -> *const LayoutResult {
        let result = self.LayoutInternal();
        match unsafe { &*result }.Status() {
            EStatus::kNeedsEarlierBreak => {
                let breakpoint = unsafe { &*result }.GetEarlyBreak();
                debug_assert!(!breakpoint.is_null());
                let additional =
                    &self.column_early_breaks_ as *const HeapVector<Member<EarlyBreak>>;
                self.base
                    .RelayoutAndBreakEarlier::<Self>(unsafe { &*breakpoint }, additional)
            }
            EStatus::kNeedsRelayoutWithNoChildScrollbarChanges => {
                debug_assert!(!self.ignore_child_scrollbar_changes_);
                self.base
                    .RelayoutDefault::<Self>(RelayoutType::kRelayoutIgnoringChildScrollbarChanges)
            }
            EStatus::kDisableFragmentation => {
                debug_assert!(self.GetConstraintSpace().HasBlockFragmentation());
                self.base.RelayoutWithoutFragmentation::<Self>()
            }
            EStatus::kNeedsRelayoutWithRowCrossSizeChanges => self.RelayoutWithNewRowSizes(),
            _ => result,
        }
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:1243-1431
    pub fn LayoutInternal(&mut self) -> *const LayoutResult {
        let freeze_scrollbars = if self.ignore_child_scrollbar_changes_ {
            Some(FreezeScrollbarsScope::new())
        } else {
            None
        };
        let _delay_clamp_scope = DelayScrollOffsetClampScope::new();
        let mut row_break_between_outputs = Vec::<EBreakBetween>::new();
        let mut flex_lines = FlexLineVector::default();
        let mut oof_children = HeapVector::default();
        let mut break_before_row = FlexBreakBeforeRow::kNotBreakBeforeRow;
        let mut total_intrinsic_block_size = LayoutUnit::default();
        let mut current_gap_data = FlexGapBreakTokenData {
            effective_gap_between_lines: self.gap_between_lines_,
            total_row_gap_count: 0,
            gap_data_for_rows: Vec::new(),
        };
        let mut previous_gap_data = None;

        if IsBreakInside(self.GetBreakToken()) {
            let flex_data =
                unsafe { &*To::<FlexBreakTokenData>((&*self.GetBreakToken()).TokenData()) };
            total_intrinsic_block_size = flex_data.intrinsic_block_size;
            flex_lines = flex_data.flex_lines.clone();
            row_break_between_outputs = flex_data.row_break_between.clone();
            break_before_row = flex_data.break_before_row;
            oof_children = flex_data.oof_children.clone();
            previous_gap_data = Some(flex_data.gap_data.clone());
            current_gap_data.effective_gap_between_lines =
                flex_data.gap_data.effective_gap_between_lines;
            current_gap_data.total_row_gap_count = flex_data.gap_data.total_row_gap_count;
        } else {
            self.PlaceFlexItems(
                Phase::kLayout,
                &mut flex_lines,
                Some(&mut oof_children),
                Some(&mut total_intrinsic_block_size),
            );
        }

        self.total_block_size_ = ComputeBlockSizeForFragment(
            self.GetConstraintSpace(),
            self.Node(),
            self.BorderPadding(),
            total_intrinsic_block_size,
            self.base.container_builder_.InlineSize(),
            kIndefiniteSize,
        );
        if !IsBreakInside(self.GetBreakToken()) {
            self.ApplyReversals(&mut flex_lines);
        }
        let mut gap_accumulator = if self.Style().HasGapRule() && !flex_lines.is_empty() {
            let placement_reversal = if self.is_wrap_reverse_ || self.is_reverse_direction_ {
                Some(FlexGapPlacementReversal::new(
                    self.is_wrap_reverse_,
                    self.is_reverse_direction_,
                ))
            } else {
                None
            };
            Some(FlexGapAccumulator::new(
                self.gap_between_items_,
                self.gap_between_lines_,
                flex_lines.len() as WtfSizeT,
                self.flex_items_.len() as WtfSizeT,
                self.is_column_,
                self.base
                    .container_builder_
                    .BorderScrollbarPadding()
                    .block_start,
                self.base
                    .container_builder_
                    .BorderScrollbarPadding()
                    .inline_start,
                placement_reversal,
            ))
        } else {
            None
        };
        if !IsBreakInside(self.GetBreakToken()) {
            let mut total_row_gap_count = 0;
            let status = self.GiveItemsFinalPositionAndSize(
                &mut flex_lines,
                &mut row_break_between_outputs,
                &mut gap_accumulator,
                &mut current_gap_data.effective_gap_between_lines,
                if self.GetConstraintSpace().HasBlockFragmentation() {
                    Some(&mut total_row_gap_count)
                } else {
                    None
                },
            );
            current_gap_data.total_row_gap_count = total_row_gap_count;
            if status != EStatus::kSuccess {
                return self.base.container_builder_.Abort(status);
            }
        }

        let previously_consumed_block_size = if self.GetBreakToken().is_null() {
            LayoutUnit::default()
        } else {
            unsafe { &*self.GetBreakToken() }.ConsumedBlockSize()
        };
        self.intrinsic_block_size_ = self.BorderScrollbarPadding().block_start;
        let fragmented = InvolvedInBlockFragmentationForBuilder(&self.base.container_builder_);
        let block_size = if fragmented {
            if flex_lines.is_empty() && self.Node().HasLineIfEmpty() {
                self.intrinsic_block_size_ = (total_intrinsic_block_size
                    - self.BorderScrollbarPadding().block_end
                    - previously_consumed_block_size)
                    .ClampNegativeToZero();
            }
            if IsBreakInside(self.GetBreakToken()) {
                if let Some(accumulator) = gap_accumulator.as_mut() {
                    accumulator
                        .SetEffectiveGapBetweenLines(current_gap_data.effective_gap_between_lines);
                }
            }
            let status = self.GiveItemsFinalPositionAndSizeForFragmentation(
                &mut flex_lines,
                &mut row_break_between_outputs,
                &mut break_before_row,
                &mut total_intrinsic_block_size,
                &mut gap_accumulator,
                current_gap_data.effective_gap_between_lines,
                previous_gap_data.as_ref(),
            );
            if status != EStatus::kSuccess {
                return self.base.container_builder_.Abort(status);
            }
            if let Some(accumulator) = gap_accumulator.as_mut() {
                current_gap_data.gap_data_for_rows = accumulator.FinalizeRowGapBreakTokenData();
            }
            self.intrinsic_block_size_ = ClampIntrinsicBlockSize(
                self.GetConstraintSpace(),
                self.Node(),
                self.GetBreakToken(),
                self.BorderScrollbarPadding(),
                self.intrinsic_block_size_ + self.BorderScrollbarPadding().block_end,
                None,
            );
            ComputeBlockSizeForFragment(
                self.GetConstraintSpace(),
                self.Node(),
                self.BorderPadding(),
                previously_consumed_block_size + self.intrinsic_block_size_,
                self.base.container_builder_.InlineSize(),
                kIndefiniteSize,
            )
        } else {
            self.intrinsic_block_size_ = total_intrinsic_block_size;
            self.total_block_size_
        };
        self.base
            .container_builder_
            .SetIntrinsicBlockSize(self.intrinsic_block_size_);
        self.base
            .container_builder_
            .SetFragmentsTotalBlockSize(block_size);
        if self.has_column_percent_flex_basis_ {
            self.base
                .container_builder_
                .SetHasDescendantThatDependsOnPercentageBlockSize(true);
        }
        if !self.layout_info_for_devtools_.is_null() {
            self.base
                .container_builder_
                .SetFlexLayoutData(self.layout_info_for_devtools_);
        }
        if fragmented {
            match FinishFragmentation(&mut self.base.container_builder_) {
                BreakStatus::kContinue => {}
                BreakStatus::kNeedsEarlierBreak => {
                    return self
                        .base
                        .container_builder_
                        .Abort(EStatus::kNeedsEarlierBreak)
                }
                BreakStatus::kDisableFragmentation => {
                    return self
                        .base
                        .container_builder_
                        .Abort(EStatus::kDisableFragmentation)
                }
                other => panic!("unexpected flex fragmentation status: {other:?}"),
            }
        } else {
            #[cfg(debug_assertions)]
            self.base.container_builder_.CheckNoBlockFragmentation();
        }
        self.SetReadingFlowNodes(&flex_lines);
        self.HandleOutOfFlowPositionedItems(total_intrinsic_block_size, &mut oof_children);
        if self.GetConstraintSpace().ShouldPropagateChildBreakValues() {
            debug_assert!(!row_break_between_outputs.is_empty());
            self.base
                .container_builder_
                .SetInitialBreakBefore(row_break_between_outputs[0]);
            self.base
                .container_builder_
                .SetPreviousBreakAfter(*row_break_between_outputs.last().unwrap());
        }
        if self.GetConstraintSpace().HasBlockFragmentation() {
            let data = MakeGarbageCollected(FlexBreakTokenData::new(
                &flex_lines,
                &row_break_between_outputs,
                &oof_children,
                total_intrinsic_block_size,
                break_before_row,
                current_gap_data,
            ));
            self.base.container_builder_.SetBreakTokenData(data.cast());
        }
        drop(freeze_scrollbars);
        self.base
            .container_builder_
            .HandleOofsAndSpecialDescendants();
        if let Some(accumulator) = gap_accumulator.as_mut() {
            let geometry = accumulator.BuildGapGeometry(&self.base.container_builder_);
            self.base.container_builder_.SetGapGeometry(geometry);
        }
        self.base.container_builder_.ToBoxFragment()
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:339-373
    pub fn MainAxisContentExtent(&self, sum_hypothetical_main_size: LayoutUnit) -> LayoutUnit {
        if self.is_column_ {
            let border_scrollbar_padding = self.BorderScrollbarPadding().BlockSum();
            let intrinsic_size = if sum_hypothetical_main_size == kIndefiniteSize {
                kIndefiniteSize
            } else {
                sum_hypothetical_main_size + border_scrollbar_padding
            };
            let block_size = ComputeBlockSizeForFragment(
                self.GetConstraintSpace(),
                self.Node(),
                self.BorderPadding(),
                intrinsic_size,
                self.base.container_builder_.InlineSize(),
                kIndefiniteSize,
            );
            if block_size != kIndefiniteSize {
                return (block_size - border_scrollbar_padding).ClampNegativeToZero();
            }
            let max_block_size = ComputeInitialMinMaxBlockSizes(
                self.GetConstraintSpace(),
                self.Node(),
                self.BorderPadding(),
                kIndefiniteSize,
            )
            .max_size;
            if max_block_size != LayoutUnit::Max() {
                return (max_block_size - border_scrollbar_padding).ClampNegativeToZero();
            }
            return LayoutUnit::Max();
        }
        self.ChildAvailableSize().inline_size
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:375-398
    pub fn BaselineAscent(&self, item: &FlexItem, fragment: &PhysicalBoxFragment) -> LayoutUnit {
        let baseline_fragment = LogicalBoxFragment::new(item.baseline_writing_direction, fragment);
        let is_last_baseline = item.alignment == ItemPosition::kLastBaseline;
        let font_baseline = self.Style().GetFontBaseline();
        let mut baseline = if is_last_baseline {
            baseline_fragment.LastBaselineOrSynthesize(font_baseline)
        } else {
            baseline_fragment.FirstBaselineOrSynthesize(font_baseline)
        };
        if self.is_wrap_reverse_ != is_last_baseline {
            baseline = baseline_fragment.BlockSize() - baseline;
        }
        let margins = PhysicalToFlex::new(
            self.GetConstraintSpace().GetWritingDirection(),
            self.is_column_,
            item.initial_margins.top,
            item.initial_margins.right,
            item.initial_margins.bottom,
            item.initial_margins.left,
        );
        if item.baseline_group == BaselineGroup::kMajor {
            margins.CrossStart() + baseline
        } else {
            margins.CrossEnd() + baseline
        }
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:400-420
    pub fn SynthesizedBaselineAscent(&self, item: &FlexItem, block_size: LayoutUnit) -> LayoutUnit {
        let is_last_baseline = item.alignment == ItemPosition::kLastBaseline;
        let font_baseline = self.Style().GetFontBaseline();
        let mut baseline = LogicalBoxFragment::SynthesizedBaseline(
            font_baseline,
            item.baseline_writing_direction.IsFlippedLines(),
            block_size,
        );
        if self.is_wrap_reverse_ != is_last_baseline {
            baseline = block_size - baseline;
        }
        let margins = PhysicalToFlex::new(
            self.GetConstraintSpace().GetWritingDirection(),
            self.is_column_,
            item.initial_margins.top,
            item.initial_margins.right,
            item.initial_margins.bottom,
            item.initial_margins.left,
        );
        if item.baseline_group == BaselineGroup::kMajor {
            margins.CrossStart() + baseline
        } else {
            margins.CrossEnd() + baseline
        }
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:422-446
    pub fn ShouldApplyAutoMinSize(&self, child: &BlockNode) -> bool {
        if self.is_webkit_box_ || child.ShouldApplySizeContainment() {
            return false;
        }
        let child_style = child.Style();
        if if self.is_horizontal_flow_ {
            child_style.IsOverflowValueScrollableX()
        } else {
            child_style.IsOverflowValueScrollableY()
        } {
            return false;
        }
        let min = if self.is_horizontal_flow_ {
            child_style.MinWidth()
        } else {
            child_style.MinHeight()
        };
        min.HasAuto()
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:602-641
    pub fn SetReadingFlowNodes(&mut self, flex_lines: &FlexLineVector) {
        let reading_flow = self.Style().ReadingFlow();
        if !matches!(
            reading_flow,
            EReadingFlow::kFlexVisual | EReadingFlow::kFlexFlow
        ) {
            return;
        }
        let mut nodes: HeapVector<Member<Node>> = HeapVector::default();
        nodes.reserve(self.flex_items_.len());
        let reverse_items = reading_flow == EReadingFlow::kFlexFlow && self.is_reverse_direction_;
        let reverse_lines = reading_flow == EReadingFlow::kFlexFlow && self.is_wrap_reverse_;
        if reverse_lines {
            for line in flex_lines.iter().rev() {
                AppendReadingFlowLine(&mut nodes, &self.flex_items_, line, reverse_items);
            }
        } else {
            for line in flex_lines.iter() {
                AppendReadingFlowLine(&mut nodes, &self.flex_items_, line, reverse_items);
            }
        }
        self.base.container_builder_.SetReadingFlowNodes(nodes);
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:495-600
    pub fn HandleOutOfFlowPositionedItems(
        &mut self,
        total_intrinsic_block_size: LayoutUnit,
        oof_children: &mut HeapVector<Member<layoutng_assembly::internal::layout_box::LayoutBox>>,
    ) {
        if oof_children.is_empty() {
            return;
        }
        let oofs = std::mem::take(oof_children);
        let break_token = self.GetBreakToken();
        let previous_consumed_block_size = if break_token.is_null() {
            LayoutUnit::default()
        } else {
            unsafe { &*break_token }.ConsumedBlockSize()
        };
        let mut should_process_block_end = true;
        let mut should_process_block_center = true;
        if InvolvedInBlockFragmentationForBuilder(&self.base.container_builder_) {
            should_process_block_end = !self.base.container_builder_.DidBreakSelf()
                && !self.base.container_builder_.ShouldBreakInside();
            if should_process_block_end {
                self.total_block_size_ = ComputeBlockSizeForFragment(
                    self.GetConstraintSpace(),
                    self.Node(),
                    self.BorderPadding(),
                    total_intrinsic_block_size,
                    self.base.container_builder_.InlineSize(),
                    kIndefiniteSize,
                );
            } else {
                let center = self.total_block_size_ / 2;
                should_process_block_center = center - previous_consumed_block_size
                    <= self.FragmentainerCapacityForChildren();
            }
        }
        let mut border_scrollbar_padding = *self.BorderScrollbarPadding();
        border_scrollbar_padding.block_start = self.OriginalBorderScrollbarPaddingBlockStart();
        let total_fragment_size = ShrinkLogicalSize(
            LogicalSize::new(
                self.base.container_builder_.InlineSize(),
                self.total_block_size_,
            ),
            &border_scrollbar_padding,
        );
        let justify_content = ResolvedJustifyContent(
            self.Style(),
            self.GetConstraintSpace().GetWritingDirection(),
            self.is_column_,
        );
        let main_axis_edge =
            MainAxisStaticPositionEdge(&justify_content, self.is_reverse_direction_);
        for oof in oofs {
            let child = BlockNode::new(oof.Get());
            let position = ResolvedAlignSelf(
                self.Style(),
                child.Style(),
                self.GetConstraintSpace().GetWritingDirection(),
                self.is_column_,
                true,
            );
            let cross_axis_edge = CrossAxisStaticPositionEdge(position, self.is_wrap_reverse_);
            let inline_axis_edge = if self.is_column_ {
                cross_axis_edge
            } else {
                main_axis_edge
            };
            let block_axis_edge = if self.is_column_ {
                main_axis_edge
            } else {
                cross_axis_edge
            };
            let mut static_pos =
                LogicalStaticPosition::from_offset(border_scrollbar_padding.StartOffset());
            match block_axis_edge {
                AxisEdge::kStart => {
                    debug_assert!(!IsBreakInside(break_token));
                    static_pos.block_edge = BlockEdge::kBlockStart;
                }
                AxisEdge::kCenter => {
                    if !should_process_block_center {
                        oof_children.push(oof);
                        continue;
                    }
                    static_pos.block_edge = BlockEdge::kBlockCenter;
                    static_pos.offset.block_offset += total_fragment_size.block_size / 2;
                }
                AxisEdge::kEnd => {
                    if !should_process_block_end {
                        oof_children.push(oof);
                        continue;
                    }
                    static_pos.block_edge = BlockEdge::kBlockEnd;
                    static_pos.offset.block_offset += total_fragment_size.block_size;
                }
            }
            match inline_axis_edge {
                AxisEdge::kStart => static_pos.inline_edge = InlineEdge::kInlineStart,
                AxisEdge::kCenter => {
                    static_pos.inline_edge = InlineEdge::kInlineCenter;
                    static_pos.offset.inline_offset += total_fragment_size.inline_size / 2;
                }
                AxisEdge::kEnd => {
                    static_pos.inline_edge = InlineEdge::kInlineEnd;
                    static_pos.offset.inline_offset += total_fragment_size.inline_size;
                }
            }
            static_pos.offset.block_offset -= previous_consumed_block_size;
            static_pos.align_self_direction = if self.is_column_ {
                LogicalAlignmentDirection::kInline
            } else {
                LogicalAlignmentDirection::kBlock
            };
            self.base
                .container_builder_
                .AddOutOfFlowChildCandidateDefault(&child, &static_pos);
        }
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:809-1218
    pub fn ConstructAndAppendFlexItems(
        &mut self,
        phase: Phase,
        mut oof_children: Option<
            &mut HeapVector<Member<layoutng_assembly::internal::layout_box::LayoutBox>>,
        >,
    ) {
        let mut item_index: WtfSizeT = 0;
        let mut iterator = FlexChildIterator::new(self.Node().clone());
        self.flex_items_.reserve(iterator.size() as usize);
        let total_child_count = iterator.size();
        loop {
            let child = iterator.NextChild();
            if child.IsNull() {
                break;
            }
            if child.IsOutOfFlowPositioned() {
                if phase == Phase::kLayout {
                    oof_children
                        .as_deref_mut()
                        .expect("layout phase requires out-of-flow children")
                        .push(Member::from_ptr(child.GetLayoutBox()));
                }
                continue;
            }
            let (item, contribution, has_percent_basis) =
                self.ConstructFlexItem(child, item_index, total_child_count, phase);
            self.largest_min_content_contribution_ =
                self.largest_min_content_contribution_.max(contribution);
            self.has_column_percent_flex_basis_ |= has_percent_basis;
            self.flex_items_.push(item);
            item_index += 1;
        }
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:826-1218
    fn ConstructFlexItem(
        &self,
        child: BlockNode,
        item_index: WtfSizeT,
        total_child_count: WtfSizeT,
        phase: Phase,
    ) -> (FlexItem, LayoutUnit, bool) {
        let child_style = child.Style();
        let flex_grow = child_style.ResolvedFlexGrow(self.Style());
        let flex_shrink = child_style.ResolvedFlexShrink(self.Style());
        let alignment = ResolvedAlignSelf(
            self.Style(),
            child_style,
            self.GetConstraintSpace().GetWritingDirection(),
            self.is_column_,
            false,
        );
        let mut max_content_contribution = None;
        let mut min_content_contribution = LayoutUnit::default();
        if phase == Phase::kColumnWrapIntrinsicSize {
            let space = self.BuildSpaceForIntrinsicInlineSize(&child, alignment);
            let mut contributions = ComputeMinAndMaxContentContribution(
                self.Style(),
                &child,
                &space,
                MinMaxSizesFloatInput::default(),
            );
            max_content_contribution = Some(contributions.sizes.max_size);
            let margins = ComputeMarginsFor(&space, child_style, self.GetConstraintSpace());
            contributions.sizes += margins.InlineSum();
            min_content_contribution = contributions.sizes.min_size;
        }
        let child_writing_mode = child_style.GetWritingMode();
        let is_main_axis_inline_axis =
            IsHorizontalWritingMode(child_writing_mode) == self.is_horizontal_flow_;
        let flex_basis_space = self.BuildSpaceForFlexBasis(&child);
        let physical_child_margins = ComputePhysicalMarginsForSpace(&flex_basis_space, child_style);
        let border_padding = ComputeBorders(&flex_basis_space, &child)
            + ComputePadding(&flex_basis_space, child_style);
        let physical_border_padding =
            border_padding.ConvertToPhysical(child_style.GetWritingDirection());
        let main_axis_auto_margin_count = if self.is_horizontal_flow_ {
            child_style.MarginLeft().IsAuto() as u8 + child_style.MarginRight().IsAuto() as u8
        } else {
            child_style.MarginTop().IsAuto() as u8 + child_style.MarginBottom().IsAuto() as u8
        };
        let main_axis_border_padding = if self.is_horizontal_flow_ {
            physical_border_padding.HorizontalSum()
        } else {
            physical_border_padding.VerticalSum()
        };
        let child_space = self.BuildSpaceForLayout(
            &child,
            alignment,
            self.is_column_ && !is_main_axis_inline_axis,
            max_content_contribution,
            None,
            None,
            None,
            false,
        );
        let depends_on_min_max_sizes = Cell::new(false);
        let min_max_sizes_func = |type_: SizeType| {
            depends_on_min_max_sizes.set(true);
            child.ComputeMinMaxSizes(
                child_writing_mode,
                type_,
                &child_space,
                MinMaxSizesFloatInput::default(),
            )
        };
        let inline_size_func = || {
            CalculateInitialFragmentGeometry(&child_space, &child, std::ptr::null(), false)
                .border_box_size
                .inline_size
        };
        let layout_result: Cell<*const LayoutResult> = Cell::new(std::ptr::null());
        let block_size_func = |type_: SizeType| -> LayoutUnit {
            if child.IsReplaced() {
                return ComputeReplacedSize(
                    &child,
                    &child_space,
                    &border_padding,
                    ReplacedSizeMode::kIgnoreBlockLengths,
                )
                .block_size;
            }
            let has_aspect_ratio = !child_style.AspectRatio().IsAuto();
            if has_aspect_ratio && type_ == SizeType::kContent {
                let inline_size = inline_size_func();
                if inline_size != kIndefiniteSize {
                    return BlockSizeFromAspectRatio(
                        &border_padding,
                        &child_style.LogicalAspectRatio(),
                        child_style.BoxSizingForAspectRatio(),
                        inline_size,
                    );
                }
            }
            let mut intrinsic_size = CalculateIntrinsicBlockSizeIgnoringChildren(
                &child,
                &(border_padding + ComputeScrollbarsForNonAnonymous(&child)),
                false,
            );
            if intrinsic_size == kIndefiniteSize {
                if layout_result.get().is_null() {
                    let _disable = if phase != Phase::kLayout
                        && !unsafe { &*child.GetLayoutBox() }.NeedsLayout()
                    {
                        Some(DisableLayoutSideEffectsScope::new())
                    } else {
                        None
                    };
                    let result = child.Layout(
                        &child_space,
                        std::ptr::null(),
                        std::ptr::null(),
                        std::ptr::null(),
                    );
                    assert!(!result.is_null());
                    layout_result.set(result);
                }
                intrinsic_size = unsafe { &*layout_result.get() }.IntrinsicBlockSize();
            }
            if has_aspect_ratio {
                let inline_min_max = ComputeMinMaxInlineSizes(
                    &flex_basis_space,
                    &child,
                    &border_padding,
                    None,
                    &min_max_sizes_func,
                    TransferredSizesMode::kIgnore,
                    FitContentMode::kNormal,
                    kIndefiniteSize,
                );
                let min_max = ComputeTransferredMinMaxBlockSizes(
                    &child_style.LogicalAspectRatio(),
                    &inline_min_max,
                    &border_padding,
                    child_style.BoxSizingForAspectRatio(),
                );
                return min_max.ClampSizeToMinAndMax(intrinsic_size);
            }
            intrinsic_size
        };
        let flex_basis = child_style.FlexBasis();
        let has_percent_basis = self.is_column_ && flex_basis.MayHavePercentDependence();
        let is_used_flex_basis_indefinite = Cell::new(false);
        let resolve_main_length = |length: &Length, auto_length: Option<&Length>| {
            if is_main_axis_inline_axis {
                let intrinsic_func = |type_: SizeType| {
                    is_used_flex_basis_indefinite.set(true);
                    min_max_sizes_func(type_)
                };
                let inline_size = ResolveMainInlineLength(
                    &flex_basis_space,
                    child_style,
                    &border_padding,
                    &intrinsic_func,
                    length,
                    auto_length,
                    kIndefiniteSize,
                    CalcSizeKeywordBehavior::kAsSpecified,
                );
                if inline_size != kIndefiniteSize {
                    return inline_size;
                }
                is_used_flex_basis_indefinite.set(true);
                return min_max_sizes_func(SizeType::kContent).sizes.max_size;
            }
            let intrinsic_func = |type_: SizeType| {
                is_used_flex_basis_indefinite.set(true);
                block_size_func(type_)
            };
            ResolveMainBlockLength(
                &flex_basis_space,
                child_style,
                &border_padding,
                length,
                auto_length,
                &intrinsic_func,
                kIndefiniteSize,
            )
        };
        let base_border_size = {
            let mut auto_flex_basis_length = None;
            if flex_basis.HasAuto() {
                let specified = if self.is_horizontal_flow_ {
                    child_style.Width()
                } else {
                    child_style.Height()
                };
                let auto_size_length = if self.is_webkit_box_
                    && (self.Style().BoxOrient() == EBoxOrient::kHorizontal
                        || self.Style().BoxAlign() != EBoxAlignment::kStretch)
                {
                    Length::FitContent()
                } else {
                    Length::MaxContent()
                };
                let mut auto_size = resolve_main_length(specified, Some(auto_size_length));
                if child_style.BoxSizing() == EBoxSizing::kContentBox {
                    auto_size -= main_axis_border_padding;
                }
                debug_assert!(auto_size >= LayoutUnit::default());
                auto_flex_basis_length = Some(Length::Fixed(auto_size.ToFloat()));
            }
            let mut main_size = resolve_main_length(flex_basis, auto_flex_basis_length.as_ref());
            if !is_main_axis_inline_axis && !is_used_flex_basis_indefinite.get() && child.IsTable()
            {
                let table_child = TableNode::new(child.GetLayoutBox());
                let algorithms = LayoutPassScope::Algorithms();
                let caption_size = if algorithms.is_null() {
                    None
                } else {
                    unsafe { &*algorithms }.table_support.caption_block_size
                }
                .expect("table caption sizing support was not assembled");
                main_size += caption_size(&table_child, &child_space);
            }
            main_size
        };
        debug_assert!(base_border_size >= main_axis_border_padding);
        let base_content_size = base_border_size - main_axis_border_padding;
        let auto_min_length = if self.ShouldApplyAutoMinSize(&child) {
            let specified_size_suggestion = {
                let specified = if self.is_horizontal_flow_ {
                    child_style.Width()
                } else {
                    child_style.Height()
                };
                if specified.HasAuto() {
                    LayoutUnit::Max()
                } else {
                    let resolved = if is_main_axis_inline_axis {
                        ResolveMainInlineLength(
                            &flex_basis_space,
                            child_style,
                            &border_padding,
                            &min_max_sizes_func,
                            specified,
                            None,
                            kIndefiniteSize,
                            CalcSizeKeywordBehavior::kAsSpecified,
                        )
                    } else {
                        ResolveMainBlockLength(
                            &flex_basis_space,
                            child_style,
                            &border_padding,
                            specified,
                            None,
                            &block_size_func,
                            kIndefiniteSize,
                        )
                    };
                    if resolved == kIndefiniteSize {
                        LayoutUnit::Max()
                    } else {
                        resolved
                    }
                }
            };
            let content_size_suggestion = (|| -> LayoutUnit {
                let min_length = if self.is_horizontal_flow_ {
                    child_style.MinWidth()
                } else {
                    child_style.MinHeight()
                };
                if min_length.IsAuto() && specified_size_suggestion <= base_border_size {
                    if flex_shrink == 0.0 {
                        return LayoutUnit::Max();
                    }
                    let main_axis_content_size = self.MainAxisContentExtent(kIndefiniteSize);
                    if main_axis_content_size != LayoutUnit::Max() {
                        let margins = if self.is_horizontal_flow_ {
                            physical_child_margins.HorizontalSum()
                        } else {
                            physical_child_margins.VerticalSum()
                        };
                        if specified_size_suggestion + margins <= main_axis_content_size
                            && (self.is_multi_line_ || total_child_count == 1)
                        {
                            return LayoutUnit::Max();
                        }
                    }
                }
                let content_size = if is_main_axis_inline_axis {
                    min_max_sizes_func(SizeType::kContent).sizes.min_size
                } else {
                    block_size_func(SizeType::kContent)
                };
                if !child.IsReplaced() && !child_style.AspectRatio().IsAuto() {
                    return content_size.max(if is_main_axis_inline_axis {
                        min_max_sizes_func(SizeType::kIntrinsic).sizes.min_size
                    } else {
                        block_size_func(SizeType::kIntrinsic)
                    });
                }
                content_size
            })();
            debug_assert!(content_size_suggestion >= main_axis_border_padding);
            let mut auto_min_size = specified_size_suggestion.min(content_size_suggestion);
            if child_style.BoxSizing() == EBoxSizing::kContentBox {
                auto_min_size -= main_axis_border_padding;
            }
            debug_assert!(auto_min_size >= LayoutUnit::default());
            Some(Length::Fixed(auto_min_size.ToFloat()))
        } else {
            None
        };
        let mut min_max_sizes = if is_main_axis_inline_axis {
            ComputeMinMaxInlineSizes(
                &flex_basis_space,
                &child,
                &border_padding,
                auto_min_length.as_ref(),
                &min_max_sizes_func,
                TransferredSizesMode::kIgnore,
                FitContentMode::kNormal,
                kIndefiniteSize,
            )
        } else {
            ComputeMinMaxBlockSizes(
                &flex_basis_space,
                &child,
                &border_padding,
                auto_min_length.as_ref(),
                &block_size_func,
                kIndefiniteSize,
            )
        };
        min_max_sizes -= main_axis_border_padding;
        debug_assert!(min_max_sizes.min_size >= LayoutUnit::default());
        debug_assert!(min_max_sizes.max_size >= LayoutUnit::default());
        let initial_scrollbars = ComputeScrollbarsForNonAnonymous(&child);
        let aspect_ratio_provides_block_main_size = !is_main_axis_inline_axis
            && !child.IsReplaced()
            && !child_style.AspectRatio().IsAuto()
            && inline_size_func() != kIndefiniteSize;
        let is_initial_block_size_indefinite = self.is_column_
            && !is_main_axis_inline_axis
            && self.ChildAvailableSize().block_size == kIndefiniteSize
            && is_used_flex_basis_indefinite.get()
            && !aspect_ratio_provides_block_main_size;
        let writing_direction = self.GetConstraintSpace().GetWritingDirection();
        let baseline_writing_mode =
            DetermineBaselineWritingMode(writing_direction, child_writing_mode, !self.is_column_);
        let baseline_group = DetermineBaselineGroup(
            writing_direction,
            baseline_writing_mode,
            !self.is_column_,
            alignment == ItemPosition::kLastBaseline,
            self.is_wrap_reverse_,
        );
        let item = FlexItem::new(
            child,
            item_index,
            flex_grow,
            flex_shrink,
            base_content_size,
            min_max_sizes,
            main_axis_border_padding,
            max_content_contribution,
            physical_child_margins,
            initial_scrollbars,
            main_axis_auto_margin_count,
            alignment,
            baseline_writing_mode,
            baseline_group,
            is_initial_block_size_indefinite,
            is_used_flex_basis_indefinite.get(),
            depends_on_min_max_sizes.get(),
            self.is_horizontal_flow_,
        );
        (item, min_content_contribution, has_percent_basis)
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:1433-1630
    pub fn PlaceFlexItems(
        &mut self,
        phase: Phase,
        flex_lines: &mut FlexLineVector,
        oof_children: Option<
            &mut HeapVector<Member<layoutng_assembly::internal::layout_box::LayoutBox>>,
        >,
        total_intrinsic_block_size_out: Option<&mut LayoutUnit>,
    ) {
        debug_assert!(oof_children.is_some() || phase != Phase::kLayout);
        self.ConstructAndAppendFlexItems(phase, oof_children);
        let line_break_size = self.MainAxisContentExtent(kIndefiniteSize);
        let result = BreakFlexItemsIntoLines(
            &self.flex_items_,
            line_break_size,
            self.gap_between_items_,
            self.is_multi_line_,
            self.balance_min_line_count_,
        );
        let main_axis_inner_size =
            self.MainAxisContentExtent(result.max_sum_hypothetical_main_size);
        let definite_line_cross_size = (|| -> Option<LayoutUnit> {
            if self.is_multi_line_ {
                return None;
            }
            let cross_available_size = if self.is_column_ {
                self.ChildAvailableSize().inline_size
            } else {
                self.ChildAvailableSize().block_size
            };
            if cross_available_size == kIndefiniteSize {
                return None;
            }
            let style = self.Style();
            if !self.is_column_ {
                if !style.AspectRatio().IsAuto()
                    && !style.IsOverflowValueScrollableBlock()
                    && style.LogicalMinHeight().HasAuto()
                {
                    return None;
                }
                if style.LogicalMinHeight().HasContentOrIntrinsic()
                    || style.LogicalMaxHeight().HasContentOrIntrinsic()
                {
                    return None;
                }
            }
            Some(cross_available_size)
        })();
        let mut sum_line_cross_size = LayoutUnit::default();
        flex_lines.reserve(result.flex_lines.len());
        let mut item_start = 0usize;
        for line in &result.flex_lines {
            let item_end = item_start + line.count as usize;
            {
                let line_items = &mut self.flex_items_[item_start..item_end];
                LineFlexer::new(
                    line_items,
                    main_axis_inner_size,
                    line.sum_hypothetical_main_size,
                    self.gap_between_items_,
                )
                .Run();
            }
            let line_items = &self.flex_items_[item_start..item_end];
            item_start = item_end;
            let mut item_indices = Vec::with_capacity(line.count as usize);
            let mut main_axis_free_space =
                main_axis_inner_size - (line.count - 1) * self.gap_between_items_;
            let mut line_cross_size = LayoutUnit::default();
            let mut max_major_ascent = LayoutUnit::Min();
            let mut max_minor_ascent = LayoutUnit::Min();
            let mut max_major_descent = LayoutUnit::Min();
            let mut max_minor_descent = LayoutUnit::Min();
            let mut main_axis_auto_margin_count = 0u32;
            for item in line_items {
                item_indices.push(item.item_index);
                main_axis_free_space -= item.FlexedMarginBoxSize();
                main_axis_auto_margin_count += item.main_axis_auto_margin_count as u32;
                let has_baseline_alignment = matches!(
                    item.alignment,
                    ItemPosition::kBaseline | ItemPosition::kLastBaseline
                );
                if !has_baseline_alignment && definite_line_cross_size.is_some() {
                    continue;
                }
                let node = &item.block_node;
                let space = self.BuildSpaceForLayout(
                    node,
                    item.alignment,
                    item.is_initial_block_size_indefinite,
                    item.max_content_contribution,
                    Some(item.FlexedBorderBoxSize()),
                    None,
                    None,
                    false,
                );
                let layout_result: Cell<*const LayoutResult> = Cell::new(std::ptr::null());
                let cross_axis_size = (|| -> LayoutUnit {
                    let item_style = node.Style();
                    let border_padding =
                        ComputeBorders(&space, node) + ComputePadding(&space, item_style);
                    let is_main_axis_inline_axis =
                        IsHorizontalWritingMode(item_style.GetWritingMode())
                            == self.is_horizontal_flow_;
                    if node.IsReplaced() {
                        let replaced_size = ComputeReplacedSize(
                            node,
                            &space,
                            &border_padding,
                            ReplacedSizeMode::kNormal,
                        );
                        return if is_main_axis_inline_axis {
                            replaced_size.block_size
                        } else {
                            replaced_size.inline_size
                        };
                    }
                    if !is_main_axis_inline_axis {
                        let min_max_sizes_func = |type_: SizeType| {
                            node.ComputeMinMaxSizes(
                                space.GetWritingMode(),
                                type_,
                                &space,
                                MinMaxSizesFloatInput::default(),
                            )
                        };
                        return ComputeInlineSizeForFragment(
                            &space,
                            node,
                            &border_padding,
                            &min_max_sizes_func,
                        );
                    }
                    if phase == Phase::kColumnWrapIntrinsicSize {
                        return item
                            .max_content_contribution
                            .expect("column wrap contribution");
                    }
                    let _disable = if phase != Phase::kLayout
                        && !unsafe { &*node.GetLayoutBox() }.NeedsLayout()
                    {
                        Some(DisableLayoutSideEffectsScope::new())
                    } else {
                        None
                    };
                    let child_result =
                        node.Layout(&space, std::ptr::null(), std::ptr::null(), std::ptr::null());
                    assert!(!child_result.is_null());
                    layout_result.set(child_result);
                    let size = unsafe { &*child_result }.GetPhysicalFragment().Size();
                    if self.is_horizontal_flow_ {
                        size.height
                    } else {
                        size.width
                    }
                })();
                let mut cross_axis_margin_size = cross_axis_size + item.CrossAxisMarginExtent();
                if has_baseline_alignment {
                    let ascent = if layout_result.get().is_null() {
                        self.SynthesizedBaselineAscent(item, cross_axis_size)
                    } else {
                        let fragment = unsafe {
                            &*To::<PhysicalBoxFragment>(
                                unsafe { &*layout_result.get() }.GetPhysicalFragment() as *const _,
                            )
                        };
                        self.BaselineAscent(item, fragment)
                    };
                    let descent = cross_axis_margin_size - ascent;
                    if item.baseline_group == BaselineGroup::kMajor {
                        max_major_ascent = max_major_ascent.max(ascent);
                        max_major_descent = max_major_descent.max(descent);
                        cross_axis_margin_size = max_major_ascent + max_major_descent;
                    } else {
                        max_minor_ascent = max_minor_ascent.max(ascent);
                        max_minor_descent = max_minor_descent.max(descent);
                        cross_axis_margin_size = max_minor_ascent + max_minor_descent;
                    }
                }
                line_cross_size = line_cross_size.max(cross_axis_margin_size);
            }
            line_cross_size = definite_line_cross_size.unwrap_or(line_cross_size);
            flex_lines.push(FlexLine::new(
                item_indices,
                main_axis_free_space,
                line_cross_size,
                max_major_ascent,
                max_minor_ascent,
                main_axis_auto_margin_count,
            ));
            sum_line_cross_size += line_cross_size;
        }
        if let Some(out) = total_intrinsic_block_size_out {
            let mut size = self.BorderScrollbarPadding().BlockSum();
            if !flex_lines.is_empty() {
                if self.is_column_ {
                    size += result.max_sum_hypothetical_main_size;
                } else {
                    size += sum_line_cross_size;
                    size += (flex_lines.len() as WtfSizeT - 1) * self.gap_between_lines_;
                }
            } else if self.Node().HasLineIfEmpty() {
                size += self.Node().EmptyLineBlockSize(self.GetBreakToken());
            }
            *out = ClampIntrinsicBlockSize(
                self.GetConstraintSpace(),
                self.Node(),
                self.GetBreakToken(),
                self.BorderScrollbarPadding(),
                size,
                None,
            );
        }
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:1632-1642
    pub fn ApplyReversals(&self, flex_lines: &mut FlexLineVector) {
        if self.is_wrap_reverse_ {
            flex_lines.reverse();
        }
        if self.is_reverse_direction_ {
            for line in flex_lines.iter_mut() {
                line.item_indices.reverse();
            }
        }
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:2738-2786
    pub fn PropagateFlexItemInfo(
        &self,
        item: &FlexItem,
        fragment: &PhysicalBoxFragment,
        physical_margins: &PhysicalBoxStrut,
        flex_line_index: WtfSizeT,
        offset: LogicalOffset,
    ) -> EStatus {
        let mut status = EStatus::kSuccess;
        if !self.layout_info_for_devtools_.is_null() {
            let logical_size = LogicalSize::new(
                self.base.container_builder_.InlineSize(),
                self.total_block_size_,
            );
            let flexbox_size =
                ToPhysicalSize(logical_size, self.GetConstraintSpace().GetWritingMode());
            let physical_offset = offset.ConvertToPhysical(
                self.GetConstraintSpace().GetWritingDirection(),
                flexbox_size,
                fragment.Size(),
            );
            let mut item_rect = PhysicalRect::new(physical_offset, fragment.Size());
            item_rect.Expand(physical_margins);
            let info = unsafe { &mut *self.layout_info_for_devtools_ };
            assert!(!info.lines.is_empty());
            info.lines[flex_line_index as usize]
                .items
                .push(DevtoolsFlexInfoItem::new(
                    item_rect,
                    self.BaselineAscent(item, fragment),
                ));
        }
        if !self.ignore_child_scrollbar_changes_ {
            if item.initial_scrollbars != ComputeScrollbarsForNonAnonymous(&item.block_node) {
                status = EStatus::kNeedsRelayoutWithNoChildScrollbarChanges;
            }
            if item.depends_on_min_max_sizes
                && unsafe { &*item.block_node.GetLayoutBox() }.IntrinsicLogicalWidthsDirty()
            {
                status = EStatus::kNeedsRelayoutWithNoChildScrollbarChanges;
            }
        } else {
            debug_assert_eq!(
                item.initial_scrollbars,
                ComputeScrollbarsForNonAnonymous(&item.block_node)
            );
        }
        status
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:651-657
    pub fn DoesItemStretch(&self, child: &BlockNode, alignment: ItemPosition) -> bool {
        alignment == ItemPosition::kStretch && self.DoesItemComputedCrossSizeHaveAuto(child)
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:659-666
    pub fn DoesItemComputedCrossSizeHaveAuto(&self, child: &BlockNode) -> bool {
        if self.is_horizontal_flow_ {
            child.Style().Height().HasAuto()
        } else {
            child.Style().Width().HasAuto()
        }
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:668-673
    pub fn WillChildCrossSizeBeContainerCrossSize(
        &self,
        child: &BlockNode,
        alignment: ItemPosition,
    ) -> bool {
        !self.is_multi_line_
            && self.is_cross_size_definite_
            && self.DoesItemStretch(child, alignment)
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:675-686
    pub fn BuildSpaceForIntrinsicInlineSize(
        &self,
        child: &BlockNode,
        alignment: ItemPosition,
    ) -> ConstraintSpace {
        let mut builder =
            MinMaxConstraintSpaceBuilder::new(self.GetConstraintSpace(), self.Style(), child, true);
        builder.SetAvailableBlockSize(self.ChildAvailableSize().block_size);
        builder.SetPercentageResolutionBlockSize(self.child_percentage_size_.block_size);
        if !self.is_column_ && !self.is_multi_line_ && alignment == ItemPosition::kStretch {
            builder.SetBlockAutoBehavior(AutoSizeBehavior::kStretchExplicit);
        }
        builder.ToConstraintSpace()
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:688-700
    pub fn BuildSpaceForFlexBasis(&self, flex_item: &BlockNode) -> ConstraintSpace {
        let mut builder = ConstraintSpaceBuilder::new(
            self.GetConstraintSpace(),
            flex_item.Style().GetWritingDirection(),
            true,
        );
        SetOrthogonalFallbackInlineSizeIfNeeded(self.Style(), flex_item.base.clone(), &mut builder);
        builder.SetAvailableSize(*self.ChildAvailableSize());
        builder.SetPercentageResolutionSize(self.child_percentage_size_);
        builder.ToConstraintSpace()
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:702-807
    #[allow(clippy::too_many_arguments)]
    pub fn BuildSpaceForLayout(
        &self,
        node: &BlockNode,
        alignment: ItemPosition,
        is_initial_block_size_indefinite: bool,
        override_inline_size: Option<LayoutUnit>,
        main_axis_final_size: Option<LayoutUnit>,
        line_cross_size: Option<LayoutUnit>,
        block_offset_for_fragmentation: Option<LayoutUnit>,
        min_block_size_should_encompass_intrinsic_size: bool,
    ) -> ConstraintSpace {
        let mut builder = ConstraintSpaceBuilder::new(
            self.GetConstraintSpace(),
            node.Style().GetWritingDirection(),
            true,
        );
        SetOrthogonalFallbackInlineSizeIfNeeded(self.Style(), node.base.clone(), &mut builder);
        builder.SetIsPaintedAtomically(true);
        if line_cross_size.is_none() {
            builder.SetCacheSlot(LayoutResultCacheSlot::kMeasure);
        }
        let mut available_size = *self.ChildAvailableSize();
        let mut percentage_size = self.child_percentage_size_;
        if let Some(line_count) = self.balance_min_line_count_ {
            assert!(line_count > 0);
            let gap_size = (line_count - 1) * self.gap_between_lines_;
            if self.is_column_ {
                if available_size.inline_size != kIndefiniteSize {
                    available_size.inline_size =
                        (available_size.inline_size - gap_size) / line_count;
                }
            } else if available_size.block_size != kIndefiniteSize {
                available_size.block_size = (available_size.block_size - gap_size) / line_count;
            }
        }
        if self.is_column_ {
            if let Some(inline_size) = override_inline_size {
                debug_assert!(line_cross_size.is_none());
                available_size.inline_size = inline_size;
                builder.SetIsFixedInlineSize(true);
            } else if let Some(cross_size) = line_cross_size {
                available_size.inline_size = cross_size;
            }
            if let Some(main_size) = main_axis_final_size {
                available_size.block_size = main_size;
                builder.SetIsFixedBlockSize(true);
            }
        } else {
            debug_assert!(override_inline_size.is_none());
            if let Some(cross_size) = line_cross_size {
                available_size.block_size = cross_size;
            }
            if let Some(main_size) = main_axis_final_size {
                available_size.inline_size = main_size;
                builder.SetIsFixedInlineSize(true);
            }
        }
        let is_cross_size_definite =
            (!self.is_multi_line_ && self.is_cross_size_definite_) || line_cross_size.is_some();
        if is_cross_size_definite && alignment == ItemPosition::kStretch {
            if self.is_column_ {
                builder.SetInlineAutoBehavior(AutoSizeBehavior::kStretchExplicit);
            } else {
                builder.SetBlockAutoBehavior(AutoSizeBehavior::kStretchExplicit);
            }
        }
        if is_initial_block_size_indefinite {
            debug_assert!(self.is_column_);
            builder.SetIsInitialBlockSizeIndefinite(true);
            if main_axis_final_size.is_none() {
                available_size.block_size = kIndefiniteSize;
                percentage_size.block_size = kIndefiniteSize;
            }
        }
        if let Some(offset) = block_offset_for_fragmentation {
            if self.GetConstraintSpace().HasBlockFragmentation() {
                if min_block_size_should_encompass_intrinsic_size {
                    builder.SetMinBlockSizeShouldEncompassIntrinsicSize();
                }
                SetupSpaceBuilderForFragmentationFromBuilder(
                    &self.base.container_builder_,
                    &node.base,
                    offset,
                    &mut builder,
                );
            }
        }
        builder.SetAvailableSize(available_size);
        builder.SetPercentageResolutionSize(percentage_size);
        builder.ToConstraintSpace()
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:1723-2095
    pub fn GiveItemsFinalPositionAndSize(
        &mut self,
        flex_lines: &mut FlexLineVector,
        row_break_between_outputs: &mut Vec<EBreakBetween>,
        gap_accumulator: &mut Option<FlexGapAccumulator>,
        effective_gap_between_lines: &mut LayoutUnit,
        total_row_gap_count: Option<&mut WtfSizeT>,
    ) -> EStatus {
        debug_assert!(!IsBreakInside(self.GetBreakToken()));
        let should_propagate_breaks = self.GetConstraintSpace().ShouldPropagateChildBreakValues();
        if should_propagate_breaks {
            let break_count = if self.is_column_ {
                2
            } else {
                flex_lines.len() + 1
            };
            *row_break_between_outputs = vec![EBreakBetween::kAuto; break_count];
        }
        if flex_lines.is_empty() {
            return EStatus::kSuccess;
        }

        let writing_direction = self.GetConstraintSpace().GetWritingDirection();
        let justify_content =
            ResolvedJustifyContent(self.Style(), writing_direction, self.is_column_);
        let align_content = self.Style().AlignContent();
        let num_lines = flex_lines.len() as WtfSizeT;
        let cross_axis_content_size = if self.is_column_ {
            self.base.container_builder_.InlineSize() - self.BorderScrollbarPadding().InlineSum()
        } else {
            self.total_block_size_ - self.BorderScrollbarPadding().BlockSum()
        }
        .ClampNegativeToZero();
        let mut cross_axis_free_space = cross_axis_content_size;
        for line in flex_lines.iter() {
            cross_axis_free_space -= line.line_cross_size;
        }
        cross_axis_free_space -= (num_lines - 1) * self.gap_between_lines_;
        let is_align_content_stretch = align_content.Distribution()
            == ContentDistributionType::kStretch
            || (align_content.GetPosition() == ContentPosition::kNormal
                && align_content.Distribution() == ContentDistributionType::kDefault);
        if !self.is_multi_line_ {
            flex_lines.last_mut().unwrap().line_cross_size = cross_axis_content_size;
            cross_axis_free_space = LayoutUnit::default();
        } else if cross_axis_free_space >= LayoutUnit::default() && is_align_content_stretch {
            let mut extra = LayoutUnitDiffuser::new(cross_axis_free_space, num_lines);
            for line in flex_lines.iter_mut() {
                line.line_cross_size += extra.Next();
            }
            cross_axis_free_space = LayoutUnit::default();
        }
        let mut space_between_lines =
            ContentDistributionSpace(&align_content, cross_axis_free_space, num_lines);
        *effective_gap_between_lines = self.gap_between_lines_ + space_between_lines.BaseSize();
        if let Some(accumulator) = gap_accumulator.as_mut() {
            accumulator.SetEffectiveGapBetweenLines(*effective_gap_between_lines);
            if let Some(count) = total_row_gap_count {
                *count = if self.is_column_ {
                    debug_assert!(self.flex_items_.len() >= flex_lines.len());
                    self.flex_items_.len() as WtfSizeT - num_lines
                } else {
                    num_lines - 1
                };
            }
        }

        let mut line_cross_axis_offset = (if self.is_column_ {
            self.BorderScrollbarPadding().inline_start
        } else {
            self.BorderScrollbarPadding().block_start
        }) + InitialContentPositionOffset(
            &align_content,
            cross_axis_free_space,
            num_lines,
            self.is_wrap_reverse_,
        );
        let mut baseline_accumulator = BaselineAccumulator::new(self.Style());
        let mut status = EStatus::kSuccess;
        let in_fragmentation =
            InvolvedInBlockFragmentationForBuilder(&self.base.container_builder_);
        for line_index in 0..flex_lines.len() {
            if !self.layout_info_for_devtools_.is_null() {
                unsafe { &mut *self.layout_info_for_devtools_ }
                    .lines
                    .push(DevtoolsFlexInfoLine { items: Vec::new() });
            }
            let is_first_line = line_index == 0;
            let is_last_line = line_index + 1 == flex_lines.len();
            flex_lines[line_index].cross_axis_offset = line_cross_axis_offset;
            if !in_fragmentation && !self.is_column_ {
                baseline_accumulator.AccumulateLine(
                    &flex_lines[line_index],
                    is_first_line,
                    is_last_line,
                );
            }
            let line = &flex_lines[line_index];
            let should_apply_auto_margin = line.main_axis_auto_margin_count > 0
                && line.main_axis_free_space > LayoutUnit::default();
            let main_axis_free_space = if should_apply_auto_margin {
                LayoutUnit::default()
            } else {
                line.main_axis_free_space
            };
            let mut main_axis_auto_margin = if should_apply_auto_margin {
                LayoutUnitDiffuser::new(line.main_axis_free_space, line.main_axis_auto_margin_count)
            } else {
                LayoutUnitDiffuser::empty()
            };
            let item_indices = line.item_indices.clone();
            let line_cross_size = line.line_cross_size;
            let mut space_between_items = ContentDistributionSpace(
                &justify_content,
                main_axis_free_space,
                item_indices.len() as WtfSizeT,
            );
            let mut main_axis_offset = (if self.is_column_ {
                self.BorderScrollbarPadding().block_start
            } else {
                self.BorderScrollbarPadding().inline_start
            }) + InitialContentPositionOffset(
                &justify_content,
                main_axis_free_space,
                item_indices.len() as WtfSizeT,
                self.is_reverse_direction_,
            );
            let border_end = if self.is_column_ {
                self.base
                    .container_builder_
                    .BorderScrollbarPadding()
                    .block_end
            } else {
                self.base
                    .container_builder_
                    .BorderScrollbarPadding()
                    .inline_end
            };
            let container_main_end = if self.is_column_ {
                self.base
                    .container_builder_
                    .InitialBorderBoxSize()
                    .block_size
                    - border_end
            } else {
                self.base.container_builder_.InlineSize() - border_end
            };
            let mut need_effective_gap = true;

            for (index_in_line, &item_index) in item_indices.iter().enumerate() {
                let item = &self.flex_items_[item_index as usize];
                let child_space = self.BuildSpaceForLayout(
                    &item.block_node,
                    item.alignment,
                    item.is_initial_block_size_indefinite,
                    None,
                    Some(item.FlexedBorderBoxSize()),
                    Some(line_cross_size),
                    None,
                    false,
                );
                let layout_result = unsafe {
                    &*item.block_node.Layout(
                        &child_space,
                        std::ptr::null(),
                        std::ptr::null(),
                        std::ptr::null(),
                    )
                };
                let item_style = item.block_node.Style();
                if should_propagate_breaks {
                    let before = JoinFragmentainerBreakValues(
                        item_style.BreakBefore(),
                        layout_result.InitialBreakBefore(),
                    );
                    let after = JoinFragmentainerBreakValues(
                        item_style.BreakAfter(),
                        layout_result.FinalBreakAfter(),
                    );
                    if !self.is_column_ {
                        row_break_between_outputs[line_index] = JoinFragmentainerBreakValues(
                            row_break_between_outputs[line_index],
                            before,
                        );
                        row_break_between_outputs[line_index + 1] = JoinFragmentainerBreakValues(
                            row_break_between_outputs[line_index + 1],
                            after,
                        );
                    } else {
                        if index_in_line == 0 {
                            row_break_between_outputs[0] =
                                JoinFragmentainerBreakValues(row_break_between_outputs[0], before);
                        }
                        if index_in_line + 1 == item_indices.len() {
                            row_break_between_outputs[1] =
                                JoinFragmentainerBreakValues(row_break_between_outputs[1], after);
                        }
                    }
                }
                let physical_fragment = unsafe {
                    &*To::<PhysicalBoxFragment>(layout_result.GetPhysicalFragment() as *const _)
                };
                let fragment = LogicalBoxFragment::new(writing_direction, physical_fragment);
                let cross_axis_size = if self.is_column_ {
                    fragment.InlineSize()
                } else {
                    fragment.BlockSize()
                };
                let mut physical_margins = item.initial_margins;
                let margin_sides = PhysicalToFlex::new(
                    writing_direction,
                    self.is_column_,
                    PhysicalMarginSide::Top,
                    PhysicalMarginSide::Right,
                    PhysicalMarginSide::Bottom,
                    PhysicalMarginSide::Left,
                );
                let cross_axis_space = line_cross_size
                    - margin_sides.CrossStart().get(&physical_margins)
                    - cross_axis_size
                    - margin_sides.CrossEnd().get(&physical_margins);
                let is_margin_auto = PhysicalToFlex::new(
                    writing_direction,
                    self.is_column_,
                    item_style.MarginTop().IsAuto(),
                    item_style.MarginRight().IsAuto(),
                    item_style.MarginBottom().IsAuto(),
                    item_style.MarginLeft().IsAuto(),
                );
                let margin_space = cross_axis_space.ClampNegativeToZero();
                if is_margin_auto.CrossStart() && is_margin_auto.CrossEnd() {
                    margin_sides
                        .CrossStart()
                        .set(&mut physical_margins, margin_space / 2);
                    margin_sides
                        .CrossEnd()
                        .set(&mut physical_margins, margin_space / 2);
                } else if is_margin_auto.CrossStart() {
                    margin_sides
                        .CrossStart()
                        .set(&mut physical_margins, margin_space);
                } else if is_margin_auto.CrossEnd() {
                    margin_sides
                        .CrossEnd()
                        .set(&mut physical_margins, margin_space);
                }
                if is_margin_auto.MainStart() {
                    margin_sides
                        .MainStart()
                        .set(&mut physical_margins, main_axis_auto_margin.Next());
                }
                if is_margin_auto.MainEnd() {
                    margin_sides
                        .MainEnd()
                        .set(&mut physical_margins, main_axis_auto_margin.Next());
                }
                let is_safe = !self.is_webkit_box_
                    && item_style
                        .ResolvedAlignSelf(
                            &StyleSelfAlignmentData::new_nonlegacy(
                                ItemPosition::kStretch,
                                OverflowAlignment::kDefault,
                            ),
                            self.Style() as *const _,
                        )
                        .Overflow()
                        == OverflowAlignment::kSafe;
                let space = if is_safe {
                    cross_axis_space.ClampNegativeToZero()
                } else {
                    cross_axis_space
                };
                let cross_offset = match item.alignment {
                    ItemPosition::kCenter => space / 2,
                    ItemPosition::kFlexStart => LayoutUnit::default(),
                    ItemPosition::kFlexEnd => space,
                    ItemPosition::kStretch => {
                        if self.is_wrap_reverse_ {
                            space
                        } else {
                            LayoutUnit::default()
                        }
                    }
                    ItemPosition::kBaseline | ItemPosition::kLastBaseline => {
                        let is_major = item.baseline_group == BaselineGroup::kMajor;
                        let ascent = self.BaselineAscent(item, physical_fragment);
                        let max_ascent = if is_major {
                            flex_lines[line_index].major_baseline
                        } else {
                            flex_lines[line_index].minor_baseline
                        };
                        let delta = max_ascent - ascent;
                        if is_major {
                            delta
                        } else {
                            space - delta
                        }
                    }
                    _ => unreachable!("flex item alignment should be resolved before placement"),
                };
                let cross_axis_offset = line_cross_axis_offset
                    + cross_offset
                    + margin_sides.CrossStart().get(&physical_margins);
                main_axis_offset += margin_sides.MainStart().get(&physical_margins);
                let offset = if self.is_column_ {
                    LogicalOffset::new(cross_axis_offset, main_axis_offset)
                } else {
                    LogicalOffset::new(main_axis_offset, cross_axis_offset)
                };
                let current_space_between = space_between_items.Next();
                main_axis_offset += item.FlexedBorderBoxSize()
                    + margin_sides.MainEnd().get(&physical_margins)
                    + current_space_between
                    + self.gap_between_items_;
                if need_effective_gap && index_in_line > 0 {
                    flex_lines[line_index].effective_gap_between_items =
                        current_space_between + self.gap_between_items_;
                    need_effective_gap = false;
                }
                let logical_margins = physical_margins.ConvertToLogical(writing_direction);
                if !in_fragmentation {
                    self.base.container_builder_.AddResult(
                        layout_result,
                        offset,
                        Some(logical_margins),
                        None,
                        std::ptr::null(),
                    );
                    baseline_accumulator.AccumulateItem(
                        &fragment,
                        offset.block_offset,
                        is_first_line,
                        is_last_line,
                    );
                } else {
                    flex_lines[line_index]
                        .line_items_data
                        .push(FlexItemData::new(
                            item.block_node.clone(),
                            item.item_index,
                            offset,
                            item.alignment,
                            item.FlexedBorderBoxSize(),
                            logical_margins.block_end,
                            fragment.BlockSize(),
                            item.is_initial_block_size_indefinite,
                            item.is_used_flex_basis_indefinite,
                            layout_result.HasDescendantThatDependsOnPercentageBlockSize(),
                        ));
                }
                if self.PropagateFlexItemInfo(
                    item,
                    physical_fragment,
                    &physical_margins,
                    line_index as WtfSizeT,
                    offset,
                ) == EStatus::kNeedsRelayoutWithNoChildScrollbarChanges
                {
                    status = EStatus::kNeedsRelayoutWithNoChildScrollbarChanges;
                }
                if !in_fragmentation {
                    if let Some(accumulator) = gap_accumulator.as_mut() {
                        let line = &flex_lines[line_index];
                        accumulator.BuildGapsForCurrentItem(
                            flex_lines,
                            line_index as WtfSizeT,
                            offset,
                            index_in_line == 0,
                            index_in_line + 1 == item_indices.len(),
                            is_last_line,
                            line.cross_axis_offset,
                            line.LineCrossEnd(),
                            container_main_end,
                            false,
                        );
                    }
                }
            }
            line_cross_axis_offset += flex_lines[line_index].line_cross_size
                + space_between_lines.Next()
                + self.gap_between_lines_;
        }
        if let Some(baseline) = baseline_accumulator.FirstBaseline() {
            self.base.container_builder_.SetFirstBaseline(baseline);
        }
        if let Some(baseline) = baseline_accumulator.LastBaseline() {
            self.base.container_builder_.SetLastBaseline(baseline);
        }
        if self.Node().IsSlider() {
            debug_assert!(!in_fragmentation);
            self.base.container_builder_.ClearBaselines();
        }
        status
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:2095-2736
    #[allow(clippy::too_many_arguments)]
    pub fn GiveItemsFinalPositionAndSizeForFragmentation(
        &mut self,
        flex_lines: &mut FlexLineVector,
        row_break_between_outputs: &mut Vec<EBreakBetween>,
        break_before_row: &mut FlexBreakBeforeRow,
        total_intrinsic_block_size: &mut LayoutUnit,
        gap_accumulator: &mut Option<FlexGapAccumulator>,
        effective_gap_between_lines: LayoutUnit,
        previous_gap_data: Option<&FlexGapBreakTokenData>,
    ) -> EStatus {
        debug_assert!(InvolvedInBlockFragmentationForBuilder(
            &self.base.container_builder_
        ));
        if self.is_column_ {
            if let Some(gaps) = gap_accumulator.as_mut() {
                gaps.InitializeFragmentedColumnGapGeometry(flex_lines);
            }
        }
        // The iterator retains indices and break tokens while the line state is
        // updated. Its source collection has a stable allocation for this pass.
        let lines_ptr = flex_lines as *mut FlexLineVector;
        let mut item_iterator =
            FlexItemIterator::new(flex_lines, self.GetBreakToken(), self.is_column_);
        let mut has_inflow_child_break_inside_line = vec![false; flex_lines.len()];
        let mut needs_earlier_break_in_column = false;
        let mut status = EStatus::kSuccess;
        let fragmentainer_space = self.FragmentainerSpaceLeftForChildren();
        let mut column_break_info = if self.is_column_ {
            (0..flex_lines.len())
                .map(|_| FlexColumnBreakInfo::default())
                .collect()
        } else {
            Vec::new()
        };
        let mut previously_consumed_block_size = LayoutUnit::default();
        let mut offset_in_stitched_container = LayoutUnit::default();
        if IsBreakInside(self.GetBreakToken()) {
            let token = unsafe { &*self.GetBreakToken() };
            previously_consumed_block_size = token.ConsumedBlockSize();
            offset_in_stitched_container = previously_consumed_block_size;
            if self.Style().BoxDecorationBreak() == EBoxDecorationBreak::kClone
                && offset_in_stitched_container != LayoutUnit::Max()
            {
                let count = token.SequenceNumber() + 1;
                offset_in_stitched_container -= count * self.BorderScrollbarPadding().BlockSum()
                    - self.BorderScrollbarPadding().block_start;
            }
        }
        let mut baseline_accumulator = BaselineAccumulator::new(self.Style());
        let broke_before_row = *break_before_row != FlexBreakBeforeRow::kNotBreakBeforeRow;
        let border_scrollbar_padding = if self.is_column_ {
            self.base
                .container_builder_
                .BorderScrollbarPadding()
                .block_end
        } else {
            self.base
                .container_builder_
                .BorderScrollbarPadding()
                .inline_end
        };
        let mut entry = item_iterator.NextItem(broke_before_row);
        while !entry.flex_item.is_null() {
            let flex_item_idx = entry.flex_item_idx as usize;
            let flex_line_idx = entry.flex_line_idx as usize;
            let is_last_line = flex_line_idx + 1 == unsafe { &*lines_ptr }.len();
            let is_first_line = flex_line_idx == 0;
            let flex_line = unsafe { &mut (&mut *lines_ptr)[flex_line_idx] };
            let flex_item = unsafe { &mut *entry.flex_item };
            let item_break_token = entry.token;
            let is_last_item_in_line = flex_item_idx + 1 == flex_line.line_items_data.len();
            let is_first_item_in_line = if self.is_column_ {
                !item_break_token.is_null() || flex_item_idx == 0
            } else {
                flex_item_idx == 0
            };
            if !self.is_column_ {
                if flex_line_idx > 0 && has_inflow_child_break_inside_line[flex_line_idx - 1] {
                    break;
                }
            } else {
                if !self.additional_early_breaks_.is_null()
                    && flex_line_idx < unsafe { &*self.additional_early_breaks_ }.len()
                {
                    self.early_break_ =
                        unsafe { &*self.additional_early_breaks_ }[flex_line_idx].Get();
                } else if !self.early_break_.is_null() && flex_line_idx != 0 {
                    self.early_break_ = std::ptr::null();
                }
                if has_inflow_child_break_inside_line[flex_line_idx] {
                    if !is_last_item_in_line {
                        item_iterator.NextLine();
                    }
                    entry = item_iterator.NextItem(broke_before_row);
                    continue;
                }
            }
            let mut row_block_offset = if self.is_column_ {
                LayoutUnit::default()
            } else {
                flex_line.cross_axis_offset
            };
            let original_offset = flex_item.offset;
            let mut offset = original_offset;
            let mut individual_item_adjustment = LayoutUnit::default();
            if !item_break_token.is_null() && unsafe { &*item_break_token }.IsBreakBefore() {
                let token = unsafe { &*item_break_token };
                if token.IsForcedBreak() {
                    flex_line.item_offset_adjustment += offset_in_stitched_container;
                } else if !self.is_column_ && flex_item_idx == 0 && broke_before_row {
                    if *break_before_row == FlexBreakBeforeRow::kAtStartOfBreakBeforeRow {
                        let previous_row_end = if is_first_line {
                            LayoutUnit::default()
                        } else {
                            unsafe { &(&*lines_ptr)[flex_line_idx - 1] }.LineCrossEnd()
                        };
                        let unused =
                            (offset_in_stitched_container - previous_row_end).ClampNegativeToZero();
                        let mut consumed_row_gap = LayoutUnit::default();
                        if unused != LayoutUnit::default() {
                            let row_gap = row_block_offset + flex_line.item_offset_adjustment
                                - previous_row_end;
                            debug_assert!(row_gap >= LayoutUnit::default());
                            consumed_row_gap = row_gap.min(unused);
                        }
                        flex_line.item_offset_adjustment +=
                            offset_in_stitched_container - previous_row_end - consumed_row_gap;
                    }
                } else {
                    individual_item_adjustment = (offset_in_stitched_container
                        - (offset.block_offset + flex_line.item_offset_adjustment))
                        .ClampNegativeToZero();
                    if self.is_column_ {
                        flex_line.item_offset_adjustment += individual_item_adjustment;
                    }
                }
            }
            if IsBreakInside(item_break_token) {
                offset.block_offset = self.BorderScrollbarPadding().block_start;
            } else if IsBreakInside(self.GetBreakToken()) {
                let adjustment = offset_in_stitched_container
                    - flex_line.item_offset_adjustment
                    - self.BorderScrollbarPadding().block_start;
                offset.block_offset -= adjustment;
                if !self.is_column_ {
                    offset.block_offset += individual_item_adjustment;
                    row_block_offset -= adjustment;
                }
            }
            let mut early_break_in_child = std::ptr::null();
            if !self.early_break_.is_null() {
                let early_break = unsafe { &*self.early_break_ };
                if !self.is_column_ {
                    self.base
                        .container_builder_
                        .SetLineCount(flex_line_idx as i32);
                }
                if IsEarlyBreakTarget(
                    early_break,
                    &self.base.container_builder_,
                    &flex_item.block_node.base,
                ) {
                    self.base.container_builder_.AddBreakBeforeChild(
                        flex_item.block_node.base.clone(),
                        Some(BreakAppeal::kBreakAppealPerfect),
                        false,
                        LogicalOffset::default(),
                    );
                    if early_break.Type() == BreakType::kLine {
                        *break_before_row = FlexBreakBeforeRow::kAtStartOfBreakBeforeRow;
                    }
                    self.ConsumeRemainingFragmentainerSpace(
                        offset_in_stitched_container,
                        flex_line,
                        None,
                    );
                    has_inflow_child_break_inside_line[flex_line_idx] = true;
                    if self.is_column_ {
                        if !is_last_item_in_line {
                            item_iterator.NextLine();
                        }
                    } else if is_last_item_in_line {
                        break;
                    }
                    self.last_line_idx_to_process_first_child_ = flex_line_idx as WtfSizeT;
                    entry = item_iterator.NextItem(broke_before_row);
                    continue;
                }
                early_break_in_child = EnterEarlyBreakInChild(&flex_item.block_node, early_break);
            }
            if !self.cross_size_adjustments_.is_null() {
                debug_assert!(!self.is_column_);
                let adjustments = unsafe { &*self.cross_size_adjustments_ };
                let key = flex_line_idx as WtfSizeT + 1;
                if let Some(&adjustment) = adjustments.get(&key) {
                    if self.last_line_idx_to_process_first_child_ == kNotFound
                        || self.last_line_idx_to_process_first_child_ < flex_line_idx as WtfSizeT
                    {
                        flex_line.line_cross_size += adjustment;
                        self.AdjustOffsetForNextLine(
                            flex_lines,
                            flex_line_idx as WtfSizeT,
                            adjustment,
                        );
                    }
                }
            }
            let mut line_cross_size = flex_line.line_cross_size;
            if !self.is_column_ && !item_break_token.is_null() {
                line_cross_size -= offset_in_stitched_container
                    - (original_offset.block_offset + flex_line.item_offset_adjustment)
                    - unsafe { &*item_break_token }.ConsumedBlockSize();
                line_cross_size = line_cross_size.ClampNegativeToZero();
            }
            let min_block_size_encompasses_intrinsic =
                self.MinBlockSizeShouldEncompassIntrinsicSize(flex_item);
            let child_space = self.BuildSpaceForLayout(
                &flex_item.block_node,
                flex_item.alignment,
                flex_item.is_initial_block_size_indefinite,
                None,
                Some(flex_item.main_axis_final_size),
                Some(line_cross_size),
                Some(offset.block_offset),
                min_block_size_encompasses_intrinsic,
            );
            let layout_result = unsafe {
                &*flex_item.block_node.Layout(
                    &child_space,
                    item_break_token,
                    early_break_in_child,
                    std::ptr::null(),
                )
            };
            let mut break_status = BreakStatus::kContinue;
            let mut current_column_break_info: *mut FlexColumnBreakInfo = std::ptr::null_mut();
            if self.early_break_.is_null() && self.GetConstraintSpace().HasBlockFragmentation() {
                let has_container_separation;
                if !self.is_column_ {
                    has_container_separation = offset.block_offset > row_block_offset
                        && (item_break_token.is_null()
                            || (broke_before_row
                                && flex_item_idx == 0
                                && unsafe { &*item_break_token }.IsBreakBefore()));
                    if flex_item_idx == 0
                        && (item_break_token.is_null()
                            || (unsafe { &*item_break_token }.IsBreakBefore() && broke_before_row))
                    {
                        let row_separation = self.has_processed_first_line_;
                        let is_first_for_row = item_break_token.is_null() || broke_before_row;
                        let row_status = self.BreakBeforeRowIfNeeded(
                            flex_line,
                            row_block_offset,
                            row_break_between_outputs[flex_line_idx],
                            flex_line_idx as WtfSizeT,
                            flex_item.block_node.base.clone(),
                            row_separation,
                            is_first_for_row,
                        );
                        if row_status == BreakStatus::kBrokeBefore {
                            if let Some(gaps) = gap_accumulator.as_mut() {
                                gaps.SuppressLastMainGap(None);
                            }
                            if flex_line_idx > 0 {
                                let previous_end = unsafe { &(&*lines_ptr)[flex_line_idx - 1] }
                                    .LineCrossEnd()
                                    - offset_in_stitched_container;
                                self.UpdateOffsetAdjustmentForSuppressedRowGap(
                                    effective_gap_between_lines,
                                    previous_end,
                                    flex_line,
                                );
                            }
                            self.ConsumeRemainingFragmentainerSpace(
                                offset_in_stitched_container,
                                flex_line,
                                None,
                            );
                            *break_before_row = if broke_before_row {
                                FlexBreakBeforeRow::kPastStartOfBreakBeforeRow
                            } else {
                                FlexBreakBeforeRow::kAtStartOfBreakBeforeRow
                            };
                            break;
                        }
                        *break_before_row = FlexBreakBeforeRow::kNotBreakBeforeRow;
                        if row_status == BreakStatus::kNeedsEarlierBreak {
                            status = EStatus::kNeedsEarlierBreak;
                            break;
                        }
                        debug_assert_eq!(row_status, BreakStatus::kContinue);
                    }
                } else {
                    has_container_separation = item_break_token.is_null()
                        && ((self.last_line_idx_to_process_first_child_ != kNotFound
                            && self.last_line_idx_to_process_first_child_
                                >= flex_line_idx as WtfSizeT)
                            || offset.block_offset > LayoutUnit::default());
                    if flex_lines.len() > 1 {
                        current_column_break_info = &mut column_break_info[flex_line_idx];
                        self.base.container_builder_.SetPreviousBreakAfter(
                            unsafe { &*current_column_break_info }.break_after,
                        );
                    }
                }
                break_status = BreakBeforeChildIfNeededFull(
                    flex_item.block_node.base.clone(),
                    layout_result,
                    self.FragmentainerOffsetForChildren() + offset.block_offset,
                    self.FragmentainerCapacityForChildren(),
                    has_container_separation,
                    &mut self.base.container_builder_,
                    !self.is_column_,
                    current_column_break_info,
                );
                if !current_column_break_info.is_null() {
                    unsafe { &mut *current_column_break_info }.break_after =
                        self.base.container_builder_.PreviousBreakAfter();
                }
            }
            if break_status == BreakStatus::kNeedsEarlierBreak {
                if !current_column_break_info.is_null() {
                    let info = unsafe { &*current_column_break_info };
                    debug_assert!(self.is_column_ && !info.early_break.Get().is_null());
                    if !needs_earlier_break_in_column {
                        needs_earlier_break_in_column = true;
                        self.base
                            .container_builder_
                            .SetEarlyBreak(info.early_break.Get());
                    }
                    self.AddColumnEarlyBreak(info.early_break.Get(), flex_line_idx as WtfSizeT);
                    if !is_last_item_in_line {
                        item_iterator.NextLine();
                    }
                    entry = item_iterator.NextItem(broke_before_row);
                    continue;
                }
                status = EStatus::kNeedsEarlierBreak;
                break;
            }
            if break_status == BreakStatus::kBrokeBefore {
                if self.is_column_ && flex_item_idx > 0 {
                    self.UpdateOffsetAdjustmentForSuppressedRowGap(
                        flex_line.effective_gap_between_items,
                        self.intrinsic_block_size_,
                        flex_line,
                    );
                    if let Some(gaps) = gap_accumulator.as_mut() {
                        gaps.IncrementRowGapCount(flex_line_idx as WtfSizeT);
                    }
                }
                self.ConsumeRemainingFragmentainerSpace(
                    offset_in_stitched_container,
                    flex_line,
                    if current_column_break_info.is_null() {
                        None
                    } else {
                        Some(unsafe { &*current_column_break_info })
                    },
                );
                has_inflow_child_break_inside_line[flex_line_idx] = true;
                if self.is_column_ {
                    if !is_last_item_in_line {
                        item_iterator.NextLine();
                    }
                } else if is_last_item_in_line {
                    break;
                }
                self.last_line_idx_to_process_first_child_ = flex_line_idx as WtfSizeT;
                entry = item_iterator.NextItem(broke_before_row);
                continue;
            }
            let physical_fragment = unsafe {
                &*To::<PhysicalBoxFragment>(layout_result.GetPhysicalFragment() as *const _)
            };
            let fragment = LogicalBoxFragment::new(
                self.GetConstraintSpace().GetWritingDirection(),
                physical_fragment,
            );
            let child_break_token = physical_fragment.GetBreakToken();
            let is_at_block_end =
                child_break_token.is_null() || unsafe { &*child_break_token }.IsAtBlockEnd();
            let mut item_block_end = offset.block_offset + fragment.BlockSize();
            if is_at_block_end {
                item_block_end += flex_item.margin_block_end;
                flex_item.margin_block_end = LayoutUnit::default();
            } else {
                has_inflow_child_break_inside_line[flex_line_idx] = true;
            }
            if self.is_column_ {
                let cloned_decorations = if !is_at_block_end
                    && flex_item.block_node.Style().BoxDecorationBreak()
                        == EBoxDecorationBreak::kClone
                {
                    fragment.BoxDecorations().BlockSum()
                } else {
                    LayoutUnit::default()
                };
                flex_item.main_axis_final_size += cloned_decorations;
                flex_item.total_remaining_block_size -= fragment.BlockSize() - cloned_decorations;
                if flex_item.total_remaining_block_size < LayoutUnit::default()
                    && child_break_token.is_null()
                {
                    flex_line.item_offset_adjustment -= flex_item.total_remaining_block_size;
                }
            } else if self.cross_size_adjustments_.is_null()
                && !flex_item.has_descendant_that_depends_on_percentage_block_size
            {
                let line_block_end = flex_line.LineCrossEnd() - offset_in_stitched_container
                    + ClonedBlockStartDecoration(&self.base.container_builder_);
                if line_block_end <= fragmentainer_space
                    && line_block_end >= LayoutUnit::default()
                    && offset_in_stitched_container != LayoutUnit::Max()
                {
                    let expansion = if is_at_block_end {
                        item_block_end - line_block_end
                    } else {
                        (fragmentainer_space - line_block_end).AddEpsilon()
                    };
                    if expansion > LayoutUnit::default() {
                        let key = flex_line_idx as WtfSizeT + 1;
                        if let Some(old) = self.row_cross_size_updates_.get_mut(&key) {
                            if expansion > *old {
                                let delta = expansion - *old;
                                *old = expansion;
                                self.AdjustOffsetForNextLine(
                                    flex_lines,
                                    flex_line_idx as WtfSizeT,
                                    delta,
                                );
                            }
                        } else {
                            self.row_cross_size_updates_.insert(key, expansion);
                            self.AdjustOffsetForNextLine(
                                flex_lines,
                                flex_line_idx as WtfSizeT,
                                expansion,
                            );
                        }
                    }
                }
            }
            if !current_column_break_info.is_null() {
                let info = unsafe { &mut *current_column_break_info };
                info.column_intrinsic_block_size =
                    item_block_end.max(info.column_intrinsic_block_size);
            }
            self.intrinsic_block_size_ = self.intrinsic_block_size_.max(item_block_end);
            self.base.container_builder_.AddResult(
                layout_result,
                offset,
                None,
                None,
                std::ptr::null(),
            );
            if let Some(gaps) = gap_accumulator.as_mut() {
                let mut container_main_end = if self.is_column_ {
                    fragmentainer_space
                } else {
                    self.base.container_builder_.InlineSize() - border_scrollbar_padding
                };
                if self.is_column_ && is_last_item_in_line {
                    container_main_end = fragmentainer_space.max(item_block_end);
                }
                let line_cross_start = if self.is_column_ {
                    flex_line.cross_axis_offset
                } else {
                    offset.block_offset
                };
                let line_cross_end = if self.is_column_ {
                    flex_line.cross_axis_offset + flex_line.line_cross_size
                } else {
                    item_block_end
                };
                if self.is_column_ && is_first_item_in_line {
                    gaps.CalculateColumnFlexLineRowGapStart(
                        unsafe { &*lines_ptr },
                        flex_line_idx as WtfSizeT,
                        previous_gap_data,
                    );
                }
                gaps.BuildGapsForCurrentItem(
                    unsafe { &*lines_ptr },
                    flex_line_idx as WtfSizeT,
                    offset,
                    is_first_item_in_line,
                    is_last_item_in_line,
                    is_last_line,
                    line_cross_start,
                    line_cross_end,
                    container_main_end,
                    true,
                );
                if !self.is_column_
                    && is_last_item_in_line
                    && has_inflow_child_break_inside_line[flex_line_idx]
                    && !is_last_line
                {
                    gaps.DecrementRowGapCount();
                    gaps.SuppressLastMainGap(Some(line_cross_end));
                }
            }
            if !current_column_break_info.is_null() {
                unsafe { &mut *current_column_break_info }.break_after =
                    self.base.container_builder_.PreviousBreakAfter();
            }
            baseline_accumulator.AccumulateItem(
                &fragment,
                offset.block_offset,
                is_first_line,
                is_last_line,
            );
            if is_last_item_in_line
                || (!self.is_column_ && !item_iterator.HasNextItemInLine(flex_line_idx as WtfSizeT))
            {
                if !has_inflow_child_break_inside_line[flex_line_idx] {
                    flex_line.has_seen_all_children = true;
                }
                if !self.has_processed_first_line_ {
                    self.has_processed_first_line_ = true;
                }
                if child_break_token.is_null() || flex_line.has_seen_all_children {
                    if !is_last_line && !self.is_column_ && !item_iterator.HasMoreBreakTokens() {
                        unsafe { &mut (&mut *lines_ptr)[flex_line_idx + 1] }
                            .item_offset_adjustment += flex_line.item_offset_adjustment;
                    }
                }
            }
            self.last_line_idx_to_process_first_child_ = flex_line_idx as WtfSizeT;
            entry = item_iterator.NextItem(broke_before_row);
        }
        if needs_earlier_break_in_column || status == EStatus::kNeedsEarlierBreak {
            return EStatus::kNeedsEarlierBreak;
        }
        if !self.row_cross_size_updates_.is_empty()
            && (self.is_multi_line_ || !self.is_cross_size_definite_)
        {
            debug_assert!(!self.is_column_);
            return EStatus::kNeedsRelayoutWithRowCrossSizeChanges;
        }
        if !self.base.container_builder_.HasInflowChildBreakInside()
            && item_iterator.NextItem(broke_before_row).flex_item.is_null()
        {
            self.base.container_builder_.SetHasSeenAllChildren();
        }
        if let Some(baseline) = baseline_accumulator.FirstBaseline() {
            self.base.container_builder_.SetFirstBaseline(baseline);
        }
        if let Some(baseline) = baseline_accumulator.LastBaseline() {
            self.base.container_builder_.SetLastBaseline(baseline);
        }
        *total_intrinsic_block_size = (*total_intrinsic_block_size)
            .max(self.intrinsic_block_size_ + previously_consumed_block_size);
        status
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:2832-2869
    pub fn ComputeMinMaxSizeOfMultilineColumnContainer(&mut self) -> MinMaxSizesResult {
        let mut min_max_sizes = MinMaxSizes::default();
        let mut flex_lines = FlexLineVector::default();
        self.PlaceFlexItems(Phase::kColumnWrapIntrinsicSize, &mut flex_lines, None, None);
        min_max_sizes.min_size = self.largest_min_content_contribution_;
        if !flex_lines.is_empty() {
            for line in &flex_lines {
                min_max_sizes.max_size += line.line_cross_size;
            }
            min_max_sizes.max_size += (flex_lines.len() as WtfSizeT - 1) * self.gap_between_lines_;
        }
        debug_assert!(min_max_sizes.min_size >= LayoutUnit::default());
        debug_assert!(min_max_sizes.min_size <= min_max_sizes.max_size);
        min_max_sizes += self.BorderScrollbarPadding().InlineSum();
        MinMaxSizesResult::new(min_max_sizes, true)
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:2871-2995
    pub fn ComputeMinMaxSizeOfRowContainer(&mut self) -> MinMaxSizesResult {
        debug_assert!(!self.is_column_);
        let mut container_sizes = MinMaxSizes::default();
        let mut depends_on_block_constraints = false;
        self.ConstructAndAppendFlexItems(Phase::kRowIntrinsicSize, None);
        let result = if self.balance_min_line_count_.is_some() {
            BreakFlexItemsIntoLines(
                &self.flex_items_,
                LayoutUnit::Max(),
                self.gap_between_items_,
                self.is_multi_line_,
                self.balance_min_line_count_,
            )
        } else {
            let mut result = FlexLineBreakerResult::default();
            result.flex_lines.push(InitialFlexLine::new(
                self.flex_items_.len() as WtfSizeT,
                LayoutUnit::default(),
            ));
            result
        };
        let mut remaining_items: &[FlexItem] = &self.flex_items_;
        let mut largest_outer_min_content_contribution = LayoutUnit::default();
        for line in &result.flex_lines {
            let (line_items, rest) = remaining_items.split_at(line.count as usize);
            remaining_items = rest;
            let mut line_sizes = MinMaxSizes::default();
            for item in line_items {
                let child = &item.block_node;
                let space = self.BuildSpaceForIntrinsicInlineSize(child, item.alignment);
                let contributions = ComputeMinAndMaxContentContribution(
                    self.Style(),
                    child,
                    &space,
                    MinMaxSizesFloatInput::default(),
                );
                depends_on_block_constraints |= contributions.depends_on_block_constraints;
                let flex_base_border_box = item.base_content_size + item.main_axis_border_padding;
                let hypothetical_border_box =
                    item.hypothetical_content_size + item.main_axis_border_padding;
                let main_axis_margins = if self.is_horizontal_flow_ {
                    item.initial_margins.HorizontalSum()
                } else {
                    item.initial_margins.VerticalSum()
                };
                let mut item_final_contribution = MinMaxSizes::default();
                if self.is_multi_line_ {
                    largest_outer_min_content_contribution = largest_outer_min_content_contribution
                        .max(contributions.sizes.min_size + main_axis_margins);
                } else {
                    let min_contribution = contributions.sizes.min_size;
                    let cant_move = (min_contribution > flex_base_border_box
                        && item.flex_grow == 0.0)
                        || (min_contribution < flex_base_border_box && item.flex_shrink == 0.0);
                    item_final_contribution.min_size =
                        if cant_move && !item.is_used_flex_basis_indefinite {
                            hypothetical_border_box
                        } else {
                            min_contribution
                        };
                }
                let max_contribution = contributions.sizes.max_size;
                let cant_move = (max_contribution > flex_base_border_box && item.flex_grow == 0.0)
                    || (max_contribution < flex_base_border_box && item.flex_shrink == 0.0);
                item_final_contribution.max_size =
                    if cant_move && !item.is_used_flex_basis_indefinite {
                        hypothetical_border_box
                    } else {
                        max_contribution
                    };
                line_sizes += item_final_contribution;
                line_sizes += main_axis_margins;
            }
            if line.count > 0 {
                line_sizes += (line.count - 1) * self.gap_between_items_;
            }
            container_sizes.Encompass(&line_sizes);
        }
        if self.is_multi_line_ {
            container_sizes.min_size = largest_outer_min_content_contribution;
        }
        debug_assert!(container_sizes.max_size >= container_sizes.min_size || self.is_multi_line_);
        container_sizes.max_size = container_sizes.max_size.max(container_sizes.min_size);
        container_sizes.EncompassValue(LayoutUnit::default());
        container_sizes += self.BorderScrollbarPadding().InlineSum();
        MinMaxSizesResult::new(container_sizes, depends_on_block_constraints)
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:2997-3043
    pub fn ComputeMinMaxSizes(
        &mut self,
        _float_input: &MinMaxSizesFloatInput,
    ) -> MinMaxSizesResult {
        if let Some(result) =
            CalculateMinMaxSizesIgnoringChildren(self.Node(), self.BorderScrollbarPadding())
        {
            return result;
        }
        if !self.is_column_ {
            return self.ComputeMinMaxSizeOfRowContainer();
        }
        if self.is_multi_line_ {
            return self.ComputeMinMaxSizeOfMultilineColumnContainer();
        }
        let mut sizes = MinMaxSizes::default();
        let mut depends_on_block_constraints = false;
        let mut iterator = FlexChildIterator::new(self.Node().clone());
        loop {
            let child = iterator.NextChild();
            if child.IsNull() {
                break;
            }
            if child.IsOutOfFlowPositioned() {
                continue;
            }
            let alignment = ResolvedAlignSelf(
                self.Style(),
                child.Style(),
                self.GetConstraintSpace().GetWritingDirection(),
                self.is_column_,
                false,
            );
            let space = self.BuildSpaceForIntrinsicInlineSize(&child, alignment);
            let mut child_result = ComputeMinAndMaxContentContribution(
                self.Style(),
                &child,
                &space,
                MinMaxSizesFloatInput::default(),
            );
            let child_margins = ComputeMarginsFor(&space, child.Style(), self.GetConstraintSpace());
            child_result.sizes += child_margins.InlineSum();
            depends_on_block_constraints |= child_result.depends_on_block_constraints;
            sizes.min_size = sizes.min_size.max(child_result.sizes.min_size);
            sizes.max_size = sizes.max_size.max(child_result.sizes.max_size);
        }
        sizes.max_size = sizes.max_size.max(sizes.min_size);
        sizes.EncompassValue(LayoutUnit::default());
        sizes += self.BorderScrollbarPadding().InlineSum();
        MinMaxSizesResult::new(sizes, depends_on_block_constraints)
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:3045-3049
    pub fn FragmentainerSpaceAvailable(&self, block_offset: LayoutUnit) -> LayoutUnit {
        (self.FragmentainerSpaceLeftForChildren() - block_offset).ClampNegativeToZero()
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:3051-3083
    pub fn ConsumeRemainingFragmentainerSpace(
        &mut self,
        offset_in_stitched_container: LayoutUnit,
        flex_line: &mut FlexLine,
        column_break_info: Option<&FlexColumnBreakInfo>,
    ) {
        let last_child_break_token = self.base.container_builder_.LastChildBreakToken();
        let last_child_break_token = unsafe { &*To::<BlockBreakToken>(last_child_break_token) };
        if last_child_break_token.IsForcedBreak() {
            let mut intrinsic_block_size = self.intrinsic_block_size_;
            if let Some(info) = column_break_info {
                debug_assert!(self.is_column_);
                intrinsic_block_size = info.column_intrinsic_block_size;
            }
            flex_line.item_offset_adjustment -= intrinsic_block_size + offset_in_stitched_container
                - ClonedBlockStartDecoration(&self.base.container_builder_);
        }
        if self.GetConstraintSpace().HasKnownFragmentainerBlockSize() {
            self.intrinsic_block_size_ +=
                self.FragmentainerSpaceAvailable(self.intrinsic_block_size_);
        }
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:3085-3141
    #[allow(clippy::too_many_arguments)]
    pub fn BreakBeforeRowIfNeeded(
        &mut self,
        row: &FlexLine,
        row_block_offset: LayoutUnit,
        row_break_between: EBreakBetween,
        row_index: WtfSizeT,
        child: LayoutInputNode,
        has_container_separation: bool,
        is_first_for_row: bool,
    ) -> BreakStatus {
        debug_assert!(!self.is_column_);
        debug_assert!(InvolvedInBlockFragmentationForBuilder(
            &self.base.container_builder_
        ));
        let fragmentainer_block_offset = self.FragmentainerOffsetForChildren() + row_block_offset;
        let fragmentainer_block_size = self.FragmentainerCapacityForChildren();
        if has_container_separation
            && IsForcedBreakValue(self.GetConstraintSpace(), row_break_between)
        {
            BreakBeforeChild(
                child,
                std::ptr::null(),
                fragmentainer_block_offset,
                fragmentainer_block_size,
                Some(BreakAppeal::kBreakAppealPerfect),
                true,
                &mut self.base.container_builder_,
                Some(row.line_cross_size),
            );
            return BreakStatus::kBrokeBefore;
        }
        let breakable_at_start = IsBreakableAtStartOfResumedContainer(
            self.GetConstraintSpace(),
            &self.base.container_builder_,
            is_first_for_row,
        );
        let appeal_before = CalculateBreakAppealBeforeStatus(
            self.GetConstraintSpace(),
            EStatus::kSuccess,
            row_break_between,
            has_container_separation,
            breakable_at_start,
        );
        if self.MovePastRowBreakPoint(
            appeal_before,
            fragmentainer_block_offset,
            row.line_cross_size,
            row_index,
            has_container_separation,
            breakable_at_start,
        ) {
            return BreakStatus::kContinue;
        }
        if !AttemptSoftBreak(
            child,
            std::ptr::null(),
            fragmentainer_block_offset,
            fragmentainer_block_size,
            appeal_before,
            &mut self.base.container_builder_,
            Some(row.line_cross_size),
            std::ptr::null_mut(),
        ) {
            return BreakStatus::kNeedsEarlierBreak;
        }
        BreakStatus::kBrokeBefore
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:3143-3183
    #[allow(clippy::too_many_arguments)]
    pub fn MovePastRowBreakPoint(
        &mut self,
        appeal_before: BreakAppeal,
        fragmentainer_block_offset: LayoutUnit,
        row_block_size: LayoutUnit,
        row_index: WtfSizeT,
        has_container_separation: bool,
        breakable_at_start_of_container: bool,
    ) -> bool {
        if !self.GetConstraintSpace().HasKnownFragmentainerBlockSize() {
            return true;
        }
        let space_left = self.FragmentainerCapacityForChildren() - fragmentainer_block_offset;
        let must_break_before = space_left < LayoutUnit::default()
            || (space_left == LayoutUnit::default() && row_block_size != LayoutUnit::default());
        if must_break_before {
            debug_assert!(space_left < self.FragmentainerCapacityForChildren());
            return false;
        }
        if (has_container_separation || breakable_at_start_of_container)
            && (!self.base.container_builder_.HasEarlyBreak()
                || appeal_before
                    >= self
                        .base
                        .container_builder_
                        .GetEarlyBreak()
                        .GetBreakAppeal())
        {
            let breakpoint =
                MakeGarbageCollected(EarlyBreak::from_line(row_index as i32, appeal_before));
            self.base.container_builder_.SetEarlyBreak(breakpoint);
        }
        true
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:3185-3190
    pub fn AddColumnEarlyBreak(&mut self, breakpoint: *mut EarlyBreak, index: WtfSizeT) {
        debug_assert!(self.is_column_);
        while self.column_early_breaks_.len() <= index as usize {
            self.column_early_breaks_.push(Member::default());
        }
        self.column_early_breaks_[index as usize] = Member::from_ptr(breakpoint);
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:3192-3201
    pub fn AdjustOffsetForNextLine(
        &self,
        flex_lines: &mut FlexLineVector,
        flex_line_idx: WtfSizeT,
        item_expansion: LayoutUnit,
    ) {
        debug_assert!((flex_line_idx as usize) < flex_lines.len());
        if flex_line_idx as usize + 1 < flex_lines.len() {
            flex_lines[flex_line_idx as usize + 1].item_offset_adjustment += item_expansion;
        }
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:3203-3230
    pub fn RelayoutWithNewRowSizes(&self) -> *const LayoutResult {
        debug_assert!(self.cross_size_adjustments_.is_null());
        debug_assert!(!self.row_cross_size_updates_.is_empty());
        debug_assert!(self.row_cross_size_updates_.len() <= 2);
        let mut params = LayoutAlgorithmParams::new(
            self.Node().clone(),
            self.base.container_builder_.InitialFragmentGeometry(),
            self.GetConstraintSpace(),
        );
        params.break_token = self.GetBreakToken();
        params.early_break = self.early_break_;
        params.additional_early_breaks = self.additional_early_breaks_;
        let mut algorithm =
            Self::new_with_cross_size_adjustments(&params, &self.row_cross_size_updates_);
        algorithm
            .base
            .container_builder_
            .SetBoxType(self.base.container_builder_.GetBoxType());
        algorithm.ignore_child_scrollbar_changes_ = self.ignore_child_scrollbar_changes_;
        if !self.early_break_.is_null() {
            algorithm
                .base
                .container_builder_
                .PropagateSpaceShortage(self.base.container_builder_.MinimalSpaceShortage());
        }
        algorithm.Layout()
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:3232-3289
    pub fn MinBlockSizeShouldEncompassIntrinsicSize(&self, item: &FlexItemData) -> bool {
        if item.has_descendant_that_depends_on_percentage_block_size
            || item.block_node.IsMonolithic()
        {
            return false;
        }
        let item_style = item.block_node.Style();
        debug_assert_eq!(
            self.GetConstraintSpace()
                .GetWritingDirection()
                .GetWritingMode(),
            item_style.GetWritingMode(),
        );
        if self.is_column_ {
            let can_shrink = item_style.ResolvedFlexShrink(self.Style()) != 0.0
                && self.ChildAvailableSize().block_size != kIndefiniteSize;
            if item.is_used_flex_basis_indefinite && !can_shrink {
                return true;
            }
            if item_style.LogicalHeight().HasAutoOrContentOrIntrinsic()
                && (!can_shrink || self.ShouldApplyAutoMinSize(&item.block_node))
            {
                return true;
            }
        } else {
            if self.WillChildCrossSizeBeContainerCrossSize(&item.block_node, item.alignment)
                && !self.Style().LogicalHeight().HasAutoOrContentOrIntrinsic()
            {
                return false;
            }
            if self.DoesItemComputedCrossSizeHaveAuto(&item.block_node) {
                return true;
            }
        }
        false
    }

    // cpp: layoutng_flex/flex_layout_algorithm.cc:2788-2830
    pub fn UpdateOffsetAdjustmentForSuppressedRowGap(
        &self,
        gap: LayoutUnit,
        previous_content_block_end: LayoutUnit,
        flex_line: &mut FlexLine,
    ) {
        if gap == LayoutUnit::default()
            || !self.GetConstraintSpace().HasKnownFragmentainerBlockSize()
        {
            return;
        }
        let last_child_break_token = self.base.container_builder_.LastChildBreakToken();
        let last_child_break_token = unsafe { &*To::<BlockBreakToken>(last_child_break_token) };
        if last_child_break_token.IsForcedBreak() {
            flex_line.item_offset_adjustment -= gap;
            return;
        }
        let available_space = self.FragmentainerSpaceAvailable(previous_content_block_end);
        if gap > available_space {
            flex_line.item_offset_adjustment -= gap - available_space;
        }
    }
}

// cpp: layoutng_flex/flex_layout_algorithm.h:25-25
impl Drop for FlexLayoutAlgorithm {
    fn drop(&mut self) {
        self.flex_items_.clear();
    }
}

impl NativeAlgorithm for FlexLayoutAlgorithm {
    fn new(params: &LayoutAlgorithmParams) -> Self {
        FlexLayoutAlgorithm::new(params)
    }

    fn layout(&mut self) -> *const LayoutResult {
        self.Layout()
    }

    fn compute_min_max_sizes(&mut self, input: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        self.ComputeMinMaxSizes(input)
    }
}

impl RelayoutAlgorithm<BlockNode> for FlexLayoutAlgorithm {
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

    fn setup_relayout_data(&mut self, previous: &Self, relayout_type: RelayoutType) {
        self.SetupRelayoutData(previous, relayout_type);
    }
}

// cpp: layoutng_flex/flex_layout_algorithm.cc:643-649
fn IsContainerCrossSizeDefinite(
    base: &LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>,
    is_column: bool,
) -> bool {
    if is_column {
        return true;
    }
    base.ChildAvailableSize().block_size != foundation::kIndefiniteSize
}
