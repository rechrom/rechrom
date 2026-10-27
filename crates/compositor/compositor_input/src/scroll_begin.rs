//! Scroll-begin hit testing, latching, and viewport routing policy.

use crate::{
    input_handler_types::{ScrollStatus, ScrollThread},
    main_thread_scrolling_reason::{MainThreadHitTestReasons, MainThreadRepaintReasons},
};

pub struct ScrollBeginHitTest<Node> {
    pub scroll_node: Option<Node>,
    pub hit_test_successful: bool,
    pub main_thread_hit_test_reasons: MainThreadHitTestReasons,
}

pub trait ScrollBeginContext {
    type Node: Clone + Eq;
    type ElementId: Copy + Eq;

    fn state_delta_is_zero(&self) -> bool;
    fn precise_pixel_scroll(&self) -> bool;
    fn animating_node_for_current_scroller(&mut self) -> Option<Self::Node>;
    fn node_element_id(&mut self, node: &Self::Node) -> Self::ElementId;
    fn abort_scroll_animation(&mut self, element_id: Self::ElementId);
    fn currently_scrolling_node(&mut self) -> Option<Self::Node>;
    fn clear_animating_snap_targets(&mut self, element_id: Self::ElementId);
    fn input_matches_latched_type(&self) -> bool;
    fn node_has_snap_data(&mut self, node: &Self::Node) -> bool;
    fn deferred_scroll_end_exists(&self, element_id: Self::ElementId) -> bool;
    fn remove_deferred_scroll_end(&mut self, element_id: Self::ElementId);
    fn can_realize_scrolls_on_pending_tree(&mut self, node: &Self::Node) -> bool;
    fn clear_currently_scrolling_node(&mut self);

    fn target_element_is_valid(&self) -> bool;
    fn main_thread_hit_test_reasons_empty(&self) -> bool;
    fn scrollbar_interaction(&self) -> bool;
    fn trace_latched_node_provided(&mut self);
    fn trace_unlatched_node_provided(&mut self);
    fn trace_hit_testing(&mut self);
    fn find_target_node(&mut self) -> Option<Self::Node>;
    fn hit_test_scroll_node(&mut self) -> ScrollBeginHitTest<Self::Node>;
    fn trace_request_main_thread_hit_test(&mut self);
    fn find_node_to_latch(&mut self, starting: Option<Self::Node>) -> Option<Self::Node>;
    fn fluent_overlay_scrollbar_enabled(&self) -> bool;
    fn notify_will_scroll_content(&mut self, element_id: Self::ElementId);
    fn embedded_frame(&self) -> bool;
    fn trace_no_scroll_node(&mut self, embedded: bool);

