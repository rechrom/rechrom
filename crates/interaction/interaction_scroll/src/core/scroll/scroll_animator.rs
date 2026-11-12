// Copyright 2011 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
//
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/scroll/scroll_animator_base.h:45-107
// Source: third_party/blink/renderer/core/scroll/scroll_animator_base.cc:39-95
// Source: third_party/blink/renderer/core/scroll/scroll_animator.h:46-161
// Source: third_party/blink/renderer/core/scroll/scroll_animator.cc:49-220,237-277,303-321,385-417
//
// The embedding supplies clock values and accepts typed scheduling commands.
// This owner retains Blink's curve, target, run state and one completion. It
// owns no Page, LayoutEngine, compositor, timer or copied scroll geometry.

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
    sync::Arc,
};

use foundation::{gfx::Vector2dF, CompositorElementId};
use scroll_animation::{
    animation_id_provider::AnimationIdProvider,
    scroll_offset_animation_curve::{ScrollCurveConfig, ScrollOffsetAnimationCurve, ScrollType},
    scroll_offset_animation_curve_factory::ScrollOffsetAnimationCurveFactory,
    PointF, TimeTicks,
};

use crate::{
    cc::trees::scroll_source_type::ScrollSourceType,
    core::scroll::scrollable_area::{ScrollCallback, ScrollCompletionMode},
};
use interaction_input::ui::events::types::scroll_types::ScrollGranularity;

