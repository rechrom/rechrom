//! Scroll-only refresh of retained source fragment paint properties.
//!
//! Included as a child of the LayoutNG boundary module so it can use the
//! translated source adapters without making the standalone fragment package
//! depend on concrete LayoutObject types.

use super::*;

#[inline(never)]
pub(super) fn refresh_scroll_paint_properties(
    root: &mut LayoutObject,
    fragments: crate::layout_engine::PaintFragmentCursor<'_>,
    reuse_sticky_constraints: bool,
) {
    let _trace = layout_commit::fragment_trace::span(
        layout_commit::fragment_trace::TracePhase::RefreshScrollPaintProperties,
    );
    let root_ptr: *mut LayoutObject = root;
    let mut object = root_ptr;
    while !object.is_null() {
        // The borrowed resident tree owns these objects; the outer heap scope
        // prevents collection during this walk and sticky-property update.
        let current = unsafe { &*object };
        let next = current.NextInPreOrder(root_ptr);
        // Constraints never change topology. Update in the same logical
        // preorder so nested sticky objects still observe refreshed ancestors,
        // without allocating an all-object vector and walking it again.
        UpdateStickyOffsets(std::iter::once(object), reuse_sticky_constraints);
        object = next;
    }
    unsafe fn update(fragment: crate::layout_engine::PaintFragmentCursor<'_>) {
        let children = fragment.Children();
        refresh_fragment_scroll_paint_properties(unsafe { fragment.PaintDataMut() });
        for child in children {
            unsafe { update(child) };
        }
    }
    unsafe { update(fragments) };
}

pub(super) fn refresh_fragment_scroll_paint_properties(fragment: &mut FragmentPaintData) -> bool {
    // A sticky box moves with every scroll of its scroller; PrePaint reads
    // its offset and constraint live (LiveStickyPaintProperties), so report
    // it changed rather than keeping copies to compare.
    let mut sticky = false;
    // Scroll containers and scrollbars are read live from the LayoutBox and
    // its PaintLayerScrollableArea (ViewScrollContainer / ViewScrollbars).
    // Their source dirty bits say what a scroll changed:
    // PaintLayerScrollableArea::UpdateScrollOffset sets
    // NeedsPaintPropertyUpdate (paint_layer_scrollable_area.cc:427-432) and
    // scrollbar paint invalidation sets ShouldCheckForPaintInvalidation
    // (scrollable_area.cc:786-826, paint_layer_scrollable_area.cc:3328-3330).
    // Both stay set until the frame's client commit clears the paint flags.
    let mut scroll_changed = false;
    if fragment.paint.HasSource() {
        // Population binds every sourced slot to its LayoutObject; HasSource()
        // reads that binding, so the bound owner is the source object.
        let object = fragment.paint.SourceObjectPointer();
        if !object.is_null() {
            let source = unsafe { &*object };
            sticky = source.StyleRef().GetPosition() == foundation::EPosition::kSticky;
            // PaintLayerScrollableArea::UpdateScrollOffset and
            // InvalidatePaintForStickyDescendants set NeedsPaintPropertyUpdate on
            // the scroller and its sticky descendants
            // (core/paint/paint_layer_scrollable_area.cc:2417-2440); republish
            // this fragment's snapshot of the LayoutObject bits.
            fragment.pre_paint_invalidation = ExportPrePaintInvalidation(object);
            let layout_box = DynamicTo::<LayoutBox>(object);
            if !layout_box.is_null() && !unsafe { &*layout_box }.GetScrollableArea().is_null() {
                scroll_changed =
                    source.NeedsPaintPropertyUpdate() || source.ShouldCheckForPaintInvalidation();
            }
        }
    }
    sticky || scroll_changed
}

/// Republish one retained fragment's snapshot of its source object's
/// PrePaint dirty bits. Scroll and sticky updates set NeedsPaintPropertyUpdate
/// on the changed object and DescendantNeedsPaintPropertyUpdate on its
/// ancestors (LayoutObject::SetNeedsPaintPropertyUpdate, core/layout/
/// layout_object.h); ancestor fragments on a targeted refresh path expose
/// those descendant bits through this.
pub(super) fn refresh_fragment_pre_paint_invalidation(fragment: &mut FragmentPaintData) {
    if !fragment.paint.HasSource() {
        return;
    }
    let object = fragment.paint.SourceObjectPointer();
    if !object.is_null() {
        fragment.pre_paint_invalidation = ExportPrePaintInvalidation(object);
    }
}
