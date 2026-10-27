// Translation status: complete
// Copyright 2011 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use super::{
    commit_earlyout_reason::CommitEarlyOutReason, draw_result::DrawResult,
    scheduler_settings::SchedulerSettings,
};
use crate::{
    begin_frame::BeginFrameArgs,
    services::{
        SchedulerFeatures, SchedulerProtoValue, SchedulerServices, SchedulerStateTrace,
        SchedulerTraceEvent, SchedulerTraceKind, SchedulerTraceScope, SchedulerTraceTrack,
        TreePriority,
    },
    TimeDelta, TimeTicks,
};
use std::sync::Arc;

// Source: cc/scheduler/scheduler_state_machine.h:41-47
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum LayerTreeFrameSinkState {
    NONE,
    ACTIVE,
    CREATING,
    WAITING_FOR_FIRST_COMMIT,
    WAITING_FOR_FIRST_ACTIVATION,
}
// Source: cc/scheduler/scheduler_state_machine.h:54-58
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum BeginImplFrameState {
    IDLE,
    INSIDE_BEGIN_FRAME,
    INSIDE_DEADLINE,
}
// Source: cc/scheduler/scheduler_state_machine.h:67-80
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum BeginImplFrameDeadlineMode {
    NONE = 0,
    IMMEDIATE = 1,
    WAIT_FOR_SCROLL = 2,
    REGULAR = 3,
    LATE = 4,
    BLOCKED = 5,
}
impl BeginImplFrameDeadlineMode {
    pub const kMaxValue: Self = Self::BLOCKED;
}
// Source: cc/scheduler/scheduler_state_machine.h:92-97
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum BeginMainFrameState {
    IDLE,
    SENT,
    READY_TO_COMMIT,
}
// Source: cc/scheduler/scheduler_state_machine.h:104-109
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum ForcedRedrawOnTimeoutState {
    IDLE,
    WAITING_FOR_COMMIT,
    WAITING_FOR_ACTIVATION,
    WAITING_FOR_DRAW,
}
// Source: cc/scheduler/scheduler_state_machine.h:131-144
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum Action {
    NONE,
    SEND_BEGIN_MAIN_FRAME,
    COMMIT,
    POST_COMMIT,
    ACTIVATE_SYNC_TREE,
    PERFORM_IMPL_SIDE_INVALIDATION,
    DRAW_IF_POSSIBLE,
    DRAW_FORCED,
    DRAW_ABORT,
    BEGIN_LAYER_TREE_FRAME_SINK_CREATION,
    PREPARE_TILES,
    INVALIDATE_LAYER_TREE_FRAME_SINK,
}
// Source: cc/scheduler/scheduler_state_machine.h:311-311
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum AnimationWorkletState {
    PROCESSING,
    IDLE,
}
// Source: cc/scheduler/scheduler_state_machine.h:312-312
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum PaintWorkletState {
    PROCESSING,
    IDLE,
}
// Source: cc/scheduler/scheduler_state_machine.h:313-313
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum TreeType {
    ACTIVE,
    PENDING,
}

// Source: cc/scheduler/scheduler_state_machine.h:465-469
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameEvents {
    pub commit_had_no_updates: bool,
    pub did_commit_during_frame: bool,
}

// Source: cc/scheduler/scheduler_state_machine.h:433-554
// Source: cc/scheduler/scheduler_state_machine.cc:86-101
pub struct SchedulerStateMachine {
    pub(super) settings_: SchedulerSettings,
    pub(super) layer_tree_frame_sink_state_: LayerTreeFrameSinkState,
    pub(super) begin_impl_frame_state_: BeginImplFrameState,
    pub(super) begin_main_frame_state_: BeginMainFrameState,
    pub(super) next_begin_main_frame_state_: BeginMainFrameState,
    pub(super) forced_redraw_state_: ForcedRedrawOnTimeoutState,
    pub(super) commit_count_: i32,
    pub(super) current_frame_number_: i32,
    pub(super) last_frame_number_submit_performed_: i32,
    pub(super) last_frame_number_draw_performed_: i32,
    pub(super) last_frame_number_begin_main_frame_sent_: i32,
    pub(super) last_frame_number_invalidate_layer_tree_frame_sink_performed_: i32,
    pub(super) last_begin_impl_frame_time_: TimeTicks,
    pub(super) last_sent_begin_main_frame_time_: TimeTicks,
    pub(super) main_frame_throttled_interval_: TimeDelta,
    pub(super) unthrottled_frame_interval_: TimeDelta,
    pub(super) main_frame_consecutive_no_damage_throttled_interval_: TimeDelta,
    pub(super) last_frame_events_: FrameEvents,
    pub(super) did_draw_: bool,
    pub(super) did_send_begin_main_frame_for_current_frame_: bool,
    pub(super) did_commit_during_frame_: bool,
    pub(super) did_invalidate_layer_tree_frame_sink_: bool,
    pub(super) did_perform_impl_side_invalidation_: bool,
    pub(super) did_prepare_tiles_: bool,
    pub(super) consecutive_checkerboard_animations_: i32,
    pub(super) consecutive_no_damage_main_frames_: i32,
    pub(super) repeated_no_damage_frame_throttling_threshold1_: i32,
    pub(super) repeated_no_damage_frame_throttling_threshold2_: i32,
    pub(super) repeated_no_damage_frame_throttling_factor1_: i32,
    pub(super) repeated_no_damage_frame_throttling_factor2_: i32,
    pub(super) pending_submit_frames_: i32,
    pub(super) submit_frames_with_current_layer_tree_frame_sink_: i32,
    pub(super) needs_redraw_: bool,
    pub(super) needs_prepare_tiles_: bool,
    pub(super) needs_begin_main_frame_: bool,
    pub(super) urgent_begin_main_frame_pending_: bool,
    pub(super) needs_one_begin_impl_frame_: bool,
    pub(super) needs_post_commit_: bool,
    pub(super) visible_: bool,
    pub(super) should_warm_up_: bool,
    pub(super) begin_frame_source_paused_: bool,
    pub(super) resourceless_draw_: bool,
    pub(super) can_draw_: bool,
    pub(super) skip_draw_: bool,
    pub(super) has_pending_tree_: bool,
    pub(super) pending_tree_is_ready_for_activation_: bool,
    pub(super) active_tree_needs_first_draw_: bool,
    pub(super) did_create_and_initialize_first_layer_tree_frame_sink_: bool,
    pub(super) tree_priority_: TreePriority,
    pub(super) is_current_scroll_main_painted_: bool,
    pub(super) critical_begin_main_frame_to_activate_is_fast_: bool,
    pub(super) main_thread_missed_last_deadline_: bool,
    pub(super) defer_begin_main_frame_: bool,
    pub(super) pause_rendering_: bool,
    pub(super) waiting_for_activation_after_rendering_resumed_: bool,
    pub(super) video_needs_begin_frames_: bool,
    pub(super) last_commit_had_no_updates_: bool,
    pub(super) active_tree_is_ready_to_draw_: bool,
    pub(super) did_attempt_draw_in_last_frame_: bool,
    pub(super) draw_succeeded_in_last_frame_: bool,
    pub(super) did_submit_in_last_frame_: bool,
    pub(super) needs_impl_side_invalidation_: bool,
    pub(super) next_invalidation_needs_first_draw_on_activation_: bool,
    pub(super) should_defer_invalidation_for_fast_main_frame_: bool,
    pub(super) begin_frame_is_animate_only_: bool,
    pub(super) processing_animation_worklets_for_active_tree_: i32,
    pub(super) processing_animation_worklets_for_pending_tree_: bool,
    pub(super) processing_paint_worklets_for_pending_tree_: bool,
    pub(super) previous_pending_tree_was_impl_side_: bool,
    pub(super) current_pending_tree_is_impl_side_: bool,
    pub(super) pending_tree_needs_first_draw_on_activation_: bool,
    pub(super) draw_aborted_for_paused_begin_frame_: bool,
    pub(super) consecutive_cant_draw_count_: u32,
    pub(super) is_scrolling_: bool,
    pub(super) waiting_for_scroll_event_: bool,
    pub(super) high_framerate_requests_count_: u64,
    pub(super) services_: Arc<dyn SchedulerServices>,
    overrides_: Option<Arc<dyn SchedulerStateMachineOverrides>>,
}

// Source: cc/scheduler/scheduler_state_machine.cc:33-36
const kMaxPendingSubmitFrames: i32 = 1;
const kFrameThrottlingSlackFactor: f32 = 0.9;

// Source: cc/scheduler/scheduler_state_machine.cc:38-48
fn IsEligibleToThrottleMainFrameRate(features: &SchedulerFeatures) -> bool {
    if features.is_android {
        return !features.is_desktop_android || features.kThrottleMainFrameTo60HzDesktopAndroid;
    } else {
        return true;
    }
}

// Source: cc/scheduler/scheduler_state_machine.cc:50-71
fn ShouldThrottleMainFrameRate(settings: &SchedulerSettings, features: &SchedulerFeatures) -> bool {
    if !features.is_eligible_for_throttle_main_frame_to_60hz {
        return false;
    }
    if features.is_android {
        let is_webview = settings.using_synchronous_renderer_compositor;
        return if is_webview {
            features.kThrottleMainFrameTo60HzWebView
        } else {
            features.kThrottleMainFrameTo60Hz
        };
    } else {
        let is_browser_compositor =
            settings.commit_to_active_tree || settings.single_threaded_proxy;
        if is_browser_compositor {
            return false;
        }
        return features.kThrottleMainFrameTo60Hz;
    }
}

// Source: cc/scheduler/scheduler_state_machine.cc:73-76
fn ThrottledFrameRateWithSlack(frame_interval: TimeDelta, factor: i32) -> TimeDelta {
    kFrameThrottlingSlackFactor * frame_interval * factor
}

/// Rust dispatch boundary for the source virtual methods. A derived scheduler
/// may return None to retain the unmodified classic implementation.
pub trait SchedulerStateMachineOverrides {
    fn NextAction(&self, _state: &SchedulerStateMachine) -> Option<Action> {
        None
    }
    fn CheckWillCommit(&self, _state: &SchedulerStateMachine) -> Option<bool> {
        None
    }
    fn ShouldSubscribeToBeginFrames(&self, _state: &SchedulerStateMachine) -> Option<bool> {
        None
    }
    fn OnBeginImplFrame(&self, _state: &mut SchedulerStateMachine, _args: &BeginFrameArgs) -> bool {
        false
    }
    fn CurrentBeginImplFrameDeadlineMode(
        &self,
        _state: &SchedulerStateMachine,
    ) -> Option<BeginImplFrameDeadlineMode> {
        None
    }
    fn ShouldTriggerBeginImplFrameDeadlineImmediately(
        &self,
        _state: &SchedulerStateMachine,
    ) -> Option<bool> {
        None
    }
    fn CheckShouldBlockDeadlineIndefinitely(&self, _state: &SchedulerStateMachine) -> Option<bool> {
        None
    }
    fn CheckShouldDraw(&self, _state: &SchedulerStateMachine) -> Option<bool> {
        None
    }
    fn ShouldActivateSyncTreeBeforeDraw(&self, _state: &SchedulerStateMachine) -> Option<bool> {
        None
    }
    fn ShouldBlockBeginMainFrameWhenIdle(&self, _state: &SchedulerStateMachine) -> Option<bool> {
        None
    }
    fn ShouldPrepareTiles(&self, _state: &SchedulerStateMachine) -> Option<bool> {
        None
    }
    fn ShouldInvalidateLayerTreeFrameSink(&self, _state: &SchedulerStateMachine) -> Option<bool> {
        None
    }
}

