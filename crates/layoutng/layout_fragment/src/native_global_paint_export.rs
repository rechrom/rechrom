//! Document-global paint facts projected from the resident native tree.
//!
//! Canvas background transfer and fixed-background scroll requirements each
//! retain fragment-qualified sampling frames.

#![allow(non_snake_case)]

use super::*;

#[inline(never)]
pub(super) fn ExportCanvasBackgroundNative(
    root_object: &LayoutObject,
    objects: &[*mut LayoutObject],
    output: crate::layout_engine::NativePaintFragmentCursor<'_>,
) {
    if !root_object.IsDocumentElement() {
        return;
    }
    unsafe fn mark(cursor: crate::layout_engine::NativePaintFragmentCursor<'_>, node_id: u64) {
        let children = cursor.Children();
        let input = unsafe { cursor.PaintDataMut() };
        if input.node_id == node_id && input.paint.has_source {
            input.paint.background_transfers_to_view = true;
        }
        for child in children {
            unsafe { mark(child, node_id) };
        }
    }
    fn source(
        cursor: crate::layout_engine::NativePaintFragmentCursor<'_>,
        node_id: u64,
    ) -> Option<CanvasBackgroundPaintData> {
        if cursor.node_id == node_id && cursor.paint.has_source {
            return Some(CanvasBackgroundPaintData {
                style: cursor.paint.style.clone(),
                effective_zoom: cursor.paint.effective_zoom,
            });
        }
        cursor
            .Children()
            .into_iter()
            .find_map(|child| source(child, node_id))
    }
    let root_id = InputId(root_object);
    unsafe { mark(output, root_id) };
    let body = objects
        .iter()
        .copied()
        .find(|object| !object.is_null() && unsafe { &**object }.IsBody());
    let body_transfers = body.is_some_and(|body| {
        !root_object.StyleRef().HasBackground()
            && !root_object.ShouldApplyAnyContainment()
            && !unsafe { &*body }.ShouldApplyAnyContainment()
    });
    let source_id = if body_transfers {
        let body_id = InputId(body.expect("transferred body exists"));
        unsafe { mark(output, body_id) };
        body_id
    } else {
        root_id
    };
    let Some(canvas) = source(output, source_id) else {
        return;
    };
    if canvas.style.background_color.alpha <= 0.0 && canvas.style.background_images.is_empty() {
        return;
    }
    unsafe { output.PaintDataMut() }.paint.canvas_background = Some(canvas);
}

#[inline(never)]
pub(super) fn ExportGlobalMainThreadScrollFactsNative(
    root: &LayoutObject,
    objects: &[*mut LayoutObject],
    output: crate::layout_engine::NativePaintFragmentCursor<'_>,
) {
    if !root.IsLayoutView() {
        return;
    }
    unsafe { output.PaintDataMut() }
        .paint
        .requires_main_thread_scrolling_for_background_attachment_fixed =
        Some(objects.iter().any(|object| {
            let object = unsafe { &**object };
            if !object.IsBackgroundAttachmentFixedObject() {
                return false;
            }
            let model = DynamicTo::<LayoutBoxModelObject>(object as *const LayoutObject);
            debug_assert!(!model.is_null());
            debug_assert!(!unsafe { &*model }.BackgroundTransfersToViewDefault());
            if object.IsLayoutView()
                && object.GetBackgroundPaintLocation()
                    == crate::internal::layout_object::kBackgroundPaintInBorderBoxSpace
            {
                return false;
            }
            !object.CanCompositeBackgroundAttachmentFixed()
        }));
}
