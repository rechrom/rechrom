#![allow(non_snake_case)]

use foundation::LayoutUnit;

use super::gap_utils::GapSegmentState;

// cpp: layoutng/internal/gap/gap_intersection.h:23-27
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverlapWindowState {
    #[default]
    kNone,
    kWindowOpen,
    kWindowClose,
}

// cpp: layoutng/internal/gap/gap_intersection.h:41-53
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExtraIntersectionState {
    pub is_above_main_gap: bool,
    pub overlap_state: OverlapWindowState,
    pub main_gap_index: Option<u32>,
}

// cpp: layoutng/internal/gap/gap_intersection.h:59-61
// cpp: layoutng/internal/gap/gap_intersection.h:135-145
#[derive(Clone, Copy, Debug)]
pub struct GapIntersection {
    offset_: LayoutUnit,
    extra_state_: Option<ExtraIntersectionState>,
    segment_state_: GapSegmentState,
}

impl Default for GapIntersection {
    fn default() -> Self {
        Self {
            offset_: LayoutUnit::default(),
            extra_state_: None,
            segment_state_: GapSegmentState::new(GapSegmentState::kNone as u32),
        }
    }
}

impl GapIntersection {
    // cpp: layoutng/internal/gap/gap_intersection.h:63-63
    pub fn new(offset: LayoutUnit) -> Self {
        Self {
            offset_: offset,
            ..Self::default()
        }
    }

    // cpp: layoutng/internal/gap/gap_intersection.h:65-66
    pub fn with_segment_state(offset: LayoutUnit, segment_state: GapSegmentState) -> Self {
        Self {
            offset_: offset,
            segment_state_: segment_state,
            ..Self::default()
        }
    }

    // cpp: layoutng/internal/gap/gap_intersection.h:68-72
    pub fn with_overlap_state(
        offset: LayoutUnit,
        state: OverlapWindowState,
        is_above_main_gap: bool,
    ) -> Self {
        Self {
            offset_: offset,
            extra_state_: Some(ExtraIntersectionState {
                is_above_main_gap: is_above_main_gap,
                overlap_state: state,
                main_gap_index: None,
            }),
            ..Self::default()
        }
    }

    // cpp: layoutng/internal/gap/gap_intersection.h:74-76
    pub fn with_side(offset: LayoutUnit, is_above_main_gap: bool) -> Self {
        Self {
            offset_: offset,
            extra_state_: Some(ExtraIntersectionState {
                is_above_main_gap: is_above_main_gap,
                ..ExtraIntersectionState::default()
            }),
            ..Self::default()
        }
    }

    // cpp: layoutng/internal/gap/gap_intersection.h:78-78
    pub fn GetOffset(&self) -> LayoutUnit {
        self.offset_
    }

    // cpp: layoutng/internal/gap/gap_intersection.h:80-83
    pub fn HasOverlapState(&self) -> bool {
        self.extra_state_
            .as_ref()
            .is_some_and(|extra| extra.overlap_state != OverlapWindowState::kNone)
    }

    // cpp: layoutng/internal/gap/gap_intersection.h:85-88
    pub fn IsOverlapWindowOpen(&self) -> bool {
        self.HasOverlapState()
            && self.extra_state_.as_ref().unwrap().overlap_state == OverlapWindowState::kWindowOpen
    }

    // cpp: layoutng/internal/gap/gap_intersection.h:90-93
    pub fn IsOverlapWindowClose(&self) -> bool {
        self.HasOverlapState()
            && self.extra_state_.as_ref().unwrap().overlap_state == OverlapWindowState::kWindowClose
    }

    // cpp: layoutng/internal/gap/gap_intersection.h:95-98
    pub fn IsAboveMainGap(&self) -> bool {
        self.extra_state_
            .as_ref()
            .expect("intersection has no extra state")
            .is_above_main_gap
    }

    // cpp: layoutng/internal/gap/gap_intersection.h:100-100
    pub fn SetOffset(&mut self, offset: LayoutUnit) {
        self.offset_ = offset;
    }

    // cpp: layoutng/internal/gap/gap_intersection.h:102-105
    pub fn SetOverlapState(&mut self, state: OverlapWindowState) {
        self.extra_state_
            .as_mut()
            .expect("intersection has no extra state")
            .overlap_state = state;
    }

    // cpp: layoutng/internal/gap/gap_intersection.h:107-110
    pub fn SetIsAboveMainGap(&mut self, is_above: bool) {
        self.extra_state_
            .as_mut()
            .expect("intersection has no extra state")
            .is_above_main_gap = is_above;
    }

    // cpp: layoutng/internal/gap/gap_intersection.h:112-115
    pub fn ResetOverlapState(&mut self) {
        self.extra_state_
            .as_mut()
            .expect("intersection has no extra state")
            .overlap_state = OverlapWindowState::kNone;
    }

    // cpp: layoutng/internal/gap/gap_intersection.h:117-119
    pub fn HasMainGapIndex(&self) -> bool {
        self.extra_state_
            .as_ref()
            .is_some_and(|extra| extra.main_gap_index.is_some())
    }

    // cpp: layoutng/internal/gap/gap_intersection.h:121-124
    pub fn GetMainGapIndex(&self) -> u32 {
        assert!(self.HasMainGapIndex());
        self.extra_state_.as_ref().unwrap().main_gap_index.unwrap()
    }

    // cpp: layoutng/internal/gap/gap_intersection.h:126-131
    pub fn SetMainGapIndex(&mut self, index: u32) {
        if self.extra_state_.is_none() {
            self.extra_state_ = Some(ExtraIntersectionState::default());
        }
        self.extra_state_.as_mut().unwrap().main_gap_index = Some(index);
    }

    // cpp: layoutng/internal/gap/gap_intersection.h:133-133
    pub fn SegmentState(&self) -> &GapSegmentState {
        &self.segment_state_
    }
}
