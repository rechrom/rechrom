//! Page-local canonical animation owner used by both main and impl compositor hosts.
//!
//! The resident state is the translated Animation/Timeline/ScrollOffsetAnimations
//! object graph. Query facades borrow that graph; they do not maintain a parallel
//! keyframe-model cache. Calls that can enter Host code take the selected owner out
//! of the mutex first, so no owner lock survives a synchronous source callback.

use super::{host_owners::MutatorEvents, input::MutatorHost};
use crate::{
    adapter::{
        animation_host::{AnimationHostQueries, AnimationMutationTarget, ScrollAnimationReentry},
        geometry::{PointF, Vector2dF},
        input_types::ElementId,
        layer_tree_host_impl::HostAnimationTickResult,
        layer_tree_impl::{
            ElementListType, LayerTreeHostId, LayerTreeMutatorHost, PropertyTreesFor,
        },
        property_tree::PropertyTreeMutatorHost,
        time::{TimeDelta, TimeTicks},
    },
    cc::{
        animation::{
            animation::{
                Animation, AnimationContext, AnimationDelegateId, AnimationHostId, AnimationRef,
            },
            animation_events::{AnimationEvents, AnimationPlaybackEvent, PlaybackType},
            animation_timeline::{AnimationTimelineContext, AnimationTimelineTypes},
            keyframe_effect::{BoundElementAnimations, KeyframeEffect},
            keyframe_model::{KeyframeCurve, KeyframeModel, KeyframeModelTrace, LimitDirection},
            scroll_offset_animation_curve::{
                ScrollCurveConfig, ScrollCurveTarget, ScrollOffsetAnimationCurve,
            },
            scroll_offset_animations_impl::{
                BorrowedScrollAnimationReceiver, ScrollAnimationTypes,
                ScrollOffsetAnimationsContext, ScrollOffsetAnimationsImpl, ScrollTimelineRef,
            },
        },
        trees::target_property::Type as TargetProperty,
    },
    NativeCompositorTypes,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, MutexGuard,
    },
};

type Curve = ScrollOffsetAnimationCurve;
type AnimationHandle = AnimationRef<Curve>;

#[derive(Debug)]
struct NativeElementAnimations {
    element: ElementId,
}
impl BoundElementAnimations for NativeElementAnimations {
    fn element_id(&self) -> ElementId {
        self.element
    }
}

#[derive(Default)]
struct ElementState {
    handle: Option<Arc<NativeElementAnimations>>,
    animations: BTreeSet<i32>,
}

struct NativeAnimationState {
    scroll: Option<ScrollOffsetAnimationsImpl>,
    timelines: BTreeMap<i32, ScrollTimelineRef>,
    animations: BTreeMap<i32, AnimationHandle>,
    ticking: BTreeSet<i32>,
    elements: BTreeMap<ElementId, ElementState>,
    layer_tree_mutator: Option<Box<super::host_owners::LayerTreeMutator>>,
    next_frame_has_pending_raf: bool,
    current_frame_had_raf: bool,
    pending_compositor_metrics_tracker_infos:
        Vec<crate::adapter::client_layer_tree_host_impl::TrackedAnimationSequenceInfo>,
    has_canvas_invalidation: bool,
    has_js_animation: bool,
    main_thread_animations_count: usize,
    has_smil_animation: bool,
    has_view_transition: bool,
    needs_push_properties: bool,
    delegate: Option<LayerTreeHostId>,
}

#[derive(Clone)]
pub(crate) struct NativeAnimationOwner {
    identity: super::types::MainMutatorHostIdentity,
    host: AnimationHostId,
    state: Arc<Mutex<NativeAnimationState>>,
    config: Arc<ScrollCurveConfig>,
    trace: Arc<NativeKeyframeTrace>,
}

impl Default for NativeAnimationOwner {
    fn default() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let raw = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        assert_ne!(raw, 0, "native animation identity exhausted");
        let host = AnimationHostId(raw);
        Self {
            identity: super::types::MainMutatorHostIdentity(raw),
            host,
            state: Arc::new(Mutex::new(NativeAnimationState {
                scroll: Some(ScrollOffsetAnimationsImpl::new(host)),
                timelines: BTreeMap::new(),
                animations: BTreeMap::new(),
                ticking: BTreeSet::new(),
                elements: BTreeMap::new(),
                layer_tree_mutator: None,
                next_frame_has_pending_raf: false,
                current_frame_had_raf: false,
                pending_compositor_metrics_tracker_infos: Vec::new(),
                has_canvas_invalidation: false,
                has_js_animation: false,
                main_thread_animations_count: 0,
                has_smil_animation: false,
                has_view_transition: false,
                needs_push_properties: false,
                delegate: None,
            })),
            config: Arc::new(ScrollCurveConfig::default()),
            trace: Arc::new(NativeKeyframeTrace),
        }
    }
}

