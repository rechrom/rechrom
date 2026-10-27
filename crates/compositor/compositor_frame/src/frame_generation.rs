//! Compositor-frame generation pipeline.
//!
//! The facade supplies concrete tree, metadata, resource, and frame operations.
//! This module owns the phase order and every branch that decides which frame
//! generation work runs, so generation cost is attributable to Frame.

use crate::view_transition_requests::ViewTransitionRequestResult;

pub trait FrameGenerationContext {
    type Metadata;
    type Frame;

    fn trees_in_viz_process(&self) -> bool;
    fn record_memory_history(&mut self);
    fn debug_rects_enabled(&self) -> bool;
    fn save_debug_rects(&mut self);
    fn trace_scroll_delta_and_reset(&mut self);
    fn is_new_trace(&mut self) -> bool;
    fn begin_layer_tracing(&mut self);
    fn frame_viewer_trace_enabled(&mut self) -> bool;
    fn trace_frame_viewer_snapshot(&mut self);
    fn update_hud(&mut self);

    fn make_metadata(&mut self) -> Self::Metadata;
    fn process_view_transition_requests(
        &mut self,
        metadata: &mut Self::Metadata,
    ) -> ViewTransitionRequestResult;
    fn populate_content_color_usage(&mut self, metadata: &mut Self::Metadata);
    fn set_shared_element_resources(&mut self, metadata: &mut Self::Metadata);
    fn set_frame_deadline(&mut self, metadata: &mut Self::Metadata, force_240_frames: bool);
    fn populate_interval_metadata(&mut self, metadata: &mut Self::Metadata);
    fn finalize_pre_observer_metadata(&mut self, metadata: &mut Self::Metadata);

    fn has_render_frame_metadata_observer(&self) -> bool;
    fn process_render_frame_metadata(&mut self, metadata: &mut Self::Metadata);
    fn commits_to_active_tree(&self) -> bool;
    fn metadata_has_latency(&self, metadata: &Self::Metadata) -> bool;
    fn annotate_latency(&mut self, metadata: &mut Self::Metadata);

