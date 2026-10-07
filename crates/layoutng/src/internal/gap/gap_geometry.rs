#![allow(non_snake_case, non_upper_case_globals)]

use std::cell::RefCell;

use foundation::{
    HashSet, LayoutUnit, PhysicalRect, PhysicalSize, RuleVisibilityItems, String, StringBuilder,
    ValueForLength, Vector, Visitor, WritingDirectionMode,
};
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::grid_enums::GridTrackSizingDirection;

use super::cross_gap::{CrossGap, EdgeIntersectionState};
use super::gap_decoration_utils::CSSGapDecorationUtils;
use super::gap_intersection::{GapIntersection, OverlapWindowState};
use super::gap_utils::{GapSegmentState, GapSegmentStateCursor, GapSegmentStateRanges};
use super::main_gap::{MainGap, SpannerMainGapType};

// cpp: layoutng/internal/gap/gap_geometry.h:36-41
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockStatusId {
    kNone = 0,
    kBlockedBefore = 1,
    kBlockedAfter = 2,
}

// cpp: layoutng/internal/gap/gap_geometry.h:36-59
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockedStatus {
    status_: u32,
}

impl BlockedStatus {
    pub const kNone: BlockStatusId = BlockStatusId::kNone;
    pub const kBlockedBefore: BlockStatusId = BlockStatusId::kBlockedBefore;
    pub const kBlockedAfter: BlockStatusId = BlockStatusId::kBlockedAfter;

    // cpp: layoutng/internal/gap/gap_geometry.h:44-47
    pub fn HasBlockedStatus(&self, status: BlockStatusId) -> bool {
        self.status_ & status as u32 != 0
    }

    pub fn SetBlockedStatus(&mut self, status: BlockStatusId) {
        self.status_ |= status as u32;
    }
}

impl std::ops::BitAnd<BlockStatusId> for BlockedStatus {
    type Output = bool;

    // cpp: layoutng/internal/gap/gap_geometry.h:49-51
    fn bitand(self, status: BlockStatusId) -> bool {
        self.HasBlockedStatus(status)
    }
}

impl std::ops::BitOrAssign<BlockStatusId> for BlockedStatus {
    // cpp: layoutng/internal/gap/gap_geometry.h:52-55
    fn bitor_assign(&mut self, status: BlockStatusId) {
        self.SetBlockedStatus(status);
    }
}

// cpp: layoutng/internal/gap/gap_geometry.h:61-62
pub type MainGaps = Vector<MainGap>;
pub type CrossGaps = Vector<CrossGap>;

// cpp: layoutng/internal/gap/gap_geometry.h:76-80
#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub end_offset: LayoutUnit,
    pub before_occupant_index: u32,
    pub after_occupant_index: u32,
}

// cpp: layoutng/internal/gap/gap_geometry.h:90-117
#[derive(Clone, Copy, Debug, Default)]
struct CrossGapRunCursor {
    start_: u32,
    current_: u32,
    end_: u32,
}

impl CrossGapRunCursor {
    fn new(start: u32, inclusive_end: u32, cross_gap_count: u32) -> Self {
        assert!(start <= inclusive_end);
        assert!(inclusive_end < cross_gap_count);
        Self {
            start_: start,
            current_: start,
            end_: inclusive_end.wrapping_add(1),
        }
    }

    fn AtEnd(&self) -> bool {
        self.current_ == self.end_
    }

    fn Size(&self) -> u32 {
        self.end_ - self.start_
    }

    fn CrossGapIndex(&self) -> u32 {
        assert!(!self.AtEnd());
        self.current_
    }

    fn ConsumedCount(&self) -> u32 {
        self.current_ - self.start_
    }

    fn Advance(&mut self) {
        assert!(!self.AtEnd());
        self.current_ = self.current_.wrapping_add(1);
    }
}

// cpp: layoutng/internal/gap/gap_geometry.h:72-133
pub struct GridLanesMainGapSegmentWalker<'a> {
    gap_geometry_: &'a GapGeometry,
    cross_direction_: GridTrackSizingDirection,
    content_start_: LayoutUnit,
    content_end_: LayoutUnit,
    before_: CrossGapRunCursor,
    after_: CrossGapRunCursor,
    intersection_capacity_: u32,
    finished_: bool,
}