impl NativeAnimationOwner {
    fn lock(&self) -> MutexGuard<'_, NativeAnimationState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(crate) fn identity(&self) -> super::types::MainMutatorHostIdentity {
        self.identity
    }

    pub(crate) fn CreateMutatorHost(&self) -> MutatorHost {
        MutatorHost(Box::new(NativeAnimationQueries {
            owner: self.clone(),
        }))
    }

    pub(crate) fn CreateEvents(&self) -> MutatorEvents {
        MutatorEvents(Box::new(NativeAnimationEvents {
            events: AnimationEvents::new(),
        }))
    }

    pub(crate) fn SetDelegate(&self, delegate: Option<LayerTreeHostId>) {
        self.lock().delegate = delegate;
    }

    pub(crate) fn Clear(&self) {
        self.WithScroll(None, None, |scroll, call| scroll.Destroy(call));
        let mut state = self.lock();
        state.timelines.clear();
        state.animations.clear();
        state.ticking.clear();
        state.elements.clear();
        state.current_frame_had_raf = false;
        state.pending_compositor_metrics_tracker_infos.clear();
        state.needs_push_properties = false;
    }

    // Source: cc/animation/animation_host.cc:227-232.
    pub(crate) fn SetCurrentFrameHadRAF(&self, value: bool) {
        self.lock().current_frame_had_raf = value;
    }

    pub(crate) fn CurrentFrameHadRAF(&self) -> bool {
        self.lock().current_frame_had_raf
    }

    // Source: cc/animation/animation_host.cc:1055-1062. The source method
    // moves the complete vector out of AnimationHost for this frame.
    pub(crate) fn AddPendingCompositorMetricsTrackerInfo(
        &self,
        info: crate::adapter::client_layer_tree_host_impl::TrackedAnimationSequenceInfo,
    ) {
        self.lock()
            .pending_compositor_metrics_tracker_infos
            .push(info);
    }

    pub(crate) fn TakePendingCompositorMetricsTrackerInfos(
        &self,
    ) -> Vec<crate::adapter::client_layer_tree_host_impl::TrackedAnimationSequenceInfo> {
        std::mem::take(&mut self.lock().pending_compositor_metrics_tracker_infos)
    }

    fn WithScroll<'a, R>(
        &'a self,
        target: Option<&'a mut dyn AnimationMutationTarget>,
        selected_element: Option<ElementId>,
        callback: impl FnOnce(&mut ScrollOffsetAnimationsImpl, &mut NativeAnimationCall<'_>) -> R,
    ) -> R {
        let mut scroll = self
            .lock()
            .scroll
            .take()
            .expect("canonical scroll animation owner is already selected");
        let mut call = NativeAnimationCall {
            owner: self,
            target,
            property_trees: None,
            selected_element,
            now: TimeTicks::default(),
            active: true,
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            callback(&mut scroll, &mut call)
        }));
        let mut state = self.lock();
        assert!(
            state.scroll.is_none(),
            "scroll animation owner was replaced"
        );
        state.scroll = Some(scroll);
        drop(state);
        match result {
            Ok(value) => value,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    pub(crate) fn CreateAutoScrollAnimation(
        &self,
        element: ElementId,
        target: PointF,
        current: PointF,
        velocity: f32,
        start: TimeDelta,
    ) {
        self.WithScroll(None, Some(element), |scroll, call| {
            scroll.AutoScrollAnimationCreate(element, target, current, velocity, start, call)
        });
    }

    pub(crate) fn CreateMouseWheelScrollAnimation(
        &self,
        element: ElementId,
        target: PointF,
        current: PointF,
        delayed: TimeDelta,
        start: TimeDelta,
    ) {
        self.WithScroll(None, Some(element), |scroll, call| {
            scroll.MouseWheelScrollAnimationCreate(element, target, current, delayed, start, call)
        });
    }

    pub(crate) fn AbortScrollAnimation(&self, element: ElementId, needs_completion: bool) {
        self.WithScroll(None, Some(element), |scroll, call| {
            scroll.ScrollAnimationAbort(needs_completion, element, call)
        });
    }

    pub(crate) fn UpdateScrollAnimationTarget(
        &self,
        delta: Vector2dF,
        max: PointF,
        time: TimeTicks,
        delayed: TimeDelta,
        element: ElementId,
    ) -> Option<PointF> {
        self.WithScroll(None, Some(element), |scroll, call| {
            scroll.ScrollAnimationUpdateTarget(&delta, &max, time, delayed, element, call)
        })
    }

    pub(crate) fn ElementHasImplOnlyScrollAnimation(&self, element: ElementId) -> bool {
        self.lock()
            .scroll
            .as_ref()
            .expect("canonical scroll animation owner")
            .ElementHasImplOnlyScrollAnimation(element)
    }

    pub(crate) fn HasImplOnlyScrollAnimatingElement(&self) -> bool {
        self.lock()
            .scroll
            .as_ref()
            .expect("canonical scroll animation owner")
            .HasImplOnlyScrollAnimatingElement()
    }

    pub(crate) fn HasImplOnlyAutoScrollAnimatingElement(&self) -> bool {
        self.lock()
            .scroll
            .as_ref()
            .expect("canonical scroll animation owner")
            .HasImplOnlyAutoScrollAnimatingElement()
    }

    pub(crate) fn TickAnimations(
        &self,
        time: TimeTicks,
        is_active: bool,
        events: Option<&mut MutatorEvents>,
        target: &mut dyn AnimationMutationTarget,
    ) -> HostAnimationTickResult {
        assert_eq!(
            self.lock().delegate,
            Some(target.AnimationHostOwner()),
            "animation target must be the registered resident Host"
        );
        let timelines: Vec<_> = self.lock().timelines.values().cloned().collect();
        let ticking = self.TickingAnimations();
        let mut animated = false;
        let _ = events;
        let mut call = NativeAnimationCall {
            owner: self,
            target: Some(target),
            property_trees: None,
            selected_element: None,
            now: time,
            active: is_active,
        };
        for timeline in timelines {
            animated |= timeline
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .TickTimeLinkedAnimations(&ticking, time, !is_active, &mut call);
        }
        HostAnimationTickResult {
            animated,
            needs_next_frame: animated,
        }
    }

    // Source: cc/animation/animation_host.cc:795-801. Worklet animations are
    // ticked after mutation output has arrived so the result affects this draw.
    pub(crate) fn TickWorkletAnimations(&self) {
        let ticking = self.TickingAnimations();
        let mut call = NativeAnimationCall::detached(self);
        for animation in ticking {
            let mut animation = animation
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if animation.IsWorkletAnimation() {
                call.selected_element = Some(animation.element_id());
                animation.Tick(TimeTicks::default(), &mut call);
            }
        }
    }

    // Source: cc/animation/animation_host.cc:870-875.
    pub(crate) fn PromoteScrollTimelinesPendingToActive(&self, mutator: &mut MutatorHost) {
        assert_eq!(
            mutator.identity(),
            self.identity,
            "resident impl mutator owner"
        );
        let timelines: Vec<_> = self.lock().timelines.values().cloned().collect();
        for timeline in timelines {
            timeline
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .ActivateTimeline();
        }
    }

    // Source: cc/animation/animation_host.cc:485-518,538-570,602-640.
    // The main and impl owners are separate resident graphs. Only translated
    // timeline/animation methods cross this call; no parallel animation cache
    // or snapshot is constructed here.
    pub(crate) fn PushMutatorPropertiesTo(
        &self,
        source_identity: super::types::MainMutatorHostIdentity,
        target: &mut MutatorHost,
        property_trees: &PropertyTreesFor<NativeCompositorTypes>,
    ) {
        assert_eq!(
            source_identity, self.identity,
            "resident main mutator owner"
        );
        let target_queries = target
            .0
            .AsAnyMut()
            .downcast_mut::<NativeAnimationQueries>()
            .expect("native compositor impl mutator host");
        let target_owner = target_queries.owner.clone();
        assert_ne!(
            self.identity, target_owner.identity,
            "main and impl animation owners are distinct"
        );

        let (
            main_thread_animations_count,
            current_frame_had_raf,
            next_frame_has_pending_raf,
            has_canvas_invalidation,
            has_js_animation,
            has_smil_animation,
            has_view_transition,
            needs_push,
            source_timelines,
            pending_tracker_infos,
        ) = {
            let mut source = self.lock();
            let values = (
                source.main_thread_animations_count,
                source.current_frame_had_raf,
                source.next_frame_has_pending_raf,
                source.has_canvas_invalidation,
                source.has_js_animation,
                source.has_smil_animation,
                source.has_view_transition,
                source.needs_push_properties,
                source.timelines.values().cloned().collect::<Vec<_>>(),
                std::mem::take(&mut source.pending_compositor_metrics_tracker_infos),
            );
            source.needs_push_properties = false;
            values
        };
        {
            let mut target = target_owner.lock();
            target.main_thread_animations_count = main_thread_animations_count;
            target.current_frame_had_raf = current_frame_had_raf;
            target.next_frame_has_pending_raf = next_frame_has_pending_raf;
            target.has_canvas_invalidation = has_canvas_invalidation;
            target.has_js_animation = has_js_animation;
            target.has_smil_animation = has_smil_animation;
            target.has_view_transition = has_view_transition;
            assert!(
                target.pending_compositor_metrics_tracker_infos.is_empty(),
                "pending compositor metrics are consumed before the next commit"
            );
            target.pending_compositor_metrics_tracker_infos = pending_tracker_infos;
        }
        if !needs_push {
            return;
        }

        let source_ids = source_timelines
            .iter()
            .map(|timeline| {
                timeline
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .id()
            })
            .collect::<BTreeSet<_>>();

        for source_timeline in &source_timelines {
            let id = source_timeline
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .id();
            if target_owner.lock().timelines.contains_key(&id) {
                continue;
            }
            let impl_timeline = Arc::new(Mutex::new(
                source_timeline
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .CreateImplInstance(),
            ));
            let mut call = NativeAnimationCall::with_property_trees(&target_owner, property_trees);
            impl_timeline
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .SetAnimationHost(Some(target_owner.host), &mut call);
            assert!(target_owner
                .lock()
                .timelines
                .insert(id, impl_timeline)
                .is_none());
        }

        let stale = {
            let target = target_owner.lock();
            target
                .timelines
                .iter()
                .filter_map(|(id, timeline)| {
                    let keep = source_ids.contains(id)
                        || timeline
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .is_impl_only();
                    (!keep).then_some((*id, timeline.clone()))
                })
                .collect::<Vec<_>>()
        };
        for (id, timeline) in stale {
            let mut call = NativeAnimationCall::with_property_trees(&target_owner, property_trees);
            let mut timeline = timeline
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            timeline.ClearAnimations(&mut call);
            timeline.SetAnimationHost(None, &mut call);
            drop(timeline);
            target_owner.lock().timelines.remove(&id);
        }

        for source_timeline in source_timelines {
            let id = source_timeline
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .id();
            let target_timeline = target_owner
                .lock()
                .timelines
                .get(&id)
                .cloned()
                .expect("impl timeline created before property push");
            let mut call = NativeAnimationCall::with_property_trees(&target_owner, property_trees);
            source_timeline
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .PushPropertiesTo(
                    &mut target_timeline
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner),
                    &mut call,
                );
        }
        target_owner.lock().needs_push_properties = true;
    }

    pub(crate) fn UpdateAnimationState(
        &self,
        start_ready: bool,
        events: Option<&mut MutatorEvents>,
        target: &mut dyn AnimationMutationTarget,
    ) -> bool {
        assert_eq!(self.lock().delegate, Some(target.AnimationHostOwner()));
        let ticking = self.TickingAnimations();
        if ticking.is_empty() {
            return false;
        }
        let mut update = |native: &mut AnimationEvents<Curve>| {
            let mut call = NativeAnimationCall::targeted(self, target);
            for animation in &ticking {
                let element = animation
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .element_id();
                call.selected_element = Some(element);
                animation
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .UpdateState(start_ready, native, &mut call);
            }
        };
        if let Some(events) = events {
            events.WithScrollAnimationEvents(&mut update);
        } else {
            let mut native = AnimationEvents::new();
            update(&mut native);
            debug_assert!(
                native.IsEmpty(),
                "animation events require the resident event owner"
            );
        }
        true
    }

    pub(crate) fn ActivateAnimations(
        &self,
        events: Option<&mut MutatorEvents>,
        target: &mut dyn AnimationMutationTarget,
    ) -> bool {
        assert_eq!(self.lock().delegate, Some(target.AnimationHostOwner()));
        let ticking = self.TickingAnimations();
        if ticking.is_empty() {
            return false;
        }
        let mut activate = |native: &mut AnimationEvents<Curve>| {
            let mut call = NativeAnimationCall::targeted(self, target);
            for animation in &ticking {
                let mut animation = animation
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                call.selected_element = Some(animation.element_id());
                animation.ActivateKeyframeModels(&mut call);
                animation.UpdateState(false, native, &mut call);
            }
        };
        if let Some(events) = events {
            events.WithScrollAnimationEvents(&mut activate);
        } else {
            let mut native = AnimationEvents::new();
            activate(&mut native);
            debug_assert!(
                native.IsEmpty(),
                "animation events require the resident event owner"
            );
        }
        true
    }

    pub(crate) fn TakeTimeUpdatedEvents(&self, events: Option<&mut MutatorEvents>) {
        if let Some(events) = events {
            events.WithScrollAnimationEvents(&mut |native| {
                // Scroll animations never produce kTimeUpdated. Preserve the
                // source owner flag and clear it only after visiting the real
                // ticking set.
                if native.needs_time_updated_events() {
                    for animation in self.TickingAnimations() {
                        animation
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .TakeTimeUpdatedEvent(native);
                    }
                    native.set_needs_time_updated_events(false);
                }
            });
        }
    }

    pub(crate) fn TickMutator(
        &self,
        _time: TimeTicks,
        _trees: &PropertyTreesFor<NativeCompositorTypes>,
        _is_active: bool,
    ) {
        let mut state = self.lock();
        if let Some(mutator) = state.layer_tree_mutator.as_mut() {
            let _ = mutator.0.HasMutators();
        }
    }

    pub(crate) fn HandleRemovedScrollAnimatingElements(
        &self,
        commits_to_active: bool,
        target: &mut dyn AnimationMutationTarget,
    ) {
        assert_eq!(self.lock().delegate, Some(target.AnimationHostOwner()));
        self.WithScroll(Some(target), None, |scroll, call| {
            scroll.HandleRemovedScrollAnimatingElements(commits_to_active, call)
        });
    }

    pub(crate) fn HasInvalidationAnimation(&self) -> bool {
        self.TickingAnimations().iter().any(|animation| {
            animation
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .RequiresInvalidation()
        })
    }
    pub(crate) fn HasNativePropertyAnimation(&self) -> bool {
        self.TickingAnimations().iter().any(|animation| {
            animation
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .AffectsNativeProperty()
        })
    }
    pub(crate) fn HasViewTransition(&self) -> bool {
        self.lock().has_view_transition
    }
    pub(crate) fn EventsIsEmpty(&self, events: &MutatorEvents) -> bool {
        events.IsEmpty()
    }

    pub(crate) fn HasScrollLinkedAnimation(&self, scroller: ElementId) -> bool {
        self.lock().timelines.values().any(|timeline| {
            timeline
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .IsLinkedToScroller(scroller)
        })
    }

    pub(crate) fn IsElementAnimating(&self, element: ElementId) -> bool {
        self.lock()
            .elements
            .get(&element)
            .is_some_and(|entry| !entry.animations.is_empty())
    }

    fn TickingAnimations(&self) -> Vec<AnimationHandle> {
        let state = self.lock();
        state
            .ticking
            .iter()
            .filter_map(|id| state.animations.get(id).cloned())
            .collect()
    }

    fn ElementAnimations(&self, element: ElementId) -> Vec<AnimationHandle> {
        let state = self.lock();
        let Some(entry) = state.elements.get(&element) else {
            return Vec::new();
        };
        entry
            .animations
            .iter()
            .filter_map(|id| state.animations.get(id).cloned())
            .collect()
    }
}

