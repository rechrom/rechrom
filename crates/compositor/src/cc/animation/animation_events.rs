// Translation status: complete
// Source: cc/animation/animation_events.h:1-104
// Source: cc/animation/animation_events.cc:1-99
use crate::adapter::time::{TimeDelta, TimeTicks};
// Source: cc/animation/animation_events.h:19-27
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackType {
    kStarted,
    kFinished,
    kAborted,
    kTakeOver,
    kTimeUpdated,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UniqueKeyframeModelId {
    pub timeline_id: i32,
    pub animation_id: i32,
    pub model_id: i32,
}
pub trait AnimationEventCurve {
    fn CloneCurve(&self) -> Self;
}
// Source: cc/animation/animation_events.h:50-64
pub struct AnimationPlaybackEvent<C> {
    pub kind: PlaybackType,
    pub uid: UniqueKeyframeModelId,
    pub group_id: i32,
    pub target_property: i32,
    pub monotonic_time: TimeTicks,
    pub is_impl_only: bool,
    pub animation_start_time: TimeTicks,
    pub curve: Option<C>,
    pub local_time: Option<TimeDelta>,
}
impl<C: AnimationEventCurve> AnimationPlaybackEvent<C> {
    // Source: cc/animation/animation_events.cc:9-21
    pub fn new(
        kind: PlaybackType,
        uid: UniqueKeyframeModelId,
        group_id: i32,
        target_property: i32,
        monotonic_time: TimeTicks,
    ) -> Self {
        Self {
            kind,
            uid,
            group_id,
            target_property,
            monotonic_time,
            is_impl_only: false,
            animation_start_time: TimeTicks::default(),
            curve: None,
            local_time: None,
        }
    }
    // Source: cc/animation/animation_events.cc:23-39
    pub fn TimeUpdated(timeline_id: i32, animation_id: i32, local_time: Option<TimeDelta>) -> Self {
        Self {
            kind: PlaybackType::kTimeUpdated,
            uid: UniqueKeyframeModelId {
                timeline_id,
                animation_id,
                model_id: 0,
            },
            group_id: 0,
            target_property: 0,
            monotonic_time: TimeTicks::default(),
            is_impl_only: false,
            animation_start_time: TimeTicks::default(),
            curve: None,
            local_time,
        }
    }
    // Source: cc/animation/animation_events.cc:54-67
    pub fn AssignFrom(&mut self, other: &Self) {
        self.kind = other.kind;
        self.uid = other.uid;
        self.group_id = other.group_id;
        self.target_property = other.target_property;
        self.monotonic_time = other.monotonic_time;
        self.is_impl_only = other.is_impl_only;
        self.animation_start_time = other.animation_start_time;
        if let Some(curve) = &other.curve {
            self.curve = Some(curve.CloneCurve());
        }
        self.local_time = other.local_time;
    }
    // Source: cc/animation/animation_events.cc:79-86
    pub fn ShouldDispatchToKeyframeEffectAndModel(&self) -> bool {
        self.kind != PlaybackType::kTimeUpdated && !self.is_impl_only
    }
}
// Source: cc/animation/animation_events.cc:41-52
impl<C: AnimationEventCurve> Clone for AnimationPlaybackEvent<C> {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind,
            uid: self.uid,
            group_id: self.group_id,
            target_property: self.target_property,
            monotonic_time: self.monotonic_time,
            is_impl_only: self.is_impl_only,
            animation_start_time: self.animation_start_time,
            curve: self.curve.as_ref().map(AnimationEventCurve::CloneCurve),
            local_time: self.local_time,
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.AssignFrom(source);
    }
}
// Source: cc/animation/animation_events.h:70-77
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggerType {
    kActivate,
    kDeactivate,
}
#[derive(Clone, Debug)]
pub struct AnimationTriggerEvent {
    pub trigger_id: i32,
    pub kind: TriggerType,
    pub time: TimeTicks,
}
impl AnimationTriggerEvent {
    // Source: cc/animation/animation_events.cc:88-92
    pub fn new(trigger_id: i32, kind: TriggerType, time: TimeTicks) -> Self {
        Self {
            trigger_id,
            kind,
            time,
        }
    }
    // Source: cc/animation/animation_events.cc:94-99
    pub fn AssignFrom(&mut self, other: &Self) {
        self.trigger_id = other.trigger_id;
        self.kind = other.kind;
        self.time = other.time;
    }
}
// Source: cc/animation/animation_events.h:94-100
pub enum Event<C> {
    Playback(AnimationPlaybackEvent<C>),
    Trigger(AnimationTriggerEvent),
}
pub struct AnimationEvents<C> {
    events_: Vec<Event<C>>,
    needs_time_updated_events_: bool,
}
impl<C> Default for AnimationEvents<C> {
    fn default() -> Self {
        Self::new()
    }
}
impl<C> AnimationEvents<C> {
    // Source: cc/animation/animation_events.cc:71-77
    pub fn new() -> Self {
        Self {
            events_: Vec::new(),
            needs_time_updated_events_: false,
        }
    }
    pub fn IsEmpty(&self) -> bool {
        self.events_.is_empty() && !self.needs_time_updated_events_
    }
    // Source: cc/animation/animation_events.h:87-98
    pub fn needs_time_updated_events(&self) -> bool {
        self.needs_time_updated_events_
    }
    pub fn set_needs_time_updated_events(&mut self, value: bool) {
        self.needs_time_updated_events_ = value;
    }
    pub fn events(&self) -> &Vec<Event<C>> {
        &self.events_
    }
    pub fn events_mut(&mut self) -> &mut Vec<Event<C>> {
        &mut self.events_
    }
}
impl AnimationEventCurve for super::scroll_offset_animation_curve::ScrollOffsetAnimationCurve {
    fn CloneCurve(&self) -> Self {
        self.CloneToScrollOffsetAnimationCurve()
    }
}
