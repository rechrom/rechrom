#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{
    kIndefiniteSize, ETextAlign, LayoutUnit, MakeGarbageCollected, To, WritingDirectionMode,
};
use layoutng_assembly::block_break_token::BlockBreakToken;
use layoutng_assembly::box_fragment_builder::BoxFragmentBuilder;
use layoutng_assembly::internal::algorithm_entry::NativeAlgorithm;
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::break_appeal::kBreakAppealPerfect;
use layoutng_assembly::internal::constraint_space::{AutoSizeBehavior, ConstraintSpace};
use layoutng_assembly::internal::constraint_space_builder::ConstraintSpaceBuilder;
use layoutng_assembly::internal::constraint_space_builder_style::MinMaxConstraintSpaceBuilder;
use layoutng_assembly::internal::fragmentation_utils::{
    BreakStatus, EnterEarlyBreakInChild, FinishFragmentation,
    InvolvedInBlockFragmentationForBuilder, IsBreakInside, IsEarlyBreakTarget,
    SetupSpaceBuilderForFragmentationFromBuilder, ShouldIncludeBlockStartBorderPadding,
};
use layoutng_assembly::internal::layout_algorithm::{
    LayoutAlgorithm, LayoutAlgorithmParams, RelayoutAlgorithm,
};
use layoutng_assembly::internal::layout_input_node::MinMaxSizesFloatInput;
use layoutng_assembly::internal::min_max_sizes::MinMaxSizesResult;
use layoutng_assembly::internal::space_utils::SetOrthogonalFallbackInlineSizeIfNeeded;
use layoutng_assembly::layout_result::EStatus;
use layoutng_assembly::layout_result::LayoutResult;
use layoutng_assembly::logical_fragment::LogicalFragment;
use layoutng_assembly::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_size::LogicalSize;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::fieldset_break_token_data::FieldsetBreakTokenData;
use layoutng_assembly::internal::length_utils::{
    CalculateChildPercentageSize, CalculateMinMaxSizesIgnoringChildren, ClampIntrinsicBlockSize,
    ComputeBlockSizeForFragment, ComputeBorders, ComputeMarginsFor, ComputeMarginsForInlineSize,
    ComputeMinAndMaxContentContribution, ComputePadding, ResolveInitialMaxBlockLength,
    ShrinkLogicalSize,
};

// cpp: layoutng_forms/fieldset_layout_algorithm.cc:22-26
#[derive(Clone, Copy, PartialEq, Eq)]
enum LegendBlockAlignment {
    kStart,
    kCenter,
    kEnd,
}

// cpp: layoutng_forms/fieldset_layout_algorithm.cc:28-59
fn ComputeLegendBlockAlignment(
    legend_style: &ComputedStyle,
    fieldset_style: &ComputedStyle,
) -> LegendBlockAlignment {
    let start_auto = legend_style.MarginInlineStartUsing(fieldset_style).IsAuto();
    let end_auto = legend_style.MarginInlineEndUsing(fieldset_style).IsAuto();
    if start_auto || end_auto {
        if start_auto {
            return if end_auto {
                LegendBlockAlignment::kCenter
            } else {
                LegendBlockAlignment::kEnd
            };
        }
        return LegendBlockAlignment::kStart;
    }
    let is_ltr = fieldset_style.IsLeftToRightDirection();
    match legend_style.GetTextAlign() {
        ETextAlign::kLeft => {
            if is_ltr {
                LegendBlockAlignment::kStart
            } else {
                LegendBlockAlignment::kEnd
            }
        }
        ETextAlign::kRight => {
            if is_ltr {
                LegendBlockAlignment::kEnd
            } else {
                LegendBlockAlignment::kStart
            }
        }
        ETextAlign::kCenter => LegendBlockAlignment::kCenter,
        _ => LegendBlockAlignment::kStart,
    }
}

// cpp: layoutng_forms/fieldset_layout_algorithm.h:18-63
#[repr(C)]
pub struct FieldsetLayoutAlgorithm {
    base: LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>,
    writing_direction_: WritingDirectionMode,
    intrinsic_block_size_: LayoutUnit,
    consumed_block_size_: LayoutUnit,
    border_box_size_: LogicalSize,
    minimum_border_box_block_size_: LayoutUnit,
}

