#![allow(non_snake_case)]

use foundation::{LayoutUnit, StrCat, String};
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_style::style::grid_enums::GridTrackSizingDirection;

use super::gap_utils::{GapSegmentStateRange, GapSegmentStateRanges};

// cpp: layoutng/internal/gap/cross_gap.h:20-54
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CrossGapRange {
    start_index_: Option<u32>,
    end_index_: Option<u32>,
}

impl CrossGapRange {
    // cpp: layoutng/internal/gap/cross_gap.h:22-23
    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start_index_: Some(start),
            end_index_: Some(end),
        }
    }

    // cpp: layoutng/internal/gap/cross_gap.h:27-29
    pub fn IsValid(&self) -> bool {
        self.start_index_.is_some() && self.end_index_.is_some()
    }

    // cpp: layoutng/internal/gap/cross_gap.h:31-34
    pub fn Start(&self) -> u32 {
        self.start_index_.expect("cross gap range has no start")
    }

    // cpp: layoutng/internal/gap/cross_gap.h:36-39
    pub fn End(&self) -> u32 {
        self.end_index_.expect("cross gap range has no end")
    }

    // cpp: layoutng/internal/gap/cross_gap.h:41-43
    // cpp: layoutng/internal/gap/cross_gap.cc:9-22
    pub fn Increment(&mut self, cross_gap_index: u32) {
        if self.start_index_.is_none() {
            self.start_index_ = Some(cross_gap_index);
            self.end_index_ = Some(cross_gap_index);
        } else {
            let end = self.end_index_.expect("cross gap range has no end");
            assert!(cross_gap_index > end);
            assert!(cross_gap_index > self.start_index_.unwrap());
            self.end_index_ = Some(cross_gap_index);
        }
    }

    // cpp: layoutng/internal/gap/cross_gap.h:45-45
    // cpp: layoutng/internal/gap/cross_gap_string.cc:6-11
    pub fn ToString(&self) -> String {
        let start = self
            .start_index_
            .map_or_else(|| String::from("null"), String::Number);
        let end = self
            .end_index_
            .map_or_else(|| String::from("null"), String::Number);
        StrCat(&[
            String::from("("),
            start,
            String::from(" --> "),
            end,
            String::from(")"),
        ])
    }
}

// cpp: layoutng/internal/gap/cross_gap.h:69-74
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EdgeIntersectionState {
    #[default]
    kNone = 0,
    kStart = 1,
    kEnd = 2,
    kBoth = 3,
}

// cpp: layoutng/internal/gap/cross_gap.h:62-142
#[derive(Clone, Debug, PartialEq)]
pub struct CrossGap {
    gap_logical_offset_: LogicalOffset,
    edge_state_: EdgeIntersectionState,
    gap_segment_state_ranges_: Option<GapSegmentStateRanges>,
}

impl CrossGap {
    // cpp: layoutng/internal/gap/cross_gap.h:76-78
    pub fn new(offset: LogicalOffset) -> Self {
        Self {
            gap_logical_offset_: offset,
            edge_state_: EdgeIntersectionState::kNone,
            gap_segment_state_ranges_: None,
        }
    }

    pub fn with_edge_state(offset: LogicalOffset, state: EdgeIntersectionState) -> Self {
        Self {
            gap_logical_offset_: offset,
            edge_state_: state,
            gap_segment_state_ranges_: None,
        }
    }

    // cpp: layoutng/internal/gap/cross_gap.h:80-80
    pub fn GetGapOffset(&self) -> LogicalOffset {
        self.gap_logical_offset_
    }

    // cpp: layoutng/internal/gap/cross_gap.h:82-85
    pub fn GetGapOffsetForDirection(&self, direction: GridTrackSizingDirection) -> LayoutUnit {
        if direction == GridTrackSizingDirection::kForColumns {
            self.gap_logical_offset_.inline_offset
        } else {
            self.gap_logical_offset_.block_offset
        }
    }

    // cpp: layoutng/internal/gap/cross_gap.h:89-92
    pub fn SetEdgeIntersectionState(&mut self, state: EdgeIntersectionState) {
        self.edge_state_ = state;
    }

    pub fn GetEdgeIntersectionState(&self) -> EdgeIntersectionState {
        self.edge_state_
    }

    // cpp: layoutng/internal/gap/cross_gap.h:94-96
    pub fn EndsAtEdge(&self) -> bool {
        self.edge_state_ == EdgeIntersectionState::kEnd
            || self.edge_state_ == EdgeIntersectionState::kBoth
    }

