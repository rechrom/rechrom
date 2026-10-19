// Translation status: complete
// Copyright 2011 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use super::{
    begin_frame_tracker::BeginFrameTracker,
    commit_earlyout_reason::{CommitEarlyOutReason, CommitEarlyOutReasonToString},
    draw_result::DrawResult,
    scheduler_settings::SchedulerSettings,
    scheduler_state_machine::{
        Action, AnimationWorkletState, BeginImplFrameDeadlineMode, BeginImplFrameState,
        BeginMainFrameState, PaintWorkletState, SchedulerStateMachine, TreeType,
    },
};
use crate::adapter::{
    begin_frame::{BeginFrameArgs, BeginFrameArgsType},
    scheduler::{
        SchedulerStateTrace, SchedulerTraceEvent, SchedulerTraceKind, SchedulerTraceScope,
        TreePriority,
    },
    scheduler_host::*,
    time::{TimeDelta, TimeTicks},
    trace_sink::{BeginImplFrameArgsTrace, SourceLocation, TraceEvent, TraceSink},
    TraceValue,
};
use std::sync::Arc;

// Source: cc/scheduler/scheduler.h:48-53
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameSkippedReason {
    kRecoverLatency,
    kNoDamage,
    kWaitingOnMain,
    kDrawThrottled,
}

// Source: cc/scheduler/scheduler.h:55-94
// Native non-const client callbacks borrow the embedding's actual sibling
// fields for this synchronous call only. The Scheduler argument permits source
// callback reentry with a reborrow of the same client. No client/backpointer is
// retained in Scheduler, a posted task, or a callback closure.
pub trait SchedulerClient {
    fn WillBeginImplFrame(&mut self, args: &BeginFrameArgs, scheduler: &mut Scheduler) -> bool;
    fn ScheduledActionSendBeginMainFrame(
        &mut self,
        args: &BeginFrameArgs,
        scheduler: &mut Scheduler,
    );
    fn ScheduledActionDrawIfPossible(&mut self, scheduler: &mut Scheduler) -> DrawResult;
    fn ScheduledActionDrawForced(&mut self, scheduler: &mut Scheduler) -> DrawResult;
    fn ScheduledActionCommit(&mut self, scheduler: &mut Scheduler);
    fn ScheduledActionPostCommit(&mut self, scheduler: &mut Scheduler);
    fn ScheduledActionActivateSyncTree(&mut self, scheduler: &mut Scheduler);
    fn ScheduledActionBeginLayerTreeFrameSinkCreation(&mut self, scheduler: &mut Scheduler);
    fn ScheduledActionPrepareTiles(&mut self, scheduler: &mut Scheduler);
    fn ScheduledActionInvalidateLayerTreeFrameSink(
        &mut self,
        needs_redraw: bool,
        scheduler: &mut Scheduler,
    );
    fn ScheduledActionPerformImplSideInvalidation(&mut self, scheduler: &mut Scheduler);
    fn DidFinishImplFrame(
        &mut self,
        last_activated_args: &BeginFrameArgs,
        scheduler: &mut Scheduler,
    );
    fn DidNotProduceFrame(
        &mut self,
        ack: BeginFrameAck,
        reason: FrameSkippedReason,
        scheduler: &mut Scheduler,
    );
    fn WillNotReceiveBeginFrame(&mut self, scheduler: &mut Scheduler);
    fn DidChangeBeginFrameSourcePaused(&mut self, paused: bool, scheduler: &mut Scheduler);
    fn FrameIntervalUpdated(&mut self, interval: TimeDelta, scheduler: &mut Scheduler);
    fn OnBeginImplFrameDeadline(&mut self, scheduler: &mut Scheduler);
}

// Source: cc/scheduler/scheduler.h:298-355
pub struct Scheduler {
    settings_: SchedulerSettings,
    layer_tree_host_id_: i32,
    host_: Arc<dyn SchedulerHost>,
    begin_frame_source_: Option<Arc<dyn BeginFrameSource>>,
    observing_begin_frame_source_: bool,
    skipped_last_frame_missed_exceeded_deadline_: bool,
    compositor_timing_history_: Arc<dyn CompositorTimingHistory>,
    compositor_frame_reporting_controller_: Option<Arc<dyn CompositorFrameReportingController>>,
    deadline_: TimeTicks,
    deadline_scheduled_at_: TimeTicks,
    deadline_mode_: BeginImplFrameDeadlineMode,
    begin_impl_frame_tracker_: BeginFrameTracker,
    last_begin_frame_ack_: BeginFrameAck,
    begin_main_frame_args_: BeginFrameArgs,
    last_dispatched_begin_main_frame_args_: BeginFrameArgs,
    next_commit_origin_frame_args_: BeginFrameArgs,
    last_commit_origin_frame_args_: BeginFrameArgs,
    last_activate_origin_frame_args_: BeginFrameArgs,
    begin_impl_frame_deadline_timer_: SchedulerTaskSlot,
    pending_begin_frame_args_: BeginFrameArgs,
    pending_begin_frame_task_: SchedulerTaskSlot,
    state_machine_: SchedulerStateMachine,
    inside_process_scheduled_actions_: ScopedValue<bool>,
    inside_scheduled_action_: ScopedValue<bool>,
    inside_action_: ScopedValue<Action>,
    stopped_: bool,
    needs_finish_frame_for_synchronous_compositor_: bool,
    last_frame_interval_: TimeDelta,
    // Source: components/viz/common/frame_sinks/begin_frame_source.h:118-120
    last_begin_frame_args_: BeginFrameArgs,
    dropped_begin_frame_args_: i64,
    wants_animate_only_begin_frames_: bool,
}

// Source: cc/scheduler/scheduler.cc:45-47
const kDeadlineFudgeFactor: TimeDelta = TimeDelta::Microseconds(1000);

struct HostFrameTraceSink(Arc<dyn SchedulerHost>);
impl TraceSink for HostFrameTraceSink {
    fn enabled(&self) -> bool {
        self.0
            .tracing_enabled("disabled-by-default-cc.debug.scheduler.frames")
    }
    fn emit(&mut self, event: TraceEvent) {
        self.0.BeginFrameTrace(event);
    }
}

// Source: components/viz/common/frame_sinks/begin_frame_source.cc:403-433,546-565
// Only a registration-stack borrow bridges the native observer's synchronous
// paused/MISSED callbacks. Source storage contains solely observer identity.
struct SchedulerBeginFrameObserver<'s, 'c> {
    scheduler: &'s mut Scheduler,
    client: &'c mut dyn SchedulerClient,
}
impl BeginFrameObserverCallback for SchedulerBeginFrameObserver<'_, '_> {
    fn OnBeginFrameSourcePausedChanged(&mut self, paused: bool) {
        self.scheduler
            .OnBeginFrameSourcePausedChanged(paused, self.client);
    }
    fn OnBeginFrame(&mut self, args: &BeginFrameArgs) {
        self.scheduler.OnBeginFrame(args, self.client);
    }
    fn LastUsedBeginFrameArgs(&self) -> &BeginFrameArgs {
        self.scheduler.LastUsedBeginFrameArgs()
    }
    fn WantsAnimateOnlyBeginFrames(&self) -> bool {
        self.scheduler.WantsAnimateOnlyBeginFrames()
    }
}

