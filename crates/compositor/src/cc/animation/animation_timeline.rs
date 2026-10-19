// Translation status: complete
// Source: cc/animation/animation_timeline.h:1-125
// Source: cc/animation/animation_timeline.cc:1-210
//
// Chromium stores scoped_refptr<Animation> values and a raw AnimationHost
// pointer. The Rust boundary keeps the same shared Animation allocations but
// stores only the host identity. Every operation that would dereference the
// source host receives its live call-scoped context explicitly.

use crate::adapter::{input_types::ElementId, time::TimeTicks};
use std::collections::HashMap;

// Source: cc/animation/animation_timeline.h:24-26,32-45,101-120.
pub trait AnimationTimelineTypes {
    /// The actual shared Animation allocation (the scoped_refptr equivalent).
    type AnimationRef: Clone;
    /// Stable identity for the resident AnimationHost; never a host pointer.
    type AnimationHostId: Copy + Eq;
}

/// Calls into an Animation or AnimationHost always occur while their actual
/// owner is available. Implementations may briefly lock an AnimationRef, but
/// this interface cannot retain that borrow or recover a host from its id.
pub trait AnimationTimelineContext<B: AnimationTimelineTypes> {
    type ScrollTree: ?Sized;

    fn AnimationId(&self, animation: &B::AnimationRef) -> i32;
    fn AnimationTimelineId(&self, animation: &B::AnimationRef) -> Option<i32>;
    fn SetAnimationHost(&mut self, animation: &B::AnimationRef, host: Option<B::AnimationHostId>);
    fn SetAnimationTimeline(&mut self, animation: &B::AnimationRef, timeline: Option<i32>);
    fn CreateImplInstance(&mut self, animation: &B::AnimationRef) -> B::AnimationRef;
    fn HasElementAnimations(&self, animation: &B::AnimationRef) -> bool;
    fn DetachElement(&mut self, animation: &B::AnimationRef);
    fn IsWorkletAnimation(&self, animation: &B::AnimationRef) -> bool;
    fn IsScrollLinkedAnimation(&self, animation: &B::AnimationRef) -> bool;
    fn AwaitingDeletion(&self, animation: &B::AnimationRef) -> bool;
    fn Tick(&mut self, animation: &B::AnimationRef, monotonic_time: TimeTicks) -> bool;
    fn PushAnimationPropertiesTo(
        &mut self,
        animation: &B::AnimationRef,
        animation_impl: &B::AnimationRef,
    );

    fn HostIsOwnerThread(&self, host: B::AnimationHostId) -> bool;
    fn HostInProtectedSequence(&self, host: B::AnimationHostId) -> bool;
    fn HostWaitForProtectedSequenceCompletion(&self, host: B::AnimationHostId);
    fn HostSetNeedsPushProperties(&mut self, host: B::AnimationHostId);
}

// Source: cc/animation/animation_timeline.h:28-43,97-120.
pub struct AnimationTimeline<B: AnimationTimelineTypes> {
    id_to_animation_map_: HashMap<i32, B::AnimationRef>,
    id_: i32,
    animation_host_: Option<B::AnimationHostId>,
    needs_push_properties_: bool,
    is_impl_only_: bool,
}

impl<B: AnimationTimelineTypes> AnimationTimeline<B> {
    // Source: cc/animation/animation_timeline.h:36-38;
    // cc/animation/animation_timeline.cc:19-28.
    pub fn Create(id: i32, impl_only: bool) -> Self {
        Self {
            id_to_animation_map_: HashMap::new(),
            id_: id,
            animation_host_: None,
            needs_push_properties_: false,
            is_impl_only_: impl_only,
        }
    }

    // Source: cc/animation/animation_timeline.cc:36-39.
    pub fn CreateImplInstance(&self) -> Self {
        // The source calls Create(id()) without forwarding is_impl_only_.
        Self::Create(self.id(), false)
    }

    // Source: cc/animation/animation_timeline.h:40-48,61.
    pub fn id(&self) -> i32 {
        self.id_
    }
    pub fn animations(&self) -> &HashMap<i32, B::AnimationRef> {
        &self.id_to_animation_map_
    }
    pub fn is_impl_only(&self) -> bool {
        self.is_impl_only_
    }

