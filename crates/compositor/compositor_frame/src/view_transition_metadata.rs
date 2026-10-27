//! View-transition resource to render-pass metadata mapping.

use std::collections::BTreeMap;

pub trait ViewTransitionSurfaceList {
    type ResourceId: Copy + Ord;
    type RenderPassId: Copy;

    fn surface_count(&self) -> usize;
    fn surface_resource(&self, index: usize) -> Option<Self::ResourceId>;
    fn resource_matches_capture_token(&self, resource: Self::ResourceId) -> bool;
    fn render_pass_id(&self, index: usize) -> Self::RenderPassId;
    fn capture_render_pass_id(&self, index: usize) -> Self::RenderPassId;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewTransitionMetadataStats {
    pub surfaces: usize,
    pub mapped_resources: usize,
    pub capture_passes: usize,
    pub normal_passes: usize,
}

pub fn BuildViewTransitionElementMap<C: ViewTransitionSurfaceList>(
    surfaces: &C,
) -> (
    BTreeMap<C::ResourceId, C::RenderPassId>,
    ViewTransitionMetadataStats,
) {
    let mut trace = browser_tracing::span(
        "compositor.frame",
        "Compositor.Frame.MapViewTransitionRenderPasses",
    );
    let mut result = BTreeMap::new();
    let mut stats = ViewTransitionMetadataStats {
        surfaces: surfaces.surface_count(),
        ..Default::default()
    };
    for index in 0..surfaces.surface_count() {
        let Some(resource) = surfaces.surface_resource(index) else {
            continue;
        };
        let (pass, capture) = if surfaces.resource_matches_capture_token(resource) {
            (surfaces.capture_render_pass_id(index), true)
        } else {
            (surfaces.render_pass_id(index), false)
        };
        assert!(
            result.insert(resource, pass).is_none(),
            "cannot map a view-transition resource to two render passes"
        );
        stats.capture_passes += usize::from(capture);
        stats.normal_passes += usize::from(!capture);
    }
    stats.mapped_resources = result.len();
    trace.set("surfaces", stats.surfaces as f64);
    trace.set("mapped_resources", stats.mapped_resources as f64);
    trace.set("capture_passes", stats.capture_passes as f64);
    trace.set("normal_passes", stats.normal_passes as f64);
    (result, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Surfaces;

    impl ViewTransitionSurfaceList for Surfaces {
        type ResourceId = i32;
        type RenderPassId = i32;
        fn surface_count(&self) -> usize {
            3
        }
        fn surface_resource(&self, index: usize) -> Option<Self::ResourceId> {
            [None, Some(8), Some(9)][index]
        }
        fn resource_matches_capture_token(&self, resource: Self::ResourceId) -> bool {
            resource == 8
        }
        fn render_pass_id(&self, index: usize) -> Self::RenderPassId {
            100 + index as i32
        }
        fn capture_render_pass_id(&self, index: usize) -> Self::RenderPassId {
            200 + index as i32
        }
    }

    #[test]
    fn selects_capture_pass_only_for_matching_resources() {
        let (elements, stats) = BuildViewTransitionElementMap(&Surfaces);
        assert_eq!(elements, BTreeMap::from([(8, 201), (9, 102)]));
        assert_eq!(stats.surfaces, 3);
        assert_eq!(stats.mapped_resources, 2);
        assert_eq!(stats.capture_passes, 1);
        assert_eq!(stats.normal_passes, 1);
    }
}
