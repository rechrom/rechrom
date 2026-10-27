//! Effect-tree render-target topology updates.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParentTargetState {
    pub parent_id: i32,
    pub parent_has_render_surface: bool,
    pub parent_target_id: i32,
}

pub trait RenderTargetUpdateContext {
    type Layer: Copy;
    type Resource: Copy + Ord;

    fn root_effect_id(&self) -> i32;
    fn effect_node_count(&self) -> i32;
    fn effect_resource(&self, id: i32) -> Option<Self::Resource>;
    fn clear_view_transition_target(&mut self, id: i32);
    fn parent_target_state(&self, id: i32) -> ParentTargetState;
    fn set_target(&mut self, id: i32, target_id: i32);
    fn backdrop_filter_disallows_lcd_text(&self, id: i32) -> bool;
    fn set_lcd_text_disallowed_by_backdrop_filter(&mut self, id: i32, value: bool);
    fn target(&self, id: i32) -> i32;

    fn layer_count(&self) -> usize;
    fn layer_at(&self, index: usize) -> Self::Layer;
    fn layer_resource(&self, layer: Self::Layer) -> Option<Self::Resource>;
    fn layer_effect_id(&self, layer: Self::Layer) -> i32;
    fn effect_has_render_surface(&self, id: i32) -> bool;
    fn set_view_transition_target(&mut self, id: i32, target_id: i32);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderTargetUpdateStats {
    pub effect_nodes: usize,
    pub resources: usize,
    pub mapped_resource_layers: usize,
    pub backdrop_filter_nodes: usize,
    pub lcd_disallowed_nodes: usize,
}

pub fn UpdateRenderTargets<C: RenderTargetUpdateContext>(
    context: &mut C,
) -> RenderTargetUpdateStats {
    let mut trace = browser_tracing::span("compositor.tree", "Compositor.Tree.UpdateRenderTargets");
    let root = context.root_effect_id();
    let end = context.effect_node_count();
    let mut resources = BTreeMap::new();
    let mut last_backdrop_filter = None;
    let mut stats = RenderTargetUpdateStats {
        effect_nodes: end.saturating_sub(root) as usize,
        ..Default::default()
    };

    for id in root..end {
        if let Some(resource) = context.effect_resource(id) {
            assert!(resources.insert(resource, id).is_none());
        }
        context.clear_view_transition_target(id);
        let target = if id == root {
            root
        } else {
            let parent = context.parent_target_state(id);
            if parent.parent_has_render_surface {
                parent.parent_id
            } else {
                parent.parent_target_id
            }
        };
        context.set_target(id, target);
        if context.backdrop_filter_disallows_lcd_text(id) {
            last_backdrop_filter = Some(id);
            stats.backdrop_filter_nodes += 1;
        }
        context.set_lcd_text_disallowed_by_backdrop_filter(id, false);
    }

    stats.resources = resources.len();
    if !resources.is_empty() {
        for index in 0..context.layer_count() {
            let layer = context.layer_at(index);
            let Some(resource) = context.layer_resource(layer) else {
                continue;
            };
            let Some(&resource_node_id) = resources.get(&resource) else {
                continue;
            };
            let layer_node_id = context.layer_effect_id(layer);
            let target = if context.effect_has_render_surface(layer_node_id) {
                layer_node_id
            } else {
                context.target(layer_node_id)
            };
            context.set_view_transition_target(resource_node_id, target);
            stats.mapped_resource_layers += 1;
        }
    }

    if let Some(last) = last_backdrop_filter {
        let mut current_target = Some(context.target(last));
        for id in (root..last).rev() {
            let disallowed = current_target.is_some_and(|target| target <= id);
            context.set_lcd_text_disallowed_by_backdrop_filter(id, disallowed);
            stats.lcd_disallowed_nodes += usize::from(disallowed);
            if current_target == Some(id) {
                current_target = None;
            }
            if current_target.is_none() && context.backdrop_filter_disallows_lcd_text(id) {
                current_target = Some(context.target(id));
            }
        }
    }

    trace.set("effect_nodes", stats.effect_nodes as f64);
    trace.set("resources", stats.resources as f64);
    trace.set(
        "mapped_resource_layers",
        stats.mapped_resource_layers as f64,
    );
    trace.set("backdrop_filter_nodes", stats.backdrop_filter_nodes as f64);
    trace.set("lcd_disallowed_nodes", stats.lcd_disallowed_nodes as f64);
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct Node {
        parent: i32,
        target: i32,
        has_surface: bool,
        resource: Option<i32>,
        backdrop: bool,
        transition_target: Option<i32>,
        lcd_disallowed: bool,
    }

    struct Context {
        nodes: Vec<Node>,
        layers: Vec<(Option<i32>, i32)>,
    }

    impl RenderTargetUpdateContext for Context {
        type Layer = usize;
        type Resource = i32;
        fn root_effect_id(&self) -> i32 {
            1
        }
        fn effect_node_count(&self) -> i32 {
            self.nodes.len() as i32
        }
        fn effect_resource(&self, id: i32) -> Option<Self::Resource> {
            self.nodes[id as usize].resource
        }
        fn clear_view_transition_target(&mut self, id: i32) {
            self.nodes[id as usize].transition_target = None;
        }
        fn parent_target_state(&self, id: i32) -> ParentTargetState {
            let parent_id = self.nodes[id as usize].parent;
            let parent = &self.nodes[parent_id as usize];
            ParentTargetState {
                parent_id,
                parent_has_render_surface: parent.has_surface,
                parent_target_id: parent.target,
            }
        }
        fn set_target(&mut self, id: i32, target_id: i32) {
            self.nodes[id as usize].target = target_id;
        }
        fn backdrop_filter_disallows_lcd_text(&self, id: i32) -> bool {
            self.nodes[id as usize].backdrop
        }
        fn set_lcd_text_disallowed_by_backdrop_filter(&mut self, id: i32, value: bool) {
            self.nodes[id as usize].lcd_disallowed = value;
        }
        fn target(&self, id: i32) -> i32 {
            self.nodes[id as usize].target
        }
        fn layer_count(&self) -> usize {
            self.layers.len()
        }
        fn layer_at(&self, index: usize) -> Self::Layer {
            index
        }
        fn layer_resource(&self, layer: Self::Layer) -> Option<Self::Resource> {
            self.layers[layer].0
        }
        fn layer_effect_id(&self, layer: Self::Layer) -> i32 {
            self.layers[layer].1
        }
        fn effect_has_render_surface(&self, id: i32) -> bool {
            self.nodes[id as usize].has_surface
        }
        fn set_view_transition_target(&mut self, id: i32, target_id: i32) {
            self.nodes[id as usize].transition_target = Some(target_id);
        }
    }

    #[test]
    fn updates_targets_maps_resources_and_propagates_backdrop_lcd_state() {
        let mut context = Context {
            nodes: vec![
                Node {
                    parent: 0,
                    target: 0,
                    has_surface: false,
                    resource: None,
                    backdrop: false,
                    transition_target: None,
                    lcd_disallowed: false,
                },
                Node {
                    parent: 0,
                    target: 1,
                    has_surface: true,
                    resource: None,
                    backdrop: false,
                    transition_target: None,
                    lcd_disallowed: false,
                },
                Node {
                    parent: 1,
                    target: 0,
                    has_surface: false,
                    resource: Some(9),
                    backdrop: false,
                    transition_target: None,
                    lcd_disallowed: false,
                },
                Node {
                    parent: 2,
                    target: 0,
                    has_surface: false,
                    resource: None,
                    backdrop: true,
                    transition_target: None,
                    lcd_disallowed: false,
                },
            ],
            layers: vec![(Some(9), 3)],
        };
        let stats = UpdateRenderTargets(&mut context);
        assert_eq!(context.nodes[1].target, 1);
        assert_eq!(context.nodes[2].target, 1);
        assert_eq!(context.nodes[3].target, 1);
        assert_eq!(context.nodes[2].transition_target, Some(1));
        assert!(context.nodes[2].lcd_disallowed);
        assert!(context.nodes[1].lcd_disallowed);
        assert_eq!(stats.resources, 1);
        assert_eq!(stats.mapped_resource_layers, 1);
        assert_eq!(stats.backdrop_filter_nodes, 1);
        assert_eq!(stats.lcd_disallowed_nodes, 2);
    }
}
