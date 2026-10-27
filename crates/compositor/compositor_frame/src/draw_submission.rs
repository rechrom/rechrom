//! Drawn-frame submission lifecycle.
//!
//! The facade supplies resident frame/tree/service operations, while Frame
//! owns early-out behavior, submit-route selection, post-submit cleanup order,
//! and the metrics that identify where submission time was spent.

pub trait DrawSubmissionContext {
    type SubmitInfo;

    fn reset_requires_high_resolution_draw(&mut self);
    fn frame_has_no_damage(&self) -> bool;
    fn trace_no_damage_early_out(&mut self);
    fn break_swap_promises_for_no_damage(&mut self);
    fn uses_external_display_tree(&mut self) -> bool;
    fn update_external_display_tree_without_damage(&mut self);
    fn reset_change_tracking(&mut self);
    fn is_ui_tree(&self) -> bool;
    fn discard_ui_event_metrics(&mut self);

    fn set_frame_sink_source_frame_number(&mut self);
    fn update_drawn_with_new_layer_tree(&mut self) -> bool;
    fn generate_compositor_frame(&mut self);
    fn maybe_dump_compositor_frame(&mut self);
    fn update_top_controls_visible_height(&mut self) -> bool;
    fn capture_debug_begin_frame_ack(&mut self);
    fn collect_event_metrics(&mut self);
    fn collect_normalized_invalidated_area(&mut self);
    fn verbose_log_compositor_frame(&mut self);
    fn submit_external_display_tree(&mut self);
    fn submit_to_frame_sink(&mut self);
    fn validate_debug_begin_frame_ack(&mut self);