struct NativeAnimationEvents {
    events: AnimationEvents<Curve>,
}
impl super::host_owners::NativeMutatorEvents for NativeAnimationEvents {
    fn IsEmpty(&self) -> bool {
        self.events.IsEmpty()
    }
    fn WithScrollAnimationEvents(&mut self, callback: &mut dyn FnMut(&mut AnimationEvents<Curve>)) {
        callback(&mut self.events)
    }
}

struct NativeKeyframeTrace;
impl KeyframeModelTrace for NativeKeyframeTrace {
    fn Begin(&self, _model_id: u64, _name: &str) {}
    fn End(&self, _model_id: u64) {}
    fn StateChange(&self, _name: &str, _old: &str, _new: &str) {}
}

struct NativeAnimationCall<'a> {
    owner: &'a NativeAnimationOwner,
    target: Option<&'a mut dyn AnimationMutationTarget>,
    property_trees: Option<&'a PropertyTreesFor<NativeCompositorTypes>>,
    selected_element: Option<ElementId>,
    now: TimeTicks,
    active: bool,
}

/// Same-stack bridge used only while the selected Animation is dispatching a
/// finished event. All operations still execute the sole ScrollImpl source
/// body; matching the selected allocation merely supplies its already-borrowed
/// KeyframeEffect instead of locking the same Animation again.
struct NativeScrollAnimationReentry<'owner, 'selected> {
    owner: &'owner NativeAnimationOwner,
    element: ElementId,
    selected: BorrowedScrollAnimationReceiver<'selected>,
}
impl ScrollAnimationReentry for NativeScrollAnimationReentry<'_, '_> {
    fn MatchesElement(&self, element: ElementId) -> bool {
        self.element == element
    }

