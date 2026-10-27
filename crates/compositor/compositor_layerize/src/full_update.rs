//! Full PaintArtifact-to-compositor-layer update pipeline.
//!
//! Concrete Blink and cc objects stay behind the context adapter. Layerize
//! owns the update phases, their ordering, early exit, and per-phase traces.

pub trait FullUpdateContext {
    fn validate_full_update(&self);
    fn begin_update_trace(&mut self);
    fn has_attached_host(&self) -> bool;
    fn install_requests_callbacks_and_sequence(&mut self);
    fn prepare_layerization(&mut self) -> usize;
    fn layerize_pending_layers(&mut self) -> usize;
    fn build_and_commit_property_trees(&mut self) -> usize;
    fn finish_update_state(&mut self);
    fn clear_paint_property_change_markers(&mut self) -> usize;
    fn debug_log_result(&mut self);
    fn release_old_layer_clients(&mut self);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FullUpdateStats {
    pub attached_to_host: bool,
    pub old_layers: usize,
    pub layerized_layers: usize,
    pub committed_layers: usize,
    pub cleared_paint_chunks: usize,
}

fn RunPhase<T>(name: &'static str, run: impl FnOnce() -> T) -> T {
    let _trace = browser_tracing::span("compositor.layerize", name);
    run()
}

pub fn Update<C: FullUpdateContext>(context: &mut C) -> FullUpdateStats {
    let mut trace = browser_tracing::span("compositor.layerize", "Compositor.Layerize.FullUpdate");
    context.validate_full_update();
    context.begin_update_trace();
    if !context.has_attached_host() {
        trace.set("attached_to_host", 0.0);
        return FullUpdateStats::default();
    }

    let mut stats = FullUpdateStats {
        attached_to_host: true,
        ..Default::default()
    };
    RunPhase("Compositor.Layerize.FullUpdate.InstallInputs", || {
        context.install_requests_callbacks_and_sequence()
    });
    stats.old_layers = RunPhase("Compositor.Layerize.FullUpdate.Prepare", || {
        context.prepare_layerization()
    });
    stats.layerized_layers = RunPhase("Compositor.Layerize.FullUpdate.Layerize", || {
        context.layerize_pending_layers()
    });
    stats.committed_layers = RunPhase("Compositor.Layerize.FullUpdate.BuildPropertyTrees", || {
        context.build_and_commit_property_trees()
    });
    RunPhase("Compositor.Layerize.FullUpdate.FinishState", || {
        context.finish_update_state()
    });
    stats.cleared_paint_chunks = RunPhase(
        "Compositor.Layerize.FullUpdate.ClearPropertyChanges",
        || context.clear_paint_property_change_markers(),
    );
    context.debug_log_result();
    RunPhase("Compositor.Layerize.FullUpdate.ReleaseOldLayers", || {
        context.release_old_layer_clients()
    });

    trace.set("old_layers", stats.old_layers as f64);
    trace.set("layerized_layers", stats.layerized_layers as f64);
    trace.set("committed_layers", stats.committed_layers as f64);
    trace.set("paint_chunks", stats.cleared_paint_chunks as f64);
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Context {
        attached: bool,
        events: Vec<&'static str>,
    }

    impl FullUpdateContext for Context {
        fn validate_full_update(&self) {}
        fn begin_update_trace(&mut self) {
            self.events.push("trace");
        }
        fn has_attached_host(&self) -> bool {
            self.attached
        }
        fn install_requests_callbacks_and_sequence(&mut self) {
            self.events.push("inputs");
        }
        fn prepare_layerization(&mut self) -> usize {
            self.events.push("prepare");
            5
        }
        fn layerize_pending_layers(&mut self) -> usize {
            self.events.push("layerize");
            3
        }
        fn build_and_commit_property_trees(&mut self) -> usize {
            self.events.push("build");
            3
        }
        fn finish_update_state(&mut self) {
            self.events.push("finish");
        }
        fn clear_paint_property_change_markers(&mut self) -> usize {
            self.events.push("clear");
            7
        }
        fn debug_log_result(&mut self) {
            self.events.push("debug");
        }
        fn release_old_layer_clients(&mut self) {
            self.events.push("release");
        }
    }

    #[test]
    fn owns_full_update_phase_order_and_counts() {
        let mut context = Context {
            attached: true,
            ..Default::default()
        };
        let stats = Update(&mut context);
        assert_eq!(
            context.events,
            [
                "trace", "inputs", "prepare", "layerize", "build", "finish", "clear", "debug",
                "release"
            ]
        );
        assert_eq!(
            stats,
            FullUpdateStats {
                attached_to_host: true,
                old_layers: 5,
                layerized_layers: 3,
                committed_layers: 3,
                cleared_paint_chunks: 7,
            }
        );
    }

    #[test]
    fn detached_host_preserves_source_early_exit() {
        let mut context = Context::default();
        assert_eq!(Update(&mut context), FullUpdateStats::default());
        assert_eq!(context.events, ["trace"]);
    }
}
