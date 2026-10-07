#![allow(non_snake_case)]
use crate::{
    table_break_token_data::TableBreakTokenData, table_child_iterator::TableChildIterator,
    table_layout_algorithm::TableLayoutAlgorithm, table_layout_support::*,
};
use foundation::{
    kIndefiniteSize, DynamicTo, EBoxDecorationBreak, ECaptionSide, HeapVector, LayoutUnit,
    MakeGarbageCollected, To, UnsupportedLayout, Vector, WritingDirectionMode,
};
use layoutng_assembly::{
    block_break_token::BlockBreakToken,
    box_fragment_builder::BoxFragmentBuilder,
    internal::{
        block_node::BlockNode,
        break_appeal::{kBreakAppealPerfect, BreakAppeal::kBreakAppealLastResort},
        constraint_space::ConstraintSpace,
        constraint_space_builder::ConstraintSpaceBuilder,
        disable_layout_side_effects_scope::DisableLayoutSideEffectsScope,
        fragmentation_utils::{
            BreakBeforeChild, BreakStatus, EnterEarlyBreakInChild, FinishFragmentation,
            InvolvedInBlockFragmentationForBuilder, IsAvoidBreakValue, IsBreakInside,
            IsEarlyBreakTarget, MovePastBreakpointFull,
            SetupSpaceBuilderForFragmentationFromBuilder,
        },
        layout_input::NodeKind,
        layout_pass_scope::LayoutPassScope,
        table_borders::TableBorders,
        table_column_location::TableColumnLocation,
        table_constraint_space_data::TableConstraintSpaceData,
        table_layout_algorithm_types::{
            CellBlockConstraints, Rows, Sections, TableGroupedChildren,
        },
    },
    layout_result::{EStatus, LayoutResult},
    logical_box_fragment::LogicalBoxFragment,
    logical_fragment::LogicalFragment,
    physical_box_fragment::PhysicalBoxFragment,
};
use layoutng_geometry::geometry::{
    box_sides::{LineLogicalBoxSides, LogicalBoxSides},
    box_strut::BoxStrut,
    logical_offset::LogicalOffset,
    logical_rect::LogicalRect,
    logical_size::LogicalSize,
};
use std::sync::Arc;
#[derive(Clone, Copy, PartialEq)]
enum SectionRepeatMode {
    NotRepeated,
    MayRepeatAgain,
    RepeatedLast,
}
// Source local lambda; the builder and context stay borrowed from the same
// algorithm run, and the Arc retains the source scoped_refptr ownership.
// cpp: layoutng_table/table_layout_algorithm.cc:1018-1068
fn CreateSectionConstraintSpace(
    parent: &BoxFragmentBuilder,
    direction: WritingDirectionMode,
    inline_size: LayoutUnit,
    data: &Arc<TableConstraintSpaceData>,
    sections: &Sections,
    section: &BlockNode,
    offset: LayoutUnit,
    index: u32,
    reserved: LayoutUnit,
    repeat: SectionRepeatMode,
) -> ConstraintSpace {
    let mut builder = ConstraintSpaceBuilder::new(parent.GetConstraintSpace(), direction, true);
    let mut available = LogicalSize::new(inline_size, kIndefiniteSize);
    if data.sections[index as usize].row_count == 0 {
        builder.SetIsFixedBlockSize(true);
        available.block_size = sections[index as usize].block_size;
    }
    builder.SetAvailableSize(available);
    builder.SetIsFixedInlineSize(true);
    builder.SetPercentageResolutionSize(LogicalSize::new(inline_size, kIndefiniteSize));
    builder.SetTableSectionData(data.clone(), index);
    if repeat != SectionRepeatMode::NotRepeated {
        builder.SetShouldRepeat(repeat == SectionRepeatMode::MayRepeatAgain);
        builder.SetIsInsideRepeatableContent(true);
        builder.SetShouldPropagateChildBreakValues(false);
    } else if parent.GetConstraintSpace().HasBlockFragmentation() {
        SetupSpaceBuilderForFragmentationFromBuilder(parent, &section.base, offset, &mut builder);
        if parent.GetConstraintSpace().HasKnownFragmentainerBlockSize() {
            builder.ReserveSpaceInFragmentainer(reserved);
        }
    }
    builder.ToConstraintSpace()
}
// cpp: layoutng_table/table_layout_algorithm.cc:980-993
fn AddCaptionResult(
    builder: &mut BoxFragmentBuilder,
    direction: WritingDirectionMode,
    caption: &TableCaptionResult,
    offset: &mut LayoutUnit,
) {
    *offset += caption.margins.block_start;
    let result = unsafe { &*caption.layout_result.Get() };
    builder.AddResult(
        result,
        LogicalOffset::new(caption.margins.inline_start, *offset),
        Some(caption.margins),
        None,
        std::ptr::null(),
    );
    *offset += LogicalFragment::new(direction, result.GetPhysicalFragment()).BlockSize()
        + caption.margins.block_end;
}
fn BlockStartBorderPadding(borders: &BoxStrut, sides: &LogicalBoxSides) -> LayoutUnit {
    if sides.block_start {
        borders.block_start
    } else {
        LayoutUnit::default()
    }
}
impl TableLayoutAlgorithm {
    // cpp: layoutng_table/table_layout_algorithm.cc:910-1743
    pub(crate) fn GenerateFragment(
        &mut self,
        inline_size: LayoutUnit,
        mut minimum_grid_size: LayoutUnit,
        grouped: &TableGroupedChildren,
        locations: &Vector<TableColumnLocation>,
        rows: &Rows,
        cells: &CellBlockConstraints,
        sections: &Sections,
        captions: &HeapVector<TableCaptionResult>,
        borders: &TableBorders,
        spacing: LogicalSize,
    ) -> *const LayoutResult {
        let node = self.Node().clone();
        let style = node.Style();
        // These point to the immutable caller-owned constraint space and heap
        // style, not to mutable algorithm fields; keep that C++ borrow contract.
        let space = unsafe { &*(self.GetConstraintSpace() as *const ConstraintSpace) };
        let token = self.GetBreakToken();
        let mut sides = LogicalBoxSides::default();
        let start_space = self.FragmentainerSpaceLeftForChildren();
        let mut previous_size = LayoutUnit::default();
        let mut previous_grid_size = LayoutUnit::default();
        let mut before_first_spacing = spacing.block_size;
        let mut monolithic = LayoutUnit::default();
        let mut past_grid = false;
        let mut incoming = std::ptr::null();
        if let Some(token) = unsafe { token.as_ref() } {
            previous_size = token.ConsumedBlockSize();
            monolithic = token.MonolithicOverflow();
            incoming = DynamicTo::<TableBreakTokenData>(token.TokenData());
            if let Some(data) = unsafe { incoming.as_ref() } {
                previous_grid_size = data.consumed_table_box_block_size;
                minimum_grid_size -= data.consumed_table_box_block_size;
                past_grid = data.is_past_table_box;
                if data.has_entered_table_box {
                    if style.BoxDecorationBreak() == EBoxDecorationBreak::kSlice || past_grid {
                        sides.block_start = false;
                    }
                    before_first_spacing = LayoutUnit::default();
                }
                if past_grid {
                    sides.block_end = false;
                }
            }
        }
        let direction = style.GetWritingDirection();
        let data = CreateConstraintSpaceData(style, locations, sections, rows, cells, spacing);
        let border_padding = *self.base.container_builder_.BorderScrollbarPadding();
        let collapsed = borders.IsCollapsed();
        let mut offset = LayoutUnit::default();
        let mut after_spacing = LayoutUnit::default();
        let mut separation = false;
        let relayout_captions =
            InvolvedInBlockFragmentationForBuilder(&self.base.container_builder_);
        if !relayout_captions {
            for caption in captions {
                if caption.node.Style().CaptionSide() == ECaptionSide::kTop {
                    AddCaptionResult(
                        &mut self.base.container_builder_,
                        direction,
                        caption,
                        &mut offset,
                    );
                }
            }
        }
        let section_inline_size =
            (inline_size - border_padding.InlineSum() - spacing.inline_size * 2i32)
                .ClampNegativeToZero();
        let section_inline_offset = border_padding.inline_start + spacing.inline_size;
        let mut extent: Option<TableBoxExtent> = None;
        let mut first_baseline = None;
        let mut last_baseline = None;
        let mut repeated_header = false;
        let mut pending_footer = false;
        let mut footer_size = LayoutUnit::default();
        if !space.IsInsideRepeatableContent()
            && !DisableLayoutSideEffectsScope::IsDisabled()
            && (grouped.header.is_non_null() || grouped.footer.is_non_null())
        {
            let maximum = space.FragmentainerBlockSize() / 4;
            let mut iterator = TableChildIterator::new(grouped, unsafe { token.as_ref() });
            loop {
                let entry = iterator.NextChild();
                let child = entry.GetNode();
                if !child.is_non_null() {
                    break;
                }
                if child != grouped.header && child != grouped.footer {
                    continue;
                }
                let child_token = entry.GetBreakToken();
                if IsBreakInside(child_token) {
                    continue;
                }
                let block_size = sections[entry.GetSectionIndex() as usize].block_size;
                if child_token.is_null() || !unsafe { &*child_token }.IsRepeated() {
                    debug_assert!(
                        child_token.is_null() || unsafe { &*child_token }.IsBreakBefore()
                    );
                    if let Some(incoming) = unsafe { incoming.as_ref() } {
                        if incoming.has_entered_table_box {
                            continue;
                        }
                    }
                    if !space.HasKnownFragmentainerBlockSize() || block_size > maximum {
                        continue;
                    }
                    if !IsAvoidBreakValue(space, child.Style().BreakInside()) {
                        continue;
                    }
                }
                if child == grouped.header {
                    repeated_header = true;
                } else {
                    debug_assert!(child == grouped.footer);
                    pending_footer = true;
                    footer_size = block_size
                        + spacing.block_size
                        + if collapsed {
                            border_padding.block_end
                        } else {
                            LayoutUnit::default()
                        };
                }
            }
        }
        let mut entered_regular = monolithic != LayoutUnit::default();
        let mut inflation = LayoutUnit::default();
        let mut header_size = LayoutUnit::default();
        let mut broke_inside = false;
        let mut ended_grid = false;
        let mut iterator = TableChildIterator::new(grouped, unsafe { token.as_ref() });
        loop {
            let entry = iterator.NextChild();
            let child = entry.GetNode();
            if !child.is_non_null() {
                break;
            }
            debug_assert!(child.IsTableCaption() || child.IsTableSection());
            let mut early_child = std::ptr::null();
            if let Some(early) = unsafe { self.early_break_.as_ref() } {
                if IsEarlyBreakTarget(early, &self.base.container_builder_, &child.base) {
                    self.base.container_builder_.AddBreakBeforeChild(
                        child.base.clone(),
                        Some(kBreakAppealPerfect),
                        false,
                        LogicalOffset::default(),
                    );
                    broke_inside = true;
                    if child == grouped.footer {
                        pending_footer = false;
                    }
                    break;
                }
                early_child = EnterEarlyBreakInChild(&child, early);
            }
            let child_token = entry.GetBreakToken();
            let result: *const LayoutResult;
            let mut before_header = None;
            let child_inline_offset;
            let mut start_margin = LayoutUnit::default();
            let mut end_margin = LayoutUnit::default();
            let mut new_extent = None;
            let mut repeated = false;
            let mut overlapping_header = false;
            if child.IsTableCaption() {
                if !relayout_captions {
                    continue;
                }
                if child.Style().CaptionSide() == ECaptionSide::kBottom && !past_grid {
                    debug_assert!(!ended_grid);
                    if extent.is_none() {
                        extent = Some(BeginTableBoxLayout(
                            offset,
                            BlockStartBorderPadding(&border_padding, &sides),
                        ));
                    }
                    offset = EndTableBoxLayout(
                        border_padding.block_end,
                        after_spacing,
                        minimum_grid_size,
                        extent.as_mut().unwrap(),
                        &mut inflation,
                    );
                    ended_grid = true;
                    past_grid = !space.HasKnownFragmentainerBlockSize()
                        || extent.as_ref().unwrap().end <= start_space;
                }
                let available =
                    LogicalSize::new(self.base.container_builder_.InlineSize(), kIndefiniteSize);
                let margins = ComputeCaptionMargins(
                    space,
                    &child,
                    self.base.container_builder_.InlineSize(),
                    child_token,
                );
                start_margin = margins.block_start;
                end_margin = margins.block_end;
                let child_space = CreateCaptionConstraintSpace(
                    space,
                    style,
                    &child,
                    available,
                    Some(offset + start_margin),
                );
                let caption = LayoutCaption(
                    space,
                    style,
                    self.base.container_builder_.InlineSize(),
                    &child_space,
                    &child,
                    margins,
                    child_token,
                    early_child,
                );
                debug_assert_eq!(
                    unsafe { &*caption.layout_result.Get() }.Status(),
                    EStatus::kSuccess
                );
                result = caption.layout_result.Get();
                child_inline_offset = caption.margins.inline_start;
                header_size = LayoutUnit::default();
            } else {
                debug_assert!(child.IsTableSection());
                let collapsible_spacing;
                if extent.is_some() {
                    collapsible_spacing = spacing.block_size;
                } else {
                    new_extent = Some(BeginTableBoxLayout(
                        offset,
                        BlockStartBorderPadding(&border_padding, &sides),
                    ));
                    offset += BlockStartBorderPadding(&border_padding, &sides);
                    collapsible_spacing = before_first_spacing;
                }
                let childless_offset = offset;
                let mut may_repeat_again = false;
                if child == grouped.header {
                    if repeated_header {
                        repeated = true;
                        may_repeat_again = !self.is_known_to_be_last_table_box_;
                        before_header = Some(offset);
                        if monolithic != LayoutUnit::default() {
                            offset = -monolithic;
                            overlapping_header = true;
                        }
                        if collapsed && !sides.block_start {
                            offset += border_padding.block_start;
                        }
                    }
                } else if child == grouped.footer {
                    if pending_footer {
                        repeated = true;
                        pending_footer = false;
                        may_repeat_again = !self.is_known_to_be_last_table_box_ && repeated_header;
                    }
                }
                offset += collapsible_spacing;
                let repeat = if repeated {
                    if may_repeat_again {
                        SectionRepeatMode::MayRepeatAgain
                    } else {
                        SectionRepeatMode::RepeatedLast
                    }
                } else {
                    SectionRepeatMode::NotRepeated
                };
                let reserved = if repeat == SectionRepeatMode::NotRepeated {
                    header_size + footer_size
                } else {
                    LayoutUnit::default()
                };
                let child_space = CreateSectionConstraintSpace(
                    &self.base.container_builder_,
                    direction,
                    section_inline_size,
                    &data,
                    sections,
                    &child,
                    offset - header_size,
                    entry.GetSectionIndex(),
                    reserved,
                    repeat,
                );
                result = if repeated {
                    child.LayoutRepeatableRoot(&child_space, child_token)
                } else {
                    child.Layout(&child_space, child_token, early_child, std::ptr::null())
                };
                child_inline_offset = section_inline_offset;
                after_spacing = spacing.block_size;
                if unsafe { &*To::<PhysicalBoxFragment>(unsafe { &*result }.GetPhysicalFragment()) }
                    .HasDescendantsForTablePart()
                {
                    if !child_token.is_null() && unsafe { &*child_token }.IsAtBlockEnd() {
                        after_spacing = LayoutUnit::default();
                    }
                } else {
                    offset = childless_offset;
                }
            }
            if space.HasBlockFragmentation() && (child_token.is_null() || !repeated) {
                let fragmentainer_offset =
                    self.FragmentainerOffsetForChildren() + start_margin + offset - header_size;
                let status = self.base.BreakBeforeChildIfNeeded(
                    child.base.clone(),
                    unsafe { &*result },
                    fragmentainer_offset,
                    separation,
                );
                if status == BreakStatus::kNeedsEarlierBreak {
                    return self
                        .base
                        .container_builder_
                        .Abort(EStatus::kNeedsEarlierBreak);
                }
                if status == BreakStatus::kBrokeBefore {
                    broke_inside = true;
                    break;
                }
                debug_assert_eq!(status, BreakStatus::kContinue);
            }
            let physical =
                unsafe { &*To::<PhysicalBoxFragment>(unsafe { &*result }.GetPhysicalFragment()) };
            let fragment = LogicalBoxFragment::new(direction, physical);
            if child.IsTableSection() {
                if !repeated {
                    entered_regular = true;
                }
                if first_baseline.is_none() {
                    if let Some(baseline) = fragment.FirstBaseline() {
                        first_baseline = Some(offset + baseline);
                    }
                }
                if let Some(baseline) = fragment.LastBaseline() {
                    last_baseline = Some(offset + baseline);
                }
            }
            offset += start_margin;
            self.base.container_builder_.AddResultAtOffset(
                unsafe { &*result },
                LogicalOffset::new(child_inline_offset, offset),
            );
            offset += fragment.BlockSize() + end_margin;
            if child.IsTableSection() {
                if let Some(before) = before_header {
                    header_size = offset - before + spacing.block_size;
                    if overlapping_header {
                        offset = offset.max(before);
                    }
                }
                if let Some(new_extent) = new_extent {
                    debug_assert!(extent.is_none());
                    extent = Some(new_extent);
                }
                extent.as_mut().unwrap().end = offset;
            } else if child.Style().CaptionSide() == ECaptionSide::kBottom {
                past_grid = true;
            }
            if space.HasBlockFragmentation() {
                if !separation {
                    separation = !repeated;
                }
                if self.base.container_builder_.HasInflowChildBreakInside() {
                    broke_inside = true;
                    break;
                }
            }
        }
        if extent.is_none() && !past_grid && !broke_inside {
            extent = Some(BeginTableBoxLayout(
                offset,
                BlockStartBorderPadding(&border_padding, &sides),
            ));
        }
        let grid_continues = extent.is_some() && !past_grid && broke_inside;
        if pending_footer && extent.is_some() {
            debug_assert!(grid_continues);
            let mut entry = iterator.NextChild();
            loop {
                let child = entry.GetNode();
                if !child.is_non_null() || child == grouped.footer {
                    break;
                }
                let token =
                    BlockBreakToken::CreateBreakBeforeAtDefaultOffset(child.base.clone(), false);
                self.base
                    .container_builder_
                    .AddBreakToken(token.cast(), false);
                entry = iterator.NextChild();
            }
            debug_assert!(entry.GetNode() == grouped.footer);
            let mut adjusted_offset = offset;
            if entered_regular {
                let footer_offset = self.FragmentainerCapacityForChildren()
                    - space.FragmentainerOffset()
                    - footer_size;
                adjusted_offset = adjusted_offset.min(footer_offset);
            }
            let footer_offset = LogicalOffset::new(section_inline_offset, adjusted_offset);
            let child_space = CreateSectionConstraintSpace(
                &self.base.container_builder_,
                direction,
                section_inline_size,
                &data,
                sections,
                &grouped.footer,
                footer_offset.block_offset,
                entry.GetSectionIndex(),
                LayoutUnit::default(),
                SectionRepeatMode::MayRepeatAgain,
            );
            let result = grouped
                .footer
                .LayoutRepeatableRoot(&child_space, entry.GetBreakToken());
            let fragmentainer_offset =
                self.FragmentainerOffsetForChildren() + footer_offset.block_offset;
            let mut break_before = false;
            if entry.GetBreakToken().is_null() || unsafe { &*entry.GetBreakToken() }.IsBreakBefore()
            {
                break_before = !MovePastBreakpointFull(
                    space,
                    grouped.footer.base.clone(),
                    unsafe { &*result },
                    fragmentainer_offset,
                    self.FragmentainerCapacityForChildren(),
                    kBreakAppealLastResort,
                    std::ptr::null_mut(),
                    false,
                    std::ptr::null_mut(),
                );
            }
            if break_before {
                BreakBeforeChild(
                    grouped.footer.base.clone(),
                    result,
                    fragmentainer_offset,
                    self.FragmentainerCapacityForChildren(),
                    Some(kBreakAppealLastResort),
                    false,
                    &mut self.base.container_builder_,
                    None,
                );
            } else {
                self.base
                    .container_builder_
                    .AddResultAtOffset(unsafe { &*result }, footer_offset);
            }
        }
        if !iterator.NextChild().is_non_null() {
            self.base.container_builder_.SetHasSeenAllChildren();
        }
        if extent.is_some() && !past_grid {
            if broke_inside {
                if space.HasKnownFragmentainerBlockSize() {
                    let extent = extent.as_mut().unwrap();
                    extent.end = extent.end.max(start_space);
                }
                after_spacing = LayoutUnit::default();
            } else if space.HasKnownFragmentainerBlockSize() {
                let new_spacing = after_spacing
                    .min(start_space - offset - border_padding.block_end)
                    .ClampNegativeToZero();
                if after_spacing != new_spacing {
                    self.base
                        .container_builder_
                        .SetIsTruncatedByFragmentationLine();
                    after_spacing = new_spacing;
                }
            }
            if !ended_grid {
                offset = EndTableBoxLayout(
                    border_padding.block_end,
                    after_spacing,
                    minimum_grid_size,
                    extent.as_mut().unwrap(),
                    &mut inflation,
                );
                ended_grid = true;
                if !broke_inside {
                    past_grid = !space.HasKnownFragmentainerBlockSize()
                        || extent.as_ref().unwrap().end <= start_space;
                }
            }
        }
        if extent.is_none() {
            sides.block_start = false;
        }
        if !past_grid
            && (!self.base.container_builder_.ShouldCloneBoxEndDecorations() || extent.is_none())
        {
            sides.block_end = false;
        }
        if !relayout_captions {
            for caption in captions {
                if caption.node.Style().CaptionSide() == ECaptionSide::kBottom {
                    AddCaptionResult(
                        &mut self.base.container_builder_,
                        direction,
                        caption,
                        &mut offset,
                    );
                }
            }
        }
        let mut size = offset.ClampNegativeToZero();
        debug_assert!(size >= inflation);
        let mut intrinsic = size - inflation;
        if !ended_grid && !past_grid {
            let fluff = border_padding.block_end + after_spacing;
            intrinsic += fluff;
            size += fluff;
        }
        self.base
            .container_builder_
            .SetIntrinsicBlockSize(intrinsic);
        size += previous_size;
        if space.IsFixedBlockSize() {
            size = space.AvailableSize().block_size;
            if space.MinBlockSizeShouldEncompassIntrinsicSize() {
                size = size.max(previous_size + intrinsic);
            }
        }
        self.base
            .container_builder_
            .SetFragmentsTotalBlockSize(size);
        let dom = node.GetDOMNode();
        if !dom.is_null() && unsafe { &*dom }.InputKind() == NodeKind::kMathTable {
            let algorithms = LayoutPassScope::Algorithms();
            let callback = unsafe { algorithms.as_ref() }
                .and_then(|a| a.mathml_support.table_baseline)
                .unwrap_or_else(|| {
                    std::panic::panic_any(UnsupportedLayout::new(
                        "MathML table layout requires the MathML package",
                    ))
                });
            self.base
                .container_builder_
                .SetBaselines(callback(style, offset));
        } else {
            if let Some(baseline) = first_baseline {
                self.base.container_builder_.SetFirstBaseline(baseline);
            }
            if let Some(baseline) = last_baseline {
                self.base.container_builder_.SetLastBaseline(baseline);
            }
        }
        self.base.container_builder_.SetIsTablePart();
        if InvolvedInBlockFragmentationForBuilder(&self.base.container_builder_) {
            let status = FinishFragmentation(&mut self.base.container_builder_);
            if status == BreakStatus::kNeedsEarlierBreak {
                return self
                    .base
                    .container_builder_
                    .Abort(EStatus::kNeedsEarlierBreak);
            }
            debug_assert_eq!(status, BreakStatus::kContinue);
            self.base
                .container_builder_
                .SetSidesToInclude(LineLogicalBoxSides::from_logical(sides, space.Direction()));
            if let Some(extent) = extent.as_mut() {
                extent.end = extent
                    .end
                    .min(self.base.container_builder_.FragmentBlockSize());
            }
        }
        let mut column_size = kIndefiniteSize;
        let mut grid = LogicalRect::default();
        let mut grid_size = LayoutUnit::default();
        if let Some(extent) = extent.as_ref() {
            grid_size = extent.end - extent.start;
            if !grid_continues {
                column_size = previous_grid_size + grid_size;
                column_size -= spacing.block_size * 2 + border_padding.BlockSum();
            }
            grid = LogicalRect::new(
                LogicalOffset::new(LayoutUnit::default(), extent.start),
                LogicalSize::new(self.base.container_builder_.InlineSize(), grid_size),
            );
        }
        let mut entered_grid = false;
        if InvolvedInBlockFragmentationForBuilder(&self.base.container_builder_) {
            let mut consumed = previous_grid_size;
            if let Some(incoming) = unsafe { incoming.as_ref() } {
                entered_grid = incoming.has_entered_table_box;
            }
            consumed += grid_size;
            entered_grid |= extent.is_some();
            let token = MakeGarbageCollected(TableBreakTokenData::new(
                rows,
                cells,
                sections,
                self.total_table_min_block_size_,
                consumed,
                entered_grid,
                past_grid,
            ));
            self.base.container_builder_.SetBreakTokenData(token.cast());
        }
        self.ComputeTableSpecificFragmentData(grouped, locations, rows, borders, grid, column_size);
        self.base
            .container_builder_
            .HandleOofsAndSpecialDescendants();
        if (repeated_header || self.base.container_builder_.ShouldCloneBoxEndDecorations())
            && entered_grid
            && !grid_continues
            && !self.is_known_to_be_last_table_box_
        {
            return self
                .base
                .container_builder_
                .Abort(EStatus::kNeedsRelayoutAsLastTableBox);
        }
        self.base.container_builder_.ToBoxFragment()
    }
}
