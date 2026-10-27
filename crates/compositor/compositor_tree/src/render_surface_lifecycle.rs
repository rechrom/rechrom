//! Stable-id reconciliation for retained effect-tree render surfaces.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderSurfaceReuse {
    /// Index after the old surfaces have been reordered by
    /// `sorted_old_original_indices`.
    pub sorted_old_index: usize,
    pub new_node_id: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderSurfaceCreate<StableId> {
    pub stable_id: StableId,
    pub new_node_id: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderSurfaceLifecyclePlan<StableId> {
    /// Permutation that puts the caller's old surface storage into stable-id
    /// order. This preserves the native algorithm's observable storage order.
    pub sorted_old_original_indices: Vec<usize>,
    pub reuse: Vec<RenderSurfaceReuse>,
    pub create: Vec<RenderSurfaceCreate<StableId>>,
    pub dropped_old_surfaces: usize,
    pub changed: bool,
}

/// Reconciles old resident surfaces with the surfaces required by the new
/// effect tree. Matching stable ids are reused; unmatched new ids are created;
/// unmatched old ids remain available for destruction by the adapter.
pub fn PlanRenderSurfaceLifecycle<StableId: Copy + Ord>(
    old_stable_ids: &[StableId],
    required_surfaces: &[(StableId, i32)],
) -> RenderSurfaceLifecyclePlan<StableId> {
    let mut trace = browser_tracing::span(
        "compositor.tree",
        "Compositor.Tree.PlanRenderSurfaceLifecycle",
    );
    let mut sorted_old: Vec<_> = old_stable_ids.iter().copied().enumerate().collect();
    sorted_old.sort_by_key(|(_, stable_id)| *stable_id);
    let mut sorted_required = required_surfaces.to_vec();
    sorted_required.sort();

    let mut reuse = Vec::new();
    let mut create = Vec::new();
    let mut dropped_old_surfaces = 0;
    let mut old_index = 0;
    let mut required_index = 0;
    let mut changed = false;

    while old_index < sorted_old.len() && required_index < sorted_required.len() {
        let old_id = sorted_old[old_index].1;
        let (required_id, new_node_id) = sorted_required[required_index];
        if old_id == required_id {
            reuse.push(RenderSurfaceReuse {
                sorted_old_index: old_index,
                new_node_id,
            });
            old_index += 1;
            required_index += 1;
        } else if required_id < old_id {
            changed = true;
            create.push(RenderSurfaceCreate {
                stable_id: required_id,
                new_node_id,
            });
            required_index += 1;
        } else {
            changed = true;
            dropped_old_surfaces += 1;
            old_index += 1;
        }
    }

    if old_index != sorted_old.len() || required_index != sorted_required.len() {
        changed = true;
    }
    dropped_old_surfaces += sorted_old.len() - old_index;
    create.extend(sorted_required[required_index..].iter().copied().map(
        |(stable_id, new_node_id)| RenderSurfaceCreate {
            stable_id,
            new_node_id,
        },
    ));

    let plan = RenderSurfaceLifecyclePlan {
        sorted_old_original_indices: sorted_old
            .into_iter()
            .map(|(original_index, _)| original_index)
            .collect(),
        reuse,
        create,
        dropped_old_surfaces,
        changed,
    };
    trace.set("old_surfaces", old_stable_ids.len() as f64);
    trace.set("required_surfaces", required_surfaces.len() as f64);
    trace.set("reused_surfaces", plan.reuse.len() as f64);
    trace.set("created_surfaces", plan.create.len() as f64);
    trace.set("dropped_old_surfaces", plan.dropped_old_surfaces as f64);
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconciles_in_stable_id_order() {
        let plan = PlanRenderSurfaceLifecycle(&[30, 10, 40], &[(20, 8), (30, 7), (10, 4)]);
        assert_eq!(plan.sorted_old_original_indices, vec![1, 0, 2]);
        assert_eq!(
            plan.reuse,
            vec![
                RenderSurfaceReuse {
                    sorted_old_index: 0,
                    new_node_id: 4,
                },
                RenderSurfaceReuse {
                    sorted_old_index: 1,
                    new_node_id: 7,
                },
            ]
        );
        assert_eq!(
            plan.create,
            vec![RenderSurfaceCreate {
                stable_id: 20,
                new_node_id: 8,
            }]
        );
        assert_eq!(plan.dropped_old_surfaces, 1);
        assert!(plan.changed);
    }

    #[test]
    fn unchanged_when_all_stable_ids_are_reused() {
        let plan = PlanRenderSurfaceLifecycle(&[2, 1], &[(1, 9), (2, 4)]);
        assert_eq!(plan.reuse.len(), 2);
        assert!(plan.create.is_empty());
        assert_eq!(plan.dropped_old_surfaces, 0);
        assert!(!plan.changed);
    }
}
