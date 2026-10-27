// Translation status: complete
// Source: cc/animation/keyframe_model.h:1-205
// Source: cc/animation/keyframe_model.cc:1-216
use super::animation_events::AnimationEventCurve;
use crate::gfx_keyframe_model as gfx;
use compositor_scheduler::{TimeDelta, TimeTicks};
use compositor_tree::{geometry::PointF, ElementId};
pub use gfx::{Direction, FillMode, LimitDirection, Phase, RunState};
#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicI32, Ordering};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};
// Source dependency: cc/paint/paint_worklet_input.h:24-28.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativePropertyType {
    kBackgroundColor,
    kClipPath,
    kInvalid,
}
// Source: cc/animation/keyframe_model.h:36-74
#[derive(Clone, Debug)]
pub struct TargetPropertyId {
    target_property_type_: i32,
    custom_property_name_: String,
    native_property_type_: NativePropertyType,
}
impl TargetPropertyId {
    // Source: cc/animation/keyframe_model.cc:44-63
    pub fn new(target_property_type: i32) -> Self {
        Self {
            target_property_type_: target_property_type,
            custom_property_name_: String::new(),
            native_property_type_: NativePropertyType::kInvalid,
        }
    }
    pub fn Custom(target_property_type: i32, name: &str) -> Self {
        Self {
            target_property_type_: target_property_type,
            custom_property_name_: name.to_string(),
            native_property_type_: NativePropertyType::kInvalid,
        }
    }
    pub fn Native(target_property_type: i32, kind: NativePropertyType) -> Self {
        Self {
            target_property_type_: target_property_type,
            custom_property_name_: String::new(),
            native_property_type_: kind,
        }
    }
    // Source: cc/animation/keyframe_model.h:56-63
    pub fn target_property_type(&self) -> i32 {
        self.target_property_type_
    }
    pub fn custom_property_name(&self) -> &str {
        &self.custom_property_name_
    }
    pub fn native_property_type(&self) -> NativePropertyType {
        self.native_property_type_
    }
}
pub trait KeyframeCurve: gfx::AnimationCurve + AnimationEventCurve {
    fn TypeName(&self) -> &'static str;
    // Source dependency: ui/gfx/animation/keyframe/animation_curve.cc:13-23.
    fn PreservesAxisAlignment(&self) -> bool {
        true
    }
    fn MaximumScale(&self, _maximum: &mut f32) -> bool {
        false
    }
    fn TickInterval(&self) -> TimeDelta {
        TimeDelta::default()
    }
    fn ScrollHasSetInitialValue(&self) -> bool;
    fn ScrollSetInitialValue(&mut self, value: PointF);
}
pub trait KeyframeModelTrace: Send + Sync {
    fn Begin(&self, model_id: u64, name: &str);
    fn End(&self, model_id: u64);
    fn StateChange(&self, name: &str, old: &str, new: &str);
}
// Source: cc/animation/keyframe_model.h:161-200
pub struct KeyframeModel<C: gfx::AnimationCurve> {
    base_: gfx::KeyframeModel<C>,
    group_: i32,
    target_property_id_: TargetPropertyId,
    element_id_: ElementId,
    #[cfg(debug_assertions)]
    debug_id_: i32,
    needs_synchronized_start_time_: bool,
    received_finished_event_: bool,
    is_controlling_instance_: bool,
    is_impl_only_: bool,
    affects_active_elements_: bool,
    affects_pending_elements_: bool,
    trace: Arc<dyn KeyframeModelTrace>,
}
impl<C: KeyframeCurve> KeyframeModel<C> {
    pub fn base(&self) -> &gfx::KeyframeModel<C> {
        &self.base_
    }
    pub fn base_mut(&mut self) -> &mut gfx::KeyframeModel<C> {
        &mut self.base_
    }
    pub const kInvalidGroup: i32 = -1;
    // Source: cc/animation/keyframe_model.cc:23-42
    pub fn ToCcKeyframeModel(&self) -> &Self {
        self
    }
    pub fn ToCcKeyframeModelMut(&mut self) -> &mut Self {
        self
    }
    // Source: cc/animation/keyframe_model.cc:77-84,115-135
    pub fn Create(
        curve: Option<C>,
        id: i32,
        group: i32,
        target: TargetPropertyId,
        trace: Arc<dyn KeyframeModelTrace>,
    ) -> Self {
        assert_ne!(group, Self::kInvalidGroup);
        #[cfg(debug_assertions)]
        let debug_id = {
            static NEXT: AtomicI32 = AtomicI32::new(0);
            NEXT.fetch_add(1, Ordering::Relaxed).wrapping_add(1)
        };
        Self {
            base_: gfx::KeyframeModel::Create(curve, id, target.target_property_type()),
            group_: group,
            target_property_id_: target,
            element_id_: ElementId::default(),
            #[cfg(debug_assertions)]
            debug_id_: debug_id,
            needs_synchronized_start_time_: false,
            received_finished_event_: false,
            is_controlling_instance_: false,
            is_impl_only_: false,
            affects_active_elements_: true,
            affects_pending_elements_: true,
            trace,
        }
    }
    // Source: cc/animation/keyframe_model.cc:86-113
    pub fn CreateImplInstance(&self, initial_run_state: RunState) -> Self {
        debug_assert!(!self.is_controlling_instance_);
        let mut result = Self::Create(
            Some(self.curve().unwrap().CloneCurve()),
            self.id(),
            self.group_,
            self.target_property_id_.clone(),
            self.trace.clone(),
        );
        result.element_id_ = self.element_id_;
        result.base_.ForceRunState(initial_run_state);
        result.set_iterations(self.iterations());
        result.set_iteration_start(self.iteration_start());
        if self.has_set_start_time() {
            result.set_start_time(self.start_time());
        }
        result.set_hold_time(self.hold_time());
        result.set_start_delay(self.start_delay());
        result.set_direction(self.direction());
        result.set_playback_rate(self.playback_rate());
        result.set_fill_mode(self.fill_mode());
        result.set_auto_fills_on_finish(self.auto_fills_on_finish());
        debug_assert!(!result.is_controlling_instance_);
        result.is_controlling_instance_ = true;
        #[cfg(debug_assertions)]
        {
            result.debug_id_ = self.debug_id_;
        }
        result
    }
    // Source: cc/animation/keyframe_model.cc:137-139
    pub fn TargetProperty(&self) -> i32 {
        self.target_property_id_.target_property_type()
    }
    // Source: cc/animation/keyframe_model.cc:141-173
    pub fn SetRunState(&mut self, state: RunState) {
        let (base, dispatch) = self.SplitRunStateDispatch();
        dispatch.SetRunState(base, state);
    }
    // Source virtual calls in the inherited gfx phase methods dispatch to the
    // actual cc sibling fields through one call-scoped borrowed facade.
    fn SplitRunStateDispatch(&mut self) -> (&mut gfx::KeyframeModel<C>, RunStateDispatch<'_>) {
        let identity = self as *const Self as usize as u64;
        (
            &mut self.base_,
            RunStateDispatch {
                identity,
                group: &self.group_,
                property: &self.target_property_id_,
                controlling: &self.is_controlling_instance_,
                trace: self.trace.as_ref(),
            },
        )
    }
    pub fn Pause(&mut self, hold: TimeDelta, state: RunState) {
        let (base, dispatch) = self.SplitRunStateDispatch();
        base.PauseWith(hold, state, |base, state| dispatch.SetRunState(base, state));
    }
    pub fn Reverse(&mut self, now: TimeTicks) {
        let deferred = self.StartShouldBeDeferred();
        let (base, dispatch) = self.SplitRunStateDispatch();
        base.ReverseWith(now, deferred, |base, state| {
            dispatch.SetRunState(base, state)
        });
    }
    pub fn UnpauseForTesting(&mut self, now: TimeTicks) {
        let (base, dispatch) = self.SplitRunStateDispatch();
        base.UnpauseForTestingWith(now, |base, state| dispatch.SetRunState(base, state));
    }
    // Source: cc/animation/keyframe_model.cc:175-177,212-214
    pub fn InEffect(&self, now: TimeTicks) -> bool {
        self.base_
            .CalculateActiveTime(now, self.StartShouldBeDeferred())
            .is_some()
    }
    pub fn StartShouldBeDeferred(&self) -> bool {
        self.needs_synchronized_start_time_
    }
    // Same resident base, per-call virtual-deferred query; no cached phase copy.
    pub fn HasActiveTime(&self, now: TimeTicks) -> bool {
        self.base_.HasActiveTime(now, self.StartShouldBeDeferred())
    }
    pub fn IsFinishedAt(&self, now: TimeTicks) -> bool {
        self.base_.IsFinishedAt(now, self.StartShouldBeDeferred())
    }
    pub fn TrimTimeToCurrentIteration(
        &self,
        now: TimeTicks,
        limit: Option<&mut LimitDirection>,
    ) -> TimeDelta {
        self.base_
            .TrimTimeToCurrentIteration(now, self.StartShouldBeDeferred(), limit)
    }
    // Source: cc/animation/keyframe_model.cc:179-191
    pub fn PushPropertiesTo(&self, other: &mut Self) {
        #[cfg(debug_assertions)]
        debug_assert_eq!(self.debug_id_, other.debug_id_);
        other.element_id_ = self.element_id_;
        if gfx::KeyframeModel::<C>::IsPaused(self.run_state())
            || gfx::KeyframeModel::<C>::IsPaused(other.run_state())
        {
            other.base_.ForceRunState(self.run_state());
            other.set_hold_time(self.hold_time());
        }
    }
    // Source: cc/animation/keyframe_model.cc:193-203
    pub fn ToString(&self) -> String {
        format!("KeyframeModel{{id={}, group={}, target_property_type={}, custom_property_name={}, native_property_type={}, run_state={}, element_id={}}}",self.id(),self.group_,self.TargetProperty(),self.target_property_id_.custom_property_name(),self.target_property_id_.native_property_type() as i32,gfx::KeyframeModel::<C>::ToString(self.run_state()),self.element_id_.ToString())
    }
    // Source: cc/animation/keyframe_model.cc:205-210
    pub fn SetIsImplOnly(&mut self) {
        self.is_impl_only_ = true;
        self.is_controlling_instance_ = true;
    }
    // Source: cc/animation/keyframe_model.h:92-156
    pub fn group(&self) -> i32 {
        self.group_
    }
    pub fn ungroup(&mut self) {
        self.group_ = Self::kInvalidGroup;
    }
    pub fn element_id(&self) -> ElementId {
        self.element_id_
    }
    pub fn set_element_id(&mut self, id: ElementId) {
        self.element_id_ = id;
    }
    pub fn needs_synchronized_start_time(&self) -> bool {
        self.needs_synchronized_start_time_
    }
    pub fn set_needs_synchronized_start_time(&mut self, value: bool) {
        self.needs_synchronized_start_time_ = value;
    }
    pub fn received_finished_event(&self) -> bool {
        self.received_finished_event_
    }
    pub fn set_received_finished_event(&mut self, value: bool) {
        self.received_finished_event_ = value;
    }
    pub fn set_is_controlling_instance_for_test(&mut self, value: bool) {
        self.is_controlling_instance_ = value;
    }
    pub fn is_controlling_instance(&self) -> bool {
        self.is_controlling_instance_
    }
    pub fn is_impl_only(&self) -> bool {
        self.is_impl_only_
    }
    pub fn set_affects_active_elements(&mut self, value: bool) {
        self.affects_active_elements_ = value;
    }
    pub fn affects_active_elements(&self) -> bool {
        self.affects_active_elements_
    }
    pub fn set_affects_pending_elements(&mut self, value: bool) {
        self.affects_pending_elements_ = value;
    }
    pub fn affects_pending_elements(&self) -> bool {
        self.affects_pending_elements_
    }
    pub fn custom_property_name(&self) -> &str {
        self.target_property_id_.custom_property_name()
    }
    pub fn native_property_type(&self) -> NativePropertyType {
        self.target_property_id_.native_property_type()
    }
}
impl<C: gfx::AnimationCurve> Deref for KeyframeModel<C> {
    type Target = gfx::KeyframeModel<C>;
    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}
