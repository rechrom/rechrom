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
    let _trace = layout_fragment_boundary::trace::span(
        layout_fragment_boundary::trace::TracePhase::RefreshScrollPaintProperties,
    );
    let root_ptr: *mut LayoutObject = root;
    // Export assigns sourced objects consecutive, one-based preorder ids.
    // Keep that dense mapping without rebuilding a hash table for each scroll.
    let mut sources = Vec::new();
    let mut object = root_ptr;
    while !object.is_null() {
        // The borrowed resident tree owns these objects; the outer heap scope
        // prevents collection during this walk and sticky-property update.
        let current = unsafe { &*object };
        if !current.GetNode().is_null() {
            sources.push(object);
        }
        let next = current.NextInPreOrder(root_ptr);
        // Constraints never change topology. Update in the same logical
        // preorder so nested sticky objects still observe refreshed ancestors,
        // without allocating an all-object vector and walking it again.
        UpdateStickyOffsets(std::iter::once(object), reuse_sticky_constraints);
        object = next;
    }
    unsafe fn update(
        fragment: crate::layout_engine::PaintFragmentCursor<'_>,
        sources: &[*mut LayoutObject],
        retained_geometry: bool,
    ) -> bool {
        let children = fragment.Children();
        let own_changed =
            refresh_fragment_scroll_paint_properties(unsafe { fragment.PaintDataMut() }, sources);
        let mut changed = own_changed;
        for child in children {
            changed |= unsafe { update(child, sources, retained_geometry) };
        }
        // The layout boundary's certification covered the exported inputs.
        // Refresh mutated them in place: version exactly these paths so a
        // different resident PaintEngine cannot mistake stale revisions for a
        // fresh layout proof. Unaffected branches retain their certification.
        if changed && retained_geometry {
            layout_fragment::fragment_tree::pre_paint_revision::MarkScrollPropertyChange(
                unsafe { fragment.PaintDataMut() },
                own_changed,
            );
        } else {
            // A refreshed layout is not the retained-geometry proof. Keep the
            // existing unobserved-input fallback until a full certification.
            if own_changed {
                unsafe { fragment.PaintDataMut() }.pre_paint_revision = 0;
            }
            if changed {
                unsafe { fragment.PaintDataMut() }.pre_paint_subtree_revision = 0;
            }
        }
        changed
    }
    unsafe { update(fragments, &sources, reuse_sticky_constraints) };
}

pub(super) fn refresh_fragment_scroll_paint_properties(
    fragment: &mut FragmentPaintData,
    sources: &[*mut LayoutObject],
) -> bool {
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
        let direct = fragment.paint.SourceObjectPointer();
        let object = if direct.is_null() {
            fragment
                .paint
                .logical_tree_order
                .checked_sub(1)
                .and_then(|order| usize::try_from(order).ok())
                .and_then(|index| sources.get(index).copied())
                .unwrap_or(std::ptr::null_mut())
        } else {
            direct
        };
        if !object.is_null() {
            let source = unsafe { &*object };
            sticky = source.StyleRef().GetPosition() == foundation::EPosition::kSticky;
            let layout_box = DynamicTo::<LayoutBox>(object);
            if !layout_box.is_null() && !unsafe { &*layout_box }.GetScrollableArea().is_null() {
                scroll_changed =
                    source.NeedsPaintPropertyUpdate() || source.ShouldCheckForPaintInvalidation();
            }
        }
    }
    sticky || scroll_changed
}