    fn set_currently_scrolling_node(&mut self, node: Self::Node);
    fn node_main_thread_repaint_reasons(&mut self, node: &Self::Node) -> MainThreadRepaintReasons;
    fn did_latch_to_scroller(&mut self);
    fn viewport_should_scroll(&mut self, node: &Self::Node) -> bool;
    fn reset_viewport_consumed_delta(&mut self);
    fn viewport_can_scroll(&mut self, node: &Self::Node) -> bool;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollBeginOutcome {
    #[default]
    Latched,
    ContinuedExistingLatch,
    IgnoredMissingTarget,
    RequestedMainThreadHitTest,
    IgnoredNoScroller,
}

pub fn BeginScroll<C: ScrollBeginContext>(context: &mut C) -> (ScrollStatus, ScrollBeginOutcome) {
    debug_assert!(context.state_delta_is_zero());
    let mut trace = browser_tracing::span("compositor.input", "Compositor.Input.ScrollBegin");
    let mut status = ScrollStatus::default();
    let mut had_animation = false;
    if context.precise_pixel_scroll() {
        if let Some(node) = context.animating_node_for_current_scroller() {
            had_animation = true;
            let id = context.node_element_id(&node);
            context.abort_scroll_animation(id);
        }
        if let Some(node) = context.currently_scrolling_node() {
            let id = context.node_element_id(&node);
            context.clear_animating_snap_targets(id);
        }
    }

    if let Some(node) = context.currently_scrolling_node() {
        if context.input_matches_latched_type()
            && (context.animating_node_for_current_scroller().is_some()
                || had_animation
                || context.node_has_snap_data(&node))
        {
            let id = context.node_element_id(&node);
            debug_assert!(context.deferred_scroll_end_exists(id));
            context.remove_deferred_scroll_end(id);
            status.raster_inducing = context.can_realize_scrolls_on_pending_tree(&node);
            trace.set("continued_existing_latch", 1.0);
            return (status, ScrollBeginOutcome::ContinuedExistingLatch);
        }
    }

    context.clear_currently_scrolling_node();
    let target_valid = context.target_element_is_valid();
    let scrolling = if target_valid
        && (context.main_thread_hit_test_reasons_empty() || context.scrollbar_interaction())
    {
        context.trace_latched_node_provided();
        context.find_target_node()
    } else {
        let starting = if target_valid {
            context.trace_unlatched_node_provided();
            debug_assert!(!context.main_thread_hit_test_reasons_empty());
            let starting = context.find_target_node();
            if starting.is_none() {
                status.thread = ScrollThread::kScrollIgnored;
                trace.set("ignored_missing_target", 1.0);
                return (status, ScrollBeginOutcome::IgnoredMissingTarget);
            }
            starting
        } else {
            context.trace_hit_testing();
            debug_assert!(context.main_thread_hit_test_reasons_empty());
            let hit = context.hit_test_scroll_node();
            if !hit.hit_test_successful {
                context.trace_request_main_thread_hit_test();
                status.thread = ScrollThread::kScrollOnImplThread;
                debug_assert!(!hit.main_thread_hit_test_reasons.empty());
                status.main_thread_hit_test_reasons = hit.main_thread_hit_test_reasons;
                trace.set("requested_main_thread_hit_test", 1.0);
                return (status, ScrollBeginOutcome::RequestedMainThreadHitTest);
            }
            hit.scroll_node
        };
        let scrolling = context.find_node_to_latch(starting.clone());
        if let Some(node) = scrolling.as_ref() {
            if context.fluent_overlay_scrollbar_enabled() {
                let selected = context.node_element_id(node);
                let starting_id = context.node_element_id(starting.as_ref().unwrap());
                if selected != starting_id {
                    context.notify_will_scroll_content(starting_id);
                }
            }
        }
        scrolling
    };

    let Some(node) = scrolling else {
        let embedded = context.embedded_frame();
        context.trace_no_scroll_node(embedded);
        status.thread = ScrollThread::kScrollIgnored;
        trace.set("ignored_no_scroller", 1.0);
        return (status, ScrollBeginOutcome::IgnoredNoScroller);
    };
    debug_assert_eq!(status.thread, ScrollThread::kScrollOnImplThread);
    context.set_currently_scrolling_node(node.clone());
    status.main_thread_repaint_reasons = context.node_main_thread_repaint_reasons(&node);
    status.raster_inducing = context.can_realize_scrolls_on_pending_tree(&node);
    context.did_latch_to_scroller();
    let current = context.currently_scrolling_node().unwrap();
    if context.viewport_should_scroll(&current) {
        context.reset_viewport_consumed_delta();
        if !context.viewport_can_scroll(&current) {
            status.viewport_cannot_scroll = true;
        }
    }
    trace.set("raster_inducing", status.raster_inducing as u8 as f64);
    trace.set(
        "viewport_cannot_scroll",
        status.viewport_cannot_scroll as u8 as f64,
    );
    (status, ScrollBeginOutcome::Latched)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::main_thread_scrolling_reason::MainThreadHitTestReason;

    #[derive(Default)]
    struct Context {
        precise: bool,
        current: Option<u64>,
        animating: bool,
        same_input: bool,
        snap: bool,
        deferred: bool,
        target_valid: bool,
        target: Option<u64>,
        hit_success: bool,
        hit_node: Option<u64>,
        selected: Option<u64>,
        viewport: bool,
        can_scroll: bool,
        events: Vec<String>,
    }

    impl ScrollBeginContext for Context {
        type Node = u64;
        type ElementId = u64;
        fn state_delta_is_zero(&self) -> bool {
            true
        }
        fn precise_pixel_scroll(&self) -> bool {
            self.precise
        }
        fn animating_node_for_current_scroller(&mut self) -> Option<u64> {
            if self.animating {
                self.current
            } else {
                None
            }
        }
        fn node_element_id(&mut self, node: &u64) -> u64 {
            *node
        }
        fn abort_scroll_animation(&mut self, id: u64) {
            self.events.push(format!("abort-{id}"));
        }
        fn currently_scrolling_node(&mut self) -> Option<u64> {
            self.current
        }
        fn clear_animating_snap_targets(&mut self, id: u64) {
            self.events.push(format!("clear-snap-{id}"));
        }
        fn input_matches_latched_type(&self) -> bool {
            self.same_input
        }
        fn node_has_snap_data(&mut self, _: &u64) -> bool {
            self.snap
        }
        fn deferred_scroll_end_exists(&self, _: u64) -> bool {
            self.deferred
        }
        fn remove_deferred_scroll_end(&mut self, id: u64) {
            self.events.push(format!("remove-deferred-{id}"));
        }
        fn can_realize_scrolls_on_pending_tree(&mut self, _: &u64) -> bool {
            true
        }
        fn clear_currently_scrolling_node(&mut self) {
            self.current = None;
            self.events.push("clear-current".into());
        }
        fn target_element_is_valid(&self) -> bool {
            self.target_valid
        }
        fn main_thread_hit_test_reasons_empty(&self) -> bool {
            true
        }
        fn scrollbar_interaction(&self) -> bool {
            false
        }
        fn trace_latched_node_provided(&mut self) {
            self.events.push("latched-target".into());
        }
        fn trace_unlatched_node_provided(&mut self) {
            self.events.push("unlatched-target".into());
        }
        fn trace_hit_testing(&mut self) {
            self.events.push("hit-test".into());
        }
        fn find_target_node(&mut self) -> Option<u64> {
            self.target
        }
        fn hit_test_scroll_node(&mut self) -> ScrollBeginHitTest<u64> {
            let mut reasons = MainThreadHitTestReasons::default();
            if !self.hit_success {
                reasons.Put(MainThreadHitTestReason::kFailedHitTest);
            }
            ScrollBeginHitTest {
                scroll_node: self.hit_node,
                hit_test_successful: self.hit_success,
                main_thread_hit_test_reasons: reasons,
            }
        }
        fn trace_request_main_thread_hit_test(&mut self) {
            self.events.push("request-main".into());
        }
        fn find_node_to_latch(&mut self, _: Option<u64>) -> Option<u64> {
            self.selected
        }
        fn fluent_overlay_scrollbar_enabled(&self) -> bool {
            false
        }
        fn notify_will_scroll_content(&mut self, id: u64) {
            self.events.push(format!("will-scroll-{id}"));
        }
        fn embedded_frame(&self) -> bool {
            false
        }
        fn trace_no_scroll_node(&mut self, _: bool) {
            self.events.push("no-node".into());
        }
        fn set_currently_scrolling_node(&mut self, node: u64) {
            self.current = Some(node);
        }
        fn node_main_thread_repaint_reasons(&mut self, _: &u64) -> MainThreadRepaintReasons {
            MainThreadRepaintReasons::default()
        }
        fn did_latch_to_scroller(&mut self) {
            self.events.push("did-latch".into());
        }
        fn viewport_should_scroll(&mut self, _: &u64) -> bool {
            self.viewport
        }
        fn reset_viewport_consumed_delta(&mut self) {
            self.events.push("reset-viewport".into());
        }
        fn viewport_can_scroll(&mut self, _: &u64) -> bool {
            self.can_scroll
        }
    }

    #[test]
    fn precise_scroll_continues_deferred_animated_latch() {
        let mut context = Context {
            precise: true,
            current: Some(4),
            animating: true,
            same_input: true,
            deferred: true,
            ..Default::default()
        };
        let (status, outcome) = BeginScroll(&mut context);
        assert_eq!(outcome, ScrollBeginOutcome::ContinuedExistingLatch);
        assert!(status.raster_inducing);
        assert_eq!(
            context.events,
            ["abort-4", "clear-snap-4", "remove-deferred-4"]
        );
    }

    #[test]
    fn failed_hit_test_requests_main_thread_reasons() {
        let mut context = Context::default();
        let (status, outcome) = BeginScroll(&mut context);
        assert_eq!(outcome, ScrollBeginOutcome::RequestedMainThreadHitTest);
        assert!(!status.main_thread_hit_test_reasons.empty());
        assert_eq!(
            context.events,
            ["clear-current", "hit-test", "request-main"]
        );
    }

    #[test]
    fn selected_viewport_scroller_sets_cannot_scroll_status() {
        let mut context = Context {
            target_valid: true,
            target: Some(2),
            selected: Some(2),
            viewport: true,
            can_scroll: false,
            ..Default::default()
        };
        let (status, outcome) = BeginScroll(&mut context);
        assert_eq!(outcome, ScrollBeginOutcome::Latched);
        assert!(status.viewport_cannot_scroll);
        assert_eq!(context.current, Some(2));
        assert!(context.events.contains(&"reset-viewport".into()));
    }
}