    fn has_unbounded_render_passes(&self) -> bool;
    fn has_unbounded_frame_sink_handler(&self) -> bool;
    fn submit_unbounded_frame(&mut self);
    fn finalize_frame(&mut self, metadata: Self::Metadata) -> Self::Frame;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameGenerationStats {
    pub recorded_memory_history: bool,
    pub saved_debug_rects: bool,
    pub began_layer_tracing: bool,
    pub traced_frame_viewer_snapshot: bool,
    pub processed_render_frame_metadata: bool,
    pub annotated_latency: bool,
    pub submitted_unbounded_frame: bool,
    pub forced_view_transition_deadline: bool,
}

pub fn GenerateCompositorFrame<C: FrameGenerationContext>(
    context: &mut C,
) -> (C::Frame, FrameGenerationStats) {
    let mut trace = browser_tracing::span("compositor.frame", "Compositor.Frame.Generate");
    let mut stats = FrameGenerationStats::default();

    if !context.trees_in_viz_process() {
        context.record_memory_history();
        stats.recorded_memory_history = true;
    }
    if context.debug_rects_enabled() {
        context.save_debug_rects();
        stats.saved_debug_rects = true;
    }
    context.trace_scroll_delta_and_reset();
    if context.is_new_trace() {
        context.begin_layer_tracing();
        stats.began_layer_tracing = true;
    }
    if context.frame_viewer_trace_enabled() {
        context.trace_frame_viewer_snapshot();
        stats.traced_frame_viewer_snapshot = true;
    }
    context.update_hud();

    let mut metadata = context.make_metadata();
    let transition = context.process_view_transition_requests(&mut metadata);
    context.populate_content_color_usage(&mut metadata);
    context.set_shared_element_resources(&mut metadata);
    stats.forced_view_transition_deadline =
        transition.delay_layer_tree_view_deletion && transition.has_animate_renderer;
    context.set_frame_deadline(&mut metadata, stats.forced_view_transition_deadline);
    context.populate_interval_metadata(&mut metadata);
    context.finalize_pre_observer_metadata(&mut metadata);

    if context.has_render_frame_metadata_observer() || context.trees_in_viz_process() {
        context.process_render_frame_metadata(&mut metadata);
        stats.processed_render_frame_metadata = true;
    }
    if !context.commits_to_active_tree() && context.metadata_has_latency(&metadata) {
        context.annotate_latency(&mut metadata);
        stats.annotated_latency = true;
    }
    if context.has_unbounded_render_passes() && context.has_unbounded_frame_sink_handler() {
        context.submit_unbounded_frame();
        stats.submitted_unbounded_frame = true;
    }

    let frame = context.finalize_frame(metadata);
    trace.set(
        "forced_view_transition_deadline",
        stats.forced_view_transition_deadline as u8 as f64,
    );
    trace.set(
        "render_frame_metadata",
        stats.processed_render_frame_metadata as u8 as f64,
    );
    trace.set("annotated_latency", stats.annotated_latency as u8 as f64);
    trace.set(
        "unbounded_frame",
        stats.submitted_unbounded_frame as u8 as f64,
    );
    (frame, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Context {
        trees_in_viz: bool,
        debug: bool,
        new_trace: bool,
        viewer_trace: bool,
        observer: bool,
        commits_to_active: bool,
        latency: bool,
        unbounded: bool,
        unbounded_sink: bool,
        transition: ViewTransitionRequestResult,
        events: Vec<&'static str>,
    }

    macro_rules! event {
        ($name:ident, $value:literal) => {
            fn $name(&mut self) {
                self.events.push($value);
            }
        };
    }

    impl FrameGenerationContext for Context {
        type Metadata = ();
        type Frame = &'static str;

        fn trees_in_viz_process(&self) -> bool {
            self.trees_in_viz
        }
        event!(record_memory_history, "memory");
        fn debug_rects_enabled(&self) -> bool {
            self.debug
        }
        event!(save_debug_rects, "debug");
        event!(trace_scroll_delta_and_reset, "scroll-trace");
        fn is_new_trace(&mut self) -> bool {
            self.events.push("is-new-trace");
            self.new_trace
        }
        event!(begin_layer_tracing, "begin-tracing");
        fn frame_viewer_trace_enabled(&mut self) -> bool {
            self.events.push("viewer-enabled");
            self.viewer_trace
        }
        event!(trace_frame_viewer_snapshot, "viewer-snapshot");
        event!(update_hud, "hud");
        fn make_metadata(&mut self) -> () {
            self.events.push("make-metadata");
        }
        fn process_view_transition_requests(&mut self, _: &mut ()) -> ViewTransitionRequestResult {
            self.events.push("transitions");
            self.transition
        }
        fn populate_content_color_usage(&mut self, _: &mut ()) {
            self.events.push("colors");
        }
        fn set_shared_element_resources(&mut self, _: &mut ()) {
            self.events.push("shared");
        }
        fn set_frame_deadline(&mut self, _: &mut (), forced: bool) {
            self.events
                .push(if forced { "deadline-240" } else { "deadline" });
        }
        fn populate_interval_metadata(&mut self, _: &mut ()) {
            self.events.push("intervals");
        }
        fn finalize_pre_observer_metadata(&mut self, _: &mut ()) {
            self.events.push("pre-observer");
        }
        fn has_render_frame_metadata_observer(&self) -> bool {
            self.observer
        }
        fn process_render_frame_metadata(&mut self, _: &mut ()) {
            self.events.push("render-metadata");
        }
        fn commits_to_active_tree(&self) -> bool {
            self.commits_to_active
        }
        fn metadata_has_latency(&self, _: &()) -> bool {
            self.latency
        }
        fn annotate_latency(&mut self, _: &mut ()) {
            self.events.push("latency");
        }
        fn has_unbounded_render_passes(&self) -> bool {
            self.unbounded
        }
        fn has_unbounded_frame_sink_handler(&self) -> bool {
            self.unbounded_sink
        }
        event!(submit_unbounded_frame, "unbounded");
        fn finalize_frame(&mut self, _: ()) -> &'static str {
            self.events.push("finalize");
            "frame"
        }
    }

    #[test]
    fn owns_full_generation_order_and_optional_branches() {
        let mut context = Context {
            debug: true,
            new_trace: true,
            viewer_trace: true,
            observer: true,
            latency: true,
            unbounded: true,
            unbounded_sink: true,
            transition: ViewTransitionRequestResult {
                delay_layer_tree_view_deletion: true,
                has_animate_renderer: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let (frame, stats) = GenerateCompositorFrame(&mut context);
        assert_eq!(frame, "frame");
        assert_eq!(
            context.events,
            [
                "memory",
                "debug",
                "scroll-trace",
                "is-new-trace",
                "begin-tracing",
                "viewer-enabled",
                "viewer-snapshot",
                "hud",
                "make-metadata",
                "transitions",
                "colors",
                "shared",
                "deadline-240",
                "intervals",
                "pre-observer",
                "render-metadata",
                "latency",
                "unbounded",
                "finalize",
            ]
        );
        assert!(stats.recorded_memory_history);
        assert!(stats.forced_view_transition_deadline);
        assert!(stats.submitted_unbounded_frame);
    }

    #[test]
    fn short_circuits_optional_queries_and_work() {
        let mut context = Context {
            trees_in_viz: true,
            commits_to_active: true,
            latency: true,
            unbounded: false,
            unbounded_sink: true,
            ..Default::default()
        };
        let (_, stats) = GenerateCompositorFrame(&mut context);
        assert_eq!(
            context.events,
            [
                "scroll-trace",
                "is-new-trace",
                "viewer-enabled",
                "hud",
                "make-metadata",
                "transitions",
                "colors",
                "shared",
                "deadline",
                "intervals",
                "pre-observer",
                "render-metadata",
                "finalize",
            ]
        );
        assert!(!stats.recorded_memory_history);
        assert!(!stats.annotated_latency);
        assert!(!stats.submitted_unbounded_frame);
    }
}
