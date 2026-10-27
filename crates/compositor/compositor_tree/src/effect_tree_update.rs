//! Ordered retained effect-tree updates.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EffectNodeUpdateResult {
    pub surface_contents_scale_changed: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EffectTreeUpdateStats {
    pub updated: bool,
    pub node_visits: usize,
    pub surface_contents_scale_invalidations: usize,
}

/// Runs the forward effect-tree update walk when the tree is dirty.
///
/// Parent state is consumed by child updates, so the package deliberately owns
/// the increasing-id traversal order. The caller only adapts a concrete node
/// update to the retained tree storage.
pub fn UpdateEffectTree(
    needs_update: bool,
    first_node_id: i32,
    node_count: i32,
    mut update_node: impl FnMut(i32) -> EffectNodeUpdateResult,
) -> EffectTreeUpdateStats {
    if !needs_update {
        return EffectTreeUpdateStats::default();
    }

    let mut trace = browser_tracing::span("compositor.tree", "Compositor.Tree.UpdateEffects");
    let mut stats = EffectTreeUpdateStats {
        updated: true,
        ..Default::default()
    };
    for id in first_node_id..node_count {
        let result = update_node(id);
        stats.node_visits += 1;
        stats.surface_contents_scale_invalidations +=
            usize::from(result.surface_contents_scale_changed);
    }
    trace.set("node_visits", stats.node_visits as f64);
    trace.set(
        "surface_contents_scale_invalidations",
        stats.surface_contents_scale_invalidations as f64,
    );
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_clean_tree_without_visiting_nodes() {
        let mut visits = 0;
        let stats = UpdateEffectTree(false, 1, 4, |_| {
            visits += 1;
            EffectNodeUpdateResult::default()
        });
        assert_eq!(visits, 0);
        assert_eq!(stats, EffectTreeUpdateStats::default());
    }

    #[test]
    fn visits_parent_before_child_and_counts_scale_invalidations() {
        let mut order = Vec::new();
        let stats = UpdateEffectTree(true, 1, 5, |id| {
            order.push(id);
            EffectNodeUpdateResult {
                surface_contents_scale_changed: id == 2 || id == 4,
            }
        });
        assert_eq!(order, vec![1, 2, 3, 4]);
        assert_eq!(stats.node_visits, 4);
        assert_eq!(stats.surface_contents_scale_invalidations, 2);
        assert!(stats.updated);
    }
}