impl Deref for FieldsetLayoutAlgorithm {
    type Target = LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for FieldsetLayoutAlgorithm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

const _: () = assert!(std::mem::offset_of!(FieldsetLayoutAlgorithm, base) == 0);

impl FieldsetLayoutAlgorithm {
    // cpp: layoutng_forms/fieldset_layout_algorithm.h:20-20
    // cpp: layoutng_forms/fieldset_layout_algorithm.cc:63-72
    pub fn new(params: &LayoutAlgorithmParams) -> Self {
        let base = LayoutAlgorithm::from_params(params);
        let writing_direction_ = base.GetConstraintSpace().GetWritingDirection();
        let consumed_block_size_ = if !base.GetBreakToken().is_null() {
            unsafe { &*base.GetBreakToken() }.ConsumedBlockSize()
        } else {
            LayoutUnit::new()
        };
        debug_assert!(unsafe { &*params.fragment_geometry }.scrollbar.IsEmpty());
        let border_box_size_ = *base.container_builder_.InitialBorderBoxSize();
        Self {
            base,
            writing_direction_,
            intrinsic_block_size_: LayoutUnit::new(),
            consumed_block_size_,
            border_box_size_,
            minimum_border_box_block_size_: LayoutUnit::new(),
        }
    }

    // cpp: layoutng_forms/fieldset_layout_algorithm.h:24-24
    // cpp: layoutng_forms/fieldset_layout_algorithm.cc:71-160
    pub fn Layout(&mut self) -> *const LayoutResult {
        if ShouldIncludeBlockStartBorderPadding(&self.base.container_builder_) {
            self.intrinsic_block_size_ = self.Borders().block_start;
        }

        if InvolvedInBlockFragmentationForBuilder(&self.base.container_builder_) {
            self.base
                .container_builder_
                .SetBreakTokenData(MakeGarbageCollected(FieldsetBreakTokenData::new()).cast());
        }

        let break_status = self.LayoutChildren();
        if break_status == BreakStatus::kNeedsEarlierBreak {
            return self
                .base
                .container_builder_
                .Abort(EStatus::kNeedsEarlierBreak);
        }

        let border_scrollbar_padding = *self.Borders() + *self.Scrollbar() + *self.Padding();
        self.intrinsic_block_size_ = ClampIntrinsicBlockSize(
            self.GetConstraintSpace(),
            self.Node(),
            self.GetBreakToken(),
            &border_scrollbar_padding,
            self.intrinsic_block_size_ + self.Borders().block_end,
            None,
        );

        self.border_box_size_.block_size = ComputeBlockSizeForFragment(
            self.GetConstraintSpace(),
            self.Node(),
            self.BorderPadding(),
            self.intrinsic_block_size_ + self.consumed_block_size_,
            self.border_box_size_.inline_size,
            kIndefiniteSize,
        );
        if !self.Node().ShouldApplyBlockSizeContainment() {
            self.border_box_size_.block_size = self
                .border_box_size_
                .block_size
                .max(self.minimum_border_box_block_size_);
        }

        let all_fragments_block_size = self.border_box_size_.block_size;
        self.base
            .container_builder_
            .SetIntrinsicBlockSize(self.intrinsic_block_size_);
        self.base
            .container_builder_
            .SetFragmentsTotalBlockSize(all_fragments_block_size);
        self.base.container_builder_.SetIsFieldsetContainer();

        if InvolvedInBlockFragmentationForBuilder(&self.base.container_builder_) {
            let status = FinishFragmentation(&mut self.base.container_builder_);
            if status == BreakStatus::kNeedsEarlierBreak {
                let early_break = self.base.container_builder_.GetEarlyBreak() as *const _;
                return self
                    .base
                    .RelayoutAndBreakEarlierDefault::<Self>(unsafe { &*early_break });
            } else if status == BreakStatus::kDisableFragmentation {
                return self.base.RelayoutWithoutFragmentation::<Self>();
            }
            debug_assert_eq!(status, BreakStatus::kContinue);
        } else {
            #[cfg(debug_assertions)]
            self.base.container_builder_.CheckNoBlockFragmentation();
        }

        self.base
            .container_builder_
            .HandleOofsAndSpecialDescendants();

        let style = self.Style();
        if style.LogicalHeight().MayHavePercentDependence()
            || style.LogicalMinHeight().MayHavePercentDependence()
            || style.LogicalMaxHeight().MayHavePercentDependence()
        {
            self.base
                .container_builder_
                .SetHasDescendantThatDependsOnPercentageBlockSize(true);
        }

        self.base.container_builder_.ToBoxFragment()
    }

