// Source: components/viz/service/display/display_scheduler_base.h:1-93
// Source: components/viz/service/display/display_scheduler_base.cc:1-42
#![allow(non_snake_case)]

use super::frame_deadline_decider::PossibleDeadline;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct DrawAndSwapParams {
    pub frame_time: Instant,
    pub interval: Duration,
    pub expected_display_time: Instant,
    pub max_pending_swaps: usize,
    pub selected_deadline: Option<PossibleDeadline>,
}
pub trait DisplaySchedulerClient {
    fn DrawAndSwap(&mut self, params: &DrawAndSwapParams) -> bool;
    fn DidFinishFrame(&mut self, _result: DisplaySchedulerDrawResult) {}
    fn GetCurrentAllocatedBuffers(&self) -> usize {
        0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplaySchedulerDrawResult {
    kDrawn,
    kDidNotDraw,
    kNotVisible,
    kOutputSurfaceLost,
    kSwapsThrottled,
}
