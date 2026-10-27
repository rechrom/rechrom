//! Host ownership for Scheduler's task/source/clock/metrics dependencies.
//! Posting a task must enqueue it; delivery re-enters through Scheduler::RunTask.
//! Source AddObserver may synchronously deliver a missed BeginFrame, like Viz.

use crate::{
    begin_frame::{BeginFrameArgs, BeginFrameId},
    scheduler::FrameSkippedReason,
    scheduler_settings::SchedulerSettings,
    scheduler_state_machine::{Action, BeginImplFrameDeadlineMode, SchedulerStateMachine},
    services::{SchedulerServices, SchedulerStateTrace},
    trace_sink::{BeginImplFrameArgsTrace, TraceEvent},
    TimeDelta, TimeTicks, TraceValue,
};
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SchedulerTaskKind {
    BeginImplFrameDeadline,
    PendingBeginFrame,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SchedulerTask {
    pub observer_id: u64,
    pub kind: SchedulerTaskKind,
    pub generation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeginFrameAck {
    pub frame_id: BeginFrameId,
    pub trace_id: i64,
    pub has_damage: bool,
}
impl Default for BeginFrameAck {
    fn default() -> Self {
        Self {
            frame_id: BeginFrameId::default(),
            trace_id: -1,
            has_damage: false,
        }
    }
}
impl BeginFrameAck {
    pub fn new(args: &BeginFrameArgs, has_damage: bool) -> Self {
        Self {
            frame_id: args.frame_id,
            trace_id: args.trace_id,
            has_damage,
        }
    }
}

/// Tokens preserve CancelableOnceClosure/DeadlineTimer invalidation even when a
/// queued platform task cannot be physically removed from the host's queue.
pub struct SchedulerTaskSlot {
    kind: SchedulerTaskKind,
    generation: u64,
    pending: Option<SchedulerTask>,
}
impl SchedulerTaskSlot {
    pub fn new(kind: SchedulerTaskKind) -> Self {
        Self {
            kind,
            generation: 0,
            pending: None,
        }
    }
    pub fn IsRunning(&self) -> bool {
        self.pending.is_some()
    }
    pub fn IsCancelled(&self) -> bool {
        !self.IsRunning()
    }
    pub fn Cancel(&mut self, host: &dyn SchedulerHost) {
        if let Some(task) = self.pending.take() {
            host.CancelTask(task);
        }
        self.generation = self.generation.wrapping_add(1);
    }
    pub fn Reset(&mut self, host: &dyn SchedulerHost, deadline: Option<TimeTicks>, precise: bool) {
        self.Cancel(host);
        let task = SchedulerTask {
            observer_id: host.observer_id(),
            kind: self.kind,
            generation: self.generation,
        };
        self.pending = Some(task);
        host.PostTask(task, deadline, precise);
    }
    pub fn Consume(&mut self, task: SchedulerTask) -> bool {
        if self.pending == Some(task) {
            self.pending = None;
            true
        } else {
            false
        }
    }
}

/// Runtime-free Rust equivalent of C++ AutoReset that permits callbacks to
/// synchronously re-enter Scheduler without borrowing one of its fields.
pub struct ScopedValue<T: Copy> {
    value: Rc<Cell<T>>,
}
impl<T: Copy> ScopedValue<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Rc::new(Cell::new(value)),
        }
    }
    pub fn get(&self) -> T {
        self.value.get()
    }
    pub fn reset(&self, value: T) -> ScopedReset<T> {
        let old = self.value.replace(value);
        ScopedReset {
            value: self.value.clone(),
            old,
        }
    }
}
pub struct ScopedReset<T: Copy> {
    value: Rc<Cell<T>>,
    old: T,
}
impl<T: Copy> Drop for ScopedReset<T> {
    fn drop(&mut self) {
        self.value.set(self.old);
    }
}

/// Only the synchronous observer calls permitted during native AddObserver.
/// This scoped borrow may not escape registration. Future events are routed by
/// stable observer_id and delivered with a fresh Page/facade client borrow.
pub trait BeginFrameObserverCallback {
    fn OnBeginFrameSourcePausedChanged(&mut self, paused: bool);
    fn OnBeginFrame(&mut self, args: &BeginFrameArgs);
    fn LastUsedBeginFrameArgs(&self) -> &BeginFrameArgs;
    fn WantsAnimateOnlyBeginFrames(&self) -> bool;
}

pub trait BeginFrameSource {
    /// Rust mapping of C++ AddObserver's synchronous paused/MISSED callbacks.
    /// The source registers only observer_id. It may call this borrowed callback
    /// in native order before returning, and cannot retain it or a Scheduler.
    fn AddObserver(&self, observer_id: u64, callback: &mut dyn BeginFrameObserverCallback);
    fn RemoveObserver(&self, observer_id: u64);
    fn DidFinishFrame(&self, observer_id: u64); // Source passes kUnknown draw result.
    fn AsProtozeroInto(&self) -> TraceValue;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchedulerStateMachineKind {
    Classic,
    Headless,
    Webview,
    Slim,
}
pub trait SchedulerStateMachineFactory {
    fn Create(
        &self,
        kind: SchedulerStateMachineKind,
        settings: &SchedulerSettings,
        host: std::sync::Arc<dyn SchedulerHost>,
    ) -> SchedulerStateMachine;
}

pub trait SchedulerHost: SchedulerServices {
    /// Source tests the category flag pointer, not its current flag value.
    fn TraceCategoryPointerPresent(&self, _category: &'static str) -> bool {
        true
    }

