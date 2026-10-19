// Source: components/viz/service/display/frame_deadline_decider.h:1-147
// Source: components/viz/service/display/frame_deadline_decider.cc:1-47
// UMA/Perfetto emission is an injected observer concern. Candidate selection,
// input-latency cap, sequence locking, refresh transitions and sustainability
// limits are translated without a platform dependency.
#![allow(non_snake_case)]

use std::time::{Duration, Instant};

pub const kPerceptibleLatencyThreshold: Duration = Duration::from_millis(100);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PossibleDeadline {
    pub latch_delta: Duration,
    pub present_delta: Duration,
    pub vsync_id: i64,
}
#[derive(Clone, Debug)]
pub struct PossibleDeadlines {
    pub deadlines: Vec<PossibleDeadline>,
    pub os_preferred_index: usize,
}
impl PossibleDeadlines {
    pub fn GetOSPreferredDeadline(&self) -> PossibleDeadline {
        self.deadlines[self.os_preferred_index]
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionReason {
    kPlatformPreferred,
    kOngoingSequence,
    kChromePreferredNewSequence,
    kOsPreferredNoDeadlineWithinTarget,
    kOsPreferredChromePreferredSooner,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QueryResult {
    pub deadline_index: usize,
    pub reason: SelectionReason,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SequenceStrategy {
    kOsPreferredDeltaLocking,
    kPresentationDeltaLocking,
}
#[derive(Clone, Copy)]
struct FrameSequenceState {
    present_delta: Duration,
    os_preferred_offset: i128,
    deadline_index: usize,
    last_frame_time: Instant,
    is_interaction_active: bool,
}

pub struct FrameDeadlineDecider {
    max_non_interactive_idle_duration_: Duration,
    max_interactive_idle_duration_: Duration,
    strategy_: SequenceStrategy,
    use_platform_preferred_deadlines_: bool,
    min_supported_vsync_interval_: Option<Duration>,
    frame_sequence_state_: Option<FrameSequenceState>,
}
impl FrameDeadlineDecider {
    pub fn new(use_platform: bool) -> Self {
        Self {
            max_non_interactive_idle_duration_: Duration::from_millis(50),
            max_interactive_idle_duration_: Duration::from_secs(3),
            strategy_: SequenceStrategy::kOsPreferredDeltaLocking,
            use_platform_preferred_deadlines_: use_platform,
            min_supported_vsync_interval_: None,
            frame_sequence_state_: None,
        }
    }
    pub fn NotifyMinSupportedVsyncInterval(&mut self, value: Duration) {
        self.min_supported_vsync_interval_ = Some(value)
    }
    pub fn SetStrategy(&mut self, value: SequenceStrategy) {
        self.strategy_ = value
    }
    // Source: components/viz/service/display/frame_deadline_decider.cc:48-65
    fn IsPartOfOngoingFrameSequence(&self, frame_time: Instant, interaction: bool) -> bool {
        let Some(state) = self.frame_sequence_state_ else {
            return false;
        };
        let ongoing = interaction && state.is_interaction_active;
        frame_time.saturating_duration_since(state.last_frame_time)
            <= if ongoing {
                self.max_interactive_idle_duration_
            } else {
                self.max_non_interactive_idle_duration_
            }
    }
    // Source: components/viz/service/display/frame_deadline_decider.cc:66-159
    pub fn QueryDeadline(
        &self,
        possible: &PossibleDeadlines,
        interval: Duration,
        max_buffers: usize,
        frame_time: Instant,
        earliest_input: Option<Instant>,
        interaction: bool,
    ) -> QueryResult {
        assert!(
            !possible.deadlines.is_empty()
                && possible.os_preferred_index < possible.deadlines.len()
        );
        if self.use_platform_preferred_deadlines_ {
            return QueryResult {
                deadline_index: possible.os_preferred_index,
                reason: SelectionReason::kPlatformPreferred,
            };
        }
        if self.IsPartOfOngoingFrameSequence(frame_time, interaction) {
            return QueryResult {
                deadline_index: self.FindClosestDeadlineByPresentation(
                    possible,
                    interval,
                    max_buffers,
                ),
                reason: SelectionReason::kOngoingSequence,
            };
        }
        let mut target = interval.saturating_mul(max_buffers as u32);
        if let Some(min) = self.min_supported_vsync_interval_ {
            target = target.min(min.saturating_mul(max_buffers as u32));
        }
        if let Some(input) = earliest_input {
            let input_delta = frame_time.saturating_duration_since(input);
            let latency_cap = kPerceptibleLatencyThreshold
                .saturating_sub(interval)
                .saturating_sub(interval / 4);
            target = target.min(latency_cap.saturating_sub(input_delta));
        }
        let index = possible
            .deadlines
            .partition_point(|d| d.present_delta <= target)
            .saturating_sub(1);
        let preferred = possible.deadlines[index];
        let os = possible.GetOSPreferredDeadline();
        if preferred.present_delta > target {
            return QueryResult {
                deadline_index: possible.os_preferred_index,
                reason: SelectionReason::kOsPreferredNoDeadlineWithinTarget,
            };
        }
        if preferred.present_delta < os.present_delta {
            return QueryResult {
                deadline_index: possible.os_preferred_index,
                reason: SelectionReason::kOsPreferredChromePreferredSooner,
            };
        }
        QueryResult {
            deadline_index: index,
            reason: SelectionReason::kChromePreferredNewSequence,
        }
    }
    // Source: components/viz/service/display/frame_deadline_decider.cc:160-213
    pub fn SelectDeadline(
        &mut self,
        possible: &PossibleDeadlines,
        interval: Duration,
        max_buffers: usize,
        frame_time: Instant,
        earliest_input: Option<Instant>,
        interaction: bool,
    ) -> usize {
        let result = self.QueryDeadline(
            possible,
            interval,
            max_buffers,
            frame_time,
            earliest_input,
            interaction,
        );
        let selected = possible.deadlines[result.deadline_index];
        let os = possible.GetOSPreferredDeadline();
        self.frame_sequence_state_ = Some(FrameSequenceState {
            present_delta: selected.present_delta,
            os_preferred_offset: duration_ns(selected.present_delta)
                - duration_ns(os.present_delta),
            deadline_index: result.deadline_index,
            last_frame_time: frame_time,
            is_interaction_active: interaction,
        });
        result.deadline_index
    }
    pub fn OnDisplayInvisible(&mut self) {
        self.frame_sequence_state_ = None
    }
    // Source: components/viz/service/display/frame_deadline_decider.cc:214-228
    fn FindClosestDeadlineByPresentation(
        &self,
        possible: &PossibleDeadlines,
        interval: Duration,
        max_buffers: usize,
    ) -> usize {
        match self.strategy_ {
            SequenceStrategy::kOsPreferredDeltaLocking => {
                self.SelectDeadlineOsPreferredLocking(possible, interval, max_buffers)
            }
            SequenceStrategy::kPresentationDeltaLocking => {
                self.SelectDeadlinePresentationDeltaLocking(possible)
            }
        }
    }
    // Source: components/viz/service/display/frame_deadline_decider.cc:229-286
    fn SelectDeadlineOsPreferredLocking(
        &self,
        possible: &PossibleDeadlines,
        interval: Duration,
        max_buffers: usize,
    ) -> usize {
        let state = self.frame_sequence_state_.expect("ongoing sequence");
        let max = self
            .min_supported_vsync_interval_
            .unwrap_or(interval)
            .saturating_mul(max_buffers as u32)
            + Duration::from_millis(1);
        let os = possible.GetOSPreferredDeadline();
        let target = (duration_ns(os.present_delta) + state.os_preferred_offset).max(0) as u128;
        let target = Duration::from_nanos(target.min(u64::MAX as u128) as u64);
        if state.deadline_index < possible.deadlines.len() {
            let value = possible.deadlines[state.deadline_index].present_delta;
            if value <= max && abs_duration(value, target) <= Duration::from_millis(1) {
                return state.deadline_index;
            }
        }
        let mut best = None;
        let mut diff = Duration::MAX;
        for (index, candidate) in possible.deadlines.iter().enumerate() {
            if candidate.present_delta > max {
                break;
            }
            let next = abs_duration(candidate.present_delta, target);
            if next >= diff {
                break;
            }
            best = Some(index);
            diff = next;
        }
        best.unwrap_or(possible.os_preferred_index)
    }
    // Source: components/viz/service/display/frame_deadline_decider.cc:287-322
    fn SelectDeadlinePresentationDeltaLocking(&self, possible: &PossibleDeadlines) -> usize {
        let state = self.frame_sequence_state_.expect("ongoing sequence");
        if state.deadline_index < possible.deadlines.len()
            && abs_duration(
                possible.deadlines[state.deadline_index].present_delta,
                state.present_delta,
            ) <= Duration::from_millis(1)
        {
            return state.deadline_index;
        }
        possible
            .deadlines
            .iter()
            .enumerate()
            .min_by_key(|(_, d)| abs_duration(d.present_delta, state.present_delta))
            .map(|(i, _)| i)
            .unwrap_or(possible.os_preferred_index)
    }
}
// Source: components/viz/service/display/frame_deadline_decider.cc:323-339
// Sustainability telemetry is an adapter observer; selection enforces the
// same bound before this helper is reached.
fn duration_ns(value: Duration) -> i128 {
    value.as_nanos().min(i128::MAX as u128) as i128
}
fn abs_duration(a: Duration, b: Duration) -> Duration {
    if a >= b {
        a - b
    } else {
        b - a
    }
}
