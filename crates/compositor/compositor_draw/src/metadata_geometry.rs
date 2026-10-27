//! Draw-time geometry collection for frame metadata.
//!
//! Concrete layer/property-tree access is injected by the facade. This package
//! owns traversal order, metadata filtering, occluder accumulation and region
//! capture collection so their cost is attributed to draw rather than frame
//! submission.

pub trait TrackedElementGeometryContext {
    type Feature: Copy;
    type TrackedRect: Clone;
    type Occluder;
    type Output: Default;

    fn layer_count(&self) -> usize;
    fn tracked_rects<'a>(
        &'a self,
        layer_index: usize,
    ) -> Vec<(Self::Feature, &'a Self::TrackedRect)>;
    fn should_add_to_compositor_frame_metadata(&self, rect: &Self::TrackedRect) -> bool;
    fn should_exclude_fixed_and_sticky_occlusions(&self, rect: &Self::TrackedRect) -> bool;
    fn calculate_visible_bounds(
        &mut self,
        layer_index: usize,
        rect: &mut Self::TrackedRect,
        front_occluders: &[Self::Occluder],
    );
    fn append_tracked_rect(
        &mut self,
        output: &mut Self::Output,
        feature: Self::Feature,
        rect: Self::TrackedRect,
    );
    fn layer_occluder(&mut self, layer_index: usize) -> Option<Self::Occluder>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TrackedElementGeometryStats {
    pub layers: usize,
    pub candidates: usize,
    pub metadata_filtered: usize,
    pub appended: usize,
    pub occluders: usize,
}

pub fn CollectTrackedElementGeometry<C: TrackedElementGeometryContext>(
    context: &mut C,
    is_for_compositor_frame_metadata: bool,
    need_occlusion: bool,
) -> (C::Output, TrackedElementGeometryStats) {
    let mut trace = browser_tracing::span(
        "compositor.draw",
        "Compositor.Draw.CollectTrackedElementGeometry",
    );
    let mut output = C::Output::default();
    let mut front_occluders = Vec::new();
    let mut stats = TrackedElementGeometryStats {
        layers: context.layer_count(),
        ..Default::default()
    };

    for layer_index in 0..context.layer_count() {
        let selected = {
            let candidates = context.tracked_rects(layer_index);
            stats.candidates += candidates.len();
            let mut selected = Vec::new();
            for (feature, rect) in candidates {
                if context.should_add_to_compositor_frame_metadata(rect)
                    != is_for_compositor_frame_metadata
                {
                    stats.metadata_filtered += 1;
                    continue;
                }
                assert!(
                    !context.should_exclude_fixed_and_sticky_occlusions(rect) || need_occlusion
                );
                selected.push((feature, rect.clone()));
            }
            selected
        };
        for (feature, mut rect) in selected {
            context.calculate_visible_bounds(layer_index, &mut rect, &front_occluders);
            context.append_tracked_rect(&mut output, feature, rect);
            stats.appended += 1;
        }
        if need_occlusion {
            if let Some(occluder) = context.layer_occluder(layer_index) {
                front_occluders.push(occluder);
                stats.occluders += 1;
            }
        }
    }

    trace.set("layers", stats.layers as f64);
    trace.set("candidates", stats.candidates as f64);
    trace.set("metadata_filtered", stats.metadata_filtered as f64);
    trace.set("appended", stats.appended as f64);
    trace.set("occluders", stats.occluders as f64);
    (output, stats)
}

pub trait RegionCaptureGeometryContext {
    type CaptureId: Copy;
    type Rect: Copy;
    type Output: Default;

    fn layer_count(&self) -> usize;
    fn capture_bounds(&self, layer_index: usize) -> Vec<(Self::CaptureId, Self::Rect)>;
    fn project_and_clip(&mut self, layer_index: usize, rect: Self::Rect) -> Self::Rect;
    fn set_capture_bound(
        &mut self,
        output: &mut Self::Output,
        id: Self::CaptureId,
        rect: Self::Rect,
    );
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RegionCaptureGeometryStats {
    pub layers: usize,
    pub capture_bounds: usize,
}

pub fn CollectRegionCaptureGeometry<C: RegionCaptureGeometryContext>(
    context: &mut C,
) -> (C::Output, RegionCaptureGeometryStats) {
    let mut trace = browser_tracing::span(
        "compositor.draw",
        "Compositor.Draw.CollectRegionCaptureGeometry",
    );
    let mut output = C::Output::default();
    let mut stats = RegionCaptureGeometryStats {
        layers: context.layer_count(),
        ..Default::default()
    };
    for layer_index in 0..context.layer_count() {
        for (id, rect) in context.capture_bounds(layer_index) {
            let screen = context.project_and_clip(layer_index, rect);
            context.set_capture_bound(&mut output, id, screen);
            stats.capture_bounds += 1;
        }
    }
    trace.set("layers", stats.layers as f64);
    trace.set("capture_bounds", stats.capture_bounds as f64);
    (output, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Tracked {
        metadata: bool,
        exclude_occlusion: bool,
        visible: i32,
    }

    struct TrackedContext {
        layers: Vec<Vec<(u8, Tracked)>>,
        occluders: Vec<bool>,
    }
    impl TrackedElementGeometryContext for TrackedContext {
        type Feature = u8;
        type TrackedRect = Tracked;
        type Occluder = usize;
        type Output = Vec<(u8, i32)>;

        fn layer_count(&self) -> usize {
            self.layers.len()
        }
        fn tracked_rects<'a>(&'a self, layer: usize) -> Vec<(u8, &'a Tracked)> {
            self.layers[layer]
                .iter()
                .map(|(feature, rect)| (*feature, rect))
                .collect()
        }
        fn should_add_to_compositor_frame_metadata(&self, rect: &Tracked) -> bool {
            rect.metadata
        }
        fn should_exclude_fixed_and_sticky_occlusions(&self, rect: &Tracked) -> bool {
            rect.exclude_occlusion
        }
        fn calculate_visible_bounds(
            &mut self,
            _layer: usize,
            rect: &mut Tracked,
            occluders: &[usize],
        ) {
            rect.visible -= occluders.len() as i32;
        }
        fn append_tracked_rect(&mut self, output: &mut Vec<(u8, i32)>, feature: u8, rect: Tracked) {
            output.push((feature, rect.visible));
        }
        fn layer_occluder(&mut self, layer: usize) -> Option<usize> {
            self.occluders[layer].then_some(layer)
        }
    }

    #[test]
    fn tracked_geometry_filters_then_accumulates_front_occluders() {
        let mut context = TrackedContext {
            layers: vec![
                vec![
                    (
                        1,
                        Tracked {
                            metadata: true,
                            exclude_occlusion: true,
                            visible: 10,
                        },
                    ),
                    (
                        2,
                        Tracked {
                            metadata: false,
                            exclude_occlusion: false,
                            visible: 20,
                        },
                    ),
                ],
                vec![(
                    3,
                    Tracked {
                        metadata: true,
                        exclude_occlusion: true,
                        visible: 30,
                    },
                )],
            ],
            occluders: vec![true, false],
        };
        let (output, stats) = CollectTrackedElementGeometry(&mut context, true, true);
        assert_eq!(output, [(1, 10), (3, 29)]);
        assert_eq!(stats.candidates, 3);
        assert_eq!(stats.metadata_filtered, 1);
        assert_eq!(stats.occluders, 1);
    }

    struct CaptureContext(Vec<Vec<(u8, i32)>>);
    impl RegionCaptureGeometryContext for CaptureContext {
        type CaptureId = u8;
        type Rect = i32;
        type Output = Vec<(u8, i32)>;
        fn layer_count(&self) -> usize {
            self.0.len()
        }
        fn capture_bounds(&self, layer: usize) -> Vec<(u8, i32)> {
            self.0[layer].clone()
        }
        fn project_and_clip(&mut self, layer: usize, rect: i32) -> i32 {
            rect + layer as i32
        }
        fn set_capture_bound(&mut self, output: &mut Vec<(u8, i32)>, id: u8, rect: i32) {
            output.push((id, rect));
        }
    }

    #[test]
    fn capture_geometry_preserves_layer_and_bound_order() {
        let mut context = CaptureContext(vec![vec![(1, 10), (2, 20)], vec![(3, 30)]]);
        let (output, stats) = CollectRegionCaptureGeometry(&mut context);
        assert_eq!(output, [(1, 10), (2, 20), (3, 31)]);
        assert_eq!(stats.layers, 2);
        assert_eq!(stats.capture_bounds, 3);
    }
}
