//! Render-pass reference validation and unreachable-pass pruning.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuadMaterial {
    SharedElement,
    RenderPass,
    Other,
}

pub trait RenderPassPruningContext {
    type PassId: Copy + Ord;
    type ResourceId: Copy + Ord;

    fn pass_count(&self) -> usize;
    fn pass_id(&self, pass: usize) -> Self::PassId;
    fn quad_count(&self, pass: usize) -> usize;
    fn quad_material(&self, pass: usize, quad: usize) -> QuadMaterial;
    fn quad_shared_resource_id(&self, pass: usize, quad: usize) -> Self::ResourceId;
    fn quad_render_pass_id(&self, pass: usize, quad: usize) -> Option<Self::PassId>;
    fn erase_quad(&mut self, pass: usize, quad: usize);
    fn pass_has_copy_requests(&self, pass: usize) -> bool;
    fn pass_has_subtree_capture(&self, pass: usize) -> bool;
    fn pass_has_filters(&self, pass: usize) -> bool;
    fn pass_has_backdrop_filters(&self, pass: usize) -> bool;
    fn pass_view_transition_resource_id(&self, pass: usize) -> Self::ResourceId;
    fn resource_id_is_valid(&self, id: Self::ResourceId) -> bool;
    fn has_view_transition_save_directive(&self) -> bool;
    fn remove_pass(&mut self, pass: usize);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderPassPruningStats {
    pub invalid_reference_quads_removed: usize,
    pub empty_passes_removed: usize,
    pub unreachable_passes_removed: usize,
    pub retained_copy_request_passes: usize,
    pub retained_view_transition_passes: usize,
}

pub fn RemoveUnreachableRenderPasses<C: RenderPassPruningContext>(
    context: &mut C,
) -> RenderPassPruningStats {
    let mut trace = browser_tracing::span(
        "compositor.draw",
        "Compositor.Draw.RemoveUnreachableRenderPasses",
    );
    debug_assert!(context.pass_count() >= 1);
    let mut stats = RenderPassPruningStats::default();
    let mut pass_exists = BTreeSet::new();
    let mut pass_references: BTreeMap<C::PassId, i32> = BTreeMap::new();
    let mut view_transition_quad_references = BTreeSet::new();

    let mut pass = 0;
    while pass < context.pass_count() {
        let mut quad = 0;
        while quad < context.quad_count(pass) {
            match context.quad_material(pass, quad) {
                QuadMaterial::SharedElement => {
                    view_transition_quad_references
                        .insert(context.quad_shared_resource_id(pass, quad));
                    quad += 1;
                }
                QuadMaterial::Other => quad += 1,
                QuadMaterial::RenderPass => {
                    let id = context
                        .quad_render_pass_id(pass, quad)
                        .expect("render-pass quad without pass id");
                    if pass_exists.contains(&id) {
                        *pass_references.entry(id).or_default() += 1;
                        quad += 1;
                    } else {
                        context.erase_quad(pass, quad);
                        stats.invalid_reference_quads_removed += 1;
                    }
                }
            }
        }
        if pass == context.pass_count() - 1 {
            break;
        }
        let resource = context.pass_view_transition_resource_id(pass);
        let removable = context.quad_count(pass) == 0
            && !context.pass_has_copy_requests(pass)
            && !context.pass_has_subtree_capture(pass)
            && !context.pass_has_filters(pass)
            && !context.pass_has_backdrop_filters(pass)
            && !context.resource_id_is_valid(resource);
        if removable {
            context.remove_pass(pass);
            stats.empty_passes_removed += 1;
            continue;
        }
        pass_exists.insert(context.pass_id(pass));
        pass += 1;
    }

    pass = 0;
    while pass < context.pass_count() - 1 {
        let index = context.pass_count() - 2 - pass;
        if context.pass_has_copy_requests(index) {
            stats.retained_copy_request_passes += 1;
            pass += 1;
            continue;
        }
        let id = context.pass_id(index);
        if *pass_references.entry(id).or_default() != 0 {
            pass += 1;
            continue;
        }
        let resource = context.pass_view_transition_resource_id(index);
        if context.resource_id_is_valid(resource)
            && (context.has_view_transition_save_directive()
                || view_transition_quad_references.contains(&resource))
        {
            stats.retained_view_transition_passes += 1;
            pass += 1;
            continue;
        }
        for quad in 0..context.quad_count(index) {
            if let Some(referenced) = context.quad_render_pass_id(index, quad) {
                *pass_references.entry(referenced).or_default() -= 1;
            }
        }
        context.remove_pass(index);
        stats.unreachable_passes_removed += 1;
        // Source decrements the reverse-loop counter before its increment, so
        // the same offset is revisited after erasure.
    }

    trace.set(
        "invalid_reference_quads_removed",
        stats.invalid_reference_quads_removed as f64,
    );
    trace.set("empty_passes_removed", stats.empty_passes_removed as f64);
    trace.set(
        "unreachable_passes_removed",
        stats.unreachable_passes_removed as f64,
    );
    trace.set(
        "retained_copy_request_passes",
        stats.retained_copy_request_passes as f64,
    );
    trace.set(
        "retained_view_transition_passes",
        stats.retained_view_transition_passes as f64,
    );
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Pass {
        id: u8,
        quads: Vec<(QuadMaterial, Option<u8>, u8)>,
        copy: bool,
        resource: u8,
    }
    struct Context {
        passes: Vec<Pass>,
        save: bool,
    }
    impl RenderPassPruningContext for Context {
        type PassId = u8;
        type ResourceId = u8;
        fn pass_count(&self) -> usize {
            self.passes.len()
        }
        fn pass_id(&self, p: usize) -> u8 {
            self.passes[p].id
        }
        fn quad_count(&self, p: usize) -> usize {
            self.passes[p].quads.len()
        }
        fn quad_material(&self, p: usize, q: usize) -> QuadMaterial {
            self.passes[p].quads[q].0
        }
        fn quad_shared_resource_id(&self, p: usize, q: usize) -> u8 {
            self.passes[p].quads[q].2
        }
        fn quad_render_pass_id(&self, p: usize, q: usize) -> Option<u8> {
            self.passes[p].quads[q].1
        }
        fn erase_quad(&mut self, p: usize, q: usize) {
            self.passes[p].quads.remove(q);
        }
        fn pass_has_copy_requests(&self, p: usize) -> bool {
            self.passes[p].copy
        }
        fn pass_has_subtree_capture(&self, _: usize) -> bool {
            false
        }
        fn pass_has_filters(&self, _: usize) -> bool {
            false
        }
        fn pass_has_backdrop_filters(&self, _: usize) -> bool {
            false
        }
        fn pass_view_transition_resource_id(&self, p: usize) -> u8 {
            self.passes[p].resource
        }
        fn resource_id_is_valid(&self, id: u8) -> bool {
            id != 0
        }
        fn has_view_transition_save_directive(&self) -> bool {
            self.save
        }
        fn remove_pass(&mut self, p: usize) {
            self.passes.remove(p);
        }
    }

    #[test]
    fn removes_forward_references_empty_and_unreachable_passes() {
        let mut context = Context {
            passes: vec![
                Pass {
                    id: 1,
                    quads: vec![(QuadMaterial::Other, None, 0)],
                    copy: false,
                    resource: 0,
                },
                Pass {
                    id: 2,
                    quads: vec![],
                    copy: false,
                    resource: 0,
                },
                Pass {
                    id: 3,
                    quads: vec![(QuadMaterial::Other, None, 0)],
                    copy: false,
                    resource: 0,
                },
                Pass {
                    id: 4,
                    quads: vec![(QuadMaterial::RenderPass, Some(9), 0)],
                    copy: false,
                    resource: 0,
                },
            ],
            save: false,
        };
        let stats = RemoveUnreachableRenderPasses(&mut context);
        assert_eq!(context.passes.len(), 1);
        assert_eq!(context.passes[0].id, 4);
        assert_eq!(stats.empty_passes_removed, 1);
        assert_eq!(stats.unreachable_passes_removed, 2);
        assert_eq!(stats.invalid_reference_quads_removed, 1);
    }

    #[test]
    fn copy_and_view_transition_passes_are_retained() {
        let mut context = Context {
            passes: vec![
                Pass {
                    id: 1,
                    quads: vec![],
                    copy: true,
                    resource: 0,
                },
                Pass {
                    id: 2,
                    quads: vec![],
                    copy: false,
                    resource: 7,
                },
                Pass {
                    id: 3,
                    quads: vec![],
                    copy: false,
                    resource: 0,
                },
            ],
            save: true,
        };
        let stats = RemoveUnreachableRenderPasses(&mut context);
        assert_eq!(context.passes.len(), 3);
        assert_eq!(stats.retained_copy_request_passes, 1);
        assert_eq!(stats.retained_view_transition_passes, 1);
    }
}
