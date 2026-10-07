#![allow(non_snake_case)]

use foundation::{kNotFound, LayoutUnit, MakeGarbageCollected, Persistent, WtfSizeT};
use layoutng_assembly::box_fragment_builder::BoxFragmentBuilder;
use layoutng_assembly::internal::gap::cross_gap::EdgeIntersectionState;
use layoutng_assembly::internal::gap::gap_geometry::{
    ContainerType, FlexGapPlacementReversal, GapGeometry,
};
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_style::style::grid_enums::GridTrackSizingDirection;
use std::ops::{Deref, DerefMut};

use crate::flex_break_token_data::{FlexGapBreakTokenData, FlexRowGapBreakTokenData};
use crate::flex_line::{FlexLine, FlexLineVector};

// A persistent root matches the C++ stack object's GC pointer lifetime until
// the physical fragment takes its own traced reference to the geometry.
pub struct RootedGapGeometry(Persistent<GapGeometry>);

impl RootedGapGeometry {
    fn new() -> Self {
        Self(Persistent::from_ptr(MakeGarbageCollected(
            GapGeometry::new(ContainerType::kFlex),
        )))
    }
    fn Get(&self) -> *const GapGeometry {
        self.0.Get()
    }
}
impl Deref for RootedGapGeometry {
    type Target = GapGeometry;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.Get() }
    }
}
impl DerefMut for RootedGapGeometry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.Get() }
    }
}

// cpp: layoutng_flex/flex_gap_accumulator.h:132-309
pub struct FlexGapAccumulator {
    gap_between_items_: LayoutUnit,
    effective_gap_between_lines_: LayoutUnit,
    is_column_: bool,
    gap_geometry_: RootedGapGeometry,
    border_scrollbar_padding_block_start_: LayoutUnit,
    border_scrollbar_padding_inline_start_: LayoutUnit,
    content_cross_start_: LayoutUnit,
    content_cross_end_: LayoutUnit,
    content_main_start_: LayoutUnit,
    content_main_end_: LayoutUnit,
    first_row_flex_line_index_: WtfSizeT,
    row_gap_break_token_data_: Vec<FlexRowGapBreakTokenData>,
}

