//! Pure BeginFrame admission and deadline policy.
//!
//! Keeping these decisions separate from task delivery makes scheduler stalls
//! attributable without moving clocks, callbacks, or ownership out of the
//! scheduler package.

use crate::{
    begin_frame::{BeginFrameArgs, BeginFrameArgsType},
    TimeDelta, TimeTicks,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissedBeginFrameDecision {
    Accept,
    DropPastDeadline,
}

pub fn DecideMissedBeginFrame(
    wait_for_all_pipeline_stages_before_draw: bool,
    frame_type: BeginFrameArgsType,
    deadline: TimeTicks,
    now: TimeTicks,
) -> MissedBeginFrameDecision {
    if !wait_for_all_pipeline_stages_before_draw
        && frame_type == BeginFrameArgsType::MISSED
        && deadline < now
    {
        MissedBeginFrameDecision::DropPastDeadline
    } else {
        MissedBeginFrameDecision::Accept
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BeginImplFrameDeadlinePlan {
    pub adjusted_args: BeginFrameArgs,
    pub begin_main_frame_to_activate_threshold: TimeDelta,
    pub critical_begin_main_frame_to_activate_is_fast: bool,
}

pub fn PlanBeginImplFrameDeadline(
    args: &BeginFrameArgs,
    deadline_draw_duration: TimeDelta,
    activation_threshold_draw_duration: TimeDelta,
    deadline_fudge_factor: TimeDelta,
    critical_begin_main_frame_to_activate_estimate: TimeDelta,
) -> BeginImplFrameDeadlinePlan {
    let mut adjusted_args = args.clone();
    adjusted_args.deadline -= deadline_draw_duration;
    adjusted_args.deadline -= deadline_fudge_factor;

    let begin_main_frame_to_activate_threshold =
        adjusted_args.interval - activation_threshold_draw_duration - deadline_fudge_factor;

    BeginImplFrameDeadlinePlan {
        adjusted_args,
        begin_main_frame_to_activate_threshold,
        critical_begin_main_frame_to_activate_is_fast:
            critical_begin_main_frame_to_activate_estimate < begin_main_frame_to_activate_threshold,
    }
}

pub fn ApplyInvalidationDelayFrames(
    threshold: TimeDelta,
    frame_interval: TimeDelta,
    delay_impl_invalidation_frames: i32,
) -> TimeDelta {
    threshold + frame_interval * delay_impl_invalidation_frames
}

pub fn ShouldDeferInvalidationForFastMainFrame(
    time_since_main_frame_sent: TimeDelta,
    ready_to_commit_estimate: TimeDelta,
    begin_main_frame_to_activate_threshold: TimeDelta,
) -> bool {
    if time_since_main_frame_sent > begin_main_frame_to_activate_threshold {
        return false;
    }
    ready_to_commit_estimate - time_since_main_frame_sent < begin_main_frame_to_activate_threshold
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::begin_frame::BeginFrameId;

    fn args() -> BeginFrameArgs {
        BeginFrameArgs {
            frame_time: TimeTicks::FromMicroseconds(1_000),
            deadline: TimeTicks::FromMicroseconds(17_000),
            interval: TimeDelta::Microseconds(16_000),
            frame_id: BeginFrameId {
                source_id: 7,
                sequence_number: 11,
            },
            type_: BeginFrameArgsType::NORMAL,
            ..BeginFrameArgs::default()
        }
    }

    #[test]
    fn drops_only_expired_missed_frames_when_pipeline_wait_is_disabled() {
        assert_eq!(
            DecideMissedBeginFrame(
                false,
                BeginFrameArgsType::MISSED,
                TimeTicks::FromMicroseconds(10),
                TimeTicks::FromMicroseconds(11),
            ),
            MissedBeginFrameDecision::DropPastDeadline
        );
        assert_eq!(
            DecideMissedBeginFrame(
                true,
                BeginFrameArgsType::MISSED,
                TimeTicks::FromMicroseconds(10),
                TimeTicks::FromMicroseconds(11),
            ),
            MissedBeginFrameDecision::Accept
        );
        assert_eq!(
            DecideMissedBeginFrame(
                false,
                BeginFrameArgsType::NORMAL,
                TimeTicks::FromMicroseconds(10),
                TimeTicks::FromMicroseconds(11),
            ),
            MissedBeginFrameDecision::Accept
        );
    }

    #[test]
    fn deadline_plan_accounts_for_draw_cost_and_fudge() {
        let plan = PlanBeginImplFrameDeadline(
            &args(),
            TimeDelta::Microseconds(2_000),
            TimeDelta::Microseconds(3_000),
            TimeDelta::Microseconds(1_000),
            TimeDelta::Microseconds(11_999),
        );

        assert_eq!(
            plan.adjusted_args.deadline,
            TimeTicks::FromMicroseconds(14_000)
        );
        assert_eq!(
            plan.begin_main_frame_to_activate_threshold,
            TimeDelta::Microseconds(12_000)
        );
        assert!(plan.critical_begin_main_frame_to_activate_is_fast);
        assert_eq!(plan.adjusted_args.frame_id, args().frame_id);
    }

    #[test]
    fn critical_estimate_equal_to_threshold_is_not_fast() {
        let plan = PlanBeginImplFrameDeadline(
            &args(),
            TimeDelta::default(),
            TimeDelta::Microseconds(3_000),
            TimeDelta::Microseconds(1_000),
            TimeDelta::Microseconds(12_000),
        );
        assert!(!plan.critical_begin_main_frame_to_activate_is_fast);
    }

    #[test]
    fn invalidation_delay_extends_response_window_by_frame_count() {
        assert_eq!(
            ApplyInvalidationDelayFrames(
                TimeDelta::Microseconds(10_000),
                TimeDelta::Microseconds(16_000),
                2,
            ),
            TimeDelta::Microseconds(42_000)
        );
    }

    #[test]
    fn fast_main_frame_must_still_fit_inside_remaining_window() {
        let threshold = TimeDelta::Microseconds(12_000);
        assert!(ShouldDeferInvalidationForFastMainFrame(
            TimeDelta::Microseconds(2_000),
            TimeDelta::Microseconds(8_000),
            threshold,
        ));
        assert!(!ShouldDeferInvalidationForFastMainFrame(
            TimeDelta::Microseconds(13_000),
            TimeDelta::Microseconds(1_000),
            threshold,
        ));
        assert!(!ShouldDeferInvalidationForFastMainFrame(
            TimeDelta::Microseconds(2_000),
            TimeDelta::Microseconds(14_000),
            threshold,
        ));
    }
}
