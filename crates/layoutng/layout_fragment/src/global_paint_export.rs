//! Document-global paint facts projected from the resident source tree.
//!
//! Canvas background transfer, paginated document backgrounds and
//! fixed-background scroll requirements retain fragment-qualified sampling
//! frames.

#![allow(non_snake_case)]

use super::*;
use crate::fragment_tree::ResidentBoxGeometry as _;

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
    // The root and body are the only objects whose fragments take part in
    // canvas-background transfer (layout_box_model_object.cc:1123-1151).
    // Outside pagination every fragment of an object lies below fragments of
    // its layout ancestors, so the search only descends through fragments
    // whose node is on the target's ancestor chain. Anonymous fragments
    // (node id 0) are always entered. Page fragments of a paginated root have
    // no such ancestry, so that case keeps the complete walk (`None`).
    fn descends(chain: Option<&[u64]>, node_id: u64) -> bool {
        chain.is_none_or(|chain| node_id == 0 || chain.contains(&node_id))
    }
    fn ancestor_chain(object: *const LayoutObject) -> Vec<u64> {
        let mut chain = Vec::new();
        let mut current = object;
        while !current.is_null() {
            chain.push(InputId(current));
            current = unsafe { &*current }.Parent();
        }
        chain
    }
    unsafe fn mark(
        cursor: crate::layout_engine::PaintFragmentCursor<'_>,
        node_id: u64,
        chain: Option<&[u64]>,
    ) {
        let children = cursor.Children();
        let input = unsafe { cursor.PaintDataMut() };
        if input.node_id == node_id && input.paint.HasSource() {
            input.paint.background_transfers_to_view = true;
        }
        for child in children {
            if descends(chain, child.node_id) {
                unsafe { mark(child, node_id, chain) };
            }
        }
    }
    fn source(
        cursor: crate::layout_engine::PaintFragmentCursor<'_>,
        node_id: u64,
        chain: Option<&[u64]>,
    ) -> Option<CanvasBackgroundPaintData> {
        if cursor.node_id == node_id && cursor.paint.HasSource() {
            let size = cursor
                .paint
                .stitched_decoration
                .as_ref()
                .map_or(cursor.size, |decoration| decoration.size);
            return Some(CanvasBackgroundPaintData {
                style: cursor.paint.style.clone(),
                effective_zoom: cursor.paint.EffectiveZoom(),
                document_element_node_id: 0,
                previous_document_element_geometry: cursor.paint.previous_box_geometry,
                document_element_paint_offset: Offset::default(),
                document_element_size: size,
                document_element_border: cursor.Borders(),
                document_element_padding: cursor.Padding(),
                document_element_content_box: FragmentPaintRect {
                    offset: Offset {
                        x: cursor.Borders().left + cursor.Padding().left,
                        y: cursor.Borders().top + cursor.Padding().top,
                    },
                    size: cursor.content_size,
                },
                document_element_scrollable_overflow: cursor.paint.scrollable_overflow,
                document_element_has_transform: false,
                background_fill_layer_count: cursor.paint.BackgroundFillLayerCount(),
                background_any_local_attachment_image: cursor.paint.BackgroundAnyLocalAttachmentImage(),
                background_any_layer_uses_content_box: cursor.paint.BackgroundAnyLayerUsesContentBox(),
            });
        }
        cursor
            .Children()
            .into_iter()
            .filter(|child| descends(chain, child.node_id))
            .find_map(|child| source(child, node_id, chain))
    }
    let root_id = InputId(root_object);
    let paginated = output.paint.is_paginated_root;
    let root_chain = ancestor_chain(root_object);
    let root_chain = (!paginated).then_some(root_chain.as_slice());
    let Some(root_geometry) = source(output, root_id, root_chain) else {
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
    // kPageContainer / kPageBorderBox fragments exist only below a paginated
    // root (PhysicalBoxFragment::IsPaginatedRoot); an ordinary screen layout
    // has none, so do not walk every fragment looking for them.
    if root_paint.is_paginated_root {
        unsafe {
            publish_page_document_style(
                output,
                &root_geometry.style,
                root_geometry.document_element_size,
                root_element_paint_offset,
                output.Borders(),
                output.Padding(),
                root_paint.is_background_attachment_fixed_object,
            )
        };
    }
    if !should_apply_root_background_behavior {
        return;
    }
    unsafe { mark(output, root_id, root_chain) };
    let body = objects
        .iter()
        .copied()
        .find(|object| !object.is_null() && unsafe { &**object }.IsBody());
    let body_transfers = body.is_some_and(|body| {
        !root_object.StyleRef().HasBackground()
            && !root_object.ShouldApplyAnyContainment()
            && !unsafe { &*body }.ShouldApplyAnyContainment()
    });
    let body_chain = body
        .filter(|_| body_transfers && !paginated)
        .map(|body| ancestor_chain(body));
    let (source_id, source_chain) = if body_transfers {
        let body_id = InputId(body.expect("transferred body exists"));
        let chain = if paginated { None } else { body_chain.as_deref() };
        unsafe { mark(output, body_id, chain) };
        (body_id, chain)
    } else {
        (root_id, root_chain)
    };
    let Some(mut canvas) = source(output, source_id, source_chain) else {
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
