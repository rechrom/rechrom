// Translation status: complete
// Source: cc/animation/scroll_offset_animations_impl.h:1-173
// Source: cc/animation/scroll_offset_animations_impl.cc:1-387
use super::{
    animation::{Animation, AnimationContext, AnimationDelegateId, AnimationHostId, AnimationRef},
    animation_id_provider::AnimationIdProvider,
    animation_timeline::{AnimationTimeline, AnimationTimelineContext, AnimationTimelineTypes},
    keyframe_effect::{KeyframeEffect, KeyframeEffectContext},
    keyframe_model::{KeyframeModel, KeyframeModelTrace, RunState, TargetPropertyId},
    scroll_offset_animation_curve::{
        ScrollCurveConfig, ScrollOffsetAnimationCurve as Curve, ScrollType,
    },
    scroll_offset_animation_curve_factory::ScrollOffsetAnimationCurveFactory as Factory,
};
use compositor_scheduler::{TimeDelta, TimeTicks};
use compositor_tree::{
    geometry::{PointF, Vector2dF},
    target_property::Type as Property,
    ElementId,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
// Source: cc/animation/scroll_offset_animations_impl.h:99-101. The source
// scoped_refptr values name the same resident allocations as the host/timeline.
pub struct ScrollAnimationTypes;
impl AnimationTimelineTypes for ScrollAnimationTypes {
    type AnimationRef = AnimationRef<Curve>;
    type AnimationHostId = AnimationHostId;
}
pub type ScrollTimelineRef = Arc<Mutex<AnimationTimeline<ScrollAnimationTypes>>>;
/// Only source raw-host dereferences cross this per-call boundary. All curve,
/// model, map, create/update/abort and ownership algorithms remain in this stem.
pub trait ScrollOffsetAnimationsContext:
    AnimationContext<Curve> + AnimationTimelineContext<ScrollAnimationTypes>
{
    fn AnimationHostIdentity(&self) -> AnimationHostId;
    fn AddAnimationTimeline(&mut self, timeline: ScrollTimelineRef);
    fn RemoveAnimationTimeline(&mut self, timeline: &ScrollTimelineRef);
    fn ScrollCurveConfig(&self) -> Arc<ScrollCurveConfig>;
    fn KeyframeTrace(&self) -> Arc<dyn KeyframeModelTrace>;
    fn ScrollOffsetAnimationFinished(&mut self, element: ElementId);
    fn IsElementInPropertyTrees(&self, element: ElementId, commits_to_active: bool) -> bool;
    fn TraceScrollAnimation(&self, event: &'static str, number: Option<f64>);
}
/// A call-scoped borrow of the currently executing Animation's actual effect
/// and sibling context. Identity comparison never locks the current Animation.
pub trait ScrollAnimationSelectedReceiver {
    fn Matches(&self, animation: &AnimationRef<Curve>) -> bool;
    fn WithEffect(
        &mut self,
        callback: &mut dyn FnMut(&mut KeyframeEffect<Curve>, &mut dyn KeyframeEffectContext<Curve>),
    );
    fn WithEffectRead(&self, callback: &mut dyn FnMut(&KeyframeEffect<Curve>));
}
/// A field-split view of the selected allocation. The strong handle is only
/// borrowed for Arc identity comparison; all reads/writes use its actual effect.
pub struct BorrowedScrollAnimationReceiver<'a> {
    pub animation: &'a AnimationRef<Curve>,
    pub effect: &'a mut KeyframeEffect<Curve>,
    pub context: &'a mut dyn KeyframeEffectContext<Curve>,
}
impl ScrollAnimationSelectedReceiver for BorrowedScrollAnimationReceiver<'_> {
    fn Matches(&self, animation: &AnimationRef<Curve>) -> bool {
        Arc::ptr_eq(self.animation, animation)
    }
    fn WithEffect(
        &mut self,
        callback: &mut dyn FnMut(&mut KeyframeEffect<Curve>, &mut dyn KeyframeEffectContext<Curve>),
    ) {
        callback(self.effect, self.context)
    }
    fn WithEffectRead(&self, callback: &mut dyn FnMut(&KeyframeEffect<Curve>)) {
        callback(self.effect)
    }
}
fn WithAnimationEffect<R>(
    animation: &AnimationRef<Curve>,
    selected: Option<&mut dyn ScrollAnimationSelectedReceiver>,
    c: &mut impl ScrollOffsetAnimationsContext,
    operation: impl FnOnce(&mut KeyframeEffect<Curve>, &mut dyn KeyframeEffectContext<Curve>) -> R,
) -> R {
    if let Some(selected) = selected {
        if selected.Matches(animation) {
            let mut operation = Some(operation);
            let mut result = None;
            selected.WithEffect(&mut |effect, context| {
                result = Some(operation.take().expect("selected receiver is lent once")(
                    effect, context,
                ))
            });
            return result.expect("selected receiver callback must execute synchronously");
        }
    }
    animation
        .lock()
        .expect("scroll animation")
        .WithEffectContext(c, operation)
}
// Source: cc/animation/scroll_offset_animations_impl.h:97-102.
pub struct ScrollOffsetAnimationImpl {
    animation_host_: AnimationHostId,
    scroll_offset_timeline_: ScrollTimelineRef,
    scroll_offset_animation_: Option<AnimationRef<Curve>>,
    animation_is_autoscroll_: bool,
}
impl ScrollOffsetAnimationImpl {
    // Source: cc/animation/scroll_offset_animations_impl.cc:25-35.
    pub fn new(c: &mut impl ScrollOffsetAnimationsContext) -> Self {
        let timeline = Arc::new(Mutex::new(AnimationTimeline::Create(
            AnimationIdProvider::NextTimelineId(),
            true,
        )));
        let animation = Animation::Create(AnimationIdProvider::NextAnimationId());
        {
            let mut a = animation.lock().expect("scroll animation");
            let id = a.id();
            a.set_animation_delegate(Some(AnimationDelegateId(id as u64)));
        }
        c.AddAnimationTimeline(timeline.clone());
        timeline
            .lock()
            .expect("scroll timeline")
            .AttachAnimation(animation.clone(), c);
        Self {
            animation_host_: c.AnimationHostIdentity(),
            scroll_offset_timeline_: timeline,
            scroll_offset_animation_: Some(animation),
            animation_is_autoscroll_: false,
        }
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:37-42. Rust cannot
    // call a live host from Drop: the owning map invokes this source destructor
    // before releasing the allocation. The Drop assertion verifies that contract.
    pub fn Destroy(&mut self, c: &mut impl ScrollOffsetAnimationsContext) {
        assert_eq!(self.animation_host_, c.AnimationHostIdentity());
        let animation = self
            .scroll_offset_animation_
            .as_ref()
            .expect("live scroll animation");
        self.scroll_offset_timeline_
            .lock()
            .expect("scroll timeline")
            .DetachAnimation(animation, c);
        c.RemoveAnimationTimeline(&self.scroll_offset_timeline_);
        animation
            .lock()
            .expect("scroll animation")
            .set_animation_delegate(None);
        self.scroll_offset_animation_ = None;
    }
    pub fn animation(&self) -> &AnimationRef<Curve> {
        self.scroll_offset_animation_
            .as_ref()
            .expect("live scroll animation")
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:44-58.
    pub fn AutoScrollAnimationCreate(
        &mut self,
        element: ElementId,
        target: PointF,
        current: PointF,
        velocity: f32,
        start: TimeDelta,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        self.AutoScrollAnimationCreateForEffect(element, target, current, velocity, start, None, c)
    }
    pub fn AutoScrollAnimationCreateForEffect(
        &mut self,
        element: ElementId,
        target: PointF,
        current: PointF,
        velocity: f32,
        start: TimeDelta,
        selected: Option<&mut dyn ScrollAnimationSelectedReceiver>,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        let mut curve =
            Factory::CreateAnimation(target, ScrollType::kAutoScroll, c.ScrollCurveConfig());
        curve.SetInitialValue(current, TimeDelta::default(), velocity);
        self.ScrollAnimationCreateInternal(element, curve, start, selected, c);
        self.animation_is_autoscroll_ = true;
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:60-74.
    pub fn MouseWheelScrollAnimationCreate(
        &mut self,
        element: ElementId,
        target: PointF,
        current: PointF,
        delayed: TimeDelta,
        start: TimeDelta,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        self.MouseWheelScrollAnimationCreateForEffect(
            element, target, current, delayed, start, None, c,
        )
    }
    pub fn MouseWheelScrollAnimationCreateForEffect(
        &mut self,
        element: ElementId,
        target: PointF,
        current: PointF,
        delayed: TimeDelta,
        start: TimeDelta,
        selected: Option<&mut dyn ScrollAnimationSelectedReceiver>,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        let mut curve =
            Factory::CreateAnimation(target, ScrollType::kMouseWheel, c.ScrollCurveConfig());
        curve.SetInitialValue(current, delayed, 0.0);
        self.ScrollAnimationCreateInternal(element, curve, start, selected, c);
        self.animation_is_autoscroll_ = false;
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:76-96. Both normal
    // and selected paths execute this sole model construction and attach body.
    fn ScrollAnimationCreateInternal(
        &mut self,
        element: ElementId,
        curve: Curve,
        start: TimeDelta,
        selected: Option<&mut dyn ScrollAnimationSelectedReceiver>,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        c.TraceScrollAnimation(
            "ScrollAnimationCreate",
            Some(curve.Duration().InMicrosecondsF() / 1000.0),
        );
        let mut model = KeyframeModel::Create(
            Some(curve),
            AnimationIdProvider::NextKeyframeModelId(),
            AnimationIdProvider::NextGroupId(),
            TargetPropertyId::new(Property::SCROLL_OFFSET as i32),
            c.KeyframeTrace(),
        );
        model.set_hold_time(Some(start));
        model.SetIsImplOnly();
        WithAnimationEffect(self.animation(), selected, c, |effect, context| {
            debug_assert!(context.AnimationTimelineId().is_some());
            Self::ReattachForEffect(effect, element, context);
            effect.AddKeyframeModel(Box::new(model), context)
        });
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:98-146.
    pub fn ScrollAnimationUpdateTarget(
        &mut self,
        delta: &Vector2dF,
        max: &PointF,
        frame: TimeTicks,
        delayed: TimeDelta,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) -> Option<PointF> {
        self.ScrollAnimationUpdateTargetForEffect(delta, max, frame, delayed, None, c)
    }
    pub fn ScrollAnimationUpdateTargetForEffect(
        &mut self,
        delta: &Vector2dF,
        max: &PointF,
        frame: TimeTicks,
        delayed: TimeDelta,
        selected: Option<&mut dyn ScrollAnimationSelectedReceiver>,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) -> Option<PointF> {
        let (value, trace) =
            WithAnimationEffect(self.animation(), selected, c, |effect, context| {
                if !effect.has_bound_element_animations() {
                    return (None, Some(("No element animation exists", None)));
                }
                let Some(model) = effect
                    .keyframe_models_mut()
                    .iter_mut()
                    .rev()
                    .find(|m| m.TargetProperty() == Property::SCROLL_OFFSET as i32)
                else {
                    Animation::DetachElementForEffect(effect, context);
                    return (None, Some(("No keyframe model exists", None)));
                };
                if delta.IsZero() {
                    return (Some(model.curve().unwrap().target_value()), None);
                }
                let target = model.curve().unwrap().target_value();
                let mut x = target.x() + delta.x();
                let mut y = target.y() + delta.y();
                if x < 0.0 {
                    x = 0.0
                }
                if y < 0.0 {
                    y = 0.0
                }
                if max.x() < x {
                    x = max.x()
                }
                if max.y() < y {
                    y = max.y()
                }
                let target = PointF::new(x, y);
                let mut trimmed = if model.run_state() == RunState::WAITING_FOR_TARGET_AVAILABILITY
                {
                    TimeDelta::default()
                } else {
                    model.TrimTimeToCurrentIteration(frame, None)
                };
                trimmed -= delayed;
                let curve = model.curve_mut().unwrap();
                curve.UpdateTarget(trimmed, target);
                (
                    Some(curve.target_value()),
                    Some((
                        "ScrollAnimationUpdateTarget",
                        Some(curve.Duration().InMicrosecondsF() / 1000.0),
                    )),
                )
            });
        if let Some((name, value)) = trace {
            c.TraceScrollAnimation(name, value)
        }
        value
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:148-192.
    pub fn ScrollAnimationApplyAdjustment(
        &mut self,
        element: ElementId,
        adjustment: &Vector2dF,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        let mut animation = self
            .scroll_offset_animation_
            .as_ref()
            .expect("live scroll animation")
            .lock()
            .expect("scroll animation");
        if element != animation.element_id() {
            c.TraceScrollAnimation("no scroll adjustment different element_ids", None);
            return;
        }
        if animation.element_animations().is_none() {
            c.TraceScrollAnimation("no scroll adjustment no element animation", None);
            return;
        }
        let Some(model) = animation.GetKeyframeModel(Property::SCROLL_OFFSET) else {
            c.TraceScrollAnimation("no scroll adjustment no keyframe model", None);
            return;
        };
        let mut curve = model.curve().unwrap().CloneToScrollOffsetAnimationCurve();
        curve.ApplyAdjustment(*adjustment);
        let mut next = KeyframeModel::Create(
            Some(curve),
            AnimationIdProvider::NextKeyframeModelId(),
            AnimationIdProvider::NextGroupId(),
            TargetPropertyId::new(Property::SCROLL_OFFSET as i32),
            c.KeyframeTrace(),
        );
        next.set_start_time(model.start_time());
        next.SetIsImplOnly();
        next.set_affects_active_elements(false);
        // The source AutoReset restores this exact resident leaf after Abort.
        let previous = self.animation_is_autoscroll_;
        Self::AbortForAnimation(&mut animation, &mut self.animation_is_autoscroll_, false, c);
        animation.AddKeyframeModel(Box::new(next), c);
        c.TraceScrollAnimation("scroll animation adjusted", None);
        self.animation_is_autoscroll_ = previous;
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:194-201. Single body
    // also used by ApplyAdjustment while the actual Animation is already borrowed.
    fn AbortForAnimation(
        animation: &mut Animation<Curve>,
        is_auto: &mut bool,
        completion: bool,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        animation.AbortKeyframeModelsWithProperty(Property::SCROLL_OFFSET, completion, c);
        c.TraceScrollAnimation("ScrollAnimationAbort", Some(completion as u8 as f64));
        *is_auto = false;
    }
    pub fn ScrollAnimationAbort(
        &mut self,
        completion: bool,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        self.ScrollAnimationAbortForEffect(completion, None, c)
    }
    pub fn ScrollAnimationAbortForEffect(
        &mut self,
        completion: bool,
        selected: Option<&mut dyn ScrollAnimationSelectedReceiver>,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        WithAnimationEffect(self.animation(), selected, c, |effect, context| {
            effect.AbortKeyframeModelsWithProperty(
                Property::SCROLL_OFFSET as i32,
                completion,
                context,
            )
        });
        c.TraceScrollAnimation("ScrollAnimationAbort", Some(completion as u8 as f64));
        self.animation_is_autoscroll_ = false;
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:203-206.
    pub fn AnimatingElementRemovedByCommit(&mut self) {
        let mut animation = self
            .scroll_offset_animation_
            .as_ref()
            .expect("live scroll animation")
            .lock()
            .expect("scroll animation");
        animation
            .keyframe_effect_mut()
            .keyframe_models_mut()
            .iter_mut()
            .rev()
            .find(|m| m.TargetProperty() == Property::SCROLL_OFFSET as i32)
            .unwrap()
            .set_affects_pending_elements(false)
    }
    // Source: cc/animation/scroll_offset_animations_impl.h:62-78. The remaining
    // source delegate notifications are intentionally empty source bodies.
    pub fn NotifyAnimationStarted(&self, _time: TimeTicks, _property: i32, _group: i32) {}
    pub fn NotifyAnimationAborted(&self, _time: TimeTicks, _property: i32, _group: i32) {}
    pub fn NotifyAnimationTakeover(
        &self,
        _time: TimeTicks,
        _property: i32,
        _start: TimeTicks,
        _curve: Curve,
    ) {
    }
    pub fn NotifyLocalTimeUpdated(&self, _local: Option<TimeDelta>) {}
    // Source: cc/animation/scroll_offset_animations_impl.cc:208-217. The selected
    // Animation supplies its actual element scalar before the callback; no same
    // receiver mutex is re-locked while its effect dispatches this synchronous event.
    pub fn NotifyAnimationFinishedForElement(
        element: ElementId,
        _time: TimeTicks,
        property: i32,
        _group: i32,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        debug_assert_eq!(property, Property::SCROLL_OFFSET as i32);
        c.ScrollOffsetAnimationFinished(element);
        c.TraceScrollAnimation("NotifyAnimationFinished", None)
    }
    pub fn NotifyAnimationFinished(
        &self,
        time: TimeTicks,
        property: i32,
        group: i32,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        Self::NotifyAnimationFinishedForElement(self.GetElementId(), time, property, group, c)
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:219-241.
    pub fn IsAnimating(&self) -> bool {
        let animation = self.animation().lock().expect("scroll animation");
        Self::IsAnimatingForEffect(animation.keyframe_effect())
    }
    fn IsAnimatingForEffect(effect: &KeyframeEffect<Curve>) -> bool {
        if !effect.has_bound_element_animations() {
            return false;
        }
        let Some(model) = effect.GetKeyframeModel(Property::SCROLL_OFFSET as i32) else {
            return false;
        };
        matches!(
            model.run_state(),
            RunState::WAITING_FOR_TARGET_AVAILABILITY
                | RunState::STARTING
                | RunState::RUNNING
                | RunState::PAUSED
                | RunState::PAUSED_EXCLUSIVE
        )
    }
    fn IsAnimatingWithSelected(
        &self,
        selected: Option<&dyn ScrollAnimationSelectedReceiver>,
    ) -> bool {
        if let Some(selected) = selected {
            if selected.Matches(self.animation()) {
                let mut result = None;
                selected.WithEffectRead(&mut |effect| {
                    result = Some(Self::IsAnimatingForEffect(effect))
                });
                return result.expect("live selected receiver query");
            }
        }
        self.IsAnimating()
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:243-249.
    pub fn IsAutoScrolling(&self) -> bool {
        self.IsAnimating() && self.animation_is_autoscroll_
    }
    pub fn GetElementId(&self) -> ElementId {
        self.animation()
            .lock()
            .expect("scroll animation")
            .element_id()
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:251-263.
    fn ReattachForEffect(
        effect: &mut KeyframeEffect<Curve>,
        element: ElementId,
        c: &mut dyn KeyframeEffectContext<Curve>,
    ) {
        if effect.element_id() != element {
            if effect.element_id().IsValid() {
                c.TraceEffect("scroll offset animation detached element", String::new());
                Animation::DetachElementForEffect(effect, c)
            }
            if element.IsValid() {
                c.TraceEffect("scroll offset animation attached element", String::new());
                Animation::AttachElementForEffect(effect, element, c)
            }
        }
    }
}
impl Drop for ScrollOffsetAnimationImpl {
    fn drop(&mut self) {
        debug_assert!(
            self.scroll_offset_animation_.is_none(),
            "Destroy requires the live host before dropping a scroll animation"
        )
    }
}
// Source: cc/animation/scroll_offset_animations_impl.h:160-168.
pub struct ScrollOffsetAnimationsImpl {
    animation_host_: AnimationHostId,
    element_to_animation_map_: BTreeMap<ElementId, Box<ScrollOffsetAnimationImpl>>,
}
impl ScrollOffsetAnimationsImpl {
    // Source: cc/animation/scroll_offset_animations_impl.cc:265-267.
    pub fn new(host: AnimationHostId) -> Self {
        Self {
            animation_host_: host,
            element_to_animation_map_: BTreeMap::new(),
        }
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:269. Owner-controlled
    // teardown maps destruction of each unique_ptr with its live Host still valid.
    pub fn Destroy(&mut self, c: &mut impl ScrollOffsetAnimationsContext) {
        for animation in self.element_to_animation_map_.values_mut() {
            animation.Destroy(c)
        }
        self.element_to_animation_map_.clear()
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:271-285.
    pub fn AutoScrollAnimationCreate(
        &mut self,
        element: ElementId,
        target: PointF,
        current: PointF,
        velocity: f32,
        start: TimeDelta,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        self.AutoScrollAnimationCreateForEffect(element, target, current, velocity, start, None, c)
    }
    pub fn AutoScrollAnimationCreateForEffect(
        &mut self,
        element: ElementId,
        target: PointF,
        current: PointF,
        velocity: f32,
        start: TimeDelta,
        selected: Option<&mut dyn ScrollAnimationSelectedReceiver>,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        assert_eq!(self.animation_host_, c.AnimationHostIdentity());
        let mut created = Box::new(ScrollOffsetAnimationImpl::new(c));
        if let std::collections::btree_map::Entry::Vacant(entry) =
            self.element_to_animation_map_.entry(element)
        {
            entry.insert(created);
        } else {
            created.Destroy(c)
        }
        self.element_to_animation_map_
            .get_mut(&element)
            .unwrap()
            .AutoScrollAnimationCreateForEffect(
                element, target, current, velocity, start, selected, c,
            )
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:287-301.
    pub fn MouseWheelScrollAnimationCreate(
        &mut self,
        element: ElementId,
        target: PointF,
        current: PointF,
        delayed: TimeDelta,
        start: TimeDelta,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        self.MouseWheelScrollAnimationCreateForEffect(
            element, target, current, delayed, start, None, c,
        )
    }
    pub fn MouseWheelScrollAnimationCreateForEffect(
        &mut self,
        element: ElementId,
        target: PointF,
        current: PointF,
        delayed: TimeDelta,
        start: TimeDelta,
        selected: Option<&mut dyn ScrollAnimationSelectedReceiver>,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        assert_eq!(self.animation_host_, c.AnimationHostIdentity());
        let mut created = Box::new(ScrollOffsetAnimationImpl::new(c));
        if let std::collections::btree_map::Entry::Vacant(entry) =
            self.element_to_animation_map_.entry(element)
        {
            entry.insert(created);
        } else {
            created.Destroy(c)
        }
        self.element_to_animation_map_
            .get_mut(&element)
            .unwrap()
            .MouseWheelScrollAnimationCreateForEffect(
                element, target, current, delayed, start, selected, c,
            )
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:303-314.
    pub fn ScrollAnimationUpdateTarget(
        &mut self,
        delta: &Vector2dF,
        max: &PointF,
        frame: TimeTicks,
        delayed: TimeDelta,
        element: ElementId,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) -> Option<PointF> {
        self.ScrollAnimationUpdateTargetForEffect(delta, max, frame, delayed, element, None, c)
    }
    pub fn ScrollAnimationUpdateTargetForEffect(
        &mut self,
        delta: &Vector2dF,
        max: &PointF,
        frame: TimeTicks,
        delayed: TimeDelta,
        element: ElementId,
        selected: Option<&mut dyn ScrollAnimationSelectedReceiver>,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) -> Option<PointF> {
        self.element_to_animation_map_
            .get_mut(&element)
            .expect("scroll animation exists")
            .ScrollAnimationUpdateTargetForEffect(delta, max, frame, delayed, selected, c)
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:316-322.
    pub fn ScrollAnimationApplyAdjustment(
        &mut self,
        element: ElementId,
        delta: &Vector2dF,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        if let Some(animation) = self.element_to_animation_map_.get_mut(&element) {
            animation.ScrollAnimationApplyAdjustment(element, delta, c)
        }
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:324-329.
    pub fn ScrollAnimationAbort(
        &mut self,
        completion: bool,
        element: ElementId,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        self.ScrollAnimationAbortForEffect(completion, element, None, c)
    }
    pub fn ScrollAnimationAbortForEffect(
        &mut self,
        completion: bool,
        element: ElementId,
        selected: Option<&mut dyn ScrollAnimationSelectedReceiver>,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        if let Some(animation) = self.element_to_animation_map_.get_mut(&element) {
            animation.ScrollAnimationAbortForEffect(completion, selected, c)
        }
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:331-352.
    pub fn HandleRemovedScrollAnimatingElements(
        &mut self,
        active: bool,
        c: &mut impl ScrollOffsetAnimationsContext,
    ) {
        let mut deleted = Vec::new();
        for (element, animation) in &mut self.element_to_animation_map_ {
            if !c.IsElementInPropertyTrees(*element, active) {
                if animation.IsAnimating() {
                    animation.AnimatingElementRemovedByCommit()
                }
                deleted.push(*element)
            }
        }
        for id in deleted {
            let mut animation = self.element_to_animation_map_.remove(&id).unwrap();
            animation.Destroy(c)
        }
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:354-358.
    pub fn ElementHasImplOnlyScrollAnimation(&self, element: ElementId) -> bool {
        self.ElementHasImplOnlyScrollAnimationForEffect(element, None)
    }
    pub fn ElementHasImplOnlyScrollAnimationForEffect(
        &self,
        element: ElementId,
        selected: Option<&dyn ScrollAnimationSelectedReceiver>,
    ) -> bool {
        self.GetScrollAnimation(element)
            .is_some_and(|a| a.IsAnimatingWithSelected(selected))
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:360-367.
    pub fn HasImplOnlyScrollAnimatingElement(&self) -> bool {
        self.element_to_animation_map_
            .values()
            .any(|a| a.IsAnimating())
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:369-376.
    pub fn HasImplOnlyAutoScrollAnimatingElement(&self) -> bool {
        self.element_to_animation_map_
            .values()
            .any(|a| a.IsAutoScrolling())
    }
    // Source: cc/animation/scroll_offset_animations_impl.cc:378-385.
    fn GetScrollAnimation(&self, element: ElementId) -> Option<&ScrollOffsetAnimationImpl> {
        self.element_to_animation_map_
            .get(&element)
            .map(Box::as_ref)
    }
}
impl Drop for ScrollOffsetAnimationsImpl {
    fn drop(&mut self) {
        debug_assert!(
            self.element_to_animation_map_.is_empty(),
            "Destroy requires the live host"
        )
    }
}
