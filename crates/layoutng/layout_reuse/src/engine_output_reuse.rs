// Reusing the resident engine output is an algorithm-neutral lifecycle cache
// decision. Keep its full admission predicate visible as a reuse-owned frame.

use super::*;

#[inline(never)]
pub(super) fn has_current_output(engine: &LayoutEngine, purpose: LayoutUpdateKind) -> bool {
    engine.fragments.is_some()
        && engine.invalidated_fragments_.is_none()
        && engine.pending_scroll_ids_.is_empty()
        && engine.pending_paint_ids_.is_empty()
        && engine.pending_inline_ink_ids_.is_empty()
        && engine.pending_subtree_layout_id_.is_none()
        && (purpose == LayoutUpdateKind::Layout || engine.output_kind == LayoutUpdateKind::Paint)
        && engine
            .tree_
            .as_ref()
            .is_some_and(|tree| !tree.Root().NeedsLayout())
}