    fn CreateAutoScrollAnimation(
        &mut self,
        element: ElementId,
        target: PointF,
        current: PointF,
        velocity: f32,
        start: TimeDelta,
    ) {
        if !self.MatchesElement(element) {
            self.owner
                .CreateAutoScrollAnimation(element, target, current, velocity, start);
            return;
        }
        let owner = self.owner.clone();
        owner.WithScroll(None, Some(element), |scroll, call| {
            scroll.AutoScrollAnimationCreateForEffect(
                element,
                target,
                current,
                velocity,
                start,
                Some(&mut self.selected),
                call,
            )
        });
    }

    fn CreateMouseWheelScrollAnimation(
        &mut self,
        element: ElementId,
        target: PointF,
        current: PointF,
        delayed: TimeDelta,
        start: TimeDelta,
    ) {
        if !self.MatchesElement(element) {
            self.owner
                .CreateMouseWheelScrollAnimation(element, target, current, delayed, start);
            return;
        }
        let owner = self.owner.clone();
        owner.WithScroll(None, Some(element), |scroll, call| {
            scroll.MouseWheelScrollAnimationCreateForEffect(
                element,
                target,
                current,
                delayed,
                start,
                Some(&mut self.selected),
                call,
            )
        });
    }

    fn AbortScrollAnimation(&mut self, element: ElementId, needs_completion: bool) {
        if !self.MatchesElement(element) {
            self.owner.AbortScrollAnimation(element, needs_completion);
            return;
        }
        let owner = self.owner.clone();
        owner.WithScroll(None, Some(element), |scroll, call| {
            scroll.ScrollAnimationAbortForEffect(
                needs_completion,
                element,
                Some(&mut self.selected),
                call,
            )
        });
    }

