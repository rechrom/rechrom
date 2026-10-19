// Source: components/viz/service/surfaces/surface_dependency_deadline.h:1-60
// Source: components/viz/service/surfaces/surface_dependency_deadline.cc:1-48
#![allow(non_snake_case)]

use crate::common::quads::frame_deadline::FrameDeadline;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Default)]
pub struct SurfaceDependencyDeadline {
    start_time_: Option<Instant>,
    deadline_: Option<Instant>,
}
impl SurfaceDependencyDeadline {
    pub fn Set(&mut self, deadline: FrameDeadline, default_frames: Option<u32>) {
        self.Cancel();
        self.start_time_ = deadline.frame_start_time();
        self.deadline_ = deadline.ToWallTime(default_frames);
    }
    pub fn has_deadline(&self) -> bool {
        self.deadline_.is_some()
    }
    pub fn HasDeadlinePassedAt(&self, now: Instant) -> bool {
        self.deadline_.is_some_and(|deadline| now >= deadline)
    }
    pub fn CancelAt(&mut self, now: Instant) -> Option<Duration> {
        self.deadline_.take()?;
        self.start_time_
            .take()
            .map(|start| now.saturating_duration_since(start))
    }
    pub fn Cancel(&mut self) -> Option<Duration> {
        self.CancelAt(Instant::now())
    }
}