    // cpp: layoutng/internal/gap/cross_gap.h:98-105
    pub fn HasGapSegmentStateRanges(&self) -> bool {
        self.gap_segment_state_ranges_.is_some()
    }

    pub fn GetGapSegmentStateRanges(&self) -> &GapSegmentStateRanges {
        self.gap_segment_state_ranges_
            .as_ref()
            .expect("cross gap has no segment state ranges")
    }

    // cpp: layoutng/internal/gap/cross_gap.h:107-108
    // cpp: layoutng/internal/gap/cross_gap.cc:28-34
    pub fn AddGapSegmentStateRange(&mut self, range: &GapSegmentStateRange) {
        if !self.HasGapSegmentStateRanges() {
            self.gap_segment_state_ranges_ = Some(GapSegmentStateRanges::default());
        }
        self.gap_segment_state_ranges_
            .as_mut()
            .unwrap()
            .push(*range);
    }

    // cpp: layoutng/internal/gap/cross_gap.h:127-130
    // cpp: layoutng/internal/gap/cross_gap.cc:36-75
    pub fn AdjustGapSegmentStateRangesForFragmentation(
        &mut self,
        last_track_in_previous_fragment: u32,
        first_track_in_next_fragment: u32,
        range_start_idx: &mut u32,
    ) {
        assert!(self.HasGapSegmentStateRanges());
        let mut adjusted_ranges = GapSegmentStateRanges::default();
        let ranges = self.gap_segment_state_ranges_.as_ref().unwrap();
        while (*range_start_idx as usize) < ranges.len() {
            let range = &ranges[*range_start_idx as usize];
            if range.start > first_track_in_next_fragment {
                break;
            }
            let adjusted_start = if range.start > last_track_in_previous_fragment {
                range.start - last_track_in_previous_fragment
            } else {
                0
            };
            assert!(range.end > last_track_in_previous_fragment);
            let adjusted_end = range.end - last_track_in_previous_fragment;
            adjusted_ranges.push(GapSegmentStateRange {
                start: adjusted_start,
                end: adjusted_end,
                state: range.state,
            });
            *range_start_idx = (*range_start_idx).wrapping_add(1);
        }
        if !adjusted_ranges.is_empty()
            && *range_start_idx > 0
            && ranges[(*range_start_idx).wrapping_sub(1) as usize].end
                > first_track_in_next_fragment
        {
            *range_start_idx -= 1;
        }
        self.gap_segment_state_ranges_ = Some(adjusted_ranges);
    }

    // cpp: layoutng/internal/gap/cross_gap.h:87-87
    // cpp: layoutng/internal/gap/cross_gap_string.cc:13-54
    pub fn ToString(&self, verbose: bool) -> String {
        if verbose {
            let edge_state = if self.edge_state_ == EdgeIntersectionState::kStart {
                String::from("kStart")
            } else if self.edge_state_ == EdgeIntersectionState::kEnd {
                String::from("kEnd")
            } else if self.edge_state_ == EdgeIntersectionState::kBoth {
                String::from("kBoth")
            } else {
                String::from("kNone")
            };

            let mut segment_state_ranges_str = String::default();
            if let Some(ranges) = &self.gap_segment_state_ranges_ {
                segment_state_ranges_str = String::from("[");
                for range in ranges {
                    segment_state_ranges_str = StrCat(&[
                        segment_state_ranges_str,
                        String::from("["),
                        String::Number(range.start),
                        String::from(", "),
                        String::Number(range.end),
                        String::from(") "),
                        range.state.ToString(),
                        String::from(", "),
                    ]);
                }
                segment_state_ranges_str = StrCat(&[segment_state_ranges_str, String::from("]")]);
            }
            return StrCat(&[
                String::from("CrossStartOffset("),
                self.gap_logical_offset_.inline_offset.ToString().into(),
                String::from(", "),
                self.gap_logical_offset_.block_offset.ToString().into(),
                String::from("); "),
                String::from("EdgeState: "),
                edge_state,
                String::from("; "),
                String::from("SegmentStateRanges: "),
                segment_state_ranges_str,
            ]);
        }
        StrCat(&[
            String::from("CrossStartOffset("),
            self.gap_logical_offset_.inline_offset.ToString().into(),
            String::from(", "),
            self.gap_logical_offset_.block_offset.ToString().into(),
            String::from(")"),
        ])
    }
}
