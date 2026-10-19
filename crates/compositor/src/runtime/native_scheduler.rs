//! Native scheduling placement and diagnostics. Canonical Scheduler owns all
//! scheduling policy; queued closures carry task values and cancellation only.
use super::{
    execution::{EngineTask, EngineTaskQueue},
    native_services::NativeEventSink,
};
use crate::{
    adapter::{
        begin_frame::BeginFrameArgs,
        scheduler::{SchedulerFeatures, SchedulerServices, SchedulerTraceEvent},
        scheduler_host::{SchedulerHost, SchedulerTask, SchedulerTrace},
        time::{TimeDelta, TimeTicks},
        trace_sink::TraceEvent,
    },
    cc::scheduler::scheduler_state_machine::BeginImplFrameDeadlineMode,
    engine::CompositorExecutor,
};
use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

pub(crate) struct NativeSchedulerConfig {
    pub features: SchedulerFeatures,
    pub slim_scheduler: bool,
    pub webview_scheduler: bool,
    pub registered_trace_categories: BTreeSet<&'static str>,
    pub enabled_trace_categories: BTreeSet<&'static str>,
}
#[derive(Clone, Debug)]
pub(crate) enum NativeSchedulerEffect {
    NeedsBeginFrameChanged { host_id: i32, needed: bool },
}
#[derive(Clone, Default)]
pub(crate) struct NativeSchedulerEffects(Arc<Mutex<VecDeque<NativeSchedulerEffect>>>);
impl NativeSchedulerEffects {
    pub(crate) fn Pop(&self) -> Option<NativeSchedulerEffect> {
        self.0
            .lock()
            .expect("scheduler effect transport")
            .pop_front()
    }
    pub(crate) fn IsEmpty(&self) -> bool {
        self.0
            .lock()
            .expect("scheduler effect transport")
            .is_empty()
    }
}
pub(crate) struct NativeSchedulerHost {
    observer_id: u64,
    executor: Arc<dyn CompositorExecutor>,
    commands: EngineTaskQueue,
    effects: NativeSchedulerEffects,
    events: Arc<dyn NativeEventSink>,
    config: Mutex<NativeSchedulerConfig>,
    pending: Arc<Mutex<HashMap<SchedulerTask, Arc<AtomicBool>>>>,
}
impl NativeSchedulerHost {
    pub(crate) fn new(
        observer_id: u64,
        executor: Arc<dyn CompositorExecutor>,
        commands: EngineTaskQueue,
        effects: NativeSchedulerEffects,
        events: Arc<dyn NativeEventSink>,
        config: NativeSchedulerConfig,
    ) -> Self {
        assert_ne!(observer_id, 0);
        Self {
            observer_id,
            executor,
            commands,
            effects,
            events,
            config: Mutex::new(config),
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
impl SchedulerServices for NativeSchedulerHost {
    fn trace_track_id(&self) -> u64 {
        self.observer_id
    }
    fn features(&self) -> SchedulerFeatures {
        self.config
            .lock()
            .expect("native scheduler feature inputs")
            .features
    }
    fn SetIsEligibleForThrottleMainFrameTo60Hz(&self, eligible: bool) {
        self.config
            .lock()
            .expect("native scheduler feature inputs")
            .features
            .is_eligible_for_throttle_main_frame_to_60hz = eligible;
    }
    fn tracing_enabled(&self, category: &'static str) -> bool {
        self.config
            .lock()
            .expect("native scheduler features")
            .enabled_trace_categories
            .contains(category)
    }
    fn Trace(&self, event: SchedulerTraceEvent) {
        self.events.Emit(event.category, format!("{event:?}"));
    }
    fn ShouldRecordSubsampledMetric(&self, probability: f64) -> bool {
        assert!((0.0..=1.0).contains(&probability));
        // Independent native sampling, not a fabricated fixed metric decision.
        let mut bytes = [0u8; 8];
        getrandom::fill(&mut bytes).expect("native metric sampling entropy");
        let uniform = (u64::from_ne_bytes(bytes) >> 11) as f64 / ((1u64 << 53) as f64);
        uniform < probability
    }
    fn HistogramBoolean(&self, name: &'static str, value: bool) {
        self.events
            .HistogramCount(name, if value { 1.0 } else { 0.0 });
    }
    fn Diagnostic(&self, message: &'static str, count: u32) {
        self.events
            .Emit("cc.scheduler", format!("{message} count={count}"));
    }
}
impl SchedulerHost for NativeSchedulerHost {
    fn TraceCategoryPointerPresent(&self, category: &'static str) -> bool {
        self.config
            .lock()
            .expect("native scheduler features")
            .registered_trace_categories
            .contains(category)
    }
    fn observer_id(&self) -> u64 {
        self.observer_id
    }
    fn Now(&self) -> crate::adapter::time::TimeTicks {
        self.executor.Now()
    }
    fn PostTask(
        &self,
        task: SchedulerTask,
        deadline: Option<crate::adapter::time::TimeTicks>,
        _precise: bool,
    ) {
        assert_eq!(task.observer_id, self.observer_id);
        let alive = Arc::new(AtomicBool::new(true));
        assert!(self
            .pending
            .lock()
            .expect("native scheduler tasks")
            .insert(task, alive.clone())
            .is_none());
        let pending = self.pending.clone();
        let commands = self.commands.clone();
        let delivery = Box::new(move || {
            let deliver = alive.swap(false, Ordering::AcqRel);
            pending
                .lock()
                .expect("native scheduler tasks")
                .remove(&task);
            if deliver {
                commands.Push(EngineTask::Scheduler(task));
            }
        });
        match deadline {
            Some(when) => self.executor.ExecuteAfter(
                (when - self.executor.Now()).max(crate::adapter::time::TimeDelta::default()),
                delivery,
            ),
            None => self.executor.Execute(delivery),
        }
    }
    fn CancelTask(&self, task: SchedulerTask) {
        assert_eq!(task.observer_id, self.observer_id);
        if let Some(alive) = self
            .pending
            .lock()
            .expect("native scheduler tasks")
            .remove(&task)
        {
            alive.store(false, Ordering::Release);
        }
    }
    fn SlimSchedulerEnabled(&self) -> bool {
        self.config
            .lock()
            .expect("native scheduler features")
            .slim_scheduler
    }
    fn WebviewSchedulerStateMachineEnabled(&self) -> bool {
        self.config
            .lock()
            .expect("native scheduler features")
            .webview_scheduler
    }
    fn NeedsBeginFrameChanged(&self, layer_tree_host_id: i32, needed: bool) {
        self.effects
            .0
            .lock()
            .expect("scheduler effect transport")
            .push_back(NativeSchedulerEffect::NeedsBeginFrameChanged {
                host_id: layer_tree_host_id,
                needed,
            });
    }
    fn HistogramDeadlineMode(&self, name: &'static str, mode: BeginImplFrameDeadlineMode) {
        self.events.HistogramCount(name, mode as u8 as f32);
    }
    fn BeginFrameTrace(&self, event: TraceEvent) {
        self.events.Emit("cc.begin_frame", format!("{event:?}"));
    }
    fn SchedulerStateTrace(&self, state: SchedulerTrace) {
        self.events.Emit("cc.scheduler.state", format!("{state:?}"));
    }
}
impl Drop for NativeSchedulerHost {
    fn drop(&mut self) {
        for (_, alive) in self.pending.lock().expect("native scheduler tasks").drain() {
            alive.store(false, Ordering::Release);
        }
    }
}

/// The adapter mutex owns the one canonical timing history. It is never held
/// across a Scheduler/Host callback; no timestamps or estimates are duplicated.
pub(crate) struct NativeTimingHistory {
    history: Mutex<crate::cc::metrics::compositor_timing_history::CompositorTimingHistory>,
}
struct NativeTimingServices {
    executor: Arc<dyn CompositorExecutor>,
    events: Arc<dyn NativeEventSink>,
}
impl crate::cc::metrics::compositor_timing_history::TimingHistoryServices for NativeTimingServices {
    fn Now(&self) -> crate::adapter::time::TimeTicks {
        self.executor.Now()
    }
    fn DrawIntervalHistogram(
        &self,
        name: &'static str,
        sample: crate::adapter::time::TimeDelta,
        buckets: &[i32],
    ) {
        self.events
            .HistogramCustomTimeBuckets(name, sample, buckets);
    }
    fn TraceClearHistory(&self) {
        self.events.Emit(
            "cc,benchmark",
            "CompositorTimingHistory::ClearHistory".to_string(),
        );
    }
}
impl NativeTimingHistory {
    pub(crate) fn new(
        category: crate::cc::metrics::compositor_timing_history::UMACategory,
        executor: Arc<dyn CompositorExecutor>,
        events: Arc<dyn NativeEventSink>,
    ) -> Self {
        Self {
            history: Mutex::new(
                crate::cc::metrics::compositor_timing_history::CompositorTimingHistory::new(
                    category,
                    Arc::new(NativeTimingServices { executor, events }),
                ),
            ),
        }
    }
}

impl crate::adapter::scheduler_host::CompositorTimingHistory for NativeTimingHistory {
    fn ReadyToActivate(&self) {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .ReadyToActivate()
    }
    fn NotifyReadyToCommit(&self) {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .NotifyReadyToCommit()
    }
    fn BeginMainFrameAborted(&self) {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .BeginMainFrameAborted()
    }
    fn BeginMainFrameStarted(&self, time: TimeTicks) {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .BeginMainFrameStarted(time)
    }
    fn BeginImplFrameNotExpectedSoon(&self) {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .BeginImplFrameNotExpectedSoon()
    }
    fn DrawDurationEstimate(&self) -> TimeDelta {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .DrawDurationEstimate()
    }
    fn BeginMainFrameQueueToActivateCriticalEstimate(&self) -> TimeDelta {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .BeginMainFrameQueueToActivateCriticalEstimate()
    }
    fn begin_main_frame_sent_time(&self) -> TimeTicks {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .begin_main_frame_sent_time()
    }
    fn BeginMainFrameStartToReadyToCommitCriticalEstimate(&self) -> TimeDelta {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .BeginMainFrameStartToReadyToCommitCriticalEstimate()
    }
    fn BeginMainFrameStartToReadyToCommitNotCriticalEstimate(&self) -> TimeDelta {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .BeginMainFrameStartToReadyToCommitNotCriticalEstimate()
    }
    fn WillFinishImplFrame(&self, needs_redraw: bool) {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .WillFinishImplFrame(needs_redraw)
    }
    fn WillDraw(&self) {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .WillDraw()
    }
    fn DidDraw(&self) {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .DidDraw()
    }
    fn WillBeginMainFrame(&self, args: &BeginFrameArgs) {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .WillBeginMainFrame(args)
    }
    fn WillCommit(&self) {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .WillCommit()
    }
    fn DidCommit(&self) {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .DidCommit()
    }
    fn WillActivate(&self) {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .WillActivate()
    }
    fn DidActivate(&self) {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .DidActivate()
    }
    fn WillInvalidateOnImplSide(&self) {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .WillInvalidateOnImplSide()
    }
    fn SetRecordingEnabled(&self, enabled: bool) {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .SetRecordingEnabled(enabled)
    }
    fn CommitDurationSampleCountForTesting(&self) -> usize {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .CommitDurationSampleCountForTesting()
    }
    fn ClearHistory(&self) {
        self.history
            .lock()
            .expect("canonical compositor timing history")
            .ClearHistory()
    }
}
