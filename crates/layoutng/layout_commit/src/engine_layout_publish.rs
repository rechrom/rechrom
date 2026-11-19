// A successful source layout becomes resident output through one commit-owned
// transaction. Fragment certification and scroll-index construction retain
// their fragment subphases beneath this out-of-line publication frame.

use super::*;

pub(super) struct LayoutOutputPreparation {
    pub(super) root: *mut LayoutObject,
    pub(super) inputs: PrePaintRevisionTracker,
    pub(super) targeted_layout: bool,
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
    let mut inputs = std::mem::take(&mut engine.pre_paint_inputs_);
    // A Paint layout's publication certifies each fragment as it binds it;
    // publish_layout_output takes the tracker back, abandon_layout_output
    // drops it when no result is published.
    engine.boundary_cache_.pre_paint_certification =
        (purpose == LayoutUpdateKind::Paint).then(|| std::mem::take(&mut inputs));
    LayoutOutputPreparation {
        root,
        inputs,
        targeted_layout: subtree_id.is_some(),
    }
}

/// Layout produced no result: discard the lent PrePaint basis, as an unwound
/// pass does, so the next success certifies from fresh revisions.
#[inline(never)]
pub(super) fn abandon_layout_output(engine: &mut LayoutEngine) {
    engine.boundary_cache_.pre_paint_certification = None;
}

#[inline(never)]
pub(super) fn run_layout_without_publication(engine: &mut LayoutEngine) {
    run_layout_without_publication_impl(engine, true);
}

#[inline(never)]
pub(super) fn run_provisional_layout_without_publication(engine: &mut LayoutEngine) {
    run_layout_without_publication_impl(engine, false);
}

fn run_layout_without_publication_impl(
    engine: &mut LayoutEngine,
    invalidate_pre_paint_basis: bool,
) {
    let _trace = layout_commit::trace_span(layout_commit::TracePhase::RunLayoutWithoutPublication);
    if let Some(index) = engine.scroll_paint_index_.take() {
        engine.scroll_paint_index_spare_ = Some(index);
    }
    engine.pending_scroll_ids_.clear();
    let root = engine.RootPointer();
    engine.fragments = None;
    if invalidate_pre_paint_basis {
        // Retained scroll refresh is owned by the caller and can mutate inputs
        // without publication. Its next ordinary layout must establish a basis.
        engine.pre_paint_inputs_.invalidate();
    }
    let constraints = engine.constraints_.clone();
    if invalidate_pre_paint_basis {
        let _ = engine.RunPaintLayoutLifecycle(
            unsafe { &mut *root },
            &constraints,
            false,
            LayoutUpdateKind::Paint,
        );
    } else {
        let _ = engine.RunLayout(
            unsafe { &mut *root },
            &constraints,
            false,
            LayoutUpdateKind::Paint,
        );
    }
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
        // Population already certified every fragment it bound.
        inputs = engine
            .boundary_cache_
            .pre_paint_certification
            .take()
            .expect("Paint publication must return its PrePaint certification");
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
            let cached_objects = engine.CachedScrollIndexObjects();
            // RunLayout's temporary pass has ended. Walking source SVG
            // children still needs this engine's real virtual factories.
            let _pass = LayoutPassScope::new(&engine.algorithms_, &engine.objects_);
            ScrollPaintPropertyIndex::new(unsafe { &mut *root }, &fragments, cached_objects, reuse)
        });
    }
    engine.published_result_stale_ = false;
    engine.fragments = Some(fragments);
    engine.pre_paint_inputs_ = inputs;
    // FrameSelection::DidLayout -> UpdateAppearance (frame_selection.cc:
    // 1254-1261): offsets are recomputed against the new native OffsetMapping.
    if let Some(tree) = engine.tree_.as_ref() {
        tree.EditingState()
            .selections
            .layout_selection
            .borrow_mut()
            .SetHasPendingSelection();
    }
}
