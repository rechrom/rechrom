use std::{cell::Cell, time::Instant};

use crate::{BlockFormattingContextReuseReason, LayoutCacheDecisionReason, LayoutCacheStatus};

const SLOW_LOOKUP_THRESHOLD_NS: u64 = 50_000;

/// Trace the retained-tree admission check performed by the native adapter.
#[inline(never)]
pub fn retain_tree_span() -> browser_tracing::Span {
    browser_tracing::span("layout", "layout_reuse.RetainTree")
}

/// Aggregate cache work for one native layout pass. Per-node trace events are
/// intentionally avoided: they would flood the timeline and perturb the hot
/// path that this telemetry is meant to diagnose.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayoutReuseTraceStats {
    pub lookups: u64,
    pub hits: u64,
    pub needs_layout: u64,
    pub simplified_layouts: u64,
    pub line_reuses: u64,
    pub lookup_ns: u64,
    pub incompatible_spaces: u64,
    pub intrinsic_size_changes: u64,
    pub equal_size_hits: u64,
    pub stable_size_hits: u64,
    pub geometry_checks: u64,
    pub bfc_float_context_rejections: u64,
    pub bfc_margin_rejections: u64,
    pub bfc_descendant_rejections: u64,
    pub bfc_self_collapsing_clearance_rejections: u64,
    pub bfc_forced_offset_rejections: u64,
    pub bfc_exclusion_clearance_rejections: u64,
    pub bfc_hits: u64,
}

pub(crate) fn record_bfc_reuse_decision(reason: BlockFormattingContextReuseReason) {
    ACTIVE.with(|active| {
        let Some(mut stats) = active.get() else {
            return;
        };
        match reason {
            BlockFormattingContextReuseReason::PushedFloatMarginChanged
            | BlockFormattingContextReuseReason::PushedFloatOffsetChanged
            | BlockFormattingContextReuseReason::PushedFloatClearanceRegressed
            | BlockFormattingContextReuseReason::SelfCollapsingPushedByFloats => {
                stats.bfc_float_context_rejections =
                    stats.bfc_float_context_rejections.saturating_add(1)
            }
            BlockFormattingContextReuseReason::ModifiedMarginStrutChanged => {
                stats.bfc_margin_rejections = stats.bfc_margin_rejections.saturating_add(1)
            }
            BlockFormattingContextReuseReason::DescendantAboveBlockStartWithFloats => {
                stats.bfc_descendant_rejections = stats.bfc_descendant_rejections.saturating_add(1)
            }
            BlockFormattingContextReuseReason::SelfCollapsingClearance => {
                stats.bfc_self_collapsing_clearance_rejections = stats
                    .bfc_self_collapsing_clearance_rejections
                    .saturating_add(1)
            }
            BlockFormattingContextReuseReason::ForcedOffsetChanged => {
                stats.bfc_forced_offset_rejections =
                    stats.bfc_forced_offset_rejections.saturating_add(1)
            }
            BlockFormattingContextReuseReason::OldExclusionClearance
            | BlockFormattingContextReuseReason::NewExclusionClearance => {
                stats.bfc_exclusion_clearance_rejections =
                    stats.bfc_exclusion_clearance_rejections.saturating_add(1)
            }
            BlockFormattingContextReuseReason::Reusable => {
                stats.bfc_hits = stats.bfc_hits.saturating_add(1)
            }
        }
        active.set(Some(stats));
    });
}

thread_local! {
    static ACTIVE: Cell<Option<LayoutReuseTraceStats>> = const { Cell::new(None) };
}

pub(crate) fn record_cache_decision(reason: LayoutCacheDecisionReason) {
    ACTIVE.with(|active| {
        let Some(mut stats) = active.get() else {
            return;
        };
        match reason {
            LayoutCacheDecisionReason::IncompatibleSpace => {
                stats.incompatible_spaces = stats.incompatible_spaces.saturating_add(1)
            }
            LayoutCacheDecisionReason::IntrinsicSizeChanged => {
                stats.intrinsic_size_changes = stats.intrinsic_size_changes.saturating_add(1)
            }
            LayoutCacheDecisionReason::EqualSizes => {
                stats.equal_size_hits = stats.equal_size_hits.saturating_add(1)
            }
            LayoutCacheDecisionReason::StableSize => {
                stats.stable_size_hits = stats.stable_size_hits.saturating_add(1)
            }
            LayoutCacheDecisionReason::GeometryCheck => {
                stats.geometry_checks = stats.geometry_checks.saturating_add(1)
            }
        }
        active.set(Some(stats));
    });
}

pub struct LayoutReuseTraceSession {
    previous: Option<LayoutReuseTraceStats>,
    finished: bool,
}

impl LayoutReuseTraceSession {
    pub fn begin(enabled: bool) -> Self {
        let previous =
            ACTIVE.with(|active| active.replace(enabled.then(LayoutReuseTraceStats::default)));
        Self {
            previous,
            finished: false,
        }
    }

