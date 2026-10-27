// Translation status: complete
// Source: ui/gfx/animation/keyframe/keyframe_effect.h:1-133
// Source: ui/gfx/animation/keyframe/keyframe_effect.cc:1-374
use crate::{
    cubic_bezier::CubicBezier,
    gfx_keyframe_model::{AnimationCurve, Direction, LimitDirection, RunState},
};
use compositor_scheduler::{TimeDelta, TimeTicks};
use std::{collections::BTreeSet, ops::Range};
// Source dependency: ui/gfx/animation/keyframe/transition.h:17-30; .cc:9-18.
#[derive(Clone)]
pub struct Transition {
    pub duration: TimeDelta,
    pub target_properties: BTreeSet<i32>,
}
impl Default for Transition {
    fn default() -> Self {
        Self {
            duration: TimeDelta::Milliseconds(225),
            target_properties: BTreeSet::new(),
        }
    }
}
pub trait EffectModel: Sized {
    type Curve: AnimationCurve;
    fn id(&self) -> i32;
    fn TargetProperty(&self) -> i32;
    fn run_state(&self) -> RunState;
    fn SetRunState(&mut self, state: RunState);
    fn iterations(&self) -> f64;
    fn direction(&self) -> Direction;
    fn playback_rate(&self) -> f64;
    fn hold_time(&self) -> Option<TimeDelta>;
    fn set_hold_time(&mut self, t: Option<TimeDelta>);
    fn set_start_time(&mut self, t: TimeTicks);
    fn curve(&self) -> Option<&Self::Curve>;
    fn Reverse(&mut self, now: TimeTicks);
    fn is_finished(&self) -> bool;
    fn IsFinishedAt(&self, now: TimeTicks) -> bool;
    fn HasActiveTime(&self, now: TimeTicks) -> bool;
    fn TrimTimeToCurrentIteration(
        &self,
        now: TimeTicks,
        limit: Option<&mut LimitDirection>,
    ) -> TimeDelta;
}
/// Source virtual dispatch and clock/curve targets are lent per call. An
/// overriding cc runtime forwards RemoveRange to its canonical same-stem body.
pub trait EffectRuntime<M: EffectModel> {
    fn Now(&self) -> TimeTicks;
    fn TickCurve(
        &mut self,
        curve: &M::Curve,
        time: TimeDelta,
        property: i32,
        model: &M,
        limit: LimitDirection,
    );
    fn RemoveRange(&mut self, models: &mut Vec<Box<M>>, range: Range<usize>);
    fn AddModel(&mut self, models: &mut Vec<Box<M>>, model: Box<M>);
}
pub trait TransitionTarget<V> {
    fn OnValueAnimated(&mut self, value: &V, property: i32);
}
pub trait TransitionValue: Clone {
    fn SufficientlyEqual(&self, other: &Self) -> bool;
}
// Source dependency: ui/gfx/animation/keyframe/keyframed_animation_curve.cc:23,564-566.
impl TransitionValue for f32 {
    fn SufficientlyEqual(&self, other: &Self) -> bool {
        (*self - *other).abs() <= 1e-5f32
    }
}
pub trait TransitionCurve<V>: AnimationCurve + Sized {
    fn Create() -> Self;
    fn AddKeyframe(&mut self, time: TimeDelta, value: V, easing: CubicBezier);
    fn SetTarget(&mut self, target: &mut dyn TransitionTarget<V>);
    fn GetTransformedValue(&self, time: TimeDelta, limit: LimitDirection) -> V;
}
pub trait TransitionModel: EffectModel {
    fn CreateTransition(curve: Self::Curve, id: i32, property: i32) -> Self;
}
// Source: ui/gfx/animation/keyframe/keyframe_effect.cc:17-18. Native sequence-local counters.
thread_local! {static s_next_keyframe_model_id:std::cell::Cell<i32>=const{std::cell::Cell::new(1)};static s_next_group_id:std::cell::Cell<i32>=const{std::cell::Cell::new(1)};}
// Source: ui/gfx/animation/keyframe/keyframe_effect.h:126-129
pub struct KeyframeEffect<M> {
    keyframe_models_: Vec<Box<M>>,
    transition_: Transition,
}
impl<M> Default for KeyframeEffect<M> {
    fn default() -> Self {
        Self {
            keyframe_models_: Vec::new(),
            transition_: Transition::default(),
        }
    }
}
impl<M: EffectModel> KeyframeEffect<M> {
    // Source: ui/gfx/animation/keyframe/keyframe_effect.cc:113-125
    pub fn GetNextKeyframeModelId() -> i32 {
        s_next_keyframe_model_id.with(|id| {
            let value = id.get();
            id.set(value.wrapping_add(1));
            value
        })
    }
    pub fn GetNextGroupId() -> i32 {
        s_next_group_id.with(|id| {
            let value = id.get();
            id.set(value.wrapping_add(1));
            value
        })
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn AddKeyframeModel(&mut self, model: Box<M>) {
        self.keyframe_models_.push(model);
    }
    // Source: ui/gfx/animation/keyframe/keyframe_effect.cc:127-156
    // std::stable_partition uses a temporary buffer of owning handles. Only Box
    // handles move here; resident model allocations and their identity stay fixed.
    fn Partition(&mut self, predicate: impl Fn(&M) -> bool) -> usize {
        let mut kept = Vec::with_capacity(self.keyframe_models_.len());
        let mut removed = Vec::new();
        for model in self.keyframe_models_.drain(..) {
            if predicate(&model) {
                kept.push(model);
            } else {
                removed.push(model);
            }
        }
        let first = kept.len();
        kept.append(&mut removed);
        self.keyframe_models_ = kept;
        first
    }
    pub fn RemoveKeyframeModel(&mut self, id: i32, context: &mut impl EffectRuntime<M>) {
        let first = self.Partition(|model| model.id() != id);
        let end = self.keyframe_models_.len();
        context.RemoveRange(&mut self.keyframe_models_, first..end);
    }
    pub fn RemoveKeyframeModels(&mut self, property: i32, context: &mut impl EffectRuntime<M>) {
        let first = self.Partition(|model| model.TargetProperty() != property);
        let end = self.keyframe_models_.len();
        context.RemoveRange(&mut self.keyframe_models_, first..end);
    }
    pub fn RemoveAllKeyframeModels(&mut self, context: &mut impl EffectRuntime<M>) {
        let end = self.keyframe_models_.len();
        context.RemoveRange(&mut self.keyframe_models_, 0..end);
    }
    // Source: ui/gfx/animation/keyframe/keyframe_effect.cc:158-166
    pub fn Tick(&mut self, now: TimeTicks, context: &mut impl EffectRuntime<M>) -> bool {
        self.TickInternal(now, true, context)
    }
    pub fn RemoveKeyframeModelRange(models: &mut Vec<Box<M>>, range: Range<usize>) {
        models.drain(range).for_each(drop);
    }
    // Source: ui/gfx/animation/keyframe/keyframe_effect.cc:168-186
    pub fn TickKeyframeModel(now: TimeTicks, model: &M, context: &mut impl EffectRuntime<M>) {
        if (!matches!(
            model.run_state(),
            RunState::STARTING
                | RunState::RUNNING
                | RunState::PAUSED
                | RunState::PAUSED_EXCLUSIVE
                | RunState::WAITING_FOR_DELETION
        )) || !model.HasActiveTime(now)
        {
            return;
        }
        let curve = model.curve().unwrap();
        let mut limit = LimitDirection::RIGHT;
        let trimmed = model.TrimTimeToCurrentIteration(now, Some(&mut limit));
        context.TickCurve(curve, trimmed, model.TargetProperty(), model, limit);
    }
    // Source: ui/gfx/animation/keyframe/keyframe_effect.cc:188-210
    fn TickInternal(
        &mut self,
        now: TimeTicks,
        include_infinite: bool,
        context: &mut impl EffectRuntime<M>,
    ) -> bool {
        self.StartKeyframeModels(now, include_infinite);
        let mut active = false;
        for model in &self.keyframe_models_ {
            if !include_infinite && model.iterations() == f64::INFINITY {
                continue;
            }
            Self::TickKeyframeModel(now, model, context);
            active = true;
        }
        self.keyframe_models_
            .retain(|model| !(!model.is_finished() && model.IsFinishedAt(now)));
        self.StartKeyframeModels(now, include_infinite);
        active
    }
    // Source: ui/gfx/animation/keyframe/keyframe_effect.cc:212-224
    pub fn FinishAll(&mut self, context: &mut impl EffectRuntime<M>) {
        let now = context.Now();
        self.TickInternal(now, false, context);
        self.TickInternal(TimeTicks::Max(), false, context);
        #[cfg(debug_assertions)]
        for model in &self.keyframe_models_ {
            debug_assert_eq!(model.iterations(), f64::INFINITY);
        }
    }
    // Source: ui/gfx/animation/keyframe/keyframe_effect.cc:226-233
    pub fn SetTransitionedProperties(&mut self, properties: &BTreeSet<i32>) {
        self.transition_.target_properties = properties.clone();
    }
    pub fn SetTransitionDuration(&mut self, d: TimeDelta) {
        self.transition_.duration = d;
    }
    // Source: ui/gfx/animation/keyframe/keyframe_effect.cc:45-110
    fn TransitionValueTo<V: TransitionValue>(
        &mut self,
        target: &mut dyn TransitionTarget<V>,
        now: TimeTicks,
        property: i32,
        from: &V,
        to: &V,
        context: &mut impl EffectRuntime<M>,
    ) where
        M: TransitionModel,
        M::Curve: TransitionCurve<V>,
    {
        if !self.transition_.target_properties.contains(&property) {
            target.OnValueAnimated(to, property);
            return;
        }
        let mut effective_current = from.clone();
        if let Some(model) = self.GetRunningKeyframeModelForPropertyMutable(property) {
            let curve = model.curve().unwrap();
            if model.IsFinishedAt(now) {
                effective_current =
                    curve.GetTransformedValue(Self::GetEndTime(model), LimitDirection::RIGHT);
            } else {
                if to.SufficientlyEqual(
                    &curve.GetTransformedValue(Self::GetEndTime(model), LimitDirection::RIGHT),
                ) {
                    return;
                }
                if to.SufficientlyEqual(
                    &curve.GetTransformedValue(Self::GetStartTime(model), LimitDirection::RIGHT),
                ) {
                    model.Reverse(now);
                    return;
                }
            }
        } else if to.SufficientlyEqual(from) {
            return;
        }
        self.RemoveKeyframeModels(property, context);
        let mut curve = M::Curve::Create();
        curve.AddKeyframe(
            TimeDelta::default(),
            effective_current,
            CubicBezier::new(0.25, 0.1, 0.25, 1.0),
        );
        curve.AddKeyframe(
            self.transition_.duration,
            to.clone(),
            CubicBezier::new(0.25, 0.1, 0.25, 1.0),
        );
        curve.SetTarget(target);
        context.AddModel(
            &mut self.keyframe_models_,
            Box::new(M::CreateTransition(
                curve,
                Self::GetNextKeyframeModelId(),
                property,
            )),
        );
    }
    // Source: ui/gfx/animation/keyframe/keyframe_effect.cc:235-270
    pub fn TransitionFloatTo(
        &mut self,
        target: &mut dyn TransitionTarget<f32>,
        now: TimeTicks,
        property: i32,
        from: f32,
        to: f32,
        context: &mut impl EffectRuntime<M>,
    ) where
        M: TransitionModel,
        M::Curve: TransitionCurve<f32>,
        f32: TransitionValue,
    {
        self.TransitionValueTo(target, now, property, &from, &to, context);
    }
    pub fn TransitionTransformOperationsTo<V: TransitionValue>(
        &mut self,
        target: &mut dyn TransitionTarget<V>,
        now: TimeTicks,
        property: i32,
        from: &V,
        to: &V,
        context: &mut impl EffectRuntime<M>,
    ) where
        M: TransitionModel,
        M::Curve: TransitionCurve<V>,
    {
        self.TransitionValueTo(target, now, property, from, to, context);
    }
    pub fn TransitionSizeTo<V: TransitionValue>(
        &mut self,
        target: &mut dyn TransitionTarget<V>,
        now: TimeTicks,
        property: i32,
        from: &V,
        to: &V,
        context: &mut impl EffectRuntime<M>,
    ) where
        M: TransitionModel,
        M::Curve: TransitionCurve<V>,
    {
        self.TransitionValueTo(target, now, property, from, to, context);
    }
    pub fn TransitionColorTo<V: TransitionValue>(
        &mut self,
        target: &mut dyn TransitionTarget<V>,
        now: TimeTicks,
        property: i32,
        from: &V,
        to: &V,
        context: &mut impl EffectRuntime<M>,
    ) where
        M: TransitionModel,
        M::Curve: TransitionCurve<V>,
    {
        self.TransitionValueTo(target, now, property, from, to, context);
    }
    // Source: ui/gfx/animation/keyframe/keyframe_effect.cc:272-303
    pub fn IsAnimatingProperty(&self, property: i32) -> bool {
        for model in &self.keyframe_models_ {
            if model.TargetProperty() == property {
                return true;
            }
        }
        false
    }
    pub fn IsAnimating(&self) -> bool {
        !self.keyframe_models_.is_empty()
    }
    pub fn GetTargetFloatValue(&self, property: i32, default: f32) -> f32
    where
        M::Curve: TransitionCurve<f32>,
    {
        self.GetTargetValue(property, &default)
    }
    pub fn GetTargetTransformOperationsValue<V: Clone>(&self, property: i32, default: &V) -> V
    where
        M::Curve: TransitionCurve<V>,
    {
        self.GetTargetValue(property, default)
    }
    pub fn GetTargetSizeValue<V: Clone>(&self, property: i32, default: &V) -> V
    where
        M::Curve: TransitionCurve<V>,
    {
        self.GetTargetValue(property, default)
    }
    pub fn GetTargetColorValue<V: Clone>(&self, property: i32, default: &V) -> V
    where
        M::Curve: TransitionCurve<V>,
    {
        self.GetTargetValue(property, default)
    }
    // Source: ui/gfx/animation/keyframe/keyframe_effect.cc:305-337
    fn StartKeyframeModels(&mut self, now: TimeTicks, include_infinite: bool) {
        let mut properties = [false; 32];
        for model in &self.keyframe_models_ {
            if !include_infinite && model.iterations() == f64::INFINITY {
                continue;
            }
            if matches!(
                model.run_state(),
                RunState::RUNNING | RunState::PAUSED | RunState::PAUSED_EXCLUSIVE
            ) {
                properties[model.TargetProperty() as usize] = true;
            }
        }
        for model in &mut self.keyframe_models_ {
            if !include_infinite && model.iterations() == f64::INFINITY {
                continue;
            }
            let property = model.TargetProperty() as usize;
            if !properties[property]
                && model.run_state() == RunState::WAITING_FOR_TARGET_AVAILABILITY
            {
                properties[property] = true;
                model.SetRunState(RunState::RUNNING);
                model.set_start_time(
                    now - model.hold_time().unwrap_or_default() / model.playback_rate(),
                );
                model.set_hold_time(None);
            }
        }
    }
    // Source: ui/gfx/animation/keyframe/keyframe_effect.cc:339-364
    pub fn GetRunningKeyframeModelForProperty(&self, property: i32) -> Option<&M> {
        for model in &self.keyframe_models_ {
            if matches!(
                model.run_state(),
                RunState::RUNNING | RunState::PAUSED | RunState::PAUSED_EXCLUSIVE
            ) && model.TargetProperty() == property
            {
                return Some(model);
            }
        }
        None
    }
    pub fn GetRunningKeyframeModelForPropertyMutable(&mut self, property: i32) -> Option<&mut M> {
        for model in &mut self.keyframe_models_ {
            if matches!(
                model.run_state(),
                RunState::RUNNING | RunState::PAUSED | RunState::PAUSED_EXCLUSIVE
            ) && model.TargetProperty() == property
            {
                return Some(model);
            }
        }
        None
    }
    pub fn GetKeyframeModel(&self, property: i32) -> Option<&M> {
        for i in 0..self.keyframe_models_.len() {
            let index = self.keyframe_models_.len() - i - 1;
            if self.keyframe_models_[index].TargetProperty() == property {
                return Some(&self.keyframe_models_[index]);
            }
        }
        None
    }
    pub fn GetKeyframeModelById(&self, id: i32) -> Option<&M> {
        for model in &self.keyframe_models_ {
            if model.id() == id {
                return Some(model);
            }
        }
        None
    }
    pub fn GetKeyframeModelByIdMutable(&mut self, id: i32) -> Option<&mut M> {
        for model in &mut self.keyframe_models_ {
            if model.id() == id {
                return Some(model);
            }
        }
        None
    }
    // Source: ui/gfx/animation/keyframe/keyframe_effect.cc:366-372
    fn GetTargetValue<V: Clone>(&self, property: i32, default: &V) -> V
    where
        M::Curve: TransitionCurve<V>,
    {
        let Some(model) = self.GetKeyframeModel(property) else {
            return default.clone();
        };
        let curve = model.curve().unwrap();
        curve.GetTransformedValue(Self::GetEndTime(model), LimitDirection::RIGHT)
    }
    // Source: ui/gfx/animation/keyframe/keyframe_effect.cc:31-43
    fn GetStartTime(model: &M) -> TimeDelta {
        if model.direction() == Direction::NORMAL {
            TimeDelta::default()
        } else {
            model.curve().unwrap().Duration()
        }
    }
    fn GetEndTime(model: &M) -> TimeDelta {
        if model.direction() == Direction::REVERSE {
            TimeDelta::default()
        } else {
            model.curve().unwrap().Duration()
        }
    }
    // Source: ui/gfx/animation/keyframe/keyframe_effect.h:57-67
    pub fn keyframe_models(&self) -> &Vec<Box<M>> {
        &self.keyframe_models_
    }
    pub fn keyframe_models_mut(&mut self) -> &mut Vec<Box<M>> {
        &mut self.keyframe_models_
    }
    pub fn transition(&self) -> &Transition {
        &self.transition_
    }
    pub fn set_transition(&mut self, t: &Transition) {
        self.transition_ = t.clone();
    }
}

// Source native virtual model interfaces resolve to the resident base owner.
impl<C: AnimationCurve> EffectModel for crate::gfx_keyframe_model::KeyframeModel<C> {
    type Curve = C;
    fn id(&self) -> i32 {
        self.id()
    }
    fn TargetProperty(&self) -> i32 {
        self.TargetProperty()
    }
    fn run_state(&self) -> RunState {
        self.run_state()
    }
    fn SetRunState(&mut self, state: RunState) {
        self.SetRunState(state);
    }
    fn iterations(&self) -> f64 {
        self.iterations()
    }
    fn direction(&self) -> Direction {
        self.direction()
    }
    fn playback_rate(&self) -> f64 {
        self.playback_rate()
    }
    fn hold_time(&self) -> Option<TimeDelta> {
        self.hold_time()
    }
    fn set_hold_time(&mut self, t: Option<TimeDelta>) {
        self.set_hold_time(t);
    }
    fn set_start_time(&mut self, t: TimeTicks) {
        self.set_start_time(t);
    }
    fn curve(&self) -> Option<&C> {
        self.curve()
    }
    fn Reverse(&mut self, now: TimeTicks) {
        self.Reverse(now, false);
    }
    fn is_finished(&self) -> bool {
        self.is_finished()
    }
    fn IsFinishedAt(&self, now: TimeTicks) -> bool {
        self.IsFinishedAt(now, false)
    }
    fn HasActiveTime(&self, now: TimeTicks) -> bool {
        self.HasActiveTime(now, false)
    }
    fn TrimTimeToCurrentIteration(
        &self,
        now: TimeTicks,
        limit: Option<&mut LimitDirection>,
    ) -> TimeDelta {
        self.TrimTimeToCurrentIteration(now, false, limit)
    }
}
impl<C: AnimationCurve> TransitionModel for crate::gfx_keyframe_model::KeyframeModel<C> {
    fn CreateTransition(curve: C, id: i32, property: i32) -> Self {
        Self::Create(Some(curve), id, property)
    }
}
impl<C: crate::keyframe_model::KeyframeCurve> EffectModel
    for crate::keyframe_model::KeyframeModel<C>
{
    type Curve = C;
    fn id(&self) -> i32 {
        self.base().id()
    }
    fn TargetProperty(&self) -> i32 {
        self.TargetProperty()
    }
    fn run_state(&self) -> RunState {
        self.base().run_state()
    }
    fn SetRunState(&mut self, state: RunState) {
        self.SetRunState(state);
    }
    fn iterations(&self) -> f64 {
        self.base().iterations()
    }
    fn direction(&self) -> Direction {
        self.base().direction()
    }
    fn playback_rate(&self) -> f64 {
        self.base().playback_rate()
    }
    fn hold_time(&self) -> Option<TimeDelta> {
        self.base().hold_time()
    }
    fn set_hold_time(&mut self, t: Option<TimeDelta>) {
        self.base_mut().set_hold_time(t);
    }
    fn set_start_time(&mut self, t: TimeTicks) {
        self.base_mut().set_start_time(t);
    }
    fn curve(&self) -> Option<&C> {
        self.base().curve()
    }
    fn Reverse(&mut self, now: TimeTicks) {
        self.Reverse(now);
    }
    fn is_finished(&self) -> bool {
        self.base().is_finished()
    }
    fn IsFinishedAt(&self, now: TimeTicks) -> bool {
        self.IsFinishedAt(now)
    }
    fn HasActiveTime(&self, now: TimeTicks) -> bool {
        self.HasActiveTime(now)
    }
    fn TrimTimeToCurrentIteration(
        &self,
        now: TimeTicks,
        limit: Option<&mut LimitDirection>,
    ) -> TimeDelta {
        self.TrimTimeToCurrentIteration(now, limit)
    }
}