    fn UpdateScrollAnimationTarget(
        &mut self,
        delta: Vector2dF,
        max: PointF,
        time: TimeTicks,
        delayed: TimeDelta,
        element: ElementId,
    ) -> Option<PointF> {
        if !self.MatchesElement(element) {
            return self
                .owner
                .UpdateScrollAnimationTarget(delta, max, time, delayed, element);
        }
        let owner = self.owner.clone();
        owner.WithScroll(None, Some(element), |scroll, call| {
            scroll.ScrollAnimationUpdateTargetForEffect(
                &delta,
                &max,
                time,
                delayed,
                element,
                Some(&mut self.selected),
                call,
            )
        })
    }

    fn ElementHasImplOnlyScrollAnimation(&self, element: ElementId) -> bool {
        if !self.MatchesElement(element) {
            return self.owner.ElementHasImplOnlyScrollAnimation(element);
        }
        self.owner
            .lock()
            .scroll
            .as_ref()
            .expect("canonical scroll animation owner")
            .ElementHasImplOnlyScrollAnimationForEffect(element, Some(&self.selected))
    }
}
impl<'a> NativeAnimationCall<'a> {
    fn detached(owner: &'a NativeAnimationOwner) -> Self {
        Self {
            owner,
            target: None,
            property_trees: None,
            selected_element: None,
            now: TimeTicks::default(),
            active: true,
        }
    }
    fn targeted(
        owner: &'a NativeAnimationOwner,
        target: &'a mut dyn AnimationMutationTarget,
    ) -> Self {
        Self {
            owner,
            target: Some(target),
            property_trees: None,
            selected_element: None,
            now: TimeTicks::default(),
            active: true,
        }
    }

    fn with_property_trees(
        owner: &'a NativeAnimationOwner,
        property_trees: &'a PropertyTreesFor<NativeCompositorTypes>,
    ) -> Self {
        Self {
            owner,
            target: None,
            property_trees: Some(property_trees),
            selected_element: None,
            now: TimeTicks::default(),
            active: true,
        }
    }
}

