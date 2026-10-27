//! Render-surface to render-pass creation walk.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug)]
pub struct RenderSurfacePassInfo<ResourceId> {
    pub should_draw: bool,
    pub unbounded: bool,
    pub has_capture_contributions: bool,
    pub resource_id: Option<ResourceId>,
}

pub trait RenderSurfacePassBuildContext {
    type ResourceId: Copy + Ord;

    fn render_surface_count(&self) -> usize;
    fn surface_info(&mut self, index: usize) -> RenderSurfacePassInfo<Self::ResourceId>;
    fn create_capture_render_pass(&mut self, index: usize, unbounded: bool);
    fn create_render_pass(&mut self, index: usize, unbounded: bool);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderSurfacePassBuildResult<ResourceId: Ord> {
    pub known_resource_ids: BTreeSet<ResourceId>,
    pub surfaces_visited: usize,
    pub bounded_passes: usize,
    pub unbounded_passes: usize,
    pub capture_passes: usize,
    pub skipped_surfaces: usize,
}

pub fn BuildRenderSurfacePasses<C: RenderSurfacePassBuildContext>(
    context: &mut C,
) -> RenderSurfacePassBuildResult<C::ResourceId> {
    let mut trace = browser_tracing::span(
        "compositor.draw",
        "Compositor.Draw.BuildRenderSurfacePasses",
    );
    let count = context.render_surface_count();
    let mut result = RenderSurfacePassBuildResult {
        known_resource_ids: BTreeSet::new(),
        surfaces_visited: 0,
        bounded_passes: 0,
        unbounded_passes: 0,
        capture_passes: 0,
        skipped_surfaces: 0,
    };
    for reverse_offset in 0..count {
        let index = count - 1 - reverse_offset;
        let info = context.surface_info(index);
        result.surfaces_visited += 1;
        if info.should_draw {
            if info.has_capture_contributions {
                context.create_capture_render_pass(index, info.unbounded);
                result.capture_passes += 1;
            }
            context.create_render_pass(index, info.unbounded);
            if info.unbounded {
                result.unbounded_passes += 1;
            } else {
                result.bounded_passes += 1;
            }
        } else {
            result.skipped_surfaces += 1;
        }
        if let Some(resource_id) = info.resource_id {
            result.known_resource_ids.insert(resource_id);
        }
    }
    trace.set("surfaces_visited", result.surfaces_visited as f64);
    trace.set("bounded_passes", result.bounded_passes as f64);
    trace.set("unbounded_passes", result.unbounded_passes as f64);
    trace.set("capture_passes", result.capture_passes as f64);
    trace.set("skipped_surfaces", result.skipped_surfaces as f64);
    trace.set("known_resources", result.known_resource_ids.len() as f64);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Context {
        infos: Vec<RenderSurfacePassInfo<u8>>,
        events: Vec<String>,
    }

    impl RenderSurfacePassBuildContext for Context {
        type ResourceId = u8;
        fn render_surface_count(&self) -> usize {
            self.infos.len()
        }
        fn surface_info(&mut self, index: usize) -> RenderSurfacePassInfo<u8> {
            self.infos[index]
        }
        fn create_capture_render_pass(&mut self, index: usize, unbounded: bool) {
            self.events.push(format!("capture-{index}-{unbounded}"));
        }
        fn create_render_pass(&mut self, index: usize, unbounded: bool) {
            self.events.push(format!("pass-{index}-{unbounded}"));
        }
    }

    #[test]
    fn visits_back_to_front_and_routes_bounded_unbounded_and_capture_passes() {
        let mut context = Context {
            infos: vec![
                RenderSurfacePassInfo {
                    should_draw: true,
                    unbounded: false,
                    has_capture_contributions: false,
                    resource_id: Some(1),
                },
                RenderSurfacePassInfo {
                    should_draw: false,
                    unbounded: false,
                    has_capture_contributions: false,
                    resource_id: Some(2),
                },
                RenderSurfacePassInfo {
                    should_draw: true,
                    unbounded: true,
                    has_capture_contributions: true,
                    resource_id: None,
                },
            ],
            events: Vec::new(),
        };
        let result = BuildRenderSurfacePasses(&mut context);
        assert_eq!(
            context.events,
            ["capture-2-true", "pass-2-true", "pass-0-false"]
        );
        assert_eq!(result.known_resource_ids, BTreeSet::from([1, 2]));
        assert_eq!(result.bounded_passes, 1);
        assert_eq!(result.unbounded_passes, 1);
        assert_eq!(result.capture_passes, 1);
        assert_eq!(result.skipped_surfaces, 1);
    }
}
