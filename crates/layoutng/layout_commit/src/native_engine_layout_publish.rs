// A successful native layout becomes resident output through one commit-owned
// transaction. Fragment certification and scroll-index construction retain
// their fragment subphases beneath this out-of-line publication frame.

use super::*;

pub(super) struct LayoutOutputPreparation {
    pub(super) root: *mut LayoutObject,
    pub(super) inputs: PrePaintRevisionTracker,
    pub(super) targeted_native_layout: bool,
}

#[inline(never)]
pub(super) fn begin_layout_output(
    engine: &mut LayoutEngine,
    purpose: LayoutUpdateKind,
) -> LayoutOutputPreparation {
    if let Some(index) = engine.scroll_paint_index_.take() {
        engine.scroll_paint_index_spare_ = Some(index);
    }
    engine.pending_scroll_ids_.clear();
    let root = engine.RootPointer();
    let subtree_id = (purpose == LayoutUpdateKind::Paint)
        .then_some(engine.pending_subtree_layout_id_)
        .flatten();
    engine.pending_subtree_layout_id_ = None;
    let _resident_fragments = if let Some(fragments) = engine.fragments.take() {
        engine.invalidated_fragments_ = None;
        Some(fragments)
    } else {
        engine.invalidated_fragments_.take()
    };
    engine.output_kind = purpose;
    LayoutOutputPreparation {
        root,
        inputs: std::mem::take(&mut engine.pre_paint_inputs_),
        targeted_native_layout: subtree_id.is_some(),
    }
}

#[inline(never)]
pub(super) fn run_layout_without_publication(engine: &mut LayoutEngine) {
    let _trace = layout_commit::trace_span(layout_commit::TracePhase::RunLayoutWithoutPublication);
    if let Some(index) = engine.scroll_paint_index_.take() {
        engine.scroll_paint_index_spare_ = Some(index);
    }
    engine.pending_scroll_ids_.clear();
    let root = engine.RootPointer();
    engine.fragments = None;
    // Retained scroll refresh is owned by the caller and can mutate inputs
    // without publication. Its next ordinary layout must establish a basis.
    engine.pre_paint_inputs_.invalidate();
    let constraints = engine.constraints_.clone();
    let _ = engine.RunLayout(
        unsafe { &mut *root },
        &constraints,
        false,
        LayoutUpdateKind::Paint,
    );
}

#[inline(never)]
pub(super) fn publish_layout_output(
    engine: &mut LayoutEngine,
    root: *mut LayoutObject,
    purpose: LayoutUpdateKind,
    mut inputs: PrePaintRevisionTracker,
    fragments: LayoutResultHandle,
) {
    if purpose == LayoutUpdateKind::Paint {
        let mut trace = layout_fragment_boundary::trace::span(
            layout_fragment_boundary::trace::TracePhase::CertifyPaintInputs,
        );
        UpdateNativePrePaintInputs(&mut inputs, &fragments, &engine.objects_);
        let stats = inputs.last_stats();
        trace.set("nodes", stats.nodes as f64);
        trace.set("new_nodes", stats.new_nodes as f64);
    }
    if purpose == LayoutUpdateKind::Paint {
        let reuse = engine
            .scroll_paint_index_
            .take()
            .or(engine.scroll_paint_index_spare_.take());
        engine.scroll_paint_index_ = Some({
            let _trace = layout_fragment_boundary::trace::span(
                layout_fragment_boundary::trace::TracePhase::BuildScrollIndex,
            );
            // RunLayout's temporary pass has ended. Walking native SVG
            // children still needs this engine's real virtual factories.
            let _pass = LayoutPassScope::new(&engine.algorithms_, &engine.objects_);
            ScrollPaintPropertyIndex::new(
                unsafe { &mut *root },
                &fragments,
                engine.CachedScrollIndexObjects(),
                reuse,
            )
        });
    }
    engine.fragments = Some(fragments);
    engine.pre_paint_inputs_ = inputs;
}
