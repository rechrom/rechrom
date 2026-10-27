// Translation status: complete
// Source: cc/metrics/compositor_timing_history.h:1-138
// Source: cc/metrics/compositor_timing_history.cc:1-168
use crate::{
    begin_frame::BeginFrameArgs, rolling_time_delta_history::RollingTimeDeltaHistory, TimeDelta,
    TimeTicks,
};
use std::sync::Arc;

// Source: cc/metrics/compositor_timing_history.cc:20-32
const kDurationHistorySize: usize = 60;
const kBeginMainFrameQueueDurationEstimationPercentile: f64 = 90.0;
const kBeginMainFrameQueueDurationCriticalEstimationPercentile: f64 = 90.0;
const kBeginMainFrameQueueDurationNotCriticalEstimationPercentile: f64 = 90.0;
const kBeginMainFrameStartToReadyToCommitEstimationPercentile: f64 = 90.0;
const kCommitEstimatePercentile: f64 = 90.0;
const kCommitToReadyToActivateEstimationPercentile: f64 = 90.0;
const kActivateEstimationPercentile: f64 = 90.0;
const kDrawEstimationPercentile: f64 = 90.0;
// Source: cc/metrics/compositor_timing_history.cc:34-149
pub const kUMAVSyncBuckets: &[i32] = &[
    1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 3906, 4032, 4167, 4310, 4464, 4630, 4808,
    5000, 5208, 5435, 5682, 5952, 6250, 6579, 6944, 7353, 7813, 8065, 8333, 8621, 8929, 9259, 9615,
    10000, 10417, 10870, 11364, 11905, 12500, 13158, 13889, 14706, 15625, 16129, 16667, 17241,
    17857, 18519, 19231, 20000, 20833, 21739, 22727, 23810, 25000, 26316, 27778, 29412, 31250,
    32258, 33333, 34483, 35714, 37037, 38462, 40000, 41667, 43478, 45455, 47619, 50000, 52632,
    55556, 58824, 62500, 66667, 71429, 76923, 83333, 90909, 100000, 111111, 125000, 142857, 166667,
    200000, 250000, 333333, 500000, 1000000, 2000000, 4000000, 8000000, 16000000, 32000000,
];

