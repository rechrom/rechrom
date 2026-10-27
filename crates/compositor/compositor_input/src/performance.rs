//! Package-owned trace intervals for live InputHandler operations.

pub fn BeginHitTestTrace() -> browser_tracing::Span {
    browser_tracing::span("compositor.input", "Compositor.Input.HitTest")
}

pub struct ScrollUpdateTrace {
    span: browser_tracing::Span,
}

impl ScrollUpdateTrace {
    pub fn new(delta_x: f64, delta_y: f64, granularity: u8) -> Self {
        let mut span = browser_tracing::span("compositor.input", "Compositor.Input.ScrollUpdate");
        span.set("delta_x", delta_x);
        span.set("delta_y", delta_y);
        span.set("granularity", granularity as f64);
        Self { span }
    }

    pub fn no_current_scroll_node(&mut self) {
        self.span.set("no_current_scroll_node", 1.0);
    }

    pub fn finish(
        &mut self,
        did_scroll: bool,
        did_overscroll_root: bool,
        hit_snap_constraint: bool,
        needs_main_thread_repaint: bool,
    ) {
        self.span.set("did_scroll", did_scroll as u8 as f64);
        self.span
            .set("did_overscroll_root", did_overscroll_root as u8 as f64);
        self.span
            .set("hit_snap_constraint", hit_snap_constraint as u8 as f64);
        self.span.set(
            "needs_main_thread_repaint",
            needs_main_thread_repaint as u8 as f64,
        );
    }
}
