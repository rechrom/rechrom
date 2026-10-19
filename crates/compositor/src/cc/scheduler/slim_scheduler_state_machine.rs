// Translation status: complete
// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use super::{
    scheduler_settings::SchedulerSettings,
    scheduler_state_machine::{
        Action, BeginImplFrameDeadlineMode, BeginImplFrameState, ForcedRedrawOnTimeoutState,
        SchedulerStateMachine, SchedulerStateMachineOverrides,
    },
};
use crate::adapter::begin_frame::BeginFrameArgs;
use std::{cell::RefCell, collections::BTreeSet};

// Source: cc/scheduler/slim_scheduler_state_machine.h:15-41
pub struct SlimSchedulerStateMachine {
    issued_actions_: RefCell<BTreeSet<Action>>,
}
impl SlimSchedulerStateMachine {
    // Source: cc/scheduler/slim_scheduler_state_machine.cc:13-19
    pub fn new(_settings: &SchedulerSettings) -> Self {
        Self {
            issued_actions_: RefCell::new(BTreeSet::new()),
        }
    }
    fn issued(&self, action: Action) -> bool {
        self.issued_actions_.borrow().contains(&action)
    }
}
impl SchedulerStateMachineOverrides for SlimSchedulerStateMachine {
    // Source: cc/scheduler/slim_scheduler_state_machine.cc:21-77
    fn NextAction(&self, state: &SchedulerStateMachine) -> Option<Action> {
        let mut action = Action::NONE;
        match state.begin_impl_frame_state_ {
            BeginImplFrameState::IDLE => {
                if !self.issued(Action::SEND_BEGIN_MAIN_FRAME) && state.ShouldSendBeginMainFrame() {
                    action = Action::SEND_BEGIN_MAIN_FRAME;
                } else if !self.issued(Action::POST_COMMIT) && state.ShouldRunPostCommit() {
                    action = Action::POST_COMMIT;
                } else if !self.issued(Action::ACTIVATE_SYNC_TREE) && state.ShouldActivateSyncTree()
                {
                    action = Action::ACTIVATE_SYNC_TREE;
                } else if !self.issued(Action::COMMIT) && state.ShouldCommit() {
                    action = Action::COMMIT;
                } else if !self.issued(Action::PREPARE_TILES)
                    && state.needs_prepare_tiles_
                    && !state.did_prepare_tiles_
                {
                    action = Action::PREPARE_TILES;
                } else if !self.issued(Action::BEGIN_LAYER_TREE_FRAME_SINK_CREATION)
                    && state.ShouldBeginLayerTreeFrameSinkCreation()
                {
                    action = Action::BEGIN_LAYER_TREE_FRAME_SINK_CREATION;
                }
            }
            BeginImplFrameState::INSIDE_BEGIN_FRAME | BeginImplFrameState::INSIDE_DEADLINE => {
                if !self.issued(Action::SEND_BEGIN_MAIN_FRAME) && state.ShouldSendBeginMainFrame() {
                    action = Action::SEND_BEGIN_MAIN_FRAME;
                } else if state.ShouldDraw() {
                    if !self.issued(Action::DRAW_ABORT) && state.PendingDrawsShouldBeAborted() {
                        action = Action::DRAW_ABORT;
                    } else if !self.issued(Action::DRAW_FORCED)
                        && state.forced_redraw_state_
                            == ForcedRedrawOnTimeoutState::WAITING_FOR_DRAW
                    {
                        action = Action::DRAW_FORCED;
                    } else if !self.issued(Action::DRAW_IF_POSSIBLE) {
                        action = Action::DRAW_IF_POSSIBLE;
                    }
                } else if !self.issued(Action::PERFORM_IMPL_SIDE_INVALIDATION)
                    && state.ShouldPerformImplSideInvalidation()
                {
                    action = Action::PERFORM_IMPL_SIDE_INVALIDATION;
                } else if !self.issued(Action::INVALIDATE_LAYER_TREE_FRAME_SINK)
                    && state.ShouldInvalidateLayerTreeFrameSink()
                {
                    action = Action::INVALIDATE_LAYER_TREE_FRAME_SINK;
                }
            }
        }
        self.issued_actions_.borrow_mut().insert(action);
        Some(action)
    }
    // Source: cc/scheduler/slim_scheduler_state_machine.cc:79-88
    fn CurrentBeginImplFrameDeadlineMode(
        &self,
        state: &SchedulerStateMachine,
    ) -> Option<BeginImplFrameDeadlineMode> {
        let mut deadline = state.CurrentBeginImplFrameDeadlineModeBase();
        if deadline == BeginImplFrameDeadlineMode::REGULAR
            || deadline == BeginImplFrameDeadlineMode::LATE
        {
            deadline = BeginImplFrameDeadlineMode::IMMEDIATE;
        }
        Some(deadline)
    }
    // Source: cc/scheduler/slim_scheduler_state_machine.cc:90-94
    fn OnBeginImplFrame(&self, state: &mut SchedulerStateMachine, args: &BeginFrameArgs) -> bool {
        self.issued_actions_.borrow_mut().clear();
        state.OnBeginImplFrameBase(args);
        true
    }
    // Source: cc/scheduler/slim_scheduler_state_machine.cc:96-100
    fn ShouldBlockBeginMainFrameWhenIdle(&self, _state: &SchedulerStateMachine) -> Option<bool> {
        Some(false)
    }
}