// Native clock and metrics capabilities only, never a Host/Scheduler callback.
pub trait TimingHistoryServices: Send + Sync {
    fn Now(&self) -> TimeTicks;
    fn DrawIntervalHistogram(&self, name: &'static str, sample: TimeDelta, buckets: &[i32]);
    fn TraceClearHistory(&self);
}
// Source: cc/metrics/compositor_timing_history.h:28-32
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UMACategory {
    RENDERER_UMA,
    BROWSER_UMA,
    NULL_UMA,
}
// Source: cc/metrics/compositor_timing_history.h:87-133
pub struct CompositorTimingHistory {
    enabled_: bool,
    compositor_drawing_continuously_: bool,
    draw_end_time_prev_: TimeTicks,
    begin_main_frame_queue_duration_history_: RollingTimeDeltaHistory,
    begin_main_frame_queue_duration_critical_history_: RollingTimeDeltaHistory,
    begin_main_frame_queue_duration_not_critical_history_: RollingTimeDeltaHistory,
    begin_main_frame_start_to_ready_to_commit_duration_history_: RollingTimeDeltaHistory,
    commit_duration_history_: RollingTimeDeltaHistory,
    commit_to_ready_to_activate_duration_history_: RollingTimeDeltaHistory,
    activate_duration_history_: RollingTimeDeltaHistory,
    draw_duration_history_: RollingTimeDeltaHistory,
    begin_main_frame_queue_duration_: TimeDelta,
    pending_tree_bmf_queue_duration_: TimeDelta,
    bmf_start_to_ready_to_activate_duration_: TimeDelta,
    begin_main_frame_on_critical_path_: bool,
    pending_commit_on_critical_path_: bool,
    pending_tree_on_critical_path_: bool,
    begin_main_frame_sent_time_: TimeTicks,
    begin_main_frame_start_time_: TimeTicks,
    ready_to_commit_time_: TimeTicks,
    commit_start_time_: TimeTicks,
    pending_tree_creation_time_: TimeTicks,
    pending_tree_ready_to_activate_time_: TimeTicks,
    activate_start_time_: TimeTicks,
    draw_start_time_: TimeTicks,
    pending_tree_is_impl_side_: bool,
    uma_category_: UMACategory,
    services: Arc<dyn TimingHistoryServices>,
}
impl CompositorTimingHistory {
    // Source: cc/metrics/compositor_timing_history.cc:151-165
    pub fn new(uma_category: UMACategory, services: Arc<dyn TimingHistoryServices>) -> Self {
        Self {
            enabled_: false,
            compositor_drawing_continuously_: false,
            draw_end_time_prev_: TimeTicks::default(),
            begin_main_frame_queue_duration_history_: RollingTimeDeltaHistory::new(
                kDurationHistorySize,
            ),
            begin_main_frame_queue_duration_critical_history_: RollingTimeDeltaHistory::new(
                kDurationHistorySize,
            ),
            begin_main_frame_queue_duration_not_critical_history_: RollingTimeDeltaHistory::new(
                kDurationHistorySize,
            ),
            begin_main_frame_start_to_ready_to_commit_duration_history_:
                RollingTimeDeltaHistory::new(kDurationHistorySize),
            commit_duration_history_: RollingTimeDeltaHistory::new(kDurationHistorySize),
            commit_to_ready_to_activate_duration_history_: RollingTimeDeltaHistory::new(
                kDurationHistorySize,
            ),
            activate_duration_history_: RollingTimeDeltaHistory::new(kDurationHistorySize),
            draw_duration_history_: RollingTimeDeltaHistory::new(kDurationHistorySize),
            begin_main_frame_queue_duration_: TimeDelta::default(),
            pending_tree_bmf_queue_duration_: TimeDelta::default(),
            bmf_start_to_ready_to_activate_duration_: TimeDelta::default(),
            begin_main_frame_on_critical_path_: false,
            pending_commit_on_critical_path_: false,
            pending_tree_on_critical_path_: false,
            begin_main_frame_sent_time_: TimeTicks::default(),
            begin_main_frame_start_time_: TimeTicks::default(),
            ready_to_commit_time_: TimeTicks::default(),
            commit_start_time_: TimeTicks::default(),
            pending_tree_creation_time_: TimeTicks::default(),
            pending_tree_ready_to_activate_time_: TimeTicks::default(),
            activate_start_time_: TimeTicks::default(),
            draw_start_time_: TimeTicks::default(),
            pending_tree_is_impl_side_: false,
            uma_category_: uma_category,
            services,
        }
    }
    // Source: cc/metrics/compositor_timing_history.cc:169-171
    fn Now(&self) -> TimeTicks {
        self.services.Now()
    }
    // Source: cc/metrics/compositor_timing_history.cc:173-175
    pub fn SetRecordingEnabled(&mut self, enabled: bool) {
        self.enabled_ = enabled;
    }
    // Source: cc/metrics/compositor_timing_history.cc:177-182
    fn SetCompositorDrawingContinuously(&mut self, active: bool) {
        if active == self.compositor_drawing_continuously_ {
            return;
        }
        self.draw_end_time_prev_ = TimeTicks::default();
        self.compositor_drawing_continuously_ = active;
    }
    // Source: cc/metrics/compositor_timing_history.cc:184-194
    pub fn BeginMainFrameQueueDurationCriticalEstimate(&mut self) -> TimeDelta {
        let all = self
            .begin_main_frame_queue_duration_history_
            .Percentile(kBeginMainFrameQueueDurationEstimationPercentile);
        let critical = self
            .begin_main_frame_queue_duration_critical_history_
            .Percentile(kBeginMainFrameQueueDurationCriticalEstimationPercentile);
        critical.min(all)
    }
    // Source: cc/metrics/compositor_timing_history.cc:196-207
    pub fn BeginMainFrameQueueDurationNotCriticalEstimate(&mut self) -> TimeDelta {
        let all = self
            .begin_main_frame_queue_duration_history_
            .Percentile(kBeginMainFrameQueueDurationEstimationPercentile);
        let not_critical = self
            .begin_main_frame_queue_duration_not_critical_history_
            .Percentile(kBeginMainFrameQueueDurationNotCriticalEstimationPercentile);
        not_critical.max(all)
    }
    // Source: cc/metrics/compositor_timing_history.cc:209-214
    pub fn BeginMainFrameStartToReadyToCommitDurationEstimate(&mut self) -> TimeDelta {
        self.begin_main_frame_start_to_ready_to_commit_duration_history_
            .Percentile(kBeginMainFrameStartToReadyToCommitEstimationPercentile)
    }
    // Source: cc/metrics/compositor_timing_history.cc:216-218
    pub fn CommitDurationEstimate(&mut self) -> TimeDelta {
        self.commit_duration_history_
            .Percentile(kCommitEstimatePercentile)
    }
    // Source: cc/metrics/compositor_timing_history.cc:220-224
    pub fn CommitToReadyToActivateDurationEstimate(&mut self) -> TimeDelta {
        self.commit_to_ready_to_activate_duration_history_
            .Percentile(kCommitToReadyToActivateEstimationPercentile)
    }
    // Source: cc/metrics/compositor_timing_history.cc:226-228
    pub fn ActivateDurationEstimate(&mut self) -> TimeDelta {
        self.activate_duration_history_
            .Percentile(kActivateEstimationPercentile)
    }
    // Source: cc/metrics/compositor_timing_history.cc:230-232
    pub fn DrawDurationEstimate(&mut self) -> TimeDelta {
        self.draw_duration_history_
            .Percentile(kDrawEstimationPercentile)
    }
    // Source: cc/metrics/compositor_timing_history.cc:234-239
    pub fn BeginMainFrameStartToReadyToCommitCriticalEstimate(&mut self) -> TimeDelta {
        self.BeginMainFrameStartToReadyToCommitDurationEstimate()
            + self.BeginMainFrameQueueDurationCriticalEstimate()
    }
    // Source: cc/metrics/compositor_timing_history.cc:241-246
    pub fn BeginMainFrameStartToReadyToCommitNotCriticalEstimate(&mut self) -> TimeDelta {
        self.BeginMainFrameStartToReadyToCommitDurationEstimate()
            + self.BeginMainFrameQueueDurationNotCriticalEstimate()
    }
    // Source: cc/metrics/compositor_timing_history.cc:248-254
    pub fn BeginMainFrameQueueToActivateCriticalEstimate(&mut self) -> TimeDelta {
        self.BeginMainFrameStartToReadyToCommitDurationEstimate()
            + self.CommitDurationEstimate()
            + self.CommitToReadyToActivateDurationEstimate()
            + self.ActivateDurationEstimate()
            + self.BeginMainFrameQueueDurationCriticalEstimate()
    }
    // Source: cc/metrics/compositor_timing_history.cc:256-259
    pub fn WillFinishImplFrame(&mut self, needs_redraw: bool) {
        if !needs_redraw {
            self.SetCompositorDrawingContinuously(false);
        }
    }
    // Source: cc/metrics/compositor_timing_history.cc:261-263
    pub fn BeginImplFrameNotExpectedSoon(&mut self) {
        self.SetCompositorDrawingContinuously(false);
    }
    // Source: cc/metrics/compositor_timing_history.cc:265-271
    pub fn WillBeginMainFrame(&mut self, args: &BeginFrameArgs) {
        debug_assert_eq!(TimeTicks::default(), self.begin_main_frame_sent_time_);
        self.begin_main_frame_on_critical_path_ = args.on_critical_path;
        self.begin_main_frame_sent_time_ = self.Now();
    }
    // Source: cc/metrics/compositor_timing_history.cc:273-278
    pub fn BeginMainFrameStarted(&mut self, main_thread_start_time: TimeTicks) {
        debug_assert_ne!(TimeTicks::default(), self.begin_main_frame_sent_time_);
        debug_assert_eq!(TimeTicks::default(), self.begin_main_frame_start_time_);
        self.begin_main_frame_start_time_ = main_thread_start_time;
    }
    // Source: cc/metrics/compositor_timing_history.cc:280-283
    pub fn BeginMainFrameAborted(&mut self) {
        let begin_main_frame_end_time = self.Now();
        self.DidBeginMainFrame(begin_main_frame_end_time);
    }
    // Source: cc/metrics/compositor_timing_history.cc:285-295
    pub fn NotifyReadyToCommit(&mut self) {
        debug_assert_ne!(self.begin_main_frame_start_time_, TimeTicks::default());
        debug_assert_eq!(self.ready_to_commit_time_, TimeTicks::default());
        self.ready_to_commit_time_ = self.Now();
        self.pending_commit_on_critical_path_ = self.begin_main_frame_on_critical_path_;
        let bmf_duration = self.ready_to_commit_time_ - self.begin_main_frame_start_time_;
        self.DidBeginMainFrame(self.ready_to_commit_time_);
        self.begin_main_frame_start_to_ready_to_commit_duration_history_
            .InsertSample(bmf_duration);
    }
    // Source: cc/metrics/compositor_timing_history.cc:297-301
    pub fn WillCommit(&mut self) {
        debug_assert_ne!(self.ready_to_commit_time_, TimeTicks::default());
        self.commit_start_time_ = self.Now();
        self.ready_to_commit_time_ = TimeTicks::default();
    }
    // Source: cc/metrics/compositor_timing_history.cc:303-312
    pub fn DidCommit(&mut self) {
        debug_assert_eq!(self.pending_tree_creation_time_, TimeTicks::default());
        debug_assert_ne!(self.commit_start_time_, TimeTicks::default());
        let commit_end_time = self.Now();
        self.commit_duration_history_
            .InsertSample(commit_end_time - self.commit_start_time_);
        self.pending_tree_is_impl_side_ = false;
        self.pending_tree_creation_time_ = commit_end_time;
    }
    // Source: cc/metrics/compositor_timing_history.cc:314-342
    fn DidBeginMainFrame(&mut self, _begin_main_frame_end_time: TimeTicks) {
        debug_assert_ne!(TimeTicks::default(), self.begin_main_frame_sent_time_);
        if self.begin_main_frame_start_time_.is_null() {
            self.begin_main_frame_start_time_ = self.begin_main_frame_sent_time_;
        }
        if self.enabled_ {
            let bmf_queue_duration =
                self.begin_main_frame_start_time_ - self.begin_main_frame_sent_time_;
            self.begin_main_frame_queue_duration_history_
                .InsertSample(bmf_queue_duration);
            if self.begin_main_frame_on_critical_path_ {
                self.begin_main_frame_queue_duration_critical_history_
                    .InsertSample(bmf_queue_duration);
            } else {
                self.begin_main_frame_queue_duration_not_critical_history_
                    .InsertSample(bmf_queue_duration);
            }
        }
        self.begin_main_frame_sent_time_ = TimeTicks::default();
        self.begin_main_frame_start_time_ = TimeTicks::default();
        self.begin_main_frame_on_critical_path_ = false;
    }
    // Source: cc/metrics/compositor_timing_history.cc:344-352
    pub fn WillInvalidateOnImplSide(&mut self) {
        debug_assert!(!self.pending_tree_is_impl_side_);
        debug_assert_eq!(self.pending_tree_creation_time_, TimeTicks::default());
        self.pending_tree_is_impl_side_ = true;
        self.pending_tree_on_critical_path_ = false;
        self.pending_tree_bmf_queue_duration_ = TimeDelta::default();
        self.pending_tree_creation_time_ = self.services.Now();
    }
    // Source: cc/metrics/compositor_timing_history.cc:354-365
    pub fn ReadyToActivate(&mut self) {
        debug_assert_ne!(self.pending_tree_creation_time_, TimeTicks::default());
        debug_assert_eq!(
            self.pending_tree_ready_to_activate_time_,
            TimeTicks::default()
        );
        self.pending_tree_ready_to_activate_time_ = self.Now();
        if !self.pending_tree_is_impl_side_ && self.enabled_ {
            let time_since_commit =
                self.pending_tree_ready_to_activate_time_ - self.pending_tree_creation_time_;
            self.commit_to_ready_to_activate_duration_history_
                .InsertSample(time_since_commit);
        }
    }
    // Source: cc/metrics/compositor_timing_history.cc:367-374
    pub fn WillActivate(&mut self) {
        debug_assert_eq!(TimeTicks::default(), self.activate_start_time_);
        self.activate_start_time_ = self.Now();
        self.pending_tree_is_impl_side_ = false;
        self.pending_tree_creation_time_ = TimeTicks::default();
    }
    // Source: cc/metrics/compositor_timing_history.cc:376-387
    pub fn DidActivate(&mut self) {
        debug_assert_ne!(TimeTicks::default(), self.activate_start_time_);
        let activate_end_time = self.Now();
        let activate_duration = activate_end_time - self.activate_start_time_;
        if self.enabled_ {
            self.activate_duration_history_
                .InsertSample(activate_duration);
        }
        self.pending_tree_ready_to_activate_time_ = TimeTicks::default();
        self.activate_start_time_ = TimeTicks::default();
    }
    // Source: cc/metrics/compositor_timing_history.cc:389-392
    pub fn WillDraw(&mut self) {
        debug_assert_eq!(TimeTicks::default(), self.draw_start_time_);
        self.draw_start_time_ = self.Now();
    }
    // Source: cc/metrics/compositor_timing_history.cc:394-411
    pub fn DidDraw(&mut self) {
        debug_assert_ne!(TimeTicks::default(), self.draw_start_time_);
        let draw_end_time = self.Now();
        if self.enabled_ {
            let draw_duration = draw_end_time - self.draw_start_time_;
            self.draw_duration_history_.InsertSample(draw_duration);
        }
        self.SetCompositorDrawingContinuously(true);
        if !self.draw_end_time_prev_.is_null() && self.uma_category_ == UMACategory::RENDERER_UMA {
            let draw_interval = draw_end_time - self.draw_end_time_prev_;
            self.services.DrawIntervalHistogram(
                "Scheduling.Renderer.DrawInterval2",
                draw_interval,
                kUMAVSyncBuckets,
            );
        }
        self.draw_end_time_prev_ = draw_end_time;
        self.draw_start_time_ = TimeTicks::default();
    }
    // Source: cc/metrics/compositor_timing_history.h:76-78
    pub fn begin_main_frame_sent_time(&self) -> TimeTicks {
        self.begin_main_frame_sent_time_
    }
    // Source: cc/metrics/compositor_timing_history.cc:413-424
    pub fn ClearHistory(&mut self) {
        self.services.TraceClearHistory();
        self.begin_main_frame_queue_duration_history_.Clear();
        self.begin_main_frame_queue_duration_critical_history_
            .Clear();
        self.begin_main_frame_queue_duration_not_critical_history_
            .Clear();
        self.begin_main_frame_start_to_ready_to_commit_duration_history_
            .Clear();
        self.commit_duration_history_.Clear();
        self.commit_to_ready_to_activate_duration_history_.Clear();
        self.activate_duration_history_.Clear();
        self.draw_duration_history_.Clear();
    }
    // Source: cc/metrics/compositor_timing_history.cc:426-428
    pub fn CommitDurationSampleCountForTesting(&self) -> usize {
        self.commit_duration_history_.sample_count()
    }
}