    // cpp: layoutng_forms/fieldset_layout_algorithm.h:29-29
    // cpp: layoutng_forms/fieldset_layout_algorithm.cc:162-238
    fn LayoutChildren(&mut self) -> BreakStatus {
        let mut content_break_token: *const BlockBreakToken = std::ptr::null();
        let mut has_seen_all_children = false;
        let token = self.GetBreakToken();
        if !token.is_null() {
            let child_tokens = unsafe { &*token }.ChildBreakTokens();
            if !child_tokens.is_empty() {
                let child_token = To::<BlockBreakToken>(child_tokens[0].Get());
                if !child_token.is_null() {
                    debug_assert!(!unsafe { &*child_token }.InputNode().IsRenderedLegend());
                    content_break_token = child_token;
                }
                debug_assert_eq!(child_tokens.len(), 1);
            }
            if unsafe { &*token }.HasSeenAllChildren() {
                self.base.container_builder_.SetHasSeenAllChildren();
                has_seen_all_children = true;
            }
        }

        let mut adjusted_padding_box_size =
            ShrinkLogicalSize(self.border_box_size_, self.Borders());
        let mut legend = self.Node().GetRenderedLegend();
        if !legend.IsNull() {
            if !IsBreakInside(self.GetBreakToken()) {
                self.LayoutLegend(&mut legend);
            }
            let legend_size_contribution = if IsBreakInside(self.GetBreakToken()) {
                let token_data =
                    To::<FieldsetBreakTokenData>(unsafe { &*self.GetBreakToken() }.TokenData());
                unsafe { &*token_data }.legend_block_size_contribution
            } else {
                self.intrinsic_block_size_ - self.Borders().block_start
            };

            if InvolvedInBlockFragmentationForBuilder(&self.base.container_builder_) {
                let token_data =
                    To::<FieldsetBreakTokenData>(self.base.container_builder_.GetBreakTokenData());
                unsafe { &mut *token_data }.legend_block_size_contribution =
                    legend_size_contribution;
            }

            if adjusted_padding_box_size.block_size != kIndefiniteSize {
                debug_assert_ne!(self.border_box_size_.block_size, kIndefiniteSize);
                adjusted_padding_box_size.block_size = (adjusted_padding_box_size.block_size
                    - legend_size_contribution)
                    .max(self.Padding().BlockSum());
            }

            if !self.Node().ShouldApplyBlockSizeContainment() {
                self.minimum_border_box_block_size_ =
                    self.BorderPadding().BlockSum() + legend_size_contribution;
            }
        }

        if !content_break_token.is_null() || !has_seen_all_children {
            let mut fieldset_content = self.Node().GetFieldsetContent();
            debug_assert!(!fieldset_content.IsNull());
            let break_status = self.LayoutFieldsetContent(
                &mut fieldset_content,
                content_break_token,
                adjusted_padding_box_size,
                !legend.IsNull(),
            );
            if break_status == BreakStatus::kNeedsEarlierBreak {
                return break_status;
            }
        }

        BreakStatus::kContinue
    }

