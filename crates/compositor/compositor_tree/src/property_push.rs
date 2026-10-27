//! Pending-to-active LayerTree property transfer ordering.

pub trait PropertyPushContext {
    type ViewTransitionRequest;

    fn assert_source_ui_resource_queue_empty(&self);
    fn push_surface_ranges(&mut self);
    fn push_pending_scroll_updates(&mut self);
    fn take_force_redraw(&mut self) -> bool;
    fn force_target_redraw(&mut self);
    fn transfer_swap_promises(&mut self);
    fn transfer_viewport_property_ids(&mut self);
    fn transfer_page_scale_and_limits(&mut self);
    fn transfer_external_page_scale(&mut self);
    fn transfer_browser_controls_params(&mut self);
    fn transfer_browser_controls_values(&mut self);
    fn transfer_overscroll_behavior(&mut self);
    fn transfer_display_color_spaces(&mut self);
    fn transfer_painted_device_scale(&mut self);
    fn transfer_device_scale(&mut self);
    fn transfer_device_viewport(&mut self);
    fn transfer_max_safe_area_inset(&mut self);
    fn take_new_local_surface_id_request(&mut self) -> bool;
    fn request_target_local_surface_id(&mut self);
    fn transfer_parent_local_surface_id(&mut self);
    fn transfer_screenshot_destination(&mut self) -> bool;
    fn transfer_primary_item_sequence(&mut self);
    fn transfer_pending_page_scale_animation(&mut self) -> bool;
    fn take_force_send_metadata_request(&mut self) -> bool;
    fn request_target_force_send_metadata(&mut self);
    fn transfer_selection(&mut self);
    fn transfer_frame_identity(&mut self);
    fn transfer_background_and_event_state(&mut self);
    fn transfer_event_listener_properties(&mut self);
    fn transfer_hud_layer(&mut self);
    fn update_tickmark_visibility(&mut self);
    fn update_scrollbar_geometries(&mut self);
    fn handle_scrollbar_show_requests(&mut self);
    fn transfer_presentation_callbacks(&mut self);
    fn transfer_successful_presentation_callbacks(&mut self);
    fn transfer_delegated_ink_metadata(&mut self) -> bool;
    fn take_view_transition_requests(&mut self) -> Vec<Self::ViewTransitionRequest>;
    fn push_view_transition_request(&mut self, request: Self::ViewTransitionRequest);
    fn trees_in_viz_in_client_process(&self) -> bool;
    fn move_change_tracking_to_layers(&mut self);
    fn update_property_tree_change_tracking(&mut self);
    fn transfer_created_begin_frame_args(&mut self);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PropertyPushStats {
    pub forced_redraw: bool,
    pub requested_local_surface_id: bool,
    pub transferred_screenshot_destination: bool,
    pub transferred_page_scale_animation: bool,
    pub forced_metadata: bool,
    pub transferred_delegated_ink_metadata: bool,
    pub view_transition_requests: usize,
    pub updated_property_tree_change_tracking: bool,
}

/// Transfers pending-tree state to the active tree in source order. Concrete
/// resident values remain private to the facade, while all ordering and
/// optional-stage decisions are owned and measured by Tree.
pub fn PushPropertiesToActive<C: PropertyPushContext>(context: &mut C) -> PropertyPushStats {
    let mut trace =
        browser_tracing::span("compositor.tree", "Compositor.Tree.PushPropertiesToActive");
    context.assert_source_ui_resource_queue_empty();
    context.push_surface_ranges();
    context.push_pending_scroll_updates();

    let forced_redraw = context.take_force_redraw();
    if forced_redraw {
        context.force_target_redraw();
    }

    context.transfer_swap_promises();
    context.transfer_viewport_property_ids();
    context.transfer_page_scale_and_limits();
    context.transfer_external_page_scale();
    context.transfer_browser_controls_params();
    context.transfer_browser_controls_values();
    context.transfer_overscroll_behavior();
    context.transfer_display_color_spaces();
    context.transfer_painted_device_scale();
    context.transfer_device_scale();
    context.transfer_device_viewport();
    context.transfer_max_safe_area_inset();

    let requested_local_surface_id = context.take_new_local_surface_id_request();
    if requested_local_surface_id {
        context.request_target_local_surface_id();
    }
    context.transfer_parent_local_surface_id();
    let transferred_screenshot_destination = context.transfer_screenshot_destination();
    context.transfer_primary_item_sequence();
    let transferred_page_scale_animation = context.transfer_pending_page_scale_animation();

    let forced_metadata = context.take_force_send_metadata_request();
    if forced_metadata {
        context.request_target_force_send_metadata();
    }

    context.transfer_selection();
    context.transfer_frame_identity();
    context.transfer_background_and_event_state();
    context.transfer_event_listener_properties();
    context.transfer_hud_layer();
    context.update_tickmark_visibility();
    context.update_scrollbar_geometries();
    context.handle_scrollbar_show_requests();
    context.transfer_presentation_callbacks();
    context.transfer_successful_presentation_callbacks();
    let transferred_delegated_ink_metadata = context.transfer_delegated_ink_metadata();

    let requests = context.take_view_transition_requests();
    let view_transition_requests = requests.len();
    for request in requests {
        context.push_view_transition_request(request);
    }

    let updated_property_tree_change_tracking = context.trees_in_viz_in_client_process();
    if updated_property_tree_change_tracking {
        context.update_property_tree_change_tracking();
    } else {
        context.move_change_tracking_to_layers();
    }
    context.transfer_created_begin_frame_args();

    trace.set("forced_redraw", forced_redraw as u8 as f64);
    trace.set(
        "requested_local_surface_id",
        requested_local_surface_id as u8 as f64,
    );
    trace.set(
        "transferred_screenshot_destination",
        transferred_screenshot_destination as u8 as f64,
    );
    trace.set(
        "transferred_page_scale_animation",
        transferred_page_scale_animation as u8 as f64,
    );
    trace.set("forced_metadata", forced_metadata as u8 as f64);
    trace.set(
        "transferred_delegated_ink_metadata",
        transferred_delegated_ink_metadata as u8 as f64,
    );
    trace.set("view_transition_requests", view_transition_requests as f64);
    trace.set(
        "updated_property_tree_change_tracking",
        updated_property_tree_change_tracking as u8 as f64,
    );

    PropertyPushStats {
        forced_redraw,
        requested_local_surface_id,
        transferred_screenshot_destination,
        transferred_page_scale_animation,
        forced_metadata,
        transferred_delegated_ink_metadata,
        view_transition_requests,
        updated_property_tree_change_tracking,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Context {
        optional: bool,
        in_viz: bool,
        requests: Vec<u8>,
        events: Vec<&'static str>,
    }

    macro_rules! event {
        ($name:ident, $value:literal) => {
            fn $name(&mut self) {
                self.events.push($value);
            }
        };
    }

    impl PropertyPushContext for Context {
        type ViewTransitionRequest = u8;
        fn assert_source_ui_resource_queue_empty(&self) {}
        event!(push_surface_ranges, "surface-ranges");
        event!(push_pending_scroll_updates, "scroll");
        fn take_force_redraw(&mut self) -> bool {
            self.optional
        }
        event!(force_target_redraw, "redraw");
        event!(transfer_swap_promises, "swap");
        event!(transfer_viewport_property_ids, "viewport-ids");
        event!(transfer_page_scale_and_limits, "page-scale");
        event!(transfer_external_page_scale, "external-scale");
        event!(transfer_browser_controls_params, "controls-params");
        event!(transfer_browser_controls_values, "controls-values");
        event!(transfer_overscroll_behavior, "overscroll");
        event!(transfer_display_color_spaces, "color-spaces");
        event!(transfer_painted_device_scale, "painted-scale");
        event!(transfer_device_scale, "device-scale");
        event!(transfer_device_viewport, "viewport");
        event!(transfer_max_safe_area_inset, "safe-area");
        fn take_new_local_surface_id_request(&mut self) -> bool {
            self.optional
        }
        event!(request_target_local_surface_id, "request-surface");
        event!(transfer_parent_local_surface_id, "parent-surface");
        fn transfer_screenshot_destination(&mut self) -> bool {
            self.events.push("screenshot");
            self.optional
        }
        event!(transfer_primary_item_sequence, "item-sequence");
        fn transfer_pending_page_scale_animation(&mut self) -> bool {
            self.events.push("pending-animation");
            self.optional
        }
        fn take_force_send_metadata_request(&mut self) -> bool {
            self.optional
        }
        event!(request_target_force_send_metadata, "metadata");
        event!(transfer_selection, "selection");
        event!(transfer_frame_identity, "frame-identity");
        event!(transfer_background_and_event_state, "background-events");
        event!(transfer_event_listener_properties, "listeners");
        event!(transfer_hud_layer, "hud");
        event!(update_tickmark_visibility, "tickmarks");
        event!(update_scrollbar_geometries, "scrollbar-geometry");
        event!(handle_scrollbar_show_requests, "scrollbar-show");
        event!(transfer_presentation_callbacks, "presentation");
        event!(
            transfer_successful_presentation_callbacks,
            "successful-presentation"
        );
        fn transfer_delegated_ink_metadata(&mut self) -> bool {
            self.events.push("ink");
            self.optional
        }
        fn take_view_transition_requests(&mut self) -> Vec<u8> {
            self.events.push("take-view-transitions");
            std::mem::take(&mut self.requests)
        }
        fn push_view_transition_request(&mut self, _: u8) {
            self.events.push("view-transition");
        }
        fn trees_in_viz_in_client_process(&self) -> bool {
            self.in_viz
        }
        event!(move_change_tracking_to_layers, "move-change-tracking");
        event!(
            update_property_tree_change_tracking,
            "update-change-tracking"
        );
        event!(transfer_created_begin_frame_args, "begin-frame");
    }

    #[test]
    fn owns_full_order_and_optional_branches() {
        let mut context = Context {
            optional: true,
            in_viz: true,
            requests: vec![1, 2],
            ..Context::default()
        };
        let stats = PushPropertiesToActive(&mut context);
        assert_eq!(context.events.first(), Some(&"surface-ranges"));
        assert_eq!(context.events.last(), Some(&"begin-frame"));
        assert_eq!(
            context
                .events
                .iter()
                .filter(|event| **event == "view-transition")
                .count(),
            2
        );
        assert_eq!(stats.view_transition_requests, 2);
        assert!(stats.forced_redraw);
        assert!(stats.requested_local_surface_id);
        assert!(stats.updated_property_tree_change_tracking);
        let change_index = context
            .events
            .iter()
            .position(|event| *event == "update-change-tracking")
            .unwrap();
        let frame_index = context
            .events
            .iter()
            .position(|event| *event == "begin-frame")
            .unwrap();
        assert!(change_index < frame_index);
    }

    #[test]
    fn inactive_options_skip_mutations_and_use_layer_change_tracking() {
        let mut context = Context::default();
        let stats = PushPropertiesToActive(&mut context);
        assert!(!context.events.contains(&"redraw"));
        assert!(!context.events.contains(&"request-surface"));
        assert!(!context.events.contains(&"metadata"));
        assert!(context.events.contains(&"move-change-tracking"));
        assert_eq!(stats, PropertyPushStats::default());
    }
}
