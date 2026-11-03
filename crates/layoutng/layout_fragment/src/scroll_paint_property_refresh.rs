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
    let old_scroll = fragment.paint.scroll_offset;
    let old_scroll_coordinates = fragment
        .paint
        .scroll_container
        .as_ref()
        .map(|scroll| (scroll.scroll_origin, scroll.scroll_position));
    let old_sticky = fragment.paint.sticky_offset;
    let old_sticky_constraint = fragment.paint.sticky_constraint;
    let old_scrollbars = fragment.paint.scrollbars.clone();
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
            let node = unsafe { &*object }.GetNode();
            if !node.is_null() && unsafe { &*node }.IsElementNode() {
                let element = unsafe { &*(node as *const Element) };
                if let Some(data) = element.InputElementData() {
                    fragment.paint.scroll_offset = data.scroll_offset;
                }
            }
            ExportStickyPaintProperties(object, &mut fragment.paint);
            let layout_box = DynamicTo::<LayoutBox>(object);
            if !layout_box.is_null() {
                let layout_box = unsafe { &*layout_box };
                let area = layout_box.GetScrollableArea();
                if let Some(scroll) = fragment.paint.scroll_container.as_mut() {
                    // Source: core/paint/paint_property_tree_builder.cc:
                    // 3574-3614. A retained Scroll node has one source
                    // ScrollableArea owner; scroll-only refresh must consume
                    // it rather than fabricate origin/position from the DOM
                    // offset when ownership is missing.
                    assert!(!area.is_null());
                    let area = unsafe { &*area };
                    let origin = area.ScrollOrigin();
                    let position = area.ScrollPosition();
                    scroll.scroll_origin = Offset {
                        x: origin.x() as f64,
                        y: origin.y() as f64,
                    };
                    scroll.scroll_position = Offset {
                        x: position.x() as f64,
                        y: position.y() as f64,
                    };
                }
                if fragment.paint.scrollbars.is_some() {
                    assert!(!area.is_null());
                    let area = unsafe { &*area };
                    let theme = layout_box.ScrollbarThemeForLayout();
                    let data = Arc::make_mut(fragment.paint.scrollbars.as_mut().unwrap());
                    if let Some(axis) = data.horizontal.as_mut() {
                        let scrollbar = area.HorizontalScrollbar();
                        if !scrollbar.is_null() {
                            SetScrollbarThumb(
                                axis,
                                unsafe { &*scrollbar },
                                theme.has_buttons,
                                theme.minimum_thumb_length as f64,
                            );
                        }
                    }
                    if let Some(axis) = data.vertical.as_mut() {
                        let scrollbar = area.VerticalScrollbar();
                        if !scrollbar.is_null() {
                            SetScrollbarThumb(
                                axis,
                                unsafe { &*scrollbar },
                                theme.has_buttons,
                                theme.minimum_thumb_length as f64,
                            );
                        }
                    }
                }
            }
        }
    }
    old_scroll != fragment.paint.scroll_offset
        || old_scroll_coordinates
            != fragment
                .paint
                .scroll_container
                .as_ref()
                .map(|scroll| (scroll.scroll_origin, scroll.scroll_position))
        || old_sticky != fragment.paint.sticky_offset
        || old_sticky_constraint != fragment.paint.sticky_constraint
        || old_scrollbars != fragment.paint.scrollbars
}