    fn stop_completed_draw_sequences(&mut self);
    fn clear_swap_promises(&mut self);
    fn frame_has_copy_requests(&self) -> bool;
    fn clear_copy_requests(&mut self);
    fn mark_draw_properties_dirty(&mut self);
    fn request_commit(&mut self);
    fn mark_surface_damage_drawn(&mut self);
    fn root_surface_exists(&self) -> bool;
    fn reset_viewport_and_root_damage(&mut self);
    fn notify_devtools_frame_drawn(&mut self);
    fn issue_rendering_stats(&mut self);
    fn throttling_enabled(&self) -> bool;
    fn throttling_changed(&self) -> bool;
    fn notify_throttling_changed(&mut self);
    fn update_checkerboard_priority(&mut self);
    fn make_submit_info(
        &mut self,
        drawn_with_new_layer_tree: bool,
        top_controls_moved: bool,
    ) -> Self::SubmitInfo;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DrawSubmissionStats {
    pub no_damage_early_out: bool,
    pub used_external_display_tree: bool,
    pub drawn_with_new_layer_tree: bool,
    pub top_controls_moved: bool,
    pub cleared_copy_requests: bool,
    pub notified_throttling_change: bool,
}

pub fn SubmitDrawnFrame<C: DrawSubmissionContext>(
    context: &mut C,
) -> (Option<C::SubmitInfo>, DrawSubmissionStats) {
    let mut trace = browser_tracing::span("compositor.frame", "Compositor.Frame.DrawLayers");
    context.reset_requires_high_resolution_draw();
    if context.frame_has_no_damage() {
        context.trace_no_damage_early_out();
        context.break_swap_promises_for_no_damage();
        let used_external_display_tree = context.uses_external_display_tree();
        if used_external_display_tree {
            context.update_external_display_tree_without_damage();
        }
        context.reset_change_tracking();
        if context.is_ui_tree() {
            context.discard_ui_event_metrics();
        }
        trace.set("no_damage_early_out", 1.0);
        trace.set(
            "external_display_tree",
            used_external_display_tree as u8 as f64,
        );
        return (
            None,
            DrawSubmissionStats {
                no_damage_early_out: true,
                used_external_display_tree,
                ..Default::default()
            },
        );
    }

    context.set_frame_sink_source_frame_number();
    let drawn_with_new_layer_tree = context.update_drawn_with_new_layer_tree();
    context.generate_compositor_frame();
    context.maybe_dump_compositor_frame();
    let top_controls_moved = context.update_top_controls_visible_height();
    context.capture_debug_begin_frame_ack();
    context.collect_event_metrics();
    context.collect_normalized_invalidated_area();
    context.verbose_log_compositor_frame();
    let used_external_display_tree = context.uses_external_display_tree();
    if used_external_display_tree {
        context.submit_external_display_tree();
    } else {
        context.submit_to_frame_sink();
    }
    context.validate_debug_begin_frame_ack();

    context.stop_completed_draw_sequences();
    context.clear_swap_promises();
    let cleared_copy_requests = context.frame_has_copy_requests();
    if cleared_copy_requests {
        context.clear_copy_requests();
        context.mark_draw_properties_dirty();
        context.request_commit();
    }
    context.mark_surface_damage_drawn();
    if context.root_surface_exists() {
        context.reset_viewport_and_root_damage();
    }
    context.reset_change_tracking();
    if !context.is_ui_tree() {
        context.notify_devtools_frame_drawn();
    }
    if !used_external_display_tree {
        context.issue_rendering_stats();
    }
    let notified_throttling_change = context.throttling_enabled() && context.throttling_changed();
    if notified_throttling_change {
        context.notify_throttling_changed();
    }
    context.update_checkerboard_priority();
    let submit_info = context.make_submit_info(drawn_with_new_layer_tree, top_controls_moved);

    trace.set("no_damage_early_out", 0.0);
    trace.set(
        "external_display_tree",
        used_external_display_tree as u8 as f64,
    );
    trace.set(
        "drawn_with_new_layer_tree",
        drawn_with_new_layer_tree as u8 as f64,
    );
    trace.set("top_controls_moved", top_controls_moved as u8 as f64);
    trace.set("cleared_copy_requests", cleared_copy_requests as u8 as f64);
    trace.set(
        "notified_throttling_change",
        notified_throttling_change as u8 as f64,
    );
    (
        Some(submit_info),
        DrawSubmissionStats {
            no_damage_early_out: false,
            used_external_display_tree,
            drawn_with_new_layer_tree,
            top_controls_moved,
            cleared_copy_requests,
            notified_throttling_change,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Context {
        no_damage: bool,
        external: bool,
        ui: bool,
        copy: bool,
        root: bool,
        throttle: bool,
        throttle_changed: bool,
        events: Vec<&'static str>,
    }

    macro_rules! event {
        ($name:ident, $value:literal) => {
            fn $name(&mut self) {
                self.events.push($value);
            }
        };
    }

    impl DrawSubmissionContext for Context {
        type SubmitInfo = &'static str;
        event!(reset_requires_high_resolution_draw, "reset-high-res");
        fn frame_has_no_damage(&self) -> bool {
            self.no_damage
        }
        event!(trace_no_damage_early_out, "trace-no-damage");
        event!(break_swap_promises_for_no_damage, "break-promises");
        fn uses_external_display_tree(&mut self) -> bool {
            self.external
        }
        event!(update_external_display_tree_without_damage, "display-empty");
        event!(reset_change_tracking, "reset-change");
        fn is_ui_tree(&self) -> bool {
            self.ui
        }
        event!(discard_ui_event_metrics, "discard-metrics");
        event!(set_frame_sink_source_frame_number, "source-frame");
        fn update_drawn_with_new_layer_tree(&mut self) -> bool {
            self.events.push("new-tree");
            true
        }
        event!(generate_compositor_frame, "generate");
        event!(maybe_dump_compositor_frame, "dump");
        fn update_top_controls_visible_height(&mut self) -> bool {
            self.events.push("controls");
            true
        }
        event!(capture_debug_begin_frame_ack, "capture-ack");
        event!(collect_event_metrics, "metrics");
        event!(collect_normalized_invalidated_area, "invalidated");
        event!(verbose_log_compositor_frame, "log");
        event!(submit_external_display_tree, "submit-display");
        event!(submit_to_frame_sink, "submit-sink");
        event!(validate_debug_begin_frame_ack, "validate-ack");
        event!(stop_completed_draw_sequences, "stop-sequences");
        event!(clear_swap_promises, "clear-promises");
        fn frame_has_copy_requests(&self) -> bool {
            self.copy
        }
        event!(clear_copy_requests, "clear-copy");
        event!(mark_draw_properties_dirty, "dirty-draw");
        event!(request_commit, "commit");
        event!(mark_surface_damage_drawn, "damage-drawn");
        fn root_surface_exists(&self) -> bool {
            self.root
        }
        event!(reset_viewport_and_root_damage, "reset-damage");
        event!(notify_devtools_frame_drawn, "devtools");
        event!(issue_rendering_stats, "stats");
        fn throttling_enabled(&self) -> bool {
            self.throttle
        }
        fn throttling_changed(&self) -> bool {
            self.throttle_changed
        }
        event!(notify_throttling_changed, "throttle");
        event!(update_checkerboard_priority, "checkerboard");
        fn make_submit_info(&mut self, _: bool, _: bool) -> &'static str {
            self.events.push("submit-info");
            "submitted"
        }
    }

    #[test]
    fn no_damage_path_owns_external_and_ui_cleanup_order() {
        let mut context = Context {
            no_damage: true,
            external: true,
            ui: true,
            ..Default::default()
        };
        let (submission, stats) = SubmitDrawnFrame(&mut context);
        assert_eq!(submission, None);
        assert!(stats.no_damage_early_out);
        assert_eq!(
            context.events,
            [
                "reset-high-res",
                "trace-no-damage",
                "break-promises",
                "display-empty",
                "reset-change",
                "discard-metrics"
            ]
        );
    }

    #[test]
    fn damaged_path_owns_submit_and_post_submit_cleanup_order() {
        let mut context = Context {
            external: false,
            copy: true,
            root: true,
            throttle: true,
            throttle_changed: true,
            ..Default::default()
        };
        let (submission, stats) = SubmitDrawnFrame(&mut context);
        assert_eq!(submission, Some("submitted"));
        assert!(stats.cleared_copy_requests);
        assert!(stats.notified_throttling_change);
        assert!(context
            .events
            .windows(3)
            .any(|w| w == ["clear-copy", "dirty-draw", "commit"]));
        assert!(context
            .events
            .windows(2)
            .any(|w| w == ["damage-drawn", "reset-damage"]));
        assert!(context.events.contains(&"submit-sink"));
        assert!(!context.events.contains(&"submit-display"));
    }
}