    // cpp: layoutng_forms/fieldset_layout_algorithm.h:30-30
    // cpp: layoutng_forms/fieldset_layout_algorithm.cc:240-314
    fn LayoutLegend(&mut self, legend: &mut BlockNode) {
        let percentage_size = CalculateChildPercentageSize(
            self.GetConstraintSpace(),
            self.Node(),
            *self.ChildAvailableSize(),
        );
        let legend_margins = ComputeMarginsForInlineSize(
            legend.Style(),
            percentage_size.inline_size,
            self.GetConstraintSpace().GetWritingDirection(),
        );
        let legend_space = self.CreateConstraintSpaceForLegend(
            legend,
            *self.ChildAvailableSize(),
            percentage_size,
        );
        let result = legend.Layout(
            &legend_space,
            self.GetBreakToken(),
            std::ptr::null(),
            std::ptr::null(),
        );
        let result = unsafe { &*result };
        debug_assert_eq!(result.Status(), EStatus::kSuccess);
        let logical_fragment =
            LogicalFragment::new(self.writing_direction_, result.GetPhysicalFragment());
        let legend_border_box_block_size = logical_fragment.BlockSize();
        let legend_margin_box_block_size =
            legend_margins.block_start + legend_border_box_block_size + legend_margins.block_end;
        let space_left = self.Borders().block_start - legend_border_box_block_size;
        let mut block_offset = LayoutUnit::new();
        if space_left > LayoutUnit::new() {
            block_offset += space_left / 2;
        }
        let legend_margin_end_offset =
            block_offset + legend_margin_box_block_size - legend_margins.block_start;
        if legend_margin_end_offset > self.Borders().block_start {
            self.intrinsic_block_size_ = legend_margin_end_offset;
        }

        let legend_border_box_inline_size = logical_fragment.InlineSize();
        let mut legend_inline_start = self.Borders().inline_start
            + self.Scrollbar().inline_start
            + self.Padding().inline_start
            + legend_margins.inline_start;
        let available_space = self.ChildAvailableSize().inline_size - legend_border_box_inline_size;
        if available_space > LayoutUnit::new() {
            match ComputeLegendBlockAlignment(legend.Style(), self.Style()) {
                LegendBlockAlignment::kCenter => legend_inline_start += available_space / 2,
                LegendBlockAlignment::kEnd => {
                    legend_inline_start += available_space - legend_margins.inline_end
                }
                LegendBlockAlignment::kStart => {}
            }
        }
        self.base.container_builder_.AddResult(
            result,
            LogicalOffset::new(legend_inline_start, block_offset),
            None,
            None,
            std::ptr::null(),
        );
    }

    // cpp: layoutng_forms/fieldset_layout_algorithm.h:51-51
    // cpp: layoutng_forms/fieldset_layout_algorithm.cc:424-432
    fn FragmentainerSpaceAvailable(&self) -> LayoutUnit {
        (self.FragmentainerSpaceLeftForChildren() - self.intrinsic_block_size_)
            .max(LayoutUnit::new())
    }

    // cpp: layoutng_forms/fieldset_layout_algorithm.h:57-57
    // cpp: layoutng_forms/fieldset_layout_algorithm.cc:434-441
    fn ConsumeRemainingFragmentainerSpace(&mut self) {
        if self.GetConstraintSpace().HasKnownFragmentainerBlockSize() {
            self.intrinsic_block_size_ += self.FragmentainerSpaceAvailable();
        }
    }