impl Scheduler {
    // Source: cc/scheduler/scheduler.cc:51-88
    pub fn new(
        client: &mut dyn SchedulerClient,
        settings: &SchedulerSettings,
        layer_tree_host_id: i32,
        host: Arc<dyn SchedulerHost>,
        compositor_timing_history: Arc<dyn CompositorTimingHistory>,
        compositor_frame_reporting_controller: Arc<dyn CompositorFrameReportingController>,
        factory: &dyn SchedulerStateMachineFactory,
    ) -> Self {
        let kind = if settings.wait_for_all_pipeline_stages_before_draw {
            SchedulerStateMachineKind::Headless
        } else if settings.using_synchronous_renderer_compositor
            && host.WebviewSchedulerStateMachineEnabled()
        {
            SchedulerStateMachineKind::Webview
        } else if host.SlimSchedulerEnabled() && !settings.single_threaded_proxy {
            SchedulerStateMachineKind::Slim
        } else {
            SchedulerStateMachineKind::Classic
        };
        let state_machine = factory.Create(kind, settings, host.clone());
        let mut scheduler = Self {
            settings_: settings.clone(),
            layer_tree_host_id_: layer_tree_host_id,
            host_: host,
            begin_frame_source_: None,
            observing_begin_frame_source_: false,
            skipped_last_frame_missed_exceeded_deadline_: false,
            compositor_timing_history_: compositor_timing_history,
            compositor_frame_reporting_controller_: Some(compositor_frame_reporting_controller),
            deadline_: TimeTicks::default(),
            deadline_scheduled_at_: TimeTicks::default(),
            deadline_mode_: BeginImplFrameDeadlineMode::NONE,
            begin_impl_frame_tracker_: BeginFrameTracker::new(SourceLocation {
                function_name: "Scheduler::Scheduler".into(),
                file_name: "cc/scheduler/scheduler.cc".into(),
                line_number: 65,
            }),
            last_begin_frame_ack_: BeginFrameAck::default(),
            begin_main_frame_args_: BeginFrameArgs::default(),
            last_dispatched_begin_main_frame_args_: BeginFrameArgs::default(),
            next_commit_origin_frame_args_: BeginFrameArgs::default(),
            last_commit_origin_frame_args_: BeginFrameArgs::default(),
            last_activate_origin_frame_args_: BeginFrameArgs::default(),
            begin_impl_frame_deadline_timer_: SchedulerTaskSlot::new(
                SchedulerTaskKind::BeginImplFrameDeadline,
            ),
            pending_begin_frame_args_: BeginFrameArgs::default(),
            pending_begin_frame_task_: SchedulerTaskSlot::new(SchedulerTaskKind::PendingBeginFrame),
            state_machine_: state_machine,
            inside_process_scheduled_actions_: ScopedValue::new(false),
            inside_scheduled_action_: ScopedValue::new(false),
            inside_action_: ScopedValue::new(Action::NONE),
            stopped_: false,
            needs_finish_frame_for_synchronous_compositor_: false,
            last_frame_interval_: TimeDelta::default(),
            last_begin_frame_args_: BeginFrameArgs::default(),
            dropped_begin_frame_args_: 0,
            wants_animate_only_begin_frames_: true,
        };
        {
            let _trace = scheduler.trace_scope("cc", "Scheduler::Scheduler", || {
                vec![("settings", scheduler.settings_.AsValue())]
            });
            debug_assert!(!scheduler.state_machine_.BeginFrameNeeded());
        }
        scheduler.ProcessScheduledActions(client);
        scheduler
    }
    // Source: cc/scheduler/scheduler.h:123-123
    pub fn settings(&self) -> &SchedulerSettings {
        &self.settings_
    }

    // Source: cc/scheduler/scheduler.h:126-126
    pub fn visible(&self) -> bool {
        return self.state_machine_.visible();
    }

    // Source: cc/scheduler/scheduler.h:187-189
    pub fn pending_tree_is_ready_for_activation(&self) -> bool {
        return self.state_machine_.pending_tree_is_ready_for_activation();
    }

    // Source: cc/scheduler/scheduler.h:223-226
    pub fn MainFrameForTestingWillHappen(&self) -> bool {
        return self.state_machine_.CommitPending()
            || self.state_machine_.CouldSendBeginMainFrame();
    }

    // Source: cc/scheduler/scheduler.h:228-228
    pub fn CommitPending(&self) -> bool {
        return self.state_machine_.CommitPending();
    }

    // Source: cc/scheduler/scheduler.h:229-229
    pub fn RedrawPending(&self) -> bool {
        return self.state_machine_.RedrawPending();
    }

    // Source: cc/scheduler/scheduler.h:230-232
    pub fn PrepareTilesPending(&self) -> bool {
        return self.state_machine_.PrepareTilesPending();
    }

    // Source: cc/scheduler/scheduler.h:233-235
    pub fn ImplLatencyTakesPriority(&self) -> bool {
        return self.state_machine_.ImplLatencyTakesPriority();
    }

    // Source: cc/scheduler/scheduler.h:268-270
    pub fn begin_frame_source(&self) -> Option<&Arc<dyn BeginFrameSource>> {
        self.begin_frame_source_.as_ref()
    }

    // Source: cc/scheduler/scheduler.h:274-276
    pub fn last_dispatched_begin_main_frame_args(&self) -> &BeginFrameArgs {
        return &self.last_dispatched_begin_main_frame_args_;
    }

    // Source: cc/scheduler/scheduler.h:277-279
    pub fn last_commit_origin_frame_args(&self) -> &BeginFrameArgs {
        return &self.last_commit_origin_frame_args_;
    }

    // Source: cc/scheduler/scheduler.h:280-282
    pub fn last_activate_origin_frame_args(&self) -> &BeginFrameArgs {
        return &self.last_activate_origin_frame_args_;
    }

    // Source: cc/scheduler/scheduler.h:290-292
    pub fn consecutive_no_damage_main_frames(&self) -> i32 {
        return self.state_machine_.consecutive_no_damage_main_frames();
    }

    // Source: cc/scheduler/scheduler.h:407-409
    fn IsInsideAction(&self, action: Action) -> bool {
        self.inside_action_.get() == action
    }

    // Source: cc/scheduler/scheduler.cc:94-100
    pub fn TearDown(&mut self) {
        debug_assert!(self.stopped_);
        self.compositor_frame_reporting_controller_ = None;
    }

    // Source: cc/scheduler/scheduler.cc:102-104
    pub fn Stop(&mut self) {
        self.stopped_ = true;
    }

