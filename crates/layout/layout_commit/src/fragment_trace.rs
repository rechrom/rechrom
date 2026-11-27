//! Trace entry points for fragment work performed by base adapters
//! (fragment publication, Paint-sidecar refresh, client rects). Keeping the
//! complete phase vocabulary here prevents base and commit code from
//! relabelling fragment costs as their own.
//!
//! The names keep the historical `layout_fragment.` category prefix: it was
//! the fragment package's name before fragments moved into base, and recorded
//! traces, `devtools` owner attribution and `tracing::diagnostics` phase
//! mapping key on it.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TracePhase {
    PublishFragmentTree,
    ValidatePaintInput,
    InstallLayoutResultPaintState,
    IndexObjects,
    NewPaintState,
    InheritPaintState,
    SupersedePaintState,
    BeginPublication,
    EnsureStyleIndices,
    BuildPaintResources,
    UpdateDescendantDependentFlags,
    RecalcLayerVisualOverflow,
    Update3DTransformedDescendantStatus,
    InstallPublication,
    FinishPublication,
    ReleasePaintState,
    RebuildBoundaryObjectIndex,
    UpdateStickyOffsets,
    PaintStyle,
    PaintInput,
    PaintInputFontBinding,
    PaintInputInvalidation,
    PaintInputEnvelope,
    PaintInputNodeData,
    PaintInputSetStyle,
    BuildScrollIndex,
    PublishPaintData,
    RefreshTargetedPaintData,
    RefreshTargetedPaintInputs,
    RefreshRetainedPaintStyle,
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
            Self::UpdatePrePaintInputs => "prepaint",
            _ => "layout",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::PublishFragmentTree => "layout_fragment.PublishFragmentTree",
            Self::ValidatePaintInput => "layout_fragment.ValidatePaintInput",
            Self::InstallLayoutResultPaintState => "layout_fragment.InstallLayoutResultPaintState",
            Self::IndexObjects => "layout_fragment.IndexObjects",
            Self::NewPaintState => "layout_fragment.NewPaintState",
            Self::InheritPaintState => "layout_fragment.InheritPaintState",
            Self::SupersedePaintState => "layout_fragment.SupersedePaintState",
            Self::BeginPublication => "layout_fragment.BeginPublication",
            Self::EnsureStyleIndices => "layout_fragment.EnsureStyleIndices",
            Self::BuildPaintResources => "layout_fragment.BuildPaintResources",
            Self::UpdateDescendantDependentFlags => {
                "layout_fragment.UpdateDescendantDependentFlags"
            }
            Self::RecalcLayerVisualOverflow => "layout_fragment.RecalcLayerVisualOverflow",
            Self::Update3DTransformedDescendantStatus => {
                "layout_fragment.Update3DTransformedDescendantStatus"
            }
            Self::InstallPublication => "layout_fragment.InstallPublication",
            Self::FinishPublication => "layout_fragment.FinishPublication",
            Self::ReleasePaintState => "layout_fragment.ReleasePaintState",
            Self::RebuildBoundaryObjectIndex => "layout_fragment.RebuildBoundaryObjectIndex",
            Self::UpdateStickyOffsets => "layout_fragment.UpdateStickyOffsets",
            Self::PaintStyle => "layout_fragment.PaintStyle",
            Self::PaintInput => "layout_fragment.PaintInput",
            Self::PaintInputFontBinding => "layout_fragment.PaintInput.FontBinding",
            Self::PaintInputInvalidation => "layout_fragment.PaintInput.Invalidation",
            Self::PaintInputEnvelope => "layout_fragment.PaintInput.Envelope",
            Self::PaintInputNodeData => "layout_fragment.PaintInput.NodeData",
            Self::PaintInputSetStyle => "layout_fragment.PaintInput.SetStyle",
            Self::BuildScrollIndex => "layout_fragment.BuildScrollIndex",
            Self::PublishPaintData => "layout_fragment.PublishPaintData",
            Self::RefreshTargetedPaintData => "layout_fragment.RefreshTargetedPaintData",
            Self::RefreshTargetedPaintInputs => "layout_fragment.RefreshTargetedPaintInputs",
            Self::RefreshRetainedPaintStyle => {
                "layout_fragment.RefreshTargetedPaintInputs.RetainedPaintStyle"
            }
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

/// Trace publication of the canonical source fragment graph (no fragment walk).
#[inline(never)]
pub fn publish_fragment_tree() -> browser_tracing::Span {
    span(TracePhase::PublishFragmentTree)
}

/// Trace a source fragment traversal used to answer CSSOM client rectangles.
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
        drop(span(TracePhase::InstallLayoutResultPaintState));
        drop(span(TracePhase::UpdatePrePaintInputs));
        browser_tracing::stop().unwrap();
        let snapshot = browser_tracing::snapshot(Some(recording)).unwrap();
        let export = snapshot
            .spans
            .iter()
            .find(|trace| trace.name == "layout_fragment.InstallLayoutResultPaintState")
            .expect("paint state installation trace");
        assert_eq!(export.category, "layout");
        let revision = snapshot
            .spans
            .iter()
            .find(|trace| trace.name == "layout_fragment.UpdatePrePaintInputs")
            .expect("PrePaint input refresh trace");
        assert_eq!(revision.category, "prepaint");
    }
}
