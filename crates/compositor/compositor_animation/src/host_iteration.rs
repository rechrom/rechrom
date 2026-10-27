//! AnimationHost timeline and ticking-set iteration.

pub trait TimelineTickContext {
    fn timeline_count(&self) -> usize;
    fn tick_timeline(&mut self, index: usize) -> bool;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimelineTickStats {
    pub timelines_visited: usize,
    pub timelines_animated: usize,
    pub animated: bool,
}

pub fn TickAnimationTimelines<C: TimelineTickContext>(context: &mut C) -> TimelineTickStats {
    let mut trace =
        browser_tracing::span("compositor.animation", "Compositor.Animation.TickTimelines");
    let mut stats = TimelineTickStats::default();
    for index in 0..context.timeline_count() {
        stats.timelines_visited += 1;
        if context.tick_timeline(index) {
            stats.timelines_animated += 1;
            stats.animated = true;
        }
    }
    trace.set("timelines_visited", stats.timelines_visited as f64);
    trace.set("timelines_animated", stats.timelines_animated as f64);
    trace.set("animated", stats.animated as u8 as f64);
    stats
}

pub trait WorkletTickContext {
    fn animation_count(&self) -> usize;
    fn tick_if_worklet_animation(&mut self, index: usize) -> bool;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorkletTickStats {
    pub animations_visited: usize,
    pub worklet_animations_ticked: usize,
}

pub fn TickWorkletAnimations<C: WorkletTickContext>(context: &mut C) -> WorkletTickStats {
    let mut trace = browser_tracing::span(
        "compositor.animation",
        "Compositor.Animation.TickWorkletAnimations",
    );
    let mut stats = WorkletTickStats::default();
    for index in 0..context.animation_count() {
        stats.animations_visited += 1;
        if context.tick_if_worklet_animation(index) {
            stats.worklet_animations_ticked += 1;
        }
    }
    trace.set("animations_visited", stats.animations_visited as f64);
    trace.set(
        "worklet_animations_ticked",
        stats.worklet_animations_ticked as f64,
    );
    stats
}

pub trait TimelineActivationContext {
    fn timeline_count(&self) -> usize;
    fn activate_timeline(&mut self, index: usize);
}

pub fn ActivateTimelines<C: TimelineActivationContext>(context: &mut C) -> usize {
    let mut trace = browser_tracing::span(
        "compositor.animation",
        "Compositor.Animation.ActivateTimelines",
    );
    let count = context.timeline_count();
    for index in 0..count {
        context.activate_timeline(index);
    }
    trace.set("timelines_activated", count as f64);
    count
}

pub trait AnimationStateUpdateContext {
    fn animation_count(&self) -> usize;
    fn update_animation_state(&mut self, index: usize);
}

pub fn UpdateAnimationStates<C: AnimationStateUpdateContext>(context: &mut C) -> usize {
    let mut trace = browser_tracing::span(
        "compositor.animation",
        "Compositor.Animation.UpdateAnimationStates",
    );
    let count = context.animation_count();
    for index in 0..count {
        context.update_animation_state(index);
    }
    trace.set("animations_updated", count as f64);
    count
}

pub trait AnimationActivationContext {
    fn animation_count(&self) -> usize;
    fn activate_animation(&mut self, index: usize);
}

pub fn ActivateAnimations<C: AnimationActivationContext>(context: &mut C) -> usize {
    let mut trace = browser_tracing::span(
        "compositor.animation",
        "Compositor.Animation.ActivateAnimations",
    );
    let count = context.animation_count();
    for index in 0..count {
        context.activate_animation(index);
    }
    trace.set("animations_activated", count as f64);
    count
}

pub trait TimelinePropertyPushContext<S, T> {
    fn source_timeline_id(&mut self, source: &S) -> i32;
    fn target_contains_timeline(&mut self, id: i32) -> bool;
    fn create_target_timeline(&mut self, source: &S, id: i32);
    fn target_timelines(&mut self) -> Vec<(i32, T)>;
    fn target_timeline_is_impl_only(&mut self, target: &T) -> bool;
    fn remove_target_timeline(&mut self, id: i32, target: T);
    fn target_timeline(&mut self, id: i32) -> T;
    fn push_timeline_properties(&mut self, source: S, target: T);
    fn mark_target_needs_push_properties(&mut self);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimelinePropertyPushStats {
    pub source_timelines: usize,
    pub created_timelines: usize,
    pub removed_timelines: usize,
    pub impl_only_timelines_kept: usize,
    pub timelines_pushed: usize,
}

pub fn PushTimelineProperties<S, T, C>(
    source_timelines: Vec<S>,
    context: &mut C,
) -> TimelinePropertyPushStats
where
    C: TimelinePropertyPushContext<S, T>,
{
    let mut trace = browser_tracing::span(
        "compositor.animation",
        "Compositor.Animation.PushTimelineProperties",
    );
    let mut stats = TimelinePropertyPushStats {
        source_timelines: source_timelines.len(),
        ..Default::default()
    };
    let source_ids = source_timelines
        .iter()
        .map(|timeline| context.source_timeline_id(timeline))
        .collect::<std::collections::BTreeSet<_>>();

    for source_timeline in &source_timelines {
        let id = context.source_timeline_id(source_timeline);
        if context.target_contains_timeline(id) {
            continue;
        }
        context.create_target_timeline(source_timeline, id);
        stats.created_timelines += 1;
    }

    let stale = context
        .target_timelines()
        .into_iter()
        .filter_map(|(id, timeline)| {
            if source_ids.contains(&id) {
                return None;
            }
            if context.target_timeline_is_impl_only(&timeline) {
                stats.impl_only_timelines_kept += 1;
                return None;
            }
            Some((id, timeline))
        })
        .collect::<Vec<_>>();
    for (id, timeline) in stale {
        context.remove_target_timeline(id, timeline);
        stats.removed_timelines += 1;
    }

    for source_timeline in source_timelines {
        let id = context.source_timeline_id(&source_timeline);
        let target_timeline = context.target_timeline(id);
        context.push_timeline_properties(source_timeline, target_timeline);
        stats.timelines_pushed += 1;
    }
    context.mark_target_needs_push_properties();

    trace.set("source_timelines", stats.source_timelines as f64);
    trace.set("created_timelines", stats.created_timelines as f64);
    trace.set("removed_timelines", stats.removed_timelines as f64);
    trace.set(
        "impl_only_timelines_kept",
        stats.impl_only_timelines_kept as f64,
    );
    trace.set("timelines_pushed", stats.timelines_pushed as f64);
    stats
}

pub trait TimeUpdatedEventContext {
    fn animation_count(&self) -> usize;
    fn take_time_updated_event(&mut self, index: usize);
}

pub fn TakeTimeUpdatedEvents<C: TimeUpdatedEventContext>(context: &mut C) -> usize {
    let mut trace = browser_tracing::span(
        "compositor.animation",
        "Compositor.Animation.TakeTimeUpdatedEvents",
    );
    let count = context.animation_count();
    for index in 0..count {
        context.take_time_updated_event(index);
    }
    trace.set("animations_visited", count as f64);
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_tick_aggregates_without_short_circuiting() {
        struct Context(Vec<bool>, Vec<usize>);
        impl TimelineTickContext for Context {
            fn timeline_count(&self) -> usize {
                self.0.len()
            }
            fn tick_timeline(&mut self, index: usize) -> bool {
                self.1.push(index);
                self.0[index]
            }
        }
        let mut context = Context(vec![false, true, true], Vec::new());
        let stats = TickAnimationTimelines(&mut context);
        assert_eq!(context.1, [0, 1, 2]);
        assert_eq!(stats.timelines_animated, 2);
        assert!(stats.animated);
    }

    #[test]
    fn worklet_tick_filters_but_counts_complete_ticking_set() {
        struct Context {
            worklets: Vec<bool>,
            ticked: Vec<usize>,
        }
        impl WorkletTickContext for Context {
            fn animation_count(&self) -> usize {
                self.worklets.len()
            }
            fn tick_if_worklet_animation(&mut self, index: usize) -> bool {
                if self.worklets[index] {
                    self.ticked.push(index);
                    true
                } else {
                    false
                }
            }
        }
        let mut context = Context {
            worklets: vec![true, false, true],
            ticked: Vec::new(),
        };
        let stats = TickWorkletAnimations(&mut context);
        assert_eq!(context.ticked, [0, 2]);
        assert_eq!(stats.animations_visited, 3);
        assert_eq!(stats.worklet_animations_ticked, 2);
    }

    #[test]
    fn activation_preserves_timeline_order() {
        struct Context(Vec<usize>);
        impl TimelineActivationContext for Context {
            fn timeline_count(&self) -> usize {
                3
            }
            fn activate_timeline(&mut self, index: usize) {
                self.0.push(index);
            }
        }
        let mut context = Context(Vec::new());
        assert_eq!(ActivateTimelines(&mut context), 3);
        assert_eq!(context.0, [0, 1, 2]);
    }

    #[test]
    fn state_update_visits_complete_ticking_set() {
        struct Context(Vec<usize>);
        impl AnimationStateUpdateContext for Context {
            fn animation_count(&self) -> usize {
                3
            }
            fn update_animation_state(&mut self, index: usize) {
                self.0.push(index);
            }
        }
        let mut context = Context(Vec::new());
        assert_eq!(UpdateAnimationStates(&mut context), 3);
        assert_eq!(context.0, [0, 1, 2]);
    }

    #[test]
    fn animation_activation_preserves_ticking_order() {
        struct Context(Vec<usize>);
        impl AnimationActivationContext for Context {
            fn animation_count(&self) -> usize {
                2
            }
            fn activate_animation(&mut self, index: usize) {
                self.0.push(index);
            }
        }
        let mut context = Context(Vec::new());
        assert_eq!(ActivateAnimations(&mut context), 2);
        assert_eq!(context.0, [0, 1]);
    }

    #[test]
    fn timeline_property_push_creates_removes_preserves_and_pushes_in_order() {
        #[derive(Default)]
        struct Context {
            targets: std::collections::BTreeMap<i32, bool>,
            log: Vec<String>,
            marked: bool,
        }
        impl TimelinePropertyPushContext<i32, i32> for Context {
            fn source_timeline_id(&mut self, source: &i32) -> i32 {
                *source
            }
            fn target_contains_timeline(&mut self, id: i32) -> bool {
                self.targets.contains_key(&id)
            }
            fn create_target_timeline(&mut self, _source: &i32, id: i32) {
                self.targets.insert(id, false);
                self.log.push(format!("create:{id}"));
            }
            fn target_timelines(&mut self) -> Vec<(i32, i32)> {
                self.targets.keys().map(|id| (*id, *id)).collect()
            }
            fn target_timeline_is_impl_only(&mut self, target: &i32) -> bool {
                self.targets[target]
            }
            fn remove_target_timeline(&mut self, id: i32, _target: i32) {
                self.targets.remove(&id);
                self.log.push(format!("remove:{id}"));
            }
            fn target_timeline(&mut self, id: i32) -> i32 {
                id
            }
            fn push_timeline_properties(&mut self, source: i32, target: i32) {
                self.log.push(format!("push:{source}->{target}"));
            }
            fn mark_target_needs_push_properties(&mut self) {
                self.marked = true;
            }
        }

        let mut context = Context {
            targets: [(1, false), (7, false), (9, true)].into_iter().collect(),
            ..Default::default()
        };
        let stats = PushTimelineProperties(vec![1, 2], &mut context);
        assert_eq!(
            context.log,
            ["create:2", "remove:7", "push:1->1", "push:2->2"]
        );
        assert!(context.targets.contains_key(&9));
        assert!(context.marked);
        assert_eq!(stats.created_timelines, 1);
        assert_eq!(stats.removed_timelines, 1);
        assert_eq!(stats.impl_only_timelines_kept, 1);
        assert_eq!(stats.timelines_pushed, 2);
    }

    #[test]
    fn time_updated_event_walk_visits_complete_ticking_set() {
        struct Context(Vec<usize>);
        impl TimeUpdatedEventContext for Context {
            fn animation_count(&self) -> usize {
                3
            }
            fn take_time_updated_event(&mut self, index: usize) {
                self.0.push(index);
            }
        }
        let mut context = Context(Vec::new());
        assert_eq!(TakeTimeUpdatedEvents(&mut context), 3);
        assert_eq!(context.0, [0, 1, 2]);
    }
}
