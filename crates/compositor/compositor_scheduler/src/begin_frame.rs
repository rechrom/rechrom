//! Viz-owned BeginFrame values at the compositor input boundary.
//! This is boundary data, not a translation of the Viz BeginFrameSource.

use crate::{trace_sink::SourceLocation, TimeDelta, TimeTicks};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PossibleDeadline {
    pub vsync_id: i64,
    pub latch_delta: TimeDelta,
    pub present_delta: TimeDelta,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PossibleDeadlines {
    pub os_preferred_index: usize,
    pub deadlines: Vec<PossibleDeadline>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct BeginFrameId {
    pub source_id: u64,
    pub sequence_number: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BeginFrameArgsType {
    #[default]
    INVALID,
    NORMAL,
    MISSED,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BeginFrameArgs {
    pub frame_time: TimeTicks,
    pub deadline: TimeTicks,
    pub interval: TimeDelta,
    pub unthrottled_interval: TimeDelta,
    pub frame_id: BeginFrameId,
    pub trace_id: i64,
    pub dispatch_time: TimeTicks,
    pub client_arrival_time: TimeTicks,
    pub type_: BeginFrameArgsType,
    pub on_critical_path: bool,
    pub animate_only: bool,
    pub frames_throttled_since_last: u64,
    pub created_from: Option<SourceLocation>,
    pub possible_deadlines: Option<PossibleDeadlines>,
}

impl Default for BeginFrameArgs {
    fn default() -> Self {
        Self {
            frame_time: TimeTicks::Min(),
            deadline: TimeTicks::Min(),
            interval: TimeDelta::Microseconds(-1),
            unthrottled_interval: TimeDelta::Microseconds(-1),
            frame_id: BeginFrameId::default(),
            trace_id: -1,
            dispatch_time: TimeTicks::default(),
            client_arrival_time: TimeTicks::default(),
            type_: BeginFrameArgsType::INVALID,
            on_critical_path: true,
            animate_only: false,
            frames_throttled_since_last: 0,
            created_from: None,
            possible_deadlines: None,
        }
    }
}

impl BeginFrameArgs {
    pub const kStartingSourceId: u64 = 0;
    pub const kManualSourceId: u64 = u32::MAX as u64;
    pub const kInvalidFrameNumber: u64 = 0;
    pub const kStartingFrameNumber: u64 = 1;

    pub fn IsValid(&self) -> bool {
        self.interval >= TimeDelta::default()
    }
    pub const fn DefaultInterval() -> TimeDelta {
        TimeDelta::Microseconds(1_000_000 / 60)
    }
    pub const fn MinInterval() -> TimeDelta {
        TimeDelta::Microseconds(0)
    }
}
