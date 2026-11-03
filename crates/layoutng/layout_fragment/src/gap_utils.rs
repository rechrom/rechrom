#![allow(non_snake_case, non_upper_case_globals)]

use foundation::{HashMap, IntWithZeroKeyHashTraits, String, Vector};
use layoutng_style::style::grid_area::GridSpan;

use super::super::cross_gap::CrossGap;
use super::super::main_gap::MainGap;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellState {
    kEmpty = 0,
    kOccupied = 1,
    kSpanner = 2,
}

pub type CellStates = Vector<CellState>;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GapSegmentStateId {
    kNone = 0,
    kEmptyBefore = 1,
    kEmptyAfter = 2,
    kBlocked = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GapSegmentState {
    pub status_: u32,
}

impl Default for GapSegmentState {
    fn default() -> Self {
        Self {
            status_: Self::kEmptyBoth,
        }
    }
}

impl GapSegmentState {
    pub const kNone: GapSegmentStateId = GapSegmentStateId::kNone;
    pub const kEmptyBefore: GapSegmentStateId = GapSegmentStateId::kEmptyBefore;
    pub const kEmptyAfter: GapSegmentStateId = GapSegmentStateId::kEmptyAfter;
    pub const kBlocked: GapSegmentStateId = GapSegmentStateId::kBlocked;
    pub const kEmptyBoth: u32 =
        GapSegmentStateId::kEmptyBefore as u32 | GapSegmentStateId::kEmptyAfter as u32;

    pub fn new(status: u32) -> Self {
        Self { status_: status }
    }

    pub fn HasGapStatus(&self, status: GapSegmentStateId) -> bool {
        self.status_ & status as u32 != 0
    }

    pub fn HasEmptyStatus(&self) -> bool {
        self.HasGapStatus(Self::kEmptyBefore) || self.HasGapStatus(Self::kEmptyAfter)
    }

    pub fn IsEmpty(&self) -> bool {
        self.status_ == Self::kEmptyBoth
    }

    pub fn OrAssignState(&mut self, other: &Self) -> &mut Self {
        self.status_ |= other.status_;
        self
    }

    pub fn OrAssignStatus(&mut self, status: GapSegmentStateId) -> &mut Self {
        self.status_ |= status as u32;
        self
    }

    pub fn ToString(&self) -> String {
        if self.status_ == Self::kNone as u32 {
            return String::from("NONE");
        } else {
            if self.HasGapStatus(Self::kEmptyBefore) {
                return String::from("EMPTY_BEFORE ");
            }
            if self.HasGapStatus(Self::kEmptyAfter) {
                return String::from("EMPTY_AFTER ");
            }
            if self.HasGapStatus(Self::kBlocked) {
                return String::from("BLOCKED ");
            }
        }
        String::from("UNKNOWN")
    }
}

impl std::ops::BitOrAssign<Self> for GapSegmentState {
    fn bitor_assign(&mut self, other: Self) {
        self.OrAssignState(&other);
    }
}

impl std::ops::BitOrAssign<GapSegmentStateId> for GapSegmentState {
    fn bitor_assign(&mut self, status: GapSegmentStateId) {
        self.OrAssignStatus(status);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GapSegmentStateRange {
    pub start: u32,
    pub end: u32,
    pub state: GapSegmentState,
}

pub type GapSegmentStateRanges = Vector<GapSegmentStateRange>;

pub struct GapSegmentStateCursor<'a> {
    ranges_: Option<&'a GapSegmentStateRanges>,
    range_index_: u32,
    current_gap_index_: u32,
}

impl<'a> GapSegmentStateCursor<'a> {
    pub fn new(ranges: Option<&'a GapSegmentStateRanges>) -> Self {
        Self {
            ranges_: ranges,
            range_index_: 0,
            current_gap_index_: 0,
        }
    }

    pub fn GetNextGapSegmentState(&mut self) -> GapSegmentState {
        let Some(ranges) = self.ranges_ else {
            return GapSegmentState::new(GapSegmentState::kNone as u32);
        };
        while (self.range_index_ as usize) < ranges.len()
            && ranges[self.range_index_ as usize].end <= self.current_gap_index_
        {
            self.range_index_ = self.range_index_.wrapping_add(1);
        }
        if (self.range_index_ as usize) < ranges.len() {
            let range = &ranges[self.range_index_ as usize];
            if range.start <= self.current_gap_index_ && self.current_gap_index_ < range.end {
                self.current_gap_index_ = self.current_gap_index_.wrapping_add(1);
                return range.state;
            }
        }
        self.current_gap_index_ = self.current_gap_index_.wrapping_add(1);
        GapSegmentState::new(GapSegmentState::kNone as u32)
    }
}

pub struct GapSegmentStateAggregator {
    pub(crate) cell_count_: u32,
    pub(crate) track_to_cell_states_: HashMap<u32, CellStates, IntWithZeroKeyHashTraits<i32>>,
}

impl Default for GapSegmentStateAggregator {
    fn default() -> Self {
        Self::new(0)
    }
}

impl GapSegmentStateAggregator {
    pub fn new(cell_count: u32) -> Self {
        Self {
            cell_count_: cell_count,
            track_to_cell_states_: HashMap::default(),
        }
    }

    pub fn GetCellCount(&self) -> u32 {
        self.cell_count_
    }
}

// Chromium's aggregation algorithm is part of
// core/layout/gap/gap_utils.cc:12-125 and is compiled here at the shared
// assembly boundary.
pub trait GapSegmentStateAggregatorOps {
    fn ProcessItem(&mut self, primary_span: &GridSpan, secondary_span: &GridSpan);
    fn FinalizeMainGapSegmentStateRangesFor(&self, gap: &mut MainGap, track_index: u32);
    fn FinalizeCrossGapSegmentStateRangesFor(&self, gap: &mut CrossGap, track_index: u32);
}

pub trait GapSegmentStateAggregatorInternal {
    fn UpdateGapStateFor(&mut self, track_index: u32, secondary_span: &GridSpan, state: CellState);
}
