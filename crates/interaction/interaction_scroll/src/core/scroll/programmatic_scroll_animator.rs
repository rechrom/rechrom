//! Chromium programmatic_scroll_animator.{h,cc}:27-226 and the corresponding
//! ScrollAnimatorCompositorCoordinator state transitions. Each instance belongs
//! to one native ScrollableArea. The embedding supplies only immutable request,
//! clock and completion values and a typed scheduling/output sink.
use super::scrollable_area::{ScrollCallback, ScrollCompletionMode};
use crate::cc::trees::scroll_source_type::ScrollSourceType;
use foundation::{gfx::Vector2dF, CompositorElementId};
use scroll_animation::{
    animation_id_provider::AnimationIdProvider,
    scroll_offset_animation_curve::{ScrollCurveConfig, ScrollOffsetAnimationCurve, ScrollType},
    scroll_offset_animation_curve_factory::ScrollOffsetAnimationCurveFactory,
    PointF, TimeDelta, TimeTicks,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProgrammaticScrollAnimationToken {
    pub model: i32,
    pub group: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgrammaticScrollRunState {
    Idle,
    WaitingToSendToCompositor,
    RunningOnMainThread,
    RunningOnCompositor,
    PostAnimationCleanup,
}
#[derive(Clone, Copy)]
pub struct ProgrammaticScrollRequest {
    pub node_id: u64,
    pub element: CompositorElementId,
    pub current_offset: Vector2dF,
    pub target_offset: Vector2dF,
    pub source_type: ScrollSourceType,
}
pub struct ProgrammaticCompositorAnimation {
    pub request: ProgrammaticScrollRequest,
    pub token: ProgrammaticScrollAnimationToken,
    pub curve: ScrollOffsetAnimationCurve,
}
#[derive(Clone, Copy)]
pub struct ProgrammaticScrollOffset {
    pub node_id: u64,
    pub element: CompositorElementId,
    pub offset: Vector2dF,
    pub source_type: ScrollSourceType,
}
pub trait ProgrammaticScrollAnimatorSink {
    /// RegisterForAnimation + ScheduleAnimation. False is the source's
    /// immediate-scroll fallback when this host has no animation scheduler.
    fn ScheduleAnimation(&self) -> bool;
    /// The actual compositor/timeline service decides admission. Returning
    /// false selects the original main-thread animation branch.
    fn AddCompositorAnimation(&self, request: ProgrammaticCompositorAnimation) -> bool;
    fn RemoveCompositorAnimation(
        &self,
        element: CompositorElementId,
        token: ProgrammaticScrollAnimationToken,
    );
    fn ApplyOffset(&self, offset: ProgrammaticScrollOffset);
}
struct Animation {
    request: ProgrammaticScrollRequest,
    curve: ScrollOffsetAnimationCurve,
    start: Option<TimeTicks>,
    token: ProgrammaticScrollAnimationToken,
    completion: Option<ScrollCallback>,
}
struct State {
    run: ProgrammaticScrollRunState,
    animation: Option<Animation>,
}
pub struct ProgrammaticScrollAnimator {
    sink: Rc<dyn ProgrammaticScrollAnimatorSink>,
    config: Arc<ScrollCurveConfig>,
    state: RefCell<State>,
}
impl ProgrammaticScrollAnimator {
    pub fn new(
        sink: Rc<dyn ProgrammaticScrollAnimatorSink>,
        config: Arc<ScrollCurveConfig>,
    ) -> Rc<Self> {
        Rc::new(Self {
            sink,
            config,
            state: RefCell::new(State {
                run: ProgrammaticScrollRunState::Idle,
                animation: None,
            }),
        })
    }
    pub fn RunState(&self) -> ProgrammaticScrollRunState {
        self.state.borrow().run
    }
    pub fn TargetOffset(&self) -> Option<Vector2dF> {
        self.state
            .borrow()
            .animation
            .as_ref()
            .map(|animation| animation.request.target_offset)
    }
    pub fn ScrollToOffsetWithoutAnimation(&self, request: ProgrammaticScrollRequest) {
        self.CancelAnimation();
        self.Apply(request, request.target_offset);
    }
    pub fn AnimateToOffset(
        &self,
        request: ProgrammaticScrollRequest,
        completion: Option<ScrollCallback>,
    ) {
        if self.RunState() == ProgrammaticScrollRunState::PostAnimationCleanup {
            self.ResetAnimationState();
        }
        let previous_completion = self
            .state
            .borrow_mut()
            .animation
            .as_mut()
            .and_then(|animation| animation.completion.take());
        if let Some(completion) = previous_completion {
            completion(ScrollCompletionMode::kInterruptedByScroll);
        }
        {
            let mut state = self.state.borrow_mut();
            if let Some(animation) = state.animation.as_mut() {
                if animation.request.target_offset == request.target_offset {
                    animation.completion = completion;
                    return;
                }
            }
        }
        self.CancelAnimation();
        let curve = ScrollOffsetAnimationCurveFactory::CreateAnimation(
            PointF::new(request.target_offset.x(), request.target_offset.y()),
            ScrollType::kProgrammatic,
            self.config.clone(),
        );
        let token = ProgrammaticScrollAnimationToken {
            model: AnimationIdProvider::NextKeyframeModelId(),
            group: AnimationIdProvider::NextGroupId(),
        };
        *self.state.borrow_mut() = State {
            run: ProgrammaticScrollRunState::WaitingToSendToCompositor,
            animation: Some(Animation {
                request,
                curve,
                token,
                start: None,
                completion,
            }),
        };
        if !self.sink.ScheduleAnimation() {
            self.Apply(request, request.target_offset);
            self.ResetAnimationState();
        }
    }
    /// Source UpdateCompositorAnimations. The caller drives this on its own
    /// rendering opportunity, after the native request selected the target.
    pub fn UpdateCompositorAnimations(&self) {
        if self.RunState() == ProgrammaticScrollRunState::PostAnimationCleanup {
            self.ResetAnimationState();
            return;
        }
        let request = {
            let state = self.state.borrow();
            if state.run != ProgrammaticScrollRunState::WaitingToSendToCompositor {
                return;
            }
            let animation = state.animation.as_ref().expect("waiting source curve");
            ProgrammaticCompositorAnimation {
                request: animation.request,
                token: animation.token,
                curve: animation.curve.Clone(),
            }
        };
        if self.sink.AddCompositorAnimation(request) {
            self.state.borrow_mut().run = ProgrammaticScrollRunState::RunningOnCompositor;
        } else {
            let mut state = self.state.borrow_mut();
            state.run = ProgrammaticScrollRunState::RunningOnMainThread;
            let animation = state.animation.as_mut().expect("main-thread source curve");
            let current = animation.request.current_offset;
            animation.curve.SetInitialValue(
                PointF::new(current.x(), current.y()),
                TimeDelta::default(),
                0.0,
            );
        }
    }
    pub fn TickAnimation(&self, now: TimeTicks) -> bool {
        self.UpdateCompositorAnimations();
        let sample = {
            let mut state = self.state.borrow_mut();
            if state.run != ProgrammaticScrollRunState::RunningOnMainThread {
                return state.run == ProgrammaticScrollRunState::RunningOnCompositor;
            }
            let animation = state.animation.as_mut().expect("running source curve");
            let start = *animation.start.get_or_insert(now);
            let elapsed = now - start;
            let value = animation.curve.GetValue(elapsed);
            let finished = elapsed > animation.curve.Duration();
            let completion = finished.then(|| animation.completion.take()).flatten();
            let request = animation.request;
            if finished {
                state.run = ProgrammaticScrollRunState::PostAnimationCleanup;
            }
            (
                request,
                Vector2dF::new(value.x(), value.y()),
                finished,
                completion,
            )
        };
        self.Apply(sample.0, sample.1);
        if let Some(completion) = sample.3 {
            completion(ScrollCompletionMode::kFinished);
        }
        if !sample.2 && !self.sink.ScheduleAnimation() {
            self.ResetAnimationState();
            return false;
        }
        !sample.2
    }
    pub fn NotifyCompositorAnimationFinished(&self, token: ProgrammaticScrollAnimationToken) {
        let completion = {
            let mut state = self.state.borrow_mut();
            let Some(animation) = state
                .animation
                .as_mut()
                .filter(|animation| animation.token == token)
            else {
                return;
            };
            let completion = animation.completion.take();
            state.run = ProgrammaticScrollRunState::PostAnimationCleanup;
            completion
        };
        if let Some(completion) = completion {
            completion(ScrollCompletionMode::kFinished);
        }
    }
    pub fn NotifyCompositorAnimationAborted(&self, _token: ProgrammaticScrollAnimationToken) {
        /* Source override intentionally empty. */
    }
    pub fn TakeOverCompositorAnimation(&self, current: Vector2dF) {
        let previous = {
            let mut state = self.state.borrow_mut();
            if state.run != ProgrammaticScrollRunState::RunningOnCompositor {
                return;
            }
            state.run = ProgrammaticScrollRunState::RunningOnMainThread;
            let animation = state.animation.as_mut().expect("compositor source curve");
            animation.curve.SetInitialValue(
                PointF::new(current.x(), current.y()),
                TimeDelta::default(),
                0.0,
            );
            (
                animation.request.element,
                animation.token,
                animation.request,
            )
        };
        self.sink.RemoveCompositorAnimation(previous.0, previous.1);
        if !self.sink.ScheduleAnimation() {
            self.Apply(previous.2, previous.2.target_offset);
            self.ResetAnimationState();
        }
    }
    pub fn CancelAnimation(&self) {
        self.Remove(ScrollCompletionMode::kInterruptedByScroll);
    }
    pub fn Dispose(&self) {
        self.CancelAnimation();
    }
    fn ResetAnimationState(&self) {
        self.Remove(ScrollCompletionMode::kFinished);
    }
    fn Remove(&self, mode: ScrollCompletionMode) {
        let (run, animation) = {
            let mut state = self.state.borrow_mut();
            let run = state.run;
            state.run = ProgrammaticScrollRunState::Idle;
            (run, state.animation.take())
        };
        if let Some(mut animation) = animation {
            if run == ProgrammaticScrollRunState::RunningOnCompositor {
                self.sink
                    .RemoveCompositorAnimation(animation.request.element, animation.token);
            }
            if let Some(completion) = animation.completion.take() {
                completion(mode);
            }
        }
    }
    fn Apply(&self, request: ProgrammaticScrollRequest, offset: Vector2dF) {
        self.sink.ApplyOffset(ProgrammaticScrollOffset {
            node_id: request.node_id,
            element: request.element,
            offset,
            source_type: request.source_type,
        });
    }
}
impl Drop for ProgrammaticScrollAnimator {
    fn drop(&mut self) {
        self.Dispose();
    }
}

pub struct ProgrammaticScrollAnimatorRegistry {
    sink: Rc<dyn ProgrammaticScrollAnimatorSink>,
    config: Arc<ScrollCurveConfig>,
    owners: RefCell<HashMap<CompositorElementId, Rc<ProgrammaticScrollAnimator>>>,
}
impl ProgrammaticScrollAnimatorRegistry {
    pub fn new(sink: Rc<dyn ProgrammaticScrollAnimatorSink>) -> Self {
        Self {
            sink,
            config: Arc::new(Default::default()),
            owners: RefCell::new(HashMap::new()),
        }
    }
    pub fn GetOrCreate(&self, element: CompositorElementId) -> Rc<ProgrammaticScrollAnimator> {
        self.owners
            .borrow_mut()
            .entry(element)
            .or_insert_with(|| {
                ProgrammaticScrollAnimator::new(self.sink.clone(), self.config.clone())
            })
            .clone()
    }
    pub fn TargetForElement(&self, element: CompositorElementId) -> Option<Vector2dF> {
        self.owners
            .borrow()
            .get(&element)
            .and_then(|owner| owner.TargetOffset())
    }
    pub fn Cancel(&self, element: CompositorElementId) {
        let owner = self.owners.borrow().get(&element).cloned();
        if let Some(owner) = owner {
            owner.CancelAnimation();
        }
    }
    pub fn CancelAll(&self) {
        let owners = self.owners.borrow().values().cloned().collect::<Vec<_>>();
        for owner in owners {
            owner.CancelAnimation();
        }
    }
    pub fn TickAnimations(&self, now: TimeTicks) -> bool {
        let owners = self.owners.borrow().values().cloned().collect::<Vec<_>>();
        let mut active = false;
        for owner in owners {
            active |= owner.TickAnimation(now);
        }
        active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Sink {
        offsets: RefCell<Vec<f32>>,
    }
    impl ProgrammaticScrollAnimatorSink for Sink {
        fn ScheduleAnimation(&self) -> bool {
            true
        }
        fn AddCompositorAnimation(&self, _: ProgrammaticCompositorAnimation) -> bool {
            false
        }
        fn RemoveCompositorAnimation(
            &self,
            _: CompositorElementId,
            _: ProgrammaticScrollAnimationToken,
        ) {
        }
        fn ApplyOffset(&self, value: ProgrammaticScrollOffset) {
            self.offsets.borrow_mut().push(value.offset.y());
        }
    }
    #[test]
    fn main_thread_programmatic_curve_retains_frames_and_finishes_once() {
        let sink = Rc::new(Sink::default());
        let animator = ProgrammaticScrollAnimator::new(sink.clone(), Arc::new(Default::default()));
        let completion = Rc::new(RefCell::new(Vec::new()));
        let completed = completion.clone();
        animator.AnimateToOffset(
            ProgrammaticScrollRequest {
                node_id: 1,
                element: CompositorElementId::default(),
                current_offset: Vector2dF::new(0., 0.),
                target_offset: Vector2dF::new(0., 300.),
                source_type: ScrollSourceType::kAbsoluteScroll,
            },
            Some(Box::new(move |mode| completed.borrow_mut().push(mode))),
        );
        assert!(animator.TickAnimation(TimeTicks::FromMicroseconds(1_000_000)));
        assert!(animator.TickAnimation(TimeTicks::FromMicroseconds(1_050_000)));
        let middle = *sink.offsets.borrow().last().unwrap();
        assert!(middle > 0. && middle < 300.);
        assert!(!animator.TickAnimation(TimeTicks::FromMicroseconds(10_000_000)));
        assert_eq!(*sink.offsets.borrow().last().unwrap(), 300.);
        animator.CancelAnimation();
        assert_eq!(*completion.borrow(), [ScrollCompletionMode::kFinished]);
    }
}