    // Source: cc/scheduler/scheduler.cc:106-117
    pub fn SetNeedsImplSideInvalidation(
        &mut self,
        needs_first_draw_on_activation: bool,
        client: &mut dyn SchedulerClient,
    ) {
        {
            let _trace = self.trace_scope(
                "disabled-by-default-cc.debug.scheduler",
                "Scheduler::SetNeedsImplSideInvalidation",
                || {
                    vec![(
                        "needs_first_draw_on_activation",
                        TraceValue::Bool(needs_first_draw_on_activation),
                    )]
                },
            );
            self.state_machine_
                .SetNeedsImplSideInvalidation(needs_first_draw_on_activation);
        }
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:119-124
    pub fn Now(&self) -> TimeTicks {
        let now = self.host_.Now();
        let _trace = self.trace_scope(
            "disabled-by-default-cc.debug.scheduler.now",
            "Scheduler::Now",
            || {
                vec![(
                    "now",
                    TraceValue::Integer(now.since_origin().InMicroseconds()),
                )]
            },
        );
        now
    }

    // Source: cc/scheduler/scheduler.cc:126-130
    pub fn SetVisible(&mut self, visible: bool, client: &mut dyn SchedulerClient) {
        self.state_machine_.SetVisible(visible);
        self.UpdateCompositorTimingHistoryRecordingEnabled();
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:132-135
    pub fn SetShouldWarmUp(&mut self, client: &mut dyn SchedulerClient) {
        self.state_machine_.SetShouldWarmUp();
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:137-140
    pub fn SetCanDraw(&mut self, can_draw: bool, client: &mut dyn SchedulerClient) {
        self.state_machine_.SetCanDraw(can_draw);
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:142-148
    pub fn NotifyReadyToActivate(&mut self, client: &mut dyn SchedulerClient) {
        if (self.state_machine_.NotifyReadyToActivate()) {
            self.compositor_timing_history_.ReadyToActivate();
        }

        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:150-152
    pub fn IsReadyToActivate(&mut self) -> bool {
        return self.state_machine_.IsReadyToActivate();
    }

    // Source: cc/scheduler/scheduler.cc:154-158
    pub fn NotifyReadyToDraw(&mut self, client: &mut dyn SchedulerClient) {
        // Future work might still needed for crbug.com/352894.
        self.state_machine_.NotifyReadyToDraw();
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:160-170
    pub fn SetBeginFrameSource(
        &mut self,
        source: Option<Arc<dyn BeginFrameSource>>,
        client: &mut dyn SchedulerClient,
    ) {
        if let Some(source) = self.SetBeginFrameSourceState(source) {
            self.AddBeginFrameObserver(source.as_ref(), client);
        }
    }

    // Source: cc/scheduler/scheduler.cc:169,385
    fn AddBeginFrameObserver(
        &mut self,
        source: &dyn BeginFrameSource,
        client: &mut dyn SchedulerClient,
    ) {
        let observer_id = self.host_.observer_id();
        let mut callback = SchedulerBeginFrameObserver {
            scheduler: self,
            client,
        };
        source.AddObserver(observer_id, &mut callback);
    }

    // Source: cc/scheduler/scheduler.cc:160-170
    // The destructor's nullptr source takes this same state-update path and
    // cannot add an observer or call a client. The returned source is only the
    // native synchronous AddObserver continuation, never stored or escaped.
    fn SetBeginFrameSourceState(
        &mut self,
        source: Option<Arc<dyn BeginFrameSource>>,
    ) -> Option<Arc<dyn BeginFrameSource>> {
        let same = match (&source, &self.begin_frame_source_) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        if same {
            return None;
        }
        if self.observing_begin_frame_source_ {
            if let Some(old) = self.begin_frame_source_.clone() {
                old.RemoveObserver(self.host_.observer_id());
            }
        }
        self.begin_frame_source_ = source;
        if self.observing_begin_frame_source_ {
            self.begin_frame_source_.clone()
        } else {
            None
        }
    }

    // Source: cc/scheduler/scheduler.cc:172-176
    pub fn NotifyAnimationWorkletStateChange(
        &mut self,
        state: AnimationWorkletState,
        tree: TreeType,
        client: &mut dyn SchedulerClient,
    ) {
        self.state_machine_
            .NotifyAnimationWorkletStateChange(state, tree);
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:178-181
    pub fn NotifyPaintWorkletStateChange(
        &mut self,
        state: PaintWorkletState,
        client: &mut dyn SchedulerClient,
    ) {
        self.state_machine_.NotifyPaintWorkletStateChange(state);
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:183-186
    pub fn SetNeedsBeginMainFrame(
        &mut self,
        now: bool,
        unthrottled: bool,
        client: &mut dyn SchedulerClient,
    ) {
        self.state_machine_.SetNeedsBeginMainFrame(now, unthrottled);
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:188-191
    pub fn SetNeedsOneBeginImplFrame(&mut self, client: &mut dyn SchedulerClient) {
        self.state_machine_.SetNeedsOneBeginImplFrame();
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:193-250
    pub fn SendEarlyFinalBeginMainFrame(&mut self, client: &mut dyn SchedulerClient) {
        let _trace = self.trace_scope("cc", "Scheduler::SendEarlyFinalBeginMainFrame", || vec![]);
        let mut args = self.last_dispatched_begin_main_frame_args_.clone();
        if self
            .begin_impl_frame_tracker_
            .DangerousMethodCurrentOrLast()
            .frame_id
            > args.frame_id
        {
            args = self
                .begin_impl_frame_tracker_
                .DangerousMethodCurrentOrLast()
                .clone();
        }
        if self.pending_begin_frame_args_.frame_id > args.frame_id {
            args = self.pending_begin_frame_args_.clone();
        }
        if !args.IsValid() || args.interval.is_zero() {
            return;
        }
        if self.pending_begin_frame_args_.IsValid()
            && args.frame_id >= self.pending_begin_frame_args_.frame_id
        {
            self.pending_begin_frame_args_ = BeginFrameArgs::default();
            self.pending_begin_frame_task_.Cancel(self.host_.as_ref());
        }
        if self.state_machine_.begin_impl_frame_state() != BeginImplFrameState::IDLE {
            self.begin_impl_frame_deadline_timer_
                .Cancel(self.host_.as_ref());
            if !self.settings_.using_synchronous_renderer_compositor {
                self.reporting()
                    .OnFinishImplFrame(self.begin_impl_frame_tracker_.Current().frame_id, false);
            }
            self.FinishImplFrame(client);
        }
        let elapsed = self.Now() - args.frame_time;
        let mut intervals = if elapsed.is_positive() {
            (elapsed / args.interval).round().clamp(0., i32::MAX as f64) as u64
        } else {
            1
        };
        if intervals == 0 {
            intervals = 1;
        }
        args.frame_id.sequence_number += intervals;
        args.frame_time += args.interval * intervals;
        args.deadline = args.frame_time + args.interval;
        args.on_critical_path = !self.state_machine_.ImplLatencyTakesPriority();
        args.frame_id.source_id = BeginFrameArgs::kManualSourceId;
        self.state_machine_.SetUrgentBeginMainFramePending();
        self.OnBeginFrame(&args, client);
    }

    // Source: cc/scheduler/scheduler.cc:252-255
    pub fn SetNeedsRedraw(&mut self, client: &mut dyn SchedulerClient) {
        self.state_machine_.SetNeedsRedraw();
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:257-261
    pub fn SetNeedsPrepareTiles(&mut self, client: &mut dyn SchedulerClient) {
        debug_assert!(!self.IsInsideAction(Action::PREPARE_TILES));
        self.state_machine_.SetNeedsPrepareTiles();
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:263-281
    pub fn DidSubmitCompositorFrame(&mut self, submit_info: &mut SubmitInfo) {
        if !self.settings_.using_synchronous_renderer_compositor
            || !self.state_machine_.resourceless_draw()
        {
            self.reporting().DidSubmitCompositorFrame(
                submit_info,
                self.begin_main_frame_args_.frame_id,
                self.last_activate_origin_frame_args_.frame_id,
            );
        }
        self.state_machine_.DidSubmitCompositorFrame();
        if !self.inside_process_scheduled_actions_.get() {
            debug_assert_eq!(self.state_machine_.NextAction(), Action::NONE);
        }
    }

    // Source: cc/scheduler/scheduler.cc:283-291
    pub fn DidReceiveCompositorFrameAck(&mut self, client: &mut dyn SchedulerClient) {
        if self.host_.features().kNoCompositorFrameAcks {
            unreachable!();
        } else {
            debug_assert!(self.state_machine_.pending_submit_frames() > 0);
        }
        self.state_machine_.DidReceiveCompositorFrameAck();
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:293-299
    pub fn SetTreePrioritiesAndScrollState(
        &mut self,
        tree_priority: TreePriority,
        is_current_scroll_main_painted: bool,
        client: &mut dyn SchedulerClient,
    ) {
        self.state_machine_
            .SetTreePrioritiesAndScrollState(tree_priority, is_current_scroll_main_painted);
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:301-312
    pub fn NotifyReadyToCommit(
        &mut self,
        details: Option<BeginMainFrameMetrics>,
        client: &mut dyn SchedulerClient,
    ) {
        {
            let _trace = self.trace_scope("cc", "Scheduler::NotifyReadyToCommit", || vec![]);
            self.compositor_timing_history_.NotifyReadyToCommit();
            self.reporting().NotifyReadyToCommit(details);
            self.state_machine_.NotifyReadyToCommit();
            self.next_commit_origin_frame_args_ =
                self.last_dispatched_begin_main_frame_args_.clone();
        }
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:314-326
    pub fn BeginMainFrameAborted(
        &mut self,
        reason: CommitEarlyOutReason,
        client: &mut dyn SchedulerClient,
    ) {
        {
            let _trace = self.trace_scope("cc", "Scheduler::BeginMainFrameAborted", || {
                vec![(
                    "reason",
                    TraceValue::String(CommitEarlyOutReasonToString(reason).into()),
                )]
            });
            self.compositor_timing_history_.BeginMainFrameAborted();
            let frame_id = self.last_dispatched_begin_main_frame_args_.frame_id;
            self.reporting().BeginMainFrameAborted(frame_id, reason);
            self.state_machine_.BeginMainFrameAborted(reason);
        }
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:328-330
    pub fn DidPrepareTiles(&mut self) {
        self.state_machine_.DidPrepareTiles();
    }

    // Source: cc/scheduler/scheduler.cc:332-337
    pub fn DidPresentCompositorFrame(&mut self, frame_token: u32, details: &FrameTimingDetails) {
        self.reporting()
            .DidPresentCompositorFrame(frame_token, details);
    }

    // Source: cc/scheduler/scheduler.cc:339-346
    pub fn DidLoseLayerTreeFrameSink(&mut self, client: &mut dyn SchedulerClient) {
        {
            let _trace = self.trace_scope("cc", "Scheduler::DidLoseLayerTreeFrameSink", || vec![]);
            self.state_machine_.DidLoseLayerTreeFrameSink();
            self.UpdateCompositorTimingHistoryRecordingEnabled();
        }
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:348-357
    pub fn DidCreateAndInitializeLayerTreeFrameSink(&mut self, client: &mut dyn SchedulerClient) {
        {
            let _trace = self.trace_scope(
                "cc",
                "Scheduler::DidCreateAndInitializeLayerTreeFrameSink",
                || vec![],
            );
            debug_assert!(!self.observing_begin_frame_source_);
            debug_assert!(!self.begin_impl_frame_deadline_timer_.IsRunning());
            self.state_machine_
                .DidCreateAndInitializeLayerTreeFrameSink();
            self.UpdateCompositorTimingHistoryRecordingEnabled();
        }
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:359-365
    pub fn NotifyBeginMainFrameStarted(&mut self, main_thread_start_time: TimeTicks) {
        let _trace = self.trace_scope("cc", "Scheduler::NotifyBeginMainFrameStarted", || vec![]);
        self.compositor_timing_history_
            .BeginMainFrameStarted(main_thread_start_time);
        self.reporting()
            .BeginMainFrameStarted(main_thread_start_time);
    }

    // Source: cc/scheduler/scheduler.cc:367-369
    pub fn LastBeginImplFrameTime(&mut self) -> TimeTicks {
        return self.begin_impl_frame_tracker_.Current().frame_time;
    }

    // Source: cc/scheduler/scheduler.cc:371-402
    pub fn StartOrStopBeginFrames(&mut self, client: &mut dyn SchedulerClient) {
        if self.state_machine_.begin_impl_frame_state() != BeginImplFrameState::IDLE {
            return;
        }
        let needs_begin_frames = self.state_machine_.ShouldSubscribeToBeginFrames();
        if needs_begin_frames == self.observing_begin_frame_source_ {
            return;
        }
        if needs_begin_frames {
            self.observing_begin_frame_source_ = true;
            if let Some(source) = self.begin_frame_source_.clone() {
                self.AddBeginFrameObserver(source.as_ref(), client);
            }
            self.host_
                .NeedsBeginFrameChanged(self.layer_tree_host_id_, true);
        } else {
            self.observing_begin_frame_source_ = false;
            if let Some(source) = self.begin_frame_source_.clone() {
                source.RemoveObserver(self.host_.observer_id());
            }
            if self.settings_.using_synchronous_renderer_compositor {
                self.FinishImplFrameSynchronous(client);
            }
            self.CancelPendingBeginFrameTask(client);
            self.compositor_timing_history_
                .BeginImplFrameNotExpectedSoon();
            self.reporting().OnStoppedRequestingBeginFrames();
            self.host_
                .NeedsBeginFrameChanged(self.layer_tree_host_id_, false);
            client.WillNotReceiveBeginFrame(self);
        }
    }

    // Source: cc/scheduler/scheduler.cc:404-413
    pub fn CancelPendingBeginFrameTask(&mut self, client: &mut dyn SchedulerClient) {
        if self.pending_begin_frame_args_.IsValid() {
            self.trace_instant("cc", "Scheduler::BeginFrameDropped", || vec![]);
            let args = self.pending_begin_frame_args_.clone();
            self.SendDidNotProduceFrame(&args, FrameSkippedReason::kNoDamage, client);
            self.pending_begin_frame_args_ = BeginFrameArgs::default();
        }
        self.pending_begin_frame_task_.Cancel(self.host_.as_ref());
    }

    // Source: cc/scheduler/scheduler.cc:415-431
    pub fn PostPendingBeginFrameTask(&mut self) {
        let is_idle = self.state_machine_.begin_impl_frame_state() == BeginImplFrameState::IDLE;
        let needs_begin_frames = self.state_machine_.BeginFrameNeeded();
        let has_pending_begin_frame_args = self.pending_begin_frame_args_.IsValid();
        let has_no_pending_begin_frame_task = self.pending_begin_frame_task_.IsCancelled();
        if is_idle
            && needs_begin_frames
            && has_pending_begin_frame_args
            && has_no_pending_begin_frame_task
        {
            self.pending_begin_frame_task_
                .Reset(self.host_.as_ref(), None, false);
        }
    }

    // Source: cc/scheduler/scheduler.cc:433-444
    pub fn OnBeginFrameSourcePausedChanged(
        &mut self,
        paused: bool,
        client: &mut dyn SchedulerClient,
    ) {
        if self.state_machine_.begin_frame_source_paused() == paused {
            return;
        }
        {
            self.trace_instant("cc", "Scheduler::SetBeginFrameSourcePaused", || {
                vec![("paused", TraceValue::Bool(paused))]
            });
            self.state_machine_.SetBeginFrameSourcePaused(paused);
            client.DidChangeBeginFrameSourcePaused(paused, self);
        }
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:446-531
    pub fn OnBeginFrameDerivedImpl(
        &mut self,
        args: &BeginFrameArgs,
        client: &mut dyn SchedulerClient,
    ) -> bool {
        if args.interval != self.last_frame_interval_ && args.interval.is_positive() {
            self.last_frame_interval_ = args.interval;
            client.FrameIntervalUpdated(self.last_frame_interval_, self);
            self.state_machine_
                .FrameIntervalUpdated(self.last_frame_interval_);
        }
        if self.begin_impl_frame_tracker_.DangerousMethodHasStarted() {
            let last_args = self
                .begin_impl_frame_tracker_
                .DangerousMethodCurrentOrLast();
            if args.frame_id.source_id == last_args.frame_id.source_id
                && args.frame_id.sequence_number <= last_args.frame_id.sequence_number
            {
                return false;
            }
        }
        let _trace = self.trace_scope("cc,benchmark", "Scheduler::BeginFrame", || {
            vec![("args", self.frame_trace(args))]
        });
        if !self.state_machine_.BeginFrameNeeded() {
            self.trace_instant("cc", "Scheduler::BeginFrameDropped", || vec![]);
            self.SendDidNotProduceFrame(args, FrameSkippedReason::kNoDamage, client);
            return false;
        }
        let _frame_trace = self.trace_scope(
            "disabled-by-default-cc.debug.scheduler.frames",
            "viz::BeginFrameArgs",
            || {
                vec![(
                    "flow_id",
                    TraceValue::Integer(args.frame_time.since_origin().InMicroseconds()),
                )]
            },
        );
        if self.settings_.using_synchronous_renderer_compositor {
            self.BeginImplFrameSynchronous(args, client);
            return true;
        }
        let inside_previous_begin_frame =
            self.state_machine_.begin_impl_frame_state() == BeginImplFrameState::INSIDE_BEGIN_FRAME;
        if self.inside_process_scheduled_actions_.get()
            || inside_previous_begin_frame
            || self.pending_begin_frame_args_.IsValid()
        {
            if self.pending_begin_frame_args_.IsValid() {
                self.trace_instant("cc", "Scheduler::BeginFrameDropped", || vec![]);
                let pending = self.pending_begin_frame_args_.clone();
                self.SendDidNotProduceFrame(&pending, FrameSkippedReason::kRecoverLatency, client);
            }
            self.pending_begin_frame_args_ = args.clone();
            self.ProcessScheduledActions(client);
        } else {
            self.BeginImplFrameWithDeadline(args, client);
        }
        true
    }

    // Source: cc/scheduler/scheduler.cc:533-536
    pub fn SetVideoNeedsBeginFrames(
        &mut self,
        video_needs_begin_frames: bool,
        client: &mut dyn SchedulerClient,
    ) {
        self.state_machine_
            .SetVideoNeedsBeginFrames(video_needs_begin_frames);
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:538-540
    pub fn SetIsScrolling(&mut self, is_scrolling: bool) {
        self.state_machine_.set_is_scrolling(is_scrolling);
    }

    // Source: cc/scheduler/scheduler.cc:542-544
    pub fn SetWaitingForScrollEvent(&mut self, waiting_for_scroll_event: bool) {
        self.state_machine_
            .set_waiting_for_scroll_event(waiting_for_scroll_event);
    }

    // Source: cc/scheduler/scheduler.cc:546-566
    pub fn OnDrawForLayerTreeFrameSink(
        &mut self,
        resourceless_software_draw: bool,
        skip_draw: bool,
        client: &mut dyn SchedulerClient,
    ) {
        debug_assert!(self.settings_.using_synchronous_renderer_compositor);
        if self.state_machine_.begin_impl_frame_state() == BeginImplFrameState::INSIDE_BEGIN_FRAME {
            debug_assert!(self.needs_finish_frame_for_synchronous_compositor_);
        } else {
            debug_assert_eq!(
                self.state_machine_.begin_impl_frame_state(),
                BeginImplFrameState::IDLE
            );
            debug_assert!(!self.needs_finish_frame_for_synchronous_compositor_);
        }
        debug_assert!(!self.begin_impl_frame_deadline_timer_.IsRunning());
        self.state_machine_
            .SetResourcelessSoftwareDraw(resourceless_software_draw);
        self.state_machine_.SetSkipDraw(skip_draw);
        self.OnBeginImplFrameDeadline(client);
        self.state_machine_.OnBeginImplFrameIdle();
        self.ProcessScheduledActions(client);
        self.state_machine_.SetResourcelessSoftwareDraw(false);
    }

    // Source: cc/scheduler/scheduler.cc:568-577
    pub fn HandlePendingBeginFrame(&mut self, client: &mut dyn SchedulerClient) {
        debug_assert!(self.pending_begin_frame_args_.IsValid());
        let args = self.pending_begin_frame_args_.clone();
        self.pending_begin_frame_args_ = BeginFrameArgs::default();
        self.pending_begin_frame_task_.Cancel(self.host_.as_ref());
        self.BeginImplFrameWithDeadline(&args, client);
    }

    // Source: cc/scheduler/scheduler.cc:579-672
    pub fn BeginImplFrameWithDeadline(
        &mut self,
        args: &BeginFrameArgs,
        client: &mut dyn SchedulerClient,
    ) {
        debug_assert!(self.pending_begin_frame_task_.IsCancelled());
        debug_assert!(!self.pending_begin_frame_args_.IsValid());
        debug_assert_eq!(
            self.state_machine_.begin_impl_frame_state(),
            BeginImplFrameState::IDLE
        );
        let main_thread_is_in_high_latency_mode =
            self.state_machine_.main_thread_missed_last_deadline();
        let _trace = self.trace_scope("cc,benchmark", "Scheduler::BeginImplFrame", || {
            vec![
                ("args", self.frame_trace(args)),
                (
                    "main_thread_missed_last_deadline",
                    TraceValue::Bool(main_thread_is_in_high_latency_mode),
                ),
            ]
        });
        if self
            .host_
            .tracing_enabled("disabled-by-default-cc.debug.scheduler")
        {
            self.host_.Trace(SchedulerTraceEvent {
                category: "disabled-by-default-cc.debug.scheduler",
                name: "MainThreadLatency",
                kind: SchedulerTraceKind::Counter,
                arguments: vec![(
                    "value",
                    TraceValue::Bool(main_thread_is_in_high_latency_mode),
                )],
                track: None,
            });
        }
        let now = self.Now();
        if !self.settings_.wait_for_all_pipeline_stages_before_draw
            && args.type_ == BeginFrameArgsType::MISSED
            && args.deadline < now
        {
            self.trace_instant("cc", "Scheduler::MissedBeginFrameDropped", || vec![]);
            self.skipped_last_frame_missed_exceeded_deadline_ = true;
            self.SendDidNotProduceFrame(args, FrameSkippedReason::kRecoverLatency, client);
            return;
        }
        self.skipped_last_frame_missed_exceeded_deadline_ = false;
        let mut adjusted_args = args.clone();
        adjusted_args.deadline -= self.compositor_timing_history_.DrawDurationEstimate();
        adjusted_args.deadline -= kDeadlineFudgeFactor;
        let mut bmf_to_activate_threshold = adjusted_args.interval
            - self.compositor_timing_history_.DrawDurationEstimate()
            - kDeadlineFudgeFactor;
        let bmf_to_activate_estimate_critical = self
            .compositor_timing_history_
            .BeginMainFrameQueueToActivateCriticalEstimate();
        self.state_machine_
            .SetCriticalBeginMainFrameToActivateIsFast(
                bmf_to_activate_estimate_critical < bmf_to_activate_threshold,
            );
        self.begin_main_frame_args_ = adjusted_args.clone();
        self.begin_main_frame_args_.on_critical_path = !self.ImplLatencyTakesPriority();
        let mut time_since_main_frame_sent = TimeDelta::default();
        if self.compositor_timing_history_.begin_main_frame_sent_time() != TimeTicks::default() {
            time_since_main_frame_sent =
                now - self.compositor_timing_history_.begin_main_frame_sent_time();
        }
        let bmf_sent_to_ready_to_commit_estimate = if self.begin_main_frame_args_.on_critical_path {
            self.compositor_timing_history_
                .BeginMainFrameStartToReadyToCommitCriticalEstimate()
        } else {
            self.compositor_timing_history_
                .BeginMainFrameStartToReadyToCommitNotCriticalEstimate()
        };
        bmf_to_activate_threshold += args.interval * self.settings_.delay_impl_invalidation_frames;
        let main_thread_response_expected_soon =
            if time_since_main_frame_sent > bmf_to_activate_threshold {
                false
            } else {
                bmf_sent_to_ready_to_commit_estimate - time_since_main_frame_sent
                    < bmf_to_activate_threshold
            };
        self.state_machine_
            .set_should_defer_invalidation_for_fast_main_frame(main_thread_response_expected_soon);
        self.BeginImplFrame(&adjusted_args, now, client);
    }

    // Source: cc/scheduler/scheduler.cc:674-707
    pub fn BeginImplFrameSynchronous(
        &mut self,
        args: &BeginFrameArgs,
        client: &mut dyn SchedulerClient,
    ) {
        self.FinishImplFrameSynchronous(client);
        let _trace = self.trace_scope("cc,benchmark", "Scheduler::BeginImplFrame", || {
            vec![("args", self.frame_trace(args))]
        });
        self.state_machine_
            .SetCriticalBeginMainFrameToActivateIsFast(false);
        self.begin_main_frame_args_ = args.clone();
        self.begin_main_frame_args_.on_critical_path = !self.ImplLatencyTakesPriority();
        let mut adjusted_args = args.clone();
        adjusted_args.deadline -= self.compositor_timing_history_.DrawDurationEstimate();
        adjusted_args.deadline -= kDeadlineFudgeFactor;
        self.BeginImplFrame(&adjusted_args, self.Now(), client);
        self.compositor_timing_history_
            .WillFinishImplFrame(self.state_machine_.needs_redraw());
        let waiting_for_main = !(self.deadline_mode_ == BeginImplFrameDeadlineMode::IMMEDIATE
            || self.deadline_mode_ == BeginImplFrameDeadlineMode::WAIT_FOR_SCROLL);
        self.reporting()
            .OnFinishImplFrame(adjusted_args.frame_id, waiting_for_main);
        self.needs_finish_frame_for_synchronous_compositor_ = true;
        if !self.state_machine_.did_invalidate_layer_tree_frame_sink() {
            self.FinishImplFrameSynchronous(client);
        }
    }

    // Source: cc/scheduler/scheduler.cc:709-760
    pub fn FinishImplFrame(&mut self, client: &mut dyn SchedulerClient) {
        let _trace = self.trace_scope("cc", "Scheduler::FinishImplFrame", || vec![]);
        debug_assert!(!self.needs_finish_frame_for_synchronous_compositor_);
        self.state_machine_.OnBeginImplFrameIdle();
        if !self.state_machine_.did_submit_in_last_frame() {
            let has_pending_tree = self.state_machine_.has_pending_tree();
            let is_waiting_on_main =
                self.state_machine_.begin_main_frame_state() != BeginMainFrameState::IDLE;
            let is_draw_throttled =
                self.state_machine_.needs_redraw() && self.state_machine_.IsDrawThrottled();
            let _trace = self.trace_scope("cc", "DidNotSubmitInLastFrame", || {
                vec![
                    ("has_pending_tree", TraceValue::Bool(has_pending_tree)),
                    ("is_waiting_on_main", TraceValue::Bool(is_waiting_on_main)),
                ]
            });
            let mut reason = FrameSkippedReason::kNoDamage;
            if is_waiting_on_main || has_pending_tree {
                reason = FrameSkippedReason::kWaitingOnMain;
            } else if is_draw_throttled {
                reason = FrameSkippedReason::kDrawThrottled;
            }
            let args = self.begin_impl_frame_tracker_.Current().clone();
            self.SendDidNotProduceFrame(&args, reason, client);
            if self.begin_impl_frame_tracker_.Current().frame_id
                != self.last_activate_origin_frame_args_.frame_id
                && self.state_machine_.draw_succeeded_in_last_frame()
            {
                self.reporting().DidNotProduceFrame(
                    self.last_activate_origin_frame_args_.frame_id,
                    FrameSkippedReason::kNoDamage,
                );
            }
        }
        self.begin_impl_frame_tracker_
            .Finish(self.Now(), &mut HostFrameTraceSink(self.host_.clone()));
        self.ProcessScheduledActions(client);
        debug_assert!(!self.inside_scheduled_action_.get());
        {
            let _mark = self.inside_scheduled_action_.reset(true);
            let args = self.last_activate_origin_frame_args_.clone();
            client.DidFinishImplFrame(&args, self);
        }
        if let Some(source) = self.begin_frame_source_.clone() {
            source.DidFinishFrame(self.host_.observer_id());
        }
    }

    // Source: cc/scheduler/scheduler.cc:762-771
    pub fn SendDidNotProduceFrame(
        &mut self,
        args: &BeginFrameArgs,
        reason: FrameSkippedReason,
        client: &mut dyn SchedulerClient,
    ) {
        let _trace = self.trace_scope("cc", "Scheduler::SendDidNotProduceFrame", || {
            vec![("reason", TraceValue::Integer(reason as i64))]
        });
        if self.last_begin_frame_ack_.frame_id == args.frame_id {
            return;
        }
        self.last_begin_frame_ack_ = BeginFrameAck::new(args, false);
        client.DidNotProduceFrame(self.last_begin_frame_ack_, reason, self);
        self.reporting().DidNotProduceFrame(args.frame_id, reason);
    }

    // Source: cc/scheduler/scheduler.cc:773-799
    pub fn BeginImplFrame(
        &mut self,
        args: &BeginFrameArgs,
        _now: TimeTicks,
        client: &mut dyn SchedulerClient,
    ) {
        debug_assert_eq!(
            self.state_machine_.begin_impl_frame_state(),
            BeginImplFrameState::IDLE
        );
        debug_assert!(!self.begin_impl_frame_deadline_timer_.IsRunning());
        debug_assert!(self.state_machine_.HasInitializedLayerTreeFrameSink());
        {
            debug_assert!(!self.inside_scheduled_action_.get());
            let _mark = self.inside_scheduled_action_.reset(true);
            self.begin_impl_frame_tracker_.Start(
                args,
                self.Now(),
                &mut HostFrameTraceSink(self.host_.clone()),
            );
            self.state_machine_.OnBeginImplFrame(args);
            self.reporting()
                .WillBeginImplFrame(args, self.state_machine_.ShouldThrottleSendBeginMainFrame());
            let current = self.begin_impl_frame_tracker_.Current().clone();
            let has_damage = client.WillBeginImplFrame(&current, self);
            if !has_damage {
                self.state_machine_.AbortDraw();
            }
        }
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:801-887
    pub fn ScheduleBeginImplFrameDeadline(&mut self, client: &mut dyn SchedulerClient) {
        self.deadline_mode_ = self.state_machine_.CurrentBeginImplFrameDeadlineMode();
        let new_deadline = match self.deadline_mode_ {
            BeginImplFrameDeadlineMode::NONE => {
                debug_assert!(!self.begin_impl_frame_deadline_timer_.IsRunning());
                return;
            }
            BeginImplFrameDeadlineMode::BLOCKED => {
                let has_pending_begin_frame = self.pending_begin_frame_args_.IsValid();
                if has_pending_begin_frame {
                    TimeTicks::default()
                } else {
                    self.begin_impl_frame_deadline_timer_
                        .Cancel(self.host_.as_ref());
                    return;
                }
            }
            BeginImplFrameDeadlineMode::LATE => {
                let new_deadline = self.begin_impl_frame_tracker_.Current().frame_time
                    + self.begin_impl_frame_tracker_.Current().interval;
                if !self.state_machine_.NewActiveTreeLikely() {
                    let args = self.begin_impl_frame_tracker_.Current().clone();
                    self.SendDidNotProduceFrame(&args, FrameSkippedReason::kNoDamage, client);
                }
                new_deadline
            }
            BeginImplFrameDeadlineMode::REGULAR => {
                self.begin_impl_frame_tracker_.Current().deadline
            }
            BeginImplFrameDeadlineMode::IMMEDIATE => TimeTicks::default(),
            BeginImplFrameDeadlineMode::WAIT_FOR_SCROLL => {
                self.begin_impl_frame_tracker_.Current().frame_time
                    + self.begin_impl_frame_tracker_.Current().interval
                        * self.settings_.scroll_deadline_ratio
            }
        };
        let has_no_deadline_task = !self.begin_impl_frame_deadline_timer_.IsRunning();
        if has_no_deadline_task || new_deadline != self.deadline_ {
            let _trace =
                self.trace_scope("cc", "Scheduler::ScheduleBeginImplFrameDeadline", || {
                    vec![
                        (
                            "new deadline",
                            TraceValue::Integer(new_deadline.since_origin().InMicroseconds()),
                        ),
                        (
                            "deadline mode",
                            TraceValue::String(
                                SchedulerStateMachine::BeginImplFrameDeadlineModeToString(
                                    self.deadline_mode_,
                                )
                                .into(),
                            ),
                        ),
                    ]
                });
            self.deadline_ = new_deadline;
            if self.host_.ShouldRecordSubsampledMetric(0.001) {
                self.host_.HistogramDeadlineMode(
                    "Compositing.Scheduler.DeadlineMode",
                    self.deadline_mode_,
                );
            }
            if self
                .host_
                .TraceCategoryPointerPresent("disabled-by-default-cc.debug.scheduler")
            {
                self.deadline_scheduled_at_ = self.Now();
            }
            self.begin_impl_frame_deadline_timer_
                .Cancel(self.host_.as_ref());
            self.begin_impl_frame_deadline_timer_.Reset(
                self.host_.as_ref(),
                Some(self.deadline_),
                true,
            );
        }
    }

    // Source: cc/scheduler/scheduler.cc:889-921
    pub fn OnBeginImplFrameDeadline(&mut self, client: &mut dyn SchedulerClient) {
        {
            let _trace = self.trace_scope(
                "cc,benchmark",
                "Scheduler::OnBeginImplFrameDeadline",
                || vec![],
            );
            self.begin_impl_frame_deadline_timer_
                .Cancel(self.host_.as_ref());
            if !self.settings_.using_synchronous_renderer_compositor {
                self.compositor_timing_history_
                    .WillFinishImplFrame(self.state_machine_.needs_redraw());
                let waiting_for_main = !(self.deadline_mode_
                    == BeginImplFrameDeadlineMode::IMMEDIATE
                    || self.deadline_mode_ == BeginImplFrameDeadlineMode::WAIT_FOR_SCROLL);
                self.reporting()
                    .OnFinishImplFrame(self.begin_main_frame_args_.frame_id, waiting_for_main);
            }
            self.state_machine_.OnBeginImplFrameDeadline();
            client.OnBeginImplFrameDeadline(self);
        }
        self.ProcessScheduledActions(client);
        if self.settings_.using_synchronous_renderer_compositor {
            self.FinishImplFrameSynchronous(client);
        } else {
            self.FinishImplFrame(client);
        }
    }

    // Source: cc/scheduler/scheduler.cc:923-929
    pub fn FinishImplFrameSynchronous(&mut self, client: &mut dyn SchedulerClient) {
        debug_assert!(self.settings_.using_synchronous_renderer_compositor);
        if self.needs_finish_frame_for_synchronous_compositor_ {
            self.needs_finish_frame_for_synchronous_compositor_ = false;
            self.FinishImplFrame(client);
        }
    }

    // Source: cc/scheduler/scheduler.cc:931-939
    pub fn DrawIfPossible(&mut self, client: &mut dyn SchedulerClient) {
        debug_assert!(!self.inside_scheduled_action_.get());
        let _mark = self.inside_scheduled_action_.reset(true);
        self.compositor_timing_history_.WillDraw();
        self.state_machine_.WillDraw();
        let result = client.ScheduledActionDrawIfPossible(self);
        self.state_machine_.DidDraw(result);
        self.compositor_timing_history_.DidDraw();
    }

    // Source: cc/scheduler/scheduler.cc:941-958
    pub fn DrawForced(&mut self, client: &mut dyn SchedulerClient) {
        debug_assert!(!self.inside_scheduled_action_.get());
        let _mark = self.inside_scheduled_action_.reset(true);
        let drawing_with_new_active_tree = self.state_machine_.active_tree_needs_first_draw()
            && !self.state_machine_.previous_pending_tree_was_impl_side();
        if drawing_with_new_active_tree {
            let _trace = self.trace_scope("viz,benchmark", "Graphics.Pipeline.DrawForced", || {
                vec![
                    (
                        "flow_id",
                        TraceValue::Integer(self.last_activate_origin_frame_args_.trace_id),
                    ),
                    (
                        "trace_id",
                        TraceValue::Integer(self.last_activate_origin_frame_args_.trace_id),
                    ),
                ]
            });
        }
        self.compositor_timing_history_.WillDraw();
        self.state_machine_.WillDraw();
        let result = client.ScheduledActionDrawForced(self);
        self.state_machine_.DidDraw(result);
        self.compositor_timing_history_.DidDraw();
    }

    // Source: cc/scheduler/scheduler.cc:960-967
    pub fn SetDeferBeginMainFrame(
        &mut self,
        defer_begin_main_frame: bool,
        client: &mut dyn SchedulerClient,
    ) {
        {
            let _trace = self.trace_scope("cc", "Scheduler::SetDeferBeginMainFrame", || {
                vec![(
                    "defer_begin_main_frame",
                    TraceValue::Bool(defer_begin_main_frame),
                )]
            });
            self.state_machine_
                .SetDeferBeginMainFrame(defer_begin_main_frame);
        }
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:969-976
    pub fn SetPauseRendering(&mut self, pause_rendering: bool, client: &mut dyn SchedulerClient) {
        {
            let _trace = self.trace_scope("cc", "Scheduler::SetPauseRendering", || {
                vec![("pause_rendering", TraceValue::Bool(pause_rendering))]
            });
            self.state_machine_.SetPauseRendering(pause_rendering);
        }
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:978-1073
    pub fn ProcessScheduledActions(&mut self, client: &mut dyn SchedulerClient) {
        if self.stopped_ {
            return;
        }
        if self.inside_process_scheduled_actions_.get() || self.inside_scheduled_action_.get() {
            return;
        }
        let _mark = self.inside_process_scheduled_actions_.reset(true);
        loop {
            let action = self.state_machine_.NextAction();
            let _trace = self.trace_scope(
                "disabled-by-default-cc.debug.scheduler",
                "SchedulerStateMachine",
                || vec![],
            );
            if self
                .host_
                .TraceCategoryPointerPresent("disabled-by-default-cc.debug.scheduler")
            {
                let state = self.AsProtozeroInto();
                self.host_.SchedulerStateTrace(state);
            }
            let _mark_action = self.inside_action_.reset(action);
            match action {
                Action::NONE => {}
                Action::SEND_BEGIN_MAIN_FRAME => {
                    self.compositor_timing_history_
                        .WillBeginMainFrame(&self.begin_main_frame_args_);
                    self.reporting()
                        .WillBeginMainFrame(&self.begin_main_frame_args_);
                    self.state_machine_.WillSendBeginMainFrame();
                    let args = self.begin_main_frame_args_.clone();
                    client.ScheduledActionSendBeginMainFrame(&args, self);
                    self.last_dispatched_begin_main_frame_args_ =
                        self.begin_main_frame_args_.clone();
                }
                Action::COMMIT => {
                    self.state_machine_.WillCommit(false);
                    self.compositor_timing_history_.WillCommit();
                    self.reporting().WillCommit();
                    client.ScheduledActionCommit(self);
                    self.compositor_timing_history_.DidCommit();
                    self.reporting().DidCommit();
                    self.state_machine_.DidCommit();
                    self.last_commit_origin_frame_args_ =
                        self.next_commit_origin_frame_args_.clone();
                }
                Action::POST_COMMIT => {
                    client.ScheduledActionPostCommit(self);
                    self.state_machine_.DidPostCommit();
                }
                Action::ACTIVATE_SYNC_TREE => {
                    self.compositor_timing_history_.WillActivate();
                    self.reporting().WillActivate();
                    self.state_machine_.WillActivate();
                    client.ScheduledActionActivateSyncTree(self);
                    self.compositor_timing_history_.DidActivate();
                    self.reporting().DidActivate();
                    self.last_activate_origin_frame_args_ =
                        self.last_commit_origin_frame_args_.clone();
                }
                Action::PERFORM_IMPL_SIDE_INVALIDATION => {
                    self.state_machine_.WillPerformImplSideInvalidation();
                    self.compositor_timing_history_.WillInvalidateOnImplSide();
                    self.reporting().WillInvalidateOnImplSide();
                    client.ScheduledActionPerformImplSideInvalidation(self);
                }
                Action::DRAW_IF_POSSIBLE => self.DrawIfPossible(client),
                Action::DRAW_FORCED => self.DrawForced(client),
                Action::DRAW_ABORT => self.state_machine_.AbortDraw(),
                Action::BEGIN_LAYER_TREE_FRAME_SINK_CREATION => {
                    self.state_machine_.WillBeginLayerTreeFrameSinkCreation();
                    client.ScheduledActionBeginLayerTreeFrameSinkCreation(self);
                }
                Action::PREPARE_TILES => {
                    self.state_machine_.WillPrepareTiles();
                    client.ScheduledActionPrepareTiles(self);
                }
                Action::INVALIDATE_LAYER_TREE_FRAME_SINK => {
                    self.state_machine_.WillInvalidateLayerTreeFrameSink();
                    let needs_redraw = self.state_machine_.RedrawPending();
                    client.ScheduledActionInvalidateLayerTreeFrameSink(needs_redraw, self);
                }
            }
            if action == Action::NONE {
                break;
            }
        }
        self.ScheduleBeginImplFrameDeadline(client);
        self.PostPendingBeginFrameTask();
        self.StartOrStopBeginFrames(client);
    }

    // Source: cc/scheduler/scheduler.cc:1075-1113
    pub fn AsProtozeroInto(&self) -> SchedulerTrace {
        let now = self.Now();
        let mut state_machine = SchedulerStateTrace::default();
        self.state_machine_.AsProtozeroInto(&mut state_machine);
        let mut begin_impl_frame_args = BeginImplFrameArgsTrace::default();
        self.begin_impl_frame_tracker_
            .AsProtozeroInto(now, &mut begin_impl_frame_args);
        SchedulerTrace {
            state_machine,
            observing_begin_frame_source: self.observing_begin_frame_source_,
            begin_impl_frame_deadline_task: self.begin_impl_frame_deadline_timer_.IsRunning(),
            pending_begin_frame_task: !self.pending_begin_frame_task_.IsCancelled(),
            skipped_last_frame_missed_exceeded_deadline: self
                .skipped_last_frame_missed_exceeded_deadline_,
            inside_action: self.inside_action_.get(),
            deadline_mode: self.deadline_mode_,
            deadline_us: self.deadline_.since_origin().InMicroseconds(),
            deadline_scheduled_at_us: self.deadline_scheduled_at_.since_origin().InMicroseconds(),
            now_us: self.Now().since_origin().InMicroseconds(),
            now_to_deadline_delta_us: (self.deadline_ - self.Now()).InMicroseconds(),
            now_to_deadline_scheduled_at_delta_us: (self.deadline_scheduled_at_ - self.Now())
                .InMicroseconds(),
            begin_impl_frame_args,
            begin_frame_observer_state: BeginFrameObserverTrace {
                last_begin_frame_args: self.last_begin_frame_args_.clone(),
                dropped_begin_frame_args: self.dropped_begin_frame_args_,
            },
            begin_frame_source_state: self
                .begin_frame_source_
                .as_ref()
                .map(|source| source.AsProtozeroInto()),
        }
    }

    // Source: cc/scheduler/scheduler.cc:1115-1119
    pub fn UpdateCompositorTimingHistoryRecordingEnabled(&mut self) {
        self.compositor_timing_history_.SetRecordingEnabled(
            self.state_machine_.HasInitializedLayerTreeFrameSink() && self.state_machine_.visible(),
        );
    }

    // Source: cc/scheduler/scheduler.cc:1121-1124
    pub fn CommitDurationSampleCountForTesting(&self) -> usize {
        return self
            .compositor_timing_history_
            .CommitDurationSampleCountForTesting(); // IN-TEST
    }

    // Source: cc/scheduler/scheduler.cc:1126-1128
    pub fn CurrentBeginFrameAckForActiveTree(&self) -> BeginFrameAck {
        BeginFrameAck::new(&self.begin_main_frame_args_, true)
    }

    // Source: cc/scheduler/scheduler.cc:1130-1134
    pub fn ClearHistory(&mut self, client: &mut dyn SchedulerClient) {
        // Ensure we reset decisions based on history from the previous navigation.
        self.compositor_timing_history_.ClearHistory();
        self.ProcessScheduledActions(client);
    }

    // Source: cc/scheduler/scheduler.cc:1137-1140
    pub fn SetRequestHighFramerate(&mut self, flag: bool) {
        let _trace = self.trace_scope("blink", "Scheduler::SetRequestHighFramerate", || vec![]);
        self.state_machine_.SetRequestHighFramerate(flag);
    }

    // Source: cc/scheduler/scheduler.h:394,404-405
    // AdvanceCommitStateIfPossible, PollToAdvanceCommitState and
    // BeginMainFrameAnimateAndLayoutOnly are unreferenced declarations with no
    // definitions in the source checkout; Rust exposes no fabricated bodies.

    // Rust adapters to the source host/trace dependencies.
    fn reporting(&self) -> Arc<dyn CompositorFrameReportingController> {
        self.compositor_frame_reporting_controller_
            .as_ref()
            .expect("reporting controller used after Scheduler::TearDown")
            .clone()
    }
    fn trace_scope(
        &self,
        category: &'static str,
        name: &'static str,
        args: impl FnOnce() -> Vec<(&'static str, TraceValue)>,
    ) -> SchedulerTraceScope {
        SchedulerTraceScope::new(self.host_.clone(), category, name, args)
    }
    fn trace_instant(
        &self,
        category: &'static str,
        name: &'static str,
        args: impl FnOnce() -> Vec<(&'static str, TraceValue)>,
    ) {
        if self.host_.tracing_enabled(category) {
            self.host_.Trace(SchedulerTraceEvent {
                category,
                name,
                kind: SchedulerTraceKind::Instant,
                arguments: args(),
                track: None,
            });
        }
    }
    // Source: components/viz/common/frame_sinks/begin_frame_args.cc:182-200
    fn frame_trace(&self, args: &BeginFrameArgs) -> TraceValue {
        let mut state = std::collections::BTreeMap::new();
        state.insert("type".into(), TraceValue::String("BeginFrameArgs".into()));
        state.insert(
            "subtype".into(),
            TraceValue::String(
                match args.type_ {
                    BeginFrameArgsType::INVALID => "INVALID",
                    BeginFrameArgsType::NORMAL => "NORMAL",
                    BeginFrameArgsType::MISSED => "MISSED",
                }
                .into(),
            ),
        );
        state.insert(
            "source_id".into(),
            TraceValue::Integer(args.frame_id.source_id as i64),
        );
        state.insert(
            "sequence_number".into(),
            TraceValue::Integer(args.frame_id.sequence_number as i64),
        );
        state.insert(
            "frames_throttled_since_last".into(),
            TraceValue::Integer(args.frames_throttled_since_last as i64),
        );
        state.insert(
            "frame_time_us".into(),
            TraceValue::Float(args.frame_time.since_origin().InMicrosecondsF()),
        );
        state.insert(
            "deadline_us".into(),
            TraceValue::Float(args.deadline.since_origin().InMicrosecondsF()),
        );
        state.insert(
            "interval_us".into(),
            TraceValue::Float(args.interval.InMicrosecondsF()),
        );
        state.insert(
            "unthrottled_interval_us".into(),
            TraceValue::Float(args.unthrottled_interval.InMicrosecondsF()),
        );
        #[cfg(debug_assertions)]
        state.insert(
            "created_from".into(),
            TraceValue::String(
                args.created_from
                    .as_ref()
                    .map(|location| location.ToString())
                    .unwrap_or_default(),
            ),
        );
        state.insert(
            "on_critical_path".into(),
            TraceValue::Bool(args.on_critical_path),
        );
        state.insert("animate_only".into(), TraceValue::Bool(args.animate_only));
        state.insert(
            "has_possible_deadlines".into(),
            TraceValue::Bool(args.possible_deadlines.is_some()),
        );
        TraceValue::Dictionary(state)
    }

    /// Delivers one host task. Canceled/replaced task generations are inert.
    pub fn RunTask(&mut self, task: SchedulerTask, client: &mut dyn SchedulerClient) {
        match task.kind {
            SchedulerTaskKind::BeginImplFrameDeadline => {
                if self.begin_impl_frame_deadline_timer_.Consume(task) {
                    self.OnBeginImplFrameDeadline(client);
                }
            }
            SchedulerTaskKind::PendingBeginFrame => {
                if self.pending_begin_frame_task_.Consume(task) {
                    self.HandlePendingBeginFrame(client);
                }
            }
        }
    }

    // Source: components/viz/common/frame_sinks/begin_frame_source.cc:83-96
    pub fn OnBeginFrame(&mut self, args: &BeginFrameArgs, client: &mut dyn SchedulerClient) {
        debug_assert!(args.IsValid());
        debug_assert!(args.frame_time >= self.last_begin_frame_args_.frame_time);
        debug_assert!(
            args.frame_id.source_id == BeginFrameArgs::kManualSourceId
                || !(self.last_begin_frame_args_.frame_id.source_id == args.frame_id.source_id
                    && self.last_begin_frame_args_.frame_id.sequence_number
                        > args.frame_id.sequence_number)
        );
        let used = self.OnBeginFrameDerivedImpl(args, client);
        if used {
            self.last_begin_frame_args_ = args.clone();
        } else {
            self.dropped_begin_frame_args_ += 1;
        }
    }
    // Source: components/viz/common/frame_sinks/begin_frame_source.cc:75-81
    pub fn LastUsedBeginFrameArgs(&self) -> &BeginFrameArgs {
        &self.last_begin_frame_args_
    }
    pub fn WantsAnimateOnlyBeginFrames(&self) -> bool {
        self.wants_animate_only_begin_frames_
    }
}

// Source: cc/scheduler/scheduler.cc:90-92
impl Drop for Scheduler {
    fn drop(&mut self) {
        let pending_observer = self.SetBeginFrameSourceState(None);
        debug_assert!(pending_observer.is_none());
        self.begin_impl_frame_deadline_timer_
            .Cancel(self.host_.as_ref());
        self.pending_begin_frame_task_.Cancel(self.host_.as_ref());
    }
}

/// Source constructor dispatch, assembled from the translated derived classes.
pub struct DefaultSchedulerStateMachineFactory;
impl SchedulerStateMachineFactory for DefaultSchedulerStateMachineFactory {
    // Source: cc/scheduler/scheduler.cc:66-77
    fn Create(
        &self,
        kind: SchedulerStateMachineKind,
        settings: &SchedulerSettings,
        host: Arc<dyn SchedulerHost>,
    ) -> SchedulerStateMachine {
        let mut state = SchedulerStateMachine::new(settings, host);
        match kind {
            SchedulerStateMachineKind::Classic => {}
            SchedulerStateMachineKind::Headless => state.SetOverrides(Arc::new(
                super::headless_scheduler_state_machine::HeadlessSchedulerStateMachine::new(
                    settings,
                ),
            )),
            SchedulerStateMachineKind::Webview => state.SetOverrides(Arc::new(
                super::webview_scheduler_state_machine::WebviewSchedulerStateMachine::new(settings),
            )),
            SchedulerStateMachineKind::Slim => state.SetOverrides(Arc::new(
                super::slim_scheduler_state_machine::SlimSchedulerStateMachine::new(settings),
            )),
        }
        state
    }
}
