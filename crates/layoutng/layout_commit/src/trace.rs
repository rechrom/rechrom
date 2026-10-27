//! Package-owned trace names for commit work and its native adapters.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TracePhase {
    CommitPaint,
    ValidateAuxiliaryClients,
    CommitNativeClients,
    CollectNativeClients,
    RefreshFragmentClients,
    CommitPrePaintInputs,
    PublishPaintResult,
    CreateEnvironment,
    CommitEnvironment,
    PublishLayoutResult,
    RunLayoutWithoutPublication,
}

impl TracePhase {
    pub const fn category(self) -> &'static str {
        "layout"
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::CommitPaint => "layout_commit.CommitPaint",
            Self::ValidateAuxiliaryClients => "layout_commit.ValidateAuxiliaryClients",
            Self::CommitNativeClients => "layout_commit.CommitNativeClients",
            Self::CollectNativeClients => "layout_commit.CollectNativeClients",
            Self::RefreshFragmentClients => "layout_commit.RefreshFragmentClients",
            Self::CommitPrePaintInputs => "layout_commit.CommitPrePaintInputs",
            Self::PublishPaintResult => "layout_commit.PublishPaintResult",
            Self::CreateEnvironment => "layout_commit.CreateEnvironment",
            Self::CommitEnvironment => "layout_commit.CommitEnvironment",
            Self::PublishLayoutResult => "layout_commit.PublishLayoutResult",
            Self::RunLayoutWithoutPublication => "layout_commit.RunLayoutWithoutPublication",
        }
    }
}

#[inline(never)]
pub fn span(phase: TracePhase) -> browser_tracing::Span {
    browser_tracing::span(phase.category(), phase.name())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phases_are_attributed_to_commit_with_the_expected_category() {
        let recording = browser_tracing::start(Default::default()).unwrap();
        drop(span(TracePhase::PublishLayoutResult));
        drop(span(TracePhase::CommitPrePaintInputs));
        drop(span(TracePhase::CollectNativeClients));
        drop(span(TracePhase::RefreshFragmentClients));
        drop(span(TracePhase::RunLayoutWithoutPublication));
        browser_tracing::stop().unwrap();
        let snapshot = browser_tracing::snapshot(Some(recording)).unwrap();
        let layout = snapshot
            .spans
            .iter()
            .find(|trace| trace.name == "layout_commit.PublishLayoutResult")
            .expect("layout commit trace");
        assert_eq!(layout.category, "layout");
        let commit = snapshot
            .spans
            .iter()
            .find(|trace| trace.name == "layout_commit.CommitPrePaintInputs")
            .expect("paint commit trace");
        assert_eq!(commit.category, "layout");
        assert!(snapshot
            .spans
            .iter()
            .any(|trace| trace.name == "layout_commit.CollectNativeClients"));
        assert!(snapshot
            .spans
            .iter()
            .any(|trace| trace.name == "layout_commit.RefreshFragmentClients"));
        assert!(snapshot
            .spans
            .iter()
            .any(|trace| trace.name == "layout_commit.RunLayoutWithoutPublication"));
    }
}