struct NativeScrollTickTarget<'a> {
    target: &'a mut dyn AnimationMutationTarget,
    element: ElementId,
    active: bool,
}
impl ScrollCurveTarget<KeyframeModel<Curve>> for NativeScrollTickTarget<'_> {
    fn OnScrollOffsetAnimated(
        &mut self,
        value: PointF,
        _target_property_id: i32,
        model: &KeyframeModel<Curve>,
    ) {
        let element = if model.element_id().IsValid() {
            model.element_id()
        } else {
            self.element
        };
        let list = if self.active {
            ElementListType::ACTIVE
        } else {
            ElementListType::PENDING
        };
        self.target
            .SetElementScrollOffsetMutated(element, list, &value);
    }
}

impl AnimationContext<Curve> for NativeAnimationCall<'_> {
    fn Now(&self) -> TimeTicks {
        self.now
    }
    fn TickCurve(
        &mut self,
        curve: &Curve,
        time: TimeDelta,
        property: i32,
        model: &KeyframeModel<Curve>,
        _limit: LimitDirection,
    ) {
        if let Some(target) = self.target.as_deref_mut() {
            let element = self.selected_element.expect("selected animation element");
            curve.Tick(
                time,
                property,
                model,
                Some(&mut NativeScrollTickTarget {
                    target,
                    element,
                    active: self.active,
                }),
            );
        }
    }
    fn HostIsOwnerThread(&self, host: AnimationHostId) -> bool {
        host == self.owner.host
    }
    fn HostInProtectedSequence(&self, _host: AnimationHostId) -> bool {
        false
    }
    fn HostWaitForProtectedSequenceCompletion(&self, _host: AnimationHostId) {}
    fn HostRegisterAnimationForElement(
        &mut self,
        host: AnimationHostId,
        element: ElementId,
        animation: i32,
    ) -> Arc<dyn BoundElementAnimations> {
        assert_eq!(host, self.owner.host);
        let mut state = self.owner.lock();
        assert!(state.animations.contains_key(&animation));
        let entry = state.elements.entry(element).or_default();
        entry.animations.insert(animation);
        entry
            .handle
            .get_or_insert_with(|| Arc::new(NativeElementAnimations { element }))
            .clone()
    }
    fn HostUnregisterAnimationForElement(
        &mut self,
        host: AnimationHostId,
        element: ElementId,
        animation: i32,
    ) {
        assert_eq!(host, self.owner.host);
        let mut state = self.owner.lock();
        let remove = state.elements.get_mut(&element).is_some_and(|entry| {
            entry.animations.remove(&animation);
            entry.animations.is_empty()
        });
        if remove {
            state.elements.remove(&element);
        }
        state.ticking.remove(&animation);
    }
    fn HostAddToTicking(&mut self, host: AnimationHostId, animation: i32) {
        assert_eq!(host, self.owner.host);
        let mut state = self.owner.lock();
        assert!(state.animations.contains_key(&animation));
        assert!(state.ticking.insert(animation));
    }
    fn HostRemoveFromTicking(&mut self, host: AnimationHostId, animation: i32) {
        assert_eq!(host, self.owner.host);
        self.owner.lock().ticking.remove(&animation);
    }
    fn HostSetNeedsCommit(&mut self, _host: AnimationHostId) {
        if let Some(target) = self.target.as_deref_mut() {
            target.SetMutatorsNeedCommit();
        }
    }
    fn TimelineIsScrollTimeline(&self, timeline: i32) -> bool {
        self.owner
            .lock()
            .timelines
            .get(&timeline)
            .is_some_and(|timeline| {
                timeline
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .IsScrollTimeline()
            })
    }
    fn TimelineSetNeedsPushProperties(&mut self, timeline: i32) {
        let selected = self.owner.lock().timelines.get(&timeline).cloned();
        if let Some(selected) = selected {
            if let Ok(mut timeline) = selected.try_lock() {
                timeline.SetNeedsPushProperties(self);
            } else if let Some(target) = self.target.as_deref_mut() {
                // The selected timeline's outer source method sets the same
                // resident flag before returning; only its Host notification
                // remains to be delivered from this nested call.
                target.SetMutatorsNeedCommit();
            }
        }
    }
    fn ElementSetNeedsPushProperties(&mut self, _element: &Arc<dyn BoundElementAnimations>) {
        if let Some(target) = self.target.as_deref_mut() {
            target.SetMutatorsNeedCommit();
        }
    }
    fn ElementUpdateClientAnimationState(&mut self, _element: &Arc<dyn BoundElementAnimations>) {
        if let Some(target) = self.target.as_deref_mut() {
            target.SetMutatorsNeedRebuildPropertyTrees();
        }
    }
    fn ElementAttachToCurve(
        &mut self,
        element: &Arc<dyn BoundElementAnimations>,
        curve: &mut Curve,
    ) {
        if !curve.ScrollHasSetInitialValue() {
            let value = self
                .ElementScrollOffsetForAnimation(element)
                .unwrap_or_default();
            curve.ScrollSetInitialValue(value);
        }
    }
    fn UpdateClientAnimationStateForElement(&mut self, _element: ElementId) {
        if let Some(target) = self.target.as_deref_mut() {
            target.SetMutatorsNeedRebuildPropertyTrees();
        }
    }
    fn ElementScrollOffsetForAnimation(
        &self,
        element: &Arc<dyn BoundElementAnimations>,
    ) -> Option<PointF> {
        if let Some(target) = self.target.as_deref() {
            return Some(target.CurrentScrollOffset(element.element_id(), ElementListType::ACTIVE));
        }
        self.property_trees.map(|trees| {
            trees
                .scroll_tree()
                .current_scroll_offset(element.element_id(), trees.state())
        })
    }
    fn QueueAnimationEvent(
        &mut self,
        animation_id: i32,
        event: &AnimationPlaybackEvent<Curve>,
        receiver: &mut KeyframeEffect<Curve>,
        host: Option<AnimationHostId>,
        timeline: Option<i32>,
        delegate: Option<AnimationDelegateId>,
    ) {
        if !event.is_impl_only {
            panic!("non-impl animation events must use the resident AnimationEvents owner");
        }
        if event.kind == PlaybackType::kFinished {
            let Some(element) = self.selected_element else {
                return;
            };
            let Some(mut target) = self.target.take() else {
                return;
            };
            let animation = self
                .owner
                .lock()
                .animations
                .get(&animation_id)
                .cloned()
                .expect("dispatching resident Animation");
            let owner = self.owner;
            let mut detached = NativeAnimationCall::detached(owner);
            Animation::<Curve>::WithExternalEffectContext(
                animation_id,
                host,
                timeline,
                delegate,
                &mut detached,
                |effect_context| {
                    let selected = BorrowedScrollAnimationReceiver {
                        animation: &animation,
                        effect: receiver,
                        context: effect_context,
                    };
                    let mut reentry = NativeScrollAnimationReentry {
                        owner,
                        element,
                        selected,
                    };
                    target.ScrollOffsetAnimationFinished(element, Some(&mut reentry));
                },
            );
            self.target = Some(target);
        }
    }
    fn DelegateAnimationStarted(
        &mut self,
        _delegate: AnimationDelegateId,
        _time: TimeTicks,
        _property: i32,
        _group: i32,
    ) {
    }
    fn DelegateAnimationFinished(
        &mut self,
        _delegate: AnimationDelegateId,
        _time: TimeTicks,
        _property: i32,
        _group: i32,
    ) {
        if let (Some(target), Some(element)) = (self.target.as_deref_mut(), self.selected_element) {
            target.ScrollOffsetAnimationFinished(element, None);
        }
    }
    fn DelegateAnimationAborted(
        &mut self,
        _delegate: AnimationDelegateId,
        _time: TimeTicks,
        _property: i32,
        _group: i32,
    ) {
    }
    fn DelegateAnimationTakeover(
        &mut self,
        _delegate: AnimationDelegateId,
        _time: TimeTicks,
        _property: i32,
        _animation_start_time: TimeTicks,
        _curve: Curve,
    ) {
    }
    fn DelegateLocalTimeUpdated(
        &mut self,
        _delegate: AnimationDelegateId,
        _local_time: Option<TimeDelta>,
    ) {
    }
    fn TraceEffect(&self, _event: &'static str, _message: String) {}
}

