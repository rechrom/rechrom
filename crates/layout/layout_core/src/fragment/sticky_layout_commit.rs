// Post-layout sticky constraint update publishes fragment-local paint
// geometry, so both its trace and sampling frame belong to layout_fragment.

use super::*;

// Source: LocalFrameView::PerformPostLayoutTasks -> ExecutePendingStickyUpdates
// (core/frame/local_frame_view.cc:1916,5566-5578). Only the scroll containers
// that PaintLayerScrollableArea::UpdateAfterLayout queued during this layout
// are visited, each over its own consumed StickyDescendants().
#[inline(never)]
pub(super) fn execute_pending_sticky_updates_after_layout(root: &mut LayoutObject) {
    let _phase = layout_fragment::trace::span(
        layout_fragment::trace::TracePhase::UpdateStickyOffsets,
    );
    assert!(root.IsLayoutView());
    unsafe { &*(root as *const LayoutObject as *const crate::internal::layout_view::LayoutView) }
        .ExecutePendingStickyUpdates();
}
