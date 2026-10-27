//! Back-to-front compositor hit-test region construction.

use compositor_tree::geometry::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceLayerState {
    pub has_pointer_events_none: bool,
    pub surface_hit_testable: bool,
    pub surface_range_valid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HitTestRegionPlan {
    pub content_rect: Rect,
    pub ignore: bool,
    pub ask: bool,
    pub overlapped_region: bool,
    pub irregular_clip: bool,
}

pub trait HitTestListContext<L: Copy> {
    fn surface_layer_state(&self, layer: L) -> Option<SurfaceLayerState>;
    fn layer_hit_testable(&self, layer: L) -> bool;
    fn layer_draws_content(&self, layer: L) -> bool;
    fn add_layer_to_overlapping_region(&mut self, layer: L);
    fn overlaps_accumulated_region(&self, layer: L) -> bool;
    fn full_content_rect(&self, layer: L) -> Rect;
    fn visible_content_rect(&self, layer: L) -> Rect;
    fn layer_is_clipped(&self, layer: L) -> bool;
    fn hit_test_may_be_affected_by_mask(&self, layer: L) -> bool;
    fn screen_transform_preserves_2d_axis_alignment(&self, layer: L) -> bool;
    fn clipped_hit_test_region_is_rectangle(&self, layer: L) -> bool;
    fn emit_surface_region(&mut self, layer: L, plan: HitTestRegionPlan);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HitTestListStats {
    pub layers: usize,
    pub surface_layers: usize,
    pub emitted_regions: usize,
    pub skipped_surfaces: usize,
    pub overlapped_regions: usize,
    pub irregular_clips: usize,
    pub overlap_threshold_reached: bool,
}

pub fn BuildHitTestRegions<L: Copy, C: HitTestListContext<L>>(
    context: &mut C,
    layers_back_to_front: impl IntoIterator<Item = L>,
    assume_overlap_threshold: usize,
) -> HitTestListStats {
    let mut trace =
        browser_tracing::span("compositor.input", "Compositor.Input.BuildHitTestRegions");
    let mut stats = HitTestListStats::default();
    let mut iterated_non_surface_layers = 0usize;
    let mut assume_overlap = false;

    for layer in layers_back_to_front {
        stats.layers += 1;
        if let Some(surface) = context.surface_layer_state(layer) {
            stats.surface_layers += 1;
            if !context.layer_hit_testable(layer)
                && !context.layer_draws_content(layer)
                && !surface.has_pointer_events_none
            {
                stats.skipped_surfaces += 1;
                continue;
            }
            if !surface.surface_hit_testable || !surface.surface_range_valid {
                if !surface.has_pointer_events_none && !assume_overlap {
                    context.add_layer_to_overlapping_region(layer);
                }
                stats.skipped_surfaces += 1;
                continue;
            }

            let overlapped = assume_overlap || context.overlaps_accumulated_region(layer);
            let masked = context.hit_test_may_be_affected_by_mask(layer);
            let clipped = context.layer_is_clipped(layer);
            let irregular_clip = if clipped || masked {
                !(!masked
                    && context.screen_transform_preserves_2d_axis_alignment(layer)
                    && context.clipped_hit_test_region_is_rectangle(layer))
            } else {
                false
            };
            let plan = HitTestRegionPlan {
                content_rect: if clipped || masked {
                    context.visible_content_rect(layer)
                } else {
                    context.full_content_rect(layer)
                },
                ignore: surface.has_pointer_events_none,
                ask: overlapped || irregular_clip,
                overlapped_region: overlapped,
                irregular_clip,
            };
            stats.overlapped_regions += usize::from(overlapped);
            stats.irregular_clips += usize::from(irregular_clip);
            stats.emitted_regions += 1;
            context.emit_surface_region(layer, plan);
            continue;
        }

        if !context.layer_hit_testable(layer) {
            continue;
        }
        iterated_non_surface_layers += 1;
        if iterated_non_surface_layers > assume_overlap_threshold {
            assume_overlap = true;
            stats.overlap_threshold_reached = true;
        }
        if !assume_overlap {
            context.add_layer_to_overlapping_region(layer);
        }
    }

    trace.set("layers", stats.layers as f64);
    trace.set("surface_layers", stats.surface_layers as f64);
    trace.set("emitted_regions", stats.emitted_regions as f64);
    trace.set("skipped_surfaces", stats.skipped_surfaces as f64);
    trace.set("overlapped_regions", stats.overlapped_regions as f64);
    trace.set("irregular_clips", stats.irregular_clips as f64);
    trace.set(
        "overlap_threshold_reached",
        usize::from(stats.overlap_threshold_reached) as f64,
    );
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Context {
        emitted: Vec<(i32, HitTestRegionPlan)>,
        overlap_layers: Vec<i32>,
    }

    impl HitTestListContext<i32> for Context {
        fn surface_layer_state(&self, layer: i32) -> Option<SurfaceLayerState> {
            match layer {
                1 => Some(SurfaceLayerState {
                    has_pointer_events_none: false,
                    surface_hit_testable: true,
                    surface_range_valid: true,
                }),
                3 => Some(SurfaceLayerState {
                    has_pointer_events_none: false,
                    surface_hit_testable: false,
                    surface_range_valid: false,
                }),
                _ => None,
            }
        }
        fn layer_hit_testable(&self, _layer: i32) -> bool {
            true
        }
        fn layer_draws_content(&self, _layer: i32) -> bool {
            true
        }
        fn add_layer_to_overlapping_region(&mut self, layer: i32) {
            self.overlap_layers.push(layer);
        }
        fn overlaps_accumulated_region(&self, layer: i32) -> bool {
            layer == 1 && !self.overlap_layers.is_empty()
        }
        fn full_content_rect(&self, layer: i32) -> Rect {
            Rect::new(layer, 0, 10, 10)
        }
        fn visible_content_rect(&self, layer: i32) -> Rect {
            Rect::new(layer, 0, 5, 5)
        }
        fn layer_is_clipped(&self, layer: i32) -> bool {
            layer == 1
        }
        fn hit_test_may_be_affected_by_mask(&self, _layer: i32) -> bool {
            false
        }
        fn screen_transform_preserves_2d_axis_alignment(&self, _layer: i32) -> bool {
            false
        }
        fn clipped_hit_test_region_is_rectangle(&self, _layer: i32) -> bool {
            false
        }
        fn emit_surface_region(&mut self, layer: i32, plan: HitTestRegionPlan) {
            self.emitted.push((layer, plan));
        }
    }

    #[test]
    fn emits_surface_flags_after_accumulating_front_occluders() {
        let mut context = Context::default();
        let stats = BuildHitTestRegions(&mut context, [2, 1, 3], 100);
        assert_eq!(context.overlap_layers, vec![2, 3]);
        assert_eq!(context.emitted.len(), 1);
        assert!(context.emitted[0].1.ask);
        assert!(context.emitted[0].1.overlapped_region);
        assert!(context.emitted[0].1.irregular_clip);
        assert_eq!(context.emitted[0].1.content_rect.width, 5);
        assert_eq!(stats.emitted_regions, 1);
        assert_eq!(stats.skipped_surfaces, 1);
    }
}