impl<'a> GridLanesMainGapSegmentWalker<'a> {
    // cpp: layoutng/internal/gap/gap_geometry.h:82-83
    // cpp: layoutng/internal/gap/gap_geometry.cc:20-55
    pub fn new(gap_geometry: &'a GapGeometry, main_gap_index: u32) -> Self {
        let cross_direction =
            if gap_geometry.GetMainDirection() == GridTrackSizingDirection::kForColumns {
                GridTrackSizingDirection::kForRows
            } else {
                GridTrackSizingDirection::kForColumns
            };
        let (content_start, content_end) = if cross_direction == GridTrackSizingDirection::kForRows
        {
            (
                gap_geometry.GetContentBlockStart(),
                gap_geometry.GetContentBlockEnd(),
            )
        } else {
            (
                gap_geometry.GetContentInlineStart(),
                gap_geometry.GetContentInlineEnd(),
            )
        };
        assert_eq!(gap_geometry.GetContainerType(), ContainerType::kGridLanes);
        assert!(main_gap_index < gap_geometry.MainGapCount());
        let main_gap = gap_geometry.MainGapAt(main_gap_index);
        let before = if main_gap.HasCrossGapsBefore() {
            CrossGapRunCursor::new(
                main_gap.GetCrossGapBeforeStart(),
                main_gap.GetCrossGapBeforeEnd(),
                gap_geometry.CrossGapCount(),
            )
        } else {
            CrossGapRunCursor::default()
        };
        let after = if main_gap.HasCrossGapsAfter() {
            CrossGapRunCursor::new(
                main_gap.GetCrossGapAfterStart(),
                main_gap.GetCrossGapAfterEnd(),
                gap_geometry.CrossGapCount(),
            )
        } else {
            CrossGapRunCursor::default()
        };
        let mut walker = Self {
            gap_geometry_: gap_geometry,
            cross_direction_: cross_direction,
            content_start_: content_start,
            content_end_: content_end,
            intersection_capacity_: 2u32.wrapping_add(before.Size()).wrapping_add(after.Size()),
            before_: before,
            after_: after,
            finished_: false,
        };
        walker.SkipGapsAtOrBeforeContentStart();
        walker
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:86-86
    pub fn IntersectionCapacity(&self) -> u32 {
        self.intersection_capacity_
    }

    // cpp: layoutng/internal/gap/gap_geometry.cc:57-61
    fn CrossGapOffset(&self, index: u32) -> LayoutUnit {
        assert!(index < self.gap_geometry_.CrossGapCount());
        self.gap_geometry_.GetCrossGaps()[index as usize]
            .GetGapOffsetForDirection(self.cross_direction_)
    }

    // cpp: layoutng/internal/gap/gap_geometry.cc:63-66
    fn SkipGapsAtOrBeforeContentStart(&mut self) {
        Self::SkipRunAtOrBeforeContentStart(
            &mut self.before_,
            self.gap_geometry_,
            self.cross_direction_,
            self.content_start_,
        );
        Self::SkipRunAtOrBeforeContentStart(
            &mut self.after_,
            self.gap_geometry_,
            self.cross_direction_,
            self.content_start_,
        );
    }

    // C++ stores the run and geometry in `this`; Rust passes them separately
    // so the run can be mutably borrowed without aliasing the walker.
    // cpp: layoutng/internal/gap/gap_geometry.cc:68-74
    fn SkipRunAtOrBeforeContentStart(
        run: &mut CrossGapRunCursor,
        geometry: &GapGeometry,
        direction: GridTrackSizingDirection,
        content_start: LayoutUnit,
    ) {
        while !run.AtEnd()
            && geometry.GetCrossGaps()[run.CrossGapIndex() as usize]
                .GetGapOffsetForDirection(direction)
                <= content_start
        {
            run.Advance();
        }
    }

    // cpp: layoutng/internal/gap/gap_geometry.cc:76-82
    fn ConsumeRunAtOffset(
        run: &mut CrossGapRunCursor,
        geometry: &GapGeometry,
        direction: GridTrackSizingDirection,
        offset: LayoutUnit,
    ) {
        while !run.AtEnd()
            && geometry.GetCrossGaps()[run.CrossGapIndex() as usize]
                .GetGapOffsetForDirection(direction)
                == offset
        {
            run.Advance();
        }
        assert!(
            run.AtEnd()
                || geometry.GetCrossGaps()[run.CrossGapIndex() as usize]
                    .GetGapOffsetForDirection(direction)
                    > offset
        );
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:85-85
    // cpp: layoutng/internal/gap/gap_geometry.cc:84-117
    pub fn Next(&mut self) -> Option<Segment> {
        if self.finished_ {
            return None;
        }
        let mut segment = Segment {
            end_offset: self.content_end_,
            before_occupant_index: self.before_.ConsumedCount(),
            after_occupant_index: self.after_.ConsumedCount(),
        };
        if self.before_.AtEnd() && self.after_.AtEnd() {
            self.finished_ = true;
            return Some(segment);
        }
        let offset = if self.before_.AtEnd() {
            self.CrossGapOffset(self.after_.CrossGapIndex())
        } else if self.after_.AtEnd() {
            self.CrossGapOffset(self.before_.CrossGapIndex())
        } else {
            let before_offset = self.CrossGapOffset(self.before_.CrossGapIndex());
            let after_offset = self.CrossGapOffset(self.after_.CrossGapIndex());
            if before_offset < after_offset {
                before_offset
            } else {
                after_offset
            }
        };
        if offset >= self.content_end_ {
            self.finished_ = true;
            return Some(segment);
        }
        Self::ConsumeRunAtOffset(
            &mut self.before_,
            self.gap_geometry_,
            self.cross_direction_,
            offset,
        );
        Self::ConsumeRunAtOffset(
            &mut self.after_,
            self.gap_geometry_,
            self.cross_direction_,
            offset,
        );
        segment.end_offset = offset;
        Some(segment)
    }
}

// The C++ closure captures the intersection vector and both line sizes. A
// function keeps the same mutation order while making each borrow explicit.
// cpp: layoutng/internal/gap/gap_geometry.cc:510-567
fn ProcessCrossGapIntersection(
    intersections: &mut Vector<GapIntersection>,
    intersection_offset: LayoutUnit,
    is_above_main_gap: bool,
    cross_gap_size_above: Option<LayoutUnit>,
    cross_gap_size_below: Option<LayoutUnit>,
) {
    assert!(!intersections.is_empty());
    let mut overlaps_with_intersection = false;
    if intersections.len() > 1 {
        let current_cross_gap_size = if is_above_main_gap {
            cross_gap_size_above.expect("missing gap size above")
        } else {
            cross_gap_size_below.expect("missing gap size below")
        };
        let previous = intersections.last().unwrap();
        let prev_cross_gap_size = if previous.IsAboveMainGap() {
            cross_gap_size_above.expect("missing gap size above")
        } else {
            cross_gap_size_below.expect("missing gap size below")
        };
        overlaps_with_intersection = intersection_offset - previous.GetOffset()
            < (prev_cross_gap_size + current_cross_gap_size) / 2;
    }
    if overlaps_with_intersection {
        if intersections.last().unwrap().IsOverlapWindowOpen() {
            intersections.push(GapIntersection::with_overlap_state(
                intersection_offset,
                OverlapWindowState::kWindowClose,
                is_above_main_gap,
            ));
        } else {
            assert!(intersections.last().unwrap().IsOverlapWindowClose());
            let previous = intersections.last_mut().unwrap();
            previous.SetOffset(intersection_offset);
            previous.SetOverlapState(OverlapWindowState::kWindowClose);
            previous.SetIsAboveMainGap(is_above_main_gap);
        }
    } else {
        if intersections.last().unwrap().IsOverlapWindowOpen() {
            intersections.last_mut().unwrap().ResetOverlapState();
        }
        intersections.push(GapIntersection::with_overlap_state(
            intersection_offset,
            OverlapWindowState::kWindowOpen,
            is_above_main_gap,
        ));
    }
}

// cpp: layoutng/internal/gap/gap_geometry.h:141-146
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerType {
    kGrid,
    kGridLanes,
    kFlex,
    kMultiColumn,
}

// cpp: layoutng/internal/gap/gap_geometry.h:149-160
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlexGapPlacementReversal {
    pub reverse_lines: bool,
    pub reverse_items_in_line: bool,
}

impl FlexGapPlacementReversal {
    pub fn new(reverse_lines: bool, reverse_items_in_line: bool) -> Self {
        assert!(reverse_lines || reverse_items_in_line);
        Self {
            reverse_lines,
            reverse_items_in_line,
        }
    }
}

// cpp: layoutng/internal/gap/gap_geometry.h:204-215
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GapDecorationInkOutsets {
    pub inline_start: LayoutUnit,
    pub inline_end: LayoutUnit,
    pub block_start: LayoutUnit,
    pub block_end: LayoutUnit,
}

impl GapDecorationInkOutsets {
    pub fn InlineOutsetThickness(&self) -> LayoutUnit {
        self.inline_start + self.inline_end
    }

    pub fn BlockOutsetThickness(&self) -> LayoutUnit {
        self.block_start + self.block_end
    }
}

// cpp: layoutng/internal/gap/gap_geometry.h:139-140
// cpp: layoutng/internal/gap/gap_geometry.h:678-733
pub struct GapGeometry {
    inline_gap_size_: LayoutUnit,
    block_gap_size_: LayoutUnit,
    container_type_: ContainerType,
    main_gaps_: MainGaps,
    cross_gaps_: CrossGaps,
    flex_cross_gap_sizes_: Option<Vector<LayoutUnit>>,
    flex_gap_placement_reversal_: Option<FlexGapPlacementReversal>,
    content_inline_start_: LayoutUnit,
    content_inline_end_: LayoutUnit,
    content_block_start_: LayoutUnit,
    content_block_end_: LayoutUnit,
    main_direction_: GridTrackSizingDirection,
    multicol_spanner_adjacent_intersections_: RefCell<HashSet<u32>>,
}

impl GapGeometry {
    // cpp: layoutng/internal/gap/gap_geometry.h:162-163
    pub fn new(container_type: ContainerType) -> Self {
        Self {
            inline_gap_size_: LayoutUnit::default(),
            block_gap_size_: LayoutUnit::default(),
            container_type_: container_type,
            main_gaps_: MainGaps::default(),
            cross_gaps_: CrossGaps::default(),
            flex_cross_gap_sizes_: None,
            flex_gap_placement_reversal_: None,
            content_inline_start_: LayoutUnit::default(),
            content_inline_end_: LayoutUnit::default(),
            content_block_start_: LayoutUnit::default(),
            content_block_end_: LayoutUnit::default(),
            main_direction_: GridTrackSizingDirection::kForRows,
            multicol_spanner_adjacent_intersections_: RefCell::new(HashSet::default()),
        }
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:166-184
    pub fn with_fragment_offsets(
        other: &Self,
        new_main_gaps: MainGaps,
        new_content_block_start: LayoutUnit,
        new_content_block_end: LayoutUnit,
    ) -> Self {
        Self {
            inline_gap_size_: other.inline_gap_size_,
            block_gap_size_: other.block_gap_size_,
            container_type_: other.container_type_,
            main_gaps_: new_main_gaps,
            cross_gaps_: other.cross_gaps_.clone(),
            flex_cross_gap_sizes_: other.flex_cross_gap_sizes_.clone(),
            flex_gap_placement_reversal_: other.flex_gap_placement_reversal_,
            content_inline_start_: other.content_inline_start_,
            content_inline_end_: other.content_inline_end_,
            content_block_start_: new_content_block_start,
            content_block_end_: new_content_block_end,
            main_direction_: other.main_direction_,
            multicol_spanner_adjacent_intersections_: RefCell::new(HashSet::default()),
        }
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:186-186
    pub fn Trace(&self, _visitor: &mut Visitor) {}

    // cpp: layoutng/internal/gap/gap_geometry.h:188-202
    pub fn Equals(&self, other: &Self) -> bool {
        self.inline_gap_size_ == other.inline_gap_size_
            && self.block_gap_size_ == other.block_gap_size_
            && self.container_type_ == other.container_type_
            && self.main_gaps_ == other.main_gaps_
            && self.cross_gaps_ == other.cross_gaps_
            && self.flex_cross_gap_sizes_ == other.flex_cross_gap_sizes_
            && self.flex_gap_placement_reversal_ == other.flex_gap_placement_reversal_
            && self.content_inline_start_ == other.content_inline_start_
            && self.content_inline_end_ == other.content_inline_end_
            && self.content_block_start_ == other.content_block_start_
            && self.content_block_end_ == other.content_block_end_
            && self.main_direction_ == other.main_direction_
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:239-240
    // cpp: layoutng/internal/gap/gap_geometry.cc:119-140
    pub fn HasRowGapFragmentation(
        &self,
        box_fragment: &PhysicalBoxFragment,
        is_main: bool,
    ) -> bool {
        if box_fragment.IsOnlyForNode() {
            return false;
        }
        if self.container_type_ == ContainerType::kGrid {
            return is_main;
        }
        if self.container_type_ == ContainerType::kFlex {
            return if self.main_direction_ == GridTrackSizingDirection::kForColumns {
                !is_main
            } else {
                is_main
            };
        }
        false
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:216-226
    // cpp: layoutng/internal/gap/gap_geometry.cc:220-248
    pub fn ComputeInkOverflowForGaps(
        &self,
        writing_direction: WritingDirectionMode,
        container_size: &PhysicalSize,
        inline_thickness: LayoutUnit,
        block_thickness: LayoutUnit,
        outsets: &GapDecorationInkOutsets,
    ) -> PhysicalRect {
        assert!(!self.main_gaps_.is_empty() || !self.cross_gaps_.is_empty());
        let mut inline_start = self.content_inline_start_;
        let mut inline_size = self.content_inline_end_ - self.content_inline_start_;
        let mut block_start = self.content_block_start_;
        let mut block_size = self.content_block_end_ - self.content_block_start_;
        inline_start -= inline_thickness / 2 + outsets.inline_start;
        inline_size += inline_thickness + outsets.InlineOutsetThickness();
        block_start -= block_thickness / 2 + outsets.block_start;
        block_size += block_thickness + outsets.BlockOutsetThickness();
        let logical_rect =
            LogicalRect::from_units(inline_start, block_start, inline_size, block_size);
        let converter = WritingModeConverter::new(writing_direction, *container_size);
        converter.ToPhysicalRect(logical_rect)
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:228-234
    // cpp: layoutng/internal/gap/gap_geometry.cc:250-264
    pub fn GetCrossingGapSize(&self, direction: GridTrackSizingDirection) -> LayoutUnit {
        let base_size = if direction == GridTrackSizingDirection::kForColumns {
            self.block_gap_size_
        } else {
            self.inline_gap_size_
        };
        let Some(sizes) = &self.flex_cross_gap_sizes_ else {
            return base_size;
        };
        if self.container_type_ != ContainerType::kFlex
            || !self.IsMainDirection(direction)
            || sizes.is_empty()
        {
            return base_size;
        }
        let mut max_size = base_size;
        for size in sizes {
            if *size > max_size {
                max_size = *size;
            }
        }
        max_size
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:534-534
    // cpp: layoutng/internal/gap/gap_geometry.cc:266-279
    pub fn ToString(&self, verbose: bool) -> String {
        let mut builder = StringBuilder::default();
        builder.Append("MainGaps: [");
        for main_gap in &self.main_gaps_ {
            builder.Append(main_gap.ToString(verbose));
            builder.Append(", ");
        }
        builder.Append("] ");
        builder.Append("CrossGaps: [");
        for cross_gap in &self.cross_gaps_ {
            builder.Append(cross_gap.ToString(verbose));
            builder.Append(", ");
        }
        builder.Append("] ");
        builder.ToString()
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:546-556
    // cpp: layoutng/internal/gap/gap_geometry.cc:281-307
    pub fn ComputeInsetEnd(
        &self,
        style: &ComputedStyle,
        _gap_index: u32,
        _intersection_index: u32,
        _intersections: &Vector<GapIntersection>,
        is_cap_intersection: bool,
        is_column_gap: bool,
        is_main: bool,
        has_joining_decoration: bool,
        cross_gap_width: LayoutUnit,
        cross_decoration_width: LayoutUnit,
    ) -> LayoutUnit {
        let inset = if is_cap_intersection {
            if is_column_gap {
                style.ColumnRuleInsetCapEnd()
            } else {
                style.RowRuleInsetCapEnd()
            }
        } else if is_column_gap {
            style.ColumnRuleInsetJunctionEnd()
        } else {
            style.RowRuleInsetJunctionEnd()
        };
        if inset.IsOverlapJoin() {
            return self.ComputeOverlapJoinInset(
                has_joining_decoration,
                is_main,
                cross_gap_width,
                cross_decoration_width,
            );
        }
        ValueForLength(inset, cross_gap_width)
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:560-570
    // cpp: layoutng/internal/gap/gap_geometry.cc:309-336
    pub fn ComputeInsetStart(
        &self,
        style: &ComputedStyle,
        _gap_index: u32,
        _intersection_index: u32,
        _intersections: &Vector<GapIntersection>,
        is_cap_intersection: bool,
        is_column_gap: bool,
        is_main: bool,
        has_joining_decoration: bool,
        cross_gap_width: LayoutUnit,
        cross_decoration_width: LayoutUnit,
    ) -> LayoutUnit {
        let inset = if is_cap_intersection {
            if is_column_gap {
                style.ColumnRuleInsetCapStart()
            } else {
                style.RowRuleInsetCapStart()
            }
        } else if is_column_gap {
            style.ColumnRuleInsetJunctionStart()
        } else {
            style.RowRuleInsetJunctionStart()
        };
        if inset.IsOverlapJoin() {
            return self.ComputeOverlapJoinInset(
                has_joining_decoration,
                is_main,
                cross_gap_width,
                cross_decoration_width,
            );
        }
        ValueForLength(inset, cross_gap_width)
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:664-674
    // cpp: layoutng/internal/gap/gap_geometry.cc:338-358
    fn ComputeOverlapJoinInset(
        &self,
        has_joining_decoration: bool,
        is_main: bool,
        cross_gap_width: LayoutUnit,
        cross_decoration_width: LayoutUnit,
    ) -> LayoutUnit {
        if !has_joining_decoration {
            return LayoutUnit::default();
        }
        if is_main
            && (self.GetContainerType() == ContainerType::kFlex
                || self.GetContainerType() == ContainerType::kMultiColumn)
        {
            return -cross_gap_width / 2;
        }
        (-cross_gap_width / 2) - (cross_decoration_width / 2)
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:236-236
    pub fn GetContainerType(&self) -> ContainerType {
        self.container_type_
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:242-246
    pub fn SetInlineGapSize(&mut self, size: LayoutUnit) {
        self.inline_gap_size_ = size;
    }

    pub fn GetInlineGapSize(&self) -> LayoutUnit {
        self.inline_gap_size_
    }

    pub fn SetBlockGapSize(&mut self, size: LayoutUnit) {
        self.block_gap_size_ = size;
    }

    pub fn GetBlockGapSize(&self) -> LayoutUnit {
        self.block_gap_size_
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:248-252
    pub fn GetFlexCrossGapSize(&self, line_index: u32) -> LayoutUnit {
        let sizes = self
            .flex_cross_gap_sizes_
            .as_ref()
            .expect("no flex cross gap sizes");
        assert!((line_index as usize) < sizes.len());
        sizes[line_index as usize]
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:254-260
    pub fn GetContentInlineStart(&self) -> LayoutUnit {
        self.content_inline_start_
    }

    pub fn GetContentInlineEnd(&self) -> LayoutUnit {
        self.content_inline_end_
    }

    pub fn GetContentBlockStart(&self) -> LayoutUnit {
        self.content_block_start_
    }

    pub fn GetContentBlockEnd(&self) -> LayoutUnit {
        self.content_block_end_
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:254-254
    // cpp: layoutng/internal/gap/gap_geometry.cc:360-364
    pub fn SetContentInlineOffsets(&mut self, start_offset: LayoutUnit, end_offset: LayoutUnit) {
        self.content_inline_start_ = start_offset;
        self.content_inline_end_ = end_offset;
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:258-258
    // cpp: layoutng/internal/gap/gap_geometry.cc:366-370
    pub fn SetContentBlockOffsets(&mut self, start_offset: LayoutUnit, end_offset: LayoutUnit) {
        self.content_block_start_ = start_offset;
        self.content_block_end_ = end_offset;
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:262-268
    pub fn ReserveMainGaps(&mut self, capacity: u32) {
        let additional = (capacity as usize).saturating_sub(self.main_gaps_.len());
        self.main_gaps_.reserve(additional);
    }

    pub fn ReserveCrossGaps(&mut self, capacity: u32) {
        let additional = (capacity as usize).saturating_sub(self.cross_gaps_.len());
        self.cross_gaps_.reserve(additional);
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:270-274
    pub fn AddMainGap(&mut self, offset: LayoutUnit) -> &mut MainGap {
        self.AddMainGapWithType(offset, SpannerMainGapType::kNone)
    }

    pub fn AddMainGapWithType(
        &mut self,
        offset: LayoutUnit,
        gap_type: SpannerMainGapType,
    ) -> &mut MainGap {
        self.main_gaps_
            .push(MainGap::with_spanner_type(offset, gap_type));
        self.main_gaps_.last_mut().unwrap()
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:276-285
    pub fn AddCrossGap(&mut self, offset: LogicalOffset) -> &mut CrossGap {
        self.cross_gaps_.push(CrossGap::new(offset));
        self.cross_gaps_.last_mut().unwrap()
    }

    pub fn AddCrossGapWithEdgeState(
        &mut self,
        offset: LogicalOffset,
        state: EdgeIntersectionState,
    ) -> &mut CrossGap {
        self.cross_gaps_
            .push(CrossGap::with_edge_state(offset, state));
        self.cross_gaps_.last_mut().unwrap()
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:287-290
    pub fn RemoveLastMainGap(&mut self) {
        assert!(!self.main_gaps_.is_empty());
        self.main_gaps_.pop();
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:292-299
    pub fn MainGapAt(&self, index: u32) -> &MainGap {
        assert!((index as usize) < self.main_gaps_.len());
        &self.main_gaps_[index as usize]
    }

    pub fn MainGapAtMut(&mut self, index: u32) -> &mut MainGap {
        assert!((index as usize) < self.main_gaps_.len());
        &mut self.main_gaps_[index as usize]
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:302-305
    pub fn CrossGapAt(&mut self, index: u32) -> &mut CrossGap {
        assert!((index as usize) < self.cross_gaps_.len());
        &mut self.cross_gaps_[index as usize]
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:307-308
    pub fn MainGapCount(&self) -> u32 {
        self.main_gaps_.len() as u32
    }

    pub fn CrossGapCount(&self) -> u32 {
        self.cross_gaps_.len() as u32
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:313-325
    pub fn ResizeFlexCrossGapSizes(&mut self, count: u32) {
        if self.flex_cross_gap_sizes_.is_none() {
            self.flex_cross_gap_sizes_ = Some(Vector::default());
        }
        self.flex_cross_gap_sizes_
            .as_mut()
            .unwrap()
            .resize(count as usize, LayoutUnit::default());
    }

    pub fn SetFlexCrossGapSize(&mut self, index: u32, size: LayoutUnit) {
        let sizes = self
            .flex_cross_gap_sizes_
            .as_mut()
            .expect("no flex cross gap sizes");
        assert!((index as usize) < sizes.len());
        sizes[index as usize] = size;
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:342-344
    pub fn SetFlexGapPlacementReversal(&mut self, reversal: FlexGapPlacementReversal) {
        self.flex_gap_placement_reversal_ = Some(reversal);
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:348-349
    // cpp: layoutng/internal/gap/gap_geometry.cc:142-149
    pub fn HasNonIdentityDecorationOrder(&self, track_direction: GridTrackSizingDirection) -> bool {
        let Some(reversal) = self.flex_gap_placement_reversal_ else {
            return false;
        };
        !self.IsMainDirection(track_direction) || reversal.reverse_lines
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:651-653
    // cpp: layoutng/internal/gap/gap_geometry.cc:151-173
    fn GetFlexLineCrossGapStartAndCount(&self, owning_main_gap_index: u32) -> (u32, u32) {
        assert!(self.flex_gap_placement_reversal_.is_some());
        if self.main_gaps_.is_empty() {
            assert_eq!(owning_main_gap_index, 0);
            assert!(!self.cross_gaps_.is_empty());
            return (0, self.cross_gaps_.len() as u32);
        }
        if (owning_main_gap_index as usize) < self.main_gaps_.len() {
            let main_gap = &self.main_gaps_[owning_main_gap_index as usize];
            assert!(main_gap.HasCrossGapsBefore());
            return (
                main_gap.GetCrossGapBeforeStart(),
                main_gap.GetCrossGapBeforeCount(),
            );
        }
        assert_eq!(owning_main_gap_index as usize, self.main_gaps_.len());
        let last_main_gap = self.main_gaps_.last().unwrap();
        assert!(last_main_gap.HasCrossGapsAfter());
        (
            last_main_gap.GetCrossGapAfterStart(),
            last_main_gap.GetCrossGapAfterCount(),
        )
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:354-358
    // cpp: layoutng/internal/gap/gap_geometry.cc:175-218
    pub fn DecorationIndexForGap(
        &self,
        track_direction: GridTrackSizingDirection,
        geometric_index: u32,
        owning_main_gap_index: Option<u32>,
        total_gap_count: u32,
    ) -> u32 {
        assert!(self.HasNonIdentityDecorationOrder(track_direction));
        assert!(geometric_index < total_gap_count);
        if self.IsMainDirection(track_direction) {
            return total_gap_count - 1 - geometric_index;
        }
        let owner = owning_main_gap_index.expect("cross gap has no owning main gap");
        let (line_start, line_gap_count) = self.GetFlexLineCrossGapStartAndCount(owner);
        assert!(geometric_index >= line_start);
        let geometric_index_in_line = geometric_index - line_start;
        assert!(geometric_index_in_line < line_gap_count);
        let mut placement_index_in_line = geometric_index_in_line;
        let reversal = self.flex_gap_placement_reversal_.unwrap();
        if reversal.reverse_items_in_line {
            assert!(placement_index_in_line <= line_gap_count - 1);
            placement_index_in_line = line_gap_count - 1 - placement_index_in_line;
        }
        assert!(line_start <= total_gap_count);
        assert!(line_gap_count <= total_gap_count - line_start);
        let line_start_in_placement_order = if reversal.reverse_lines {
            total_gap_count - line_start - line_gap_count
        } else {
            line_start
        };
        line_start_in_placement_order + placement_index_in_line
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:365-367
    pub fn InitPaintState(&self) {
        self.multicol_spanner_adjacent_intersections_
            .borrow_mut()
            .clear();
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:369-371
    pub fn SetMainDirection(&mut self, direction: GridTrackSizingDirection) {
        self.main_direction_ = direction;
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:369-381
    pub fn GetMainDirection(&self) -> GridTrackSizingDirection {
        self.main_direction_
    }

    pub fn IsMainDirection(&self, direction: GridTrackSizingDirection) -> bool {
        self.main_direction_ == direction
    }

    pub fn GetMainGaps(&self) -> &MainGaps {
        &self.main_gaps_
    }

    pub fn GetCrossGaps(&self) -> &CrossGaps {
        &self.cross_gaps_
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:383-396
    pub fn GetGapCenterOffset(
        &self,
        direction: GridTrackSizingDirection,
        gap_index: u32,
    ) -> LayoutUnit {
        if self.IsMainDirection(direction) {
            return self.GetMainGaps()[gap_index as usize].GetGapOffset();
        }
        if direction == GridTrackSizingDirection::kForColumns {
            self.GetCrossGaps()[gap_index as usize]
                .GetGapOffset()
                .inline_offset
        } else {
            self.GetCrossGaps()[gap_index as usize]
                .GetGapOffset()
                .block_offset
        }
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:409-413
    // cpp: layoutng/internal/gap/gap_geometry.cc:372-386
    pub fn GenerateIntersectionListForGap(
        &self,
        direction: GridTrackSizingDirection,
        gap_index: u32,
        intersections: &mut Vector<GapIntersection>,
        cross_gap_owner_index: Option<u32>,
    ) {
        intersections.clear();
        if self.IsMainDirection(direction) {
            self.GenerateMainIntersectionList(direction, gap_index, intersections);
        } else {
            self.GenerateCrossIntersectionList(
                direction,
                gap_index,
                intersections,
                cross_gap_owner_index,
            );
        }
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:577-580
    // cpp: layoutng/internal/gap/gap_geometry.cc:388-427
    fn GenerateMainIntersectionList(
        &self,
        direction: GridTrackSizingDirection,
        gap_index: u32,
        intersections: &mut Vector<GapIntersection>,
    ) {
        let mut cursor =
            GapSegmentStateCursor::new(self.GetGapSegmentStateRangesForGap(direction, gap_index));
        if self.GetContainerType() == ContainerType::kMultiColumn {
            assert_eq!(direction, GridTrackSizingDirection::kForRows);
            if self.GetMainGaps()[gap_index as usize].IsSpannerMainGap() {
                return;
            }
        }
        match self.GetContainerType() {
            ContainerType::kGridLanes => {
                let mut walker = GridLanesMainGapSegmentWalker::new(self, gap_index);
                let additional =
                    (walker.IntersectionCapacity() as usize).saturating_sub(intersections.len());
                intersections.reserve(additional);
                let content_start = if direction == GridTrackSizingDirection::kForColumns {
                    self.content_block_start_
                } else {
                    self.content_inline_start_
                };
                intersections.push(GapIntersection::with_segment_state(
                    content_start,
                    cursor.GetNextGapSegmentState(),
                ));
                while let Some(segment) = walker.Next() {
                    intersections.push(GapIntersection::with_segment_state(
                        segment.end_offset,
                        cursor.GetNextGapSegmentState(),
                    ));
                }
            }
            ContainerType::kGrid | ContainerType::kMultiColumn => {
                self.GenerateMainIntersectionListForGridAndMulticol(
                    direction,
                    intersections,
                    &mut cursor,
                );
            }
            ContainerType::kFlex => {
                self.GenerateMainIntersectionListForFlex(
                    direction,
                    gap_index,
                    intersections,
                    &mut cursor,
                );
            }
        }
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:583-586
    // cpp: layoutng/internal/gap/gap_geometry.cc:429-450
    fn GenerateMainIntersectionListForGridAndMulticol(
        &self,
        direction: GridTrackSizingDirection,
        intersections: &mut Vector<GapIntersection>,
        cursor: &mut GapSegmentStateCursor<'_>,
    ) {
        let capacity = self.GetCrossGaps().len().saturating_add(2);
        intersections.reserve(capacity.saturating_sub(intersections.len()));
        let content_start = if direction == GridTrackSizingDirection::kForColumns {
            self.content_block_start_
        } else {
            self.content_inline_start_
        };
        intersections.push(GapIntersection::with_segment_state(
            content_start,
            cursor.GetNextGapSegmentState(),
        ));
        assert_eq!(self.GetMainDirection(), GridTrackSizingDirection::kForRows);
        for cross_gap in self.GetCrossGaps() {
            intersections.push(GapIntersection::with_segment_state(
                cross_gap.GetGapOffset().inline_offset,
                cursor.GetNextGapSegmentState(),
            ));
        }
        let content_end = if direction == GridTrackSizingDirection::kForColumns {
            self.content_block_end_
        } else {
            self.content_inline_end_
        };
        intersections.push(GapIntersection::with_segment_state(
            content_end,
            cursor.GetNextGapSegmentState(),
        ));
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:592-597
    // cpp: layoutng/internal/gap/gap_geometry.cc:452-641
    fn GenerateMainIntersectionListForFlex(
        &self,
        direction: GridTrackSizingDirection,
        gap_index: u32,
        intersections: &mut Vector<GapIntersection>,
        cursor: &mut GapSegmentStateCursor<'_>,
    ) {
        let main_gap = self.GetMainGaps()[gap_index as usize].clone();
        let has_cross_gaps_before = main_gap.HasCrossGapsBefore();
        let has_cross_gaps_after = main_gap.HasCrossGapsAfter();
        let num_cross_gaps_before = if has_cross_gaps_before {
            main_gap.GetCrossGapBeforeCount()
        } else {
            0
        };
        let num_cross_gaps_after = if has_cross_gaps_after {
            main_gap.GetCrossGapAfterCount()
        } else {
            0
        };
        let capacity = num_cross_gaps_before
            .wrapping_add(num_cross_gaps_after)
            .wrapping_add(2) as usize;
        intersections.reserve(capacity.saturating_sub(intersections.len()));
        let content_start = if direction == GridTrackSizingDirection::kForColumns {
            self.content_block_start_
        } else {
            self.content_inline_start_
        };
        intersections.push(GapIntersection::with_segment_state(
            content_start,
            cursor.GetNextGapSegmentState(),
        ));
        if !has_cross_gaps_before && !has_cross_gaps_after {
            let content_end = if direction == GridTrackSizingDirection::kForColumns {
                self.content_block_end_
            } else {
                self.content_inline_end_
            };
            intersections.push(GapIntersection::with_segment_state(
                content_end,
                cursor.GetNextGapSegmentState(),
            ));
            return;
        }

        let cross_direction = if direction == GridTrackSizingDirection::kForRows {
            GridTrackSizingDirection::kForColumns
        } else {
            GridTrackSizingDirection::kForRows
        };
        let mut cross_gap_size_above = None;
        if has_cross_gaps_before {
            cross_gap_size_above = Some(self.GetFlexCrossGapSize(gap_index));
        }
        let mut cross_gap_size_below = None;
        if has_cross_gaps_after {
            cross_gap_size_below = Some(self.GetFlexCrossGapSize(gap_index.wrapping_add(1)));
        }

        let mut cross_gaps_before_current_idx = if has_cross_gaps_before {
            main_gap.GetCrossGapBeforeStart()
        } else {
            u32::MAX
        };
        let cross_gaps_before_end_idx = if has_cross_gaps_before {
            main_gap.GetCrossGapBeforeEnd()
        } else {
            0
        };
        let mut cross_gaps_after_current_idx = if has_cross_gaps_after {
            main_gap.GetCrossGapAfterStart()
        } else {
            u32::MAX
        };
        let cross_gaps_after_end_idx = if has_cross_gaps_after {
            main_gap.GetCrossGapAfterEnd()
        } else {
            0
        };

        while cross_gaps_before_current_idx <= cross_gaps_before_end_idx
            && cross_gaps_after_current_idx <= cross_gaps_after_end_idx
        {
            let cross_gap_before_offset = self.GetCrossGaps()
                [cross_gaps_before_current_idx as usize]
                .GetGapOffsetForDirection(cross_direction);
            let cross_gap_after_offset = self.GetCrossGaps()[cross_gaps_after_current_idx as usize]
                .GetGapOffsetForDirection(cross_direction);
            if cross_gap_before_offset <= cross_gap_after_offset {
                ProcessCrossGapIntersection(
                    intersections,
                    cross_gap_before_offset,
                    true,
                    cross_gap_size_above,
                    cross_gap_size_below,
                );
                cross_gaps_before_current_idx = cross_gaps_before_current_idx.wrapping_add(1);
                if cross_gap_before_offset == cross_gap_after_offset {
                    cross_gaps_after_current_idx = cross_gaps_after_current_idx.wrapping_add(1);
                }
            } else {
                ProcessCrossGapIntersection(
                    intersections,
                    cross_gap_after_offset,
                    false,
                    cross_gap_size_above,
                    cross_gap_size_below,
                );
                cross_gaps_after_current_idx = cross_gaps_after_current_idx.wrapping_add(1);
            }
        }

        while cross_gaps_before_current_idx <= cross_gaps_before_end_idx {
            ProcessCrossGapIntersection(
                intersections,
                self.GetCrossGaps()[cross_gaps_before_current_idx as usize]
                    .GetGapOffsetForDirection(cross_direction),
                true,
                cross_gap_size_above,
                cross_gap_size_below,
            );
            cross_gaps_before_current_idx = cross_gaps_before_current_idx.wrapping_add(1);
        }
        while cross_gaps_after_current_idx <= cross_gaps_after_end_idx {
            ProcessCrossGapIntersection(
                intersections,
                self.GetCrossGaps()[cross_gaps_after_current_idx as usize]
                    .GetGapOffsetForDirection(cross_direction),
                false,
                cross_gap_size_above,
                cross_gap_size_below,
            );
            cross_gaps_after_current_idx = cross_gaps_after_current_idx.wrapping_add(1);
        }

        if intersections.last().unwrap().IsOverlapWindowOpen() {
            intersections.last_mut().unwrap().ResetOverlapState();
        }
        let content_end = if direction == GridTrackSizingDirection::kForColumns {
            self.content_block_end_
        } else {
            self.content_inline_end_
        };
        intersections.push(GapIntersection::with_segment_state(
            content_end,
            cursor.GetNextGapSegmentState(),
        ));
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:603-608
    // cpp: layoutng/internal/gap/gap_geometry.cc:643-674
    fn GenerateCrossIntersectionList(
        &self,
        direction: GridTrackSizingDirection,
        gap_index: u32,
        intersections: &mut Vector<GapIntersection>,
        cross_gap_owner_index: Option<u32>,
    ) {
        let mut cursor =
            GapSegmentStateCursor::new(self.GetGapSegmentStateRangesForGap(direction, gap_index));
        match self.GetContainerType() {
            ContainerType::kGridLanes => {
                let owner = cross_gap_owner_index.expect("grid-lanes cross gap has no owner");
                self.GenerateCrossIntersectionListForGridLanes(owner, intersections);
            }
            ContainerType::kGrid => {
                assert!(cross_gap_owner_index.is_none());
                self.GenerateCrossIntersectionListForGrid(direction, intersections, &mut cursor);
            }
            ContainerType::kFlex => {
                let owner = cross_gap_owner_index.expect("flex cross gap has no owner");
                self.GenerateCrossIntersectionListForFlex(
                    direction,
                    gap_index,
                    intersections,
                    &mut cursor,
                    owner,
                );
            }
            ContainerType::kMultiColumn => {
                self.GenerateCrossIntersectionListForMulticol(
                    direction,
                    gap_index,
                    intersections,
                    &mut cursor,
                );
            }
        }
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:613-617
    // cpp: layoutng/internal/gap/gap_geometry.cc:676-701
    fn GenerateCrossIntersectionListForGrid(
        &self,
        direction: GridTrackSizingDirection,
        intersections: &mut Vector<GapIntersection>,
        cursor: &mut GapSegmentStateCursor<'_>,
    ) {
        let capacity = self.main_gaps_.len().saturating_add(2);
        intersections.reserve(capacity.saturating_sub(intersections.len()));
        let content_start = if direction == GridTrackSizingDirection::kForColumns {
            self.content_block_start_
        } else {
            self.content_inline_start_
        };
        intersections.push(GapIntersection::with_segment_state(
            content_start,
            cursor.GetNextGapSegmentState(),
        ));
        for main_gap in self.GetMainGaps() {
            intersections.push(GapIntersection::with_segment_state(
                main_gap.GetGapOffset(),
                cursor.GetNextGapSegmentState(),
            ));
        }
        let content_end = if direction == GridTrackSizingDirection::kForColumns {
            self.content_block_end_
        } else {
            self.content_inline_end_
        };
        intersections.push(GapIntersection::with_segment_state(
            content_end,
            cursor.GetNextGapSegmentState(),
        ));
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:632-638
    // cpp: layoutng/internal/gap/gap_geometry.cc:703-762
    fn GenerateCrossIntersectionListForFlex(
        &self,
        direction: GridTrackSizingDirection,
        gap_index: u32,
        intersections: &mut Vector<GapIntersection>,
        cursor: &mut GapSegmentStateCursor<'_>,
        main_gap_index: u32,
    ) {
        intersections.reserve(2usize.saturating_sub(intersections.len()));
        let cross_gap = self.GetCrossGaps()[gap_index as usize].clone();
        let offset = if direction == GridTrackSizingDirection::kForColumns {
            cross_gap.GetGapOffset().block_offset
        } else {
            cross_gap.GetGapOffset().inline_offset
        };
        intersections.push(GapIntersection::with_segment_state(
            offset,
            cursor.GetNextGapSegmentState(),
        ));
        let end_offset_for_flex_cross_gap =
            self.ComputeEndOffsetForFlexCrossGap(direction, cross_gap.EndsAtEdge(), main_gap_index);
        intersections.push(GapIntersection::with_segment_state(
            end_offset_for_flex_cross_gap,
            cursor.GetNextGapSegmentState(),
        ));
        let edge_state = cross_gap.GetEdgeIntersectionState();
        let is_start_edge = edge_state == EdgeIntersectionState::kStart
            || edge_state == EdgeIntersectionState::kBoth;
        if !is_start_edge {
            intersections[0].SetMainGapIndex(if edge_state == EdgeIntersectionState::kEnd {
                (self.GetMainGaps().len() as u32).wrapping_sub(1)
            } else {
                main_gap_index.wrapping_sub(1)
            });
        }
        let is_end_edge =
            edge_state == EdgeIntersectionState::kEnd || edge_state == EdgeIntersectionState::kBoth;
        if !is_end_edge && (main_gap_index as usize) < self.GetMainGaps().len() {
            intersections[1].SetMainGapIndex(main_gap_index);
        }
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:643-647
    // cpp: layoutng/internal/gap/gap_geometry.cc:764-803
    fn GenerateCrossIntersectionListForMulticol(
        &self,
        direction: GridTrackSizingDirection,
        gap_index: u32,
        intersections: &mut Vector<GapIntersection>,
        cursor: &mut GapSegmentStateCursor<'_>,
    ) {
        assert_eq!(direction, GridTrackSizingDirection::kForColumns);
        let capacity = self.main_gaps_.len().saturating_add(2);
        intersections.reserve(capacity.saturating_sub(intersections.len()));
        assert!((gap_index as usize) < self.GetCrossGaps().len());
        let cross_gap = self.GetCrossGaps()[gap_index as usize].clone();
        intersections.push(GapIntersection::with_segment_state(
            cross_gap.GetGapOffset().block_offset,
            cursor.GetNextGapSegmentState(),
        ));
        for main_gap in self.GetMainGaps() {
            intersections.push(GapIntersection::with_segment_state(
                main_gap.GetGapOffset(),
                cursor.GetNextGapSegmentState(),
            ));
            if main_gap.IsSpannerMainGap() {
                self.multicol_spanner_adjacent_intersections_
                    .borrow_mut()
                    .insert((intersections.len() as u32).wrapping_sub(1));
            }
        }
        intersections.push(GapIntersection::with_segment_state(
            self.content_block_end_,
            cursor.GetNextGapSegmentState(),
        ));
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:334-334
    // cpp: layoutng/internal/gap/gap_geometry.cc:805-826
    pub fn GridAxisOffsetForLaneBoundary(&self, lane_boundary: u32) -> LayoutUnit {
        assert_eq!(self.container_type_, ContainerType::kGridLanes);
        let grid_axis_is_inline = self.main_direction_ == GridTrackSizingDirection::kForColumns;
        let content_start = if grid_axis_is_inline {
            self.content_inline_start_
        } else {
            self.content_block_start_
        };
        let content_end = if grid_axis_is_inline {
            self.content_inline_end_
        } else {
            self.content_block_end_
        };
        let lane_count = (self.main_gaps_.len() as u32).wrapping_add(1);
        if lane_boundary == 0 {
            return content_start;
        }
        if lane_boundary == lane_count {
            return content_end;
        }
        self.MainGapAt(lane_boundary.wrapping_sub(1)).GetGapOffset()
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:623-627
    // cpp: layoutng/internal/gap/gap_geometry.cc:828-853
    fn GenerateCrossIntersectionListForGridLanes(
        &self,
        lane: u32,
        intersections: &mut Vector<GapIntersection>,
    ) {
        let lane_count = (self.main_gaps_.len() as u32).wrapping_add(1);
        assert!(lane < lane_count);
        assert!(intersections.is_empty());
        intersections.reserve(2);
        let mut start = GapIntersection::new(self.GridAxisOffsetForLaneBoundary(lane));
        if lane > 0 {
            start.SetMainGapIndex(lane - 1);
        }
        intersections.push(start);
        let mut end =
            GapIntersection::new(self.GridAxisOffsetForLaneBoundary(lane.wrapping_add(1)));
        if lane.wrapping_add(1) < lane_count {
            end.SetMainGapIndex(lane);
        }
        intersections.push(end);
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:661-662
    // cpp: layoutng/internal/gap/gap_geometry.cc:855-872
    fn ComputeEndOffsetForFlexCrossGap(
        &self,
        direction: GridTrackSizingDirection,
        cross_gap_is_at_end: bool,
        main_gap_index: u32,
    ) -> LayoutUnit {
        let main_gaps = self.GetMainGaps();
        debug_assert!((main_gap_index as usize) <= main_gaps.len());
        if (main_gap_index as usize) == main_gaps.len() || cross_gap_is_at_end {
            return if direction == GridTrackSizingDirection::kForRows {
                self.content_inline_end_
            } else {
                self.content_block_end_
            };
        }
        main_gaps[main_gap_index as usize].GetGapOffset()
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:420-425
    // cpp: layoutng/internal/gap/gap_geometry.cc:874-933
    pub fn IsIntersectionAtContainerEdge(
        &self,
        gap_index: u32,
        intersection_index: u32,
        intersection_count: u32,
        is_main_gap: bool,
        intersections: &Vector<GapIntersection>,
    ) -> bool {
        assert!(intersection_count > 0);
        let last_intersection_index = intersection_count - 1;
        if is_main_gap || self.GetContainerType() == ContainerType::kGrid {
            return intersection_index == 0 || intersection_index == last_intersection_index;
        }
        if self.GetContainerType() == ContainerType::kGridLanes {
            assert!(!is_main_gap);
            return !intersections[intersection_index as usize].HasMainGapIndex();
        }
        if self.GetContainerType() == ContainerType::kFlex {
            assert!(!is_main_gap);
            let edge_state = self.GetCrossGaps()[gap_index as usize].GetEdgeIntersectionState();
            if edge_state == EdgeIntersectionState::kBoth {
                return intersection_index == 0 || intersection_index == last_intersection_index;
            } else if edge_state == EdgeIntersectionState::kStart {
                return intersection_index == 0;
            } else if edge_state == EdgeIntersectionState::kEnd {
                return intersection_index == last_intersection_index;
            }
        }
        if self.GetContainerType() == ContainerType::kMultiColumn {
            assert!(!is_main_gap);
            return intersection_index == 0
                || intersection_index == last_intersection_index
                || self
                    .multicol_spanner_adjacent_intersections_
                    .borrow()
                    .contains(&intersection_index);
        }
        false
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:437-444
    // cpp: layoutng/internal/gap/gap_geometry.cc:935-949
    pub fn IsCapIntersection(
        &self,
        cross_direction: GridTrackSizingDirection,
        gap_index: u32,
        intersection_index: u32,
        is_main_gap: bool,
        rule_visibility: RuleVisibilityItems,
        cross_rule_visibility: RuleVisibilityItems,
        intersections: &Vector<GapIntersection>,
    ) -> bool {
        self.IsIntersectionAtContainerEdge(
            gap_index,
            intersection_index,
            intersections.len() as u32,
            is_main_gap,
            intersections,
        ) || !CSSGapDecorationUtils::HasCrossGapSegment(
            cross_direction,
            gap_index,
            intersection_index,
            rule_visibility,
            cross_rule_visibility,
            self,
            intersections,
        )
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:451-457
    // cpp: layoutng/internal/gap/gap_geometry.cc:951-981
    pub fn GetCrossDecorationWidthForIntersection(
        &self,
        _gap_index: u32,
        intersection_index: u32,
        is_main_gap: bool,
        intersections: &Vector<GapIntersection>,
        is_cap_intersection: bool,
        cross_decoration_widths: &Vector<i32>,
    ) -> LayoutUnit {
        if is_cap_intersection {
            return LayoutUnit::default();
        }
        let intersection = &intersections[intersection_index as usize];
        if self.GetContainerType() == ContainerType::kGridLanes && is_main_gap {
            return LayoutUnit::default();
        }
        if intersection.HasMainGapIndex() {
            assert!(
                self.GetContainerType() == ContainerType::kGridLanes
                    || self.GetContainerType() == ContainerType::kFlex
            );
            return LayoutUnit::from_signed(
                cross_decoration_widths[intersection.GetMainGapIndex() as usize],
            );
        }
        LayoutUnit::from_signed(
            cross_decoration_widths[intersection_index.wrapping_sub(1) as usize],
        )
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:466-473
    // cpp: layoutng/internal/gap/gap_geometry.cc:983-1028
    pub fn GetMaxInsetWidth(
        &self,
        track_direction: GridTrackSizingDirection,
        gap_index: u32,
        intersection_index: u32,
        is_main_gap: bool,
        intersections: &Vector<GapIntersection>,
    ) -> LayoutUnit {
        let intersection = &intersections[intersection_index as usize];
        if self.GetContainerType() != ContainerType::kFlex
            || !self.IsMainDirection(track_direction)
            || !intersection.HasOverlapState()
        {
            return self.GetCrossWidthForIntersection(
                track_direction,
                gap_index,
                intersection_index,
                is_main_gap,
                intersections,
            );
        }
        assert!(!self.IsIntersectionAtContainerEdge(
            gap_index,
            intersection_index,
            intersections.len() as u32,
            is_main_gap,
            intersections,
        ));
        let open_intersection = &intersections[if intersection.IsOverlapWindowOpen() {
            intersection_index as usize
        } else {
            intersection_index.wrapping_sub(1) as usize
        }];
        let close_intersection = &intersections[if intersection.IsOverlapWindowClose() {
            intersection_index as usize
        } else {
            intersection_index.wrapping_add(1) as usize
        }];
        assert!(open_intersection.IsOverlapWindowOpen());
        assert!(close_intersection.IsOverlapWindowClose());
        let open_gap_width = self.GetFlexCrossGapSize(if open_intersection.IsAboveMainGap() {
            gap_index
        } else {
            gap_index.wrapping_add(1)
        });
        let close_gap_width = self.GetFlexCrossGapSize(if close_intersection.IsAboveMainGap() {
            gap_index
        } else {
            gap_index.wrapping_add(1)
        });
        close_intersection.GetOffset() + close_gap_width / 2
            - (open_intersection.GetOffset() - open_gap_width / 2)
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:479-485
    // cpp: layoutng/internal/gap/gap_geometry.cc:1030-1056
    pub fn GetCrossWidthForIntersection(
        &self,
        track_direction: GridTrackSizingDirection,
        gap_index: u32,
        intersection_index: u32,
        is_main_gap: bool,
        intersections: &Vector<GapIntersection>,
    ) -> LayoutUnit {
        if self.IsIntersectionAtContainerEdge(
            gap_index,
            intersection_index,
            intersections.len() as u32,
            is_main_gap,
            intersections,
        ) {
            return LayoutUnit::default();
        }
        let cross_gutter_width = if track_direction == GridTrackSizingDirection::kForRows {
            self.GetInlineGapSize()
        } else {
            self.GetBlockGapSize()
        };
        if self.GetContainerType() != ContainerType::kFlex || !self.IsMainDirection(track_direction)
        {
            return cross_gutter_width;
        }
        self.GetFlexCrossGapSize(
            if intersections[intersection_index as usize].IsAboveMainGap() {
                gap_index
            } else {
                gap_index.wrapping_add(1)
            },
        )
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:488-491
    // cpp: layoutng/internal/gap/gap_geometry.cc:1058-1103
    pub fn GetIntersectionGapSegmentState(
        &self,
        track_direction: GridTrackSizingDirection,
        primary_index: u32,
        secondary_index: u32,
    ) -> GapSegmentState {
        let mut ranges = None;
        if self.IsMainDirection(track_direction) {
            assert!((primary_index as usize) < self.main_gaps_.len());
            let main_gap = &self.main_gaps_[primary_index as usize];
            if main_gap.HasGapSegmentStateRanges() {
                ranges = Some(main_gap.GetGapSegmentStateRanges());
            }
        } else {
            assert!((primary_index as usize) < self.cross_gaps_.len());
            let cross_gap = &self.cross_gaps_[primary_index as usize];
            if cross_gap.HasGapSegmentStateRanges() {
                ranges = Some(cross_gap.GetGapSegmentStateRanges());
            }
        }
        let Some(ranges) = ranges else {
            return GapSegmentState::new(GapSegmentState::kNone as u32);
        };
        let mut first = 0usize;
        let mut last = ranges.len();
        while first < last {
            let middle = first + (last - first) / 2;
            if ranges[middle].end <= secondary_index {
                first = middle + 1;
            } else {
                last = middle;
            }
        }
        if first < ranges.len()
            && secondary_index >= ranges[first].start
            && secondary_index < ranges[first].end
        {
            return ranges[first].state;
        }
        GapSegmentState::new(GapSegmentState::kNone as u32)
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:503-505
    // cpp: layoutng/internal/gap/gap_geometry.cc:1105-1112
    pub fn IsTrackCovered(
        &self,
        track_direction: GridTrackSizingDirection,
        primary_index: u32,
        secondary_index: u32,
    ) -> bool {
        let gap_state =
            self.GetIntersectionGapSegmentState(track_direction, primary_index, secondary_index);
        gap_state.HasGapStatus(GapSegmentState::kBlocked)
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:522-526
    // cpp: layoutng/internal/gap/gap_geometry.cc:1114-1132
    pub fn GetIntersectionBlockedStatus(
        &self,
        track_direction: GridTrackSizingDirection,
        primary_index: u32,
        secondary_index: u32,
        _intersections: &Vector<GapIntersection>,
    ) -> BlockedStatus {
        let mut status = BlockedStatus::default();
        if secondary_index > 0
            && self.IsTrackCovered(track_direction, primary_index, secondary_index - 1)
        {
            status.SetBlockedStatus(BlockedStatus::kBlockedBefore);
        }
        if self.IsTrackCovered(track_direction, primary_index, secondary_index) {
            status.SetBlockedStatus(BlockedStatus::kBlockedAfter);
        }
        status
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:530-532
    // cpp: layoutng/internal/gap/gap_geometry.cc:1134-1148
    pub fn BlockedStatusFromGapStates(
        intersections: &Vector<GapIntersection>,
        index: u32,
    ) -> BlockedStatus {
        assert!((index as usize) < intersections.len());
        let mut status = BlockedStatus::default();
        if index > 0
            && intersections[index.wrapping_sub(1) as usize]
                .SegmentState()
                .HasGapStatus(GapSegmentState::kBlocked)
        {
            status.SetBlockedStatus(BlockedStatus::kBlockedBefore);
        }
        if intersections[index as usize]
            .SegmentState()
            .HasGapStatus(GapSegmentState::kBlocked)
        {
            status.SetBlockedStatus(BlockedStatus::kBlockedAfter);
        }
        status
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:536-541
    pub fn IsMultiColSpanner(&self, gap_index: u32, direction: GridTrackSizingDirection) -> bool {
        self.GetContainerType() == ContainerType::kMultiColumn
            && self.IsMainDirection(direction)
            && self.main_gaps_[gap_index as usize].IsSpannerMainGap()
    }

    pub fn IsMultiColSpannerInRows(&self, gap_index: u32) -> bool {
        self.IsMultiColSpanner(gap_index, GridTrackSizingDirection::kForRows)
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:493-496
    // cpp: layoutng/internal/gap/gap_geometry.cc:1150-1165
    pub fn GetGapSegmentStateRangesForGap(
        &self,
        track_direction: GridTrackSizingDirection,
        gap_index: u32,
    ) -> Option<&GapSegmentStateRanges> {
        if self.IsMainDirection(track_direction) {
            assert!((gap_index as usize) < self.main_gaps_.len());
            let main_gap = &self.main_gaps_[gap_index as usize];
            if main_gap.HasGapSegmentStateRanges() {
                return Some(main_gap.GetGapSegmentStateRanges());
            }
        } else {
            assert!((gap_index as usize) < self.cross_gaps_.len());
            let cross_gap = &self.cross_gaps_[gap_index as usize];
            if cross_gap.HasGapSegmentStateRanges() {
                return Some(cross_gap.GetGapSegmentStateRanges());
            }
        }
        None
    }

    // cpp: layoutng/internal/gap/gap_geometry.h:514-518
    // cpp: layoutng/internal/gap/gap_geometry.cc:1167-1178
    pub fn AdjustCrossGapsRangesForFragmentation(
        &mut self,
        last_track_in_previous_fragment: u32,
        first_track_in_next_fragment: u32,
        column_gaps_segment_ranges_start_indices: &mut Vector<u32>,
    ) {
        for i in 0..self.cross_gaps_.len() {
            let cross_gap = &mut self.cross_gaps_[i];
            if cross_gap.HasGapSegmentStateRanges() {
                cross_gap.AdjustGapSegmentStateRangesForFragmentation(
                    last_track_in_previous_fragment,
                    first_track_in_next_fragment,
                    &mut column_gaps_segment_ranges_start_indices[i],
                );
            }
        }
    }
}

impl PartialEq for GapGeometry {
    fn eq(&self, other: &Self) -> bool {
        self.Equals(other)
    }
}
