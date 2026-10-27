//! Per-frame draw preparation ordering.

pub trait PrepareToDrawContext {
    type DrawResult: Copy;

    fn trace_source_prepare(&mut self);
    fn input_will_draw(&mut self);
    fn tick_worklet_animations(&mut self);
    fn update_draw_properties(&mut self) -> bool;
    fn uses_external_tree_raster(&self) -> bool;
    fn prepare_resident_tiles_to_draw(&mut self) -> bool;
    fn set_checkerboarded_needs_raster(&mut self, value: bool);
    fn reset_frame_for_draw(&mut self);
    fn root_render_surface_exists(&self) -> bool;
    fn add_viewport_damage(&mut self);
    fn calculate_render_passes(&mut self, expects_to_draw: bool) -> Self::DrawResult;
    fn verbose_log_frame(&mut self);
    fn draw_succeeded(&self, result: Self::DrawResult) -> bool;
    fn assert_not_resourceless_draw(&self);
    fn report_expected_draw_failure(&mut self);
    fn success_result(&self) -> Self::DrawResult;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrepareToDrawStats {
    pub resident_tile_preparation: bool,
    pub checkerboarded_needs_raster: bool,
    pub added_viewport_damage: bool,
    pub coerced_expected_failure_to_success: bool,
}

pub fn PrepareToDraw<C: PrepareToDrawContext>(
    context: &mut C,
    expects_to_draw: bool,
) -> (C::DrawResult, PrepareToDrawStats) {
    let mut trace = browser_tracing::span("compositor.draw", "Compositor.Draw.PrepareToDraw");
    context.trace_source_prepare();
    context.input_will_draw();
    context.tick_worklet_animations();
    let draw_properties_ok = context.update_draw_properties();
    debug_assert!(
        draw_properties_ok,
        "UpdateDrawProperties failed during draw"
    );

    let resident_tile_preparation = !context.uses_external_tree_raster();
    let mut checkerboarded_needs_raster = false;
    if resident_tile_preparation {
        checkerboarded_needs_raster = !context.prepare_resident_tiles_to_draw();
        context.set_checkerboarded_needs_raster(checkerboarded_needs_raster);
    }

    context.reset_frame_for_draw();
    let added_viewport_damage = context.root_render_surface_exists();
    if added_viewport_damage {
        context.add_viewport_damage();
    }

    let draw_result = context.calculate_render_passes(expects_to_draw);
    context.verbose_log_frame();
    let mut coerced_expected_failure_to_success = false;
    let result = if context.draw_succeeded(draw_result) {
        draw_result
    } else {
        context.assert_not_resourceless_draw();
        if expects_to_draw {
            context.report_expected_draw_failure();
            coerced_expected_failure_to_success = true;
            context.success_result()
        } else {
            draw_result
        }
    };

    trace.set(
        "resident_tile_preparation",
        resident_tile_preparation as u8 as f64,
    );
    trace.set(
        "checkerboarded_needs_raster",
        checkerboarded_needs_raster as u8 as f64,
    );
    trace.set("added_viewport_damage", added_viewport_damage as u8 as f64);
    trace.set(
        "coerced_expected_failure_to_success",
        coerced_expected_failure_to_success as u8 as f64,
    );
    (
        result,
        PrepareToDrawStats {
            resident_tile_preparation,
            checkerboarded_needs_raster,
            added_viewport_damage,
            coerced_expected_failure_to_success,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Result {
        Success,
        Aborted,
    }

    struct Context {
        external_raster: bool,
        tiles_ready: bool,
        root: bool,
        result: Result,
        events: Vec<&'static str>,
    }

    impl PrepareToDrawContext for Context {
        type DrawResult = Result;
        fn trace_source_prepare(&mut self) {
            self.events.push("trace");
        }
        fn input_will_draw(&mut self) {
            self.events.push("input");
        }
        fn tick_worklet_animations(&mut self) {
            self.events.push("worklet");
        }
        fn update_draw_properties(&mut self) -> bool {
            self.events.push("properties");
            true
        }
        fn uses_external_tree_raster(&self) -> bool {
            self.external_raster
        }
        fn prepare_resident_tiles_to_draw(&mut self) -> bool {
            self.events.push("tiles");
            self.tiles_ready
        }
        fn set_checkerboarded_needs_raster(&mut self, _: bool) {
            self.events.push("checkerboard");
        }
        fn reset_frame_for_draw(&mut self) {
            self.events.push("reset-frame");
        }
        fn root_render_surface_exists(&self) -> bool {
            self.root
        }
        fn add_viewport_damage(&mut self) {
            self.events.push("damage");
        }
        fn calculate_render_passes(&mut self, _: bool) -> Result {
            self.events.push("passes");
            self.result
        }
        fn verbose_log_frame(&mut self) {
            self.events.push("log");
        }
        fn draw_succeeded(&self, result: Result) -> bool {
            result == Result::Success
        }
        fn assert_not_resourceless_draw(&self) {}
        fn report_expected_draw_failure(&mut self) {
            self.events.push("expected-failure");
        }
        fn success_result(&self) -> Result {
            Result::Success
        }
    }

    #[test]
    fn prepares_tiles_damage_and_passes_in_order() {
        let mut context = Context {
            external_raster: false,
            tiles_ready: false,
            root: true,
            result: Result::Success,
            events: Vec::new(),
        };
        let (result, stats) = PrepareToDraw(&mut context, false);
        assert_eq!(result, Result::Success);
        assert_eq!(
            context.events,
            [
                "trace",
                "input",
                "worklet",
                "properties",
                "tiles",
                "checkerboard",
                "reset-frame",
                "damage",
                "passes",
                "log"
            ]
        );
        assert!(stats.checkerboarded_needs_raster);
        assert!(stats.added_viewport_damage);
    }

    #[test]
    fn expected_draw_coerces_abort_after_reporting() {
        let mut context = Context {
            external_raster: true,
            tiles_ready: true,
            root: false,
            result: Result::Aborted,
            events: Vec::new(),
        };
        let (result, stats) = PrepareToDraw(&mut context, true);
        assert_eq!(result, Result::Success);
        assert!(stats.coerced_expected_failure_to_success);
        assert!(context.events.contains(&"expected-failure"));
        assert!(!context.events.contains(&"tiles"));
    }
}
