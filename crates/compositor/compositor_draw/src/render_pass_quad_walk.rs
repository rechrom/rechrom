//! Ordered layer/effect-tree walk that appends render-pass quads.
//!
//! The resident layer tree and Viz pass types stay behind the context trait,
//! while this package owns traversal, branching, aggregation, and performance
//! attribution for the hot quad-building loop.

use crate::append_quads_data::AppendQuadsData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderPassQuadWalkState {
    Layer,
    TargetSurface,
    ContributingSurface,
}

#[derive(Clone, Copy, Debug)]
pub struct RenderPassQuadWalkEntry<Surface, Layer> {
    pub state: RenderPassQuadWalkState,
    pub target_surface: Surface,
    pub current_surface: Option<Surface>,
    pub current_layer: Option<Layer>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrackedElementClass {
    #[default]
    None,
    Present,
    NeedsOcclusion,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderPassQuadWalkOptions {
    pub output_frame_data: bool,
    pub throttling_enabled: bool,
    pub ui_tree: bool,
}

pub trait RenderPassQuadWalkContext {
    type Surface: Copy + Eq;
    type Layer: Copy + Eq;
    type SurfaceId: Clone;

    fn current_entry(&mut self) -> Option<RenderPassQuadWalkEntry<Self::Surface, Self::Layer>>;
    fn advance(&mut self);

    fn target_has_copy_request(&mut self, target: Self::Surface) -> bool;
    fn take_target_copy_requests(&mut self, target: Self::Surface);
    fn process_target_for_throttling(&mut self, target: Self::Surface);

    fn contributing_surface_should_append(&mut self, surface: Self::Surface) -> bool;
    fn append_contributing_surface(
        &mut self,
        surface: Self::Surface,
        target: Self::Surface,
    ) -> AppendQuadsData<Self::SurfaceId>;

    fn classify_tracked_elements(&mut self, layer: Self::Layer) -> TrackedElementClass;
    fn layer_will_draw(&mut self, layer: Self::Layer) -> bool;
    fn record_will_draw_layer(&mut self, layer: Self::Layer);
    fn notify_layer_known_resources(&mut self, layer: Self::Layer);
    fn append_layer(
        &mut self,
        layer: Self::Layer,
        target: Self::Surface,
    ) -> AppendQuadsData<Self::SurfaceId>;
    fn layer_is_animating(&mut self, layer: Self::Layer) -> bool;
    fn layer_has_missing_tiles(&mut self, layer: Self::Layer) -> bool;
    fn layer_checkerboarded_needs_record(&mut self, layer: Self::Layer) -> bool;
    fn process_layer_not_to_draw(&mut self, layer: Self::Layer);
    fn record_visible_content(&mut self, visible_area: i64, approximated_visible_area: i64);
    fn accumulate_invalidated_area(&mut self, layer: Self::Layer);
}

#[derive(Clone, Debug)]
pub struct RenderPassQuadWalkResult<SurfaceId> {
    pub frame_data: AppendQuadsData<SurfaceId>,
    pub tracked_element_class: TrackedElementClass,
    pub have_missing_animated_tiles: bool,
    pub checkerboarded_needs_record: bool,
    pub positions: usize,
    pub layers: usize,
    pub target_surfaces: usize,
    pub contributing_surfaces: usize,
    pub copy_request_surfaces: usize,
    pub appended_layers: usize,
    pub skipped_layers: usize,
    pub appended_surfaces: usize,
}

impl<SurfaceId> Default for RenderPassQuadWalkResult<SurfaceId> {
    fn default() -> Self {
        Self {
            frame_data: AppendQuadsData::default(),
            tracked_element_class: TrackedElementClass::None,
            have_missing_animated_tiles: false,
            checkerboarded_needs_record: false,
            positions: 0,
            layers: 0,
            target_surfaces: 0,
            contributing_surfaces: 0,
            copy_request_surfaces: 0,
            appended_layers: 0,
            skipped_layers: 0,
            appended_surfaces: 0,
        }
    }
}

fn merge_frame_data<SurfaceId>(
    target: &mut AppendQuadsData<SurfaceId>,
    mut source: AppendQuadsData<SurfaceId>,
) {
    target
        .activation_dependencies
        .append(&mut source.activation_dependencies);
    if let Some(deadline) = source.deadline_in_frames {
        target.deadline_in_frames = Some(
            target
                .deadline_in_frames
                .map_or(deadline, |current| current.max(deadline)),
        );
    }
    target.use_default_lower_bound_deadline |= source.use_default_lower_bound_deadline;
    target.has_shared_element_resources |= source.has_shared_element_resources;
}

pub fn AppendRenderPassQuads<C: RenderPassQuadWalkContext>(
    context: &mut C,
    options: RenderPassQuadWalkOptions,
) -> RenderPassQuadWalkResult<C::SurfaceId> {
    let mut trace =
        browser_tracing::span("compositor.draw", "Compositor.Draw.AppendRenderPassQuads");
    let mut result = RenderPassQuadWalkResult::default();
    while let Some(entry) = context.current_entry() {
        result.positions += 1;
        let mut append_data = AppendQuadsData::default();
        match entry.state {
            RenderPassQuadWalkState::TargetSurface => {
                result.target_surfaces += 1;
                if options.output_frame_data
                    && context.target_has_copy_request(entry.target_surface)
                {
                    context.take_target_copy_requests(entry.target_surface);
                    result.copy_request_surfaces += 1;
                }
                if options.throttling_enabled {
                    context.process_target_for_throttling(entry.target_surface);
                }
            }
            RenderPassQuadWalkState::ContributingSurface => {
                result.contributing_surfaces += 1;
                if options.output_frame_data {
                    let surface = entry
                        .current_surface
                        .expect("contributing-surface walk entry");
                    if context.contributing_surface_should_append(surface) {
                        append_data =
                            context.append_contributing_surface(surface, entry.target_surface);
                        result.appended_surfaces += 1;
                    }
                }
            }
            RenderPassQuadWalkState::Layer => {
                result.layers += 1;
                let layer = entry.current_layer.expect("layer walk entry");
                if result.tracked_element_class != TrackedElementClass::NeedsOcclusion {
                    result.tracked_element_class = result
                        .tracked_element_class
                        .max(context.classify_tracked_elements(layer));
                }
                let will_draw = context.layer_will_draw(layer);
                if will_draw {
                    result.appended_layers += 1;
                    context.record_will_draw_layer(layer);
                    context.notify_layer_known_resources(layer);
                    if options.output_frame_data {
                        append_data = context.append_layer(layer, entry.target_surface);
                    }
                } else {
                    result.skipped_layers += 1;
                }

                let layer_animating = context.layer_is_animating(layer);
                if !options.output_frame_data
                    && will_draw
                    && context.layer_has_missing_tiles(layer)
                    && layer_animating
                {
                    result.have_missing_animated_tiles = true;
                }
                if context.layer_checkerboarded_needs_record(layer) {
                    result.checkerboarded_needs_record = true;
                }
                if !will_draw && options.throttling_enabled {
                    context.process_layer_not_to_draw(layer);
                }
                if options.output_frame_data {
                    context.record_visible_content(
                        append_data.visible_layer_area,
                        append_data.approximated_visible_content_area,
                    );
                    result.frame_data.num_missing_tiles += append_data.num_missing_tiles;
                    if append_data.num_missing_tiles > 0 && context.layer_is_animating(layer) {
                        result.have_missing_animated_tiles = true;
                    }
                }
                if options.ui_tree {
                    context.accumulate_invalidated_area(layer);
                }
            }
        }
        if options.output_frame_data {
            merge_frame_data(&mut result.frame_data, append_data);
        }
        context.advance();
    }
    trace.set("positions", result.positions as f64);
    trace.set("layers", result.layers as f64);
    trace.set("target_surfaces", result.target_surfaces as f64);
    trace.set("contributing_surfaces", result.contributing_surfaces as f64);
    trace.set("copy_request_surfaces", result.copy_request_surfaces as f64);
    trace.set("appended_layers", result.appended_layers as f64);
    trace.set("skipped_layers", result.skipped_layers as f64);
    trace.set("appended_surfaces", result.appended_surfaces as f64);
    trace.set("missing_tiles", result.frame_data.num_missing_tiles as f64);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    struct Context {
        entries: Vec<RenderPassQuadWalkEntry<i32, i32>>,
        index: usize,
        events: Vec<String>,
        layer_data: BTreeMap<i32, AppendQuadsData<i32>>,
    }

    impl RenderPassQuadWalkContext for Context {
        type Surface = i32;
        type Layer = i32;
        type SurfaceId = i32;

        fn current_entry(&mut self) -> Option<RenderPassQuadWalkEntry<i32, i32>> {
            self.entries.get(self.index).copied()
        }
        fn advance(&mut self) {
            self.events.push("advance".into());
            self.index += 1;
        }
        fn target_has_copy_request(&mut self, target: i32) -> bool {
            target == 1
        }
        fn take_target_copy_requests(&mut self, target: i32) {
            self.events.push(format!("copy-{target}"));
        }
        fn process_target_for_throttling(&mut self, target: i32) {
            self.events.push(format!("throttle-target-{target}"));
        }
        fn contributing_surface_should_append(&mut self, surface: i32) -> bool {
            surface == 2
        }
        fn append_contributing_surface(
            &mut self,
            surface: i32,
            target: i32,
        ) -> AppendQuadsData<i32> {
            self.events.push(format!("surface-{surface}-to-{target}"));
            AppendQuadsData {
                activation_dependencies: vec![20],
                deadline_in_frames: Some(2),
                ..Default::default()
            }
        }
        fn classify_tracked_elements(&mut self, layer: i32) -> TrackedElementClass {
            if layer == 4 {
                TrackedElementClass::NeedsOcclusion
            } else {
                TrackedElementClass::None
            }
        }
        fn layer_will_draw(&mut self, layer: i32) -> bool {
            layer == 4
        }
        fn record_will_draw_layer(&mut self, layer: i32) {
            self.events.push(format!("will-draw-{layer}"));
        }
        fn notify_layer_known_resources(&mut self, layer: i32) {
            self.events.push(format!("resources-{layer}"));
        }
        fn append_layer(&mut self, layer: i32, target: i32) -> AppendQuadsData<i32> {
            self.events.push(format!("layer-{layer}-to-{target}"));
            self.layer_data.remove(&layer).unwrap_or_default()
        }
        fn layer_is_animating(&mut self, layer: i32) -> bool {
            layer == 4
        }
        fn layer_has_missing_tiles(&mut self, layer: i32) -> bool {
            layer == 4
        }
        fn layer_checkerboarded_needs_record(&mut self, layer: i32) -> bool {
            layer == 4
        }
        fn process_layer_not_to_draw(&mut self, layer: i32) {
            self.events.push(format!("throttle-layer-{layer}"));
        }
        fn record_visible_content(&mut self, visible: i64, approximate: i64) {
            self.events.push(format!("visible-{visible}-{approximate}"));
        }
        fn accumulate_invalidated_area(&mut self, layer: i32) {
            self.events.push(format!("invalidated-{layer}"));
        }
    }

    #[test]
    fn owns_walk_branching_and_frame_metadata_aggregation() {
        let mut data = AppendQuadsData::default();
        data.num_missing_tiles = 3;
        data.visible_layer_area = 10;
        data.approximated_visible_content_area = 12;
        data.deadline_in_frames = Some(5);
        data.activation_dependencies = vec![40];
        data.has_shared_element_resources = true;
        let mut context = Context {
            entries: vec![
                RenderPassQuadWalkEntry {
                    state: RenderPassQuadWalkState::TargetSurface,
                    target_surface: 1,
                    current_surface: None,
                    current_layer: None,
                },
                RenderPassQuadWalkEntry {
                    state: RenderPassQuadWalkState::ContributingSurface,
                    target_surface: 1,
                    current_surface: Some(2),
                    current_layer: None,
                },
                RenderPassQuadWalkEntry {
                    state: RenderPassQuadWalkState::Layer,
                    target_surface: 1,
                    current_surface: None,
                    current_layer: Some(4),
                },
                RenderPassQuadWalkEntry {
                    state: RenderPassQuadWalkState::Layer,
                    target_surface: 1,
                    current_surface: None,
                    current_layer: Some(5),
                },
            ],
            index: 0,
            events: Vec::new(),
            layer_data: BTreeMap::from([(4, data)]),
        };
        let result = AppendRenderPassQuads(
            &mut context,
            RenderPassQuadWalkOptions {
                output_frame_data: true,
                throttling_enabled: true,
                ui_tree: true,
            },
        );
        assert_eq!(result.positions, 4);
        assert_eq!(result.appended_layers, 1);
        assert_eq!(result.skipped_layers, 1);
        assert_eq!(result.appended_surfaces, 1);
        assert_eq!(result.copy_request_surfaces, 1);
        assert_eq!(result.frame_data.num_missing_tiles, 3);
        assert_eq!(result.frame_data.deadline_in_frames, Some(5));
        assert_eq!(result.frame_data.activation_dependencies, [20, 40]);
        assert!(result.frame_data.has_shared_element_resources);
        assert_eq!(
            result.tracked_element_class,
            TrackedElementClass::NeedsOcclusion
        );
        assert!(result.checkerboarded_needs_record);
        assert!(result.have_missing_animated_tiles);
        assert!(context.events.contains(&"copy-1".into()));
        assert!(context.events.contains(&"throttle-layer-5".into()));
    }

    #[test]
    fn preserves_non_output_missing_animation_path_without_appending() {
        let mut context = Context {
            entries: vec![RenderPassQuadWalkEntry {
                state: RenderPassQuadWalkState::Layer,
                target_surface: 1,
                current_surface: None,
                current_layer: Some(4),
            }],
            index: 0,
            events: Vec::new(),
            layer_data: BTreeMap::new(),
        };
        let result = AppendRenderPassQuads(
            &mut context,
            RenderPassQuadWalkOptions {
                output_frame_data: false,
                ..Default::default()
            },
        );
        assert!(result.have_missing_animated_tiles);
        assert!(!context
            .events
            .iter()
            .any(|event| event.starts_with("layer-")));
    }
}