    // cpp: layoutng_forms/fieldset_layout_algorithm.h:31-35
    // cpp: layoutng_forms/fieldset_layout_algorithm.cc:316-422
    fn LayoutFieldsetContent(
        &mut self,
        fieldset_content: &mut BlockNode,
        content_break_token: *const BlockBreakToken,
        mut adjusted_padding_box_size: LogicalSize,
        _has_legend: bool,
    ) -> BreakStatus {
        let mut early_break_in_child = std::ptr::null();
        if !self.early_break_.is_null() {
            let early_break = unsafe { &*self.early_break_ };
            if IsEarlyBreakTarget(
                early_break,
                &self.base.container_builder_,
                &fieldset_content.base,
            ) {
                self.base.container_builder_.AddBreakBeforeChild(
                    fieldset_content.base.clone(),
                    Some(kBreakAppealPerfect),
                    false,
                    LogicalOffset::default(),
                );
                self.ConsumeRemainingFragmentainerSpace();
                return BreakStatus::kContinue;
            }
            early_break_in_child = EnterEarlyBreakInChild(fieldset_content, early_break);
        }

        let mut result: *const LayoutResult = std::ptr::null();
        let is_past_end =
            !self.GetBreakToken().is_null() && unsafe { &*self.GetBreakToken() }.IsAtBlockEnd();
        let mut max_content_block_size = LayoutUnit::Max();
        if adjusted_padding_box_size.block_size == kIndefiniteSize {
            max_content_block_size = ResolveInitialMaxBlockLength(
                self.GetConstraintSpace(),
                self.Style(),
                self.BorderPadding(),
                self.Style().LogicalMaxHeight(),
                kIndefiniteSize,
            );
        }
        if !is_past_end || max_content_block_size == LayoutUnit::Max() {
            let child_space = self.CreateConstraintSpaceForFieldsetContent(
                fieldset_content,
                adjusted_padding_box_size,
                self.intrinsic_block_size_,
            );
            result = fieldset_content.Layout(
                &child_space,
                content_break_token,
                early_break_in_child,
                std::ptr::null(),
            );
        }
        if max_content_block_size != LayoutUnit::Max()
            && (result.is_null() || unsafe { &*result }.Status() == EStatus::kSuccess)
        {
            debug_assert_eq!(adjusted_padding_box_size.block_size, kIndefiniteSize);
            if max_content_block_size > self.Padding().BlockSum() {
                max_content_block_size = (max_content_block_size
                    - (self.intrinsic_block_size_ + self.Borders().block_end))
                    .max(self.Padding().BlockSum());
            }
            if !result.is_null() {
                let fragment = unsafe { &*result }.GetPhysicalFragment();
                let mut total_block_size =
                    LogicalFragment::new(self.writing_direction_, fragment).BlockSize();
                if !content_break_token.is_null() {
                    total_block_size += unsafe { &*content_break_token }.ConsumedBlockSize();
                }
                if total_block_size >= max_content_block_size {
                    result = std::ptr::null();
                }
            } else {
                debug_assert!(is_past_end);
            }
            if result.is_null() {
                adjusted_padding_box_size.block_size = max_content_block_size;
                let adjusted_child_space = self.CreateConstraintSpaceForFieldsetContent(
                    fieldset_content,
                    adjusted_padding_box_size,
                    self.intrinsic_block_size_,
                );
                result = fieldset_content.Layout(
                    &adjusted_child_space,
                    content_break_token,
                    early_break_in_child,
                    std::ptr::null(),
                );
            }
        }
        debug_assert!(!result.is_null());
        let result = unsafe { &*result };
        let mut break_status = BreakStatus::kContinue;
        if self.GetConstraintSpace().HasBlockFragmentation() && self.early_break_.is_null() {
            let block_offset = self.FragmentainerOffsetForChildren() + self.intrinsic_block_size_;
            break_status = self.BreakBeforeChildIfNeeded(
                fieldset_content.base.clone(),
                result,
                block_offset,
                false,
            );
        }
        if break_status == BreakStatus::kContinue {
            debug_assert_eq!(result.Status(), EStatus::kSuccess);
            let offset =
                LogicalOffset::new(self.Borders().inline_start, self.intrinsic_block_size_);
            self.base
                .container_builder_
                .AddResult(result, offset, None, None, std::ptr::null());
            let fragment =
                unsafe { &*To::<PhysicalBoxFragment>(result.GetPhysicalFragment() as *const _) };
            if let Some(first_baseline) = fragment.FirstBaseline() {
                self.base
                    .container_builder_
                    .SetFirstBaseline(offset.block_offset + first_baseline);
            }
            if let Some(last_baseline) = fragment.LastBaseline() {
                self.base
                    .container_builder_
                    .SetLastBaseline(offset.block_offset + last_baseline);
            }
            if fragment.UseLastBaselineForInlineBaseline() {
                self.base
                    .container_builder_
                    .SetUseLastBaselineForInlineBaseline();
            }
            self.intrinsic_block_size_ +=
                LogicalFragment::new(self.writing_direction_, fragment).BlockSize();
            self.base.container_builder_.SetHasSeenAllChildren();
        } else if break_status == BreakStatus::kBrokeBefore {
            self.ConsumeRemainingFragmentainerSpace();
        }
        break_status
    }

