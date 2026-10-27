// Translation status: complete
// Source: cc/animation/animation.h:1-242
// Source: cc/animation/animation.cc:1-462
//
// Chromium keeps raw AnimationHost/AnimationTimeline/AnimationDelegate pointers
// beside one uniquely owned KeyframeEffect and shares Animation itself by
// scoped_refptr.  This translation preserves those ownership relations without
// retaining reverse pointers: host, timeline and delegate are stable identities,
// while every dereference is performed through a live call-scoped context.

use super::{
    animation_events::{
        AnimationEventCurve, AnimationEvents, AnimationPlaybackEvent, PlaybackType,
        UniqueKeyframeModelId,
    },
    keyframe_effect::{
        BoundElementAnimations, KeyframeEffect, KeyframeEffectContext, KeyframeEffectPushContext,
    },
    keyframe_model::{KeyframeCurve, KeyframeModel, LimitDirection, RunState},
};
use crate::gfx_keyframe_model as gfx_model;
use compositor_scheduler::{TimeDelta, TimeTicks};
use compositor_tree::{geometry::PointF, target_property::Type as TargetProperty, ElementId};
use std::sync::{Arc, Mutex};

// Source dependency: source raw pointers are represented by non-owning stable
// identities.  The live owner is supplied only for the duration of a call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimationHostId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimationDelegateId(pub u64);

/// `scoped_refptr<Animation>` correspondence.  The allocation is shared; its
/// single KeyframeEffect remains owned by the Animation allocation.
pub type AnimationRef<C> = Arc<Mutex<Animation<C>>>;

/// Operations that dereference source host/timeline/delegate pointers.  A
/// context is lent by the actual owner for one call and cannot be retained by
/// Animation or KeyframeEffect.
pub trait AnimationContext<C: KeyframeCurve + AnimationEventCurve> {
    fn Now(&self) -> TimeTicks;
    fn TickCurve(
        &mut self,
        curve: &C,
        time: TimeDelta,
        property: i32,
        model: &KeyframeModel<C>,
        limit: LimitDirection,
    );
    fn HostIsOwnerThread(&self, host: AnimationHostId) -> bool;
    fn HostInProtectedSequence(&self, host: AnimationHostId) -> bool;
    fn HostWaitForProtectedSequenceCompletion(&self, host: AnimationHostId);
    fn HostRegisterAnimationForElement(
        &mut self,
        host: AnimationHostId,
        element: ElementId,
        animation: i32,
    ) -> Arc<dyn BoundElementAnimations>;
    fn HostUnregisterAnimationForElement(
        &mut self,
        host: AnimationHostId,
        element: ElementId,
        animation: i32,
    );
    fn HostAddToTicking(&mut self, host: AnimationHostId, animation: i32);
    fn HostRemoveFromTicking(&mut self, host: AnimationHostId, animation: i32);
    fn HostSetNeedsCommit(&mut self, host: AnimationHostId);
    fn TimelineIsScrollTimeline(&self, timeline: i32) -> bool;
    fn TimelineSetNeedsPushProperties(&mut self, timeline: i32);
    fn ElementSetNeedsPushProperties(&mut self, element: &Arc<dyn BoundElementAnimations>);
    fn ElementUpdateClientAnimationState(&mut self, element: &Arc<dyn BoundElementAnimations>);
    fn ElementAttachToCurve(&mut self, element: &Arc<dyn BoundElementAnimations>, curve: &mut C);
    fn UpdateClientAnimationStateForElement(&mut self, element: ElementId);
    fn ElementScrollOffsetForAnimation(
        &self,
        element: &Arc<dyn BoundElementAnimations>,
    ) -> Option<PointF>;
    fn QueueAnimationEvent(
        &mut self,
        animation: i32,
        event: &AnimationPlaybackEvent<C>,
        receiver: &mut KeyframeEffect<C>,
        host: Option<AnimationHostId>,
        timeline: Option<i32>,
        delegate: Option<AnimationDelegateId>,
    );
    fn DelegateAnimationStarted(
        &mut self,
        delegate: AnimationDelegateId,
        monotonic_time: TimeTicks,
        target_property: i32,
        group_id: i32,
    );
    fn DelegateAnimationFinished(
        &mut self,
        delegate: AnimationDelegateId,
        monotonic_time: TimeTicks,
        target_property: i32,
        group_id: i32,
    );
    fn DelegateAnimationAborted(
        &mut self,
        delegate: AnimationDelegateId,
        monotonic_time: TimeTicks,
        target_property: i32,
        group_id: i32,
    );
    fn DelegateAnimationTakeover(
        &mut self,
        delegate: AnimationDelegateId,
        monotonic_time: TimeTicks,
        target_property: i32,
        animation_start_time: TimeTicks,
        curve: C,
    );
    fn DelegateLocalTimeUpdated(
        &mut self,
        delegate: AnimationDelegateId,
        local_time: Option<TimeDelta>,
    );
    fn TraceEffect(&self, event: &'static str, message: String);
}

// Source: cc/animation/animation.h:171-180.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoRewind {
    kDisabled,
    kEnabled,
    kForced,
}