impl SchedulerStateMachine {
    // Source: cc/scheduler/scheduler_state_machine.cc:86-99
    pub fn new(settings: &SchedulerSettings, services: Arc<dyn SchedulerServices>) -> Self {
        let features = services.features();
        Self {
            settings_: settings.clone(),
            layer_tree_frame_sink_state_: LayerTreeFrameSinkState::NONE,
            begin_impl_frame_state_: BeginImplFrameState::IDLE,
            begin_main_frame_state_: BeginMainFrameState::IDLE,
            next_begin_main_frame_state_: BeginMainFrameState::IDLE,
            forced_redraw_state_: ForcedRedrawOnTimeoutState::IDLE,
            commit_count_: 0,
            current_frame_number_: 0,
            last_frame_number_submit_performed_: -1,
            last_frame_number_draw_performed_: -1,
            last_frame_number_begin_main_frame_sent_: -1,
            last_frame_number_invalidate_layer_tree_frame_sink_performed_: -1,
            last_begin_impl_frame_time_: TimeTicks::default(),
            last_sent_begin_main_frame_time_: TimeTicks::default(),
            main_frame_throttled_interval_: TimeDelta::default(),
            unthrottled_frame_interval_: TimeDelta::default(),
            main_frame_consecutive_no_damage_throttled_interval_: TimeDelta::default(),
            last_frame_events_: FrameEvents::default(),
            did_draw_: false,
            did_send_begin_main_frame_for_current_frame_: true,
            did_commit_during_frame_: false,
            did_invalidate_layer_tree_frame_sink_: false,
            did_perform_impl_side_invalidation_: false,
            did_prepare_tiles_: false,
            consecutive_checkerboard_animations_: 0,
            consecutive_no_damage_main_frames_: 0,
            repeated_no_damage_frame_throttling_threshold1_: std::cmp::max(
                1,
                features.kThrottleRepeatedNoDamageFramesThreshold1,
            ),
            repeated_no_damage_frame_throttling_threshold2_: std::cmp::max(
                1,
                features.kThrottleRepeatedNoDamageFramesThreshold2,
            ),
            repeated_no_damage_frame_throttling_factor1_: std::cmp::max(
                1,
                features.kThrottleRepeatedNoDamageFramesIntervalFactor1,
            ),
            repeated_no_damage_frame_throttling_factor2_: std::cmp::max(
                1,
                features.kThrottleRepeatedNoDamageFramesIntervalFactor2,
            ),
            pending_submit_frames_: 0,
            submit_frames_with_current_layer_tree_frame_sink_: 0,
            needs_redraw_: false,
            needs_prepare_tiles_: false,
            needs_begin_main_frame_: false,
            urgent_begin_main_frame_pending_: false,
            needs_one_begin_impl_frame_: false,
            needs_post_commit_: false,
            visible_: false,
            should_warm_up_: false,
            begin_frame_source_paused_: false,
            resourceless_draw_: false,
            can_draw_: false,
            skip_draw_: false,
            has_pending_tree_: false,
            pending_tree_is_ready_for_activation_: false,
            active_tree_needs_first_draw_: false,
            did_create_and_initialize_first_layer_tree_frame_sink_: false,
            tree_priority_: TreePriority::NEW_CONTENT_TAKES_PRIORITY,
            is_current_scroll_main_painted_: false,
            critical_begin_main_frame_to_activate_is_fast_: true,
            main_thread_missed_last_deadline_: false,
            defer_begin_main_frame_: false,
            pause_rendering_: false,
            waiting_for_activation_after_rendering_resumed_: false,
            video_needs_begin_frames_: false,
            last_commit_had_no_updates_: false,
            active_tree_is_ready_to_draw_: true,
            did_attempt_draw_in_last_frame_: false,
            draw_succeeded_in_last_frame_: false,
            did_submit_in_last_frame_: false,
            needs_impl_side_invalidation_: false,
            next_invalidation_needs_first_draw_on_activation_: false,
            should_defer_invalidation_for_fast_main_frame_: true,
            begin_frame_is_animate_only_: false,
            processing_animation_worklets_for_active_tree_: 0,
            processing_animation_worklets_for_pending_tree_: false,
            processing_paint_worklets_for_pending_tree_: false,
            previous_pending_tree_was_impl_side_: false,
            current_pending_tree_is_impl_side_: false,
            pending_tree_needs_first_draw_on_activation_: false,
            draw_aborted_for_paused_begin_frame_: false,
            consecutive_cant_draw_count_: 0,
            is_scrolling_: false,
            waiting_for_scroll_event_: false,
            high_framerate_requests_count_: 0,
            services_: services,
            overrides_: None,
        }
    }
    // Source: cc/scheduler/scheduler_state_machine.h:115-117
    pub fn begin_main_frame_state(&self) -> BeginMainFrameState {
        return self.begin_main_frame_state_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:119-121
    pub fn CommitPending(&self) -> bool {
        return self.begin_main_frame_state_ != BeginMainFrameState::IDLE;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:123-126
    pub fn NewActiveTreeLikely(&self) -> bool {
        return (self.needs_begin_main_frame_ && !self.last_commit_had_no_updates_)
            || self.CommitPending()
            || self.has_pending_tree_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:128-128
    pub fn RedrawPending(&self) -> bool {
        return self.needs_redraw_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:129-129
    pub fn PrepareTilesPending(&self) -> bool {
        return self.needs_prepare_tiles_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:188-188
    pub fn current_frame_number(&self) -> i32 {
        return self.current_frame_number_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:190-192
    pub fn begin_impl_frame_state(&self) -> BeginImplFrameState {
        return self.begin_impl_frame_state_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:199-201
    pub fn main_thread_missed_last_deadline(&self) -> bool {
        return self.main_thread_missed_last_deadline_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:214-214
    pub fn visible(&self) -> bool {
        return self.visible_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:221-221
    pub fn begin_frame_source_paused(&self) -> bool {
        return self.begin_frame_source_paused_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:229-229
    pub fn needs_redraw(&self) -> bool {
        return self.needs_redraw_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:232-234
    pub fn did_invalidate_layer_tree_frame_sink(&self) -> bool {
        return self.did_invalidate_layer_tree_frame_sink_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:249-249
    pub fn pending_submit_frames(&self) -> i32 {
        return self.pending_submit_frames_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:274-274
    pub fn needs_begin_main_frame(&self) -> bool {
        return self.needs_begin_main_frame_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:328-328
    pub fn has_pending_tree(&self) -> bool {
        return self.has_pending_tree_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:329-331
    pub fn active_tree_needs_first_draw(&self) -> bool {
        return self.active_tree_needs_first_draw_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:350-350
    pub fn did_submit_in_last_frame(&self) -> bool {
        return self.did_submit_in_last_frame_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:351-353
    pub fn draw_succeeded_in_last_frame(&self) -> bool {
        return self.draw_succeeded_in_last_frame_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:355-357
    pub fn needs_impl_side_invalidation(&self) -> bool {
        return self.needs_impl_side_invalidation_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:358-360
    pub fn previous_pending_tree_was_impl_side(&self) -> bool {
        return self.previous_pending_tree_was_impl_side_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:361-363
    pub fn critical_begin_main_frame_to_activate_is_fast(&self) -> bool {
        return self.critical_begin_main_frame_to_activate_is_fast_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:368-370
    pub fn should_defer_invalidation_for_fast_main_frame(&self) -> bool {
        return self.should_defer_invalidation_for_fast_main_frame_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:372-374
    pub fn pending_tree_is_ready_for_activation(&self) -> bool {
        return self.pending_tree_is_ready_for_activation_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:376-376
    pub fn resourceless_draw(&self) -> bool {
        return self.resourceless_draw_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:385-387
    pub fn consecutive_no_damage_main_frames(&self) -> i32 {
        return self.consecutive_no_damage_main_frames_;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:365-367
    pub fn set_should_defer_invalidation_for_fast_main_frame(&mut self, defer: bool) {
        self.should_defer_invalidation_for_fast_main_frame_ = defer;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:378-378
    pub fn set_is_scrolling(&mut self, is_scrolling: bool) {
        self.is_scrolling_ = is_scrolling;
    }

    // Source: cc/scheduler/scheduler_state_machine.h:379-381
    pub fn set_waiting_for_scroll_event(&mut self, waiting_for_scroll_event: bool) {
        self.waiting_for_scroll_event_ = waiting_for_scroll_event;
    }
    // Source: cc/scheduler/scheduler_state_machine.cc:103-122
    pub fn LayerTreeFrameSinkStateToProtozeroEnum(
        state: LayerTreeFrameSinkState,
    ) -> SchedulerProtoValue {
        match state {
            LayerTreeFrameSinkState::NONE => SchedulerProtoValue::LAYER_TREE_FRAME_NONE,
            LayerTreeFrameSinkState::ACTIVE => SchedulerProtoValue::LAYER_TREE_FRAME_ACTIVE,
            LayerTreeFrameSinkState::CREATING => SchedulerProtoValue::LAYER_TREE_FRAME_CREATING,
            LayerTreeFrameSinkState::WAITING_FOR_FIRST_COMMIT => {
                SchedulerProtoValue::LAYER_TREE_FRAME_WAITING_FOR_FIRST_COMMIT
            }
            LayerTreeFrameSinkState::WAITING_FOR_FIRST_ACTIVATION => {
                SchedulerProtoValue::LAYER_TREE_FRAME_WAITING_FOR_FIRST_ACTIVATION
            }
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:124-139
    pub fn BeginImplFrameStateToProtozeroEnum(state: BeginImplFrameState) -> SchedulerProtoValue {
        match state {
            BeginImplFrameState::IDLE => SchedulerProtoValue::BEGIN_IMPL_FRAME_IDLE,
            BeginImplFrameState::INSIDE_BEGIN_FRAME => {
                SchedulerProtoValue::BEGIN_IMPL_FRAME_INSIDE_BEGIN_FRAME
            }
            BeginImplFrameState::INSIDE_DEADLINE => {
                SchedulerProtoValue::BEGIN_IMPL_FRAME_INSIDE_DEADLINE
            }
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:141-158
    pub fn BeginImplFrameDeadlineModeToString(mode: BeginImplFrameDeadlineMode) -> &'static str {
        match mode {
            BeginImplFrameDeadlineMode::NONE => "BeginImplFrameDeadlineMode::NONE",
            BeginImplFrameDeadlineMode::IMMEDIATE => "BeginImplFrameDeadlineMode::IMMEDIATE",
            BeginImplFrameDeadlineMode::WAIT_FOR_SCROLL => {
                "BeginImplFrameDeadlineMode::WAIT_FOR_SCROLL"
            }
            BeginImplFrameDeadlineMode::REGULAR => "BeginImplFrameDeadlineMode::REGULAR",
            BeginImplFrameDeadlineMode::LATE => "BeginImplFrameDeadlineMode::LATE",
            BeginImplFrameDeadlineMode::BLOCKED => "BeginImplFrameDeadlineMode::BLOCKED",
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:160-181
    pub fn BeginImplFrameDeadlineModeToProtozeroEnum(
        mode: BeginImplFrameDeadlineMode,
    ) -> SchedulerProtoValue {
        match mode {
            BeginImplFrameDeadlineMode::NONE => SchedulerProtoValue::DEADLINE_MODE_NONE,
            BeginImplFrameDeadlineMode::IMMEDIATE => SchedulerProtoValue::DEADLINE_MODE_IMMEDIATE,
            BeginImplFrameDeadlineMode::WAIT_FOR_SCROLL => {
                SchedulerProtoValue::DEADLINE_MODE_WAIT_FOR_SCROLL
            }
            BeginImplFrameDeadlineMode::REGULAR => SchedulerProtoValue::DEADLINE_MODE_REGULAR,
            BeginImplFrameDeadlineMode::LATE => SchedulerProtoValue::DEADLINE_MODE_LATE,
            BeginImplFrameDeadlineMode::BLOCKED => SchedulerProtoValue::DEADLINE_MODE_BLOCKED,
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:183-198
    pub fn BeginMainFrameStateToProtozeroEnum(state: BeginMainFrameState) -> SchedulerProtoValue {
        match state {
            BeginMainFrameState::IDLE => SchedulerProtoValue::BEGIN_MAIN_FRAME_IDLE,
            BeginMainFrameState::SENT => SchedulerProtoValue::BEGIN_MAIN_FRAME_SENT,
            BeginMainFrameState::READY_TO_COMMIT => {
                SchedulerProtoValue::BEGIN_MAIN_FRAME_READY_TO_COMMIT
            }
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:200-217
    pub fn ForcedRedrawOnTimeoutStateToProtozeroEnum(
        state: ForcedRedrawOnTimeoutState,
    ) -> SchedulerProtoValue {
        match state {
            ForcedRedrawOnTimeoutState::IDLE => SchedulerProtoValue::FORCED_REDRAW_IDLE,
            ForcedRedrawOnTimeoutState::WAITING_FOR_COMMIT => {
                SchedulerProtoValue::FORCED_REDRAW_WAITING_FOR_COMMIT
            }
            ForcedRedrawOnTimeoutState::WAITING_FOR_ACTIVATION => {
                SchedulerProtoValue::FORCED_REDRAW_WAITING_FOR_ACTIVATION
            }
            ForcedRedrawOnTimeoutState::WAITING_FOR_DRAW => {
                SchedulerProtoValue::FORCED_REDRAW_WAITING_FOR_DRAW
            }
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:219-254
    pub fn ActionToProtozeroEnum(action: Action) -> SchedulerProtoValue {
        match action {
            Action::NONE => SchedulerProtoValue::CC_SCHEDULER_ACTION_V2_NONE,
            Action::SEND_BEGIN_MAIN_FRAME => {
                SchedulerProtoValue::CC_SCHEDULER_ACTION_V2_SEND_BEGIN_MAIN_FRAME
            }
            Action::COMMIT => SchedulerProtoValue::CC_SCHEDULER_ACTION_V2_COMMIT,
            Action::POST_COMMIT => SchedulerProtoValue::CC_SCHEDULER_ACTION_V2_POST_COMMIT,
            Action::ACTIVATE_SYNC_TREE => {
                SchedulerProtoValue::CC_SCHEDULER_ACTION_V2_ACTIVATE_SYNC_TREE
            }
            Action::DRAW_IF_POSSIBLE => {
                SchedulerProtoValue::CC_SCHEDULER_ACTION_V2_DRAW_IF_POSSIBLE
            }
            Action::DRAW_FORCED => SchedulerProtoValue::CC_SCHEDULER_ACTION_V2_DRAW_FORCED,
            Action::DRAW_ABORT => SchedulerProtoValue::CC_SCHEDULER_ACTION_V2_DRAW_ABORT,
            Action::BEGIN_LAYER_TREE_FRAME_SINK_CREATION => {
                SchedulerProtoValue::CC_SCHEDULER_ACTION_V2_BEGIN_LAYER_TREE_FRAME_SINK_CREATION
            }
            Action::PREPARE_TILES => SchedulerProtoValue::CC_SCHEDULER_ACTION_V2_PREPARE_TILES,
            Action::INVALIDATE_LAYER_TREE_FRAME_SINK => {
                SchedulerProtoValue::CC_SCHEDULER_ACTION_V2_INVALIDATE_LAYER_TREE_FRAME_SINK
            }
            Action::PERFORM_IMPL_SIDE_INVALIDATION => {
                SchedulerProtoValue::CC_SCHEDULER_ACTION_V2_PERFORM_IMPL_SIDE_INVALIDATION
            }
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:256-329
    pub fn AsProtozeroInto(&self, state: &mut SchedulerStateTrace) {
        state.major_state.insert(
            "next_action",
            Self::ActionToProtozeroEnum(self.NextAction()),
        );
        state.major_state.insert(
            "begin_impl_frame_state",
            Self::BeginImplFrameStateToProtozeroEnum(self.begin_impl_frame_state_),
        );
        state.major_state.insert(
            "begin_main_frame_state",
            Self::BeginMainFrameStateToProtozeroEnum(self.begin_main_frame_state_),
        );
        state.major_state.insert(
            "layer_tree_frame_sink_state",
            Self::LayerTreeFrameSinkStateToProtozeroEnum(self.layer_tree_frame_sink_state_),
        );
        state.major_state.insert(
            "forced_redraw_state",
            Self::ForcedRedrawOnTimeoutStateToProtozeroEnum(self.forced_redraw_state_),
        );
        state.minor_state.insert(
            "commit_count",
            crate::adapter::TraceValue::Integer(i64::from(self.commit_count_)),
        );
        state.minor_state.insert(
            "current_frame_number",
            crate::adapter::TraceValue::Integer(i64::from(self.current_frame_number_)),
        );
        state.minor_state.insert(
            "last_frame_number_submit_performed",
            crate::adapter::TraceValue::Integer(i64::from(
                self.last_frame_number_submit_performed_,
            )),
        );
        state.minor_state.insert(
            "last_frame_number_draw_performed",
            crate::adapter::TraceValue::Integer(i64::from(self.last_frame_number_draw_performed_)),
        );
        state.minor_state.insert(
            "last_frame_number_begin_main_frame_sent",
            crate::adapter::TraceValue::Integer(i64::from(
                self.last_frame_number_begin_main_frame_sent_,
            )),
        );
        state
            .minor_state
            .insert("did_draw", crate::adapter::TraceValue::Bool(self.did_draw_));
        state.minor_state.insert(
            "did_send_begin_main_frame_for_current_frame",
            crate::adapter::TraceValue::Bool(self.did_send_begin_main_frame_for_current_frame_),
        );
        state.minor_state.insert(
            "did_commit_during_frame",
            crate::adapter::TraceValue::Bool(self.did_commit_during_frame_),
        );
        state.minor_state.insert(
            "did_invalidate_layer_tree_frame_sink",
            crate::adapter::TraceValue::Bool(self.did_invalidate_layer_tree_frame_sink_),
        );
        state.minor_state.insert(
            "did_perform_impl_side_invalidaion",
            crate::adapter::TraceValue::Bool(self.did_perform_impl_side_invalidation_),
        );
        state.minor_state.insert(
            "did_prepare_tiles",
            crate::adapter::TraceValue::Bool(self.did_prepare_tiles_),
        );
        state.minor_state.insert(
            "consecutive_checkerboard_animations",
            crate::adapter::TraceValue::Integer(i64::from(
                self.consecutive_checkerboard_animations_,
            )),
        );
        state.minor_state.insert(
            "pending_submit_frames",
            crate::adapter::TraceValue::Integer(i64::from(self.pending_submit_frames_)),
        );
        state.minor_state.insert(
            "submit_frames_with_current_layer_tree_frame_sink",
            crate::adapter::TraceValue::Integer(i64::from(
                self.submit_frames_with_current_layer_tree_frame_sink_,
            )),
        );
        state.minor_state.insert(
            "needs_redraw",
            crate::adapter::TraceValue::Bool(self.needs_redraw_),
        );
        state.minor_state.insert(
            "needs_prepare_tiles",
            crate::adapter::TraceValue::Bool(self.needs_prepare_tiles_),
        );
        state.minor_state.insert(
            "needs_begin_main_frame",
            crate::adapter::TraceValue::Bool(self.needs_begin_main_frame_),
        );
        state.minor_state.insert(
            "needs_one_begin_impl_frame",
            crate::adapter::TraceValue::Bool(self.needs_one_begin_impl_frame_),
        );
        state
            .minor_state
            .insert("visible", crate::adapter::TraceValue::Bool(self.visible_));
        state.minor_state.insert(
            "begin_frame_source_paused",
            crate::adapter::TraceValue::Bool(self.begin_frame_source_paused_),
        );
        state
            .minor_state
            .insert("can_draw", crate::adapter::TraceValue::Bool(self.can_draw_));
        state.minor_state.insert(
            "resourceless_draw",
            crate::adapter::TraceValue::Bool(self.resourceless_draw_),
        );
        state.minor_state.insert(
            "has_pending_tree",
            crate::adapter::TraceValue::Bool(self.has_pending_tree_),
        );
        state.minor_state.insert(
            "pending_tree_is_ready_for_activation",
            crate::adapter::TraceValue::Bool(self.pending_tree_is_ready_for_activation_),
        );
        state.minor_state.insert(
            "active_tree_needs_first_draw",
            crate::adapter::TraceValue::Bool(self.active_tree_needs_first_draw_),
        );
        state.minor_state.insert(
            "active_tree_is_ready_to_draw",
            crate::adapter::TraceValue::Bool(self.active_tree_is_ready_to_draw_),
        );
        state.minor_state.insert(
            "did_create_and_initialize_first_layer_tree_frame_sink",
            crate::adapter::TraceValue::Bool(
                self.did_create_and_initialize_first_layer_tree_frame_sink_,
            ),
        );
        state.tree_priority = Some(match self.tree_priority_ {
            TreePriority::SAME_PRIORITY_FOR_BOTH_TREES => {
                SchedulerProtoValue::TREE_PRIORITY_SAME_PRIORITY_FOR_BOTH_TREES
            }
            TreePriority::SMOOTHNESS_TAKES_PRIORITY => {
                SchedulerProtoValue::TREE_PRIORITY_SMOOTHNESS_TAKES_PRIORITY
            }
            TreePriority::NEW_CONTENT_TAKES_PRIORITY => {
                SchedulerProtoValue::TREE_PRIORITY_NEW_CONTENT_TAKES_PRIORITY
            }
        });
        state.minor_state.insert(
            "critical_begin_main_frame_to_activate_is_fast",
            crate::adapter::TraceValue::Bool(self.critical_begin_main_frame_to_activate_is_fast_),
        );
        state.minor_state.insert(
            "main_thread_missed_last_deadline",
            crate::adapter::TraceValue::Bool(self.main_thread_missed_last_deadline_),
        );
        state.minor_state.insert(
            "video_needs_begin_frames",
            crate::adapter::TraceValue::Bool(self.video_needs_begin_frames_),
        );
        state.minor_state.insert(
            "defer_begin_main_frame",
            crate::adapter::TraceValue::Bool(self.defer_begin_main_frame_),
        );
        state.minor_state.insert(
            "last_commit_had_no_updates",
            crate::adapter::TraceValue::Bool(self.last_commit_had_no_updates_),
        );
        state.minor_state.insert(
            "did_draw_in_last_frame",
            crate::adapter::TraceValue::Bool(self.did_attempt_draw_in_last_frame_),
        );
        state.minor_state.insert(
            "did_submit_in_last_frame",
            crate::adapter::TraceValue::Bool(self.did_submit_in_last_frame_),
        );
        state.minor_state.insert(
            "needs_impl_side_invalidation",
            crate::adapter::TraceValue::Bool(self.needs_impl_side_invalidation_),
        );
        state.minor_state.insert(
            "current_pending_tree_is_impl_side",
            crate::adapter::TraceValue::Bool(self.current_pending_tree_is_impl_side_),
        );
        state.minor_state.insert(
            "previous_pending_tree_was_impl_side",
            crate::adapter::TraceValue::Bool(self.previous_pending_tree_was_impl_side_),
        );
        state.minor_state.insert(
            "processing_animation_worklets_for_active_tree",
            crate::adapter::TraceValue::Integer(i64::from(
                self.processing_animation_worklets_for_active_tree_,
            )),
        );
        state.minor_state.insert(
            "processing_animation_worklets_for_pending_tree",
            crate::adapter::TraceValue::Bool(self.processing_animation_worklets_for_pending_tree_),
        );
        state.minor_state.insert(
            "processing_paint_worklets_for_pending_tree",
            crate::adapter::TraceValue::Bool(self.processing_paint_worklets_for_pending_tree_),
        );
        state.minor_state.insert(
            "should_warm_up",
            crate::adapter::TraceValue::Bool(self.should_warm_up_),
        );
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:331-351
    pub fn PendingDrawsShouldBeAborted(&self) -> bool {
        // Normally when |visible_| is false or |begin_frame_source_paused_| is true,
        // pending activations will be forced and draws will be aborted. However,
        // when the embedder is Android WebView, software draws could be scheduled by
        // the Android OS at any time and draws should not be aborted in this case.
        let mut is_layer_tree_frame_sink_lost =
            (self.layer_tree_frame_sink_state_ == LayerTreeFrameSinkState::NONE);
        if (self.resourceless_draw_) {
            return is_layer_tree_frame_sink_lost || !self.can_draw_;
        }

        // These are all the cases where we normally cannot or do not want
        // to draw but, if |needs_redraw_| is true and we do not draw to
        // make forward progress, we might deadlock with the main
        // thread. This should be a superset of ShouldAbortCurrentFrame()
        // since activation of the pending tree is blocked by drawing of the
        // active tree and the main thread might be blocked on activation of
        // the most recent commit.
        return is_layer_tree_frame_sink_lost
            || !self.can_draw_
            || !self.visible_
            || self.begin_frame_source_paused_
            || self.waiting_for_activation_after_rendering_resumed_;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:353-376
    pub fn ShouldAbortCurrentFrame(&self) -> bool {
        // Abort the frame if there is no output surface to trigger our
        // activations, avoiding deadlock with the main thread.
        if (self.layer_tree_frame_sink_state_ == LayerTreeFrameSinkState::NONE) {
            return true;
        }

        // If we're not visible, we should just abort the frame. Since we
        // set RequiresHighResToDraw when becoming visible, we ensure that
        // we don't checkerboard until all visible resources are
        // done. Furthermore, if we do keep the pending tree around, when
        // becoming visible we might activate prematurely causing
        // RequiresHighResToDraw flag to be reset. In all cases, we can
        // simply activate on becoming invisible since we don't need to draw
        // the active tree when we're in this state.
        if (!self.visible_) {
            return true;
        }

        // Abort the frame when viz::BeginFrameSource is paused to avoid
        // deadlocking the main thread.
        if (self.begin_frame_source_paused_) {
            return true;
        }

        return false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:378-405
    pub fn ShouldBeginLayerTreeFrameSinkCreation(&self) -> bool {
        if (!self.should_warm_up_ && !self.visible_) {
            return false;
        }

        // We only want to start output surface initialization after the
        // previous commit is complete.
        if (self.begin_main_frame_state_ != BeginMainFrameState::IDLE
            || self.next_begin_main_frame_state_ != BeginMainFrameState::IDLE)
        {
            return false;
        }

        // Make sure the BeginImplFrame from any previous LayerTreeFrameSinks
        // are complete before creating the new LayerTreeFrameSink.
        if (self.begin_impl_frame_state_ != BeginImplFrameState::IDLE) {
            return false;
        }

        // We want to clear the pipeline of any pending draws and activations
        // before starting output surface initialization. This allows us to avoid
        // weird corner cases where we abort draws or force activation while we
        // are initializing the output surface.
        if (self.active_tree_needs_first_draw_ || self.has_pending_tree_) {
            return false;
        }

        // We need to create the output surface if we don't have one and we haven't
        // started creating one yet.
        return self.layer_tree_frame_sink_state_ == LayerTreeFrameSinkState::NONE;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:407-412
    pub fn CheckShouldDrawBase(&self) -> bool {
        // Wait for ready to draw in full-pipeline mode or the browser compositor's
        // commit-to-active-tree mode.
        // When
        return (self.settings_.commit_to_active_tree && !self.active_tree_is_ready_to_draw_);
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:414-459
    pub fn ShouldDraw(&self) -> bool {
        // If we need to abort draws, we should do so ASAP since the draw could
        // be blocking other important actions (like output surface initialization),
        // from occurring. If we are waiting for the first draw, then perform the
        // aborted draw to keep things moving. If we are not waiting for the first
        // draw however, we don't want to abort for no reason.
        if (self.PendingDrawsShouldBeAborted()) {
            return self.active_tree_needs_first_draw_;
        }

        // Do not draw more than once in the deadline. Aborted draws are ok because
        // those are effectively nops.
        if (self.did_draw_) {
            return false;
        }

        // Don't draw if an early check determined the frame does not have damage.
        if (self.skip_draw_) {
            return false;
        }

        // Don't draw if we are waiting on the first commit after a surface.
        if (self.layer_tree_frame_sink_state_ != LayerTreeFrameSinkState::ACTIVE) {
            return false;
        }

        // Do not queue too many draws.
        if (self.IsDrawThrottled()) {
            return false;
        }

        // Except for the cases above, do not draw outside of the BeginImplFrame
        // deadline.
        if (self.begin_impl_frame_state_ != BeginImplFrameState::INSIDE_DEADLINE) {
            return false;
        }

        if (self.CheckShouldDraw()) {
            return false;
        }

        // Browser compositor commit steals any resources submitted in draw. Therefore
        // drawing while a commit is pending is wasteful.
        if (self.settings_.commit_to_active_tree && self.CommitPending()) {
            return false;
        }

        // Only handle forced redraws due to timeouts on the regular deadline.
        if (self.forced_redraw_state_ == ForcedRedrawOnTimeoutState::WAITING_FOR_DRAW) {
            return true;
        }

        return self.needs_redraw_;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:461-464
    pub fn ShouldActivateSyncTreeBeforeDrawBase(&self) -> bool {
        return (!self.settings_.using_synchronous_renderer_compositor
            && self.active_tree_needs_first_draw_);
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:466-513
    pub fn ShouldActivateSyncTree(&self) -> bool {
        // There is nothing to activate.
        if (!self.has_pending_tree_) {
            self.trace(
                SchedulerTraceKind::Instant,
                "disabled-by-default-cc.debug",
                "Not activating sync tree due to no pending tree",
                || vec![],
            );
            return false;
        }

        // We should not activate a second tree before drawing the first one.
        // Even if we need to force activation of the pending tree, we should abort
        // drawing the active tree first.
        if (self.ShouldActivateSyncTreeBeforeDraw()) {
            self.trace(
                SchedulerTraceKind::Instant,
                "disabled-by-default-cc.debug",
                "Not activating before drawing active first",
                || vec![],
            );
            return false;
        }

        // Delay pending tree activation until paint worklets have completed painting
        // the pending tree. This must occur before the |ShouldAbortCurrentFrame|
        // check as we cannot have an unpainted active tree.
        //
        // Note that paint worklets continue to paint when the page is not visible, so
        // any abort will eventually happen when they complete.
        if (self.processing_paint_worklets_for_pending_tree_) {
            self.trace(
                SchedulerTraceKind::Instant,
                "disabled-by-default-cc.debug",
                "Not activating due to processing paint worklets",
                || vec![],
            );
            return false;
        }

        if (self.ShouldAbortCurrentFrame()) {
            return true;
        }

        // Delay pending tree activation until animation worklets have completed
        // their asynchronous updates to pick up initial values.
        if (self.processing_animation_worklets_for_pending_tree_) {
            self.trace(
                SchedulerTraceKind::Instant,
                "disabled-by-default-cc.debug",
                "Not activating due to processing animation worklets",
                || vec![],
            );
            return false;
        }

        // At this point, only activate if we are ready to activate.
        if (!self.pending_tree_is_ready_for_activation_) {
            self.trace(
                SchedulerTraceKind::Instant,
                "disabled-by-default-cc.debug",
                "Not activating because pending tree not ready",
                || vec![],
            );
            return false;
        }
        return true;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:515-537
    pub fn CouldSendBeginMainFrame(&self) -> bool {
        if (!self.needs_begin_main_frame_) {
            return false;
        }

        // We can not perform commits if we are not visible.
        if (!self.visible_) {
            return false;
        }

        // There are no BeginImplFrames while viz::BeginFrameSource is paused,
        // so should also stop BeginMainFrames.
        if (self.begin_frame_source_paused_) {
            return false;
        }

        // Do not send begin main frame when it is deferred.
        if (self.defer_begin_main_frame_) {
            return false;
        }

        // Do not send begin main frames if we want to pause rendering.
        if (self.pause_rendering_) {
            return false;
        }

        return true;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:539-542
    pub fn ShouldBlockBeginMainFrameWhenIdleBase(&self) -> bool {
        return (!self.settings_.using_synchronous_renderer_compositor
            && self.begin_impl_frame_state_ == BeginImplFrameState::IDLE);
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:544-637
    pub fn ShouldSendBeginMainFrame(&self) -> bool {
        if !self.CouldSendBeginMainFrame() {
            return false;
        }
        if self.did_send_begin_main_frame_for_current_frame_
            && !self.urgent_begin_main_frame_pending_
        {
            return false;
        }
        match self.begin_main_frame_state_ {
            BeginMainFrameState::IDLE => {}
            BeginMainFrameState::SENT => return false,
            BeginMainFrameState::READY_TO_COMMIT => {
                if !self.settings_.main_frame_before_commit_enabled
                    || self.next_begin_main_frame_state_ != BeginMainFrameState::IDLE
                {
                    return false;
                }
            }
        }
        let can_send_main_frame_with_pending_tree =
            self.settings_.main_frame_before_activation_enabled
                || self.current_pending_tree_is_impl_side_;
        if self.has_pending_tree_ && !can_send_main_frame_with_pending_tree {
            return false;
        }
        if self.settings_.commit_to_active_tree
            && (self.active_tree_needs_first_draw_ || self.IsDrawThrottled())
        {
            return false;
        }
        if self.ImplLatencyTakesPriority()
            && ((self.has_pending_tree_ && !self.current_pending_tree_is_impl_side_)
                || (self.active_tree_needs_first_draw_
                    && !self.previous_pending_tree_was_impl_side_))
        {
            return false;
        }
        if self.ShouldBlockBeginMainFrameWhenIdle() && !self.urgent_begin_main_frame_pending_ {
            return false;
        }
        if self.forced_redraw_state_ == ForcedRedrawOnTimeoutState::WAITING_FOR_COMMIT {
            return true;
        }
        if !self.HasInitializedLayerTreeFrameSink() {
            return false;
        }
        let just_submitted_in_deadline = self.begin_impl_frame_state_
            == BeginImplFrameState::INSIDE_DEADLINE
            && self.did_submit_in_last_frame_;
        if self.IsDrawThrottled() && !just_submitted_in_deadline {
            return false;
        }
        if self.ShouldWaitForScrollEvent() {
            return false;
        }
        if self.ShouldThrottleSendBeginMainFrame() {
            return false;
        }
        true
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:639-665
    pub fn ShouldThrottleSendBeginMainFrame(&self) -> bool {
        let mut result = false;
        let mut throttled_interval = self.MainFrameThrottledInterval();

        if (self.services_.features().kThrottleRepeatedNoDamageFrames) {
            throttled_interval = std::cmp::max(
                throttled_interval,
                self.main_frame_consecutive_no_damage_throttled_interval_,
            );
        }

        if (throttled_interval.is_positive()
            && self.last_begin_impl_frame_time_ - self.last_sent_begin_main_frame_time_
                < throttled_interval)
        {
            result = true;
        }

        // Throttling main frame production when the current scroll is blocked on it
        // is visually bad, as it loses the benefit of high refresh rate. Don't
        // throttle. This is more expensive, but is required to reach perceptual
        // visual parity between throttled and non-throttled scrolling.
        if (self.is_current_scroll_main_painted_) {
            result = false;
        }

        self.trace(
            SchedulerTraceKind::Instant,
            "cc",
            "SchedulerStateMachine::ShouldThrottleSendBeginMainFrame",
            || vec![("result", crate::adapter::TraceValue::Bool(result))],
        );
        return result;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:667-687
    pub fn ShouldCommit(&self) -> bool {
        if (self.begin_main_frame_state_ != BeginMainFrameState::READY_TO_COMMIT) {
            return false;
        }

        // We must not finish the commit until the pending tree is free.
        if (self.has_pending_tree_) {
            debug_assert!(
                self.settings_.main_frame_before_activation_enabled
                    || self.settings_.main_frame_before_commit_enabled
                    || self.current_pending_tree_is_impl_side_
            );
            return false;
        }

        // If we only have an active tree, it is incorrect to replace it before we've
        // drawn it.
        debug_assert!(!self.settings_.commit_to_active_tree || !self.active_tree_needs_first_draw_);

        // In browser compositor commit reclaims any resources submitted during draw.
        debug_assert!(!self.settings_.commit_to_active_tree || !self.IsDrawThrottled());

        return true;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:689-692
    pub fn DidCommit(&mut self) {
        debug_assert!(!self.needs_post_commit_);
        self.needs_post_commit_ = true;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:694-696
    pub fn ShouldRunPostCommit(&self) -> bool {
        return self.needs_post_commit_;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:698-701
    pub fn DidPostCommit(&mut self) {
        debug_assert!(self.needs_post_commit_);
        self.needs_post_commit_ = false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:703-715
    pub fn ShouldPrepareTilesBase(&self) -> bool {
        // Do not prepare tiles if we've already done so in commit or impl side
        // invalidation.
        if (self.did_prepare_tiles_) {
            return false;
        }

        // Limiting to once per-frame is not enough, since we only want to prepare
        // tiles _after_ draws.
        if (self.begin_impl_frame_state_ != BeginImplFrameState::INSIDE_DEADLINE) {
            return false;
        }

        return self.needs_prepare_tiles_;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:717-736
    pub fn ShouldInvalidateLayerTreeFrameSinkBase(&self) -> bool {
        // Do not invalidate more than once per begin frame.
        if (self.did_invalidate_layer_tree_frame_sink_) {
            return false;
        }

        // Only the synchronous compositor requires invalidations.
        if (!self.settings_.using_synchronous_renderer_compositor) {
            return false;
        }

        // Invalidations are only performed inside a BeginFrame.
        if (self.begin_impl_frame_state_ != BeginImplFrameState::INSIDE_BEGIN_FRAME) {
            return false;
        }

        // Don't invalidate for draw if we cannnot draw.
        // TODO(sunnyps): needs_prepare_tiles_ is needed here because PrepareTiles is
        // called only inside the deadline / draw phase. We could remove this if we
        // allowed PrepareTiles to happen in OnBeginImplFrame.
        return (self.needs_redraw_ && !self.PendingDrawsShouldBeAborted())
            || self.needs_prepare_tiles_;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:738-765
    pub fn NextActionBase(&self) -> Action {
        if (self.ShouldSendBeginMainFrame()) {
            return Action::SEND_BEGIN_MAIN_FRAME;
        }
        if (self.ShouldRunPostCommit()) {
            return Action::POST_COMMIT;
        }
        if (self.ShouldActivateSyncTree()) {
            return Action::ACTIVATE_SYNC_TREE;
        }
        if (self.ShouldCommit()) {
            return Action::COMMIT;
        }
        if (self.ShouldDraw()) {
            if (self.PendingDrawsShouldBeAborted()) {
                return Action::DRAW_ABORT;
            } else if (self.forced_redraw_state_ == ForcedRedrawOnTimeoutState::WAITING_FOR_DRAW) {
                return Action::DRAW_FORCED;
            } else {
                return Action::DRAW_IF_POSSIBLE;
            }
        }
        if (self.ShouldPerformImplSideInvalidation()) {
            return Action::PERFORM_IMPL_SIDE_INVALIDATION;
        }
        if (self.ShouldPrepareTiles()) {
            return Action::PREPARE_TILES;
        }
        if (self.ShouldInvalidateLayerTreeFrameSink()) {
            return Action::INVALIDATE_LAYER_TREE_FRAME_SINK;
        }
        if (self.ShouldBeginLayerTreeFrameSinkCreation()) {
            return Action::BEGIN_LAYER_TREE_FRAME_SINK_CREATION;
        }
        return Action::NONE;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:767-803
    pub fn ShouldPerformImplSideInvalidation(&self) -> bool {
        if (self.pause_rendering_) {
            return false;
        }

        if (self.begin_frame_is_animate_only_) {
            return false;
        }

        if (!self.needs_impl_side_invalidation_) {
            return false;
        }

        // Don't invalidate if we've already done so either from the scheduler or as
        // part of commit.
        if (self.did_perform_impl_side_invalidation_) {
            return false;
        }

        // No invalidations should be done outside the impl frame.
        if (self.begin_impl_frame_state_ == BeginImplFrameState::IDLE) {
            return false;
        }

        // We need to be able to create a pending tree to perform an invalidation.
        if (!self.CouldCreatePendingTree()) {
            return false;
        }

        // Check if we should defer invalidating so we can merge these invalidations
        // with the main frame.
        if (self.ShouldDeferInvalidatingForMainFrame()) {
            return false;
        }

        // If invalidations go to the active tree and we are waiting for the previous
        // frame to be drawn, submitted and acked.
        if (self.settings_.commit_to_active_tree
            && (self.active_tree_needs_first_draw_ || self.IsDrawThrottled()))
        {
            return false;
        }

        return true;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:805-846
    pub fn ShouldDeferInvalidatingForMainFrame(&self) -> bool {
        debug_assert_ne!(self.begin_impl_frame_state_, BeginImplFrameState::IDLE);

        // If the main thread is ready to commit, the impl-side invalidations will be
        // merged with the incoming main frame.
        if (self.begin_main_frame_state_ == BeginMainFrameState::READY_TO_COMMIT) {
            return true;
        }

        // If commits are being aborted (which would be the common case for a
        // compositor scroll), don't defer the invalidation.
        if (self.last_frame_events_.commit_had_no_updates || self.last_commit_had_no_updates_) {
            return false;
        }

        // If we prefer to invalidate over waiting on the main frame, do the
        // invalidation now.
        if (!self.should_defer_invalidation_for_fast_main_frame_) {
            return false;
        }

        // If there is a request for a main frame, then this could either be a
        // request that we need to respond to in this impl frame or its possible the
        // request is for the next frame (a rAF issued at the beginning of the current
        // main frame). In either case, defer invalidating so we can merge it with the
        // main frame.
        if (self.needs_begin_main_frame_) {
            return true;
        }

        // If the main frame was already sent, wait for the main thread to respond.
        if (self.begin_main_frame_state_ == BeginMainFrameState::SENT) {
            return true;
        }

        // If the main thread committed during the last frame, i.e. it was not
        // aborted, then we might get another main frame request later in the impl
        // frame. This could be the case for a timer based animation running on the
        // main thread which doesn't align with our vsync. For such cases,
        // conservatively defer invalidating until the deadline.
        if (self.last_frame_events_.did_commit_during_frame) {
            return true;
        }

        // If the main thread is not requesting any frames, perform the invalidation
        // at the beginning of the impl frame.
        return false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:848-851
    pub fn WillPerformImplSideInvalidation(&mut self) {
        self.current_pending_tree_is_impl_side_ = true;
        self.WillPerformImplSideInvalidationInternal();
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:853-865
    pub fn WillPerformImplSideInvalidationInternal(&mut self) {
        debug_assert!(self.needs_impl_side_invalidation_);
        debug_assert!(!self.has_pending_tree_);

        self.needs_impl_side_invalidation_ = false;
        self.has_pending_tree_ = true;
        self.did_perform_impl_side_invalidation_ = true;
        self.pending_tree_needs_first_draw_on_activation_ =
            self.next_invalidation_needs_first_draw_on_activation_;
        self.next_invalidation_needs_first_draw_on_activation_ = false;
        // TODO(eseckler): Track impl-side invalidations for pending/active tree and
        // CompositorFrame freshness computation.
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:867-890
    pub fn CouldCreatePendingTree(&self) -> bool {
        // Can't create a new pending tree till the current one is activated.
        if (self.has_pending_tree_) {
            return false;
        }

        // Can't make frames while we're invisible.
        if (!self.visible_) {
            return false;
        }

        // If the viz::BeginFrameSource is paused, we will not be able to make any
        // impl frames.
        if (self.begin_frame_source_paused_) {
            return false;
        }

        // Don't create a pending tree till a frame sink is fully initialized.  Check
        // for the ACTIVE state explicitly instead of calling
        // HasInitializedLayerTreeFrameSink() because that only checks if frame sink
        // has been recreated, but doesn't check if we're waiting for first commit or
        // activation.
        if (self.layer_tree_frame_sink_state_ != LayerTreeFrameSinkState::ACTIVE) {
            return false;
        }

        return true;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:892-918
    pub fn WillSendBeginMainFrame(&mut self) {
        debug_assert!(
            !self.has_pending_tree_
                || self.settings_.main_frame_before_activation_enabled
                || self.current_pending_tree_is_impl_side_
        );
        debug_assert!(self.visible_);
        debug_assert!(!self.begin_frame_source_paused_);
        debug_assert!(
            !self.did_send_begin_main_frame_for_current_frame_
                || self.urgent_begin_main_frame_pending_
        );
        if (self.begin_main_frame_state_ == BeginMainFrameState::IDLE) {
            self.begin_main_frame_state_ = BeginMainFrameState::SENT;
        } else {
            // We are sending BMF to the main thread while the previous BMF has not yet
            // finished commit on the impl thread.
            debug_assert!(self.settings_.main_frame_before_commit_enabled);
            debug_assert_eq!(
                self.begin_main_frame_state_,
                BeginMainFrameState::READY_TO_COMMIT
            );
            debug_assert_eq!(self.next_begin_main_frame_state_, BeginMainFrameState::IDLE);
            self.next_begin_main_frame_state_ = BeginMainFrameState::SENT;
        }
        self.needs_begin_main_frame_ = false;
        self.did_send_begin_main_frame_for_current_frame_ = true;
        // TODO(szager): Make sure this doesn't break perfetto
        self.last_frame_number_begin_main_frame_sent_ = self.current_frame_number_;
        // Not setting this to Now(), as we align everything on the last impl frame,
        // in order to avoid the effects of delay in-between BeginImplFrame and
        // SendBeginMainFrame(), that might lead to frame pacing issues.
        self.last_sent_begin_main_frame_time_ = self.last_begin_impl_frame_time_;
        self.urgent_begin_main_frame_pending_ = false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:920-922
    pub fn CheckWillCommitBase(&self) -> bool {
        return true;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:924-994
    pub fn WillCommit(&mut self, commit_has_no_updates: bool) {
        if commit_has_no_updates {
            self.consecutive_no_damage_main_frames_ += 1;
        } else {
            self.consecutive_no_damage_main_frames_ = 0;
        }
        self.UpdateConsecutiveNoDamageThrottlingInterval();
        let can_have_pending_tree = commit_has_no_updates
            && (self.settings_.main_frame_before_activation_enabled
                || self.settings_.main_frame_before_commit_enabled
                || self.current_pending_tree_is_impl_side_);
        debug_assert!(!self.has_pending_tree_ || can_have_pending_tree);
        debug_assert!(
            self.settings_.main_frame_before_commit_enabled
                || self.next_begin_main_frame_state_ == BeginMainFrameState::IDLE
        );
        debug_assert!(self.next_begin_main_frame_state_ < BeginMainFrameState::READY_TO_COMMIT);
        self.commit_count_ += 1;
        if commit_has_no_updates {
            // The primary BMF was aborted and cannot have a pipelined BMF.
            debug_assert_eq!(self.next_begin_main_frame_state_, BeginMainFrameState::IDLE);
            self.begin_main_frame_state_ = BeginMainFrameState::IDLE;
        } else {
            // Move the pipelined BMF into the primary slot being vacated.
            debug_assert!(
                self.settings_.main_frame_before_commit_enabled
                    || self.next_begin_main_frame_state_ == BeginMainFrameState::IDLE
            );
            debug_assert_ne!(
                self.next_begin_main_frame_state_,
                BeginMainFrameState::READY_TO_COMMIT
            );
            self.begin_main_frame_state_ = self.next_begin_main_frame_state_;
            self.next_begin_main_frame_state_ = BeginMainFrameState::IDLE;
        }
        self.last_commit_had_no_updates_ = commit_has_no_updates;
        self.did_commit_during_frame_ = true;
        if !commit_has_no_updates {
            if self.needs_impl_side_invalidation_ {
                self.WillPerformImplSideInvalidationInternal();
            }
            self.did_perform_impl_side_invalidation_ = true;
            self.has_pending_tree_ = true;
            self.pending_tree_needs_first_draw_on_activation_ = true;
            self.pending_tree_is_ready_for_activation_ = false;
            if self.CheckWillCommit() {
                self.active_tree_is_ready_to_draw_ = false;
            }
        }
        if self.forced_redraw_state_ == ForcedRedrawOnTimeoutState::WAITING_FOR_COMMIT {
            self.forced_redraw_state_ = if self.has_pending_tree_ {
                ForcedRedrawOnTimeoutState::WAITING_FOR_ACTIVATION
            } else {
                ForcedRedrawOnTimeoutState::WAITING_FOR_DRAW
            };
        }
        if self.layer_tree_frame_sink_state_ == LayerTreeFrameSinkState::WAITING_FOR_FIRST_COMMIT {
            self.layer_tree_frame_sink_state_ = if self.has_pending_tree_ {
                LayerTreeFrameSinkState::WAITING_FOR_FIRST_ACTIVATION
            } else {
                LayerTreeFrameSinkState::ACTIVE
            };
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:996-1019
    pub fn WillActivate(&mut self) {
        // We cannot activate the pending tree while paint worklets are still being
        // processed; the pending tree *must* be fully painted before it can ever be
        // activated because we cannot paint the active tree.
        debug_assert!(!self.processing_paint_worklets_for_pending_tree_);

        if (self.layer_tree_frame_sink_state_
            == LayerTreeFrameSinkState::WAITING_FOR_FIRST_ACTIVATION)
        {
            self.layer_tree_frame_sink_state_ = LayerTreeFrameSinkState::ACTIVE;
        }

        if (self.forced_redraw_state_ == ForcedRedrawOnTimeoutState::WAITING_FOR_ACTIVATION) {
            self.forced_redraw_state_ = ForcedRedrawOnTimeoutState::WAITING_FOR_DRAW;
        }

        self.has_pending_tree_ = false;
        self.pending_tree_is_ready_for_activation_ = false;
        self.needs_redraw_ = true;
        self.active_tree_needs_first_draw_ = self.pending_tree_needs_first_draw_on_activation_;
        self.pending_tree_needs_first_draw_on_activation_ = false;
        self.waiting_for_activation_after_rendering_resumed_ = false;

        self.previous_pending_tree_was_impl_side_ = self.current_pending_tree_is_impl_side_;
        self.current_pending_tree_is_impl_side_ = false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1021-1041
    pub fn WillDrawInternal(&mut self) {
        // If a new active tree is pending after the one we are about to draw,
        // the main thread is in a high latency mode.
        // main_thread_missed_last_deadline_ is here in addition to
        // OnBeginImplFrameIdle for cases where the scheduler aborts draws outside
        // of the deadline.
        self.main_thread_missed_last_deadline_ = self.CommitPending()
            || (self.has_pending_tree_ && !self.current_pending_tree_is_impl_side_);

        // We need to reset needs_redraw_ before we draw since the
        // draw itself might request another draw.
        self.needs_redraw_ = false;

        self.did_draw_ = true;
        self.active_tree_needs_first_draw_ = false;
        self.last_frame_number_draw_performed_ = self.current_frame_number_;

        if (self.forced_redraw_state_ == ForcedRedrawOnTimeoutState::WAITING_FOR_DRAW) {
            self.forced_redraw_state_ = ForcedRedrawOnTimeoutState::IDLE;
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1043-1090
    pub fn DidDrawInternal(&mut self, draw_result: DrawResult) {
        match draw_result {
            DrawResult::kInvalidResult => unreachable!(
                "Invalid return DrawResult: {}",
                DrawResult::kInvalidResult as i32
            ),
            DrawResult::kAbortedCantDraw => {
                let previous_count = self.consecutive_cant_draw_count_;
                self.consecutive_cant_draw_count_ += 1;
                if previous_count < 3 {
                    self.needs_redraw_ = true;
                } else if !self.settings_.using_synchronous_renderer_compositor {
                    self.services_.Diagnostic(
                        "consecutive draws with DrawResult::kAbortedCantDraw result",
                        self.consecutive_cant_draw_count_,
                    );
                }
            }
            DrawResult::kAbortedDrainingPipeline | DrawResult::kSuccess => {
                self.consecutive_checkerboard_animations_ = 0;
                self.consecutive_cant_draw_count_ = 0;
                self.forced_redraw_state_ = ForcedRedrawOnTimeoutState::IDLE;
            }
            DrawResult::kAbortedCheckerboardAnimations => {
                debug_assert!(!self.did_submit_in_last_frame_);
                self.needs_begin_main_frame_ = true;
                self.needs_redraw_ = true;
                self.consecutive_checkerboard_animations_ += 1;
                self.consecutive_cant_draw_count_ = 0;
                if self.consecutive_checkerboard_animations_
                    >= self
                        .settings_
                        .maximum_number_of_failed_draws_before_draw_is_forced
                    && self.forced_redraw_state_ == ForcedRedrawOnTimeoutState::IDLE
                {
                    self.forced_redraw_state_ = ForcedRedrawOnTimeoutState::WAITING_FOR_COMMIT;
                }
            }
            DrawResult::kAbortedMissingHighResContent => {
                debug_assert!(!self.did_submit_in_last_frame_);
                self.needs_begin_main_frame_ = true;
                self.consecutive_cant_draw_count_ = 0;
            }
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1092-1099
    pub fn WillDraw(&mut self) {
        debug_assert!(!self.did_draw_);
        self.WillDrawInternal();
        // Set this to true to proactively request a new BeginFrame. We can't set this
        // in WillDrawInternal because AbortDraw calls WillDrawInternal but shouldn't
        // request another frame.
        self.did_attempt_draw_in_last_frame_ = true;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1101-1104
    pub fn DidDraw(&mut self, draw_result: DrawResult) {
        self.draw_succeeded_in_last_frame_ = draw_result == DrawResult::kSuccess;
        self.DidDrawInternal(draw_result);
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1106-1111
    pub fn SetNeedsImplSideInvalidation(&mut self, needs_first_draw_on_activation: bool) {
        self.needs_impl_side_invalidation_ = true;
        self.next_invalidation_needs_first_draw_on_activation_ |= needs_first_draw_on_activation;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1113-1122
    pub fn AbortDraw(&mut self) {
        if (self.begin_frame_source_paused_) {
            self.draw_aborted_for_paused_begin_frame_ = true;
        }
        // Pretend like the draw was successful.
        // Note: We may abort at any time and cannot DCHECK that
        // we haven't drawn in or swapped in the last frame here.
        self.WillDrawInternal();
        self.DidDrawInternal(DrawResult::kAbortedDrainingPipeline);
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1124-1126
    pub fn WillPrepareTiles(&mut self) {
        self.needs_prepare_tiles_ = false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1128-1141
    pub fn WillBeginLayerTreeFrameSinkCreation(&mut self) {
        debug_assert_eq!(
            self.layer_tree_frame_sink_state_,
            LayerTreeFrameSinkState::NONE
        );
        self.layer_tree_frame_sink_state_ = LayerTreeFrameSinkState::CREATING;

        // The following DCHECKs make sure we are in the proper quiescent state.
        // The pipeline should be flushed entirely before we start output
        // surface creation to avoid complicated corner cases.
        debug_assert!(self.begin_main_frame_state_ == BeginMainFrameState::IDLE);
        debug_assert!(self.next_begin_main_frame_state_ == BeginMainFrameState::IDLE);
        debug_assert!(!self.has_pending_tree_);
        debug_assert!(!self.active_tree_needs_first_draw_);

        self.should_warm_up_ = false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1143-1148
    pub fn WillInvalidateLayerTreeFrameSink(&mut self) {
        debug_assert!(!self.did_invalidate_layer_tree_frame_sink_);
        self.did_invalidate_layer_tree_frame_sink_ = true;
        self.last_frame_number_invalidate_layer_tree_frame_sink_performed_ =
            self.current_frame_number_;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1150-1152
    pub fn BeginFrameNeededForVideo(&self) -> bool {
        return self.video_needs_begin_frames_;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1154-1177
    pub fn BeginFrameNeeded(&self) -> bool {
        // If we shouldn't subscribe to BeginFrames it implies BeginFrames are not
        // needed.
        if (!self.ShouldSubscribeToBeginFrames()) {
            return false;
        }

        if (!self.pause_rendering_) {
            return true;
        }

        // Drain any in-flight main frame updates before pausing impl frames.
        if (self.begin_main_frame_state_ != BeginMainFrameState::IDLE) {
            return true;
        }

        // If a pending tree exists, activate and draw before pausing impl frames.
        if (self.has_pending_tree_) {
            return true;
        }

        // If a newly activated tree hasn't been drawn yet, draw it before pausing
        // impl frames.
        if (self.active_tree_needs_first_draw_) {
            return true;
        }

        return false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1179-1191
    pub fn ShouldSubscribeToBeginFramesBase(&self) -> bool {
        // We can't handle BeginFrames when output surface isn't initialized.
        // TODO(brianderson): Support output surface creation inside a BeginFrame.
        if (!self.HasInitializedLayerTreeFrameSink()) {
            return false;
        }

        // If we are not visible, we don't need BeginFrame messages.
        if (!self.visible_) {
            return false;
        }

        return self.BeginFrameRequiredForAction()
            || self.BeginFrameNeededForVideo()
            || self.ProactiveBeginFrameWanted();
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1193-1196
    pub fn SetVideoNeedsBeginFrames(&mut self, video_needs_begin_frames: bool) {
        self.video_needs_begin_frames_ = video_needs_begin_frames;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1198-1201
    pub fn SetDeferBeginMainFrame(&mut self, defer_begin_main_frame: bool) {
        self.defer_begin_main_frame_ = defer_begin_main_frame;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1203-1212
    pub fn SetPauseRendering(&mut self, pause_rendering: bool) {
        if (self.pause_rendering_ == pause_rendering) {
            return;
        }

        self.pause_rendering_ = pause_rendering;
        // When resuming rendering, main thread always commits at least one frame.
        // Dont draw any impl frames until this commit is activated.
        self.waiting_for_activation_after_rendering_resumed_ = !self.pause_rendering_;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1214-1225
    pub fn BeginFrameRequiredForAction(&self) -> bool {
        // The forced draw respects our normal draw scheduling, so we need to
        // request a BeginImplFrame for it.
        if (self.forced_redraw_state_ == ForcedRedrawOnTimeoutState::WAITING_FOR_DRAW) {
            return true;
        }

        return self.needs_redraw_
            || self.needs_one_begin_impl_frame_
            || (self.needs_begin_main_frame_ && !self.defer_begin_main_frame_)
            || self.needs_impl_side_invalidation_;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1227-1275
    pub fn ProactiveBeginFrameWanted(&self) -> bool {
        // Do not be proactive when invisible.
        if (!self.visible_) {
            return false;
        }

        // We should proactively request a BeginImplFrame if a commit is pending
        // because we will want to draw if the commit completes quickly. Do not
        // request frames when commits are disabled, because the frame requests will
        // not provide the needed commit (and will wake up the process when it could
        // stay idle).
        if ((self.begin_main_frame_state_ != BeginMainFrameState::IDLE)
            && !self.defer_begin_main_frame_)
        {
            return true;
        }

        // If the pending tree activates quickly, we'll want a BeginImplFrame soon
        // to draw the new active tree.
        if (self.has_pending_tree_) {
            return true;
        }

        // Changing priorities may allow us to activate (given the new priorities),
        // which may result in a new frame.
        if (self.needs_prepare_tiles_) {
            return true;
        }

        // If we just tried to draw, it's likely that we are going to produce another
        // frame soon. This helps avoid negative glitches in our SetNeedsBeginFrame
        // requests, which may propagate to the BeginImplFrame provider and get
        // sampled at an inopportune time, delaying the next BeginImplFrame.
        if (self.did_attempt_draw_in_last_frame_) {
            return true;
        }

        // If the last commit was aborted because of early out (no updates), we should
        // still want a begin frame in case there is a commit coming again.
        if (self.last_commit_had_no_updates_) {
            return true;
        }

        // If there is active interaction happening (e.g. scroll/pinch), then keep
        // reqeusting frames.
        if (self.tree_priority_ == TreePriority::SMOOTHNESS_TAKES_PRIORITY) {
            return true;
        }

        return false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1277-1307
    pub fn OnBeginImplFrameBase(&mut self, args: &BeginFrameArgs) {
        self.begin_impl_frame_state_ = BeginImplFrameState::INSIDE_BEGIN_FRAME;
        self.current_frame_number_ += 1;
        self.begin_frame_is_animate_only_ = args.animate_only;
        // Pin the timestamp as passed from the caller. This makes timestamps more
        // consistent, and insensitive to e.g. descheduling between receiving a
        // BeginFrame() call, and actually getting to BeginImplFrame().
        self.last_begin_impl_frame_time_ = args.frame_time;

        // Cache the values from the previous impl frame before reseting them for this
        // frame.
        self.last_frame_events_.commit_had_no_updates = self.last_commit_had_no_updates_;
        self.last_frame_events_.did_commit_during_frame = self.did_commit_during_frame_;

        self.last_commit_had_no_updates_ = false;
        self.did_attempt_draw_in_last_frame_ = false;
        self.draw_succeeded_in_last_frame_ = false;
        self.did_submit_in_last_frame_ = false;
        self.needs_one_begin_impl_frame_ = false;

        self.did_send_begin_main_frame_for_current_frame_ = false;
        self.did_commit_during_frame_ = false;
        self.did_invalidate_layer_tree_frame_sink_ = false;
        self.did_perform_impl_side_invalidation_ = false;
        self.waiting_for_scroll_event_ = false;

        if (self.services_.ShouldRecordSubsampledMetric(0.001)) {
            self.services_.HistogramBoolean(
                "Compositing.Scheduler.HighFramerateRequested",
                self.high_framerate_requests_count_ > 0,
            );
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1309-1314
    pub fn OnBeginImplFrameDeadline(&mut self) {
        self.begin_impl_frame_state_ = BeginImplFrameState::INSIDE_DEADLINE;

        // Clear funnels for any actions we perform during the deadline.
        self.did_draw_ = false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1316-1334
    pub fn OnBeginImplFrameIdle(&mut self) {
        self.begin_impl_frame_state_ = BeginImplFrameState::IDLE;

        // Count any prepare tiles that happens in commits in between frames. We want
        // to prevent a prepare tiles during the next frame's deadline in that case.
        // This also allows synchronous compositor to do one PrepareTiles per draw.
        // This is same as the old prepare tiles funnel behavior.
        self.did_prepare_tiles_ = false;

        // If a new or undrawn active tree is pending after the deadline,
        // then the main thread is in a high latency mode.
        self.main_thread_missed_last_deadline_ =
            self.CommitPending() || self.has_pending_tree_ || self.active_tree_needs_first_draw_;

        // If we're entering a state where we won't get BeginFrames set all the
        // funnels so that we don't perform any actions that we shouldn't.
        if (!self.BeginFrameNeeded()) {
            self.did_send_begin_main_frame_for_current_frame_ = true;
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1336-1365
    pub fn CurrentBeginImplFrameDeadlineModeBase(&self) -> BeginImplFrameDeadlineMode {
        let mut outside_begin_frame =
            self.begin_impl_frame_state_ != BeginImplFrameState::INSIDE_BEGIN_FRAME;
        if (self.settings_.using_synchronous_renderer_compositor || outside_begin_frame) {
            // No deadline for synchronous compositor, or when outside the begin frame.
            return BeginImplFrameDeadlineMode::NONE;
        } else if (self.ShouldBlockDeadlineIndefinitely()) {
            // We do not want to wait for a deadline because we're waiting for full
            // pipeline to be flushed for headless.
            return BeginImplFrameDeadlineMode::BLOCKED;
        } else if (self.ShouldTriggerBeginImplFrameDeadlineImmediately()) {
            if (self.ShouldWaitForScrollEvent()) {
                // We are scrolling but have not received a scroll event for this begin
                // frame. We want to wait before attempting to draw.
                return BeginImplFrameDeadlineMode::WAIT_FOR_SCROLL;
            }
            // We are ready to draw a new active tree immediately because there's no
            // commit expected or we're prioritizing active tree latency.
            return BeginImplFrameDeadlineMode::IMMEDIATE;
        } else if (self.needs_redraw_) {
            // We have an animation or fast input path on the impl thread that wants
            // to draw, so don't wait too long for a new active tree.
            return BeginImplFrameDeadlineMode::REGULAR;
        } else {
            // The impl thread doesn't have anything it wants to draw and we are just
            // waiting for a new active tree.
            return BeginImplFrameDeadlineMode::LATE;
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1367-1382
    pub fn ShouldWaitForScrollEvent(&self) -> bool {
        // We only apply this mode during frame production
        if (self.begin_impl_frame_state_ != BeginImplFrameState::INSIDE_BEGIN_FRAME) {
            return false;
        }
        // Once the deadline has been triggered, we should stop waiting.
        if (self.begin_impl_frame_state_ == BeginImplFrameState::INSIDE_DEADLINE) {
            return false;
        }
        // We are scrolling but have not received a scroll event for this begin frame.
        if (self.settings_.scroll_deadline_mode_enabled
            && self.is_scrolling_
            && self.waiting_for_scroll_event_)
        {
            return true;
        }
        return false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1384-1433
    pub fn ShouldTriggerBeginImplFrameDeadlineImmediatelyBase(&self) -> bool {
        // If we aborted the current frame we should end the deadline right now.
        if (self.ShouldAbortCurrentFrame() && !self.has_pending_tree_) {
            return true;
        }

        // Throttle the deadline on CompositorFrameAck since we wont draw and submit
        // anyway.
        if (self.IsDrawThrottled()) {
            return false;
        }

        // Delay immediate draws when we have pending animation worklet updates to
        // give them time to produce output before we draw.
        if (self.processing_animation_worklets_for_active_tree_ != 0) {
            return false;
        }

        if (self.active_tree_needs_first_draw_) {
            return true;
        }

        // TODO(XXX): This condition should not need to be there. There was an attempt
        // to remove it, which was reverted due to regressions (see
        // https://chromium-review.googlesource.com/c/chromium/src/+/1211664).
        // Investigate why this is the case.
        if (!self.needs_redraw_) {
            return false;
        }

        // We did not send a BeginMainFrame() and main is not busy, we can trigger the
        // deadline immediately.
        let mut wait_for_main = self.CommitPending() || self.has_pending_tree_;
        if (self.last_begin_impl_frame_time_ != self.last_sent_begin_main_frame_time_
            && !wait_for_main)
        {
            self.trace(
                SchedulerTraceKind::Instant,
                "cc",
                "TriggerDeadlineDueToThrottling",
                || vec![],
            );
            return true;
        }

        // This is used to prioritize impl-thread draws when the main thread isn't
        // producing anything, e.g., after an aborted commit. We also check that we
        // don't have a pending tree -- otherwise we should give it a chance to
        // activate.
        // TODO(skyostil): Revisit this when we have more accurate deadline estimates.
        if (!wait_for_main) {
            return true;
        }

        // Prioritize impl-thread draws in ImplLatencyTakesPriority mode.
        if (self.ImplLatencyTakesPriority()) {
            return true;
        }

        return false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1435-1437
    pub fn CheckShouldBlockDeadlineIndefinitelyBase(&self) -> bool {
        return !self.settings_.commit_to_active_tree;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1439-1474
    pub fn ShouldBlockDeadlineIndefinitely(&self) -> bool {
        if (self.CheckShouldBlockDeadlineIndefinitely()) {
            return false;
        }

        // Avoid blocking for any reason if we don't have a layer tree frame sink or
        // are invisible.
        if (self.layer_tree_frame_sink_state_ == LayerTreeFrameSinkState::NONE) {
            return false;
        }

        if (!self.visible_) {
            return false;
        }

        // Wait for main frame if one is in progress or about to be started.
        if (self.ShouldSendBeginMainFrame()) {
            return true;
        }

        if (self.begin_main_frame_state_ != BeginMainFrameState::IDLE) {
            return true;
        }

        // Wait for tiles and activation.
        if (self.has_pending_tree_) {
            return true;
        }

        // Avoid blocking for draw when we can't draw. We block in the above cases
        // even if we cannot draw, because we may still be waiting for the first
        // active tree.
        if (!self.can_draw_) {
            return false;
        }

        // Wait for remaining tiles and draw.
        if (!self.active_tree_is_ready_to_draw_) {
            return true;
        }

        return false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1476-1505
    pub fn FrameIntervalUpdated(&mut self, frame_interval: TimeDelta) {
        self.unthrottled_frame_interval_ = frame_interval;
        // We only want to evaluate the feature on clients that see short VSync
        // intervals, to have the control and experiment only contain high refresh
        // rate clients. This ignores 90Hz displays, on purpose, and adds some
        // leeway.
        //
        // Apply some slack, so that if for some reason the interval is a bit larger
        // than 8.33333333333333ms, then we catch it still.
        let mut fast_vsync_interval = frame_interval
            < TimeDelta::Microseconds(1_000_000 / 120) * (1_f32 / kFrameThrottlingSlackFactor);
        if (fast_vsync_interval && IsEligibleToThrottleMainFrameRate(&self.services_.features())) {
            self.services_.SetIsEligibleForThrottleMainFrameTo60Hz(true);
        }
        if (fast_vsync_interval
            && ShouldThrottleMainFrameRate(&self.settings_, &self.services_.features()))
        {
            // Here as well, use a slack factor, to make sure that small timing
            // variations don't result in uneven pacing.
            //
            // Use interval / 2 rather than an actual interval as refresh rates are
            // not necessarily 120: it could be something really close, or it could be
            // 144Hz for instance.
            self.main_frame_throttled_interval_ = ThrottledFrameRateWithSlack(frame_interval, 2);
            let _trace_event = self.trace_scope("cc", "ThrottleMainFrame", || {
                vec![("interval", self.main_frame_throttled_interval_.into())]
            });
        } else {
            self.main_frame_throttled_interval_ = TimeDelta::default();
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1507-1513
    pub fn IsDrawThrottled(&self) -> bool {
        if (self.services_.features().kNoCompositorFrameAcks) {
            return false;
        }
        return self.pending_submit_frames_ >= kMaxPendingSubmitFrames
            && !self.settings_.disable_frame_rate_limit;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1515-1527
    pub fn SetVisible(&mut self, visible: bool) {
        if (self.visible_ == visible) {
            return;
        }

        self.visible_ = visible;

        if (visible) {
            self.main_thread_missed_last_deadline_ = false;
            self.should_warm_up_ = false;
        }

        self.did_prepare_tiles_ = false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1529-1531
    pub fn SetShouldWarmUp(&mut self) {
        self.should_warm_up_ = true;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1533-1539
    pub fn SetBeginFrameSourcePaused(&mut self, paused: bool) {
        self.begin_frame_source_paused_ = paused;
        if (!paused) {
            self.needs_redraw_ = self.draw_aborted_for_paused_begin_frame_;
            self.draw_aborted_for_paused_begin_frame_ = false;
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1541-1544
    pub fn SetResourcelessSoftwareDraw(&mut self, resourceless_draw: bool) {
        self.resourceless_draw_ = resourceless_draw;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1546-1548
    pub fn SetCanDraw(&mut self, can_draw: bool) {
        self.can_draw_ = can_draw;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1550-1552
    pub fn SetSkipDraw(&mut self, skip_draw: bool) {
        self.skip_draw_ = skip_draw;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1554-1556
    pub fn SetNeedsRedraw(&mut self) {
        self.needs_redraw_ = true;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1558-1563
    pub fn SetNeedsPrepareTiles(&mut self) {
        if (!self.needs_prepare_tiles_) {
            let _trace_event = self.trace_scope(
                "cc",
                "SchedulerStateMachine::SetNeedsPrepareTiles",
                || vec![],
            );
            self.needs_prepare_tiles_ = true;
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1564-1590
    pub fn DidSubmitCompositorFrame(&mut self) {
        if (!self.services_.features().kNoCompositorFrameAcks) {
            self.trace(
                SchedulerTraceKind::Begin,
                "cc",
                "Scheduler:pending_submit_frames",
                || {
                    vec![(
                        "pending_frames",
                        crate::adapter::TraceValue::Integer(i64::from(self.pending_submit_frames_)),
                    )]
                },
            );

            // If we are running with no frame rate limits, the GPU process can submit
            // a new BeginFrame request if the deadline for the pending BeginFrame
            // request expires. It will basically cause this DCHECK to fire as we may
            // not have received acks for previously submitted requests.
            // Please see SchedulerStateMachine::IsDrawThrottled() where throttling
            // is disabled when the disable_frame_rate_limit setting is enabled.
            // TODO(ananta/jonross/sunnyps)
            // http://crbug.com/346931323
            // We should remove or change this once VRR support is implemented for
            // Windows and other platforms potentially.
            if (!self.settings_.disable_frame_rate_limit) {
                debug_assert!(self.pending_submit_frames_ < kMaxPendingSubmitFrames);
            }

            self.pending_submit_frames_ += 1;
        }
        self.submit_frames_with_current_layer_tree_frame_sink_ += 1;

        self.did_submit_in_last_frame_ = true;
        self.last_frame_number_submit_performed_ = self.current_frame_number_;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1592-1601
    pub fn DidReceiveCompositorFrameAck(&mut self) {
        if self.services_.features().kNoCompositorFrameAcks {
            unreachable!("Unexpected CompositorFrameAck with kNoCompositorFrameAcks");
        } else {
            self.trace(
                SchedulerTraceKind::End,
                "cc",
                "Scheduler:pending_submit_frames",
                || {
                    vec![(
                        "pending_frames",
                        crate::adapter::TraceValue::Integer(i64::from(self.pending_submit_frames_)),
                    )]
                },
            );
            self.pending_submit_frames_ -= 1;
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1603-1608
    pub fn SetTreePrioritiesAndScrollState(
        &mut self,
        tree_priority: TreePriority,
        is_current_scroll_main_painted: bool,
    ) {
        self.tree_priority_ = tree_priority;
        self.is_current_scroll_main_painted_ = is_current_scroll_main_painted;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1610-1613
    pub fn SetCriticalBeginMainFrameToActivateIsFast(&mut self, is_fast: bool) {
        self.critical_begin_main_frame_to_activate_is_fast_ = is_fast;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1615-1625
    pub fn ImplLatencyTakesPriority(&self) -> bool {
        // Don't wait for the main thread if we are prioritizing smoothness.
        // We do not attempt to synchronize with the main thread for scroll handlers.
        // Even when `critical_begin_main_frame_to_activate_is_fast_` the threshold
        // for that calculation does not account for GPU process time required. Due to
        // this we end up having significant scroll jank when synchronizing.
        if (TreePriority::SMOOTHNESS_TAKES_PRIORITY == self.tree_priority_) {
            return true;
        }

        return false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1627-1640
    pub fn SetNeedsBeginMainFrame(&mut self, now: bool, unthrottled: bool) {
        let _trace_event = self.trace_scope(
            "cc",
            "SchedulerStateMachine::SetNeedsBeginMainFrame",
            || {
                vec![
                    ("now", crate::adapter::TraceValue::Bool(now)),
                    ("unthrottled", crate::adapter::TraceValue::Bool(unthrottled)),
                ]
            },
        );
        self.needs_begin_main_frame_ = true;

        if (unthrottled) {
            // Reset the throttling interval for the next frame only.
            self.main_frame_consecutive_no_damage_throttled_interval_ = TimeDelta::default();
        }

        if (now) {
            self.last_sent_begin_main_frame_time_ = TimeTicks::default();
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1642-1645
    pub fn SetUrgentBeginMainFramePending(&mut self) {
        self.urgent_begin_main_frame_pending_ = true;
        self.SetNeedsBeginMainFrame(true, false);
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1647-1649
    pub fn SetNeedsOneBeginImplFrame(&mut self) {
        self.needs_one_begin_impl_frame_ = true;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1651-1657
    pub fn NotifyReadyToCommit(&mut self) {
        debug_assert_eq!(self.begin_main_frame_state_, BeginMainFrameState::SENT);
        self.begin_main_frame_state_ = BeginMainFrameState::READY_TO_COMMIT;
        // In commit_to_active_tree mode, commit should happen right after BeginFrame,
        // meaning when this function is called, next action should be commit.
        debug_assert!(!self.settings_.commit_to_active_tree || self.ShouldCommit());
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1659-1706
    pub fn BeginMainFrameAborted(&mut self, reason: CommitEarlyOutReason) {
        if self.begin_main_frame_state_ == BeginMainFrameState::SENT {
            debug_assert_eq!(self.next_begin_main_frame_state_, BeginMainFrameState::IDLE);
            self.main_thread_missed_last_deadline_ = false;
            match reason {
                CommitEarlyOutReason::kAbortedNotVisible
                | CommitEarlyOutReason::kAbortedDeferredMainFrameUpdate
                | CommitEarlyOutReason::kAbortedDeferredCommit => {
                    self.begin_main_frame_state_ = BeginMainFrameState::IDLE;
                    self.SetNeedsBeginMainFrame(false, false);
                }
                CommitEarlyOutReason::kFinishedNoUpdates => self.WillCommit(true),
                CommitEarlyOutReason::kNoEarlyOut => {
                    unreachable!("Not a real early-out reason, only used for metrics")
                }
            }
        } else {
            debug_assert!(self.settings_.main_frame_before_commit_enabled);
            debug_assert_eq!(self.next_begin_main_frame_state_, BeginMainFrameState::SENT);
            debug_assert_eq!(
                self.begin_main_frame_state_,
                BeginMainFrameState::READY_TO_COMMIT
            );
            self.next_begin_main_frame_state_ = BeginMainFrameState::IDLE;
            match reason {
                CommitEarlyOutReason::kAbortedNotVisible
                | CommitEarlyOutReason::kAbortedDeferredMainFrameUpdate
                | CommitEarlyOutReason::kAbortedDeferredCommit => {
                    self.SetNeedsBeginMainFrame(false, false)
                }
                CommitEarlyOutReason::kFinishedNoUpdates => self.commit_count_ += 1,
                CommitEarlyOutReason::kNoEarlyOut => {
                    unreachable!("Not a real early-out reason, only used for metrics")
                }
            }
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1708-1711
    pub fn DidPrepareTiles(&mut self) {
        self.needs_prepare_tiles_ = false;
        self.did_prepare_tiles_ = true;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1713-1719
    pub fn DidLoseLayerTreeFrameSink(&mut self) {
        if (self.layer_tree_frame_sink_state_ == LayerTreeFrameSinkState::NONE
            || self.layer_tree_frame_sink_state_ == LayerTreeFrameSinkState::CREATING)
        {
            return;
        }
        self.layer_tree_frame_sink_state_ = LayerTreeFrameSinkState::NONE;
        self.needs_redraw_ = false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1721-1733
    pub fn NotifyReadyToActivate(&mut self) -> bool {
        // It is not valid for clients to try and activate the pending tree whilst
        // paint worklets are still being processed; the pending tree *must* be fully
        // painted before it can ever be activated (even if e.g. it is not visible),
        // because we cannot paint the active tree.
        debug_assert!(!self.processing_paint_worklets_for_pending_tree_);

        if (!self.has_pending_tree_ || self.pending_tree_is_ready_for_activation_) {
            return false;
        }

        self.pending_tree_is_ready_for_activation_ = true;
        return true;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1735-1737
    pub fn IsReadyToActivate(&mut self) -> bool {
        return self.pending_tree_is_ready_for_activation_;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1739-1741
    pub fn NotifyReadyToDraw(&mut self) {
        self.active_tree_is_ready_to_draw_ = true;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1743-1763
    pub fn NotifyAnimationWorkletStateChange(
        &mut self,
        state: AnimationWorkletState,
        tree: TreeType,
    ) {
        if tree == TreeType::ACTIVE {
            match state {
                AnimationWorkletState::PROCESSING => {
                    debug_assert!(self.processing_animation_worklets_for_active_tree_ >= 0);
                    debug_assert!(self.processing_animation_worklets_for_active_tree_ <= 1);
                    self.processing_animation_worklets_for_active_tree_ += 1;
                }
                AnimationWorkletState::IDLE => {
                    debug_assert!(self.processing_animation_worklets_for_active_tree_ <= 2);
                    debug_assert!(self.processing_animation_worklets_for_active_tree_ >= 1);
                    self.processing_animation_worklets_for_active_tree_ -= 1;
                }
            }
        } else {
            self.processing_animation_worklets_for_pending_tree_ =
                state == AnimationWorkletState::PROCESSING;
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1765-1773
    pub fn NotifyPaintWorkletStateChange(&mut self, state: PaintWorkletState) {
        let mut processing_paint_worklets_for_pending_tree =
            (state == PaintWorkletState::PROCESSING);
        debug_assert_ne!(
            processing_paint_worklets_for_pending_tree,
            self.processing_paint_worklets_for_pending_tree_
        );
        self.processing_paint_worklets_for_pending_tree_ =
            processing_paint_worklets_for_pending_tree;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1775-1789
    pub fn DidCreateAndInitializeLayerTreeFrameSink(&mut self) {
        debug_assert_eq!(
            self.layer_tree_frame_sink_state_,
            LayerTreeFrameSinkState::CREATING
        );
        self.layer_tree_frame_sink_state_ = LayerTreeFrameSinkState::WAITING_FOR_FIRST_COMMIT;

        if (self.did_create_and_initialize_first_layer_tree_frame_sink_) {
            // TODO(boliu): See if we can remove this when impl-side painting is always
            // on. Does anything on the main thread need to update after recreate?
            self.needs_begin_main_frame_ = true;
        }
        self.did_create_and_initialize_first_layer_tree_frame_sink_ = true;
        self.pending_submit_frames_ = 0;
        self.submit_frames_with_current_layer_tree_frame_sink_ = 0;
        self.main_thread_missed_last_deadline_ = false;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1791-1802
    pub fn HasInitializedLayerTreeFrameSink(&self) -> bool {
        match self.layer_tree_frame_sink_state_ {
            LayerTreeFrameSinkState::NONE | LayerTreeFrameSinkState::CREATING => false,
            LayerTreeFrameSinkState::ACTIVE
            | LayerTreeFrameSinkState::WAITING_FOR_FIRST_COMMIT
            | LayerTreeFrameSinkState::WAITING_FOR_FIRST_ACTIVATION => true,
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1806-1822
    pub fn UpdateConsecutiveNoDamageThrottlingInterval(&mut self) {
        if !self.services_.features().kThrottleRepeatedNoDamageFrames {
            return;
        }
        if self.DisableThrottlingDueToHighFramerateRequests() {
            self.consecutive_no_damage_main_frames_ = 0;
            self.main_frame_consecutive_no_damage_throttled_interval_ = TimeDelta::default();
            return;
        }
        let factor = crate::GetThrottlingFactor(
            self.consecutive_no_damage_main_frames_,
            &self.services_.features(),
        );
        self.main_frame_consecutive_no_damage_throttled_interval_ = if factor != 0 {
            ThrottledFrameRateWithSlack(self.unthrottled_frame_interval_, factor)
        } else {
            TimeDelta::default()
        };
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1824-1833
    pub fn SetRequestHighFramerate(&mut self, flag: bool) {
        let _trace_event = self.trace_scope(
            "blink",
            "SchedulerStateMachine::SetRequestHighFramerate",
            || vec![],
        );
        if (flag) {
            self.high_framerate_requests_count_ += 1;
        } else {
            debug_assert!(self.high_framerate_requests_count_ >= 1_u64);
            self.high_framerate_requests_count_ =
                std::cmp::max(1_u64, self.high_framerate_requests_count_) - 1;
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1835-1840
    pub fn DisableThrottlingDueToHighFramerateRequests(&self) -> bool {
        return self.high_framerate_requests_count_ > 0
            && self.services_.features().kHighFramerateRequestFromClient;
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:1842-1848
    pub fn MainFrameThrottledInterval(&self) -> TimeDelta {
        if (self.DisableThrottlingDueToHighFramerateRequests()) {
            return TimeDelta::default();
        } else {
            return self.main_frame_throttled_interval_;
        }
    }

    // Source: cc/scheduler/scheduler_state_machine.cc:78-82
    fn GetTracingTrack(&self) -> SchedulerTraceTrack {
        SchedulerTraceTrack {
            name: "cc::SchedulerStateMachine",
            id: self.services_.trace_track_id(),
        }
    }

    // Rust lifetime adapter for source scoped TRACE_EVENT macros.
    fn trace_scope(
        &self,
        category: &'static str,
        name: &'static str,
        arguments: impl FnOnce() -> Vec<(&'static str, crate::adapter::TraceValue)>,
    ) -> SchedulerTraceScope {
        SchedulerTraceScope::new(self.services_.clone(), category, name, arguments)
    }

    // Rust adapter for source TRACE_EVENT macros. Arguments remain lazy.
    fn trace(
        &self,
        kind: SchedulerTraceKind,
        category: &'static str,
        name: &'static str,
        arguments: impl FnOnce() -> Vec<(&'static str, crate::adapter::TraceValue)>,
    ) {
        if self.services_.tracing_enabled(category) {
            self.services_.Trace(SchedulerTraceEvent {
                category,
                name,
                kind,
                arguments: arguments(),
                track: if name == "Scheduler:pending_submit_frames" {
                    Some(self.GetTracingTrack())
                } else {
                    None
                },
            });
        }
    }

    // Rust assembly of source virtual ownership; does not create host services.
    pub fn SetOverrides(&mut self, overrides: Arc<dyn SchedulerStateMachineOverrides>) {
        self.overrides_ = Some(overrides);
    }
    // Rust virtual-dispatch bridge for cc/scheduler/scheduler_state_machine.h.
    pub fn NextAction(&self) -> Action {
        if let Some(result) = self.overrides_.as_ref().and_then(|o| o.NextAction(self)) {
            return result;
        }
        self.NextActionBase()
    }

    // Rust virtual-dispatch bridge for cc/scheduler/scheduler_state_machine.h.
    pub fn CheckWillCommit(&self) -> bool {
        if let Some(result) = self
            .overrides_
            .as_ref()
            .and_then(|o| o.CheckWillCommit(self))
        {
            return result;
        }
        self.CheckWillCommitBase()
    }

    // Rust virtual-dispatch bridge for cc/scheduler/scheduler_state_machine.h.
    pub fn ShouldSubscribeToBeginFrames(&self) -> bool {
        if let Some(result) = self
            .overrides_
            .as_ref()
            .and_then(|o| o.ShouldSubscribeToBeginFrames(self))
        {
            return result;
        }
        self.ShouldSubscribeToBeginFramesBase()
    }

    // Rust virtual-dispatch bridge for cc/scheduler/scheduler_state_machine.h.
    pub fn OnBeginImplFrame(&mut self, args: &BeginFrameArgs) {
        if let Some(overrides) = self.overrides_.clone() {
            if overrides.OnBeginImplFrame(self, args) {
                return;
            }
        }
        self.OnBeginImplFrameBase(args);
    }

    // Rust virtual-dispatch bridge for cc/scheduler/scheduler_state_machine.h.
    pub fn CurrentBeginImplFrameDeadlineMode(&self) -> BeginImplFrameDeadlineMode {
        if let Some(result) = self
            .overrides_
            .as_ref()
            .and_then(|o| o.CurrentBeginImplFrameDeadlineMode(self))
        {
            return result;
        }
        self.CurrentBeginImplFrameDeadlineModeBase()
    }

    // Rust virtual-dispatch bridge for cc/scheduler/scheduler_state_machine.h.
    pub fn ShouldTriggerBeginImplFrameDeadlineImmediately(&self) -> bool {
        if let Some(result) = self
            .overrides_
            .as_ref()
            .and_then(|o| o.ShouldTriggerBeginImplFrameDeadlineImmediately(self))
        {
            return result;
        }
        self.ShouldTriggerBeginImplFrameDeadlineImmediatelyBase()
    }

    // Rust virtual-dispatch bridge for cc/scheduler/scheduler_state_machine.h.
    pub fn CheckShouldBlockDeadlineIndefinitely(&self) -> bool {
        if let Some(result) = self
            .overrides_
            .as_ref()
            .and_then(|o| o.CheckShouldBlockDeadlineIndefinitely(self))
        {
            return result;
        }
        self.CheckShouldBlockDeadlineIndefinitelyBase()
    }

    // Rust virtual-dispatch bridge for cc/scheduler/scheduler_state_machine.h.
    pub fn CheckShouldDraw(&self) -> bool {
        if let Some(result) = self
            .overrides_
            .as_ref()
            .and_then(|o| o.CheckShouldDraw(self))
        {
            return result;
        }
        self.CheckShouldDrawBase()
    }

    // Rust virtual-dispatch bridge for cc/scheduler/scheduler_state_machine.h.
    pub fn ShouldActivateSyncTreeBeforeDraw(&self) -> bool {
        if let Some(result) = self
            .overrides_
            .as_ref()
            .and_then(|o| o.ShouldActivateSyncTreeBeforeDraw(self))
        {
            return result;
        }
        self.ShouldActivateSyncTreeBeforeDrawBase()
    }

    // Rust virtual-dispatch bridge for cc/scheduler/scheduler_state_machine.h.
    pub fn ShouldBlockBeginMainFrameWhenIdle(&self) -> bool {
        if let Some(result) = self
            .overrides_
            .as_ref()
            .and_then(|o| o.ShouldBlockBeginMainFrameWhenIdle(self))
        {
            return result;
        }
        self.ShouldBlockBeginMainFrameWhenIdleBase()
    }

    // Rust virtual-dispatch bridge for cc/scheduler/scheduler_state_machine.h.
    pub fn ShouldPrepareTiles(&self) -> bool {
        if let Some(result) = self
            .overrides_
            .as_ref()
            .and_then(|o| o.ShouldPrepareTiles(self))
        {
            return result;
        }
        self.ShouldPrepareTilesBase()
    }

    // Rust virtual-dispatch bridge for cc/scheduler/scheduler_state_machine.h.
    pub fn ShouldInvalidateLayerTreeFrameSink(&self) -> bool {
        if let Some(result) = self
            .overrides_
            .as_ref()
            .and_then(|o| o.ShouldInvalidateLayerTreeFrameSink(self))
        {
            return result;
        }
        self.ShouldInvalidateLayerTreeFrameSinkBase()
    }
}
