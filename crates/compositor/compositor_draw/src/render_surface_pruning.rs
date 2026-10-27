//! Render-surface list pruning and contributor cleanup decisions.
//!
//! The retained effect tree stays with `compositor_tree`; this module owns the
//! ordered draw decision that turns its initial surface list into the list that
//! will actually produce render passes.

use crate::draw_property_policy::ShouldRemoveRenderSurface;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderSurfacePruneCandidate {
    pub id: i32,
    pub target_id: i32,
    pub is_root: bool,
    pub content_is_empty: bool,
    /// Populated only when the source decision would evaluate the filter.
    pub has_reference_filter: Option<bool>,
    pub target_is_surface_list_member: bool,
    pub copy_output_required: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderSurfacePruneStats {
    pub candidates: usize,
    pub kept_candidates: usize,
    pub temporarily_removed: usize,
    pub restored_to_output: usize,
    pub final_removed: usize,
    pub output_surfaces: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderSurfacePrunePlan {
    pub final_surfaces: Vec<i32>,
    /// Surfaces whose membership bit ends false. This intentionally includes
    /// a removed target later restored to the output list: Chromium writes the
    /// current surface's bit in that case, not the target's bit.
    pub nonmember_surfaces: Vec<i32>,
    /// Targets whose contributor count loses one removed child surface.
    pub surface_contributor_decrement_targets: Vec<i32>,
    pub stats: RenderSurfacePruneStats,
}

impl RenderSurfacePrunePlan {
    pub fn requires_layer_cleanup(&self) -> bool {
        self.stats.final_removed != 0
    }
}

/// Runs the ordered pruning state machine over an immutable snapshot. The
/// caller only applies this plan to the resident effect tree afterwards.
pub fn PlanRenderSurfacePruning(
    candidates: &[RenderSurfacePruneCandidate],
) -> RenderSurfacePrunePlan {
    let mut trace = browser_tracing::span("compositor.draw", "Compositor.Draw.PruneRenderSurfaces");
    let mut targets = BTreeMap::new();
    let mut removed_for_cleanup = BTreeSet::new();
    let mut nonmembers = BTreeSet::new();
    let mut final_surfaces = Vec::with_capacity(candidates.len());
    let mut stats = RenderSurfacePruneStats {
        candidates: candidates.len(),
        ..Default::default()
    };

    for candidate in candidates {
        targets.insert(candidate.id, candidate.target_id);
        let target_is_member =
            candidate.target_is_surface_list_member && !nonmembers.contains(&candidate.target_id);
        let should_remove = ShouldRemoveRenderSurface(
            candidate.is_root,
            candidate.content_is_empty,
            || candidate.has_reference_filter.unwrap_or(false),
            target_is_member,
            candidate.copy_output_required,
        );
        if should_remove {
            stats.temporarily_removed += 1;
            removed_for_cleanup.insert(candidate.id);
            nonmembers.insert(candidate.id);
            continue;
        }

        stats.kept_candidates += 1;
        let mut target = candidate.target_id;
        while removed_for_cleanup.remove(&target) {
            final_surfaces.push(target);
            stats.restored_to_output += 1;
            target = *targets
                .get(&target)
                .expect("a removed render target must have been visited first");
        }
        final_surfaces.push(candidate.id);
    }

    let surface_contributor_decrement_targets = removed_for_cleanup
        .iter()
        .map(|id| {
            *targets
                .get(id)
                .expect("a removed surface must retain its render target")
        })
        .collect::<Vec<_>>();
    stats.final_removed = removed_for_cleanup.len();
    stats.output_surfaces = final_surfaces.len();
    trace.set("candidates", stats.candidates as f64);
    trace.set("kept_candidates", stats.kept_candidates as f64);
    trace.set("temporarily_removed", stats.temporarily_removed as f64);
    trace.set("restored_to_output", stats.restored_to_output as f64);
    trace.set("final_removed", stats.final_removed as f64);
    trace.set("output_surfaces", stats.output_surfaces as f64);

    RenderSurfacePrunePlan {
        final_surfaces,
        nonmember_surfaces: nonmembers.into_iter().collect(),
        surface_contributor_decrement_targets,
        stats,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerContributorCandidate {
    pub layer_index: usize,
    pub target_id: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LayerContributorPrunePlan {
    pub dropped_layer_indices: Vec<usize>,
    pub contributor_decrement_targets: Vec<i32>,
}

/// Selects contributing layers that target surfaces whose membership bit was
/// cleared by the surface-pruning pass.
pub fn PlanLayerContributorPruning(
    nonmember_surfaces: &[i32],
    candidates: &[LayerContributorCandidate],
) -> LayerContributorPrunePlan {
    let mut trace =
        browser_tracing::span("compositor.draw", "Compositor.Draw.PruneLayerContributors");
    let nonmembers = nonmember_surfaces.iter().copied().collect::<BTreeSet<_>>();
    let mut plan = LayerContributorPrunePlan::default();
    for candidate in candidates {
        if nonmembers.contains(&candidate.target_id) {
            plan.dropped_layer_indices.push(candidate.layer_index);
            plan.contributor_decrement_targets.push(candidate.target_id);
        }
    }
    trace.set("candidate_layers", candidates.len() as f64);
    trace.set("dropped_layers", plan.dropped_layer_indices.len() as f64);
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(
        id: i32,
        target_id: i32,
        is_root: bool,
        content_is_empty: bool,
        has_reference_filter: Option<bool>,
        target_is_surface_list_member: bool,
        copy_output_required: bool,
    ) -> RenderSurfacePruneCandidate {
        RenderSurfacePruneCandidate {
            id,
            target_id,
            is_root,
            content_is_empty,
            has_reference_filter,
            target_is_surface_list_member,
            copy_output_required,
        }
    }

    #[test]
    fn copy_request_restores_removed_target_to_output_without_resetting_membership() {
        let plan = PlanRenderSurfacePruning(&[
            candidate(1, 1, true, false, None, true, false),
            candidate(2, 1, false, true, Some(false), true, false),
            candidate(3, 2, false, false, None, true, true),
        ]);
        assert_eq!(plan.final_surfaces, vec![1, 2, 3]);
        assert_eq!(plan.nonmember_surfaces, vec![2]);
        assert!(plan.surface_contributor_decrement_targets.is_empty());
        assert_eq!(plan.stats.temporarily_removed, 1);
        assert_eq!(plan.stats.restored_to_output, 1);
        assert_eq!(plan.stats.final_removed, 0);
        assert!(!plan.requires_layer_cleanup());
    }

    #[test]
    fn final_removal_decrements_its_target_contributor() {
        let plan = PlanRenderSurfacePruning(&[
            candidate(1, 1, true, false, None, true, false),
            candidate(2, 1, false, true, Some(false), true, false),
        ]);
        assert_eq!(plan.final_surfaces, vec![1]);
        assert_eq!(plan.nonmember_surfaces, vec![2]);
        assert_eq!(plan.surface_contributor_decrement_targets, vec![1]);
        assert!(plan.requires_layer_cleanup());
    }

    #[test]
    fn layer_cleanup_is_owned_by_the_same_nonmember_decision() {
        let plan = PlanLayerContributorPruning(
            &[2, 4],
            &[
                LayerContributorCandidate {
                    layer_index: 0,
                    target_id: 1,
                },
                LayerContributorCandidate {
                    layer_index: 3,
                    target_id: 2,
                },
                LayerContributorCandidate {
                    layer_index: 5,
                    target_id: 4,
                },
            ],
        );
        assert_eq!(plan.dropped_layer_indices, vec![3, 5]);
        assert_eq!(plan.contributor_decrement_targets, vec![2, 4]);
    }
}