// Source: cc/animation/animation.h:28-219.
pub struct Animation<C: KeyframeCurve + AnimationEventCurve> {
    animation_delegate_: Option<AnimationDelegateId>,
    id_: i32,
    is_replacement_: bool,
    animation_host_: Option<AnimationHostId>,
    animation_timeline_: Option<i32>,
    keyframe_effect_: KeyframeEffect<C>,
}

/// Split borrow used when KeyframeEffect calls source Animation/Host methods.
/// It sees the actual Animation sibling fields and delegates external ownership
/// operations to the current live context; no owner is recovered from an id.
struct EffectContext<'a, C, X>
where
    C: KeyframeCurve + AnimationEventCurve,
    X: AnimationContext<C>,
{
    id: i32,
    host: &'a Option<AnimationHostId>,
    timeline: &'a Option<i32>,
    delegate: &'a Option<AnimationDelegateId>,
    external: &'a mut X,
    _curve: std::marker::PhantomData<C>,
}
impl<C, X> KeyframeEffectContext<C> for EffectContext<'_, C, X>
where
    C: KeyframeCurve + AnimationEventCurve,
    X: AnimationContext<C>,
{
    fn Now(&self) -> TimeTicks {
        self.external.Now()
    }
    fn TickCurve(
        &mut self,
        curve: &C,
        time: TimeDelta,
        property: i32,
        model: &KeyframeModel<C>,
        limit: LimitDirection,
    ) {
        self.external.TickCurve(curve, time, property, model, limit)
    }
    fn AnimationRegisterForElement(
        &mut self,
        element: ElementId,
    ) -> Arc<dyn BoundElementAnimations> {
        self.external.HostRegisterAnimationForElement(
            self.host.expect("attached AnimationHost"),
            element,
            self.id,
        )
    }
    fn AnimationUnregisterForElement(&mut self, element: ElementId) {
        self.external.HostUnregisterAnimationForElement(
            self.host.expect("attached AnimationHost"),
            element,
            self.id,
        )
    }
    fn ElementAttachToCurve(&mut self, element: &Arc<dyn BoundElementAnimations>, curve: &mut C) {
        self.external.ElementAttachToCurve(element, curve)
    }
    fn UpdateClientAnimationStateForElement(&mut self, element: ElementId) {
        self.external.UpdateClientAnimationStateForElement(element)
    }
    fn AnimationId(&self) -> i32 {
        self.id
    }
    fn AnimationTimelineId(&self) -> Option<i32> {
        *self.timeline
    }
    fn AnimationHasHost(&self) -> bool {
        self.host.is_some()
    }
    fn AnimationIsScrollLinked(&self) -> bool {
        self.timeline
            .is_some_and(|timeline| self.external.TimelineIsScrollTimeline(timeline))
    }
    fn AnimationSetNeedsPushProperties(&mut self) {
        if let Some(timeline) = *self.timeline {
            self.external.TimelineSetNeedsPushProperties(timeline);
        }
    }
    fn AnimationSetNeedsCommit(&mut self) {
        self.external
            .HostSetNeedsCommit(self.host.expect("attached AnimationHost"));
    }
    fn AnimationAddToTicking(&mut self) {
        self.external
            .HostAddToTicking(self.host.expect("attached AnimationHost"), self.id);
    }
    fn AnimationRemoveFromTicking(&mut self) {
        self.external
            .HostRemoveFromTicking(self.host.expect("attached AnimationHost"), self.id);
    }
    fn ElementSetNeedsPushProperties(&mut self, element: &Arc<dyn BoundElementAnimations>) {
        self.external.ElementSetNeedsPushProperties(element)
    }
    fn ElementUpdateClientAnimationState(&mut self, element: &Arc<dyn BoundElementAnimations>) {
        self.external.ElementUpdateClientAnimationState(element)
    }
    fn ElementScrollOffsetForAnimation(
        &self,
        element: &Arc<dyn BoundElementAnimations>,
    ) -> Option<PointF> {
        self.external.ElementScrollOffsetForAnimation(element)
    }
    fn DispatchAndDelegateAnimationEvent(
        &mut self,
        event: &AnimationPlaybackEvent<C>,
        receiver: &mut KeyframeEffect<C>,
    ) {
        self.external.QueueAnimationEvent(
            self.id,
            event,
            receiver,
            *self.host,
            *self.timeline,
            *self.delegate,
        )
    }
    fn TraceEffect(&self, event: &'static str, message: String) {
        self.external.TraceEffect(event, message)
    }
}

