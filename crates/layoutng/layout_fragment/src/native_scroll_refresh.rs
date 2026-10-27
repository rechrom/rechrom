//! Scroll-only refresh of retained native fragment paint properties.
//!
//! Included as a child of the LayoutNG boundary module so it can use the
//! translated native adapters without making the standalone fragment package
//! depend on concrete LayoutObject types.

use super::*;

#[inline(never)]
pub(super) fn refresh_scroll_paint_properties(
    root: &mut LayoutObject,
    fragments: crate::layout_engine::NativePaintFragmentCursor<'_>,
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
        fragment: crate::layout_engine::NativePaintFragmentCursor<'_>,
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
    let old_scroll = fragment.paint.scroll_offset;
    let old_sticky = fragment.paint.sticky_offset;
    let old_sticky_constraint = fragment.paint.sticky_constraint;
    let old_scrollbars = fragment.paint.scrollbars.clone();
    if fragment.paint.has_source {
        if let Some(&object) = fragment
            .paint
            .logical_tree_order
            .checked_sub(1)
            .and_then(|order| usize::try_from(order).ok())
            .and_then(|index| sources.get(index))
        {
            let node = unsafe { &*(*object).GetNode() };
            if node.IsElementNode() {
                let element = unsafe { &*(node as *const Node as *const Element) };
                if let Some(data) = element.InputElementData() {
                    fragment.paint.scroll_offset = data.scroll_offset;
                }
            }
            ExportStickyPaintProperties(object, &mut fragment.paint);
            let layout_box = DynamicTo::<LayoutBox>(object);
            if !layout_box.is_null() && fragment.paint.scrollbars.is_some() {
                let layout_box = unsafe { &*layout_box };
                let area = layout_box.GetScrollableArea();
                if !area.is_null() {
                    let area = unsafe { &*area };
                    let theme = layout_box.ScrollbarThemeForLayout();
                    let visible =
                        area.VisibleContentRect(IncludeScrollbarsInRect::kExcludeScrollbars);
                    let contents = area.ContentsSize();
                    let minimum = area.MinimumScrollOffsetInt();
                    let maximum = area.MaximumScrollOffsetInt();
                    let current = area.GetScrollOffset();
                    let data = Arc::make_mut(fragment.paint.scrollbars.as_mut().unwrap());
                    if let Some(axis) = data.horizontal.as_mut() {
                        SetScrollbarThumb(
                            axis,
                            visible.width() as f64,
                            contents.width() as f64,
                            current.x() as f64,
                            minimum.x() as f64,
                            maximum.x() as f64,
                            theme.has_buttons,
                            theme.minimum_thumb_length as f64,
                        );
                    }
                    if let Some(axis) = data.vertical.as_mut() {
                        SetScrollbarThumb(
                            axis,
                            visible.height() as f64,
                            contents.height() as f64,
                            current.y() as f64,
                            minimum.y() as f64,
                            maximum.y() as f64,
                            theme.has_buttons,
                            theme.minimum_thumb_length as f64,
                        );
                    }
                } else if layout_box.IsLayoutView() {
                    let theme = layout_box.ScrollbarThemeForLayout();
                    let visible = fragment
                        .paint
                        .scroll_container
                        .map_or(fragment.size, |container| container.container_rect.size);
                    let contents = Size {
                        width: fragment.paint.scroll_size.width.max(visible.width),
                        height: fragment.paint.scroll_size.height.max(visible.height),
                    };
                    let current = fragment.paint.scroll_offset;
                    let data = Arc::make_mut(fragment.paint.scrollbars.as_mut().unwrap());
                    if let Some(axis) = data.horizontal.as_mut() {
                        SetScrollbarThumb(
                            axis,
                            visible.width,
                            contents.width,
                            current.x,
                            0.0,
                            (contents.width - visible.width).max(0.0),
                            theme.has_buttons,
                            theme.minimum_thumb_length as f64,
                        );
                    }
                    if let Some(axis) = data.vertical.as_mut() {
                        SetScrollbarThumb(
                            axis,
                            visible.height,
                            contents.height,
                            current.y,
                            0.0,
                            (contents.height - visible.height).max(0.0),
                            theme.has_buttons,
                            theme.minimum_thumb_length as f64,
                        );
                    }
                }
            }
        }
    }
    old_scroll != fragment.paint.scroll_offset
        || old_sticky != fragment.paint.sticky_offset
        || old_sticky_constraint != fragment.paint.sticky_constraint
        || old_scrollbars != fragment.paint.scrollbars
}
