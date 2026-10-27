//! Paint-artifact fast-path update selection and sequencing.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FastPathUpdateKind {
    NoChanges,
    RasterInducingScroll,
    Repaint,
    Full,
}

pub trait FastPathUpdateContext {
    fn validate_fast_path_inputs(&mut self);
    fn update_kind(&self) -> FastPathUpdateKind;
    fn update_raster_inducing_scroll(&mut self) -> usize;
    fn update_repaint(&mut self) -> usize;
    fn finish_fast_path_update(&mut self, kind: FastPathUpdateKind);
    fn debug_log_fast_path_result(&mut self);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FastPathUpdateStats {
    pub layers_updated: usize,
}

pub fn TryFastPathUpdate<C: FastPathUpdateContext>(context: &mut C) -> (bool, FastPathUpdateStats) {
    let mut trace =
        browser_tracing::span("compositor.layerize", "Compositor.Layerize.FastPathUpdate");
    context.validate_fast_path_inputs();
    let kind = context.update_kind();
    trace.set("kind", kind as u8 as f64);
    let mut stats = FastPathUpdateStats::default();
    match kind {
        FastPathUpdateKind::NoChanges => return (true, stats),
        FastPathUpdateKind::RasterInducingScroll => {
            stats.layers_updated = context.update_raster_inducing_scroll();
        }
        FastPathUpdateKind::Repaint => {
            stats.layers_updated = context.update_repaint();
        }
        FastPathUpdateKind::Full => return (false, stats),
    }
    context.finish_fast_path_update(kind);
    context.debug_log_fast_path_result();
    trace.set("layers_updated", stats.layers_updated as f64);
    (true, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Context {
        kind: FastPathUpdateKind,
        log: Vec<&'static str>,
    }
    impl FastPathUpdateContext for Context {
        fn validate_fast_path_inputs(&mut self) {
            self.log.push("validate");
        }
        fn update_kind(&self) -> FastPathUpdateKind {
            self.kind
        }
        fn update_raster_inducing_scroll(&mut self) -> usize {
            self.log.push("scroll");
            3
        }
        fn update_repaint(&mut self) -> usize {
            self.log.push("repaint");
            4
        }
        fn finish_fast_path_update(&mut self, _kind: FastPathUpdateKind) {
            self.log.push("finish");
        }
        fn debug_log_fast_path_result(&mut self) {
            self.log.push("debug");
        }
    }

    #[test]
    fn repaint_path_owns_action_finish_and_debug_order() {
        let mut context = Context {
            kind: FastPathUpdateKind::Repaint,
            log: Vec::new(),
        };
        let (used, stats) = TryFastPathUpdate(&mut context);
        assert!(used);
        assert_eq!(stats.layers_updated, 4);
        assert_eq!(context.log, ["validate", "repaint", "finish", "debug"]);
    }

    #[test]
    fn no_changes_and_full_exit_without_finishing() {
        for (kind, used) in [
            (FastPathUpdateKind::NoChanges, true),
            (FastPathUpdateKind::Full, false),
        ] {
            let mut context = Context {
                kind,
                log: Vec::new(),
            };
            assert_eq!(TryFastPathUpdate(&mut context).0, used);
            assert_eq!(context.log, ["validate"]);
        }
    }
}
