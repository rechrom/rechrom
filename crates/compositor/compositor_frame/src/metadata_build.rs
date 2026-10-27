//! Compositor and browser render-frame metadata build pipelines.

pub trait CompositorMetadataContext {
    type Metadata;

    fn new_metadata(&mut self) -> Self::Metadata;
    fn trees_in_viz_process(&self) -> bool;
    fn set_client_frame_token(&mut self, metadata: &mut Self::Metadata);
    fn increment_and_set_frame_token(&mut self, metadata: &mut Self::Metadata);
    fn populate_viewport_fields(&mut self, metadata: &mut Self::Metadata);
    fn register_presentation_callbacks(&mut self, metadata: &mut Self::Metadata);
    fn populate_activity_fields(&mut self, metadata: &mut Self::Metadata);
    fn copy_referenced_surfaces(&mut self, metadata: &mut Self::Metadata);
    fn populate_page_scale_and_browser_controls(&mut self, metadata: &mut Self::Metadata);
    fn populate_root_scroll_display_and_mobile(&mut self, metadata: &mut Self::Metadata);
    fn populate_delegated_ink(&mut self, metadata: &mut Self::Metadata);
    fn populate_capture_bounds(&mut self, metadata: &mut Self::Metadata);
    fn copy_client_tracked_element_rects(&mut self, metadata: &mut Self::Metadata);
    fn collect_tracked_element_rects(&mut self, metadata: &mut Self::Metadata);
    fn has_screenshot_destination(&self) -> bool;
    fn set_screenshot_destination(&mut self, metadata: &mut Self::Metadata);
    fn trees_in_viz_client_process(&mut self) -> bool;
    fn clear_screenshot_destination(&mut self);
    fn populate_draw_mode(&mut self, metadata: &mut Self::Metadata);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompositorMetadataStats {
    pub client_frame_token: bool,
    pub client_tracked_rects: bool,
    pub included_screenshot_destination: bool,
    pub consumed_screenshot_destination: bool,
}

pub fn BuildCompositorMetadata<C: CompositorMetadataContext>(
    context: &mut C,
) -> (C::Metadata, CompositorMetadataStats) {
    let mut trace = browser_tracing::span(
        "compositor.frame",
        "Compositor.Frame.BuildCompositorMetadata",
    );
    let mut metadata = context.new_metadata();
    let trees_in_viz = context.trees_in_viz_process();
    if trees_in_viz {
        context.set_client_frame_token(&mut metadata);
    } else {
        context.increment_and_set_frame_token(&mut metadata);
    }
    context.populate_viewport_fields(&mut metadata);
    context.register_presentation_callbacks(&mut metadata);
    context.populate_activity_fields(&mut metadata);
    context.copy_referenced_surfaces(&mut metadata);
    context.populate_page_scale_and_browser_controls(&mut metadata);
    context.populate_root_scroll_display_and_mobile(&mut metadata);
    context.populate_delegated_ink(&mut metadata);
    context.populate_capture_bounds(&mut metadata);
    if trees_in_viz {
        context.copy_client_tracked_element_rects(&mut metadata);
    } else {
        context.collect_tracked_element_rects(&mut metadata);
    }
    let screenshot = context.has_screenshot_destination();
    let mut consumed_screenshot = false;
    if screenshot {
        context.set_screenshot_destination(&mut metadata);
        if !context.trees_in_viz_client_process() {
            context.clear_screenshot_destination();
            consumed_screenshot = true;
        }
    }
    context.populate_draw_mode(&mut metadata);
    let stats = CompositorMetadataStats {
        client_frame_token: trees_in_viz,
        client_tracked_rects: trees_in_viz,
        included_screenshot_destination: screenshot,
        consumed_screenshot_destination: consumed_screenshot,
    };
    trace.set("client_frame_token", trees_in_viz as u8 as f64);
    trace.set("client_tracked_rects", trees_in_viz as u8 as f64);
    trace.set("screenshot_destination", screenshot as u8 as f64);
    trace.set(
        "consumed_screenshot_destination",
        consumed_screenshot as u8 as f64,
    );
    (metadata, stats)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TrackedElementStatus {
    #[default]
    None,
    Present,
    NeedsOcclusion,
}

pub trait RenderMetadataContext {
    type Metadata;

    fn new_metadata(&mut self) -> Self::Metadata;
    fn populate_base_fields(&mut self, metadata: &mut Self::Metadata);
    fn tracked_element_status(&self) -> TrackedElementStatus;
    fn collect_tracked_element_rects(
        &mut self,
        metadata: &mut Self::Metadata,
        needs_occlusion: bool,
    );
    fn has_previous_metadata(&self) -> bool;
    fn update_vertical_scroll_direction(&mut self, metadata: &mut Self::Metadata) -> bool;
    fn previous_metadata_requires_new_local_surface_id(
        &mut self,
        metadata: &Self::Metadata,
    ) -> bool;
    fn current_local_surface_id_is_valid(&mut self) -> bool;
    fn trees_in_viz_process(&self) -> bool;
    fn allocate_local_surface_id(&mut self);
    fn set_current_local_surface_id(&mut self, metadata: &mut Self::Metadata);
    fn set_primary_main_frame_item_sequence_number(&mut self, metadata: &mut Self::Metadata);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderMetadataStats {
    pub collected_tracked_elements: bool,
    pub tracked_elements_need_occlusion: bool,
    pub vertical_scroll_direction_changed: bool,
    pub requested_new_local_surface_id: bool,
    pub allocated_local_surface_id: bool,
    pub attached_local_surface_id: bool,
}

pub fn BuildRenderMetadata<C: RenderMetadataContext>(
    context: &mut C,
) -> (C::Metadata, RenderMetadataStats) {
    let mut trace =
        browser_tracing::span("compositor.frame", "Compositor.Frame.BuildRenderMetadata");
    let mut metadata = context.new_metadata();
    context.populate_base_fields(&mut metadata);
    let tracked_status = context.tracked_element_status();
    let collected_tracked_elements = tracked_status != TrackedElementStatus::None;
    let tracked_elements_need_occlusion = tracked_status == TrackedElementStatus::NeedsOcclusion;
    if collected_tracked_elements {
        context.collect_tracked_element_rects(&mut metadata, tracked_elements_need_occlusion);
    }

    let mut vertical_scroll_direction_changed = false;
    let mut requested_new_local_surface_id = false;
    if context.has_previous_metadata() {
        vertical_scroll_direction_changed = context.update_vertical_scroll_direction(&mut metadata);
        requested_new_local_surface_id =
            context.previous_metadata_requires_new_local_surface_id(&metadata);
    }

    let mut allocated_local_surface_id = false;
    let current_surface_is_valid = context.current_local_surface_id_is_valid();
    if current_surface_is_valid {
        if requested_new_local_surface_id && !context.trees_in_viz_process() {
            context.allocate_local_surface_id();
            allocated_local_surface_id = true;
        }
        context.set_current_local_surface_id(&mut metadata);
    }
    context.set_primary_main_frame_item_sequence_number(&mut metadata);

    let stats = RenderMetadataStats {
        collected_tracked_elements,
        tracked_elements_need_occlusion,
        vertical_scroll_direction_changed,
        requested_new_local_surface_id,
        allocated_local_surface_id,
        attached_local_surface_id: current_surface_is_valid,
    };
    trace.set(
        "tracked_elements",
        stats.collected_tracked_elements as u8 as f64,
    );
    trace.set(
        "vertical_scroll_direction_changed",
        stats.vertical_scroll_direction_changed as u8 as f64,
    );
    trace.set(
        "allocated_local_surface_id",
        stats.allocated_local_surface_id as u8 as f64,
    );
    (metadata, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Context {
        viz: bool,
        screenshot: bool,
        viz_client: bool,
        events: Vec<&'static str>,
    }

    macro_rules! metadata_event {
        ($name:ident, $value:literal) => {
            fn $name(&mut self, _: &mut ()) {
                self.events.push($value);
            }
        };
    }

    impl CompositorMetadataContext for Context {
        type Metadata = ();
        fn new_metadata(&mut self) -> () {
            self.events.push("new");
        }
        fn trees_in_viz_process(&self) -> bool {
            self.viz
        }
        metadata_event!(set_client_frame_token, "client-token");
        metadata_event!(increment_and_set_frame_token, "increment-token");
        metadata_event!(populate_viewport_fields, "viewport");
        metadata_event!(register_presentation_callbacks, "callbacks");
        metadata_event!(populate_activity_fields, "activity");
        metadata_event!(copy_referenced_surfaces, "surfaces");
        metadata_event!(populate_page_scale_and_browser_controls, "controls");
        metadata_event!(populate_root_scroll_display_and_mobile, "root");
        metadata_event!(populate_delegated_ink, "ink");
        metadata_event!(populate_capture_bounds, "capture");
        metadata_event!(copy_client_tracked_element_rects, "client-rects");
        metadata_event!(collect_tracked_element_rects, "collect-rects");
        fn has_screenshot_destination(&self) -> bool {
            self.screenshot
        }
        metadata_event!(set_screenshot_destination, "screenshot");
        fn trees_in_viz_client_process(&mut self) -> bool {
            self.events.push("viz-client");
            self.viz_client
        }
        fn clear_screenshot_destination(&mut self) {
            self.events.push("clear-screenshot");
        }
        metadata_event!(populate_draw_mode, "draw-mode");
    }

    #[test]
    fn local_path_increments_token_collects_rects_and_consumes_screenshot() {
        let mut context = Context {
            viz: false,
            screenshot: true,
            viz_client: false,
            events: vec![],
        };
        let (_, stats) = BuildCompositorMetadata(&mut context);
        assert_eq!(
            context.events,
            [
                "new",
                "increment-token",
                "viewport",
                "callbacks",
                "activity",
                "surfaces",
                "controls",
                "root",
                "ink",
                "capture",
                "collect-rects",
                "screenshot",
                "viz-client",
                "clear-screenshot",
                "draw-mode"
            ]
        );
        assert!(stats.consumed_screenshot_destination);
    }

    #[test]
    fn viz_path_uses_client_token_and_client_rects_without_screenshot_queries() {
        let mut context = Context {
            viz: true,
            screenshot: false,
            viz_client: false,
            events: vec![],
        };
        let (_, stats) = BuildCompositorMetadata(&mut context);
        assert_eq!(context.events[1], "client-token");
        assert!(context.events.contains(&"client-rects"));
        assert!(!context.events.contains(&"viz-client"));
        assert!(stats.client_frame_token);
    }

    struct RenderContext {
        tracked: TrackedElementStatus,
        previous: bool,
        request_surface: bool,
        surface_valid: bool,
        viz: bool,
        events: Vec<&'static str>,
    }

    impl RenderMetadataContext for RenderContext {
        type Metadata = ();
        fn new_metadata(&mut self) -> () {
            self.events.push("new");
        }
        fn populate_base_fields(&mut self, _: &mut ()) {
            self.events.push("base");
        }
        fn tracked_element_status(&self) -> TrackedElementStatus {
            self.tracked
        }
        fn collect_tracked_element_rects(&mut self, _: &mut (), occlusion: bool) {
            self.events.push(if occlusion {
                "tracked-occlusion"
            } else {
                "tracked"
            });
        }
        fn has_previous_metadata(&self) -> bool {
            self.previous
        }
        fn update_vertical_scroll_direction(&mut self, _: &mut ()) -> bool {
            self.events.push("direction");
            true
        }
        fn previous_metadata_requires_new_local_surface_id(&mut self, _: &()) -> bool {
            self.events.push("compare");
            self.request_surface
        }
        fn current_local_surface_id_is_valid(&mut self) -> bool {
            self.events.push("surface-valid");
            self.surface_valid
        }
        fn trees_in_viz_process(&self) -> bool {
            self.viz
        }
        fn allocate_local_surface_id(&mut self) {
            self.events.push("allocate");
        }
        fn set_current_local_surface_id(&mut self, _: &mut ()) {
            self.events.push("set-surface");
        }
        fn set_primary_main_frame_item_sequence_number(&mut self, _: &mut ()) {
            self.events.push("sequence");
        }
    }

    #[test]
    fn render_metadata_owns_tracked_previous_and_surface_order() {
        let mut context = RenderContext {
            tracked: TrackedElementStatus::NeedsOcclusion,
            previous: true,
            request_surface: true,
            surface_valid: true,
            viz: false,
            events: vec![],
        };
        let (_, stats) = BuildRenderMetadata(&mut context);
        assert_eq!(
            context.events,
            [
                "new",
                "base",
                "tracked-occlusion",
                "direction",
                "compare",
                "surface-valid",
                "allocate",
                "set-surface",
                "sequence"
            ]
        );
        assert!(stats.allocated_local_surface_id);
    }

    #[test]
    fn render_metadata_skips_previous_and_invalid_surface_work() {
        let mut context = RenderContext {
            tracked: TrackedElementStatus::None,
            previous: false,
            request_surface: true,
            surface_valid: false,
            viz: false,
            events: vec![],
        };
        let (_, stats) = BuildRenderMetadata(&mut context);
        assert_eq!(context.events, ["new", "base", "surface-valid", "sequence"]);
        assert!(!stats.requested_new_local_surface_id);
        assert!(!stats.attached_local_surface_id);
    }
}
