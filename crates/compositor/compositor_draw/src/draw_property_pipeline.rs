//! Top-level draw-property computation pipeline.

use crate::draw_property_policy::DrawPropertyStats;

pub trait DrawPropertyPipelineContext {
    type Layer;

    fn input_layer_count(&self) -> usize;
    fn transform_node_count(&self) -> usize;
    fn effect_node_count(&self) -> usize;
    fn clip_node_count(&self) -> usize;

    fn clear_render_surface_list(&mut self);
    fn update_page_scale(&mut self);
    fn update_viewport_clip(&mut self);
    fn update_root_transform_and_scale(&mut self);
    fn update_property_trees_and_render_surfaces(&mut self);
    fn find_visible_layers(&mut self) -> Vec<Self::Layer>;
    fn compute_visible_layer_properties(&mut self, visible: &[Self::Layer]);
    fn calculate_render_surface_list(&mut self);
    fn validate_no_double_background_blur(&self) -> bool;
    fn record_render_surface_reasons(&self);
    fn render_surface_count(&self) -> usize;
    fn root_render_surface_exists(&self) -> bool;
}

/// Owns the cross-stage draw-property order and its package metrics. Concrete
/// geometry/property-tree operations remain borrowed adapter calls.
pub fn CalculateDrawProperties<C: DrawPropertyPipelineContext>(
    context: &mut C,
) -> (Vec<C::Layer>, DrawPropertyStats) {
    let mut trace = browser_tracing::span("compositor.draw", "Compositor.Draw.CalculateProperties");
    let mut stats = DrawPropertyStats {
        input_layers: context.input_layer_count(),
        transform_nodes: context.transform_node_count(),
        effect_nodes: context.effect_node_count(),
        clip_nodes: context.clip_node_count(),
        ..Default::default()
    };

    context.clear_render_surface_list();
    context.update_page_scale();
    context.update_viewport_clip();
    context.update_root_transform_and_scale();
    context.update_property_trees_and_render_surfaces();
    let visible = context.find_visible_layers();
    context.compute_visible_layer_properties(&visible);
    context.calculate_render_surface_list();
    #[cfg(debug_assertions)]
    debug_assert!(context.validate_no_double_background_blur());
    context.record_render_surface_reasons();

    stats.visible_layers = visible.len();
    stats.render_surfaces = context.render_surface_count();
    debug_assert!(context.root_render_surface_exists());
    trace.set("input_layers", stats.input_layers as f64);
    trace.set("visible_layers", stats.visible_layers as f64);
    trace.set("render_surfaces", stats.render_surfaces as f64);
    trace.set("transform_nodes", stats.transform_nodes as f64);
    trace.set("effect_nodes", stats.effect_nodes as f64);
    trace.set("clip_nodes", stats.clip_nodes as f64);
    (visible, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Context {
        events: Vec<&'static str>,
        surfaces: usize,
    }

    impl DrawPropertyPipelineContext for Context {
        type Layer = i32;
        fn input_layer_count(&self) -> usize {
            3
        }
        fn transform_node_count(&self) -> usize {
            4
        }
        fn effect_node_count(&self) -> usize {
            5
        }
        fn clip_node_count(&self) -> usize {
            6
        }
        fn clear_render_surface_list(&mut self) {
            self.events.push("clear");
        }
        fn update_page_scale(&mut self) {
            self.events.push("page-scale");
        }
        fn update_viewport_clip(&mut self) {
            self.events.push("viewport-clip");
        }
        fn update_root_transform_and_scale(&mut self) {
            self.events.push("root-transform");
        }
        fn update_property_trees_and_render_surfaces(&mut self) {
            self.events.push("property-trees");
        }
        fn find_visible_layers(&mut self) -> Vec<Self::Layer> {
            self.events.push("find-visible");
            vec![1, 2]
        }
        fn compute_visible_layer_properties(&mut self, _: &[Self::Layer]) {
            self.events.push("visible-properties");
        }
        fn calculate_render_surface_list(&mut self) {
            self.events.push("surface-list");
            self.surfaces = 2;
        }
        fn validate_no_double_background_blur(&self) -> bool {
            true
        }
        fn record_render_surface_reasons(&self) {}
        fn render_surface_count(&self) -> usize {
            self.surfaces
        }
        fn root_render_surface_exists(&self) -> bool {
            true
        }
    }

    #[test]
    fn owns_cross_stage_order_and_metrics() {
        let mut context = Context {
            events: Vec::new(),
            surfaces: 0,
        };
        let (visible, stats) = CalculateDrawProperties(&mut context);
        assert_eq!(visible, vec![1, 2]);
        assert_eq!(
            context.events,
            [
                "clear",
                "page-scale",
                "viewport-clip",
                "root-transform",
                "property-trees",
                "find-visible",
                "visible-properties",
                "surface-list",
            ]
        );
        assert_eq!(stats.input_layers, 3);
        assert_eq!(stats.visible_layers, 2);
        assert_eq!(stats.render_surfaces, 2);
        assert_eq!(stats.transform_nodes, 4);
        assert_eq!(stats.effect_nodes, 5);
        assert_eq!(stats.clip_nodes, 6);
    }
}
