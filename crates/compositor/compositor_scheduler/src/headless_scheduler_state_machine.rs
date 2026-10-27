// Translation status: complete; owned by compositor_scheduler.
// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use super::{
    scheduler_settings::SchedulerSettings,
    scheduler_state_machine::{SchedulerStateMachine, SchedulerStateMachineOverrides},
};

// Source: cc/scheduler/headless_scheduler_state_machine.h:13-27
pub struct HeadlessSchedulerStateMachine;
impl HeadlessSchedulerStateMachine {
    // Source: cc/scheduler/headless_scheduler_state_machine.cc:12-19
    pub fn new(settings: &SchedulerSettings) -> Self {
        debug_assert!(settings.wait_for_all_pipeline_stages_before_draw);
        Self
    }
}
impl SchedulerStateMachineOverrides for HeadlessSchedulerStateMachine {
    // Source: cc/scheduler/headless_scheduler_state_machine.cc:21-24
    fn CheckShouldDraw(&self, state: &SchedulerStateMachine) -> Option<bool> {
        Some(!state.active_tree_is_ready_to_draw_)
    }
    // Source: cc/scheduler/headless_scheduler_state_machine.cc:26-30
    fn ShouldPrepareTiles(&self, state: &SchedulerStateMachine) -> Option<bool> {
        Some(state.needs_prepare_tiles_)
    }
    // Source: cc/scheduler/headless_scheduler_state_machine.cc:32-44
    fn ShouldSubscribeToBeginFrames(&self, state: &SchedulerStateMachine) -> Option<bool> {
        if !state.HasInitializedLayerTreeFrameSink() {
            return Some(false);
        }
        Some(true)
    }
    // Source: cc/scheduler/headless_scheduler_state_machine.cc:48-51
    fn CheckShouldBlockDeadlineIndefinitely(&self, _state: &SchedulerStateMachine) -> Option<bool> {
        Some(false)
    }
    // Source: cc/scheduler/headless_scheduler_state_machine.cc:53-74
    fn ShouldTriggerBeginImplFrameDeadlineImmediately(
        &self,
        state: &SchedulerStateMachine,
    ) -> Option<bool> {
        if state.ShouldAbortCurrentFrame() && !state.has_pending_tree_ {
            return Some(true);
        }
        if state.IsDrawThrottled() {
            return Some(false);
        }
        if state.processing_animation_worklets_for_active_tree_ != 0 {
            return Some(false);
        }
        Some(true)
    }
    // Source: cc/scheduler/headless_scheduler_state_machine.cc:76-78
    fn CheckWillCommit(&self, state: &SchedulerStateMachine) -> Option<bool> {
        Some(!state.active_tree_needs_first_draw_)
    }
}
