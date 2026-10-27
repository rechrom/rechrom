//! Package-owned trace entry points for fragment work performed by native
//! assembly adapters. Keeping the complete phase vocabulary here prevents
//! assembly and commit code from relabelling fragment costs as their own.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TracePhase {
    ExportTree,
    ValidatePaintInput,
    PopulateLayoutResultPaintData,
    IndexObjects,
    RebuildBoundaryObjectIndex,
    UpdateStickyOffsets,
    BuildPaintCullRect,
    PopulateFragmentPaintData,
    ResolveLogicalParents,
    PopulateGlobalPaintData,
    PaintStyle,
    PaintInput,
    CertifyPaintInputs,
    BuildScrollIndex,
    PopulatePaintData,
    RefreshTargetedPaintData,
    RefreshTargetedPaintInputs,
    RefreshBoundaryCache,
    RefreshScrollPaintProperties,
    RefreshScrollPaintIndex,
    RefreshScrollCullData,
    UpdatePrePaintInputs,
    BuildScrollPaintIndex,
    ClientRects,
}

impl TracePhase {
    pub const fn category(self) -> &'static str {
        match self {
            Self::CertifyPaintInputs | Self::UpdatePrePaintInputs => "prepaint",
            _ => "layout",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::ExportTree => "layout_fragment.ExportTree",
            Self::ValidatePaintInput => "layout_fragment.ValidatePaintInput",
            Self::PopulateLayoutResultPaintData => "layout_fragment.PopulateLayoutResultPaintData",
            Self::IndexObjects => "layout_fragment.IndexObjects",
            Self::RebuildBoundaryObjectIndex => "layout_fragment.RebuildBoundaryObjectIndex",
            Self::UpdateStickyOffsets => "layout_fragment.UpdateStickyOffsets",
            Self::BuildPaintCullRect => "layout_fragment.BuildPaintCullRect",
            Self::PopulateFragmentPaintData => "layout_fragment.PopulateFragmentPaintData",
            Self::ResolveLogicalParents => "layout_fragment.ResolveLogicalParents",
            Self::PopulateGlobalPaintData => "layout_fragment.PopulateGlobalPaintData",
            Self::PaintStyle => "layout_fragment.PaintStyle",
            Self::PaintInput => "layout_fragment.PaintInput",
            Self::CertifyPaintInputs => "layout_fragment.CertifyPaintInputs",
            Self::BuildScrollIndex => "layout_fragment.BuildScrollIndex",
            Self::PopulatePaintData => "layout_fragment.PopulatePaintData",
            Self::RefreshTargetedPaintData => "layout_fragment.RefreshTargetedPaintData",
            Self::RefreshTargetedPaintInputs => "layout_fragment.RefreshTargetedPaintInputs",
            Self::RefreshBoundaryCache => "layout_fragment.RefreshBoundaryCache",
            Self::RefreshScrollPaintProperties => "layout_fragment.RefreshScrollPaintProperties",
            Self::RefreshScrollPaintIndex => "layout_fragment.RefreshScrollPaintIndex",
            Self::RefreshScrollCullData => "layout_fragment.RefreshScrollCullData",
            Self::UpdatePrePaintInputs => "layout_fragment.UpdatePrePaintInputs",
            Self::BuildScrollPaintIndex => "layout_fragment.BuildScrollPaintIndex",
            Self::ClientRects => "layout_fragment.ClientRects",
        }
    }
}

#[inline(never)]
pub fn span(phase: TracePhase) -> browser_tracing::Span {
    browser_tracing::span(phase.category(), phase.name())
}

/// Trace export of the canonical native fragment graph at publication time.
#[inline(never)]
pub fn export_tree() -> browser_tracing::Span {
    span(TracePhase::ExportTree)
}

/// Trace a native fragment traversal used to answer CSSOM client rectangles.
#[inline(never)]
pub fn client_rects() -> browser_tracing::Span {
    span(TracePhase::ClientRects)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fragment_phases_keep_package_identity_and_lifecycle_category() {
        let _trace_test = crate::TRACE_TEST_LOCK.lock().unwrap();
        let recording = browser_tracing::start(Default::default()).unwrap();
        drop(span(TracePhase::PopulateFragmentPaintData));
        drop(span(TracePhase::CertifyPaintInputs));
        browser_tracing::stop().unwrap();
        let snapshot = browser_tracing::snapshot(Some(recording)).unwrap();
        let export = snapshot
            .spans
            .iter()
            .find(|trace| trace.name == "layout_fragment.PopulateFragmentPaintData")
            .expect("fragment export trace");
        assert_eq!(export.category, "layout");
        let revision = snapshot
            .spans
            .iter()
            .find(|trace| trace.name == "layout_fragment.CertifyPaintInputs")
            .expect("fragment certification trace");
        assert_eq!(revision.category, "prepaint");
    }
}
