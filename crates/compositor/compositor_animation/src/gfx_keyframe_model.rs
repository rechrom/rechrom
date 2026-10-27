// Translation status: complete
// Source: ui/gfx/animation/keyframe/keyframe_model.h:1-258
// Source: ui/gfx/animation/keyframe/keyframe_model.cc:1-364
use compositor_scheduler::{TimeDelta, TimeTicks};
// Source: ui/gfx/animation/keyframe/keyframe_model.h:38-66
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunState {
    WAITING_FOR_TARGET_AVAILABILITY = 0,
    WAITING_FOR_DELETION,
    STARTING,
    RUNNING,
    PAUSED,
    PAUSED_EXCLUSIVE,
    FINISHED,
    ABORTED,
    ABORTED_BUT_NEEDS_COMPLETION,
}
impl RunState {
    pub const LAST_RUN_STATE: Self = Self::ABORTED_BUT_NEEDS_COMPLETION;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    NORMAL,
    REVERSE,
    ALTERNATE_NORMAL,
    ALTERNATE_REVERSE,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillMode {
    NONE,
    FORWARDS,
    BACKWARDS,
    BOTH,
    AUTO,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    BEFORE,
    ACTIVE,
    AFTER,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitDirection {
    LEFT,
    RIGHT,
}
pub trait AnimationCurve {
    fn Duration(&self) -> TimeDelta;
}
pub trait RetargetCurve<T>: AnimationCurve {
    fn Retarget(&self, time: TimeDelta, value: &T) -> Option<Self>
    where
        Self: Sized;
}
// Source: ui/gfx/animation/keyframe/keyframe_model.h:235-253
pub struct KeyframeModel<C> {
    curve_: Option<C>,
    id_: i32,
    target_property_: i32,
    run_state_: RunState,
    iterations_: f64,
    iteration_start_: f64,
    direction_: Direction,
    playback_rate_: f64,
    fill_mode_: FillMode,
    start_time_: Option<TimeTicks>,
    start_delay_: TimeDelta,
    hold_time_: Option<TimeDelta>,
    auto_fills_on_finish_: bool,
}
impl<C: AnimationCurve> KeyframeModel<C> {
    // Source: ui/gfx/animation/keyframe/keyframe_model.cc:14-29
    pub fn ToString(state: RunState) -> &'static str {
        match state {
            RunState::WAITING_FOR_TARGET_AVAILABILITY => "WAITING_FOR_TARGET_AVAILABILITY",
            RunState::WAITING_FOR_DELETION => "WAITING_FOR_DELETION",
            RunState::STARTING => "STARTING",
            RunState::RUNNING => "RUNNING",
            RunState::PAUSED => "PAUSED",
            RunState::PAUSED_EXCLUSIVE => "PAUSED_EXCLUSIVE",
            RunState::FINISHED => "FINISHED",
            RunState::ABORTED => "ABORTED",
            RunState::ABORTED_BUT_NEEDS_COMPLETION => "ABORTED_BUT_NEEDS_COMPLETION",
        }
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.cc:31-33
    pub fn IsPaused(state: RunState) -> bool {
        matches!(state, RunState::PAUSED | RunState::PAUSED_EXCLUSIVE)
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.cc:35-56
    pub fn Create(curve: Option<C>, id: i32, target: i32) -> Self {
        Self {
            curve_: curve,
            id_: id,
            target_property_: target,
            run_state_: RunState::WAITING_FOR_TARGET_AVAILABILITY,
            iterations_: 1.0,
            iteration_start_: 0.0,
            direction_: Direction::NORMAL,
            playback_rate_: 1.0,
            fill_mode_: FillMode::BOTH,
            start_time_: None,
            start_delay_: TimeDelta::default(),
            hold_time_: None,
            auto_fills_on_finish_: false,
        }
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.cc:65-75
    pub fn TargetProperty(&self) -> i32 {
        self.target_property_
    }
    pub fn SetRunState(&mut self, state: RunState) {
        self.run_state_ = state;
        if matches!(state, RunState::STARTING | RunState::RUNNING) && self.auto_fills_on_finish_ {
            self.EnsureFillsWhenFinished();
        }
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.cc:77-82
    pub fn Pause(&mut self, hold: TimeDelta, state: RunState) {
        self.PauseWith(hold, state, |base, state| base.SetRunState(state));
    }
    pub fn PauseWith(
        &mut self,
        hold: TimeDelta,
        state: RunState,
        dispatch: impl FnOnce(&mut Self, RunState),
    ) {
        assert!(Self::IsPaused(state));
        self.start_time_ = None;
        self.set_hold_time(Some(hold));
        dispatch(self, state);
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.cc:84-91
    pub fn Reverse(&mut self, now: TimeTicks, deferred: bool) {
        self.ReverseWith(now, deferred, |base, state| base.SetRunState(state));
    }
    pub fn ReverseWith(
        &mut self,
        now: TimeTicks,
        deferred: bool,
        dispatch: impl FnOnce(&mut Self, RunState),
    ) {
        let current = self.CalculateCurrentTime(now, -self.playback_rate_, deferred);
        self.set_hold_time(None);
        self.set_playback_rate(-self.playback_rate_);
        self.set_start_time(now - current / self.playback_rate_);
        dispatch(self, RunState::RUNNING);
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.cc:93-97
    pub fn UnpauseForTesting(&mut self, now: TimeTicks) {
        self.UnpauseForTestingWith(now, |base, state| base.SetRunState(state));
    }
    pub fn UnpauseForTestingWith(
        &mut self,
        now: TimeTicks,
        dispatch: impl FnOnce(&mut Self, RunState),
    ) {
        self.set_start_time(now - self.hold_time_.unwrap() / self.playback_rate_);
        self.set_hold_time(None);
        dispatch(self, RunState::RUNNING);
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.cc:99-127
    pub fn CalculatePhaseForTesting(&self, local: TimeDelta) -> Phase {
        self.CalculatePhase(local)
    }
    fn CalculatePhase(&self, local: TimeDelta) -> Phase {
        if local < self.start_delay_
            || (local == self.start_delay_
                && (self.playback_rate_ < 0.0
                    || (self.playback_rate_ > 0.0
                        && self.run_state_ == RunState::PAUSED_EXCLUSIVE)))
        {
            return Phase::BEFORE;
        }
        let mut boundary = TimeDelta::Max();
        if self.iterations_.is_finite() {
            let active = self.curve().unwrap().Duration() * self.iterations_;
            boundary = (self.start_delay_ + active).max(TimeDelta::default());
        }
        if local > boundary
            || (local == boundary
                && (self.playback_rate_ > 0.0
                    || (self.playback_rate_ < 0.0
                        && self.run_state_ == RunState::PAUSED_EXCLUSIVE)))
        {
            return Phase::AFTER;
        }
        Phase::ACTIVE
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.cc:129-159
    pub fn CalculateActiveTime(&self, now: TimeTicks, deferred: bool) -> Option<TimeDelta> {
        let local = self.ConvertMonotonicTimeToLocalTime(now, deferred);
        self.CalculateActiveTimeForPhase(local, self.CalculatePhase(local))
    }
    fn CalculateActiveTimeForPhase(&self, local: TimeDelta, phase: Phase) -> Option<TimeDelta> {
        debug_assert_ne!(self.playback_rate_, 0.0);
        match phase {
            Phase::BEFORE => {
                if matches!(self.fill_mode_, FillMode::BACKWARDS | FillMode::BOTH) {
                    Some((local - self.start_delay_).max(TimeDelta::default()))
                } else {
                    None
                }
            }
            Phase::ACTIVE => Some(local - self.start_delay_),
            Phase::AFTER => {
                if matches!(self.fill_mode_, FillMode::FORWARDS | FillMode::BOTH) {
                    debug_assert_ne!(self.iterations_, f64::INFINITY);
                    let active = self.curve().unwrap().Duration() * self.iterations_;
                    Some(
                        (local - self.start_delay_)
                            .min(active)
                            .max(TimeDelta::default()),
                    )
                } else {
                    None
                }
            }
        }
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.cc:161-185
    pub fn IsFinishedAt(&self, now: TimeTicks, deferred: bool) -> bool {
        if self.is_finished() {
            return true;
        }
        if deferred {
            return false;
        }
        if self.playback_rate_ == 0.0 {
            return false;
        }
        if self.run_state_ != RunState::RUNNING || !self.iterations_.is_finite() {
            return false;
        }
        self.IsFinishedAtMonotonicTime(now, deferred)
    }
    pub fn IsFinishedAtMonotonicTime(&self, now: TimeTicks, deferred: bool) -> bool {
        let local = self.ConvertMonotonicTimeToLocalTime(now, deferred);
        let active = local - self.start_delay_;
        if self.playback_rate_ < 0.0 {
            active <= TimeDelta::default()
        } else {
            active >= self.curve().unwrap().Duration() * self.iterations_
        }
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.cc:187-193
    pub fn HasActiveTime(&self, now: TimeTicks, deferred: bool) -> bool {
        self.CalculateActiveTime(now, deferred).is_some()
    }
    pub fn StartShouldBeDeferred(&self) -> bool {
        false
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.cc:195-271
    pub fn TrimTimeToCurrentIteration(
        &self,
        now: TimeTicks,
        deferred: bool,
        limit: Option<&mut LimitDirection>,
    ) -> TimeDelta {
        debug_assert_ne!(self.playback_rate_, 0.0);
        debug_assert!(self.iteration_start_ >= 0.0);
        debug_assert!(self.HasActiveTime(now, deferred));
        let local = self.ConvertMonotonicTimeToLocalTime(now, deferred);
        let phase = self.CalculatePhase(local);
        let mut active = self.CalculateActiveTimeForPhase(local, phase).unwrap();
        let start_offset = self.curve().unwrap().Duration() * self.iteration_start_;
        if let Some(limit) = limit {
            *limit = if phase == Phase::BEFORE {
                LimitDirection::LEFT
            } else {
                LimitDirection::RIGHT
            };
        }
        debug_assert!(!active.is_negative());
        if self.iterations_ == 0.0 {
            return TimeDelta::default();
        }
        if self.curve().unwrap().Duration() <= TimeDelta::default() {
            return TimeDelta::default();
        }
        let repeated = if self.iterations_.is_finite() {
            self.curve().unwrap().Duration() * self.iterations_
        } else {
            TimeDelta::Max()
        };
        active += start_offset;
        let mut iteration_time;
        let defined = start_offset != active
            || !(start_offset == TimeDelta::Max() || start_offset == TimeDelta::Min());
        if defined
            && active - start_offset == repeated
            && (self.iterations_ + self.iteration_start_) % 1.0 == 0.0
        {
            iteration_time = self.curve().unwrap().Duration();
        } else {
            iteration_time = TimeDelta::Microseconds(
                active.InMicroseconds() % self.curve().unwrap().Duration().InMicroseconds(),
            );
        }
        let iteration: i32 = if active <= TimeDelta::default() {
            0
        } else if iteration_time == self.curve().unwrap().Duration() {
            (self.iteration_start_ + self.iterations_ - 1.0).ceil() as i32
        } else {
            (active / self.curve().unwrap().Duration()).floor() as i32
        };
        let reverse = self.direction_ == Direction::REVERSE
            || (self.direction_ == Direction::ALTERNATE_NORMAL && iteration % 2 == 1)
            || (self.direction_ == Direction::ALTERNATE_REVERSE && iteration % 2 == 0);
        if reverse {
            iteration_time = self.curve().unwrap().Duration() - iteration_time;
        }
        iteration_time
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.cc:273-292
    fn ConvertMonotonicTimeToLocalTime(&self, now: TimeTicks, deferred: bool) -> TimeDelta {
        if let Some(hold) = self.hold_time_ {
            return hold;
        }
        if self.run_state_ == RunState::STARTING && !self.has_set_start_time() || deferred {
            return TimeDelta::default();
        }
        (now - self.start_time_.unwrap_or(now)) * self.playback_rate_
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.cc:294-320
    pub fn CalculateCurrentTime(&self, now: TimeTicks, rate: f64, deferred: bool) -> TimeDelta {
        if let Some(hold) = self.hold_time_ {
            return hold;
        }
        if self.start_time_.is_some() {
            return self
                .ConvertMonotonicTimeToLocalTime(now, deferred)
                .min(self.CalculateEndTime())
                .max(TimeDelta::default());
        }
        self.CalculateInitialHoldTime(rate)
    }
    pub fn CalculateInitialHoldTime(&self, rate: f64) -> TimeDelta {
        if rate > 0.0 {
            self.start_delay_
        } else {
            self.CalculateEndTime()
        }
    }
    pub fn CalculateEndTime(&self) -> TimeDelta {
        self.curve().unwrap().Duration() * self.iterations_ + self.start_delay_
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.cc:322-362
    fn EnsureFillsWhenFinished(&mut self) {
        if self.playback_rate_ >= 0.0 {
            match self.fill_mode_ {
                FillMode::NONE => self.set_fill_mode(FillMode::FORWARDS),
                FillMode::BACKWARDS => self.set_fill_mode(FillMode::BOTH),
                FillMode::FORWARDS | FillMode::BOTH => {}
                FillMode::AUTO => unreachable!("AUTO fill must be resolved by producer"),
            }
        } else {
            match self.fill_mode_ {
                FillMode::NONE => self.set_fill_mode(FillMode::BACKWARDS),
                FillMode::FORWARDS => self.set_fill_mode(FillMode::BOTH),
                FillMode::BACKWARDS | FillMode::BOTH => {}
                FillMode::AUTO => unreachable!("AUTO fill must be resolved by producer"),
            }
        }
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.h:70-142
    pub fn id(&self) -> i32 {
        self.id_
    }
    pub fn run_state(&self) -> RunState {
        self.run_state_
    }
    pub fn start_time(&self) -> TimeTicks {
        self.start_time_.unwrap_or_default()
    }
    pub fn set_start_time(&mut self, t: TimeTicks) {
        self.start_time_ = Some(t);
    }
    pub fn has_set_start_time(&self) -> bool {
        self.start_time_.is_some()
    }
    pub fn ResetStartTimeForTesting(&mut self) {
        self.start_time_ = None;
    }
    pub fn start_delay(&self) -> TimeDelta {
        self.start_delay_
    }
    pub fn set_start_delay(&mut self, d: TimeDelta) {
        self.start_delay_ = d;
    }
    pub fn direction(&self) -> Direction {
        self.direction_
    }
    pub fn set_direction(&mut self, d: Direction) {
        self.direction_ = d;
    }
    pub fn fill_mode(&self) -> FillMode {
        self.fill_mode_
    }
    pub fn set_fill_mode(&mut self, m: FillMode) {
        self.fill_mode_ = m;
    }
    pub fn playback_rate(&self) -> f64 {
        self.playback_rate_
    }
    pub fn set_playback_rate(&mut self, r: f64) {
        self.playback_rate_ = r;
    }
    pub fn hold_time(&self) -> Option<TimeDelta> {
        self.hold_time_
    }
    pub fn set_hold_time(&mut self, t: Option<TimeDelta>) {
        self.hold_time_ = t;
    }
    pub fn auto_fills_on_finish(&self) -> bool {
        self.auto_fills_on_finish_
    }
    pub fn set_auto_fills_on_finish(&mut self, v: bool) {
        self.auto_fills_on_finish_ = v;
    }
    pub fn iterations(&self) -> f64 {
        self.iterations_
    }
    pub fn set_iterations(&mut self, n: f64) {
        self.iterations_ = n;
    }
    pub fn iteration_start(&self) -> f64 {
        self.iteration_start_
    }
    pub fn set_iteration_start(&mut self, n: f64) {
        self.iteration_start_ = n;
    }
    pub fn curve(&self) -> Option<&C> {
        self.curve_.as_ref()
    }
    pub fn curve_mut(&mut self) -> Option<&mut C> {
        self.curve_.as_mut()
    }
    pub fn is_finished(&self) -> bool {
        matches!(
            self.run_state_,
            RunState::FINISHED | RunState::ABORTED | RunState::WAITING_FOR_DELETION
        )
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.h:147-160
    pub fn Retarget<T>(&mut self, now: TimeTicks, _property: i32, value: &T, deferred: bool)
    where
        C: RetargetCurve<T>,
    {
        if self.curve_.is_none() {
            return;
        }
        let now_delta = self.TrimTimeToCurrentIteration(now, deferred, None);
        debug_assert_eq!(self.CalculatePhase(now_delta), Phase::ACTIVE);
        if let Some(curve) = self.curve().unwrap().Retarget(now_delta, value) {
            self.curve_ = Some(curve);
        }
    }
    // Source: ui/gfx/animation/keyframe/keyframe_model.h:198
    pub(crate) fn ForceRunState(&mut self, state: RunState) {
        self.run_state_ = state;
    }
}
// Source: ui/gfx/animation/keyframe/keyframe_model.cc:58-63
impl<C> Drop for KeyframeModel<C> {
    fn drop(&mut self) {
        if matches!(
            self.run_state_,
            RunState::RUNNING | RunState::PAUSED | RunState::PAUSED_EXCLUSIVE
        ) {
            self.run_state_ = RunState::ABORTED;
        }
    }
}
