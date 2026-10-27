// Translation status: complete
// Source: cc/animation/keyframe_effect.h:1-238
// Source: cc/animation/keyframe_effect.cc:1-1183
use super::{
    animation_events::{AnimationEvents, AnimationPlaybackEvent},
    keyframe_model::{KeyframeCurve, KeyframeModel, RunState},
};
use crate::gfx_keyframe_effect as gfx;
use compositor_scheduler::{TimeDelta, TimeTicks};
use compositor_tree::{geometry::PointF, ElementId};
use std::sync::Arc;
/// Native ElementAnimations allocation: each bound effect retains the same
/// strong lease that the source scoped_refptr retains. No host pointer is kept.
pub trait BoundElementAnimations: Send + Sync {
    fn element_id(&self) -> ElementId;
}
/// A short borrow of Animation's actual siblings, host and element capabilities.
/// Implementations cannot re-lock the current Animation while lending its effect.
pub trait KeyframeEffectContext<C: KeyframeCurve> {
    fn Now(&self) -> TimeTicks;
    fn TickCurve(
        &mut self,
        curve: &C,
        time: TimeDelta,
        property: i32,
        model: &KeyframeModel<C>,
        limit: super::keyframe_model::LimitDirection,
    );
    fn AnimationRegisterForElement(
        &mut self,
        element: ElementId,
    ) -> Arc<dyn BoundElementAnimations>;
    fn AnimationUnregisterForElement(&mut self, element: ElementId);
    fn ElementAttachToCurve(&mut self, element: &Arc<dyn BoundElementAnimations>, curve: &mut C);
    fn UpdateClientAnimationStateForElement(&mut self, element: ElementId);
    fn AnimationId(&self) -> i32;
    fn AnimationTimelineId(&self) -> Option<i32>;
    fn AnimationHasHost(&self) -> bool;
    fn AnimationIsScrollLinked(&self) -> bool;
    fn AnimationSetNeedsPushProperties(&mut self);
    fn AnimationSetNeedsCommit(&mut self);
    fn AnimationAddToTicking(&mut self);
    fn AnimationRemoveFromTicking(&mut self);
    fn ElementSetNeedsPushProperties(&mut self, element: &Arc<dyn BoundElementAnimations>);
    fn ElementUpdateClientAnimationState(&mut self, element: &Arc<dyn BoundElementAnimations>);
    fn ElementScrollOffsetForAnimation(
        &self,
        element: &Arc<dyn BoundElementAnimations>,
    ) -> Option<PointF>;
    fn DispatchAndDelegateAnimationEvent(
        &mut self,
        event: &AnimationPlaybackEvent<C>,
        receiver: &mut KeyframeEffect<C>,
    );
    fn TraceEffect(&self, event: &'static str, message: String);
}
/// Each operation lends only the actual source or target Animation siblings.
pub trait KeyframeEffectPushContext<C: KeyframeCurve> {
    fn WithSourceContext(&mut self, callback: &mut dyn FnMut(&mut dyn KeyframeEffectContext<C>));
    fn WithTargetContext(&mut self, callback: &mut dyn FnMut(&mut dyn KeyframeEffectContext<C>));
}
// Source: cc/animation/keyframe_effect.h:195-233. The state subobject only
// permits splitting borrows of source fields from the inherited model vector.
pub struct KeyframeEffect<C: KeyframeCurve> {
    base_: gfx::KeyframeEffect<KeyframeModel<C>>,
    state_: KeyframeEffectState,
}
pub struct KeyframeEffectState {
    animation_id_: i32,
    element_id_: ElementId,
    element_animations_: Option<Arc<dyn BoundElementAnimations>>,
    replaced_group_: Option<i32>,
    needs_to_start_keyframe_models_: bool,
    scroll_offset_animation_was_interrupted_: bool,
    is_ticking_: bool,
    awaiting_deletion_: bool,
    last_tick_time_: Option<TimeTicks>,
    needs_push_properties_: bool,
}
impl<C: KeyframeCurve> KeyframeEffect<C> {
    // Source: cc/animation/keyframe_effect.cc:56-63
    pub fn new(animation_id: i32) -> Self {
        Self {
            base_: gfx::KeyframeEffect::new(),
            state_: KeyframeEffectState {
                animation_id_: animation_id,
                element_id_: ElementId::default(),
                element_animations_: None,
                replaced_group_: None,
                needs_to_start_keyframe_models_: false,
                scroll_offset_animation_was_interrupted_: false,
                is_ticking_: false,
                awaiting_deletion_: false,
                last_tick_time_: None,
                needs_push_properties_: false,
            },
        }
    }
    // Source: cc/animation/keyframe_effect.h:60-83,174-185
    pub fn element_animations(&self) -> Option<&Arc<dyn BoundElementAnimations>> {
        self.element_animations_.as_ref()
    }
    pub fn has_bound_element_animations(&self) -> bool {
        self.element_animations_.is_some()
    }
    pub fn has_attached_element(&self) -> bool {
        self.element_id_.IsValid()
    }
    pub fn element_id(&self) -> ElementId {
        self.element_id_
    }
    pub fn has_any_keyframe_model(&self) -> bool {
        !self.base_.keyframe_models().is_empty()
    }
    pub fn scroll_offset_animation_was_interrupted(&self) -> bool {
        self.scroll_offset_animation_was_interrupted_
    }
    pub fn needs_push_properties(&self) -> bool {
        self.needs_push_properties_
    }
    pub fn awaiting_deletion(&self) -> bool {
        self.awaiting_deletion_
    }
    pub fn set_replaced_group(&mut self, group: i32) {
        self.replaced_group_ = Some(group);
    }
    pub fn last_tick_time(&self) -> Option<TimeTicks> {
        self.last_tick_time_
    }
    pub fn keyframe_models(&self) -> &Vec<Box<KeyframeModel<C>>> {
        self.base_.keyframe_models()
    }
    pub fn keyframe_models_mut(&mut self) -> &mut Vec<Box<KeyframeModel<C>>> {
        self.base_.keyframe_models_mut()
    }
    // Source: cc/animation/keyframe_effect.cc:69-87
    pub fn SetNeedsPushProperties(
        &mut self,
        context: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        SetNeedsPushPropertiesForParts(&mut self.state_, context);
    }
    pub fn ResetNeedsPushProperties(&mut self) {
        self.needs_push_properties_ = false;
    }
    // Source: cc/animation/keyframe_effect.cc:108-117
    pub fn AttachElement(&mut self, id: ElementId) {
        debug_assert!(!self.element_id_.IsValid());
        debug_assert!(id.IsValid());
        self.element_id_ = id;
    }
    pub fn DetachElement(&mut self) {
        debug_assert!(self.element_id_.IsValid());
        self.element_id_ = ElementId::default();
    }
}
// Source: cc/animation/keyframe_effect.cc:65-67. Unbinding requires a live
// context and is performed by the actual Animation owner before final drop.
impl<C: KeyframeCurve> Drop for KeyframeEffect<C> {
    fn drop(&mut self) {
        debug_assert!(!self.element_animations_.is_some());
    }
}

impl<C: KeyframeCurve> std::ops::Deref for KeyframeEffect<C> {
    type Target = KeyframeEffectState;
    fn deref(&self) -> &Self::Target {
        &self.state_
    }
}
impl<C: KeyframeCurve> std::ops::DerefMut for KeyframeEffect<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state_
    }
}
use super::animation_events::{Event, PlaybackType, UniqueKeyframeModelId};
use crate::{property_animation_state::PropertyAnimationState, ElementListType};
use compositor_tree::target_property::Type as Property;
use std::ops::Range;
// Source: cc/animation/keyframe_effect.cc:29-51.
fn NeedsFinishedEvent<C: KeyframeCurve>(model: &KeyframeModel<C>) -> bool {
    !model.is_controlling_instance() && !model.received_finished_event()
}
fn FindAnimationsWithSameGroupId<C: KeyframeCurve>(
    models: &[Box<KeyframeModel<C>>],
    group: i32,
) -> Vec<usize> {
    models
        .iter()
        .enumerate()
        .filter_map(|(i, m)| (m.group() == group).then_some(i))
        .collect()
}
// Source: cc/animation/keyframe_effect.cc:69-83.
fn SetNeedsPushPropertiesForParts<C: KeyframeCurve>(
    state: &mut KeyframeEffectState,
    c: &mut (impl KeyframeEffectContext<C> + ?Sized),
) {
    state.needs_push_properties_ = true;
    if let Some(element) = &state.element_animations_ {
        c.ElementSetNeedsPushProperties(element)
    }
    c.AnimationSetNeedsPushProperties();
}
// Source: cc/animation/keyframe_effect.cc:143-149.
fn RemoveFromTickingForParts<C: KeyframeCurve>(
    state: &mut KeyframeEffectState,
    c: &mut (impl KeyframeEffectContext<C> + ?Sized),
) {
    state.is_ticking_ = false;
    state.last_tick_time_ = None;
    c.AnimationRemoveFromTicking();
}
// Source: cc/animation/keyframe_effect.cc:177-215.
fn UpdateTickingStateForParts<C: KeyframeCurve>(
    models: &[Box<KeyframeModel<C>>],
    state: &mut KeyframeEffectState,
    c: &mut (impl KeyframeEffectContext<C> + ?Sized),
) {
    if !c.AnimationHasHost() {
        return;
    }
    let was = state.is_ticking_;
    state.is_ticking_ = false;
    for model in models {
        if model.run_state() != RunState::WAITING_FOR_DELETION {
            state.is_ticking_ = true;
            state.awaiting_deletion_ = false;
            break;
        }
    }
    if was && !state.is_ticking_ {
        for model in models {
            if model.run_state() == RunState::WAITING_FOR_DELETION
                && model.is_controlling_instance()
                && !model.is_impl_only()
            {
                state.awaiting_deletion_ = true;
                state.is_ticking_ = true;
                break;
            }
        }
    }
    if !was && state.is_ticking_ {
        c.AnimationAddToTicking()
    } else if was && !state.is_ticking_ {
        RemoveFromTickingForParts(state, c)
    }
}
// Source: cc/animation/keyframe_effect.cc:382-393.
fn KeyframeModelAddedForParts<C: KeyframeCurve>(
    models: &mut Vec<Box<KeyframeModel<C>>>,
    state: &mut KeyframeEffectState,
    c: &mut (impl KeyframeEffectContext<C> + ?Sized),
) {
    assert!(state.element_animations_.is_some());
    c.AnimationSetNeedsCommit();
    state.needs_to_start_keyframe_models_ = true;
    UpdateTickingStateForParts(models, state, c);
    for model in models {
        c.ElementAttachToCurve(
            state.element_animations_.as_ref().unwrap(),
            model.curve_mut().unwrap(),
        )
    }
    c.ElementUpdateClientAnimationState(state.element_animations_.as_ref().unwrap());
}
// Source: cc/animation/keyframe_effect.cc:231-287.
fn AddModelForParts<C: KeyframeCurve>(
    models: &mut Vec<Box<KeyframeModel<C>>>,
    state: &mut KeyframeEffectState,
    model: Box<KeyframeModel<C>>,
    c: &mut (impl KeyframeEffectContext<C> + ?Sized),
) {
    debug_assert!(
        !model.is_impl_only() || model.TargetProperty() == Property::SCROLL_OFFSET as i32
    );
    #[cfg(debug_assertions)]
    for other in models.iter() {
        if other.group() == model.group() && other.TargetProperty() == model.TargetProperty() {
            if model.TargetProperty() == Property::NATIVE_PROPERTY as i32
                && other.native_property_type() != model.native_property_type()
            {
                continue;
            }
            debug_assert!(!(other.affects_active_elements() && model.affects_active_elements()));
            debug_assert!(!(other.affects_pending_elements() && model.affects_pending_elements()));
        }
    }
    #[cfg(debug_assertions)]
    if model.TargetProperty() == Property::SCROLL_OFFSET as i32 {
        debug_assert!(!models
            .iter()
            .any(|m| m.TargetProperty() == Property::SCROLL_OFFSET as i32
                && !m.is_finished()
                && (!m.is_controlling_instance() || m.affects_pending_elements())))
    }
    models.push(model);
    if state.element_animations_.is_some() {
        KeyframeModelAddedForParts(models, state, c);
        SetNeedsPushPropertiesForParts(state, c)
    }
}
// Source: cc/animation/keyframe_effect.cc:808-830.
fn RemoveRangeForParts<C: KeyframeCurve>(
    models: &mut Vec<Box<KeyframeModel<C>>>,
    state: &mut KeyframeEffectState,
    range: Range<usize>,
    c: &mut (impl KeyframeEffectContext<C> + ?Sized),
) {
    let mut removed = false;
    for m in &models[range.clone()] {
        if m.TargetProperty() == Property::SCROLL_OFFSET as i32 {
            if state.element_animations_.is_some() {
                state.scroll_offset_animation_was_interrupted_ = true
            }
        } else if !m.is_finished() {
            removed = true
        }
    }
    gfx::KeyframeEffect::RemoveKeyframeModelRange(models, range);
    if state.element_animations_.is_some() {
        UpdateTickingStateForParts(models, state, c);
        if removed {
            c.ElementUpdateClientAnimationState(state.element_animations_.as_ref().unwrap())
        }
        c.AnimationSetNeedsCommit();
        SetNeedsPushPropertiesForParts(state, c)
    }
}
struct EffectRuntimeBridge<'a, C: KeyframeCurve, S: ?Sized> {
    state: &'a mut KeyframeEffectState,
    context: &'a mut S,
    curve: std::marker::PhantomData<C>,
}
impl<C: KeyframeCurve, S: KeyframeEffectContext<C> + ?Sized> gfx::EffectRuntime<KeyframeModel<C>>
    for EffectRuntimeBridge<'_, C, S>
{
    fn Now(&self) -> TimeTicks {
        self.context.Now()
    }
    fn TickCurve(
        &mut self,
        curve: &C,
        time: TimeDelta,
        property: i32,
        model: &KeyframeModel<C>,
        limit: super::keyframe_model::LimitDirection,
    ) {
        self.context.TickCurve(curve, time, property, model, limit)
    }
    fn RemoveRange(&mut self, models: &mut Vec<Box<KeyframeModel<C>>>, range: Range<usize>) {
        RemoveRangeForParts(models, self.state, range, self.context)
    }
    fn AddModel(&mut self, models: &mut Vec<Box<KeyframeModel<C>>>, model: Box<KeyframeModel<C>>) {
        AddModelForParts(models, self.state, model, self.context)
    }
}
impl<C: KeyframeCurve> KeyframeEffect<C> {
    // Source: cc/animation/keyframe_effect.cc:89-106.
    pub fn BindElementAnimations(
        &mut self,
        element: Arc<dyn BoundElementAnimations>,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        debug_assert!(self.element_animations_.is_none());
        debug_assert!(self.element_id_.IsValid());
        self.state_.element_animations_ = Some(element);
        debug_assert_eq!(
            self.element_id_,
            self.element_animations_.as_ref().unwrap().element_id()
        );
        if self.has_any_keyframe_model() {
            self.KeyframeModelAdded(c)
        }
        self.SetNeedsPushProperties(c);
    }
    pub fn UnbindElementAnimations(&mut self, c: &mut (impl KeyframeEffectContext<C> + ?Sized)) {
        self.SetNeedsPushProperties(c);
        self.state_.element_animations_ = None
    }
    // Source: cc/animation/keyframe_effect.cc:119-140.
    pub fn Tick(
        &mut self,
        now: TimeTicks,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) -> bool {
        debug_assert!(self.has_bound_element_animations());
        if self.needs_to_start_keyframe_models_ {
            self.StartKeyframeModels(now)
        }
        let mut active = false;
        let mut became_inactive = false;
        for model in self.base_.keyframe_models() {
            let mut bridge = EffectRuntimeBridge {
                state: &mut self.state_,
                context: c,
                curve: std::marker::PhantomData,
            };
            gfx::KeyframeEffect::TickKeyframeModel(now, model.as_ref(), &mut bridge);
            let was = bridge
                .state
                .last_tick_time_
                .is_some_and(|t| model.HasActiveTime(t));
            let current = model.HasActiveTime(now);
            active |= current;
            became_inactive |= was && !current;
        }
        self.state_.last_tick_time_ = Some(now);
        c.ElementUpdateClientAnimationState(self.element_animations_.as_ref().unwrap());
        if became_inactive {
            c.AnimationSetNeedsCommit()
        }
        active
    }
    // Source: cc/animation/keyframe_effect.cc:143-215.
    pub fn RemoveFromTicking(&mut self, c: &mut (impl KeyframeEffectContext<C> + ?Sized)) {
        RemoveFromTickingForParts(&mut self.state_, c)
    }
    pub fn UpdateTickingState(&mut self, c: &mut (impl KeyframeEffectContext<C> + ?Sized)) {
        UpdateTickingStateForParts(self.base_.keyframe_models(), &mut self.state_, c)
    }
    pub fn UpdateState(
        &mut self,
        mut start_ready: bool,
        mut events: Option<&mut AnimationEvents<C>>,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        debug_assert!(self.has_bound_element_animations());
        if self.last_tick_time_.is_none() || self.awaiting_deletion_ {
            start_ready = false
        }
        if start_ready {
            self.PromoteStartedKeyframeModels(events.as_deref_mut(), c)
        }
        let now = self.last_tick_time_.unwrap_or_default();
        self.MarkFinishedKeyframeModels(now, c);
        self.MarkKeyframeModelsForDeletion(now, events.as_deref_mut(), c);
        self.PurgeKeyframeModelsMarkedForDeletion(true);
        if start_ready && self.needs_to_start_keyframe_models_ {
            self.StartKeyframeModels(now);
            self.PromoteStartedKeyframeModels(events, c)
        }
    }
    // Source: cc/animation/keyframe_effect.cc:217-229.
    pub fn Pause(
        &mut self,
        hold: TimeDelta,
        state: RunState,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        let mut paused = false;
        for model in self.base_.keyframe_models_mut() {
            model.Pause(hold, state);
            paused = true
        }
        if !paused || !self.has_bound_element_animations() {
            return;
        }
        c.AnimationSetNeedsCommit();
        self.SetNeedsPushProperties(c);
    }
    // Source: cc/animation/keyframe_effect.cc:231-287.
    pub fn AddKeyframeModel(
        &mut self,
        model: Box<KeyframeModel<C>>,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        AddModelForParts(self.base_.keyframe_models_mut(), &mut self.state_, model, c)
    }
    // Source: cc/animation/keyframe_effect.cc:290-318.
    pub fn PauseKeyframeModelForTesting(
        &mut self,
        id: i32,
        hold: TimeDelta,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        for model in self.base_.keyframe_models_mut() {
            if model.id() == id {
                model.Pause(hold, RunState::PAUSED)
            }
        }
        if !self.has_bound_element_animations() {
            return;
        }
        c.AnimationSetNeedsCommit();
        self.SetNeedsPushProperties(c)
    }
    pub fn AbortKeyframeModel(
        &mut self,
        id: i32,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        if let Some(model) = self.base_.GetKeyframeModelByIdMutable(id) {
            if !model.is_finished() {
                model.SetRunState(RunState::ABORTED);
                if let Some(element) = &self.state_.element_animations_ {
                    c.ElementUpdateClientAnimationState(element)
                }
            }
        }
        if !self.has_bound_element_animations() {
            return;
        }
        c.AnimationSetNeedsCommit();
        self.SetNeedsPushProperties(c)
    }
    // Source: cc/animation/keyframe_effect.cc:320-350.
    pub fn AbortKeyframeModelsWithProperty(
        &mut self,
        property: i32,
        needs_completion: bool,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        debug_assert!(!needs_completion || property == Property::SCROLL_OFFSET as i32);
        let mut aborted = false;
        for model in self.base_.keyframe_models_mut() {
            if model.TargetProperty() == property && !model.is_finished() {
                model.SetRunState(if needs_completion && model.is_impl_only() {
                    RunState::ABORTED_BUT_NEEDS_COMPLETION
                } else {
                    RunState::ABORTED
                });
                aborted = true
            }
        }
        if let Some(element) = &self.state_.element_animations_ {
            if aborted {
                c.ElementUpdateClientAnimationState(element)
            }
            c.AnimationSetNeedsCommit();
            self.SetNeedsPushProperties(c)
        }
    }
    // Source: cc/animation/keyframe_effect.cc:352-380.
    pub fn ActivateKeyframeModels(&mut self, c: &mut (impl KeyframeEffectContext<C> + ?Sized)) {
        debug_assert!(self.has_bound_element_animations());
        let mut activated = false;
        for model in self.base_.keyframe_models_mut() {
            if self.state_.replaced_group_ == Some(model.group())
                && !model.affects_pending_elements()
            {
                assert_ne!(model.group(), KeyframeModel::<C>::kInvalidGroup);
                model.ungroup()
            }
            if model.affects_active_elements() != model.affects_pending_elements() {
                activated = true
            }
            model.set_affects_active_elements(model.affects_pending_elements());
        }
        if activated {
            c.ElementUpdateClientAnimationState(self.element_animations_.as_ref().unwrap())
        }
        self.state_.replaced_group_ = None;
        self.state_.scroll_offset_animation_was_interrupted_ = false;
    }
    // Source: cc/animation/keyframe_effect.cc:382-393.
    pub fn KeyframeModelAdded(&mut self, c: &mut (impl KeyframeEffectContext<C> + ?Sized)) {
        KeyframeModelAddedForParts(self.base_.keyframe_models_mut(), &mut self.state_, c)
    }
    // Source: cc/animation/keyframe_effect.cc:395-460.
    pub fn DispatchAnimationEventToKeyframeModel(
        &mut self,
        event: &AnimationPlaybackEvent<C>,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) -> bool {
        debug_assert!(!event.is_impl_only);
        let mut index = self
            .base_
            .keyframe_models()
            .iter()
            .position(|m| m.id() == event.uid.model_id);
        let mut dispatched = false;
        match event.kind {
            PlaybackType::kStarted => {
                if index.is_none() {
                    if let Some(m) = self.base_.GetKeyframeModel(event.target_property) {
                        if m.group() == event.group_id {
                            index = self
                                .base_
                                .keyframe_models()
                                .iter()
                                .position(|other| other.id() == m.id())
                        }
                    }
                }
                if let Some(i) = index {
                    let model = &mut self.base_.keyframe_models_mut()[i];
                    if model.needs_synchronized_start_time() {
                        model.set_needs_synchronized_start_time(false);
                        if !model.has_set_start_time() {
                            {
                                let start = event.monotonic_time
                                    - model.hold_time().unwrap_or_default() / model.playback_rate();
                                model.set_start_time(start)
                            }
                        }
                        model.set_hold_time(None);
                        dispatched = true
                    }
                }
            }
            PlaybackType::kFinished => {
                if let Some(i) = index {
                    self.base_.keyframe_models_mut()[i].set_received_finished_event(true);
                    dispatched = true
                } else {
                    self.SetNeedsPushProperties(c)
                }
            }
            PlaybackType::kAborted => {
                if let Some(i) = index {
                    let model = &mut self.base_.keyframe_models_mut()[i];
                    model.SetRunState(RunState::ABORTED);
                    model.set_received_finished_event(true);
                    dispatched = true;
                    c.UpdateClientAnimationStateForElement(self.element_id_)
                }
            }
            PlaybackType::kTakeOver => {
                self.SetNeedsPushProperties(c);
                dispatched = true
            }
            PlaybackType::kTimeUpdated => {
                unreachable!("time updated events do not dispatch to models")
            }
        }
        dispatched
    }
    // Source: cc/animation/keyframe_effect.cc:462-501.
    pub fn HasTickingKeyframeModel(&self) -> bool {
        self.base_
            .keyframe_models()
            .iter()
            .any(|m| !m.is_finished())
    }
    pub fn RequiresInvalidation(&self) -> bool {
        self.base_.keyframe_models().iter().any(|m| {
            m.TargetProperty() == Property::NATIVE_PROPERTY as i32
                || m.TargetProperty() == Property::CSS_CUSTOM_PROPERTY as i32
        })
    }
    pub fn AffectsNativeProperty(&self) -> bool {
        self.base_.keyframe_models().iter().any(|m| {
            m.TargetProperty() != Property::NATIVE_PROPERTY as i32
                && m.TargetProperty() != Property::CSS_CUSTOM_PROPERTY as i32
        })
    }
    pub fn AnimationsPreserveAxisAlignment(&self) -> bool {
        for m in self.base_.keyframe_models() {
            if m.is_finished() {
                continue;
            }
            if !m.curve().unwrap().PreservesAxisAlignment() {
                return false;
            }
        }
        true
    }
    // Source: cc/animation/keyframe_effect.cc:503-530.
    pub fn MaximumScale(&self, element: ElementId, list: ElementListType) -> f32 {
        let mut maximum = 0.0f32;
        for m in self.base_.keyframe_models() {
            if m.is_finished() {
                continue;
            }
            let id = if m.element_id().IsValid() {
                m.element_id()
            } else {
                self.element_id_
            };
            if id != element {
                continue;
            }
            if (list == ElementListType::ACTIVE && !m.affects_active_elements())
                || (list == ElementListType::PENDING && !m.affects_pending_elements())
            {
                continue;
            }
            let mut candidate = 0.0;
            if m.curve().unwrap().MaximumScale(&mut candidate) {
                maximum = maximum.max(candidate)
            }
        }
        maximum
    }
    // Source: cc/animation/keyframe_effect.cc:532-568.
    pub fn IsPotentiallyAnimatingProperty(&self, property: i32, list: ElementListType) -> bool {
        self.base_.keyframe_models().iter().any(|m| {
            !m.is_finished()
                && m.TargetProperty() == property
                && ((list == ElementListType::ACTIVE && m.affects_active_elements())
                    || (list == ElementListType::PENDING && m.affects_pending_elements()))
        })
    }
    pub fn IsCurrentlyAnimatingProperty(&self, property: i32, list: ElementListType) -> bool {
        self.base_.keyframe_models().iter().any(|m| {
            !m.is_finished()
                && m.InEffect(self.last_tick_time_.unwrap_or_default())
                && m.TargetProperty() == property
                && ((list == ElementListType::ACTIVE && m.affects_active_elements())
                    || (list == ElementListType::PENDING && m.affects_pending_elements()))
        })
    }
    // Source: cc/animation/keyframe_effect.cc:570-597.
    pub fn GetPropertyAnimationState(
        &self,
        pending: &mut PropertyAnimationState,
        active: &mut PropertyAnimationState,
    ) {
        pending.Clear();
        active.Clear();
        for m in self.base_.keyframe_models() {
            if !m.is_finished() {
                let in_effect = m.InEffect(self.last_tick_time_.unwrap_or_default());
                let p = m.TargetProperty() as usize;
                if m.affects_pending_elements() {
                    pending.potentially_animating.set(p, true);
                    if in_effect {
                        pending.currently_running.set(p, true)
                    }
                }
                if m.affects_active_elements() {
                    active.potentially_animating.set(p, true);
                    if in_effect {
                        active.currently_running.set(p, true)
                    }
                }
            }
        }
    }
}
impl<C: KeyframeCurve> KeyframeEffect<C> {
    // Source: cc/animation/keyframe_effect.cc:599-620.
    pub fn MarkAbortedKeyframeModelsForDeletion(
        &mut self,
        target: &mut Self,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        let mut aborted = false;
        for other in target.base_.keyframe_models_mut() {
            if let Some(model) = self.base_.GetKeyframeModelByIdMutable(other.id()) {
                if model.run_state() == RunState::ABORTED {
                    other.SetRunState(RunState::WAITING_FOR_DELETION);
                    model.SetRunState(RunState::WAITING_FOR_DELETION);
                    aborted = true
                }
            }
        }
        if let Some(element) = &self.state_.element_animations_ {
            if aborted {
                c.ElementSetNeedsPushProperties(element)
            }
        }
    }
    // Source: cc/animation/keyframe_effect.cc:622-638.
    pub fn PurgeKeyframeModelsMarkedForDeletion(&mut self, impl_only: bool) {
        self.base_.keyframe_models_mut().retain(|m| {
            !(m.run_state() == RunState::WAITING_FOR_DELETION && (!impl_only || m.is_impl_only()))
        })
    }
    fn PurgeDeletedKeyframeModels(&mut self) {
        self.base_.keyframe_models_mut().retain(|m| {
            !(m.run_state() == RunState::WAITING_FOR_DELETION && !m.affects_pending_elements())
        })
    }
    // Source: cc/animation/keyframe_effect.cc:640-678. Native const owns mutable
    // curve pointers; only the explicit source CreateImplInstance clones a model.
    pub fn PushNewKeyframeModelsToImplThread(
        &mut self,
        target: &mut Self,
        push: &mut (impl KeyframeEffectPushContext<C> + ?Sized),
    ) {
        for model in self.base_.keyframe_models_mut() {
            if model.is_finished() {
                continue;
            }
            if target.base_.GetKeyframeModelById(model.id()).is_some() {
                continue;
            }
            if model.TargetProperty() == Property::SCROLL_OFFSET as i32
                && !model.curve().unwrap().ScrollHasSetInitialValue()
            {
                let mut offset = None;
                push.WithTargetContext(&mut |c| offset = target.ScrollOffsetForAnimation(c));
                if offset.is_none() {
                    push.WithSourceContext(&mut |c| {
                        offset = self
                            .state_
                            .element_animations_
                            .as_ref()
                            .and_then(|e| c.ElementScrollOffsetForAnimation(e))
                    })
                }
                debug_assert!(offset.is_some());
                model
                    .curve_mut()
                    .unwrap()
                    .ScrollSetInitialValue(offset.unwrap());
            }
            let mut next = model.CreateImplInstance(RunState::WAITING_FOR_TARGET_AVAILABILITY);
            debug_assert!(!next.needs_synchronized_start_time());
            next.set_affects_active_elements(false);
            let mut next = Some(Box::new(next));
            push.WithTargetContext(&mut |c| target.AddKeyframeModel(next.take().unwrap(), c));
        }
    }
    // Source: cc/animation/keyframe_effect.cc:680-719.
    pub fn RemoveKeyframeModelsCompletedOnMainThread(
        &self,
        target: &mut Self,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        let mut completed = false;
        for model in target.base_.keyframe_models_mut() {
            let done = if model.is_impl_only() {
                model.run_state() == RunState::WAITING_FOR_DELETION
            } else {
                self.base_
                    .GetKeyframeModelById(model.id())
                    .is_none_or(|main| main.is_finished())
            };
            if done {
                model.set_affects_pending_elements(false);
                completed = true
            }
        }
        target.PurgeDeletedKeyframeModels();
        if let Some(element) = &self.state_.element_animations_ {
            if completed {
                c.ElementSetNeedsPushProperties(element)
            }
        }
    }
    // Source: cc/animation/keyframe_effect.cc:721-783. Source/target callbacks
    // borrow their respective actual Animation siblings one operation at a time.
    pub fn PushPropertiesTo(
        &mut self,
        target: &mut Self,
        replaced_start: Option<TimeTicks>,
        push: &mut (impl KeyframeEffectPushContext<C> + ?Sized),
    ) {
        if !self.needs_push_properties_ {
            return;
        }
        self.state_.needs_push_properties_ = false;
        push.WithTargetContext(&mut |c| target.SetNeedsPushProperties(c));
        if self.element_id_ != target.element_id_ {
            if target.has_attached_element() {
                push.WithTargetContext(&mut |c| {
                    if c.AnimationHasHost() {
                        c.AnimationUnregisterForElement(target.element_id_);
                        target.UnbindElementAnimations(c)
                    }
                    target.DetachElement()
                })
            }
            if self.element_id_.IsValid() {
                push.WithTargetContext(&mut |c| {
                    target.AttachElement(self.element_id_);
                    if c.AnimationHasHost() {
                        let lease = c.AnimationRegisterForElement(self.element_id_);
                        target.BindElementAnimations(lease, c)
                    }
                })
            }
        }
        target.state_.scroll_offset_animation_was_interrupted_ =
            self.scroll_offset_animation_was_interrupted_;
        self.state_.scroll_offset_animation_was_interrupted_ = false;
        if !self.has_any_keyframe_model() && !target.has_any_keyframe_model() {
            return;
        }
        if let Some(start) = replaced_start {
            for model in self.base_.keyframe_models_mut() {
                model.set_start_time(start);
                model.set_hold_time(None)
            }
        }
        push.WithSourceContext(&mut |c| self.MarkAbortedKeyframeModelsForDeletion(target, c));
        self.PurgeKeyframeModelsMarkedForDeletion(false);
        push.WithSourceContext(&mut |c| self.RemoveKeyframeModelsCompletedOnMainThread(target, c));
        self.PushNewKeyframeModelsToImplThread(target, push);
        for model in self.base_.keyframe_models() {
            if let Some(other) = target.base_.GetKeyframeModelByIdMutable(model.id()) {
                model.PushPropertiesTo(other)
            }
        }
        push.WithTargetContext(&mut |c| target.UpdateTickingState(c));
    }
    // Source: cc/animation/keyframe_effect.cc:785-806.
    pub fn KeyframeModelsToString(&self) -> String {
        let mut out = String::new();
        for (i, m) in self.base_.keyframe_models().iter().enumerate() {
            if i > 0 {
                out.push_str(", ")
            }
            out.push_str(&m.ToString())
        }
        out
    }
    pub fn MinimumTickInterval(&self) -> TimeDelta {
        let mut minimum = TimeDelta::Max();
        for model in self.base_.keyframe_models() {
            let interval = model.curve().unwrap().TickInterval();
            if interval.is_zero() {
                return interval;
            }
            if interval < minimum {
                minimum = interval
            }
        }
        minimum
    }
    // Source: cc/animation/keyframe_effect.h:186-190; .cc:808-830.
    pub fn RemoveKeyframeModelRange(
        &mut self,
        range: Range<usize>,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        RemoveRangeForParts(self.base_.keyframe_models_mut(), &mut self.state_, range, c)
    }
    // Source: inherited gfx Remove entrypoints dispatch to this same override body.
    pub fn RemoveKeyframeModel(
        &mut self,
        id: i32,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        let mut bridge = EffectRuntimeBridge {
            state: &mut self.state_,
            context: c,
            curve: std::marker::PhantomData,
        };
        self.base_.RemoveKeyframeModel(id, &mut bridge)
    }
    pub fn RemoveAllKeyframeModels(&mut self, c: &mut (impl KeyframeEffectContext<C> + ?Sized)) {
        let mut bridge = EffectRuntimeBridge {
            state: &mut self.state_,
            context: c,
            curve: std::marker::PhantomData,
        };
        self.base_.RemoveAllKeyframeModels(&mut bridge)
    }
    pub fn RemoveKeyframeModels(
        &mut self,
        property: i32,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        let mut bridge = EffectRuntimeBridge {
            state: &mut self.state_,
            context: c,
            curve: std::marker::PhantomData,
        };
        self.base_.RemoveKeyframeModels(property, &mut bridge)
    }
    pub fn GetKeyframeModel(&self, property: i32) -> Option<&KeyframeModel<C>> {
        self.base_.GetKeyframeModel(property)
    }
    pub fn GetKeyframeModelById(&self, id: i32) -> Option<&KeyframeModel<C>> {
        self.base_.GetKeyframeModelById(id)
    }
    pub fn GetKeyframeModelByIdMutable(&mut self, id: i32) -> Option<&mut KeyframeModel<C>> {
        self.base_.GetKeyframeModelByIdMutable(id)
    }
    // Source: cc/animation/keyframe_effect.cc:832-936.
    fn StartKeyframeModels(&mut self, _now: TimeTicks) {
        debug_assert!(self.needs_to_start_keyframe_models_);
        self.state_.needs_to_start_keyframe_models_ = false;
        let mut active = [false; 32];
        let mut pending = [false; 32];
        let mut waiting = Vec::with_capacity(self.base_.keyframe_models().len());
        for (i, m) in self.base_.keyframe_models().iter().enumerate() {
            if matches!(m.run_state(), RunState::STARTING | RunState::RUNNING) {
                let p = m.TargetProperty() as usize;
                if m.affects_active_elements() {
                    active[p] = true
                }
                if m.affects_pending_elements() {
                    pending[p] = true
                }
            } else if m.run_state() == RunState::WAITING_FOR_TARGET_AVAILABILITY {
                waiting.push(i)
            }
        }
        for index in waiting {
            if self.base_.keyframe_models()[index].run_state()
                != RunState::WAITING_FOR_TARGET_AVAILABILITY
            {
                continue;
            }
            let model = &self.base_.keyframe_models()[index];
            let group = model.group();
            let mut enqueued = [false; 32];
            let mut affects_active = model.affects_active_elements();
            let mut affects_pending = model.affects_pending_elements();
            enqueued[model.TargetProperty() as usize] = true;
            for other in &self.base_.keyframe_models()[index + 1..] {
                if other.group() == group {
                    enqueued[other.TargetProperty() as usize] = true;
                    affects_active |= other.affects_active_elements();
                    affects_pending |= other.affects_pending_elements()
                }
            }
            let mut disjoint = true;
            for property in compositor_tree::target_property::ALL_TARGET_PROPERTIES {
                let p = property as usize;
                if enqueued[p] {
                    if affects_active {
                        if active[p] {
                            disjoint = false
                        } else {
                            active[p] = true
                        }
                    }
                    if affects_pending {
                        if pending[p] {
                            disjoint = false
                        } else {
                            pending[p] = true
                        }
                    }
                }
            }
            if disjoint {
                self.base_.keyframe_models_mut()[index].SetRunState(RunState::STARTING);
                for other in &mut self.base_.keyframe_models_mut()[index + 1..] {
                    if other.group() == group {
                        other.SetRunState(RunState::STARTING)
                    }
                }
            } else {
                self.state_.needs_to_start_keyframe_models_ = true
            }
        }
    }
    // Source: cc/animation/keyframe_effect.cc:938-974. Index cursors retain the
    // source original iteration range; each resident model borrow ends before
    // a synchronous delegate callback can mutate this same effect.
    fn PromoteStartedKeyframeModels(
        &mut self,
        mut events: Option<&mut AnimationEvents<C>>,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        for i in 0..self.base_.keyframe_models().len() {
            let model = &mut self.base_.keyframe_models_mut()[i];
            if model.run_state() == RunState::STARTING && model.affects_active_elements() {
                model.SetRunState(RunState::RUNNING);
                let mut adjusted = false;
                if !model.has_set_start_time() && !model.needs_synchronized_start_time() {
                    adjusted = model.hold_time().is_some();
                    let start = self.state_.last_tick_time_.unwrap_or_default()
                        - model.hold_time().unwrap_or_default() / model.playback_rate();
                    model.set_start_time(start);
                    model.set_hold_time(None)
                }
                let start = if model.has_set_start_time() && !adjusted {
                    model.start_time()
                } else {
                    self.state_.last_tick_time_.unwrap_or_default()
                };
                self.GenerateEvent(events.as_deref_mut(), i, PlaybackType::kStarted, start, c);
            }
        }
    }
    // Source: cc/animation/keyframe_effect.cc:976-1067.
    fn MarkKeyframeModelsForDeletion(
        &mut self,
        now: TimeTicks,
        mut events: Option<&mut AnimationEvents<C>>,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        let mut marked = false;
        for i in 0..self.base_.keyframe_models().len() {
            let model = &self.base_.keyframe_models()[i];
            if model.run_state() == RunState::ABORTED {
                self.GenerateEvent(events.as_deref_mut(), i, PlaybackType::kAborted, now, c);
                if !NeedsFinishedEvent(&self.base_.keyframe_models()[i]) {
                    self.base_.keyframe_models_mut()[i].SetRunState(RunState::WAITING_FOR_DELETION);
                    marked = true
                }
                continue;
            }
            if model.is_controlling_instance()
                && model.run_state() == RunState::ABORTED_BUT_NEEDS_COMPLETION
            {
                self.GenerateTakeoverEventForScrollAnimation(events.as_deref_mut(), i, now, c);
                self.base_.keyframe_models_mut()[i].SetRunState(RunState::WAITING_FOR_DELETION);
                marked = true;
                continue;
            }
            if model.run_state() != RunState::FINISHED {
                continue;
            }
            if NeedsFinishedEvent(model) {
                continue;
            }
            let same = FindAnimationsWithSameGroupId(self.base_.keyframe_models(), model.group());
            if same.iter().any(|&j| {
                let m = &self.base_.keyframe_models()[j];
                !m.is_finished() || (m.run_state() == RunState::FINISHED && NeedsFinishedEvent(m))
            }) {
                continue;
            }
            for j in same {
                if matches!(
                    self.base_.keyframe_models()[j].run_state(),
                    RunState::WAITING_FOR_DELETION | RunState::ABORTED
                ) {
                    continue;
                }
                self.GenerateEvent(events.as_deref_mut(), j, PlaybackType::kFinished, now, c);
                self.base_.keyframe_models_mut()[j].SetRunState(RunState::WAITING_FOR_DELETION);
                marked = true
            }
        }
        if marked {
            self.SetNeedsPushProperties(c)
        }
    }
    // Source: cc/animation/keyframe_effect.cc:1069-1105.
    fn MarkFinishedKeyframeModels(
        &mut self,
        now: TimeTicks,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        debug_assert!(self.has_bound_element_animations());
        let mut finished = false;
        for model in self.base_.keyframe_models_mut() {
            if !c.AnimationIsScrollLinked() && !model.is_finished() && model.IsFinishedAt(now) {
                model.SetRunState(RunState::FINISHED);
                finished = true;
                SetNeedsPushPropertiesForParts(&mut self.state_, c)
            }
            if !model.affects_active_elements()
                && !model.affects_pending_elements()
                && matches!(
                    model.run_state(),
                    RunState::WAITING_FOR_TARGET_AVAILABILITY
                        | RunState::STARTING
                        | RunState::RUNNING
                        | RunState::PAUSED
                        | RunState::PAUSED_EXCLUSIVE
                )
            {
                model.SetRunState(RunState::FINISHED);
                finished = true
            }
        }
        if finished {
            c.ElementUpdateClientAnimationState(self.element_animations_.as_ref().unwrap())
        }
    }
    // Source: cc/animation/keyframe_effect.cc:1107-1109.
    fn ScrollOffsetForAnimation(
        &self,
        c: &(impl KeyframeEffectContext<C> + ?Sized),
    ) -> Option<PointF> {
        c.ElementScrollOffsetForAnimation(self.element_animations_.as_ref().unwrap())
    }
}
impl<C: KeyframeCurve> KeyframeEffect<C> {
    // Source: cc/animation/keyframe_effect.cc:1111-1145. Event construction reads
    // only the actual resident model. Source-local event values outlive that short
    // borrow, allowing synchronous delegation to borrow this same effect safely.
    fn GenerateEvent(
        &mut self,
        events: Option<&mut AnimationEvents<C>>,
        index: usize,
        kind: PlaybackType,
        now: TimeTicks,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        let Some(events) = events else { return };
        let model = &self.base_.keyframe_models()[index];
        if model.group() == KeyframeModel::<C>::kInvalidGroup {
            return;
        }
        let mut event = AnimationPlaybackEvent::new(
            kind,
            UniqueKeyframeModelId {
                timeline_id: c.AnimationTimelineId().unwrap(),
                animation_id: c.AnimationId(),
                model_id: model.id(),
            },
            model.group(),
            model.TargetProperty(),
            now,
        );
        if model.is_impl_only() {
            event.is_impl_only = true;
            c.DispatchAndDelegateAnimationEvent(&event, self)
        } else {
            events.events_mut().push(Event::Playback(event))
        }
    }
    // Source: cc/animation/keyframe_effect.cc:1147-1181.
    fn GenerateTakeoverEventForScrollAnimation(
        &mut self,
        events: Option<&mut AnimationEvents<C>>,
        index: usize,
        now: TimeTicks,
        c: &mut (impl KeyframeEffectContext<C> + ?Sized),
    ) {
        let model = &self.base_.keyframe_models()[index];
        debug_assert_eq!(model.TargetProperty(), Property::SCROLL_OFFSET as i32);
        let Some(events) = events else { return };
        let uid = UniqueKeyframeModelId {
            timeline_id: c.AnimationTimelineId().unwrap(),
            animation_id: c.AnimationId(),
            model_id: model.id(),
        };
        let group = model.group();
        let property = model.TargetProperty();
        let mut takeover =
            AnimationPlaybackEvent::new(PlaybackType::kTakeOver, uid, group, property, now);
        takeover.animation_start_time = model.start_time();
        takeover.curve = Some(model.curve().unwrap().CloneCurve());
        events.events_mut().push(Event::Playback(takeover));
        let mut finished =
            AnimationPlaybackEvent::new(PlaybackType::kFinished, uid, group, property, now);
        finished.is_impl_only = true;
        c.DispatchAndDelegateAnimationEvent(&finished, self);
    }
}