impl<C: gfx::AnimationCurve> DerefMut for KeyframeModel<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_
    }
}
impl KeyframeCurve for super::scroll_offset_animation_curve::ScrollOffsetAnimationCurve {
    fn ScrollHasSetInitialValue(&self) -> bool {
        self.HasSetInitialValue()
    }
    fn ScrollSetInitialValue(&mut self, value: PointF) {
        self.SetInitialValue(value, TimeDelta::default(), 0.0)
    }
    fn TypeName(&self) -> &'static str {
        self.TypeName()
    }
}

// Source: cc/animation/keyframe_model.cc:141-173. This is the sole override
// body; explicit sibling borrowing maps source virtual dispatch without owner
// backpointers or a second copy of model state.
struct RunStateDispatch<'a> {
    identity: u64,
    group: &'a i32,
    property: &'a TargetPropertyId,
    controlling: &'a bool,
    trace: &'a dyn KeyframeModelTrace,
}
impl RunStateDispatch<'_> {
    fn SetRunState<C: KeyframeCurve>(&self, base: &mut gfx::KeyframeModel<C>, state: RunState) {
        let name = format!(
            "{}-{}-{}",
            base.curve().unwrap().TypeName(),
            self.property.target_property_type(),
            self.group
        );
        let waiting = matches!(
            base.run_state(),
            RunState::WAITING_FOR_TARGET_AVAILABILITY | RunState::STARTING
        );
        if *self.controlling && waiting && state == RunState::RUNNING {
            self.trace.Begin(self.identity, &name);
        }
        let was_finished = base.is_finished();
        let old = gfx::KeyframeModel::<C>::ToString(base.run_state());
        base.SetRunState(state);
        let new = gfx::KeyframeModel::<C>::ToString(state);
        if *self.controlling && !was_finished && base.is_finished() {
            self.trace.End(self.identity);
        }
        self.trace.StateChange(&name, old, new);
    }
}
