// Translation status: complete; owned by compositor_scheduler.
// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use super::{
    scheduler_settings::SchedulerSettings,
    scheduler_state_machine::{
        BeginImplFrameDeadlineMode, BeginImplFrameState, SchedulerStateMachine,
        SchedulerStateMachineOverrides,
    },
};

// Source: cc/scheduler/webview_scheduler_state_machine.h:13-23
pub struct WebviewSchedulerStateMachine;
impl WebviewSchedulerStateMachine {
    // Source: cc/scheduler/webview_scheduler_state_machine.cc:9-16
    pub fn new(settings: &SchedulerSettings) -> Self {
        debug_assert!(settings.using_synchronous_renderer_compositor);
        Self
    }
}
impl SchedulerStateMachineOverrides for WebviewSchedulerStateMachine {
    // Source: cc/scheduler/webview_scheduler_state_machine.cc:18-23
    fn ShouldActivateSyncTreeBeforeDraw(&self, _state: &SchedulerStateMachine) -> Option<bool> {
        Some(false)
    }
    // Source: cc/scheduler/webview_scheduler_state_machine.cc:25-29
    fn ShouldBlockBeginMainFrameWhenIdle(&self, _state: &SchedulerStateMachine) -> Option<bool> {
        Some(false)
    }
    // Source: cc/scheduler/webview_scheduler_state_machine.cc:33-50
    fn ShouldInvalidateLayerTreeFrameSink(&self, state: &SchedulerStateMachine) -> Option<bool> {
        if state.did_invalidate_layer_tree_frame_sink_ {
            return Some(false);
        }
        if state.begin_impl_frame_state_ != BeginImplFrameState::INSIDE_BEGIN_FRAME {
            return Some(false);
        }
        Some(
            (state.needs_redraw_ && !state.PendingDrawsShouldBeAborted())
                || state.needs_prepare_tiles_,
        )
    }
    // Source: cc/scheduler/webview_scheduler_state_machine.cc:52-56
    fn CurrentBeginImplFrameDeadlineMode(
        &self,
        _state: &SchedulerStateMachine,
    ) -> Option<BeginImplFrameDeadlineMode> {
        Some(BeginImplFrameDeadlineMode::NONE)
    }
}
