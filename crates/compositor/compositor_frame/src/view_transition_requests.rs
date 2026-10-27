//! View-transition request handling while compositor-frame metadata is built.

pub trait ViewTransitionRequestContext {
    fn embeds_transition_directives(&mut self) -> bool;

    fn prepare_embedded_requests(&mut self);
    fn embedded_request_count(&self) -> usize;
    fn resourceless_draw(&self) -> bool;
    fn embedded_request_delays_deletion(&self, index: usize) -> bool;
    fn embedded_request_animates_renderer(&self, index: usize) -> bool;
    fn append_embedded_directive(&mut self, index: usize);
    fn notify_embedded_request_processed(&mut self, index: usize);

    fn resident_request_count(&self) -> usize;
    fn mark_draw_properties_dirty(&mut self);
    fn resident_request_delays_deletion(&self, index: usize) -> bool;
    fn resident_request_animates_renderer(&self, index: usize) -> bool;
    fn notify_resident_request_processed(&mut self, index: usize);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewTransitionRequestResult {
    pub requests_visited: usize,
    pub directives_appended: usize,
    pub processed_notifications: usize,
    pub has_animate_renderer: bool,
    pub delay_layer_tree_view_deletion: bool,
    pub used_embedded_directives: bool,
}

pub fn ProcessViewTransitionRequests<C: ViewTransitionRequestContext>(
    context: &mut C,
) -> ViewTransitionRequestResult {
    let mut trace = browser_tracing::span(
        "compositor.frame",
        "Compositor.Frame.ProcessViewTransitionRequests",
    );
    let mut result = ViewTransitionRequestResult::default();
    result.used_embedded_directives = context.embeds_transition_directives();
    if result.used_embedded_directives {
        context.prepare_embedded_requests();
        let count = context.embedded_request_count();
        let resourceless = context.resourceless_draw();
        for index in 0..count {
            result.requests_visited += 1;
            if resourceless {
                context.notify_embedded_request_processed(index);
                result.processed_notifications += 1;
                continue;
            }
            context.append_embedded_directive(index);
            result.directives_appended += 1;
            if context.embedded_request_delays_deletion(index) {
                context.notify_embedded_request_processed(index);
                result.processed_notifications += 1;
                result.delay_layer_tree_view_deletion = true;
                result.has_animate_renderer |= context.embedded_request_animates_renderer(index);
            }
        }
    } else {
        let count = context.resident_request_count();
        if count > 0 {
            context.mark_draw_properties_dirty();
        }
        for index in 0..count {
            result.requests_visited += 1;
            if context.resident_request_delays_deletion(index) {
                let animate = context.resident_request_animates_renderer(index);
                context.notify_resident_request_processed(index);
                result.processed_notifications += 1;
                result.delay_layer_tree_view_deletion = true;
                result.has_animate_renderer |= animate;
            }
        }
    }
    trace.set("requests_visited", result.requests_visited as f64);
    trace.set("directives_appended", result.directives_appended as f64);
    trace.set(
        "processed_notifications",
        result.processed_notifications as f64,
    );
    trace.set(
        "has_animate_renderer",
        result.has_animate_renderer as u8 as f64,
    );
    trace.set(
        "delay_layer_tree_view_deletion",
        result.delay_layer_tree_view_deletion as u8 as f64,
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Context {
        embedded: bool,
        resourceless: bool,
        delays: Vec<bool>,
        animates: Vec<bool>,
        events: Vec<String>,
    }

    impl ViewTransitionRequestContext for Context {
        fn embeds_transition_directives(&mut self) -> bool {
            self.embedded
        }
        fn prepare_embedded_requests(&mut self) {
            self.events.push("prepare".into());
        }
        fn embedded_request_count(&self) -> usize {
            self.delays.len()
        }
        fn resourceless_draw(&self) -> bool {
            self.resourceless
        }
        fn embedded_request_delays_deletion(&self, index: usize) -> bool {
            self.delays[index]
        }
        fn embedded_request_animates_renderer(&self, index: usize) -> bool {
            self.animates[index]
        }
        fn append_embedded_directive(&mut self, index: usize) {
            self.events.push(format!("append-{index}"));
        }
        fn notify_embedded_request_processed(&mut self, index: usize) {
            self.events.push(format!("processed-{index}"));
        }
        fn resident_request_count(&self) -> usize {
            self.delays.len()
        }
        fn mark_draw_properties_dirty(&mut self) {
            self.events.push("dirty".into());
        }
        fn resident_request_delays_deletion(&self, index: usize) -> bool {
            self.delays[index]
        }
        fn resident_request_animates_renderer(&self, index: usize) -> bool {
            self.animates[index]
        }
        fn notify_resident_request_processed(&mut self, index: usize) {
            self.events.push(format!("resident-{index}"));
        }
    }

    #[test]
    fn embedded_path_appends_all_directives_and_notifies_delayed_requests() {
        let mut context = Context {
            embedded: true,
            delays: vec![false, true],
            animates: vec![false, true],
            ..Default::default()
        };
        let result = ProcessViewTransitionRequests(&mut context);
        assert_eq!(
            context.events,
            ["prepare", "append-0", "append-1", "processed-1"]
        );
        assert_eq!(result.directives_appended, 2);
        assert!(result.has_animate_renderer);
        assert!(result.delay_layer_tree_view_deletion);
    }

    #[test]
    fn resourceless_path_only_acknowledges_requests() {
        let mut context = Context {
            embedded: true,
            resourceless: true,
            delays: vec![true, false],
            animates: vec![true, false],
            ..Default::default()
        };
        let result = ProcessViewTransitionRequests(&mut context);
        assert_eq!(context.events, ["prepare", "processed-0", "processed-1"]);
        assert_eq!(result.directives_appended, 0);
        assert!(!result.delay_layer_tree_view_deletion);
    }

    #[test]
    fn resident_path_marks_dirty_and_only_processes_delayed_requests() {
        let mut context = Context {
            delays: vec![false, true],
            animates: vec![false, true],
            ..Default::default()
        };
        let result = ProcessViewTransitionRequests(&mut context);
        assert_eq!(context.events, ["dirty", "resident-1"]);
        assert!(result.has_animate_renderer);
        assert_eq!(result.processed_notifications, 1);
    }
}
