#![allow(non_snake_case)]

use std::time::{Duration, Instant};

use super::display_scheduler_base::DisplaySchedulerDrawResult;

// Source: components/viz/service/display/display_scheduler.h:1-38
// Source: components/viz/service/display/display_scheduler.cc:1-50
// BeginFrameSource observation, task posting, timers, ADPF hint sessions and
// presentation feedback cross the Engine boundary. The core returns explicit
// scheduling actions so the embedding can provide those platform services.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameTiming {
    pub frame_time: Instant,
    pub interval: Duration,
    pub source_deadline: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameDeadlines {
    pub compositor_prepare: Instant,
    pub draw_and_swap: Instant,
}

impl FrameTiming {
    pub fn Deadlines(self) -> FrameDeadlines {
        FrameDeadlines {
            compositor_prepare: self.frame_time + self.interval / 3,
            draw_and_swap: self
                .source_deadline
                .checked_sub(self.interval / 3)
                .unwrap_or(self.frame_time)
                .max(self.frame_time),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ScheduledFrame<T> {
    payload: T,
    timing: FrameTiming,
    deadlines: FrameDeadlines,
    prepared: bool,
    generation: u64,
    candidate_produced: bool,
    retry_pending: bool,
    retry_consumed: bool,
}

impl<T: Copy> ScheduledFrame<T> {
    pub fn Payload(self) -> T {
        self.payload
    }

    pub fn IsPrepared(self) -> bool {
        self.prepared
    }
}

// Source: components/viz/service/display/display_scheduler.h:39-89
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingSwapParams {
    pub max_pending_swaps: usize,
    pub max_pending_swaps_72hz: Option<usize>,
    pub max_pending_swaps_90hz: Option<usize>,
    pub max_pending_swaps_120hz: Option<usize>,
}

impl Default for PendingSwapParams {
    fn default() -> Self {
        Self {
            max_pending_swaps: 1,
            max_pending_swaps_72hz: None,
            max_pending_swaps_90hz: None,
            max_pending_swaps_120hz: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchedulerAction<T> {
    StartObserving,
    StopObserving,
    ScheduleDeadline(Option<Instant>),
    DrawAndSwap(T),
    FinishFrame(DisplaySchedulerDrawResult),
}

// Source: components/viz/service/display/display_scheduler.h:90-167
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BeginFrameDeadlineMode {
    kImmediate,
    kRegular,
    kLate,
    kNone,
}

pub struct DisplayScheduler<T> {
    pending: Option<ScheduledFrame<T>>,
    last_undrawn: Option<ScheduledFrame<T>>,
    generation: u64,
    visible_: bool,
    output_surface_lost_: bool,
    root_frame_missing_: bool,
    expecting_resize_damage_: bool,
    has_damage_due_to_interaction_: bool,
    needs_draw_: bool,
    has_pending_surfaces_: bool,
    pending_swaps_: usize,
    pending_swap_params_: PendingSwapParams,
    wait_for_all_surfaces_before_draw_: bool,
    allow_multiple_swaps_per_vsync_: bool,
    inside_deadline_: bool,
    inside_surface_damaged_: bool,
    observing_begin_frames_: bool,
    scheduled_deadline_: Option<Instant>,
    next_swap_id_: u32,
}

// Source: components/viz/service/display/display_scheduler.h:168-222
impl<T> Default for DisplayScheduler<T> {
    fn default() -> Self {
        Self::new(PendingSwapParams::default(), false)
    }
}

impl<T> DisplayScheduler<T> {
    pub fn new(params: PendingSwapParams, wait_for_all_surfaces_before_draw: bool) -> Self {
        Self {
            pending: None,
            last_undrawn: None,
            generation: 0,
            visible_: true,
            output_surface_lost_: false,
            root_frame_missing_: false,
            expecting_resize_damage_: false,
            has_damage_due_to_interaction_: false,
            needs_draw_: false,
            has_pending_surfaces_: false,
            pending_swaps_: 0,
            pending_swap_params_: params,
            wait_for_all_surfaces_before_draw_: wait_for_all_surfaces_before_draw,
            allow_multiple_swaps_per_vsync_: false,
            inside_deadline_: false,
            inside_surface_damaged_: false,
            observing_begin_frames_: false,
            scheduled_deadline_: None,
            next_swap_id_: 1,
        }
    }
}

// Source: components/viz/service/display/display_scheduler.cc:51-150
impl<T: Copy> DisplayScheduler<T> {
    pub fn BeginFrame(&mut self, payload: T, timing: FrameTiming) -> Option<ScheduledFrame<T>> {
        let previous = if self.inside_deadline_ {
            let pending = self.pending;
            self.OnBeginFrameDeadline(timing.frame_time);
            pending
        } else {
            None
        };
        self.generation = self.generation.wrapping_add(1);
        self.inside_deadline_ = true;
        self.needs_draw_ = true;
        self.pending = Some(ScheduledFrame {
            payload,
            timing,
            deadlines: timing.Deadlines(),
            prepared: false,
            generation: self.generation,
            candidate_produced: false,
            retry_pending: false,
            retry_consumed: false,
        });
        self.UpdateHasPendingSurfaces(self.has_pending_surfaces_);
        self.ScheduleBeginFrameDeadline(timing.frame_time);
        previous
    }

    pub fn PendingFrame(&self) -> Option<ScheduledFrame<T>> {
        self.pending
    }

    pub fn PendingPayload(&self) -> Option<T> {
        self.pending.map(|frame| frame.payload)
    }

    // Source: components/viz/service/display/display_scheduler.cc:151-240
    pub fn SetVisible(&mut self, visible: bool) {
        if self.visible_ == visible {
            return;
        }
        self.visible_ = visible;
        if !visible {
            self.last_undrawn = None;
        }
        self.MaybeStartObservingBeginFrames();
        self.RescheduleFromPending();
    }

    pub fn OnRootFrameMissing(&mut self, missing: bool) {
        self.root_frame_missing_ = missing;
        self.MaybeStartObservingBeginFrames();
        self.RescheduleFromPending();
    }

    pub fn OnDisplayDamaged(&mut self, now: Instant) {
        self.inside_surface_damaged_ = true;
        self.NotifyVisualDamage(now);
        self.MaybeStartObservingBeginFrames();
        self.inside_surface_damaged_ = false;
        self.ScheduleBeginFrameDeadline(now);
    }

    pub fn OnPendingSurfacesChanged(&mut self, has_pending: bool) {
        if self.UpdateHasPendingSurfaces(has_pending) {
            self.RescheduleFromPending();
        }
    }

    pub fn UpdateHasPendingSurfaces(&mut self, has_pending: bool) -> bool {
        if !self.inside_deadline_ {
            return false;
        }
        let changed = self.has_pending_surfaces_ != has_pending;
        self.has_pending_surfaces_ = has_pending;
        changed
    }

    pub fn OutputSurfaceLost(&mut self) {
        self.output_surface_lost_ = true;
        self.RescheduleFromPending();
    }

    pub fn RestoreOutputSurface(&mut self) {
        self.output_surface_lost_ = false;
        self.MaybeStartObservingBeginFrames();
        self.RescheduleFromPending();
    }

    // Source: components/viz/service/display/display_scheduler.cc:241-330
    // ADPF sessions and presentation feedback are external services. These
    // methods keep the state transitions the core needs from those callbacks.
    pub fn NotifyMinSupportedVsyncInterval(&mut self, _interval: Duration) {}

    pub fn SetExpectingResizeDamage(&mut self, expecting: bool) {
        self.expecting_resize_damage_ = expecting;
        self.RescheduleFromPending();
    }

    pub fn SetDamageDueToInteraction(&mut self, interaction: bool) {
        self.has_damage_due_to_interaction_ = interaction;
        self.RescheduleFromPending();
    }

    // Source: components/viz/service/display/display_scheduler.cc:331-412
    pub fn CanDrawAndSwap(&self) -> bool {
        self.ShouldDraw()
            && self.pending.as_ref().is_none_or(|frame| {
                self.pending_swaps_ < self.MaxPendingSwapsForRefreshRate(frame.timing.interval)
            })
    }

    pub fn MarkPrepared(&mut self, attempted: ScheduledFrame<T>, produced: bool) {
        let Some(frame) = &mut self.pending else {
            return;
        };
        if frame.generation != attempted.generation {
            return;
        }
        frame.retry_consumed |= frame.prepared;
        frame.prepared = true;
        frame.retry_pending = false;
        frame.candidate_produced |= produced;
    }

    pub fn NeedsPrepare(&self, now: Instant) -> bool {
        self.pending.is_some_and(|frame| {
            self.CanDrawAndSwap()
                && ((!frame.prepared && now >= frame.deadlines.compositor_prepare)
                    || (frame.retry_pending && now < frame.deadlines.draw_and_swap))
        })
    }

    pub fn NotifyVisualDamage(&mut self, now: Instant) {
        self.needs_draw_ = true;
        if let Some(frame) = &mut self.pending {
            if frame.prepared
                && !frame.candidate_produced
                && !frame.retry_consumed
                && now < frame.deadlines.draw_and_swap
            {
                frame.retry_pending = true;
            }
        }
    }

    // Source: components/viz/service/display/display_scheduler.cc:413-485
    pub fn OnBeginFrameContinuation(&mut self, payload: T, timing: FrameTiming) -> Option<T> {
        let previous = if self.inside_deadline_ {
            self.OnBeginFrameDeadline(timing.frame_time)
        } else {
            None
        };
        self.BeginFrame(payload, timing);
        previous
    }

    // Source: components/viz/service/display/display_scheduler.cc:486-569
    pub fn MaxPendingSwapsForRefreshRate(&self, interval: Duration) -> usize {
        const HZ_72: Duration = Duration::from_micros(14_000);
        const HZ_90: Duration = Duration::from_micros(11_500);
        const HZ_120: Duration = Duration::from_micros(8_500);
        if interval < HZ_120 {
            self.pending_swap_params_
                .max_pending_swaps_120hz
                .unwrap_or(self.pending_swap_params_.max_pending_swaps)
        } else if interval < HZ_90 {
            self.pending_swap_params_
                .max_pending_swaps_90hz
                .unwrap_or(self.pending_swap_params_.max_pending_swaps)
        } else if interval < HZ_72 {
            self.pending_swap_params_
                .max_pending_swaps_72hz
                .unwrap_or(self.pending_swap_params_.max_pending_swaps)
        } else {
            self.pending_swap_params_.max_pending_swaps
        }
        .max(1)
    }

    pub fn MaxPendingSwapsForDeadline(&self, present_delta: Duration, interval: Duration) -> usize {
        if interval.is_zero() {
            return 1;
        }
        ((present_delta.as_nanos() + interval.as_nanos() * 4 / 5) / interval.as_nanos()).max(1)
            as usize
    }

    pub fn SetNeedsOneBeginFrame(&mut self, needs_draw: bool) {
        self.StartObservingBeginFrames();
        self.needs_draw_ |= needs_draw;
    }

    // Source: components/viz/service/display/display_scheduler.cc:570-668
    pub fn MaybeStartObservingBeginFrames(&mut self) {
        if self.ShouldDraw() {
            self.StartObservingBeginFrames();
        }
    }

    pub fn StartObservingBeginFrames(&mut self) {
        self.observing_begin_frames_ = true;
    }

    pub fn StopObservingBeginFrames(&mut self) {
        self.observing_begin_frames_ = false;
    }

    pub fn IsObservingBeginFrames(&self) -> bool {
        self.observing_begin_frames_
    }

    pub fn ShouldDraw(&self) -> bool {
        self.needs_draw_ && !self.output_surface_lost_ && self.visible_ && !self.root_frame_missing_
    }

    pub fn CanDrawForPreviousFrame(&self, now: Instant) -> bool {
        let Some(frame) = self.last_undrawn else {
            return false;
        };
        !self.inside_deadline_
            && self.pending_swaps_ < self.MaxPendingSwapsForRefreshRate(frame.timing.interval)
            && now < frame.timing.source_deadline
    }

    pub fn ForceImmediateSwapForPreviousFrame(&mut self, now: Instant) -> Option<T> {
        if !self.CanDrawForPreviousFrame(now) {
            return None;
        }
        let frame = self.last_undrawn.take()?;
        self.needs_draw_ = false;
        Some(frame.payload)
    }

    // Source: components/viz/service/display/display_scheduler.cc:669-771
    fn DesiredBeginFrameDeadlineTime(
        mode: BeginFrameDeadlineMode,
        frame: ScheduledFrame<T>,
    ) -> Option<Instant> {
        match mode {
            BeginFrameDeadlineMode::kImmediate => Some(frame.timing.frame_time),
            BeginFrameDeadlineMode::kRegular => Some(frame.deadlines.draw_and_swap),
            BeginFrameDeadlineMode::kLate => Some(frame.timing.frame_time + frame.timing.interval),
            BeginFrameDeadlineMode::kNone => None,
        }
    }

    fn AdjustedBeginFrameDeadlineMode(&self) -> BeginFrameDeadlineMode {
        let mode = self.DesiredBeginFrameDeadlineMode();
        if self.wait_for_all_surfaces_before_draw_
            && matches!(
                mode,
                BeginFrameDeadlineMode::kRegular | BeginFrameDeadlineMode::kLate
            )
        {
            BeginFrameDeadlineMode::kNone
        } else {
            mode
        }
    }

    fn DesiredBeginFrameDeadlineMode(&self) -> BeginFrameDeadlineMode {
        if self.output_surface_lost_ {
            return BeginFrameDeadlineMode::kImmediate;
        }
        let max_swaps = self.pending.as_ref().map_or(1, |frame| {
            self.MaxPendingSwapsForRefreshRate(frame.timing.interval)
        });
        if self.pending_swaps_ >= max_swaps || self.root_frame_missing_ {
            return BeginFrameDeadlineMode::kLate;
        }
        let wait_for_surfaces = self.has_pending_surfaces_ && !self.has_damage_due_to_interaction_;
        let all_ready = !wait_for_surfaces && !self.expecting_resize_damage_;
        if all_ready && (self.needs_draw_ || self.wait_for_all_surfaces_before_draw_) {
            return BeginFrameDeadlineMode::kImmediate;
        }
        if !self.needs_draw_ || self.expecting_resize_damage_ {
            return BeginFrameDeadlineMode::kLate;
        }
        BeginFrameDeadlineMode::kRegular
    }

    // Source: components/viz/service/display/display_scheduler.cc:772-855
    pub fn ScheduleBeginFrameDeadline(&mut self, now: Instant) -> Option<Instant> {
        if !self.inside_deadline_ {
            self.scheduled_deadline_ = None;
            return None;
        }
        let frame = self.pending?;
        let desired =
            Self::DesiredBeginFrameDeadlineTime(self.AdjustedBeginFrameDeadlineMode(), frame);
        self.scheduled_deadline_ = desired.map(|deadline| deadline.max(now));
        self.scheduled_deadline_
    }

    fn AttemptDrawAndSwap(&mut self) -> Option<T> {
        if self.ShouldDraw() {
            let frame = self.pending?;
            if self.pending_swaps_ < self.MaxPendingSwapsForRefreshRate(frame.timing.interval) {
                self.inside_deadline_ = false;
                self.scheduled_deadline_ = None;
                self.needs_draw_ = false;
                self.last_undrawn = None;
                return self.pending.take().map(|value| value.payload);
            }
        } else {
            self.expecting_resize_damage_ = false;
            self.StopObservingBeginFrames();
        }
        None
    }

    pub fn OnBeginFrameDeadline(&mut self, _now: Instant) -> Option<T> {
        let drawn = self.AttemptDrawAndSwap();
        if drawn.is_none() && self.allow_multiple_swaps_per_vsync_ {
            self.last_undrawn = self.pending;
        } else if drawn.is_none() {
            self.last_undrawn = None;
        }
        self.inside_deadline_ = false;
        self.scheduled_deadline_ = None;
        drawn
    }

    pub fn TakeIfDrawDue(&mut self, now: Instant) -> Option<T> {
        if self
            .scheduled_deadline_
            .is_none_or(|deadline| now < deadline)
        {
            return None;
        }
        self.OnBeginFrameDeadline(now)
    }

    pub fn NextDeadline(&self) -> Option<Instant> {
        self.scheduled_deadline_
    }

    fn RescheduleFromPending(&mut self) {
        if let Some(frame) = self.pending {
            self.ScheduleBeginFrameDeadline(frame.timing.frame_time);
        }
    }

    // Source: components/viz/service/display/display_scheduler.cc:856-897
    pub fn DidFinishFrame(
        &mut self,
        result: DisplaySchedulerDrawResult,
    ) -> DisplaySchedulerDrawResult {
        self.has_damage_due_to_interaction_ = false;
        result
    }

    pub fn DidSwapBuffers(&mut self) -> u32 {
        self.pending_swaps_ = self.pending_swaps_.saturating_add(1);
        let id = self.next_swap_id_;
        self.next_swap_id_ = self.next_swap_id_.wrapping_add(1);
        id
    }

    pub fn DidReceiveSwapBuffersAck(&mut self) {
        self.pending_swaps_ = self.pending_swaps_.saturating_sub(1);
        self.RescheduleFromPending();
    }
}
