//! Pending-to-active tree activation ordering.
//!
//! The compositor facade adapts resident tree objects and embedding services
//! through [`ActivationContext`].  All activation branching and stage order
//! stays here so both CPU time and work-volume counters belong to Tree.

pub trait ActivationContext {
    fn trace_source_activation(&mut self);
    fn has_pending_tree(&self) -> bool;
    fn trace_pending_tree_activated(&mut self);
    fn begin_sync(&mut self);
    fn process_pending_ui_resources(&mut self);
    fn pending_needs_full_tree_sync(&self) -> bool;
    fn synchronize_full_tree(&mut self);
    fn push_scrollbar_opacities(&mut self);
    fn push_property_trees(&mut self);
    fn mark_property_trees_synced(&mut self);
    fn push_layer_properties(&mut self);
    fn mark_layer_properties_synced(&mut self);
    fn push_tree_properties(&mut self);
    fn reset_pending_change_tracking(&mut self);
    fn finish_sync(&mut self);
    fn recycle_pending_tree(&mut self);
    fn promote_scroll_timelines(&mut self);
    fn activate_animations(&mut self);
    fn activate_image_state(&mut self);
    fn process_active_ui_resources(&mut self);
    fn update_viewport_container_sizes(&mut self);
    fn clamp_viewport_scroll_offsets(&mut self);
    fn active_tree_did_become_active(&mut self);
    fn renew_tree_priority(&mut self);
    fn active_tree_has_picture_layers(&self) -> bool;
    fn mark_tile_priorities_modified(&mut self);
    fn update_screenshot_destination(&mut self);
    fn update_child_local_surface_id(&mut self);
    fn notify_can_draw_state_changed(&mut self);
    fn delegate_did_activate(&mut self);
    fn run_tree_activation_callback(&mut self);
    fn start_pending_page_scale_animation(&mut self) -> bool;
    fn notify_input_tree_activated(&mut self);
    fn should_flush_display_tree(&self) -> bool;
    fn flush_display_tree(&mut self);
    fn verbose_log_activated_tree(&mut self);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActivationStats {
    pub activated_pending_tree: bool,
    pub performed_full_tree_sync: bool,
    pub reprioritized_tiles: bool,
    pub started_page_scale_animation: bool,
    pub flushed_display_tree: bool,
}

/// Activates the current sync tree while preserving Chromium's exact stage
/// order.  Adapter methods perform only the concrete resident-object mutation
/// associated with a stage; this function owns all cross-stage decisions.
pub fn ActivateSyncTree<C: ActivationContext>(context: &mut C) -> ActivationStats {
    let mut trace = browser_tracing::span("compositor.tree", "Compositor.Tree.ActivateSyncTree");
    context.trace_source_activation();

    let activated_pending_tree = context.has_pending_tree();
    let mut performed_full_tree_sync = false;
    if activated_pending_tree {
        context.trace_pending_tree_activated();
        context.begin_sync();
        context.process_pending_ui_resources();
        performed_full_tree_sync = context.pending_needs_full_tree_sync();
        if performed_full_tree_sync {
            context.synchronize_full_tree();
        }
        context.push_scrollbar_opacities();
        context.push_property_trees();
        context.mark_property_trees_synced();
        context.push_layer_properties();
        context.mark_layer_properties_synced();
        context.push_tree_properties();
        context.reset_pending_change_tracking();
        context.finish_sync();
        context.recycle_pending_tree();
        context.promote_scroll_timelines();
        context.activate_animations();
        context.activate_image_state();
    } else {
        context.process_active_ui_resources();
    }

    context.update_viewport_container_sizes();
    context.clamp_viewport_scroll_offsets();
    context.active_tree_did_become_active();
    context.renew_tree_priority();

    let reprioritized_tiles = context.active_tree_has_picture_layers();
    if reprioritized_tiles {
        context.mark_tile_priorities_modified();
    }

    context.update_screenshot_destination();
    context.update_child_local_surface_id();
    context.notify_can_draw_state_changed();
    context.delegate_did_activate();
    context.run_tree_activation_callback();
    let started_page_scale_animation = context.start_pending_page_scale_animation();
    context.notify_input_tree_activated();

    let flushed_display_tree = context.should_flush_display_tree();
    if flushed_display_tree {
        context.flush_display_tree();
    }
    context.verbose_log_activated_tree();

    trace.set(
        "activated_pending_tree",
        activated_pending_tree as u8 as f64,
    );
    trace.set(
        "performed_full_tree_sync",
        performed_full_tree_sync as u8 as f64,
    );
    trace.set("reprioritized_tiles", reprioritized_tiles as u8 as f64);
    trace.set(
        "started_page_scale_animation",
        started_page_scale_animation as u8 as f64,
    );
    trace.set("flushed_display_tree", flushed_display_tree as u8 as f64);

    ActivationStats {
        activated_pending_tree,
        performed_full_tree_sync,
        reprioritized_tiles,
        started_page_scale_animation,
        flushed_display_tree,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Context {
        pending: bool,
        full_sync: bool,
        pictures: bool,
        page_scale: bool,
        flush: bool,
        events: Vec<&'static str>,
    }

    macro_rules! event {
        ($name:ident, $value:literal) => {
            fn $name(&mut self) {
                self.events.push($value);
            }
        };
    }

    impl ActivationContext for Context {
        event!(trace_source_activation, "trace");
        fn has_pending_tree(&self) -> bool {
            self.pending
        }
        event!(trace_pending_tree_activated, "trace-pending");
        event!(begin_sync, "begin-sync");
        event!(process_pending_ui_resources, "pending-ui");
        fn pending_needs_full_tree_sync(&self) -> bool {
            self.full_sync
        }
        event!(synchronize_full_tree, "full-sync");
        event!(push_scrollbar_opacities, "scrollbar-opacity");
        event!(push_property_trees, "property-trees");
        event!(mark_property_trees_synced, "property-trees-synced");
        event!(push_layer_properties, "layer-properties");
        event!(mark_layer_properties_synced, "layer-properties-synced");
        event!(push_tree_properties, "tree-properties");
        event!(reset_pending_change_tracking, "reset-change-tracking");
        event!(finish_sync, "finish-sync");
        event!(recycle_pending_tree, "recycle");
        event!(promote_scroll_timelines, "timelines");
        event!(activate_animations, "animations");
        event!(activate_image_state, "images");
        event!(process_active_ui_resources, "active-ui");
        event!(update_viewport_container_sizes, "viewport-size");
        event!(clamp_viewport_scroll_offsets, "clamp-scroll");
        event!(active_tree_did_become_active, "became-active");
        event!(renew_tree_priority, "tree-priority");
        fn active_tree_has_picture_layers(&self) -> bool {
            self.pictures
        }
        event!(mark_tile_priorities_modified, "tile-priority");
        event!(update_screenshot_destination, "screenshot");
        event!(update_child_local_surface_id, "surface-id");
        event!(notify_can_draw_state_changed, "notify-can-draw");
        event!(delegate_did_activate, "delegate-activated");
        event!(run_tree_activation_callback, "activation-callback");
        fn start_pending_page_scale_animation(&mut self) -> bool {
            self.events.push("page-scale");
            self.page_scale
        }
        event!(notify_input_tree_activated, "input");
        fn should_flush_display_tree(&self) -> bool {
            self.flush
        }
        event!(flush_display_tree, "flush");
        event!(verbose_log_activated_tree, "log");
    }

    #[test]
    fn pending_tree_uses_complete_native_activation_order() {
        let mut context = Context {
            pending: true,
            full_sync: true,
            pictures: true,
            page_scale: true,
            flush: true,
            ..Context::default()
        };
        let stats = ActivateSyncTree(&mut context);
        assert_eq!(
            context.events,
            [
                "trace",
                "trace-pending",
                "begin-sync",
                "pending-ui",
                "full-sync",
                "scrollbar-opacity",
                "property-trees",
                "property-trees-synced",
                "layer-properties",
                "layer-properties-synced",
                "tree-properties",
                "reset-change-tracking",
                "finish-sync",
                "recycle",
                "timelines",
                "animations",
                "images",
                "viewport-size",
                "clamp-scroll",
                "became-active",
                "tree-priority",
                "tile-priority",
                "screenshot",
                "surface-id",
                "notify-can-draw",
                "delegate-activated",
                "activation-callback",
                "page-scale",
                "input",
                "flush",
                "log",
            ]
        );
        assert_eq!(
            stats,
            ActivationStats {
                activated_pending_tree: true,
                performed_full_tree_sync: true,
                reprioritized_tiles: true,
                started_page_scale_animation: true,
                flushed_display_tree: true,
            }
        );
    }

    #[test]
    fn active_only_path_skips_pending_sync_and_optional_work() {
        let mut context = Context::default();
        let stats = ActivateSyncTree(&mut context);
        assert_eq!(
            context.events,
            [
                "trace",
                "active-ui",
                "viewport-size",
                "clamp-scroll",
                "became-active",
                "tree-priority",
                "screenshot",
                "surface-id",
                "notify-can-draw",
                "delegate-activated",
                "activation-callback",
                "page-scale",
                "input",
                "log",
            ]
        );
        assert_eq!(stats, ActivationStats::default());
    }
}
