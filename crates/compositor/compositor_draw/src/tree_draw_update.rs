//! Active LayerTree draw-property update lifecycle.

pub trait TreeDrawUpdateContext {
    fn needs_draw_property_update(&self) -> bool;
    fn begin_draw_property_update(&mut self);
    fn has_frame_sink(&self) -> bool;
    fn clear_render_surface_list(&mut self);
    fn layer_list_is_empty(&self) -> bool;
    fn calculate_draw_properties(&mut self);
    fn run_occlusion_walk(&mut self);
    fn is_resourceless_software_draw(&self) -> bool;
    fn mark_tiles_need_update(&mut self);
    fn update_tiles(&mut self) -> bool;
    fn did_modify_tile_priorities(&mut self, tile_update_was_deferred: bool);
    fn has_image_animation_controller(&self) -> bool;
    fn update_image_animation_state(&mut self);
    fn finish_draw_property_update(&mut self);
    fn assert_draw_properties_clean(&self);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TreeDrawUpdateStats {
    pub success: bool,
    pub skipped_clean_tree: bool,
    pub missing_frame_sink: bool,
    pub empty_layer_tree: bool,
    pub tile_update_requested: bool,
    pub tile_priorities_modified: bool,
    pub image_state_updated: bool,
}

pub fn UpdateTreeDrawProperties<C: TreeDrawUpdateContext>(
    context: &mut C,
    update_tiles: bool,
    update_images: bool,
) -> TreeDrawUpdateStats {
    let mut trace = browser_tracing::span(
        "compositor.draw",
        "Compositor.Draw.UpdateTreeDrawProperties",
    );
    if !context.needs_draw_property_update() {
        trace.set("skipped_clean_tree", 1.0);
        return TreeDrawUpdateStats {
            success: true,
            skipped_clean_tree: true,
            ..TreeDrawUpdateStats::default()
        };
    }

    context.begin_draw_property_update();
    if !context.has_frame_sink() {
        trace.set("missing_frame_sink", 1.0);
        return TreeDrawUpdateStats {
            missing_frame_sink: true,
            ..TreeDrawUpdateStats::default()
        };
    }

    context.clear_render_surface_list();
    if context.layer_list_is_empty() {
        trace.set("empty_layer_tree", 1.0);
        return TreeDrawUpdateStats {
            success: true,
            empty_layer_tree: true,
            ..TreeDrawUpdateStats::default()
        };
    }

    context.calculate_draw_properties();
    context.run_occlusion_walk();

    let tile_update_requested = !context.is_resourceless_software_draw();
    let mut tile_priorities_modified = false;
    if tile_update_requested {
        context.mark_tiles_need_update();
        let may_update_priorities = !update_tiles || context.update_tiles();
        if may_update_priorities {
            context.did_modify_tile_priorities(!update_tiles);
            tile_priorities_modified = true;
        }
    }

    let image_state_updated = update_images && context.has_image_animation_controller();
    if image_state_updated {
        context.update_image_animation_state();
    }

    context.finish_draw_property_update();
    context.assert_draw_properties_clean();
    trace.set("tile_update_requested", tile_update_requested as u8 as f64);
    trace.set(
        "tile_priorities_modified",
        tile_priorities_modified as u8 as f64,
    );
    trace.set("image_state_updated", image_state_updated as u8 as f64);
    TreeDrawUpdateStats {
        success: true,
        tile_update_requested,
        tile_priorities_modified,
        image_state_updated,
        ..TreeDrawUpdateStats::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Context {
        dirty: bool,
        frame_sink: bool,
        empty: bool,
        resourceless: bool,
        tile_changed: bool,
        images: bool,
        events: Vec<&'static str>,
    }

    impl TreeDrawUpdateContext for Context {
        fn needs_draw_property_update(&self) -> bool {
            self.dirty
        }
        fn begin_draw_property_update(&mut self) {
            self.events.push("begin");
            self.dirty = false;
        }
        fn has_frame_sink(&self) -> bool {
            self.frame_sink
        }
        fn clear_render_surface_list(&mut self) {
            self.events.push("clear-surfaces");
        }
        fn layer_list_is_empty(&self) -> bool {
            self.empty
        }
        fn calculate_draw_properties(&mut self) {
            self.events.push("properties");
        }
        fn run_occlusion_walk(&mut self) {
            self.events.push("occlusion");
        }
        fn is_resourceless_software_draw(&self) -> bool {
            self.resourceless
        }
        fn mark_tiles_need_update(&mut self) {
            self.events.push("mark-tiles");
        }
        fn update_tiles(&mut self) -> bool {
            self.events.push("tiles");
            self.tile_changed
        }
        fn did_modify_tile_priorities(&mut self, deferred: bool) {
            self.events.push(if deferred {
                "priorities-deferred"
            } else {
                "priorities"
            });
        }
        fn has_image_animation_controller(&self) -> bool {
            self.images
        }
        fn update_image_animation_state(&mut self) {
            self.events.push("images");
        }
        fn finish_draw_property_update(&mut self) {
            self.events.push("finish");
        }
        fn assert_draw_properties_clean(&self) {
            assert!(!self.dirty);
        }
    }

    #[test]
    fn complete_update_preserves_draw_then_occlusion_then_tile_order() {
        let mut context = Context {
            dirty: true,
            frame_sink: true,
            empty: false,
            resourceless: false,
            tile_changed: true,
            images: true,
            events: Vec::new(),
        };
        let stats = UpdateTreeDrawProperties(&mut context, true, true);
        assert_eq!(
            context.events,
            [
                "begin",
                "clear-surfaces",
                "properties",
                "occlusion",
                "mark-tiles",
                "tiles",
                "priorities",
                "images",
                "finish",
            ]
        );
        assert!(stats.success);
        assert!(stats.tile_update_requested);
        assert!(stats.tile_priorities_modified);
        assert!(stats.image_state_updated);
    }

    #[test]
    fn clean_tree_and_missing_sink_have_distinct_outcomes() {
        let mut clean = Context {
            dirty: false,
            frame_sink: true,
            empty: false,
            resourceless: false,
            tile_changed: false,
            images: false,
            events: Vec::new(),
        };
        let clean_stats = UpdateTreeDrawProperties(&mut clean, true, true);
        assert!(clean_stats.success && clean_stats.skipped_clean_tree);
        assert!(clean.events.is_empty());

        clean.dirty = true;
        clean.frame_sink = false;
        let missing = UpdateTreeDrawProperties(&mut clean, true, true);
        assert!(!missing.success && missing.missing_frame_sink);
        assert_eq!(clean.events, ["begin"]);
    }

    #[test]
    fn deferred_tiles_reprioritize_without_running_tile_update() {
        let mut context = Context {
            dirty: true,
            frame_sink: true,
            empty: false,
            resourceless: false,
            tile_changed: false,
            images: false,
            events: Vec::new(),
        };
        let stats = UpdateTreeDrawProperties(&mut context, false, false);
        assert!(!context.events.contains(&"tiles"));
        assert!(context.events.contains(&"priorities-deferred"));
        assert!(stats.tile_priorities_modified);
    }
}
