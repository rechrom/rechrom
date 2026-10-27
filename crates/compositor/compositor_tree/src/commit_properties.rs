//! Main-thread commit property transfer into a retained impl tree.

pub trait CommitPropertyContext {
    fn trees_in_viz_client_process(&self) -> bool;
    fn current_needs_full_tree_sync(&self) -> bool;
    fn commit_needs_full_tree_sync(&self) -> bool;
    fn set_needs_full_tree_sync(&mut self, value: bool);
    fn update_hud_layer(&mut self);
    fn pull_background_color(&mut self);
    fn pull_scroll_event_handler_state(&mut self);
    fn event_listener_class_count(&self) -> usize;
    fn pull_event_listener_properties(&mut self, index: usize);
    fn pull_property_change_commit_criteria(&mut self);
    fn pull_viewport_property_ids(&mut self);
    fn pull_selection(&mut self);
    fn pull_page_scale(&mut self);
    fn pull_browser_controls_params(&mut self);
    fn pull_overscroll_behavior(&mut self);
    fn pull_browser_controls_ratios(&mut self);
    fn pull_display_color_spaces(&mut self);
    fn pull_external_page_scale(&mut self);
    fn pull_painted_device_scale(&mut self);
    fn pull_device_scale(&mut self);
    fn pull_device_viewport(&mut self);
    fn pull_max_safe_area_inset(&mut self);
    fn commit_requests_new_local_surface_id(&self) -> bool;
    fn request_new_local_surface_id(&mut self);
    fn commit_has_screenshot_destination(&self) -> bool;
    fn pull_screenshot_destination(&mut self);
    fn pull_primary_main_frame_sequence(&mut self);
    fn pull_parent_local_surface_id(&mut self);
    fn pull_pending_page_scale_animation(&mut self) -> bool;
    fn commit_requests_force_send_metadata(&self) -> bool;
    fn request_force_send_metadata(&mut self);
    fn pull_display_transform_hint(&mut self);
    fn pull_delegated_ink_metadata(&mut self);
    fn view_transition_request_count(&mut self) -> usize;
    fn pull_view_transition_request(&mut self, index: usize);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CommitPropertyStats {
    pub preserved_full_tree_sync: bool,
    pub event_listener_classes: usize,
    pub requested_local_surface_id: bool,
    pub pulled_screenshot_destination: bool,
    pub pulled_page_scale_animation: bool,
    pub requested_force_send_metadata: bool,
    pub view_transition_requests: usize,
}

pub fn PullCommitProperties<C: CommitPropertyContext>(context: &mut C) -> CommitPropertyStats {
    let mut trace =
        browser_tracing::span("compositor.tree", "Compositor.Tree.PullCommitProperties");
    let preserve_full_tree_sync =
        context.trees_in_viz_client_process() && context.current_needs_full_tree_sync();
    context
        .set_needs_full_tree_sync(preserve_full_tree_sync || context.commit_needs_full_tree_sync());
    context.update_hud_layer();
    context.pull_background_color();
    context.pull_scroll_event_handler_state();
    let event_listener_classes = context.event_listener_class_count();
    for index in 0..event_listener_classes {
        context.pull_event_listener_properties(index);
    }
    context.pull_property_change_commit_criteria();
    context.pull_viewport_property_ids();
    context.pull_selection();
    context.pull_page_scale();
    context.pull_browser_controls_params();
    context.pull_overscroll_behavior();
    context.pull_browser_controls_ratios();
    context.pull_display_color_spaces();
    context.pull_external_page_scale();
    context.pull_painted_device_scale();
    context.pull_device_scale();
    context.pull_device_viewport();
    context.pull_max_safe_area_inset();

    let requested_local_surface_id = context.commit_requests_new_local_surface_id();
    if requested_local_surface_id {
        context.request_new_local_surface_id();
    }
    let pulled_screenshot_destination = context.commit_has_screenshot_destination();
    if pulled_screenshot_destination {
        context.pull_screenshot_destination();
    }
    context.pull_primary_main_frame_sequence();
    context.pull_parent_local_surface_id();
    let pulled_page_scale_animation = context.pull_pending_page_scale_animation();
    let requested_force_send_metadata = context.commit_requests_force_send_metadata();
    if requested_force_send_metadata {
        context.request_force_send_metadata();
    }
    context.pull_display_transform_hint();
    context.pull_delegated_ink_metadata();
    let view_transition_requests = context.view_transition_request_count();
    for index in 0..view_transition_requests {
        context.pull_view_transition_request(index);
    }

    trace.set(
        "preserved_full_tree_sync",
        preserve_full_tree_sync as u8 as f64,
    );
    trace.set("event_listener_classes", event_listener_classes as f64);
    trace.set(
        "requested_local_surface_id",
        requested_local_surface_id as u8 as f64,
    );
    trace.set(
        "pulled_screenshot_destination",
        pulled_screenshot_destination as u8 as f64,
    );
    trace.set(
        "pulled_page_scale_animation",
        pulled_page_scale_animation as u8 as f64,
    );
    trace.set(
        "requested_force_send_metadata",
        requested_force_send_metadata as u8 as f64,
    );
    trace.set("view_transition_requests", view_transition_requests as f64);
    CommitPropertyStats {
        preserved_full_tree_sync: preserve_full_tree_sync,
        event_listener_classes,
        requested_local_surface_id,
        pulled_screenshot_destination,
        pulled_page_scale_animation,
        requested_force_send_metadata,
        view_transition_requests,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Context {
        viz: bool,
        current_sync: bool,
        commit_sync: bool,
        local_surface: bool,
        screenshot: bool,
        animation: bool,
        force_metadata: bool,
        requests: usize,
        events: Vec<String>,
    }

    macro_rules! event {
        ($name:ident, $value:literal) => {
            fn $name(&mut self) {
                self.events.push($value.into());
            }
        };
    }

    impl CommitPropertyContext for Context {
        fn trees_in_viz_client_process(&self) -> bool {
            self.viz
        }
        fn current_needs_full_tree_sync(&self) -> bool {
            self.current_sync
        }
        fn commit_needs_full_tree_sync(&self) -> bool {
            self.commit_sync
        }
        fn set_needs_full_tree_sync(&mut self, value: bool) {
            self.events.push(format!("sync-{value}"));
        }
        event!(update_hud_layer, "hud");
        event!(pull_background_color, "background");
        event!(pull_scroll_event_handler_state, "scroll-handlers");
        fn event_listener_class_count(&self) -> usize {
            3
        }
        fn pull_event_listener_properties(&mut self, index: usize) {
            self.events.push(format!("listener-{index}"));
        }
        event!(pull_property_change_commit_criteria, "criteria");
        event!(pull_viewport_property_ids, "viewport-ids");
        event!(pull_selection, "selection");
        event!(pull_page_scale, "page-scale");
        event!(pull_browser_controls_params, "controls-params");
        event!(pull_overscroll_behavior, "overscroll");
        event!(pull_browser_controls_ratios, "controls-ratios");
        event!(pull_display_color_spaces, "colors");
        event!(pull_external_page_scale, "external-scale");
        event!(pull_painted_device_scale, "painted-scale");
        event!(pull_device_scale, "device-scale");
        event!(pull_device_viewport, "viewport");
        event!(pull_max_safe_area_inset, "safe-area");
        fn commit_requests_new_local_surface_id(&self) -> bool {
            self.local_surface
        }
        event!(request_new_local_surface_id, "local-surface");
        fn commit_has_screenshot_destination(&self) -> bool {
            self.screenshot
        }
        event!(pull_screenshot_destination, "screenshot");
        event!(pull_primary_main_frame_sequence, "sequence");
        event!(pull_parent_local_surface_id, "parent-surface");
        fn pull_pending_page_scale_animation(&mut self) -> bool {
            self.events.push("animation".into());
            self.animation
        }
        fn commit_requests_force_send_metadata(&self) -> bool {
            self.force_metadata
        }
        event!(request_force_send_metadata, "force-metadata");
        event!(pull_display_transform_hint, "transform");
        event!(pull_delegated_ink_metadata, "ink");
        fn view_transition_request_count(&mut self) -> usize {
            self.requests
        }
        fn pull_view_transition_request(&mut self, index: usize) {
            self.events.push(format!("request-{index}"));
        }
    }

    #[test]
    fn owns_full_property_order_loops_and_optional_branches() {
        let mut context = Context {
            viz: true,
            current_sync: true,
            local_surface: true,
            screenshot: true,
            animation: true,
            force_metadata: true,
            requests: 2,
            ..Default::default()
        };
        let stats = PullCommitProperties(&mut context);
        assert!(stats.preserved_full_tree_sync);
        assert_eq!(stats.event_listener_classes, 3);
        assert_eq!(stats.view_transition_requests, 2);
        assert!(context
            .events
            .windows(3)
            .any(|w| w == ["listener-0", "listener-1", "listener-2"]));
        assert_eq!(
            &context.events[context.events.len() - 4..],
            ["transform", "ink", "request-0", "request-1"]
        );
    }

    #[test]
    fn non_viz_tree_uses_commit_full_sync_value_and_skips_optionals() {
        let mut context = Context::default();
        let stats = PullCommitProperties(&mut context);
        assert!(!stats.preserved_full_tree_sync);
        assert!(!stats.requested_local_surface_id);
        assert!(!stats.pulled_screenshot_destination);
        assert!(!context.events.contains(&"local-surface".into()));
        assert!(!context.events.contains(&"force-metadata".into()));
    }
}
