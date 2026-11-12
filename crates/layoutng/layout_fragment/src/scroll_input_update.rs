//! Chromium scroll-input mutation for retained fragment paint state.
//!
//! Included by `layout_object_builder` at the reciprocal assembly boundary.
//! The id-to-record fan-out and node metadata synchronization are fragment
//! input work, so their samples should not accrue to the generic tree facade.

use super::*;

#[inline(never)]
pub(super) fn update_scroll_offset(
    tree: &mut LayoutObjectTree,
    id: u64,
    offset: crate::internal::layout_input::Offset,
    scroll_type: layoutng_style::style::scroll_enums::mojom::blink::ScrollType,
) -> bool {
    let storage = unsafe { &mut *tree.storage_ };
    assert!(!storage.updating);
    let mut found = false;
    for key in storage.InputKeys(id).to_vec() {
        let record = storage.managed.get_mut(&key).unwrap();
        if let Some(element) = record.input.element.as_mut() {
            let mut area =
                crate::internal::layout_scrollable_area::PaintLayerScrollableArea::FromNode(
                    unsafe { &*record.node },
                );
            // The document element represents the viewport scroll owner in
            // DOM/Page mutations, while Blink stores that ScrollableArea on
            // the ancestor LayoutView. Resolve that existing native owner by
            // reference instead of introducing a second root-scroll model.
            let mut object = unsafe { &*record.node }.GetLayoutObject();
            while area.is_null() && !object.is_null() {
                let box_model = DynamicTo::<
                    crate::internal::layout_box_model_object::LayoutBoxModelObject,
                >(object);
                if !box_model.is_null() {
                    area = unsafe { &*box_model }.GetScrollableArea();
                }
                object = unsafe { &*object }.Parent();
            }
            if !area.is_null() {
                // This function is entered only for an explicit scroll
                // mutation. Clear the previous anchor before any layout can
                // compare geometry against that obsolete user-scroll basis.
                unsafe { &mut *area }.ClearScrollAnchorForExplicitScroll();
            }
            element.scroll_offset = offset;
            unsafe { &mut *record.node }.UpdateInputScrollOffsetWithType(offset, scroll_type);
            found = true;
        }
    }
    found
}

// Source: core/page/scrolling/scrolling_coordinator.cc:88-100 and
// core/scroll/scrollable_area.cc:911-932.  The compositor reports the admitted
// overlay-visibility result; the resident ScrollableArea remains its owner.
#[inline(never)]
pub(super) fn update_scrollbars_hidden_if_overlay(
    tree: &mut LayoutObjectTree,
    id: u64,
    hidden: bool,
) -> bool {
    let storage = unsafe { &mut *tree.storage_ };
    assert!(!storage.updating);
    let mut changed = false;
    for key in storage.InputKeys(id).to_vec() {
        let record = storage.managed.get_mut(&key).unwrap();
        if record.input.element.is_none() {
            continue;
        }
        let area =
            crate::internal::layout_scrollable_area::PaintLayerScrollableArea::FromNode(unsafe {
                &*record.node
            });
        if !area.is_null() {
            changed |= unsafe { &mut *area }.SetScrollbarsHiddenIfOverlayFromOwner(hidden);
        }
    }
    changed
}
