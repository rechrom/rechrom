//! Package-owned names for native invalidation adapter work.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TracePhase {
    CollectTreeFacts,
    EditingInvalidation,
    SelectionPaintInvalidation,
    CollectSubtree,
    PrepareSubtree,
    RetireObjects,
    Constraints,
    TreeUpdate,
    SubtreeUpdate,
    NodeInput,
    PrepareConstraints,
    PrepareTree,
    FinalizeRelations,
    RebindFonts,
    RefreshImageInputs,
    PrepareInlineFragments,
}

impl TracePhase {
    pub const fn name(self) -> &'static str {
        match self {
            Self::CollectTreeFacts => "layout_invalidation.CollectTreeFacts",
            Self::EditingInvalidation => "layout_invalidation.EditingInvalidation",
            Self::SelectionPaintInvalidation => "layout_invalidation.SelectionPaintInvalidation",
            Self::CollectSubtree => "layout_invalidation.CollectSubtree",
            Self::PrepareSubtree => "layout_invalidation.PrepareSubtree",
            Self::RetireObjects => "layout_invalidation.RetireObjects",
            Self::Constraints => "layout_invalidation.Constraints",
            Self::TreeUpdate => "layout_invalidation.TreeUpdate",
            Self::SubtreeUpdate => "layout_invalidation.SubtreeUpdate",
            Self::NodeInput => "layout_invalidation.NodeInput",
            Self::PrepareConstraints => "layout_invalidation.PrepareConstraints",
            Self::PrepareTree => "layout_invalidation.PrepareTree",
            Self::FinalizeRelations => "layout_invalidation.FinalizeRelations",
            Self::RebindFonts => "layout_invalidation.RebindFonts",
            Self::RefreshImageInputs => "layout_invalidation.RefreshImageInputs",
            Self::PrepareInlineFragments => "layout_invalidation.PrepareInlineFragments",
        }
    }
}

#[inline(never)]
pub fn span(phase: TracePhase) -> browser_tracing::Span {
    browser_tracing::span("layout", phase.name())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_phase_is_attributed_to_the_invalidation_package() {
        let recording = browser_tracing::start(Default::default()).unwrap();
        drop(span(TracePhase::PrepareTree));
        browser_tracing::stop().unwrap();
        let snapshot = browser_tracing::snapshot(Some(recording)).unwrap();
        let trace = snapshot
            .spans
            .iter()
            .find(|trace| trace.name == "layout_invalidation.PrepareTree")
            .expect("invalidation trace");
        assert_eq!(trace.category, "layout");
    }
}