    pub fn finish(mut self) -> LayoutReuseTraceStats {
        let stats = ACTIVE.with(|active| active.replace(self.previous).unwrap_or_default());
        self.finished = true;
        browser_tracing::instant(
            "layout",
            "layout_reuse.PassSummary",
            &[
                ("lookups", stats.lookups as f64),
                ("hits", stats.hits as f64),
                ("needs_layout", stats.needs_layout as f64),
                ("simplified", stats.simplified_layouts as f64),
                ("line_reuse", stats.line_reuses as f64),
                ("lookup_us", stats.lookup_ns as f64 / 1_000.0),
            ],
        );
        browser_tracing::instant(
            "layout",
            "layout_reuse.DecisionSummary",
            &[
                ("incompatible_spaces", stats.incompatible_spaces as f64),
                (
                    "intrinsic_size_changes",
                    stats.intrinsic_size_changes as f64,
                ),
                ("equal_size_hits", stats.equal_size_hits as f64),
                ("stable_size_hits", stats.stable_size_hits as f64),
                ("geometry_checks", stats.geometry_checks as f64),
            ],
        );
        browser_tracing::instant(
            "layout",
            "layout_reuse.BfcDecisionSummary",
            &[
                (
                    "float_context_rejections",
                    stats.bfc_float_context_rejections as f64,
                ),
                ("margin_rejections", stats.bfc_margin_rejections as f64),
                (
                    "descendant_rejections",
                    stats.bfc_descendant_rejections as f64,
                ),
                (
                    "self_collapse_clearance_rejections",
                    stats.bfc_self_collapsing_clearance_rejections as f64,
                ),
                (
                    "forced_offset_rejections",
                    stats.bfc_forced_offset_rejections as f64,
                ),
                (
                    "exclusion_clearance_rejections",
                    stats.bfc_exclusion_clearance_rejections as f64,
                ),
                ("hits", stats.bfc_hits as f64),
            ],
        );
        stats
    }
}

impl Drop for LayoutReuseTraceSession {
    fn drop(&mut self) {
        if !self.finished {
            ACTIVE.with(|active| active.set(self.previous));
        }
    }
}

pub struct LayoutReuseLookupTrace {
    started: Option<Instant>,
}

impl LayoutReuseLookupTrace {
    pub fn begin() -> Self {
        Self {
            started: ACTIVE.with(|active| active.get().map(|_| Instant::now())),
        }
    }

    pub fn finish(self, status: LayoutCacheStatus) {
        let Some(started) = self.started else {
            return;
        };
        let ended = Instant::now();
        let elapsed = ended
            .duration_since(started)
            .as_nanos()
            .min(u64::MAX as u128) as u64;
        if elapsed >= SLOW_LOOKUP_THRESHOLD_NS {
            browser_tracing::interval(
                "layout",
                "layout_reuse.SlowLookup",
                started,
                ended,
                &[("status", status as i32 as f64)],
            );
        }
        ACTIVE.with(|active| {
            let Some(mut stats) = active.get() else {
                return;
            };
            stats.lookups = stats.lookups.saturating_add(1);
            stats.lookup_ns = stats.lookup_ns.saturating_add(elapsed);
            match status {
                LayoutCacheStatus::kHit => stats.hits = stats.hits.saturating_add(1),
                LayoutCacheStatus::kNeedsLayout => {
                    stats.needs_layout = stats.needs_layout.saturating_add(1)
                }
                LayoutCacheStatus::kNeedsSimplifiedLayout => {
                    stats.simplified_layouts = stats.simplified_layouts.saturating_add(1)
                }
                LayoutCacheStatus::kCanReuseLines => {
                    stats.line_reuses = stats.line_reuses.saturating_add(1)
                }
            }
            active.set(Some(stats));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_lookup_outcomes_without_per_node_events() {
        let recording = browser_tracing::start(Default::default()).unwrap();
        drop(retain_tree_span());
        let session = LayoutReuseTraceSession::begin(true);
        record_cache_decision(LayoutCacheDecisionReason::EqualSizes);
        record_cache_decision(LayoutCacheDecisionReason::GeometryCheck);
        record_bfc_reuse_decision(BlockFormattingContextReuseReason::ForcedOffsetChanged);
        record_bfc_reuse_decision(BlockFormattingContextReuseReason::Reusable);
        LayoutReuseLookupTrace::begin().finish(LayoutCacheStatus::kHit);
        LayoutReuseLookupTrace::begin().finish(LayoutCacheStatus::kNeedsLayout);
        let stats = session.finish();
        browser_tracing::stop().unwrap();

        assert_eq!(stats.lookups, 2);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.needs_layout, 1);
        assert_eq!(stats.equal_size_hits, 1);
        assert_eq!(stats.geometry_checks, 1);
        assert_eq!(stats.bfc_forced_offset_rejections, 1);
        assert_eq!(stats.bfc_hits, 1);

        let snapshot = browser_tracing::snapshot(Some(recording)).unwrap();
        let retained = snapshot
            .spans
            .iter()
            .find(|event| event.name == "layout_reuse.RetainTree")
            .expect("reuse retain-tree trace");
        assert_eq!(retained.category, "layout");
        let pass = snapshot
            .instants
            .iter()
            .find(|event| event.name == "layout_reuse.PassSummary")
            .expect("reuse pass summary");
        assert_eq!(pass.fields.get("lookups"), Some(&2.0));
        let decisions = snapshot
            .instants
            .iter()
            .find(|event| event.name == "layout_reuse.DecisionSummary")
            .expect("reuse decision summary");
        assert_eq!(decisions.fields.get("equal_size_hits"), Some(&1.0));
        assert_eq!(decisions.fields.get("geometry_checks"), Some(&1.0));
        let bfc_decisions = snapshot
            .instants
            .iter()
            .find(|event| event.name == "layout_reuse.BfcDecisionSummary")
            .expect("BFC reuse decision summary");
        assert_eq!(
            bfc_decisions.fields.get("forced_offset_rejections"),
            Some(&1.0)
        );
        assert_eq!(bfc_decisions.fields.get("hits"), Some(&1.0));
    }
}
