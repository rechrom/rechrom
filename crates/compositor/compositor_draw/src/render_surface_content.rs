//! Bottom-up render-surface content propagation.
//!
//! Tree storage and geometry implementations remain adapter concerns. Draw
//! owns the traversal order and the decision of which surface contributes to
//! which target and view-transition output.

pub trait RenderSurfaceContentContext {
    type Resource: Copy;
    type ContentRect: Copy;

    fn is_root_surface(&self, id: i32) -> bool;
    fn set_root_content_rect_to_viewport(&mut self, id: i32);
    fn calculate_surface_content_rect(&mut self, id: i32);
    fn render_target(&self, id: i32) -> i32;
    fn target_is_surface_list_member(&self, target_id: i32) -> bool;
    fn accumulate_surface_into_target(&mut self, target_id: i32, contributor_id: i32);
    fn increment_target_contributors(&mut self, target_id: i32);
    fn view_transition_resource(&self, id: i32) -> Option<Self::Resource>;
    fn view_transition_content_rect(&self, id: i32, resource: Self::Resource) -> Self::ContentRect;
    fn record_view_transition_content_rect(
        &mut self,
        resource: Self::Resource,
        rect: Self::ContentRect,
    );
    fn update_originating_content_layer(
        &mut self,
        resource: Self::Resource,
        rect: Self::ContentRect,
    );
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderSurfaceContentStats {
    pub surfaces: usize,
    pub root_surfaces: usize,
    pub contributing_surfaces: usize,
    pub view_transition_surfaces: usize,
}

/// Propagates accumulated content bottom-up, matching the reverse surface-list
/// order required by target content bounds.
pub fn ComputeSurfaceContentRects<C: RenderSurfaceContentContext>(
    context: &mut C,
    surface_list: &[i32],
) -> RenderSurfaceContentStats {
    let mut trace = browser_tracing::span(
        "compositor.draw",
        "Compositor.Draw.ComputeSurfaceContentRects",
    );
    let mut stats = RenderSurfaceContentStats {
        surfaces: surface_list.len(),
        ..Default::default()
    };

    for &id in surface_list.iter().rev() {
        if context.is_root_surface(id) {
            stats.root_surfaces += 1;
            context.set_root_content_rect_to_viewport(id);
            continue;
        }

        stats.contributing_surfaces += 1;
        context.calculate_surface_content_rect(id);
        let target_id = context.render_target(id);
        debug_assert!(context.target_is_surface_list_member(target_id));
        context.accumulate_surface_into_target(target_id, id);
        context.increment_target_contributors(target_id);

        let Some(resource) = context.view_transition_resource(id) else {
            continue;
        };
        stats.view_transition_surfaces += 1;
        let rect = context.view_transition_content_rect(id, resource);
        context.record_view_transition_content_rect(resource, rect);
        context.update_originating_content_layer(resource, rect);
    }

    trace.set("surfaces", stats.surfaces as f64);
    trace.set("root_surfaces", stats.root_surfaces as f64);
    trace.set("contributing_surfaces", stats.contributing_surfaces as f64);
    trace.set(
        "view_transition_surfaces",
        stats.view_transition_surfaces as f64,
    );
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeContext {
        calls: Vec<String>,
    }

    impl RenderSurfaceContentContext for FakeContext {
        type Resource = i32;
        type ContentRect = i32;

        fn is_root_surface(&self, id: i32) -> bool {
            id == 1
        }
        fn set_root_content_rect_to_viewport(&mut self, id: i32) {
            self.calls.push(format!("root:{id}"));
        }
        fn calculate_surface_content_rect(&mut self, id: i32) {
            self.calls.push(format!("calculate:{id}"));
        }
        fn render_target(&self, _id: i32) -> i32 {
            1
        }
        fn target_is_surface_list_member(&self, _target_id: i32) -> bool {
            true
        }
        fn accumulate_surface_into_target(&mut self, target_id: i32, contributor_id: i32) {
            self.calls
                .push(format!("accumulate:{contributor_id}->{target_id}"));
        }
        fn increment_target_contributors(&mut self, target_id: i32) {
            self.calls.push(format!("increment:{target_id}"));
        }
        fn view_transition_resource(&self, id: i32) -> Option<Self::Resource> {
            (id == 3).then_some(30)
        }
        fn view_transition_content_rect(
            &self,
            id: i32,
            resource: Self::Resource,
        ) -> Self::ContentRect {
            id + resource
        }
        fn record_view_transition_content_rect(
            &mut self,
            resource: Self::Resource,
            rect: Self::ContentRect,
        ) {
            self.calls.push(format!("record:{resource}:{rect}"));
        }
        fn update_originating_content_layer(
            &mut self,
            resource: Self::Resource,
            rect: Self::ContentRect,
        ) {
            self.calls.push(format!("origin:{resource}:{rect}"));
        }
    }

    #[test]
    fn propagates_non_root_surfaces_bottom_up_before_setting_root_viewport() {
        let mut context = FakeContext::default();
        let stats = ComputeSurfaceContentRects(&mut context, &[1, 2, 3]);
        assert_eq!(
            context.calls,
            vec![
                "calculate:3",
                "accumulate:3->1",
                "increment:1",
                "record:30:33",
                "origin:30:33",
                "calculate:2",
                "accumulate:2->1",
                "increment:1",
                "root:1",
            ]
        );
        assert_eq!(stats.surfaces, 3);
        assert_eq!(stats.root_surfaces, 1);
        assert_eq!(stats.contributing_surfaces, 2);
        assert_eq!(stats.view_transition_surfaces, 1);
    }
}
