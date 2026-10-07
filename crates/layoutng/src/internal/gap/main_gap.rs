#![allow(non_snake_case)]

use foundation::{LayoutUnit, StrCat, String};

use super::cross_gap::CrossGapRange;
use super::gap_utils::{GapSegmentState, GapSegmentStateRange, GapSegmentStateRanges};

// cpp: layoutng/internal/gap/main_gap.h:15-19
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpannerMainGapType {
    kStart,
    kEnd,
    #[default]
    kNone,
}

// cpp: layoutng/internal/gap/main_gap.h:25-30
// cpp: layoutng/internal/gap/main_gap.h:94-130
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
    // cpp: layoutng/internal/gap/main_gap.h:28-30
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

    // cpp: layoutng/internal/gap/main_gap.h:32-42
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

    // cpp: layoutng/internal/gap/main_gap.h:44-44
    pub fn GetGapOffset(&self) -> LayoutUnit {
        self.gap_offset_
    }

    // cpp: layoutng/internal/gap/main_gap.h:46-52
    pub fn HasCrossGapsBefore(&self) -> bool {
        self.range_of_cross_gaps_before_.IsValid()
    }

    pub fn HasCrossGapsAfter(&self) -> bool {
        self.range_of_cross_gaps_after_.IsValid()
    }

    // cpp: layoutng/internal/gap/main_gap.h:54-54
    // cpp: layoutng/internal/gap/main_gap.cc:23-26
    pub fn GetCrossGapBeforeStart(&self) -> u32 {
        assert!(self.HasCrossGapsBefore());
        self.range_of_cross_gaps_before_.Start()
    }

    // cpp: layoutng/internal/gap/main_gap.h:55-55
    // cpp: layoutng/internal/gap/main_gap.cc:28-31
    pub fn GetCrossGapBeforeEnd(&self) -> u32 {
        assert!(self.HasCrossGapsBefore());
        self.range_of_cross_gaps_before_.End()
    }

    // cpp: layoutng/internal/gap/main_gap.h:56-56
    // cpp: layoutng/internal/gap/main_gap.cc:33-38
    pub fn GetCrossGapBeforeCount(&self) -> u32 {
        let start = self.GetCrossGapBeforeStart();
        let end = self.GetCrossGapBeforeEnd();
        assert!(start <= end);
        (end - start).wrapping_add(1)
    }

    // cpp: layoutng/internal/gap/main_gap.h:57-57
    // cpp: layoutng/internal/gap/main_gap.cc:40-43
    pub fn GetCrossGapAfterStart(&self) -> u32 {
        assert!(self.HasCrossGapsAfter());
        self.range_of_cross_gaps_after_.Start()
    }

    // cpp: layoutng/internal/gap/main_gap.h:58-58
    // cpp: layoutng/internal/gap/main_gap.cc:45-48
    pub fn GetCrossGapAfterEnd(&self) -> u32 {
        assert!(self.HasCrossGapsAfter());
        self.range_of_cross_gaps_after_.End()
    }

    // cpp: layoutng/internal/gap/main_gap.h:59-59
    // cpp: layoutng/internal/gap/main_gap.cc:50-55
    pub fn GetCrossGapAfterCount(&self) -> u32 {
        let start = self.GetCrossGapAfterStart();
        let end = self.GetCrossGapAfterEnd();
        assert!(start <= end);
        (end - start).wrapping_add(1)
    }

    // cpp: layoutng/internal/gap/main_gap.h:61-67
    pub fn IncrementRangeOfCrossGapsBefore(&mut self, cross_gap_index: u32) {
        self.range_of_cross_gaps_before_.Increment(cross_gap_index);
    }

    pub fn IncrementRangeOfCrossGapsAfter(&mut self, cross_gap_index: u32) {
        self.range_of_cross_gaps_after_.Increment(cross_gap_index);
    }

    // cpp: layoutng/internal/gap/main_gap.h:71-79
    pub fn IsStartSpannerMainGap(&self) -> bool {
        self.spanner_main_gap_type_ == SpannerMainGapType::kStart
    }

    pub fn IsEndSpannerMainGap(&self) -> bool {
        self.spanner_main_gap_type_ == SpannerMainGapType::kEnd
    }

    pub fn IsSpannerMainGap(&self) -> bool {
        self.spanner_main_gap_type_ != SpannerMainGapType::kNone
    }

    // cpp: layoutng/internal/gap/main_gap.h:81-87
    pub fn HasGapSegmentStateRanges(&self) -> bool {
        self.gap_segment_state_ranges_.is_some()
    }

    pub fn HasBlockedRange(&self) -> bool {
        self.has_blocked_range_
    }

    // cpp: layoutng/internal/gap/main_gap.h:89-89
    // cpp: layoutng/internal/gap/main_gap.cc:57-60
    pub fn GetGapSegmentStateRanges(&self) -> &GapSegmentStateRanges {
        self.gap_segment_state_ranges_
            .as_ref()
            .expect("main gap has no segment state ranges")
    }

    // cpp: layoutng/internal/gap/main_gap.h:91-92
    // cpp: layoutng/internal/gap/main_gap.cc:12-21
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

    // cpp: layoutng/internal/gap/main_gap.h:69-69
    // cpp: layoutng/internal/gap/main_gap_string.cc:6-13
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
