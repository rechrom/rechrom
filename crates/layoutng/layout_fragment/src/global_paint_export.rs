//! Document-global paint facts projected from the resident source tree.
//!
//! Canvas background transfer, paginated document backgrounds and
//! fixed-background scroll requirements retain fragment-qualified sampling
//! frames.

#![allow(non_snake_case)]

use super::*;

#[inline(never)]
pub(super) fn ExportCanvasBackground(
    root_object: &LayoutObject,
    objects: &[*mut LayoutObject],
    output: crate::layout_engine::PaintFragmentCursor<'_>,
    should_apply_root_background_behavior: bool,
) {
    if !root_object.IsDocumentElement() {
        return;
    }
    unsafe fn mark(cursor: crate::layout_engine::PaintFragmentCursor<'_>, node_id: u64) {
        let children = cursor.Children();
        let input = unsafe { cursor.PaintDataMut() };
        if input.node_id == node_id && input.paint.HasSource() {
            input.paint.background_transfers_to_view = true;
        }
        for child in children {
            unsafe { mark(child, node_id) };
        }
    }
    fn source(
        cursor: crate::layout_engine::PaintFragmentCursor<'_>,
        node_id: u64,
    ) -> Option<CanvasBackgroundPaintData> {
        if cursor.node_id == node_id && cursor.paint.HasSource() {
            let size = cursor
                .paint
                .stitched_decoration
                .as_ref()
                .map_or(cursor.size, |decoration| decoration.size);
            return Some(CanvasBackgroundPaintData {
                style: cursor.paint.style.clone(),
                effective_zoom: cursor.paint.effective_zoom,
                document_element_node_id: 0,
                previous_document_element_geometry: cursor.paint.previous_box_geometry,
                document_element_paint_offset: Offset::default(),
                document_element_size: size,
                document_element_border: cursor.paint.border,
                document_element_padding: cursor.paint.padding,
                document_element_content_box: FragmentPaintRect {
                    offset: Offset {
                        x: cursor.paint.border.left + cursor.paint.padding.left,
                        y: cursor.paint.border.top + cursor.paint.padding.top,
                    },
                    size: cursor.content_size,
                },
                document_element_scrollable_overflow: cursor.paint.scrollable_overflow,
                document_element_has_transform: false,
                background_fill_layer_count: cursor.paint.background_fill_layer_count,
                background_any_local_attachment_image: cursor
                    .paint
                    .background_any_local_attachment_image,
                background_any_layer_uses_content_box: cursor
                    .paint
                    .background_any_layer_uses_content_box,
            });
        }
        cursor
            .Children()
            .into_iter()
            .find_map(|child| source(child, node_id))
    }
    let root_id = InputId(root_object);
    let Some(root_geometry) = source(output, root_id) else {
        return;
    };
    unsafe fn publish_page_document_style(
        cursor: crate::layout_engine::PaintFragmentCursor<'_>,
        style: &PaintStyle,
        positioning_size: Size,
        root_element_paint_offset: Offset,
        border: Edges,
        padding: Edges,
        has_background_fixed_to_viewport: bool,
    ) {
        let children = cursor.Children();
        let input = unsafe { cursor.PaintDataMut() };
        if input.paint.is_page_border_box {
            input.paint.page_document_background_style = Some(style.clone());
            let offset = input
                .paint
                .page_document_background_geometry
                .map_or(Offset::default(), |geometry| {
                    geometry.positioning_area_offset
                });
            input.paint.page_document_background_geometry =
                Some(crate::fragment_tree::PageBackgroundPaintData {
                    positioning_size,
                    positioning_area_offset: offset,
                    border,
                    padding,
                    has_background_fixed_to_viewport,
                });
        }
        if input.paint.is_page_container {
            let local = input
                .paint
                .page_container_background_geometry
                .unwrap_or_default();
            input.paint.page_container_background_geometry =
                Some(crate::fragment_tree::PageContainerBackgroundPaintData {
                    positioning_size,
                    root_element_paint_offset,
                    has_background_fixed_to_viewport,
                    ..local
                });
        }
        for child in children {
            unsafe {
                publish_page_document_style(
                    child,
                    style,
                    positioning_size,
                    root_element_paint_offset,
                    border,
                    padding,
                    has_background_fixed_to_viewport,
                )
            };
        }
    }
    // Rechrom's root LayoutObject is the LayoutView carrying the document
    // element's style, so this is the immutable equivalent of
    // document.GetLayoutView()->StyleRef() in BoxFragmentPainter.
    let root_paint = &output.paint;
    let root_box = DynamicTo::<LayoutBox>(root_object as *const LayoutObject);
    let root_element_paint_offset = if root_box.is_null() {
        Offset::default()
    } else {
        let offset = unsafe { &*root_box }.FirstFragment().PaintOffset();
        Offset {
            x: Number(offset.left),
            y: Number(offset.top),
        }
    };
    unsafe {
        publish_page_document_style(
            output,
            &root_geometry.style,
            root_geometry.document_element_size,
            root_element_paint_offset,
            root_paint.border,
            root_paint.padding,
            root_paint.is_background_attachment_fixed_object,
        )
    };
    if !should_apply_root_background_behavior {
        return;
    }
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
    let Some(mut canvas) = source(output, source_id) else {
        return;
    };
    canvas.document_element_node_id = root_id;
    canvas.document_element_size = root_geometry.document_element_size;
    canvas.document_element_paint_offset = root_element_paint_offset;
    canvas.document_element_border = root_geometry.document_element_border;
    canvas.document_element_padding = root_geometry.document_element_padding;
    canvas.previous_document_element_geometry = root_geometry.previous_document_element_geometry;
    canvas.document_element_content_box = root_geometry.document_element_content_box;
    canvas.document_element_scrollable_overflow =
        root_geometry.document_element_scrollable_overflow;
    canvas.document_element_has_transform = root_object.HasTransform();
    unsafe { output.PaintDataMut() }
        .paint
        .view_background_invalidation = Some(canvas.clone());
    if canvas.style.background_color().alpha <= 0.0 && !canvas.style.has_background_images() {
        return;
    }
    unsafe { output.PaintDataMut() }.paint.canvas_background = Some(canvas);
}

#[inline(never)]
pub(super) fn ExportGlobalMainThreadScrollFacts(
    root: &LayoutObject,
    objects: &[*mut LayoutObject],
    output: crate::layout_engine::PaintFragmentCursor<'_>,
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