impl FlexGapAccumulator {
    // cpp: layoutng_flex/flex_gap_accumulator.cc:15-54
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        gap_between_items: LayoutUnit,
        effective_gap_between_lines: LayoutUnit,
        num_lines: WtfSizeT,
        num_flex_items: WtfSizeT,
        is_column: bool,
        border_scrollbar_padding_block_start: LayoutUnit,
        border_scrollbar_padding_inline_start: LayoutUnit,
        placement_reversal: Option<FlexGapPlacementReversal>,
    ) -> Self {
        let mut gap_geometry = RootedGapGeometry::new();
        gap_geometry.ReserveCrossGaps(num_flex_items);
        if num_lines > 0 {
            gap_geometry.ReserveMainGaps(num_lines - 1);
        }
        gap_geometry.ResizeFlexCrossGapSizes(num_lines);
        if let Some(reversal) = placement_reversal {
            gap_geometry.SetFlexGapPlacementReversal(reversal);
        }
        let row_gap_break_token_data = if is_column {
            vec![
                FlexRowGapBreakTokenData {
                    first_row_gap_index: 0,
                    row_gap_count: 0
                };
                num_lines as usize
            ]
        } else {
            Vec::new()
        };
        Self {
            gap_between_items_: gap_between_items,
            effective_gap_between_lines_: effective_gap_between_lines,
            is_column_: is_column,
            gap_geometry_: gap_geometry,
            border_scrollbar_padding_block_start_: border_scrollbar_padding_block_start,
            border_scrollbar_padding_inline_start_: border_scrollbar_padding_inline_start,
            content_cross_start_: LayoutUnit::Max(),
            content_cross_end_: LayoutUnit::default(),
            content_main_start_: LayoutUnit::Max(),
            content_main_end_: LayoutUnit::default(),
            first_row_flex_line_index_: kNotFound,
            row_gap_break_token_data_: row_gap_break_token_data,
        }
    }

    // cpp: layoutng_flex/flex_gap_accumulator.cc:55-92
    pub fn BuildGapGeometry(
        &mut self,
        container_builder: &BoxFragmentBuilder,
    ) -> *const GapGeometry {
        if self.gap_geometry_.MainGapCount() == 0 && self.gap_geometry_.CrossGapCount() == 0 {
            return std::ptr::null();
        }
        if self.is_column_ {
            self.FinalizeContentMainEndForColumnFlex(container_builder);
            self.gap_geometry_
                .SetInlineGapSize(self.effective_gap_between_lines_);
            self.gap_geometry_.SetBlockGapSize(self.gap_between_items_);
            self.gap_geometry_
                .SetMainDirection(GridTrackSizingDirection::kForColumns);
        } else {
            self.gap_geometry_
                .SetBlockGapSize(self.effective_gap_between_lines_);
            self.gap_geometry_.SetInlineGapSize(self.gap_between_items_);
        }
        let (inline_start, inline_end, block_start, block_end) = if self.is_column_ {
            (
                self.content_cross_start_,
                self.content_cross_end_,
                self.content_main_start_,
                self.content_main_end_,
            )
        } else {
            (
                self.content_main_start_,
                self.content_main_end_,
                self.content_cross_start_,
                self.content_cross_end_,
            )
        };
        self.gap_geometry_
            .SetContentInlineOffsets(inline_start, inline_end);
        self.gap_geometry_
            .SetContentBlockOffsets(block_start, block_end);
        self.gap_geometry_.Get()
    }

    // cpp: layoutng_flex/flex_gap_accumulator.cc:94-112
    pub fn FinalizeRowGapBreakTokenData(&mut self) -> Vec<FlexRowGapBreakTokenData> {
        if self.is_column_ {
            return std::mem::take(&mut self.row_gap_break_token_data_);
        }
        if self
            .row_gap_break_token_data_
            .first()
            .is_some_and(|row| row.row_gap_count == 0)
        {
            self.row_gap_break_token_data_.clear();
        }
        std::mem::take(&mut self.row_gap_break_token_data_)
    }

    // cpp: layoutng_flex/flex_gap_accumulator.cc:114-130
    pub fn InitializeFragmentedColumnGapGeometry(&mut self, flex_lines: &FlexLineVector) {
        assert!(self.is_column_ && !flex_lines.is_empty());
        assert_eq!(self.gap_geometry_.MainGapCount(), 0);
        self.content_cross_start_ = flex_lines[0].cross_axis_offset;
        let last_line = flex_lines.last().unwrap();
        self.content_cross_end_ = last_line.cross_axis_offset + last_line.line_cross_size;
        self.content_main_start_ = self.border_scrollbar_padding_block_start_;
        for line in &flex_lines[..flex_lines.len() - 1] {
            self.PopulateMainGapForFirstItem(line.cross_axis_offset + line.line_cross_size);
        }
    }

    // cpp: layoutng_flex/flex_gap_accumulator.cc:132-233
    #[allow(clippy::too_many_arguments)]
    pub fn BuildGapsForCurrentItem(
        &mut self,
        flex_lines: &FlexLineVector,
        global_line_index: WtfSizeT,
        item_offset: LogicalOffset,
        is_first_item: bool,
        is_last_item: bool,
        is_last_line: bool,
        line_cross_start: LayoutUnit,
        line_cross_end: LayoutUnit,
        container_main_end: LayoutUnit,
        in_fragmentation: bool,
    ) {
        let flex_line = &flex_lines[global_line_index as usize];
        let is_fragmented_column = self.is_column_ && in_fragmentation;
        let mut fragment_relative_line_index = global_line_index;
        if !self.is_column_ {
            if self.first_row_flex_line_index_ == kNotFound {
                self.first_row_flex_line_index_ = global_line_index;
            }
            fragment_relative_line_index -= self.first_row_flex_line_index_;
        }
        let main_gap_count = self.gap_geometry_.MainGapCount();
        let need_to_add_main_gap = !is_fragmented_column
            && (main_gap_count == 0 || main_gap_count - 1 < fragment_relative_line_index)
            && !is_last_line;
        let is_first_line = fragment_relative_line_index == 0;
        let single_line = is_first_line && is_last_line;
        if single_line && is_first_item {
            assert!(!need_to_add_main_gap);
            self.SetContentStartOffsetsIfNeeded(item_offset, line_cross_start);
        }
        if !is_fragmented_column && is_last_line && is_first_item {
            self.content_cross_end_ = line_cross_end;
        }
        if need_to_add_main_gap {
            self.SetContentStartOffsetsIfNeeded(item_offset, line_cross_start);
            self.PopulateMainGapForFirstItem(line_cross_end);
            if !self.is_column_ {
                if self.row_gap_break_token_data_.is_empty() {
                    self.row_gap_break_token_data_
                        .push(FlexRowGapBreakTokenData {
                            first_row_gap_index: global_line_index,
                            row_gap_count: 1,
                        });
                } else {
                    self.IncrementRowGapCount(0);
                }
            }
            if is_last_item {
                self.content_main_end_ = container_main_end;
            }
        }
        if in_fragmentation || !is_first_item || is_last_item {
            self.gap_geometry_.SetFlexCrossGapSize(
                fragment_relative_line_index,
                flex_line.effective_gap_between_items,
            );
        }
        if is_first_item {
            return;
        }
        let main_offset = if self.is_column_ {
            item_offset.block_offset
        } else {
            item_offset.inline_offset
        };
        let main_intersection_offset = main_offset - flex_line.effective_gap_between_items / 2;
        self.PopulateCrossGapForCurrentItem(
            flex_line,
            global_line_index,
            fragment_relative_line_index,
            is_first_line,
            is_last_line,
            single_line,
            main_intersection_offset,
            line_cross_start,
        );
        if is_last_item {
            let last_offset = self
                .gap_geometry_
                .GetCrossGaps()
                .last()
                .unwrap()
                .GetGapOffset();
            let last_gap_offset = if self.is_column_ {
                last_offset.block_offset
            } else {
                last_offset.inline_offset
            };
            self.content_main_end_ = std::cmp::max(last_gap_offset, container_main_end);
        }
    }

    // cpp: layoutng_flex/flex_gap_accumulator.cc:235-238
    pub fn PopulateMainGapForFirstItem(&mut self, cross_end: LayoutUnit) {
        self.gap_geometry_
            .AddMainGap(cross_end + self.effective_gap_between_lines_ / 2);
    }

    // cpp: layoutng_flex/flex_gap_accumulator.cc:240-259
    pub fn HandleCrossGapRangesForCurrentItem(
        &mut self,
        line_index: WtfSizeT,
        cross_gap_index: WtfSizeT,
    ) {
        let main_gap_count = self.gap_geometry_.MainGapCount();
        if main_gap_count == 0 {
            return;
        }
        if line_index < main_gap_count {
            self.gap_geometry_
                .MainGapAtMut(line_index)
                .IncrementRangeOfCrossGapsBefore(cross_gap_index);
        }
        if line_index > 0 && line_index - 1 < main_gap_count {
            self.gap_geometry_
                .MainGapAtMut(line_index - 1)
                .IncrementRangeOfCrossGapsAfter(cross_gap_index);
        }
    }

    // cpp: layoutng_flex/flex_gap_accumulator.cc:261-317
    #[allow(clippy::too_many_arguments)]
    pub fn PopulateCrossGapForCurrentItem(
        &mut self,
        _flex_line: &FlexLine,
        global_line_index: WtfSizeT,
        fragment_relative_line_index: WtfSizeT,
        is_first_line: bool,
        is_last_line: bool,
        single_line: bool,
        main_intersection_offset: LayoutUnit,
        cross_start: LayoutUnit,
    ) {
        let mut cross_intersection_offset = cross_start;
        let edge_state = if single_line {
            EdgeIntersectionState::kBoth
        } else if is_first_line {
            EdgeIntersectionState::kStart
        } else if is_last_line {
            cross_intersection_offset -= self.effective_gap_between_lines_ / 2;
            EdgeIntersectionState::kEnd
        } else {
            cross_intersection_offset = cross_start - self.effective_gap_between_lines_ / 2;
            EdgeIntersectionState::kNone
        };
        let offset = if self.is_column_ {
            LogicalOffset::new(cross_intersection_offset, main_intersection_offset)
        } else {
            LogicalOffset::new(main_intersection_offset, cross_intersection_offset)
        };
        self.gap_geometry_
            .AddCrossGapWithEdgeState(offset, edge_state);
        if self.is_column_ {
            self.IncrementRowGapCount(global_line_index);
        }
        self.HandleCrossGapRangesForCurrentItem(
            fragment_relative_line_index,
            self.gap_geometry_.CrossGapCount() - 1,
        );
    }

    // cpp: layoutng_flex/flex_gap_accumulator.h:245-252
    pub fn SetContentMainEnd(&mut self, end: LayoutUnit) {
        self.content_main_end_ = end;
    }
    pub fn SetEffectiveGapBetweenLines(&mut self, gap: LayoutUnit) {
        self.effective_gap_between_lines_ = gap;
    }

    // cpp: layoutng_flex/flex_gap_accumulator.cc:319-322
    pub fn IncrementRowGapCount(&mut self, index: WtfSizeT) {
        self.row_gap_break_token_data_[index as usize].row_gap_count += 1;
    }

    // cpp: layoutng_flex/flex_gap_accumulator.cc:324-333
    pub fn DecrementRowGapCount(&mut self) {
        assert!(!self.is_column_);
        if self.gap_geometry_.MainGapCount() == 0 || self.row_gap_break_token_data_.is_empty() {
            return;
        }
        assert_eq!(self.row_gap_break_token_data_.len(), 1);
        assert!(self.row_gap_break_token_data_[0].row_gap_count > 0);
        self.row_gap_break_token_data_[0].row_gap_count -= 1;
    }

    // cpp: layoutng_flex/flex_gap_accumulator.cc:335-376
    pub fn CalculateColumnFlexLineRowGapStart(
        &mut self,
        flex_lines: &FlexLineVector,
        global_line_index: WtfSizeT,
        previous_gap_data: Option<&FlexGapBreakTokenData>,
    ) {
        assert!(self.is_column_);
        let index = global_line_index as usize;
        assert!(index < self.row_gap_break_token_data_.len());
        if index == 0 && previous_gap_data.is_none() {
            assert_eq!(self.row_gap_break_token_data_[index].first_row_gap_index, 0);
            return;
        }
        let (previous_start, previous_gap_count) = if let Some(previous) = previous_gap_data {
            let row = &previous.gap_data_for_rows[index];
            (row.first_row_gap_index, row.row_gap_count)
        } else {
            let item_count = flex_lines[index - 1].item_indices.len() as WtfSizeT;
            (
                self.row_gap_break_token_data_[index - 1].first_row_gap_index,
                item_count.saturating_sub(1),
            )
        };
        self.row_gap_break_token_data_[index].first_row_gap_index =
            previous_start + previous_gap_count;
    }

    // cpp: layoutng_flex/flex_gap_accumulator.cc:378-388
    fn FinalizeContentMainEndForColumnFlex(&mut self, builder: &BoxFragmentBuilder) {
        assert!(self.is_column_);
        let end = builder.ApplicableBorders().block_end
            + builder.ApplicableScrollbar().block_end
            + builder.ApplicablePadding().block_end;
        self.SetContentMainEnd(builder.FragmentBlockSize() - end);
    }

    // cpp: layoutng_flex/flex_gap_accumulator.cc:390-426
    pub fn SuppressLastMainGap(&mut self, new_cross_end: Option<LayoutUnit>) {
        if self.gap_geometry_.MainGapCount() == 0 {
            return;
        }
        let last = self.gap_geometry_.GetMainGaps().last().unwrap();
        let start = if last.HasCrossGapsBefore() {
            Some(last.GetCrossGapBeforeStart())
        } else {
            None
        };
        let end = if last.HasCrossGapsBefore() {
            Some(last.GetCrossGapBeforeEnd())
        } else {
            None
        };
        self.content_cross_end_ =
            new_cross_end.unwrap_or(last.GetGapOffset() - self.effective_gap_between_lines_ / 2);
        self.gap_geometry_.RemoveLastMainGap();
        if let (Some(start), Some(end)) = (start, end) {
            for index in start..=end {
                let gap = self.gap_geometry_.CrossGapAt(index);
                let state = gap.GetEdgeIntersectionState();
                if state == EdgeIntersectionState::kStart {
                    gap.SetEdgeIntersectionState(EdgeIntersectionState::kBoth);
                } else if state == EdgeIntersectionState::kNone {
                    gap.SetEdgeIntersectionState(EdgeIntersectionState::kEnd);
                }
            }
        }
    }

    // cpp: layoutng_flex/flex_gap_accumulator.cc:428-442
    fn SetContentStartOffsetsIfNeeded(
        &mut self,
        offset: LogicalOffset,
        line_cross_start: LayoutUnit,
    ) {
        if self.content_main_start_ != LayoutUnit::Max()
            && self.content_cross_start_ != LayoutUnit::Max()
        {
            return;
        }
        self.content_cross_start_ = line_cross_start;
        self.content_main_start_ = if self.is_column_ {
            self.border_scrollbar_padding_block_start_
        } else {
            self.border_scrollbar_padding_inline_start_
        };
        let main_offset = if self.is_column_ {
            offset.block_offset
        } else {
            offset.inline_offset
        };
        self.content_main_start_ = std::cmp::min(self.content_main_start_, main_offset);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(value: i32) -> LayoutUnit {
        LayoutUnit::from_signed(value)
    }

    fn line(items: Vec<WtfSizeT>, cross_start: i32) -> FlexLine {
        let mut result = FlexLine::new(items, unit(0), unit(20), unit(0), unit(0), 0);
        result.cross_axis_offset = unit(cross_start);
        result.effective_gap_between_items = unit(10);
        result
    }

    #[test]
    fn row_gap_geometry_and_break_count_follow_placed_items() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut lines = FlexLineVector::default();
        lines.push(line(vec![0, 1], 0));
        lines.push(line(vec![2], 30));
        let mut accumulator =
            FlexGapAccumulator::new(unit(10), unit(10), 2, 3, false, unit(0), unit(0), None);
        accumulator.BuildGapsForCurrentItem(
            &lines,
            0,
            LogicalOffset::new(unit(0), unit(0)),
            true,
            false,
            false,
            unit(0),
            unit(20),
            unit(100),
            false,
        );
        accumulator.BuildGapsForCurrentItem(
            &lines,
            0,
            LogicalOffset::new(unit(20), unit(0)),
            false,
            true,
            false,
            unit(0),
            unit(20),
            unit(100),
            false,
        );
        accumulator.BuildGapsForCurrentItem(
            &lines,
            1,
            LogicalOffset::new(unit(0), unit(30)),
            true,
            true,
            true,
            unit(30),
            unit(50),
            unit(100),
            false,
        );
        assert_eq!(accumulator.gap_geometry_.MainGapCount(), 1);
        assert_eq!(accumulator.gap_geometry_.CrossGapCount(), 1);
        assert_eq!(
            accumulator.gap_geometry_.MainGapAt(0).GetGapOffset(),
            unit(25)
        );
        let row_data = accumulator.FinalizeRowGapBreakTokenData();
        assert_eq!(row_data.len(), 1);
        assert_eq!(row_data[0].row_gap_count, 1);
    }

    #[test]
    fn fragmented_column_preserves_global_line_gap_slots() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut lines = FlexLineVector::default();
        lines.push(line(vec![0, 1, 2], 0));
        lines.push(line(vec![3, 4], 30));
        let mut accumulator =
            FlexGapAccumulator::new(unit(10), unit(10), 2, 5, true, unit(0), unit(0), None);
        accumulator.InitializeFragmentedColumnGapGeometry(&lines);
        accumulator.CalculateColumnFlexLineRowGapStart(&lines, 0, None);
        accumulator.CalculateColumnFlexLineRowGapStart(&lines, 1, None);
        assert_eq!(accumulator.gap_geometry_.MainGapCount(), 1);
        let row_data = accumulator.FinalizeRowGapBreakTokenData();
        assert_eq!(row_data.len(), 2);
        assert_eq!(row_data[1].first_row_gap_index, 2);
    }
}