impl<C: KeyframeCurve + AnimationEventCurve> Animation<C> {
    // Source: cc/animation/animation.cc:25-40.
    pub fn Create(id: i32) -> AnimationRef<C> {
        Arc::new(Mutex::new(Self::new(id)))
    }
    pub fn new(id: i32) -> Self {
        assert_ne!(id, 0);
        Self {
            animation_delegate_: None,
            id_: id,
            is_replacement_: false,
            animation_host_: None,
            animation_timeline_: None,
            keyframe_effect_: KeyframeEffect::new(id),
        }
    }
    pub fn CreateImplInstance(&self) -> AnimationRef<C> {
        Self::Create(self.id())
    }

    // Source: cc/animation/animation.h:38-69; animation.cc:42-54.
    pub fn id(&self) -> i32 {
        self.id_
    }
    pub fn element_id(&self) -> ElementId {
        self.keyframe_effect_.element_id()
    }
    pub fn keyframe_effect(&self) -> &KeyframeEffect<C> {
        &self.keyframe_effect_
    }
    pub fn keyframe_effect_mut(&mut self) -> &mut KeyframeEffect<C> {
        &mut self.keyframe_effect_
    }
    pub fn animation_host(&self) -> Option<AnimationHostId> {
        self.animation_host_
    }
    pub fn has_animation_host(&self) -> bool {
        self.animation_host_.is_some()
    }
    pub fn SetAnimationHost<X: AnimationContext<C>>(
        &mut self,
        animation_host: Option<AnimationHostId>,
        context: &mut X,
    ) {
        debug_assert!(self.IsOwnerThread(context));
        if animation_host == self.animation_host_ {
            return;
        }
        self.WaitForProtectedSequenceCompletion(context);
        self.animation_host_ = animation_host;
    }

    // Source: cc/animation/animation.h:71-95; animation.cc:56-97.
    pub fn animation_timeline(&self) -> Option<i32> {
        self.animation_timeline_
    }
    pub fn SetAnimationTimeline<X: AnimationContext<C>>(
        &mut self,
        timeline: Option<i32>,
        context: &mut X,
    ) {
        if self.animation_timeline_ == timeline {
            return;
        }
        if self.keyframe_effect_.has_attached_element()
            && self.keyframe_effect_.has_bound_element_animations()
        {
            self.UnregisterAnimation(context);
        }
        self.animation_timeline_ = timeline;
        if self.keyframe_effect_.has_attached_element() && self.animation_host_.is_some() {
            self.RegisterAnimation(context);
        }
    }
    pub fn element_animations(&self) -> Option<Arc<dyn BoundElementAnimations>> {
        self.keyframe_effect_.element_animations().cloned()
    }
    pub fn set_animation_delegate(&mut self, delegate: Option<AnimationDelegateId>) {
        self.animation_delegate_ = delegate;
    }
    pub fn AttachElement<X: AnimationContext<C>>(
        &mut self,
        element_id: ElementId,
        context: &mut X,
    ) {
        assert_ne!(element_id, reserved_element_id_for_paint_worklet());
        self.AttachElementInternal(element_id, context);
    }
    pub fn AttachPaintWorkletElement<X: AnimationContext<C>>(&mut self, context: &mut X) {
        self.AttachElementInternal(reserved_element_id_for_paint_worklet(), context);
    }
    fn AttachElementInternal<X: AnimationContext<C>>(
        &mut self,
        element_id: ElementId,
        context: &mut X,
    ) {
        self.WithEffectContext(context, |effect, effect_context| {
            Self::AttachElementForEffect(effect, element_id, effect_context)
        });
    }
    pub fn SetKeyframeEffectForTesting(&mut self, effect: KeyframeEffect<C>) {
        self.keyframe_effect_ = effect;
    }