    // Source: cc/animation/animation_timeline.h:50-59.
    pub fn animation_host(&self) -> Option<B::AnimationHostId> {
        self.animation_host_
    }

    // Source: cc/animation/animation_timeline.cc:41-55.
    pub fn SetAnimationHost<C: AnimationTimelineContext<B>>(
        &mut self,
        animation_host: Option<B::AnimationHostId>,
        context: &mut C,
    ) {
        debug_assert!(
            animation_host.is_none_or(|host| context.HostIsOwnerThread(host)),
            "AnimationHost mutation must run on its owner sequence"
        );
        if self.animation_host_ == animation_host {
            return;
        }
        self.WaitForProtectedSequenceCompletion(context);
        self.animation_host_ = animation_host;
        for animation in self.id_to_animation_map_.values() {
            context.SetAnimationHost(animation, animation_host);
        }
        self.SetNeedsPushProperties(context);
    }

    // Source: cc/animation/animation_timeline.h:63-69;
    // cc/animation/animation_timeline.cc:57-73.
    pub fn AttachAnimation<C: AnimationTimelineContext<B>>(
        &mut self,
        animation: B::AnimationRef,
        context: &mut C,
    ) {
        let id = context.AnimationId(&animation);
        debug_assert_ne!(id, 0);
        context.SetAnimationHost(&animation, self.animation_host());
        context.SetAnimationTimeline(&animation, Some(self.id_));
        match self.id_to_animation_map_.entry(id) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(animation);
            }
            std::collections::hash_map::Entry::Occupied(_) => {
                panic!("AnimationTimeline animation ids are unique")
            }
        }
        self.SetNeedsPushProperties(context);
    }

    pub fn DetachAnimation<C: AnimationTimelineContext<B>>(
        &mut self,
        animation: &B::AnimationRef,
        context: &mut C,
    ) {
        let id = context.AnimationId(animation);
        debug_assert_ne!(id, 0);
        self.EraseAnimation(animation, context);
        self.id_to_animation_map_.remove(&id);
        self.SetNeedsPushProperties(context);
    }

    // Source: cc/animation/animation_timeline.cc:75-88.
    pub fn GetAnimationById(&self, animation_id: i32) -> Option<&B::AnimationRef> {
        self.id_to_animation_map_.get(&animation_id)
    }
    pub fn ClearAnimations<C: AnimationTimelineContext<B>>(&mut self, context: &mut C) {
        // Move the source scoped owners out before callbacks. This preserves the
        // allocations and avoids a reentrant borrow of the resident map.
        let animations = std::mem::take(&mut self.id_to_animation_map_);
        for animation in animations.values() {
            self.EraseAnimation(animation, context);
        }
        self.SetNeedsPushProperties(context);
    }
    pub fn HasAnimation(&self) -> bool {
        !self.id_to_animation_map_.is_empty()
    }

    // Source: cc/animation/animation_timeline.h:70-77;
    // cc/animation/animation_timeline.cc:90-122.
    pub fn TickTimeLinkedAnimations<C: AnimationTimelineContext<B>>(
        &self,
        ticking_animations: &[B::AnimationRef],
        monotonic_time: TimeTicks,
        tick_finished: bool,
        context: &mut C,
    ) -> bool {
        debug_assert!(!self.IsScrollTimeline());
        let mut animated = false;
        for animation in ticking_animations {
            if context.AnimationTimelineId(animation) != Some(self.id_) {
                continue;
            }
            if context.IsWorkletAnimation(animation) {
                continue;
            }
            if context.IsScrollLinkedAnimation(animation) {
                continue;
            }
            if !tick_finished && context.AwaitingDeletion(animation) {
                continue;
            }
            animated |= context.Tick(animation, monotonic_time);
        }
        animated
    }
    pub fn TickScrollLinkedAnimations<C: AnimationTimelineContext<B>>(
        &self,
        _ticking_animations: &[B::AnimationRef],
        _scroll_tree: &C::ScrollTree,
        _is_active_tree: bool,
        _context: &mut C,
    ) -> bool {
        false
    }

    // Source: cc/animation/animation_timeline.h:79-90;
    // cc/animation/animation_timeline.cc:124-138,188-194.
    pub fn SetNeedsPushProperties<C: AnimationTimelineContext<B>>(&mut self, context: &mut C) {
        self.needs_push_properties_ = true;
        if let Some(host) = self.animation_host() {
            context.HostSetNeedsPushProperties(host);
        }
    }
    pub fn needs_push_properties(&self) -> bool {
        self.needs_push_properties_
    }
    pub fn PushPropertiesTo<C: AnimationTimelineContext<B>>(
        &mut self,
        timeline_impl: &mut Self,
        context: &mut C,
    ) {
        if self.needs_push_properties_ {
            self.needs_push_properties_ = false;
            self.PushAttachedAnimationsToImplThread(timeline_impl, context);
            self.RemoveDetachedAnimationsFromImplThread(timeline_impl, context);
            self.PushPropertiesToImplThread(timeline_impl, context);
        }
    }
    pub fn ActivateTimeline(&mut self) {}
    pub fn IsScrollTimeline(&self) -> bool {
        false
    }
    pub fn IsLinkedToScroller(&self, _scroller: ElementId) -> bool {
        false
    }

    // Source: cc/animation/animation_timeline.cc:140-168.
    fn PushAttachedAnimationsToImplThread<C: AnimationTimelineContext<B>>(
        &self,
        timeline_impl: &mut Self,
        context: &mut C,
    ) {
        for (id, animation) in &self.id_to_animation_map_ {
            if timeline_impl.GetAnimationById(*id).is_some() {
                continue;
            }
            let to_add = context.CreateImplInstance(animation);
            timeline_impl.AttachAnimation(to_add, context);
        }
    }
    fn RemoveDetachedAnimationsFromImplThread<C: AnimationTimelineContext<B>>(
        &self,
        timeline_impl: &mut Self,
        context: &mut C,
    ) {
        timeline_impl.id_to_animation_map_.retain(|id, animation| {
            if self.id_to_animation_map_.contains_key(id) {
                true
            } else {
                Self::EraseAnimationValue(animation, context);
                false
            }
        });
    }

    // Source: cc/animation/animation_timeline.cc:170-186.
    fn EraseAnimation<C: AnimationTimelineContext<B>>(
        &self,
        animation: &B::AnimationRef,
        context: &mut C,
    ) {
        Self::EraseAnimationValue(animation, context)
    }
    fn EraseAnimationValue<C: AnimationTimelineContext<B>>(
        animation: &B::AnimationRef,
        context: &mut C,
    ) {
        if context.HasElementAnimations(animation) {
            context.DetachElement(animation);
        }
        context.SetAnimationHost(animation, None);
        context.SetAnimationTimeline(animation, None);
    }
    fn PushPropertiesToImplThread<C: AnimationTimelineContext<B>>(
        &self,
        timeline_impl: &mut Self,
        context: &mut C,
    ) {
        for (id, animation) in &self.id_to_animation_map_ {
            if let Some(animation_impl) = timeline_impl.id_to_animation_map_.get(id) {
                context.PushAnimationPropertiesTo(animation, animation_impl);
            }
        }
    }

    // Source: cc/animation/animation_timeline.cc:30-34. Rust Drop cannot
    // acquire an injected live Host context, so the owning AnimationHost calls
    // Destroy before releasing its last timeline owner.
    pub fn Destroy<C: AnimationTimelineContext<B>>(&mut self, context: &mut C) {
        let animations = std::mem::take(&mut self.id_to_animation_map_);
        for animation in animations.values() {
            context.SetAnimationTimeline(animation, None);
        }
    }

    // Source: cc/animation/animation_timeline.h:92-95;
    // cc/animation/animation_timeline.cc:196-208.
    pub fn IsOwnerThread<C: AnimationTimelineContext<B>>(&self, context: &C) -> bool {
        self.animation_host_
            .is_none_or(|host| context.HostIsOwnerThread(host))
    }
    pub fn InProtectedSequence<C: AnimationTimelineContext<B>>(&self, context: &C) -> bool {
        self.animation_host_
            .is_none_or(|host| context.HostInProtectedSequence(host))
    }
    pub fn WaitForProtectedSequenceCompletion<C: AnimationTimelineContext<B>>(&self, context: &C) {
        if let Some(host) = self.animation_host_ {
            context.HostWaitForProtectedSequenceCompletion(host);
        }
    }
}