    fn observer_id(&self) -> u64;
    fn Now(&self) -> TimeTicks;
    fn PostTask(&self, task: SchedulerTask, deadline: Option<TimeTicks>, precise: bool);
    fn CancelTask(&self, task: SchedulerTask);
    fn SlimSchedulerEnabled(&self) -> bool;
    fn WebviewSchedulerStateMachineEnabled(&self) -> bool;
    fn NeedsBeginFrameChanged(&self, layer_tree_host_id: i32, needed: bool);
    fn HistogramDeadlineMode(&self, name: &'static str, mode: BeginImplFrameDeadlineMode);
    fn BeginFrameTrace(&self, event: TraceEvent);
    fn SchedulerStateTrace(&self, state: SchedulerTrace);
}

#[derive(Clone, Debug, Default)]
pub struct BeginMainFrameMetrics {
    pub handle_input_events: TimeDelta,
    pub animate: TimeDelta,
    pub style_update: TimeDelta,
    pub layout_update: TimeDelta,
    pub accessibility: TimeDelta,
    pub prepaint: TimeDelta,
    pub compositing_inputs: TimeDelta,
    pub paint: TimeDelta,
    pub composite_commit: TimeDelta,
    pub update_layers: TimeDelta,
    pub should_measure_smoothness: bool,
}
/// Event/feedback contents belong to the metrics owner. Scheduler only carries
/// the host-owned identity through to that owner's reporting controller.
#[derive(Clone, Debug)]
pub struct MetricsPayload {
    pub id: u64,
}
/// The scheduler and Host transport the same move-only submission value.
/// Event owners must reach reporting intact; a detached numeric payload cannot
/// represent their lifetime or be silently synthesized by Engine glue.
pub trait SchedulerTypes: 'static {
    type SubmitInfo;
    type FrameTimingDetails;
}

pub trait CompositorTimingHistory {
    fn ReadyToActivate(&self);
    fn NotifyReadyToCommit(&self);
    fn BeginMainFrameAborted(&self);
    fn BeginMainFrameStarted(&self, time: TimeTicks);
    fn BeginImplFrameNotExpectedSoon(&self);
    fn DrawDurationEstimate(&self) -> TimeDelta;
    fn BeginMainFrameQueueToActivateCriticalEstimate(&self) -> TimeDelta;
    fn begin_main_frame_sent_time(&self) -> TimeTicks;
    fn BeginMainFrameStartToReadyToCommitCriticalEstimate(&self) -> TimeDelta;
    fn BeginMainFrameStartToReadyToCommitNotCriticalEstimate(&self) -> TimeDelta;
    fn WillFinishImplFrame(&self, needs_redraw: bool);
    fn WillDraw(&self);
    fn DidDraw(&self);
    fn WillBeginMainFrame(&self, args: &BeginFrameArgs);
    fn WillCommit(&self);
    fn DidCommit(&self);
    fn WillActivate(&self);
    fn DidActivate(&self);
    fn WillInvalidateOnImplSide(&self);
    fn SetRecordingEnabled(&self, enabled: bool);
    fn CommitDurationSampleCountForTesting(&self) -> usize;
    fn ClearHistory(&self);
}

pub trait CompositorFrameReportingController<B: SchedulerTypes> {
    fn OnFinishImplFrame(&self, frame_id: BeginFrameId, waiting_for_main: bool);
    fn DidSubmitCompositorFrame(
        &self,
        submit_info: &mut B::SubmitInfo,
        begin_main_frame_id: BeginFrameId,
        last_activated_frame_id: BeginFrameId,
    );
    fn NotifyReadyToCommit(&self, details: Option<BeginMainFrameMetrics>);
    fn BeginMainFrameAborted(
        &self,
        frame_id: BeginFrameId,
        reason: crate::commit_earlyout_reason::CommitEarlyOutReason,
    );
    fn DidPresentCompositorFrame(&self, frame_token: u32, details: &B::FrameTimingDetails);
    fn BeginMainFrameStarted(&self, time: TimeTicks);
    fn OnStoppedRequestingBeginFrames(&self);
    fn DidNotProduceFrame(&self, frame_id: BeginFrameId, reason: FrameSkippedReason);
    fn WillBeginImplFrame(&self, args: &BeginFrameArgs, throttled: bool);
    fn WillBeginMainFrame(&self, args: &BeginFrameArgs);
    fn WillCommit(&self);
    fn DidCommit(&self);
    fn WillActivate(&self);
    fn DidActivate(&self);
    fn WillInvalidateOnImplSide(&self);
}

#[derive(Clone, Debug, Default)]
pub struct BeginFrameObserverTrace {
    pub last_begin_frame_args: BeginFrameArgs,
    pub dropped_begin_frame_args: i64,
}
#[derive(Clone, Debug)]
pub struct SchedulerTrace {
    pub state_machine: SchedulerStateTrace,
    pub observing_begin_frame_source: bool,
    pub begin_impl_frame_deadline_task: bool,
    pub pending_begin_frame_task: bool,
    pub skipped_last_frame_missed_exceeded_deadline: bool,
    pub inside_action: Action,
    pub deadline_mode: BeginImplFrameDeadlineMode,
    pub deadline_us: i64,
    pub deadline_scheduled_at_us: i64,
    pub now_us: i64,
    pub now_to_deadline_delta_us: i64,
    pub now_to_deadline_scheduled_at_delta_us: i64,
    pub begin_impl_frame_args: BeginImplFrameArgsTrace,
    pub begin_frame_observer_state: BeginFrameObserverTrace,
    pub begin_frame_source_state: Option<TraceValue>,
}
