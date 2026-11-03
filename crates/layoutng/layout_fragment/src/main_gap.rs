#![allow(non_snake_case)]

use foundation::{LayoutUnit, StrCat, String};

use super::super::cross_gap::CrossGapRange;
use super::super::gap_utils::{GapSegmentState, GapSegmentStateRange, GapSegmentStateRanges};

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpannerMainGapType {
    kStart,
    kEnd,
    #[default]
    kNone,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MainGap {
    gap_offset_: LayoutUnit,
    range_of_cross_gaps_before_: CrossGapRange,
    range_of_cross_gaps_after_: CrossGapRange,
    gap_segment_state_ranges_: Option<GapSegmentStateRanges>,
    has_blocked_range_: bool,
    spanner_main_gap_type_: SpannerMainGapType,
}

impl MainGap {
    pub fn new(offset: LayoutUnit) -> Self {
        Self::with_spanner_type(offset, SpannerMainGapType::kNone)
    }

    pub fn with_spanner_type(
        offset: LayoutUnit,
        spanner_main_gap_type: SpannerMainGapType,
    ) -> Self {
        Self {
            gap_offset_: offset,
            spanner_main_gap_type_: spanner_main_gap_type,
            ..Self::default()
        }
    }

    pub fn with_new_offset(other: &Self, new_offset: LayoutUnit) -> Self {
        Self {
            gap_offset_: new_offset,
            range_of_cross_gaps_before_: other.range_of_cross_gaps_before_,
            range_of_cross_gaps_after_: other.range_of_cross_gaps_after_,
            gap_segment_state_ranges_: other.gap_segment_state_ranges_.clone(),
            has_blocked_range_: other.has_blocked_range_,
            spanner_main_gap_type_: other.spanner_main_gap_type_,
        }
    }

    pub fn GetGapOffset(&self) -> LayoutUnit {
        self.gap_offset_
    }

    pub fn HasCrossGapsBefore(&self) -> bool {
        self.range_of_cross_gaps_before_.IsValid()
    }

    pub fn HasCrossGapsAfter(&self) -> bool {
        self.range_of_cross_gaps_after_.IsValid()
    }

    pub fn GetCrossGapBeforeStart(&self) -> u32 {
        assert!(self.HasCrossGapsBefore());
        self.range_of_cross_gaps_before_.Start()
    }

    pub fn GetCrossGapBeforeEnd(&self) -> u32 {
        assert!(self.HasCrossGapsBefore());
        self.range_of_cross_gaps_before_.End()
    }

    pub fn GetCrossGapBeforeCount(&self) -> u32 {
        let start = self.GetCrossGapBeforeStart();
        let end = self.GetCrossGapBeforeEnd();
        assert!(start <= end);
        (end - start).wrapping_add(1)
    }

    pub fn GetCrossGapAfterStart(&self) -> u32 {
        assert!(self.HasCrossGapsAfter());
        self.range_of_cross_gaps_after_.Start()
    }

    pub fn GetCrossGapAfterEnd(&self) -> u32 {
        assert!(self.HasCrossGapsAfter());
        self.range_of_cross_gaps_after_.End()
    }

    pub fn GetCrossGapAfterCount(&self) -> u32 {
        let start = self.GetCrossGapAfterStart();
        let end = self.GetCrossGapAfterEnd();
        assert!(start <= end);
        (end - start).wrapping_add(1)
    }

    pub fn IncrementRangeOfCrossGapsBefore(&mut self, cross_gap_index: u32) {
        self.range_of_cross_gaps_before_.Increment(cross_gap_index);
    }

    pub fn IncrementRangeOfCrossGapsAfter(&mut self, cross_gap_index: u32) {
        self.range_of_cross_gaps_after_.Increment(cross_gap_index);
    }

    pub fn IsStartSpannerMainGap(&self) -> bool {
        self.spanner_main_gap_type_ == SpannerMainGapType::kStart
    }

    pub fn IsEndSpannerMainGap(&self) -> bool {
        self.spanner_main_gap_type_ == SpannerMainGapType::kEnd
    }

    pub fn IsSpannerMainGap(&self) -> bool {
        self.spanner_main_gap_type_ != SpannerMainGapType::kNone
    }

    pub fn HasGapSegmentStateRanges(&self) -> bool {
        self.gap_segment_state_ranges_.is_some()
    }

    pub fn HasBlockedRange(&self) -> bool {
        self.has_blocked_range_
    }

    pub fn GetGapSegmentStateRanges(&self) -> &GapSegmentStateRanges {
        self.gap_segment_state_ranges_
            .as_ref()
            .expect("main gap has no segment state ranges")
    }

    pub fn AddGapSegmentStateRange(&mut self, range: &GapSegmentStateRange) {
        if !self.HasGapSegmentStateRanges() {
            self.gap_segment_state_ranges_ = Some(GapSegmentStateRanges::default());
        }
        if range.state.HasGapStatus(GapSegmentState::kBlocked) {
            self.has_blocked_range_ = true;
        }
        self.gap_segment_state_ranges_
            .as_mut()
            .unwrap()
            .push(*range);
    }

    pub fn ToString(&self, verbose: bool) -> String {
        if verbose {
            return StrCat(&[
                String::from("MainOffset("),
                self.gap_offset_.ToString().into(),
                String::from("); "),
                String::from("Before: "),
                self.range_of_cross_gaps_before_.ToString(),
                String::from(";"),
                String::from("After: "),
                self.range_of_cross_gaps_after_.ToString(),
                String::from(";"),
            ]);
        }
        StrCat(&[
            String::from("MainOffset("),
            self.gap_offset_.ToString().into(),
            String::from("); "),
        ])
    }
}