    // Source: cc/animation/animation.cc:99-139.
    pub fn IsOwnerThread<X: AnimationContext<C>>(&self, context: &X) -> bool {
        self.animation_host_
            .is_none_or(|host| context.HostIsOwnerThread(host))
    }
    pub fn InProtectedSequence<X: AnimationContext<C>>(&self, context: &X) -> bool {
        self.animation_host_
            .is_none_or(|host| context.HostInProtectedSequence(host))
    }
    pub fn WaitForProtectedSequenceCompletion<X: AnimationContext<C>>(&self, context: &X) {
        if let Some(host) = self.animation_host_ {
            context.HostWaitForProtectedSequenceCompletion(host);
        }
    }
    pub fn DetachElement<X: AnimationContext<C>>(&mut self, context: &mut X) {
        debug_assert!(self.keyframe_effect_.has_attached_element());
        self.WithEffectContext(context, Self::DetachElementForEffect);
    }

    /// Source `Animation::{Attach,Detach}Element` body for a live effect
    /// borrow. ScrollOffsetAnimationsImpl uses the same body while an event
    /// callback already owns the selected Animation's KeyframeEffect.
    pub(crate) fn AttachElementForEffect(
        effect: &mut KeyframeEffect<C>,
        element_id: ElementId,
        context: &mut dyn KeyframeEffectContext<C>,
    ) {
        effect.AttachElement(element_id);
        if context.AnimationHasHost() {
            let element = context.AnimationRegisterForElement(element_id);
            effect.BindElementAnimations(element, context);
        }
    }

    pub(crate) fn DetachElementForEffect(
        effect: &mut KeyframeEffect<C>,
        context: &mut dyn KeyframeEffectContext<C>,
    ) {
        debug_assert!(effect.has_attached_element());
        if context.AnimationHasHost() {
            let element_id = effect.element_id();
            context.AnimationUnregisterForElement(element_id);
            effect.UnbindElementAnimations(context);
        }
        effect.DetachElement();
    }
    fn RegisterAnimation<X: AnimationContext<C>>(&mut self, context: &mut X) {
        let host = self.animation_host_.expect("attached AnimationHost");
        debug_assert!(self.keyframe_effect_.has_attached_element());
        debug_assert!(!self.keyframe_effect_.has_bound_element_animations());
        let element = context.HostRegisterAnimationForElement(
            host,
            self.keyframe_effect_.element_id(),
            self.id_,
        );
        let mut effect_context = Self::EffectContext(
            self.id_,
            &self.animation_host_,
            &self.animation_timeline_,
            &self.animation_delegate_,
            context,
        );
        self.keyframe_effect_
            .BindElementAnimations(element, &mut effect_context);
    }
    fn UnregisterAnimation<X: AnimationContext<C>>(&mut self, context: &mut X) {
        let host = self.animation_host_.expect("attached AnimationHost");
        debug_assert!(self.keyframe_effect_.has_attached_element());
        debug_assert!(self.keyframe_effect_.has_bound_element_animations());
        context.HostUnregisterAnimationForElement(
            host,
            self.keyframe_effect_.element_id(),
            self.id_,
        );
        let mut effect_context = Self::EffectContext(
            self.id_,
            &self.animation_host_,
            &self.animation_timeline_,
            &self.animation_delegate_,
            context,
        );
        self.keyframe_effect_
            .UnbindElementAnimations(&mut effect_context);
    }