impl AnimationTimelineContext<ScrollAnimationTypes> for NativeAnimationCall<'_> {
    type ScrollTree = crate::cc::trees::property_tree::ScrollTree;
    fn AnimationId(&self, animation: &AnimationHandle) -> i32 {
        animation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .id()
    }
    fn AnimationTimelineId(&self, animation: &AnimationHandle) -> Option<i32> {
        animation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .animation_timeline()
    }
    fn SetAnimationHost(&mut self, animation: &AnimationHandle, host: Option<AnimationHostId>) {
        animation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .SetAnimationHost(host, self)
    }
    fn SetAnimationTimeline(&mut self, animation: &AnimationHandle, timeline: Option<i32>) {
        let id = {
            let mut selected = animation
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let id = selected.id();
            selected.SetAnimationTimeline(timeline, self);
            id
        };
        let mut state = self.owner.lock();
        if timeline.is_some() {
            state.animations.insert(id, animation.clone());
        } else {
            state.animations.remove(&id);
            state.ticking.remove(&id);
        }
    }
    fn CreateImplInstance(&mut self, animation: &AnimationHandle) -> AnimationHandle {
        animation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .CreateImplInstance()
    }
    fn HasElementAnimations(&self, animation: &AnimationHandle) -> bool {
        animation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .element_animations()
            .is_some()
    }
    fn DetachElement(&mut self, animation: &AnimationHandle) {
        let mut animation = animation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.selected_element = Some(animation.element_id());
        animation.DetachElement(self)
    }
    fn IsWorkletAnimation(&self, animation: &AnimationHandle) -> bool {
        animation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .IsWorkletAnimation()
    }
    fn IsScrollLinkedAnimation(&self, animation: &AnimationHandle) -> bool {
        animation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .IsScrollLinkedAnimation(self)
    }
    fn AwaitingDeletion(&self, animation: &AnimationHandle) -> bool {
        animation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .keyframe_effect()
            .awaiting_deletion()
    }
    fn Tick(&mut self, animation: &AnimationHandle, time: TimeTicks) -> bool {
        let mut animation = animation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.selected_element = Some(animation.element_id());
        animation.Tick(time, self)
    }
    fn PushAnimationPropertiesTo(
        &mut self,
        animation: &AnimationHandle,
        animation_impl: &AnimationHandle,
    ) {
        let mut source = animation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut target = animation_impl
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        source.PushPropertiesTo(&mut target, self)
    }
    fn HostIsOwnerThread(&self, host: AnimationHostId) -> bool {
        host == self.owner.host
    }
    fn HostInProtectedSequence(&self, _host: AnimationHostId) -> bool {
        false
    }
    fn HostWaitForProtectedSequenceCompletion(&self, _host: AnimationHostId) {}
    fn HostSetNeedsPushProperties(&mut self, _host: AnimationHostId) {
        self.owner.lock().needs_push_properties = true;
        if let Some(target) = self.target.as_deref_mut() {
            target.SetMutatorsNeedCommit();
        }
    }
}

