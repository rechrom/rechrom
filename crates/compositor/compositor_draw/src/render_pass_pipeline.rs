//! Frame render-pass calculation and abort policy.

use crate::DrawResult;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderPassContentResult {
    pub num_missing_tiles: i32,
    pub have_missing_animated_tiles: bool,
}

pub trait RenderPassPipelineContext {
    fn layer_list_empty(&self) -> bool;
    fn create_empty_root_pass(&mut self);
    fn update_damage_tracking(&mut self);
    fn has_damage(&mut self) -> bool;
    fn validate_expected_draw_damage(&mut self, has_damage: bool);
    fn increment_consecutive_damage_frames(&mut self);
    fn trace_empty_damage(&mut self);
    fn mark_frame_has_no_damage(&mut self);
    fn reset_consecutive_damage_frames(&mut self);
    fn begin_render_pass_trace(&mut self);
    fn reset_handle_visibility_changed(&mut self);
    fn update_view_transition_save_state(&mut self);
    fn build_render_surface_passes(&mut self);
    fn damage_root_pass_for_hud(&mut self);
    fn prepare_compositing_throttling(&mut self);
    fn append_render_pass_quads(&mut self) -> RenderPassContentResult;
    fn commits_to_active_tree(&self) -> bool;
    fn report_expected_checkerboard_animation(&mut self);
    fn frame_checkerboarded_needs_raster(&self) -> bool;
    fn requires_high_resolution_to_draw(&self) -> bool;
    fn prevent_visible_scroll_checkerboarding(&mut self) -> bool;
    fn report_expected_missing_high_resolution_content(&mut self);
    fn resourceless_software_draw(&self) -> bool;
    fn frame_has_view_transition_save_directive(&self) -> bool;
    fn validate_render_passes(&mut self);
    fn fill_root_background(&mut self, num_missing_tiles: i32);
    fn outputs_frame_data(&self) -> bool;
    fn prune_render_passes(&mut self);
    fn finish_render_pass_trace(&mut self, result: DrawResult, num_missing_tiles: i32);
    fn validate_final_result(&self, result: DrawResult);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderPassPipelineStats {
    pub empty_tree: bool,
    pub no_damage_early_out: bool,
    pub num_missing_tiles: i32,
    pub aborted_checkerboard_animation: bool,
    pub aborted_missing_high_resolution_content: bool,
    pub forced_success: bool,
}

pub fn CalculateRenderPasses<C: RenderPassPipelineContext>(
    context: &mut C,
    expects_to_draw: bool,
) -> (DrawResult, RenderPassPipelineStats) {
    let mut trace =
        browser_tracing::span("compositor.draw", "Compositor.Draw.CalculateRenderPasses");
    if context.layer_list_empty() {
        context.create_empty_root_pass();
        trace.set("empty_tree", 1.0);
        return (
            DrawResult::kSuccess,
            RenderPassPipelineStats {
                empty_tree: true,
                ..Default::default()
            },
        );
    }

    context.update_damage_tracking();
    let mut has_damage = context.has_damage();
    if expects_to_draw {
        context.validate_expected_draw_damage(has_damage);
        has_damage = true;
    }
    if has_damage {
        context.increment_consecutive_damage_frames();
    } else {
        context.trace_empty_damage();
        context.mark_frame_has_no_damage();
        context.reset_consecutive_damage_frames();
        trace.set("no_damage_early_out", 1.0);
        return (
            DrawResult::kSuccess,
            RenderPassPipelineStats {
                no_damage_early_out: true,
                ..Default::default()
            },
        );
    }

    context.begin_render_pass_trace();
    context.reset_handle_visibility_changed();
    context.update_view_transition_save_state();
    context.build_render_surface_passes();
    context.damage_root_pass_for_hud();
    context.prepare_compositing_throttling();
    let content = context.append_render_pass_quads();

    let mut result = DrawResult::kSuccess;
    let mut stats = RenderPassPipelineStats {
        num_missing_tiles: content.num_missing_tiles,
        ..Default::default()
    };
    if content.have_missing_animated_tiles && !context.commits_to_active_tree() {
        if expects_to_draw {
            context.report_expected_checkerboard_animation();
        } else {
            result = DrawResult::kAbortedCheckerboardAnimations;
            stats.aborted_checkerboard_animation = true;
        }
    }
    if (context.frame_checkerboarded_needs_raster() && context.requires_high_resolution_to_draw())
        || (content.num_missing_tiles > 0 && context.prevent_visible_scroll_checkerboarding())
    {
        if expects_to_draw {
            context.report_expected_missing_high_resolution_content();
        } else {
            result = DrawResult::kAbortedMissingHighResContent;
            stats.aborted_missing_high_resolution_content = true;
        }
    }
    if context.resourceless_software_draw() || context.frame_has_view_transition_save_directive() {
        result = DrawResult::kSuccess;
        stats.forced_success = true;
    }
    context.validate_render_passes();
    context.fill_root_background(content.num_missing_tiles);
    if context.outputs_frame_data() {
        context.prune_render_passes();
    }
    context.finish_render_pass_trace(result, content.num_missing_tiles);
    context.validate_final_result(result);

    trace.set("missing_tiles", content.num_missing_tiles as f64);
    trace.set(
        "aborted_checkerboard_animation",
        stats.aborted_checkerboard_animation as u8 as f64,
    );
    trace.set(
        "aborted_missing_high_resolution_content",
        stats.aborted_missing_high_resolution_content as u8 as f64,
    );
    trace.set("forced_success", stats.forced_success as u8 as f64);
    (result, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Context {
        empty: bool,
        damage: bool,
        content: RenderPassContentResult,
        commit_active: bool,
        missing_high_res: bool,
        require_high_res: bool,
        prevent_scroll_checkerboard: bool,
        resourceless: bool,
        transition: bool,
        output: bool,
        events: Vec<&'static str>,
    }

    macro_rules! event {
        ($name:ident, $value:literal) => {
            fn $name(&mut self) {
                self.events.push($value);
            }
        };
    }

    impl RenderPassPipelineContext for Context {
        fn layer_list_empty(&self) -> bool {
            self.empty
        }
        event!(create_empty_root_pass, "empty-pass");
        event!(update_damage_tracking, "damage-tracking");
        fn has_damage(&mut self) -> bool {
            self.events.push("has-damage");
            self.damage
        }
        fn validate_expected_draw_damage(&mut self, _: bool) {
            self.events.push("validate-expected");
        }
        event!(increment_consecutive_damage_frames, "increment-damage");
        event!(trace_empty_damage, "trace-empty");
        event!(mark_frame_has_no_damage, "mark-empty");
        event!(reset_consecutive_damage_frames, "reset-damage");
        event!(begin_render_pass_trace, "begin");
        event!(reset_handle_visibility_changed, "reset-visibility");
        event!(update_view_transition_save_state, "transition-state");
        event!(build_render_surface_passes, "build-passes");
        event!(damage_root_pass_for_hud, "hud");
        event!(prepare_compositing_throttling, "throttle");
        fn append_render_pass_quads(&mut self) -> RenderPassContentResult {
            self.events.push("quads");
            self.content
        }
        fn commits_to_active_tree(&self) -> bool {
            self.commit_active
        }
        event!(
            report_expected_checkerboard_animation,
            "checkerboard-diagnostic"
        );
        fn frame_checkerboarded_needs_raster(&self) -> bool {
            self.missing_high_res
        }
        fn requires_high_resolution_to_draw(&self) -> bool {
            self.require_high_res
        }
        fn prevent_visible_scroll_checkerboarding(&mut self) -> bool {
            self.prevent_scroll_checkerboard
        }
        event!(
            report_expected_missing_high_resolution_content,
            "high-res-diagnostic"
        );
        fn resourceless_software_draw(&self) -> bool {
            self.resourceless
        }
        fn frame_has_view_transition_save_directive(&self) -> bool {
            self.transition
        }
        event!(validate_render_passes, "validate-passes");
        fn fill_root_background(&mut self, _: i32) {
            self.events.push("background");
        }
        fn outputs_frame_data(&self) -> bool {
            self.output
        }
        event!(prune_render_passes, "prune");
        fn finish_render_pass_trace(&mut self, _: DrawResult, _: i32) {
            self.events.push("finish");
        }
        fn validate_final_result(&self, _: DrawResult) {}
    }

    #[test]
    fn no_damage_stops_before_pass_building() {
        let mut context = Context::default();
        let (result, stats) = CalculateRenderPasses(&mut context, false);
        assert_eq!(result, DrawResult::kSuccess);
        assert!(stats.no_damage_early_out);
        assert_eq!(
            context.events,
            [
                "damage-tracking",
                "has-damage",
                "trace-empty",
                "mark-empty",
                "reset-damage"
            ]
        );
    }

    #[test]
    fn owns_abort_precedence_pruning_and_final_order() {
        let mut context = Context {
            damage: true,
            content: RenderPassContentResult {
                num_missing_tiles: 4,
                have_missing_animated_tiles: true,
            },
            missing_high_res: true,
            require_high_res: true,
            output: true,
            ..Default::default()
        };
        let (result, stats) = CalculateRenderPasses(&mut context, false);
        assert_eq!(result, DrawResult::kAbortedMissingHighResContent);
        assert!(stats.aborted_checkerboard_animation);
        assert!(stats.aborted_missing_high_resolution_content);
        assert_eq!(
            &context.events[context.events.len() - 4..],
            ["validate-passes", "background", "prune", "finish"]
        );
    }

    #[test]
    fn resourceless_draw_overrides_abort() {
        let mut context = Context {
            damage: true,
            content: RenderPassContentResult {
                num_missing_tiles: 1,
                have_missing_animated_tiles: true,
            },
            resourceless: true,
            ..Default::default()
        };
        let (result, stats) = CalculateRenderPasses(&mut context, false);
        assert_eq!(result, DrawResult::kSuccess);
        assert!(stats.forced_success);
    }

    #[test]
    fn visible_missing_tile_aborts_checkerboarded_scroll_only() {
        let mut context = Context {
            damage: true,
            content: RenderPassContentResult {
                num_missing_tiles: 1,
                have_missing_animated_tiles: false,
            },
            prevent_scroll_checkerboard: true,
            ..Default::default()
        };
        let (result, stats) = CalculateRenderPasses(&mut context, false);
        assert_eq!(result, DrawResult::kAbortedMissingHighResContent);
        assert!(stats.aborted_missing_high_resolution_content);
    }
}