    fn EffectContext<'a, X: AnimationContext<C>>(
        id: i32,
        host: &'a Option<AnimationHostId>,
        timeline: &'a Option<i32>,
        delegate: &'a Option<AnimationDelegateId>,
        external: &'a mut X,
    ) -> EffectContext<'a, C, X> {
        EffectContext {
            id,
            host,
            timeline,
            delegate,
            external,
            _curve: std::marker::PhantomData,
        }
    }

    /// Lends the actual KeyframeEffect together with a split borrow of its
    /// Animation siblings. The callback cannot recover or re-lock Animation.
    pub(crate) fn WithEffectContext<R, X: AnimationContext<C>>(
        &mut self,
        context: &mut X,
        callback: impl FnOnce(&mut KeyframeEffect<C>, &mut dyn KeyframeEffectContext<C>) -> R,
    ) -> R {
        let mut effect_context = Self::EffectContext(
            self.id_,
            &self.animation_host_,
            &self.animation_timeline_,
            &self.animation_delegate_,
            context,
        );
        callback(&mut self.keyframe_effect_, &mut effect_context)
    }

    /// Rebuilds the same source sibling context from copied raw-pointer
    /// identities when a synchronous callback has already lent the effect.
    /// The external context is a separate call-scoped value, so the currently
    /// executing AnimationContext is never aliased or re-entered.
    pub fn WithExternalEffectContext<R, X: AnimationContext<C>>(
        id: i32,
        host: Option<AnimationHostId>,
        timeline: Option<i32>,
        delegate: Option<AnimationDelegateId>,
        external: &mut X,
        callback: impl FnOnce(&mut dyn KeyframeEffectContext<C>) -> R,
    ) -> R {
        let mut effect_context = Self::EffectContext(id, &host, &timeline, &delegate, external);
        callback(&mut effect_context)
    }

    // Source: cc/animation/animation.cc:141-191.
    pub fn PushPropertiesTo<X: AnimationContext<C>>(
        &mut self,
        animation_impl: &mut Self,
        context: &mut X,
    ) {
        let mut impl_start_time = None;
        if self.is_replacement_ && !self.keyframe_effect_.keyframe_models().is_empty() {
            let group = self.keyframe_effect_.keyframe_models()[0].group();
            animation_impl.keyframe_effect_.set_replaced_group(group);
        }
        if self.is_replacement_ && self.GetStartTime().is_none() {
            impl_start_time = animation_impl.GetStartTime();
            assert!(self.keyframe_effect_.needs_push_properties());
        }
        self.is_replacement_ = false;

        // PushPropertiesTo uses a special alternating source/target context.
        // The body is supplied below once KeyframeEffectPushContext is declared
        // by the same-stem effect translation.
        self.PushEffectProperties(animation_impl, impl_start_time, context);
    }
    fn PushEffectProperties<X: AnimationContext<C>>(
        &mut self,
        animation_impl: &mut Self,
        replaced_start_time: Option<TimeTicks>,
        context: &mut X,
    ) {
        let mut push = AnimationPushContext {
            source_id: self.id_,
            source_host: &self.animation_host_,
            source_timeline: &self.animation_timeline_,
            source_delegate: &self.animation_delegate_,
            target_id: animation_impl.id_,
            target_host: &animation_impl.animation_host_,
            target_timeline: &animation_impl.animation_timeline_,
            target_delegate: &animation_impl.animation_delegate_,
            external: context,
            _curve: std::marker::PhantomData,
        };
        self.keyframe_effect_.PushPropertiesTo(
            &mut animation_impl.keyframe_effect_,
            replaced_start_time,
            &mut push,
        );
    }
    pub fn Tick<X: AnimationContext<C>>(&mut self, tick_time: TimeTicks, context: &mut X) -> bool {
        debug_assert!(!self.IsWorkletAnimation());
        let mut effect_context = Self::EffectContext(
            self.id_,
            &self.animation_host_,
            &self.animation_timeline_,
            &self.animation_delegate_,
            context,
        );
        self.keyframe_effect_.Tick(tick_time, &mut effect_context)
    }
    pub fn IsScrollLinkedAnimation<X: AnimationContext<C>>(&self, context: &X) -> bool {
        self.animation_timeline_
            .is_some_and(|timeline| context.TimelineIsScrollTimeline(timeline))
    }
    pub fn UpdateState<X: AnimationContext<C>>(
        &mut self,
        start_ready_keyframe_models: bool,
        events: &mut AnimationEvents<C>,
        context: &mut X,
    ) {
        let mut effect_context = Self::EffectContext(
            self.id_,
            &self.animation_host_,
            &self.animation_timeline_,
            &self.animation_delegate_,
            context,
        );
        self.keyframe_effect_.UpdateState(
            start_ready_keyframe_models,
            Some(events),
            &mut effect_context,
        );
        self.keyframe_effect_
            .UpdateTickingState(&mut effect_context);
    }
    pub fn AddToTicking<X: AnimationContext<C>>(&self, context: &mut X) {
        context.HostAddToTicking(
            self.animation_host_.expect("attached AnimationHost"),
            self.id_,
        );
    }
    pub fn RemoveFromTicking<X: AnimationContext<C>>(&self, context: &mut X) {
        context.HostRemoveFromTicking(
            self.animation_host_.expect("attached AnimationHost"),
            self.id_,
        );
    }

    // Source: cc/animation/animation.cc:193-255.
    pub fn DispatchAndDelegateAnimationEvent<X: AnimationContext<C>>(
        &mut self,
        event: &AnimationPlaybackEvent<C>,
        context: &mut X,
    ) {
        let mut effect_context = Self::EffectContext(
            self.id_,
            &self.animation_host_,
            &self.animation_timeline_,
            &self.animation_delegate_,
            context,
        );
        if event.ShouldDispatchToKeyframeEffectAndModel()
            && !self
                .keyframe_effect_
                .DispatchAnimationEventToKeyframeModel(event, &mut effect_context)
        {
            return;
        }
        self.DelegateAnimationEvent(event, context);
    }
    fn DelegateAnimationEvent<X: AnimationContext<C>>(
        &self,
        event: &AnimationPlaybackEvent<C>,
        context: &mut X,
    ) {
        let Some(delegate) = self.animation_delegate_ else {
            return;
        };
        match event.kind {
            PlaybackType::kStarted => context.DelegateAnimationStarted(
                delegate,
                event.monotonic_time,
                event.target_property,
                event.group_id,
            ),
            PlaybackType::kFinished => context.DelegateAnimationFinished(
                delegate,
                event.monotonic_time,
                event.target_property,
                event.group_id,
            ),
            PlaybackType::kAborted => context.DelegateAnimationAborted(
                delegate,
                event.monotonic_time,
                event.target_property,
                event.group_id,
            ),
            PlaybackType::kTakeOver => {
                debug_assert!(!event.is_impl_only);
                debug_assert_eq!(event.target_property, TargetProperty::SCROLL_OFFSET as i32);
                context.DelegateAnimationTakeover(
                    delegate,
                    event.monotonic_time,
                    event.target_property,
                    event.animation_start_time,
                    event.curve.as_ref().expect("takeover curve").CloneCurve(),
                );
            }
            PlaybackType::kTimeUpdated => {
                debug_assert!(!event.is_impl_only);
                context.DelegateLocalTimeUpdated(delegate, event.local_time);
            }
        }
    }
    pub fn RequiresInvalidation(&self) -> bool {
        self.keyframe_effect_.RequiresInvalidation()
    }
    pub fn AffectsNativeProperty(&self) -> bool {
        self.keyframe_effect_.AffectsNativeProperty()
    }
    pub fn SetNeedsCommit<X: AnimationContext<C>>(&self, context: &mut X) {
        context.HostSetNeedsCommit(self.animation_host_.expect("attached AnimationHost"));
    }

    // Source: cc/animation/animation.cc:257-328.
    pub fn GetStartTime(&self) -> Option<TimeTicks> {
        let first = self.keyframe_effect_.keyframe_models().first()?;
        first.has_set_start_time().then(|| first.start_time())
    }
    pub fn SetStartTime(&mut self, start_time: TimeTicks) {
        for model in self.keyframe_effect_.keyframe_models_mut() {
            model.set_start_time(start_time);
            model.set_needs_synchronized_start_time(false);
        }
    }
    pub fn SetHoldTime(&mut self, hold_time: Option<TimeDelta>) {
        for model in self.keyframe_effect_.keyframe_models_mut() {
            model.set_hold_time(hold_time);
        }
    }
    pub fn GetPlaybackRate(&self) -> f64 {
        self.keyframe_effect_.keyframe_models()[0].playback_rate()
    }
    pub fn SetPlaybackRate(&mut self, playback_rate: f64) {
        for model in self.keyframe_effect_.keyframe_models_mut() {
            model.set_playback_rate(playback_rate);
        }
    }
    pub fn CalculateCurrentTime(&self, monotonic_time: TimeTicks) -> TimeDelta {
        let model = &self.keyframe_effect_.keyframe_models()[0];
        model.CalculateCurrentTime(
            monotonic_time,
            model.playback_rate(),
            model.StartShouldBeDeferred(),
        )
    }
    pub fn GetRunState(&self) -> RunState {
        self.keyframe_effect_.keyframe_models()[0].run_state()
    }
    pub fn SetRunState(&mut self, run_state: RunState) {
        for model in self.keyframe_effect_.keyframe_models_mut() {
            model.SetRunState(run_state);
        }
    }
    pub fn IsPaused(&self) -> bool {
        let model = &self.keyframe_effect_.keyframe_models()[0];
        gfx_model::KeyframeModel::<C>::IsPaused(model.run_state())
    }
    pub fn IsFinished(&self) -> bool {
        let model = &self.keyframe_effect_.keyframe_models()[0];
        self.keyframe_effect_
            .last_tick_time()
            .is_some_and(|last| model.is_finished() || model.IsFinishedAt(last))
    }

    // Source: cc/animation/animation.cc:330-391.
    pub fn Play(&mut self, monotonic_time: TimeTicks, auto_rewind: AutoRewind) {
        self.PlayInternal(monotonic_time, auto_rewind, self.GetPlaybackRate());
    }
    pub fn Reverse(&mut self, monotonic_time: TimeTicks, auto_rewind: AutoRewind) {
        self.PlayInternal(monotonic_time, auto_rewind, -self.GetPlaybackRate());
    }
    pub fn PlayInternal(
        &mut self,
        monotonic_time: TimeTicks,
        auto_rewind: AutoRewind,
        new_playback_rate: f64,
    ) {
        let old_current_time = self.CalculateCurrentTime(monotonic_time);
        let old_playback_rate = self.GetPlaybackRate();
        self.SetPlaybackRate(new_playback_rate);
        let is_running = matches!(self.GetRunState(), RunState::RUNNING | RunState::STARTING);
        let first = &self.keyframe_effect_.keyframe_models()[0];
        let is_finished = if new_playback_rate < 0.0 {
            old_current_time <= TimeDelta::default()
        } else {
            old_current_time >= first.CalculateEndTime()
        };
        let should_rewind = auto_rewind == AutoRewind::kForced
            || (auto_rewind == AutoRewind::kEnabled && is_finished);
        if !should_rewind && (is_running || is_finished) && old_playback_rate == new_playback_rate {
            return;
        }
        let new_current_time = if should_rewind {
            first.CalculateInitialHoldTime(new_playback_rate)
        } else {
            old_current_time
        };
        let start_time = monotonic_time - new_current_time / new_playback_rate;
        for model in self.keyframe_effect_.keyframe_models_mut() {
            model.SetRunState(RunState::RUNNING);
            model.set_start_time(start_time);
            model.set_hold_time(None);
            model.set_needs_synchronized_start_time(false);
        }
    }

    // Source: cc/animation/animation.cc:393-419.
    pub fn SetNeedsPushProperties<X: AnimationContext<C>>(&self, context: &mut X) {
        if let Some(timeline) = self.animation_timeline_ {
            context.TimelineSetNeedsPushProperties(timeline);
        }
    }
    pub fn ActivateKeyframeModels<X: AnimationContext<C>>(&mut self, context: &mut X) {
        let mut effect_context = Self::EffectContext(
            self.id_,
            &self.animation_host_,
            &self.animation_timeline_,
            &self.animation_delegate_,
            context,
        );
        self.keyframe_effect_
            .ActivateKeyframeModels(&mut effect_context);
        self.keyframe_effect_
            .UpdateTickingState(&mut effect_context);
    }
    pub fn GetKeyframeModel(&self, target_property: TargetProperty) -> Option<&KeyframeModel<C>> {
        self.keyframe_effect_
            .GetKeyframeModel(target_property as i32)
    }
    pub fn ToString(&self) -> String {
        let models = self
            .keyframe_effect_
            .keyframe_models()
            .iter()
            .map(|model| model.ToString())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "Animation{{id={}, element_id={}, keyframe_models=[{}]}}",
            self.id_,
            self.keyframe_effect_.element_id().ToString(),
            models
        )
    }
    pub fn IsWorkletAnimation(&self) -> bool {
        false
    }
    // Source: cc/animation/animation.h:119. The base Animation virtual has an
    // intentionally empty body; worklet subclasses provide their own event.
    pub fn TakeTimeUpdatedEvent(&self, _events: &mut AnimationEvents<C>) {}
    pub fn set_is_replacement(&mut self) {
        self.is_replacement_ = true;
    }

    // Source: cc/animation/animation.cc:421-460.
    pub fn AddKeyframeModel<X: AnimationContext<C>>(
        &mut self,
        model: Box<KeyframeModel<C>>,
        context: &mut X,
    ) {
        let mut effect_context = Self::EffectContext(
            self.id_,
            &self.animation_host_,
            &self.animation_timeline_,
            &self.animation_delegate_,
            context,
        );
        self.keyframe_effect_
            .AddKeyframeModel(model, &mut effect_context);
    }
    pub fn PauseKeyframeModelForTesting<X: AnimationContext<C>>(
        &mut self,
        model_id: i32,
        hold_time: TimeDelta,
        context: &mut X,
    ) {
        let mut effect_context = Self::EffectContext(
            self.id_,
            &self.animation_host_,
            &self.animation_timeline_,
            &self.animation_delegate_,
            context,
        );
        self.keyframe_effect_.PauseKeyframeModelForTesting(
            model_id,
            hold_time,
            &mut effect_context,
        );
    }
    pub fn Pause<X: AnimationContext<C>>(
        &mut self,
        hold_time: TimeDelta,
        run_state: RunState,
        context: &mut X,
    ) {
        let mut effect_context = Self::EffectContext(
            self.id_,
            &self.animation_host_,
            &self.animation_timeline_,
            &self.animation_delegate_,
            context,
        );
        self.keyframe_effect_
            .Pause(hold_time, run_state, &mut effect_context);
    }
    pub fn RemoveKeyframeModel<X: AnimationContext<C>>(&mut self, model_id: i32, context: &mut X) {
        let mut effect_context = Self::EffectContext(
            self.id_,
            &self.animation_host_,
            &self.animation_timeline_,
            &self.animation_delegate_,
            context,
        );
        self.keyframe_effect_
            .RemoveKeyframeModel(model_id, &mut effect_context);
    }
    pub fn AbortKeyframeModel<X: AnimationContext<C>>(&mut self, model_id: i32, context: &mut X) {
        let mut effect_context = Self::EffectContext(
            self.id_,
            &self.animation_host_,
            &self.animation_timeline_,
            &self.animation_delegate_,
            context,
        );
        self.keyframe_effect_
            .AbortKeyframeModel(model_id, &mut effect_context);
    }
    pub fn AbortKeyframeModelsWithProperty<X: AnimationContext<C>>(
        &mut self,
        target_property: TargetProperty,
        needs_completion: bool,
        context: &mut X,
    ) {
        let mut effect_context = Self::EffectContext(
            self.id_,
            &self.animation_host_,
            &self.animation_timeline_,
            &self.animation_delegate_,
            context,
        );
        self.keyframe_effect_.AbortKeyframeModelsWithProperty(
            target_property as i32,
            needs_completion,
            &mut effect_context,
        );
    }
    pub fn NotifyKeyframeModelFinishedForTesting<X: AnimationContext<C>>(
        &mut self,
        timeline_id: i32,
        model_id: i32,
        target_property: TargetProperty,
        group_id: i32,
        context: &mut X,
    ) {
        let event = AnimationPlaybackEvent::new(
            PlaybackType::kFinished,
            UniqueKeyframeModelId {
                timeline_id,
                animation_id: self.id(),
                model_id,
            },
            group_id,
            target_property as i32,
            TimeTicks::default(),
        );
        self.DispatchAndDelegateAnimationEvent(&event, context);
    }
}