// Source: scroll_animator_compositor_coordinator.h:46-88. The adapter does not
// recreate compositor internals; states collapse only where the source has the
// same observable target/completion behavior at this package boundary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UserScrollAnimatorRunState {
    #[default]
    Idle,
    WaitingToSchedule,
    RunningOnMainThread,
    RunningOnCompositor,
    WaitingToCancel,
    PostAnimationCleanup,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UserScrollAnimatorSnapshot {
    pub enabled: bool,
    pub has_running_animation: bool,
    /// None has Chromium's `DesiredTargetOffset() == CurrentOffset()` meaning.
    pub desired_target_offset: Option<Vector2dF>,
    pub run_state: UserScrollAnimatorRunState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UserScrollAnimationToken {
    pub animation: i32,
    pub group: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserScrollAnimationRoute {
    MainThread,
    Compositor,
}

pub struct UserScrollAnimationStart {
    pub area_identity: usize,
    pub node_id: Option<u64>,
    pub scroll_element_id: CompositorElementId,
    pub source_type: ScrollSourceType,
    pub granularity: ScrollGranularity,
    pub token: UserScrollAnimationToken,
    pub start_time: TimeTicks,
    pub target_offset: Vector2dF,
    pub replaces_running_animation: bool,
    /// The receiver gets a clone; this owner keeps the canonical curve for
    /// retargeting and main-thread ticks.
    pub curve: ScrollOffsetAnimationCurve,
}

#[derive(Clone, Copy, Debug)]
pub struct UserScrollAnimatedOffset {
    pub area_identity: usize,
    pub node_id: Option<u64>,
    pub scroll_element_id: CompositorElementId,
    pub source_type: ScrollSourceType,
    pub token: UserScrollAnimationToken,
    pub offset: Vector2dF,
    pub finished: bool,
}

/// The existing Page/compositor owners implement this typed scheduling edge.
/// Acceptance is synchronous, matching `RegisterAndScheduleAnimation()`.
pub trait UserScrollAnimatorSink {
    fn StartOrUpdate(&self, request: UserScrollAnimationStart) -> Option<UserScrollAnimationRoute>;
    fn Cancel(&self, area_identity: usize, token: UserScrollAnimationToken);
    fn ApplyOffset(&self, mutation: UserScrollAnimatedOffset);
    fn RequestAnimationFrame(&self, area_identity: usize);
}

pub struct UserScrollAnimationRequest {
    pub node_id: Option<u64>,
    pub scroll_element_id: CompositorElementId,
    pub source_type: ScrollSourceType,
    pub granularity: ScrollGranularity,
    pub current_offset: Vector2dF,
    pub target_offset: Vector2dF,
}

struct RunningAnimation {
    node_id: Option<u64>,
    scroll_element_id: CompositorElementId,
    source_type: ScrollSourceType,
    granularity: ScrollGranularity,
    token: UserScrollAnimationToken,
    route: UserScrollAnimationRoute,
    start_time: TimeTicks,
    target_offset: Vector2dF,
    curve: ScrollOffsetAnimationCurve,
    completion: Option<ScrollCallback>,
}

#[derive(Default)]
struct AnimatorState {
    run_state: UserScrollAnimatorRunState,
    running: Option<RunningAnimation>,
}

/// One lazily-created `ScrollAnimator` for one native ScrollableArea identity.
pub struct RetainedUserScrollAnimator {
    area_identity: usize,
    enabled: Cell<bool>,
    sink: Rc<dyn UserScrollAnimatorSink>,
    curve_config: Arc<ScrollCurveConfig>,
    state: RefCell<AnimatorState>,
}

impl RetainedUserScrollAnimator {
    pub fn new(
        area_identity: usize,
        enabled: bool,
        sink: Rc<dyn UserScrollAnimatorSink>,
        curve_config: Arc<ScrollCurveConfig>,
    ) -> Self {
        Self {
            area_identity,
            enabled: Cell::new(enabled),
            sink,
            curve_config,
            state: RefCell::new(AnimatorState::default()),
        }
    }

    pub fn AreaIdentity(&self) -> usize {
        self.area_identity
    }

    pub fn SetEnabled(&self, enabled: bool) {
        if self.enabled.replace(enabled) && !enabled {
            self.CancelAnimation();
        }
    }

    // Source: scroll_animator.cc:68-82; coordinator.h:117-121.
    pub fn Snapshot(&self) -> UserScrollAnimatorSnapshot {
        let state = self.state.borrow();
        let running = state.running.as_ref();
        UserScrollAnimatorSnapshot {
            enabled: self.enabled.get(),
            has_running_animation: running.is_some()
                && state.run_state != UserScrollAnimatorRunState::PostAnimationCleanup,
            desired_target_offset: running.map(|running| running.target_offset),
            run_state: state.run_state,
        }
    }

    /// `ScrollAnimator::UserScroll`'s animated branch. Layout has already
    /// clamped the target and selected the animation branch; this method owns
    /// curve retargeting plus completion replacement.
    // Source: scroll_animator.cc:93-160,163-220,303-315.
    pub fn StartOrUpdate(
        &self,
        request: UserScrollAnimationRequest,
        now: TimeTicks,
        completion: ScrollCallback,
    ) -> Result<UserScrollAnimationToken, ScrollCallback> {
        if !self.enabled.get() || request.granularity == ScrollGranularity::kScrollByPrecisePixel {
            return Err(completion);
        }

        let (mut curve, start_time, replaces_running) = {
            let state = self.state.borrow();
            if let Some(running) = state.running.as_ref() {
                let mut curve = running.curve.Clone();
                curve.UpdateTarget(
                    now - running.start_time,
                    PointF::new(request.target_offset.x(), request.target_offset.y()),
                );
                (curve, running.start_time, true)
            } else {
                let scroll_type = if request.granularity == ScrollGranularity::kScrollByPixel {
                    ScrollType::kMouseWheel
                } else {
                    ScrollType::kKeyboard
                };
                let mut curve = ScrollOffsetAnimationCurveFactory::CreateAnimation(
                    PointF::new(request.target_offset.x(), request.target_offset.y()),
                    scroll_type,
                    self.curve_config.clone(),
                );
                curve.SetInitialValue(
                    PointF::new(request.current_offset.x(), request.current_offset.y()),
                    Default::default(),
                    0.0,
                );
                (curve, now, false)
            }
        };
        let token = UserScrollAnimationToken {
            animation: AnimationIdProvider::NextKeyframeModelId(),
            group: AnimationIdProvider::NextGroupId(),
        };
        let route = self.sink.StartOrUpdate(UserScrollAnimationStart {
            area_identity: self.area_identity,
            node_id: request.node_id,
            scroll_element_id: request.scroll_element_id,
            source_type: request.source_type,
            granularity: request.granularity,
            token,
            start_time,
            target_offset: request.target_offset,
            replaces_running_animation: replaces_running,
            curve: curve.Clone(),
        });
        let Some(route) = route else {
            let previous = self.TakeRunning(ScrollCompletionMode::kFinished, true);
            if let Some(previous) = previous {
                previous(ScrollCompletionMode::kFinished);
            }
            return Err(completion);
        };

        let previous = {
            let mut state = self.state.borrow_mut();
            let previous = state
                .running
                .as_mut()
                .and_then(|running| running.completion.take());
            state.run_state = match route {
                UserScrollAnimationRoute::MainThread => {
                    UserScrollAnimatorRunState::RunningOnMainThread
                }
                UserScrollAnimationRoute::Compositor => {
                    UserScrollAnimatorRunState::RunningOnCompositor
                }
            };
            state.running = Some(RunningAnimation {
                node_id: request.node_id,
                scroll_element_id: request.scroll_element_id,
                source_type: request.source_type,
                granularity: request.granularity,
                token,
                route,
                start_time,
                target_offset: request.target_offset,
                curve,
                completion: Some(completion),
            });
            previous
        };
        // Source cc:135-139: listener reentry occurs only after the new owner
        // state has been installed, and no RefCell borrow crosses the callback.
        if let Some(previous) = previous {
            previous(ScrollCompletionMode::kInterruptedByScroll);
        }
        Ok(token)
    }

    /// Main-thread tick. Compositor-routed animations complete through
    /// `NotifyCompositorAnimationFinished` and are never sampled here.
    // Source: scroll_animator.cc:248-277.
    pub fn TickAnimation(&self, now: TimeTicks) -> bool {
        let mutation = {
            let state = self.state.borrow();
            let Some(running) = state.running.as_ref() else {
                return false;
            };
            if running.route != UserScrollAnimationRoute::MainThread {
                return false;
            }
            let elapsed = now - running.start_time;
            let finished = elapsed > running.curve.Duration();
            let point = if finished {
                running.curve.target_value()
            } else {
                running.curve.GetValue(elapsed)
            };
            UserScrollAnimatedOffset {
                area_identity: self.area_identity,
                node_id: running.node_id,
                scroll_element_id: running.scroll_element_id,
                source_type: running.source_type,
                token: running.token,
                offset: Vector2dF::new(point.x(), point.y()),
                finished,
            }
        };
        self.sink.ApplyOffset(mutation);
        if mutation.finished {
            self.FinishToken(mutation.token, ScrollCompletionMode::kFinished)
        } else {
            self.sink.RequestAnimationFrame(self.area_identity);
            true
        }
    }

    // Source: scroll_animator.cc:385-397.
    pub fn NotifyCompositorAnimationFinished(&self, token: UserScrollAnimationToken) -> bool {
        self.FinishToken(token, ScrollCompletionMode::kFinished)
    }

    pub fn NotifyCompositorAnimationAborted(&self, token: UserScrollAnimationToken) -> bool {
        self.FinishToken(token, ScrollCompletionMode::kFinished)
    }

    // Source: scroll_animator.cc:399-403.
    pub fn CancelAnimation(&self) -> bool {
        let (token, completion) = {
            let mut state = self.state.borrow_mut();
            let Some(mut running) = state.running.take() else {
                state.run_state = UserScrollAnimatorRunState::Idle;
                return false;
            };
            state.run_state = UserScrollAnimatorRunState::WaitingToCancel;
            (running.token, running.completion.take())
        };
        self.sink.Cancel(self.area_identity, token);
        {
            let mut state = self.state.borrow_mut();
            if state.running.is_none()
                && state.run_state == UserScrollAnimatorRunState::WaitingToCancel
            {
                state.run_state = UserScrollAnimatorRunState::Idle;
            }
        }
        if let Some(completion) = completion {
            completion(ScrollCompletionMode::kFinished);
        }
        true
    }

    fn FinishToken(&self, token: UserScrollAnimationToken, mode: ScrollCompletionMode) -> bool {
        let completion = {
            let mut state = self.state.borrow_mut();
            if state
                .running
                .as_ref()
                .is_none_or(|running| running.token != token)
            {
                return false;
            }
            let mut running = state.running.take().expect("checked running animation");
            state.run_state = UserScrollAnimatorRunState::PostAnimationCleanup;
            running.completion.take()
        };
        if let Some(completion) = completion {
            completion(mode);
        }
        {
            let mut state = self.state.borrow_mut();
            // The completion may synchronously start the next animation. Do
            // not let cleanup of the old token overwrite that new run state.
            if state.running.is_none()
                && state.run_state == UserScrollAnimatorRunState::PostAnimationCleanup
            {
                state.run_state = UserScrollAnimatorRunState::Idle;
            }
        }
        true
    }

    fn TakeRunning(
        &self,
        _mode: ScrollCompletionMode,
        cancel_host: bool,
    ) -> Option<ScrollCallback> {
        let (token, completion) = {
            let mut state = self.state.borrow_mut();
            let mut running = state.running.take()?;
            state.run_state = UserScrollAnimatorRunState::Idle;
            (running.token, running.completion.take())
        };
        if cancel_host {
            self.sink.Cancel(self.area_identity, token);
        }
        completion
    }
}

impl Drop for RetainedUserScrollAnimator {
    // Source: scroll_animator.cc:62-66.
    fn drop(&mut self) {
        let completion = self
            .state
            .get_mut()
            .running
            .as_mut()
            .and_then(|running| running.completion.take());
        if let Some(completion) = completion {
            completion(ScrollCompletionMode::kFinished);
        }
    }
}

/// One per Page domain. It preserves the native ScrollableArea identity while
/// each entry remains the lazily-created source animator for that area.
pub struct UserScrollAnimatorRegistry {
    sink: Rc<dyn UserScrollAnimatorSink>,
    curve_config: Arc<ScrollCurveConfig>,
    animators: RefCell<HashMap<usize, Rc<RetainedUserScrollAnimator>>>,
}

impl UserScrollAnimatorRegistry {
    pub fn new(sink: Rc<dyn UserScrollAnimatorSink>, curve_config: Arc<ScrollCurveConfig>) -> Self {
        Self {
            sink,
            curve_config,
            animators: RefCell::new(HashMap::new()),
        }
    }

    pub fn GetOrCreate(
        &self,
        area_identity: usize,
        enabled: bool,
    ) -> Rc<RetainedUserScrollAnimator> {
        if let Some(animator) = self.animators.borrow().get(&area_identity) {
            animator.SetEnabled(enabled);
            return animator.clone();
        }
        let animator = Rc::new(RetainedUserScrollAnimator::new(
            area_identity,
            enabled,
            self.sink.clone(),
            self.curve_config.clone(),
        ));
        self.animators
            .borrow_mut()
            .insert(area_identity, animator.clone());
        animator
    }

    pub fn Remove(&self, area_identity: usize) -> bool {
        self.animators.borrow_mut().remove(&area_identity).is_some()
    }

    pub fn Get(&self, area_identity: usize) -> Option<Rc<RetainedUserScrollAnimator>> {
        self.animators.borrow().get(&area_identity).cloned()
    }

    /// Samples the retained main-thread animators for one BeginFrame. Clone
    /// only the Rc owner list so callbacks may synchronously remove areas.
    // Source: scroll_animator.cc:248-277.
    pub fn TickAnimations(&self, now: TimeTicks) -> bool {
        let animators: Vec<_> = self.animators.borrow().values().cloned().collect();
        animators.into_iter().fold(false, |running, animator| {
            animator.TickAnimation(now) || running
        })
    }

    pub fn Cancel(&self, area_identity: usize) -> bool {
        self.Get(area_identity)
            .is_some_and(|animator| animator.CancelAnimation())
    }
}
