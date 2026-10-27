//! Main-thread commit to compositor sync-tree ordering.

pub trait CommitSyncContext {
    fn begin_sync(&mut self);
    fn commit_forces_redraw(&self) -> bool;
    fn force_redraw_next_activation(&mut self);
    fn commit_forces_raster_scale_recalculation(&self) -> bool;
    fn force_raster_scale_recalculation(&mut self);
    fn transfer_presentation_callbacks(&mut self);
    fn transfer_successful_presentation_callbacks(&mut self);
    fn needs_full_tree_sync(&self) -> bool;
    fn synchronize_full_tree(&mut self);
    fn should_clear_caches(&self) -> bool;
    fn clear_history_and_caches(&mut self);
    fn pull_property_trees(&mut self);
    fn mark_property_trees_synced(&mut self);
    fn needs_surface_ranges_sync(&self) -> bool;
    fn synchronize_surface_ranges(&mut self);
    fn push_layer_properties_and_scroll_updates(&mut self);
    fn pull_layer_tree_properties(&mut self);
    fn transfer_swap_promises(&mut self);
    fn transfer_event_metrics(&mut self);
    fn transfer_ui_resource_requests(&mut self);
    fn update_property_tree_animation(&mut self);
    fn push_mutator_properties(&mut self);
    fn move_change_tracking_to_layers(&mut self);
    fn update_scrollbar_geometries(&mut self);
    fn finish_sync(&mut self);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CommitSyncStats {
    pub forced_redraw: bool,
    pub forced_raster_scale_recalculation: bool,
    pub full_tree_sync: bool,
    pub cleared_caches: bool,
    pub synchronized_surface_ranges: bool,
}

/// Applies one main-thread commit to the sync tree in Chromium source order.
/// The facade supplies resident object mutations, but owns no conditional or
/// cross-stage scheduling in this path.
pub fn PullCommitIntoSyncTree<C: CommitSyncContext>(context: &mut C) -> CommitSyncStats {
    let mut trace =
        browser_tracing::span("compositor.tree", "Compositor.Tree.PullCommitIntoSyncTree");
    context.begin_sync();

    let forced_redraw = context.commit_forces_redraw();
    if forced_redraw {
        context.force_redraw_next_activation();
    }
    let forced_raster_scale_recalculation = context.commit_forces_raster_scale_recalculation();
    if forced_raster_scale_recalculation {
        context.force_raster_scale_recalculation();
    }

    context.transfer_presentation_callbacks();
    context.transfer_successful_presentation_callbacks();

    let full_tree_sync = context.needs_full_tree_sync();
    if full_tree_sync {
        context.synchronize_full_tree();
    }

    let cleared_caches = context.should_clear_caches();
    if cleared_caches {
        context.clear_history_and_caches();
    }

    context.pull_property_trees();
    context.mark_property_trees_synced();

    let synchronized_surface_ranges = context.needs_surface_ranges_sync();
    if synchronized_surface_ranges {
        context.synchronize_surface_ranges();
    }

    context.push_layer_properties_and_scroll_updates();
    context.pull_layer_tree_properties();
    context.transfer_swap_promises();
    context.transfer_event_metrics();
    context.transfer_ui_resource_requests();
    context.update_property_tree_animation();
    context.push_mutator_properties();
    context.move_change_tracking_to_layers();
    context.update_scrollbar_geometries();
    context.finish_sync();

    trace.set("forced_redraw", forced_redraw as u8 as f64);
    trace.set(
        "forced_raster_scale_recalculation",
        forced_raster_scale_recalculation as u8 as f64,
    );
    trace.set("full_tree_sync", full_tree_sync as u8 as f64);
    trace.set("cleared_caches", cleared_caches as u8 as f64);
    trace.set(
        "synchronized_surface_ranges",
        synchronized_surface_ranges as u8 as f64,
    );

    CommitSyncStats {
        forced_redraw,
        forced_raster_scale_recalculation,
        full_tree_sync,
        cleared_caches,
        synchronized_surface_ranges,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Context {
        optional: bool,
        events: Vec<&'static str>,
    }

    macro_rules! event {
        ($name:ident, $value:literal) => {
            fn $name(&mut self) {
                self.events.push($value);
            }
        };
    }

    impl CommitSyncContext for Context {
        event!(begin_sync, "begin");
        fn commit_forces_redraw(&self) -> bool {
            self.optional
        }
        event!(force_redraw_next_activation, "redraw");
        fn commit_forces_raster_scale_recalculation(&self) -> bool {
            self.optional
        }
        event!(force_raster_scale_recalculation, "raster-scale");
        event!(transfer_presentation_callbacks, "presentation");
        event!(
            transfer_successful_presentation_callbacks,
            "successful-presentation"
        );
        fn needs_full_tree_sync(&self) -> bool {
            self.optional
        }
        event!(synchronize_full_tree, "full-tree");
        fn should_clear_caches(&self) -> bool {
            self.optional
        }
        event!(clear_history_and_caches, "clear-caches");
        event!(pull_property_trees, "property-trees");
        event!(mark_property_trees_synced, "property-trees-synced");
        fn needs_surface_ranges_sync(&self) -> bool {
            self.optional
        }
        event!(synchronize_surface_ranges, "surface-ranges");
        event!(push_layer_properties_and_scroll_updates, "layers-scroll");
        event!(pull_layer_tree_properties, "tree-properties");
        event!(transfer_swap_promises, "swap");
        event!(transfer_event_metrics, "metrics");
        event!(transfer_ui_resource_requests, "ui-resources");
        event!(update_property_tree_animation, "animation");
        event!(push_mutator_properties, "mutator");
        event!(move_change_tracking_to_layers, "change-tracking");
        event!(update_scrollbar_geometries, "scrollbars");
        event!(finish_sync, "finish");
    }

    #[test]
    fn complete_commit_preserves_stage_order() {
        let mut context = Context {
            optional: true,
            events: Vec::new(),
        };
        let stats = PullCommitIntoSyncTree(&mut context);
        assert_eq!(
            context.events,
            [
                "begin",
                "redraw",
                "raster-scale",
                "presentation",
                "successful-presentation",
                "full-tree",
                "clear-caches",
                "property-trees",
                "property-trees-synced",
                "surface-ranges",
                "layers-scroll",
                "tree-properties",
                "swap",
                "metrics",
                "ui-resources",
                "animation",
                "mutator",
                "change-tracking",
                "scrollbars",
                "finish",
            ]
        );
        assert_eq!(
            stats,
            CommitSyncStats {
                forced_redraw: true,
                forced_raster_scale_recalculation: true,
                full_tree_sync: true,
                cleared_caches: true,
                synchronized_surface_ranges: true,
            }
        );
    }

    #[test]
    fn incremental_commit_skips_optional_stages() {
        let mut context = Context::default();
        let stats = PullCommitIntoSyncTree(&mut context);
        assert_eq!(stats, CommitSyncStats::default());
        for skipped in [
            "redraw",
            "raster-scale",
            "full-tree",
            "clear-caches",
            "surface-ranges",
        ] {
            assert!(!context.events.contains(&skipped));
        }
        assert_eq!(context.events.first(), Some(&"begin"));
        assert_eq!(context.events.last(), Some(&"finish"));
    }
}
