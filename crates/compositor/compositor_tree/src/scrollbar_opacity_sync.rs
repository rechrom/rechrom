//! Active-to-pending effect-node opacity synchronization for scrollbars.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollbarOpacityCandidate<ElementId> {
    MissingSource,
    MissingTarget,
    Present {
        element_id: ElementId,
        source_opacity: f32,
        target_opacity: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarOpacityUpdate<ElementId> {
    pub element_id: ElementId,
    pub opacity: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollbarOpacitySyncPlan<ElementId> {
    pub updates: Vec<ScrollbarOpacityUpdate<ElementId>>,
    pub candidates: usize,
    pub missing_sources: usize,
    pub missing_targets: usize,
    pub unchanged: usize,
}

impl<ElementId> Default for ScrollbarOpacitySyncPlan<ElementId> {
    fn default() -> Self {
        Self {
            updates: Vec::new(),
            candidates: 0,
            missing_sources: 0,
            missing_targets: 0,
            unchanged: 0,
        }
    }
}

/// Determines the exact set of pending effect nodes that inherit active-tree
/// scrollbar opacity. The input iterator is intentionally lazy: resident node
/// lookup performed by the facade is charged to this Tree trace interval.
pub fn PlanScrollbarOpacitySync<ElementId: Copy>(
    candidates: impl IntoIterator<Item = ScrollbarOpacityCandidate<ElementId>>,
) -> ScrollbarOpacitySyncPlan<ElementId> {
    let mut trace =
        browser_tracing::span("compositor.tree", "Compositor.Tree.SyncScrollbarOpacities");
    let mut plan = ScrollbarOpacitySyncPlan::default();
    for candidate in candidates {
        plan.candidates += 1;
        match candidate {
            ScrollbarOpacityCandidate::MissingSource => plan.missing_sources += 1,
            ScrollbarOpacityCandidate::MissingTarget => plan.missing_targets += 1,
            ScrollbarOpacityCandidate::Present {
                element_id,
                source_opacity,
                target_opacity,
            } => {
                if source_opacity == target_opacity {
                    plan.unchanged += 1;
                } else {
                    plan.updates.push(ScrollbarOpacityUpdate {
                        element_id,
                        opacity: source_opacity,
                    });
                }
            }
        }
    }
    trace.set("candidates", plan.candidates as f64);
    trace.set("updates", plan.updates.len() as f64);
    trace.set("missing_sources", plan.missing_sources as f64);
    trace.set("missing_targets", plan.missing_targets as f64);
    trace.set("unchanged", plan.unchanged as f64);
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updates_only_changed_nodes_and_accounts_for_missing_nodes() {
        let plan = PlanScrollbarOpacitySync([
            ScrollbarOpacityCandidate::Present {
                element_id: 1,
                source_opacity: 0.25,
                target_opacity: 0.75,
            },
            ScrollbarOpacityCandidate::Present {
                element_id: 2,
                source_opacity: 0.5,
                target_opacity: 0.5,
            },
            ScrollbarOpacityCandidate::MissingSource,
            ScrollbarOpacityCandidate::MissingTarget,
        ]);
        assert_eq!(
            plan.updates,
            [ScrollbarOpacityUpdate {
                element_id: 1,
                opacity: 0.25,
            }]
        );
        assert_eq!(plan.candidates, 4);
        assert_eq!(plan.missing_sources, 1);
        assert_eq!(plan.missing_targets, 1);
        assert_eq!(plan.unchanged, 1);
    }
}
