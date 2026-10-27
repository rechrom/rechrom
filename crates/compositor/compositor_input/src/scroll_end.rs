//! Scroll-end latching, deferral, snap, and cleanup policy.

pub trait ScrollEndContext {
    type Node: Clone + Eq;
    type ElementId: Copy + Eq;

    fn currently_scrolling_node(&mut self) -> Option<Self::Node>;
    fn updates_need_main_thread_repaint(&mut self, node: &Self::Node) -> bool;
    fn node_element_id(&mut self, node: &Self::Node) -> Self::ElementId;
    fn insert_pending_scrollend_container(&mut self, element_id: Self::ElementId);
    fn remove_deferred_scroll_end(&mut self, element_id: Self::ElementId);
    fn deferred_scroll_end_exists(&mut self, element_id: Self::ElementId) -> bool;
    fn insert_deferred_scroll_end(&mut self, element_id: Self::ElementId);
    fn reset_scrollbar_state(&mut self);
    fn end_of_scroll_cleanup(&mut self, node: &Self::Node);
    fn remove_snap_animation_data(&mut self, element_id: Self::ElementId);
    fn current_scroller_is_animating(&mut self) -> bool;
    fn has_elastic_overscroll(&mut self, element_id: Self::ElementId) -> bool;
    fn snap_at_scroll_end(&mut self) -> bool;
    fn has_latched_scroll_type(&self) -> bool;
    fn gesture_consumed_scroll_delta(&self) -> bool;
    fn clear_currently_scrolling_node(&mut self);
    fn request_commit(&mut self);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollEndOutcome {
    #[default]
    NoScroller,
    ExplicitNodeCleanup,
    LatchedNodeCleanup,
    DeferredForAnimation,
    DeferredForElasticOverscroll,
    DeferredForSnap,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollEndResult {
    pub updates_need_main_thread_repaint: bool,
    pub outcome: ScrollEndOutcome,
    pub requested_commit: bool,
}

pub fn FinishScroll<C: ScrollEndContext>(
    context: &mut C,
    explicit_node: Option<C::Node>,
    snap: bool,
) -> ScrollEndResult {
    let mut trace = browser_tracing::span("compositor.input", "Compositor.Input.ScrollEnd");
    let latched = context.currently_scrolling_node();
    let mut result = ScrollEndResult::default();
    if let Some(node) = explicit_node.as_ref().or(latched.as_ref()) {
        result.updates_need_main_thread_repaint = context.updates_need_main_thread_repaint(node);
    }

    if explicit_node.is_some() && explicit_node != latched {
        debug_assert!(!snap);
        let node = explicit_node.unwrap();
        let id = context.node_element_id(&node);
        context.insert_pending_scrollend_container(id);
        context.remove_deferred_scroll_end(id);
        if latched.is_none() {
            context.reset_scrollbar_state();
            context.end_of_scroll_cleanup(&node);
        }
        context.remove_snap_animation_data(id);
        result.outcome = ScrollEndOutcome::ExplicitNodeCleanup;
    } else if let Some(node) = latched {
        let id = context.node_element_id(&node);
        context.reset_scrollbar_state();
        if context.current_scroller_is_animating() {
            debug_assert!(!context.deferred_scroll_end_exists(id));
            context.insert_deferred_scroll_end(id);
            result.outcome = ScrollEndOutcome::DeferredForAnimation;
            finish_trace(&mut trace, result);
            return result;
        }
        if context.has_elastic_overscroll(id) {
            context.insert_deferred_scroll_end(id);
            result.outcome = ScrollEndOutcome::DeferredForElasticOverscroll;
            finish_trace(&mut trace, result);
            return result;
        }
        if snap && context.snap_at_scroll_end() {
            context.insert_deferred_scroll_end(id);
            result.outcome = ScrollEndOutcome::DeferredForSnap;
            finish_trace(&mut trace, result);
            return result;
        }
        debug_assert!(context.has_latched_scroll_type());
        if context.gesture_consumed_scroll_delta() {
            context.insert_pending_scrollend_container(id);
        }
        context.end_of_scroll_cleanup(&node);
        context.remove_snap_animation_data(id);
        context.clear_currently_scrolling_node();
        result.outcome = ScrollEndOutcome::LatchedNodeCleanup;
    } else {
        context.reset_scrollbar_state();
        finish_trace(&mut trace, result);
        return result;
    }

    context.request_commit();
    result.requested_commit = true;
    finish_trace(&mut trace, result);
    result
}

fn finish_trace(trace: &mut browser_tracing::Span, result: ScrollEndResult) {
    trace.set(
        "main_thread_repaint",
        result.updates_need_main_thread_repaint as u8 as f64,
    );
    trace.set("outcome", result.outcome as u8 as f64);
    trace.set("requested_commit", result.requested_commit as u8 as f64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[derive(Default)]
    struct Context {
        latched: Option<u64>,
        animating: bool,
        overscrolled: bool,
        snaps: bool,
        consumed: bool,
        deferred: BTreeSet<u64>,
        events: Vec<String>,
    }

    impl ScrollEndContext for Context {
        type Node = u64;
        type ElementId = u64;
        fn currently_scrolling_node(&mut self) -> Option<u64> {
            self.latched
        }
        fn updates_need_main_thread_repaint(&mut self, node: &u64) -> bool {
            *node == 9
        }
        fn node_element_id(&mut self, node: &u64) -> u64 {
            *node
        }
        fn insert_pending_scrollend_container(&mut self, id: u64) {
            self.events.push(format!("pending-{id}"));
        }
        fn remove_deferred_scroll_end(&mut self, id: u64) {
            self.deferred.remove(&id);
            self.events.push(format!("remove-deferred-{id}"));
        }
        fn deferred_scroll_end_exists(&mut self, id: u64) -> bool {
            self.deferred.contains(&id)
        }
        fn insert_deferred_scroll_end(&mut self, id: u64) {
            self.deferred.insert(id);
            self.events.push(format!("defer-{id}"));
        }
        fn reset_scrollbar_state(&mut self) {
            self.events.push("reset-scrollbar".into());
        }
        fn end_of_scroll_cleanup(&mut self, node: &u64) {
            self.events.push(format!("cleanup-{node}"));
        }
        fn remove_snap_animation_data(&mut self, id: u64) {
            self.events.push(format!("remove-snap-{id}"));
        }
        fn current_scroller_is_animating(&mut self) -> bool {
            self.animating
        }
        fn has_elastic_overscroll(&mut self, _: u64) -> bool {
            self.overscrolled
        }
        fn snap_at_scroll_end(&mut self) -> bool {
            self.snaps
        }
        fn has_latched_scroll_type(&self) -> bool {
            true
        }
        fn gesture_consumed_scroll_delta(&self) -> bool {
            self.consumed
        }
        fn clear_currently_scrolling_node(&mut self) {
            self.latched = None;
            self.events.push("clear".into());
        }
        fn request_commit(&mut self) {
            self.events.push("commit".into());
        }
    }

    #[test]
    fn animation_defers_before_overscroll_and_snap() {
        let mut context = Context {
            latched: Some(7),
            animating: true,
            overscrolled: true,
            snaps: true,
            ..Default::default()
        };
        let result = FinishScroll(&mut context, None, true);
        assert_eq!(result.outcome, ScrollEndOutcome::DeferredForAnimation);
        assert_eq!(context.events, ["reset-scrollbar", "defer-7"]);
        assert!(!result.requested_commit);
    }

    #[test]
    fn latched_cleanup_enqueues_scrollend_and_commits() {
        let mut context = Context {
            latched: Some(9),
            consumed: true,
            ..Default::default()
        };
        let result = FinishScroll(&mut context, None, false);
        assert!(result.updates_need_main_thread_repaint);
        assert_eq!(result.outcome, ScrollEndOutcome::LatchedNodeCleanup);
        assert_eq!(
            context.events,
            [
                "reset-scrollbar",
                "pending-9",
                "cleanup-9",
                "remove-snap-9",
                "clear",
                "commit"
            ]
        );
    }

    #[test]
    fn explicit_non_latched_node_does_not_end_latched_scroller() {
        let mut context = Context {
            latched: Some(3),
            ..Default::default()
        };
        let result = FinishScroll(&mut context, Some(8), false);
        assert_eq!(result.outcome, ScrollEndOutcome::ExplicitNodeCleanup);
        assert_eq!(
            context.events,
            ["pending-8", "remove-deferred-8", "remove-snap-8", "commit"]
        );
        assert_eq!(context.latched, Some(3));
    }
}
