use std::sync::atomic::{AtomicU64, Ordering};

// Across engines and failed passes, certified revisions must never alias.
static NEXT_REVISION: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Default)]
pub struct PrePaintRevisionStats {
    pub nodes: usize,
    pub new_nodes: usize,
    pub own_invalidations: usize,
    pub descendant_invalidations: usize,
    pub needs_paint_property_update: usize,
    pub descendant_needs_paint_property_update: usize,
    pub should_check_for_paint_invalidation: usize,
    pub should_check_layout_for_paint_invalidation: usize,
    pub descendant_should_check_layout_for_paint_invalidation: usize,
    pub subtree_should_check_for_paint_invalidation: usize,
    pub identity_changes: usize,
    pub geometry_changes: usize,
    pub paint_changes: usize,
    pub resource_changes: usize,
    pub child_changes: usize,
}

pub struct RevisionUpdateTrace(browser_tracing::Span);

impl RevisionUpdateTrace {
    pub fn finish(mut self, stats: PrePaintRevisionStats) {
        self.0.set("nodes", stats.nodes as f64);
        self.0.set("new_nodes", stats.new_nodes as f64);
        self.0
            .set("own_invalidations", stats.own_invalidations as f64);
        self.0.set(
            "descendant_invalidations",
            stats.descendant_invalidations as f64,
        );
        self.0
            .set("identity_changes", stats.identity_changes as f64);
        self.0
            .set("geometry_changes", stats.geometry_changes as f64);
        self.0.set("paint_changes", stats.paint_changes as f64);
        self.0
            .set("resource_changes", stats.resource_changes as f64);
        browser_tracing::instant(
            "prepaint",
            "layout_fragment.RevisionReasonSummary",
            &[
                (
                    "needs_paint_property_update",
                    stats.needs_paint_property_update as f64,
                ),
                (
                    "descendant_needs_paint_property_update",
                    stats.descendant_needs_paint_property_update as f64,
                ),
                (
                    "should_check_for_paint_invalidation",
                    stats.should_check_for_paint_invalidation as f64,
                ),
                (
                    "should_check_layout_for_paint_invalidation",
                    stats.should_check_layout_for_paint_invalidation as f64,
                ),
                (
                    "descendant_should_check_layout",
                    stats.descendant_should_check_layout_for_paint_invalidation as f64,
                ),
                (
                    "subtree_should_check_for_paint",
                    stats.subtree_should_check_for_paint_invalidation as f64,
                ),
                ("child_changes", stats.child_changes as f64),
            ],
        );
    }
}

/// Start the package-owned interval for a complete native revision pass.
/// Keeping the trace name here makes the standalone package, not its temporary
/// assembly adapter, authoritative for performance attribution.
#[doc(hidden)]
#[inline(never)]
pub fn update_span() -> RevisionUpdateTrace {
    RevisionUpdateTrace(browser_tracing::span(
        "prepaint",
        "layout_fragment.RevisionUpdate",
    ))
}

/// Allocate one process-wide fragment revision. This is public only for the
/// native tracker adapter composed by `layoutng_assembly`.
#[doc(hidden)]
#[inline(never)]
pub fn next_revision() -> u64 {
    NEXT_REVISION
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |revision| {
            revision.checked_add(1)
        })
        .expect("pre-paint revision exhausted")
}

/// Minimal field access needed by fragment-local scroll invalidation.
/// Concrete native fragment data remains private to the assembly adapter.
pub trait FragmentRevisionNode {
    fn pre_paint_revision(&self) -> u64;
    fn set_pre_paint_revision(&mut self, revision: u64);
    fn pre_paint_subtree_revision(&self) -> u64;
    fn set_pre_paint_subtree_revision(&mut self, revision: u64);
}

/// Version an in-place native scroll/sticky/control change on an already
/// certified fragment. Zero remains unobserved and cannot be promoted into a
/// certified revision by an offset-only update.
#[inline(never)]
pub fn mark_scroll_property_change<N: FragmentRevisionNode>(node: &mut N, own_changed: bool) {
    let generation = next_revision();
    if own_changed && node.pre_paint_revision() != 0 {
        node.set_pre_paint_revision(generation);
    }
    if node.pre_paint_revision() != 0 && node.pre_paint_subtree_revision() != 0 {
        node.set_pre_paint_subtree_revision(generation);
    } else {
        node.set_pre_paint_subtree_revision(0);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        mark_scroll_property_change, update_span, FragmentRevisionNode, PrePaintRevisionStats,
    };

    #[derive(Default)]
    struct Node {
        own: u64,
        subtree: u64,
    }

    impl FragmentRevisionNode for Node {
        fn pre_paint_revision(&self) -> u64 {
            self.own
        }
        fn set_pre_paint_revision(&mut self, revision: u64) {
            self.own = revision;
        }
        fn pre_paint_subtree_revision(&self) -> u64 {
            self.subtree
        }
        fn set_pre_paint_subtree_revision(&mut self, revision: u64) {
            self.subtree = revision;
        }
    }

    #[test]
    fn own_change_versions_the_node_and_its_subtree() {
        let mut node = Node {
            own: u64::MAX,
            subtree: u64::MAX,
        };
        mark_scroll_property_change(&mut node, true);
        assert_ne!(node.own, u64::MAX);
        assert_eq!(node.subtree, node.own);
    }

    #[test]
    fn descendant_change_preserves_the_own_revision() {
        let mut node = Node {
            own: u64::MAX,
            subtree: u64::MAX,
        };
        mark_scroll_property_change(&mut node, false);
        assert_eq!(node.own, u64::MAX);
        assert_ne!(node.subtree, u64::MAX);
    }

    #[test]
    fn offset_only_update_does_not_certify_unobserved_input() {
        let mut node = Node::default();
        mark_scroll_property_change(&mut node, true);
        assert_eq!(node.own, 0);
        assert_eq!(node.subtree, 0);
    }

    #[test]
    fn traces_are_attributed_to_the_fragment_package() {
        let _trace_test = crate::TRACE_TEST_LOCK.lock().unwrap();
        let recording = browser_tracing::start(Default::default()).unwrap();
        update_span().finish(PrePaintRevisionStats {
            nodes: 3,
            child_changes: 2,
            ..Default::default()
        });
        drop(crate::trace::export_tree());
        drop(crate::trace::client_rects());
        browser_tracing::stop().unwrap();
        let snapshot = browser_tracing::snapshot(Some(recording)).unwrap();
        let trace = snapshot
            .spans
            .iter()
            .find(|trace| trace.name == "layout_fragment.RevisionUpdate")
            .expect("fragment revision trace");
        assert_eq!(trace.category, "prepaint");
        assert_eq!(trace.fields.get("nodes"), Some(&3.0));
        let reasons = snapshot
            .instants
            .iter()
            .find(|trace| trace.name == "layout_fragment.RevisionReasonSummary")
            .expect("fragment revision reason summary");
        assert_eq!(reasons.category, "prepaint");
        assert_eq!(reasons.fields.get("child_changes"), Some(&2.0));
        let export = snapshot
            .spans
            .iter()
            .find(|trace| trace.name == "layout_fragment.ExportTree")
            .expect("fragment export trace");
        assert_eq!(export.category, "layout");
        assert!(snapshot
            .spans
            .iter()
            .any(|trace| trace.name == "layout_fragment.ClientRects"));
    }
}
