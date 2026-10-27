//! Initial render-surface list construction.
//!
//! This package owns traversal, inclusion, capture propagation, and layer
//! contribution decisions. The facade supplies thin access to resident layers
//! and property-tree surfaces through [`InitialRenderSurfaceContext`].

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceDescriptor {
    pub effect_id: i32,
    pub target_id: i32,
    pub view_transition_target_id: Option<i32>,
    pub has_valid_view_transition_resource: bool,
    pub capture_token_matches: bool,
    pub filter_moves_pixels: bool,
    pub contributes_to_drawn_surface: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceIdentity {
    pub effect_id: i32,
    pub is_list_member: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerEligibility {
    pub is_root: bool,
    pub skip_draw: bool,
    pub skip_invertibility: bool,
    pub will_change_transform: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerDrawCandidate<R> {
    pub target_id: i32,
    pub view_transition_resource: Option<R>,
    pub needs_update: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaptureSurfaceDescriptor {
    pub target_id: i32,
    pub has_valid_resource: bool,
    pub has_capture_contributions: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaptureChainSurface {
    pub effect_id: i32,
    pub target_id: i32,
    pub has_capture_contributions: bool,
}

pub trait InitialRenderSurfaceContext {
    type Layer: Copy;
    type Resource: Copy + Ord;

    fn effect_node_count(&self) -> i32;
    fn reset_surface_state(&mut self, id: i32);
    fn root_surface_id(&self) -> i32;
    fn surface_descriptor(&self, id: i32) -> SurfaceDescriptor;
    fn surface_identity(&self, id: i32) -> SurfaceIdentity;
    fn initialize_surface(&mut self, id: i32, is_root: bool, descriptor: SurfaceDescriptor);
    fn compute_surface_draw_properties(&mut self, id: i32);
    fn surface_copy_output_required(&self, id: i32) -> bool;
    fn allow_undamaged_nonroot_render_pass_to_skip(&self) -> bool;
    fn surface_is_unbounded(&self, id: i32) -> bool;
    fn nearest_occlusion_immune_ancestor(&self, id: i32) -> Option<i32>;
    fn set_nearest_occlusion_immune_ancestor(&mut self, id: i32, ancestor: Option<i32>);

    fn layer_count(&self) -> usize;
    fn layer_at(&self, index: usize) -> Self::Layer;
    fn prepare_layer(&mut self, layer: Self::Layer);
    fn layer_eligibility(&self, layer: Self::Layer) -> LayerEligibility;
    fn set_layer_raster_even_if_not_drawn(&mut self, layer: Self::Layer, value: bool);
    fn layer_draw_candidate(&self, layer: Self::Layer) -> LayerDrawCandidate<Self::Resource>;
    fn adjust_layer_for_pixel_alignment(&mut self, layer: Self::Layer);
    fn set_layer_contributes_to_drawn_surface(&mut self, layer: Self::Layer, value: bool);
    fn accumulate_layer_into_surface(&mut self, surface_id: i32, layer: Self::Layer);
    fn increment_surface_contributors(&mut self, surface_id: i32);
    fn resource_matches_capture_token(&self, resource: Self::Resource) -> bool;
    fn clear_originating_surface_content_rect(&mut self, layer: Self::Layer);

    fn capture_surface_descriptor(&self, id: i32) -> CaptureSurfaceDescriptor;
    fn capture_chain_surface(&self, id: i32) -> CaptureChainSurface;
    fn mark_surface_has_capture_contributions(&mut self, id: i32);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InitialRenderSurfaceStats {
    pub effect_nodes: usize,
    pub surfaces_added: usize,
    pub input_layers: usize,
    pub skipped_layers: usize,
    pub contributing_layers: usize,
    pub deferred_view_transition_layers: usize,
    pub capture_surfaces: usize,
    pub capture_links_marked: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitialRenderSurfaceResult<R: Ord, L> {
    pub surface_list: Vec<i32>,
    pub content_layers: BTreeMap<R, L>,
    pub stats: InitialRenderSurfaceStats,
}

fn AddSurface<C: InitialRenderSurfaceContext>(
    context: &mut C,
    id: i32,
    list: &mut Vec<i32>,
    captures: &mut Vec<i32>,
    stats: &mut InitialRenderSurfaceStats,
) {
    let descriptor = context.surface_descriptor(id);
    let is_root = descriptor.effect_id == context.root_surface_id();
    if let Some(view_transition_target_id) = descriptor.view_transition_target_id {
        assert!(!is_root);
        assert!(descriptor.has_valid_view_transition_resource);
        let target = context.surface_identity(view_transition_target_id);
        if !target.is_list_member {
            AddSurface(context, target.effect_id, list, captures, stats);
        }
    }
    if !is_root {
        let target = context.surface_identity(descriptor.target_id);
        if !target.is_list_member {
            AddSurface(context, target.effect_id, list, captures, stats);
        }
    }

    context.initialize_surface(id, is_root, descriptor);
    list.push(descriptor.effect_id);
    stats.surfaces_added += 1;
    if descriptor.capture_token_matches {
        captures.push(id);
    }
    context.compute_surface_draw_properties(id);

    let is_occlusion_immune = context.surface_copy_output_required(id)
        || descriptor.filter_moves_pixels
        || context.allow_undamaged_nonroot_render_pass_to_skip()
        || context.surface_is_unbounded(id);
    let nearest = if is_occlusion_immune {
        Some(id)
    } else if is_root {
        None
    } else {
        context.nearest_occlusion_immune_ancestor(descriptor.target_id)
    };
    context.set_nearest_occlusion_immune_ancestor(id, nearest);
}

fn PropagateCaptureContributions<C: InitialRenderSurfaceContext>(
    context: &mut C,
    captures: &[i32],
    root_id: i32,
    stats: &mut InitialRenderSurfaceStats,
) {
    for &id in captures {
        let capture = context.capture_surface_descriptor(id);
        assert!(capture.has_valid_resource);
        assert!(capture.has_capture_contributions);
        let surface_target = capture.target_id;
        let mut target_id = surface_target;
        loop {
            let target = context.capture_chain_surface(target_id);
            if target.effect_id == root_id || target.has_capture_contributions {
                break;
            }
            target_id = target.target_id;
        }
        let target = context.capture_chain_surface(target_id);
        if !target.has_capture_contributions {
            continue;
        }
        let mut intermediate_id = surface_target;
        loop {
            let intermediate = context.capture_chain_surface(intermediate_id);
            if intermediate.effect_id == target.effect_id {
                break;
            }
            context.mark_surface_has_capture_contributions(intermediate_id);
            stats.capture_links_marked += 1;
            intermediate_id = intermediate.target_id;
        }
    }
}

/// Builds the initial render-surface list and the deferred view-transition
/// content-layer map in one Draw-owned traversal.
pub fn ComputeInitialRenderSurfaceList<C: InitialRenderSurfaceContext>(
    context: &mut C,
) -> InitialRenderSurfaceResult<C::Resource, C::Layer> {
    let mut trace = browser_tracing::span(
        "compositor.draw",
        "Compositor.Draw.ComputeInitialRenderSurfaceList",
    );
    let mut stats = InitialRenderSurfaceStats {
        effect_nodes: context.effect_node_count() as usize,
        input_layers: context.layer_count(),
        ..Default::default()
    };
    let root_id = context.root_surface_id();
    for id in root_id..context.effect_node_count() {
        context.reset_surface_state(id);
    }

    let mut surface_list = Vec::new();
    let mut captures = Vec::new();
    let mut content_layers = BTreeMap::new();
    AddSurface(
        context,
        root_id,
        &mut surface_list,
        &mut captures,
        &mut stats,
    );

    for index in 0..context.layer_count() {
        let layer = context.layer_at(index);
        context.prepare_layer(layer);
        let eligibility = context.layer_eligibility(layer);
        let skip_layer =
            !eligibility.is_root && (eligibility.skip_draw || eligibility.skip_invertibility);
        context.set_layer_raster_even_if_not_drawn(
            layer,
            (eligibility.skip_invertibility && !eligibility.skip_draw)
                || eligibility.will_change_transform,
        );
        if skip_layer {
            stats.skipped_layers += 1;
            continue;
        }

        let candidate = context.layer_draw_candidate(layer);
        if !candidate.needs_update {
            continue;
        }
        let target = context.surface_identity(candidate.target_id);
        if !target.is_list_member {
            AddSurface(
                context,
                target.effect_id,
                &mut surface_list,
                &mut captures,
                &mut stats,
            );
        }
        context.adjust_layer_for_pixel_alignment(layer);
        context.set_layer_contributes_to_drawn_surface(layer, true);
        context.accumulate_layer_into_surface(candidate.target_id, layer);
        context.increment_surface_contributors(candidate.target_id);
        stats.contributing_layers += 1;

        let Some(resource) = candidate.view_transition_resource else {
            continue;
        };
        if context.resource_matches_capture_token(resource) {
            continue;
        }
        context.clear_originating_surface_content_rect(layer);
        content_layers.insert(resource, layer);
        stats.deferred_view_transition_layers += 1;
    }

    stats.capture_surfaces = captures.len();
    PropagateCaptureContributions(context, &captures, root_id, &mut stats);
    trace.set("effect_nodes", stats.effect_nodes as f64);
    trace.set("surfaces_added", stats.surfaces_added as f64);
    trace.set("input_layers", stats.input_layers as f64);
    trace.set("skipped_layers", stats.skipped_layers as f64);
    trace.set("contributing_layers", stats.contributing_layers as f64);
    trace.set(
        "deferred_view_transition_layers",
        stats.deferred_view_transition_layers as f64,
    );
    trace.set("capture_surfaces", stats.capture_surfaces as f64);
    trace.set("capture_links_marked", stats.capture_links_marked as f64);

    InitialRenderSurfaceResult {
        surface_list,
        content_layers,
        stats,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct FakeSurface {
        target: i32,
        member: bool,
        filter_moves_pixels: bool,
        copy_required: bool,
        nearest: Option<i32>,
        contributors: usize,
    }

    impl FakeSurface {
        fn new(target: i32) -> Self {
            Self {
                target,
                member: false,
                filter_moves_pixels: false,
                copy_required: false,
                nearest: None,
                contributors: 0,
            }
        }
    }

    #[derive(Clone, Debug)]
    struct FakeLayer {
        target: i32,
        resource: Option<i32>,
        eligibility: LayerEligibility,
        needs_update: bool,
        raster_even_if_not_drawn: bool,
        contributes: bool,
        origin_cleared: bool,
    }

    #[derive(Debug)]
    struct FakeContext {
        surfaces: BTreeMap<i32, FakeSurface>,
        layers: Vec<FakeLayer>,
        surface_add_order: Vec<i32>,
        accumulated_layers: Vec<(i32, usize)>,
    }

    impl InitialRenderSurfaceContext for FakeContext {
        type Layer = usize;
        type Resource = i32;

        fn effect_node_count(&self) -> i32 {
            4
        }
        fn reset_surface_state(&mut self, id: i32) {
            if let Some(surface) = self.surfaces.get_mut(&id) {
                surface.member = false;
                surface.contributors = 0;
            }
        }
        fn root_surface_id(&self) -> i32 {
            1
        }
        fn surface_descriptor(&self, id: i32) -> SurfaceDescriptor {
            let surface = &self.surfaces[&id];
            SurfaceDescriptor {
                effect_id: id,
                target_id: surface.target,
                view_transition_target_id: None,
                has_valid_view_transition_resource: false,
                capture_token_matches: false,
                filter_moves_pixels: surface.filter_moves_pixels,
                contributes_to_drawn_surface: id != 1,
            }
        }
        fn surface_identity(&self, id: i32) -> SurfaceIdentity {
            SurfaceIdentity {
                effect_id: id,
                is_list_member: self.surfaces[&id].member,
            }
        }
        fn initialize_surface(&mut self, id: i32, _is_root: bool, _descriptor: SurfaceDescriptor) {
            self.surfaces.get_mut(&id).unwrap().member = true;
            self.surface_add_order.push(id);
        }
        fn compute_surface_draw_properties(&mut self, _id: i32) {}
        fn surface_copy_output_required(&self, id: i32) -> bool {
            self.surfaces[&id].copy_required
        }
        fn allow_undamaged_nonroot_render_pass_to_skip(&self) -> bool {
            false
        }
        fn surface_is_unbounded(&self, _id: i32) -> bool {
            false
        }
        fn nearest_occlusion_immune_ancestor(&self, id: i32) -> Option<i32> {
            self.surfaces[&id].nearest
        }
        fn set_nearest_occlusion_immune_ancestor(&mut self, id: i32, ancestor: Option<i32>) {
            self.surfaces.get_mut(&id).unwrap().nearest = ancestor;
        }
        fn layer_count(&self) -> usize {
            self.layers.len()
        }
        fn layer_at(&self, index: usize) -> Self::Layer {
            index
        }
        fn prepare_layer(&mut self, layer: Self::Layer) {
            self.layers[layer].contributes = false;
            self.layers[layer].raster_even_if_not_drawn = false;
        }
        fn layer_eligibility(&self, layer: Self::Layer) -> LayerEligibility {
            self.layers[layer].eligibility
        }
        fn set_layer_raster_even_if_not_drawn(&mut self, layer: Self::Layer, value: bool) {
            self.layers[layer].raster_even_if_not_drawn = value;
        }
        fn layer_draw_candidate(&self, layer: Self::Layer) -> LayerDrawCandidate<Self::Resource> {
            LayerDrawCandidate {
                target_id: self.layers[layer].target,
                view_transition_resource: self.layers[layer].resource,
                needs_update: self.layers[layer].needs_update,
            }
        }
        fn adjust_layer_for_pixel_alignment(&mut self, _layer: Self::Layer) {}
        fn set_layer_contributes_to_drawn_surface(&mut self, layer: Self::Layer, value: bool) {
            self.layers[layer].contributes = value;
        }
        fn accumulate_layer_into_surface(&mut self, surface_id: i32, layer: Self::Layer) {
            self.accumulated_layers.push((surface_id, layer));
        }
        fn increment_surface_contributors(&mut self, surface_id: i32) {
            self.surfaces.get_mut(&surface_id).unwrap().contributors += 1;
        }
        fn resource_matches_capture_token(&self, _resource: Self::Resource) -> bool {
            false
        }
        fn clear_originating_surface_content_rect(&mut self, layer: Self::Layer) {
            self.layers[layer].origin_cleared = true;
        }
        fn capture_surface_descriptor(&self, id: i32) -> CaptureSurfaceDescriptor {
            CaptureSurfaceDescriptor {
                target_id: self.surfaces[&id].target,
                has_valid_resource: false,
                has_capture_contributions: false,
            }
        }
        fn capture_chain_surface(&self, id: i32) -> CaptureChainSurface {
            CaptureChainSurface {
                effect_id: id,
                target_id: self.surfaces[&id].target,
                has_capture_contributions: false,
            }
        }
        fn mark_surface_has_capture_contributions(&mut self, _id: i32) {
            unreachable!()
        }
    }

    fn eligibility(is_root: bool) -> LayerEligibility {
        LayerEligibility {
            is_root,
            skip_draw: false,
            skip_invertibility: false,
            will_change_transform: false,
        }
    }

    #[test]
    fn recursively_adds_targets_and_owns_layer_contribution_decisions() {
        let mut surface_two = FakeSurface::new(1);
        surface_two.filter_moves_pixels = true;
        let mut context = FakeContext {
            surfaces: BTreeMap::from([
                (1, FakeSurface::new(1)),
                (2, surface_two),
                (3, FakeSurface::new(2)),
            ]),
            layers: vec![
                FakeLayer {
                    target: 3,
                    resource: Some(7),
                    eligibility: eligibility(true),
                    needs_update: true,
                    raster_even_if_not_drawn: false,
                    contributes: false,
                    origin_cleared: false,
                },
                FakeLayer {
                    target: 1,
                    resource: None,
                    eligibility: LayerEligibility {
                        is_root: false,
                        skip_draw: true,
                        skip_invertibility: false,
                        will_change_transform: true,
                    },
                    needs_update: true,
                    raster_even_if_not_drawn: false,
                    contributes: false,
                    origin_cleared: false,
                },
            ],
            surface_add_order: Vec::new(),
            accumulated_layers: Vec::new(),
        };

        let result = ComputeInitialRenderSurfaceList(&mut context);
        assert_eq!(result.surface_list, vec![1, 2, 3]);
        assert_eq!(context.surface_add_order, vec![1, 2, 3]);
        assert_eq!(context.accumulated_layers, vec![(3, 0)]);
        assert_eq!(result.content_layers, BTreeMap::from([(7, 0)]));
        assert!(context.layers[0].contributes);
        assert!(context.layers[0].origin_cleared);
        assert!(!context.layers[1].contributes);
        assert!(context.layers[1].raster_even_if_not_drawn);
        assert_eq!(context.surfaces[&2].nearest, Some(2));
        assert_eq!(context.surfaces[&3].nearest, Some(2));
        assert_eq!(context.surfaces[&3].contributors, 1);
        assert_eq!(result.stats.surfaces_added, 3);
        assert_eq!(result.stats.skipped_layers, 1);
        assert_eq!(result.stats.contributing_layers, 1);
        assert_eq!(result.stats.deferred_view_transition_layers, 1);
    }
}