    // cpp: layoutng_forms/fieldset_layout_algorithm.h:26-26
    // cpp: layoutng_forms/fieldset_layout_algorithm.cc:443-500
    pub fn ComputeMinMaxSizes(&mut self, _input: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        let mut result = MinMaxSizesResult::default();
        let has_inline_size_containment = self.Node().ShouldApplyInlineSizeContainment();
        if has_inline_size_containment {
            let edges = *self.Borders() + *self.Scrollbar() + *self.Padding();
            if let Some(result_without_children) =
                CalculateMinMaxSizesIgnoringChildren(self.Node(), &edges)
            {
                return result_without_children;
            }
        } else {
            let legend = self.Node().GetRenderedLegend();
            if !legend.IsNull() {
                let mut builder = MinMaxConstraintSpaceBuilder::new(
                    self.GetConstraintSpace(),
                    self.Style(),
                    &legend.base,
                    true,
                );
                builder.SetAvailableBlockSize(kIndefiniteSize);
                let space = builder.ToConstraintSpace();
                result = ComputeMinAndMaxContentContribution(
                    self.Style(),
                    &legend,
                    &space,
                    MinMaxSizesFloatInput::default(),
                );
                result.sizes +=
                    ComputeMarginsFor(&space, legend.Style(), self.GetConstraintSpace())
                        .InlineSum();
            }
        }
        result.sizes += ComputePadding(self.GetConstraintSpace(), self.Style()).InlineSum();
        if !has_inline_size_containment {
            let content = self.Node().GetFieldsetContent();
            debug_assert!(!content.IsNull());
            let mut builder = MinMaxConstraintSpaceBuilder::new(
                self.GetConstraintSpace(),
                self.Style(),
                &content.base,
                true,
            );
            builder.SetAvailableBlockSize(kIndefiniteSize);
            let space = builder.ToConstraintSpace();
            let mut content_result = ComputeMinAndMaxContentContribution(
                self.Style(),
                &content,
                &space,
                MinMaxSizesFloatInput::default(),
            );
            content_result.sizes +=
                ComputeMarginsFor(&space, content.Style(), self.GetConstraintSpace()).InlineSum();
            result.sizes.Encompass(&content_result.sizes);
            result.depends_on_block_constraints |= content_result.depends_on_block_constraints;
        }
        result.sizes += ComputeBorders(self.GetConstraintSpace(), self.Node()).InlineSum();
        result
    }

    // cpp: layoutng_forms/fieldset_layout_algorithm.h:37-40
    // cpp: layoutng_forms/fieldset_layout_algorithm.cc:502-514
    fn CreateConstraintSpaceForLegend(
        &self,
        legend: &BlockNode,
        available_size: LogicalSize,
        percentage_size: LogicalSize,
    ) -> ConstraintSpace {
        let mut builder = ConstraintSpaceBuilder::new(
            self.GetConstraintSpace(),
            legend.Style().GetWritingDirection(),
            true,
        );
        SetOrthogonalFallbackInlineSizeIfNeeded(self.Style(), legend.base.clone(), &mut builder);
        builder.SetAvailableSize(available_size);
        builder.SetPercentageResolutionSize(percentage_size);
        builder.ToConstraintSpace()
    }

    // cpp: layoutng_forms/fieldset_layout_algorithm.h:41-45
    // cpp: layoutng_forms/fieldset_layout_algorithm.cc:516-547
    fn CreateConstraintSpaceForFieldsetContent(
        &self,
        fieldset_content: &BlockNode,
        padding_box_size: LogicalSize,
        block_offset: LayoutUnit,
    ) -> ConstraintSpace {
        debug_assert!(fieldset_content.CreatesNewFormattingContext());
        let mut builder = ConstraintSpaceBuilder::new(
            self.GetConstraintSpace(),
            fieldset_content.Style().GetWritingDirection(),
            true,
        );
        builder.SetAvailableSize(padding_box_size);
        builder.SetInlineAutoBehavior(AutoSizeBehavior::kStretchImplicit);
        builder.SetPercentageResolutionSize(self.GetConstraintSpace().PercentageResolutionSize());
        builder.SetIsFixedBlockSize(padding_box_size.block_size != kIndefiniteSize);
        builder.SetBaselineAlgorithmType(self.GetConstraintSpace().GetBaselineAlgorithmType());
        if self.GetConstraintSpace().HasBlockFragmentation() {
            SetupSpaceBuilderForFragmentationFromBuilder(
                &self.base.container_builder_,
                &fieldset_content.base,
                block_offset,
                &mut builder,
            );
        }
        builder.ToConstraintSpace()
    }
}

// cpp: layoutng_forms/fieldset_layout_algorithm.h:18-26
impl NativeAlgorithm for FieldsetLayoutAlgorithm {
    fn new(params: &LayoutAlgorithmParams) -> Self {
        FieldsetLayoutAlgorithm::new(params)
    }
    fn layout(&mut self) -> *const LayoutResult {
        self.Layout()
    }
    fn compute_min_max_sizes(&mut self, input: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        self.ComputeMinMaxSizes(input)
    }
}

// cpp: layoutng_forms/fieldset_layout_algorithm.h:18-63
impl RelayoutAlgorithm<BlockNode> for FieldsetLayoutAlgorithm {
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