impl ScrollOffsetAnimationsContext for NativeAnimationCall<'_> {
    fn AnimationHostIdentity(&self) -> AnimationHostId {
        self.owner.host
    }
    fn AddAnimationTimeline(&mut self, timeline: ScrollTimelineRef) {
        timeline
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .SetAnimationHost(Some(self.owner.host), self);
        let id = timeline
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .id();
        assert!(self.owner.lock().timelines.insert(id, timeline).is_none());
    }
    fn RemoveAnimationTimeline(&mut self, timeline: &ScrollTimelineRef) {
        let id = timeline
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .id();
        let removed = self.owner.lock().timelines.remove(&id);
        assert!(removed.is_some());
        timeline
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .SetAnimationHost(None, self);
    }
    fn ScrollCurveConfig(&self) -> Arc<ScrollCurveConfig> {
        self.owner.config.clone()
    }
    fn KeyframeTrace(&self) -> Arc<dyn KeyframeModelTrace> {
        self.owner.trace.clone()
    }
    fn ScrollOffsetAnimationFinished(&mut self, element: ElementId) {
        if let Some(target) = self.target.as_deref_mut() {
            target.ScrollOffsetAnimationFinished(element, None)
        }
    }
    fn IsElementInPropertyTrees(&self, element: ElementId, commits_to_active: bool) -> bool {
        if let Some(target) = self.target.as_deref() {
            target.IsElementInPropertyTrees(
                element,
                if commits_to_active {
                    ElementListType::ACTIVE
                } else {
                    ElementListType::PENDING
                },
            )
        } else {
            self.owner.lock().elements.contains_key(&element)
        }
    }
    fn TraceScrollAnimation(&self, _event: &'static str, _number: Option<f64>) {}
}

struct NativeAnimationQueries {
    owner: NativeAnimationOwner,
}
impl LayerTreeMutatorHost for NativeAnimationQueries {
    fn HasAnyAnimationTargetingProperty(
        &self,
        element: ElementId,
        property: TargetProperty,
    ) -> bool {
        self.owner
            .ElementAnimations(element)
            .iter()
            .any(|animation| {
                animation
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .keyframe_effect()
                    .keyframe_models()
                    .iter()
                    .any(|model| model.TargetProperty() == property as i32)
            })
    }
    fn HasPotentiallyRunningAnimationForProperty(
        &self,
        element: ElementId,
        list: ElementListType,
        property: TargetProperty,
    ) -> bool {
        self.owner
            .ElementAnimations(element)
            .iter()
            .any(|animation| {
                animation
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .keyframe_effect()
                    .IsPotentiallyAnimatingProperty(property as i32, list)
            })
    }
    fn MaximumScale(&self, element: ElementId, list: ElementListType) -> f32 {
        let mut found = false;
        let mut maximum = 0.0_f32;
        for animation in self.owner.ElementAnimations(element) {
            let scale = animation
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .keyframe_effect()
                .MaximumScale(element, list);
            if scale == 0.0 {
                return 0.0;
            }
            found = true;
            maximum = maximum.max(scale);
        }
        if found {
            maximum
        } else {
            0.0
        }
    }
}
impl PropertyTreeMutatorHost for NativeAnimationQueries {
    fn ScrollOffsetAnimationWasInterrupted(&self, element: ElementId) -> bool {
        self.owner
            .ElementAnimations(element)
            .iter()
            .any(|animation| {
                animation
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .keyframe_effect()
                    .scroll_offset_animation_was_interrupted()
            })
    }
}
impl super::input::NativeMutatorHost for NativeAnimationQueries {
    fn AsAnyMut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn SetLayerTreeMutator(&mut self, mutator: Option<Box<super::host_owners::LayerTreeMutator>>) {
        self.owner.lock().layer_tree_mutator = mutator;
    }
    fn identity(&self) -> super::types::MainMutatorHostIdentity {
        self.owner.identity
    }
    fn NextFrameHasPendingRAF(&self) -> bool {
        self.owner.lock().next_frame_has_pending_raf
    }
    fn HasCanvasInvalidation(&self) -> bool {
        self.owner.lock().has_canvas_invalidation
    }
    fn HasJSAnimation(&self) -> bool {
        self.owner.lock().has_js_animation
    }
    fn MainThreadAnimationsCount(&self) -> usize {
        self.owner.lock().main_thread_animations_count
    }
    fn HasSmilAnimation(&self) -> bool {
        self.owner.lock().has_smil_animation
    }
}