impl<C: KeyframeCurve + AnimationEventCurve> Drop for Animation<C> {
    // Source: cc/animation/animation.cc:34-36.
    fn drop(&mut self) {
        debug_assert!(self.animation_timeline_.is_none());
    }
}

// Source: cc/animation/keyframe_effect.h:38-39.
pub const fn reserved_element_id_for_paint_worklet() -> ElementId {
    ElementId::new(u64::MAX - 1)
}

/// Alternating source/target callback carrier used by Effect property push.
/// It never exposes both mutable contexts at once and therefore cannot create
/// an owner cycle or a second mutable view of either Animation.
struct AnimationPushContext<'a, C, X>
where
    C: KeyframeCurve + AnimationEventCurve,
    X: AnimationContext<C>,
{
    source_id: i32,
    source_host: &'a Option<AnimationHostId>,
    source_timeline: &'a Option<i32>,
    source_delegate: &'a Option<AnimationDelegateId>,
    target_id: i32,
    target_host: &'a Option<AnimationHostId>,
    target_timeline: &'a Option<i32>,
    target_delegate: &'a Option<AnimationDelegateId>,
    external: &'a mut X,
    _curve: std::marker::PhantomData<C>,
}

impl<C, X> KeyframeEffectPushContext<C> for AnimationPushContext<'_, C, X>
where
    C: KeyframeCurve + AnimationEventCurve,
    X: AnimationContext<C>,
{
    fn WithSourceContext(&mut self, callback: &mut dyn FnMut(&mut dyn KeyframeEffectContext<C>)) {
        let mut context = EffectContext {
            id: self.source_id,
            host: self.source_host,
            timeline: self.source_timeline,
            delegate: self.source_delegate,
            external: &mut *self.external,
            _curve: std::marker::PhantomData,
        };
        callback(&mut context);
    }
    fn WithTargetContext(&mut self, callback: &mut dyn FnMut(&mut dyn KeyframeEffectContext<C>)) {
        let mut context = EffectContext {
            id: self.target_id,
            host: self.target_host,
            timeline: self.target_timeline,
            delegate: self.target_delegate,
            external: &mut *self.external,
            _curve: std::marker::PhantomData,
        };
        callback(&mut context);
    }
}
