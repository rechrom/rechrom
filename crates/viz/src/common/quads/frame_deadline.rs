// Source: components/viz/common/quads/frame_deadline.h:1-78
// Source: components/viz/common/quads/frame_deadline.cc:1-53
#![allow(non_snake_case)]

use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameDeadline {
    frame_start_time_: Option<Instant>,
    deadline_in_frames_: u32,
    frame_interval_: Duration,
    use_default_lower_bound_deadline_: bool,
}
impl Default for FrameDeadline {
    fn default() -> Self {
        Self {
            frame_start_time_: Some(Instant::now()),
            deadline_in_frames_: 0,
            frame_interval_: Duration::from_nanos(16_666_667),
            use_default_lower_bound_deadline_: true,
        }
    }
}
impl FrameDeadline {
    pub fn new(start: Instant, frames: u32, interval: Duration, use_default: bool) -> Self {
        Self {
            frame_start_time_: Some(start),
            deadline_in_frames_: frames,
            frame_interval_: interval,
            use_default_lower_bound_deadline_: use_default,
        }
    }
    pub const fn MakeZero() -> Self {
        Self {
            frame_start_time_: None,
            deadline_in_frames_: 0,
            frame_interval_: Duration::ZERO,
            use_default_lower_bound_deadline_: false,
        }
    }
    pub fn ToWallTime(self, default_frames: Option<u32>) -> Option<Instant> {
        let mut frames = self.deadline_in_frames_;
        if self.use_default_lower_bound_deadline_ {
            frames = frames.max(default_frames.unwrap_or(u32::MAX));
        }
        self.frame_start_time_?
            .checked_add(self.frame_interval_.saturating_mul(frames))
    }
    pub fn frame_start_time(self) -> Option<Instant> {
        self.frame_start_time_
    }
    pub fn deadline_in_frames(self) -> u32 {
        self.deadline_in_frames_
    }
    pub fn frame_interval(self) -> Duration {
        self.frame_interval_
    }
    pub fn use_default_lower_bound_deadline(self) -> bool {
        self.use_default_lower_bound_deadline_
    }
    pub fn IsZero(self) -> bool {
        self.deadline_in_frames_ == 0 && !self.use_default_lower_bound_deadline_
    }
    pub fn ToString(self) -> String {
        format!(
            "FrameDeadline(deadline in frames: {}, frame interval: {:?})",
            if self.use_default_lower_bound_deadline_ {
                "unresolved".into()
            } else {
                self.deadline_in_frames_.to_string()
            },
            self.frame_interval_
        )
    }
}
