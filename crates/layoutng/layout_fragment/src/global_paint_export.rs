//! Document-global paint facts read from the resident source tree.
//!
//! Canvas background selection (ViewPainter / BackgroundTransfersToView) is
//! a live read of the root and body fragments. Paginated document
//! backgrounds publish only their LayoutView-global facts, and the
//! fixed-background scroll requirement retains a fragment-qualified frame.

#![allow(non_snake_case)]

use super::*;
use crate::fragment_tree::{CanvasBackgroundPaintView, ResidentBoxGeometry as _};

/// Document::body() for the canvas background: the first child of the
/// document element with the body role (document.cc FirstBodyElement), read
/// from the retained node metadata rather than a document-order object list.
fn BodyLayoutObject(root_object: &LayoutObject) -> *const LayoutObject {
    if root_object.IsBody() {
        return root_object;
    }
    let node = root_object.GetNode();
    if node.is_null() || !unsafe { &*node }.IsContainerNode() {
        return std::ptr::null();
    }
    let mut child =
        unsafe { &*node.cast::<crate::internal::layout_node_metadata::ContainerNode>() }
            .firstChild();
    while !child.is_null() {
        let child_ref = unsafe { &*child };
        if matches!(
            child_ref.InputDocumentRole(),
            crate::internal::layout_input::DocumentRole::kBody
        ) {
            return child_ref.GetLayoutObject();
        }
        child = child_ref.nextSibling();
    }
    std::ptr::null()
}

/// The selected element's own background facts, borrowed from its resident
/// paint slot. The style is that slot's PaintStyle projection (still
/// populated per fragment by ExportPaintProperties); borders, padding and
/// size come from the resident fragment.
fn SelectedFragmentBackground<'a>(
    cursor: crate::layout_engine::PaintFragmentCursor<'a>,
) -> CanvasBackgroundPaintView<'a> {
    let data = cursor.PaintData();
    let size = data
        .paint
        .stitched_decoration
        .as_ref()
        .map_or(data.size, |decoration| decoration.size);
    let border = data.Borders();
    let padding = data.Padding();
    CanvasBackgroundPaintView {
        style: &data.paint.style,
        effective_zoom: data.paint.effective_zoom,
        document_element_node_id: 0,
        previous_document_element_geometry: data.paint.previous_box_geometry,
        document_element_paint_offset: Offset::default(),
        document_element_size: size,
        document_element_border: border,
        document_element_padding: padding,
        document_element_content_box: FragmentPaintRect {
            offset: Offset {
                x: border.left + padding.left,
                y: border.top + padding.top,
            },
            size: data.ViewContentSize(),
        },
        document_element_scrollable_overflow: data.paint.scrollable_overflow,
        document_element_has_transform: false,
        background_fill_layer_count: data.paint.background_fill_layer_count,
        background_any_local_attachment_image: data.paint.background_any_local_attachment_image,
        background_any_layer_uses_content_box: data.paint.background_any_layer_uses_content_box,
    }
}

/// The first resident fragment of `object` that carries a paint source,
/// through LayoutBox's retained fragment list or InlineCursor (no walk).
fn SourceFragment<'a>(
    output: crate::layout_engine::PaintFragmentCursor<'a>,
    object: &'a LayoutObject,
) -> Option<crate::layout_engine::PaintFragmentCursor<'a>> {
    if output.node_id == InputId(object) && output.paint.HasSource() {
        return Some(output);
    }
    output
        .ForLayoutObject(object)
        .filter(|cursor| cursor.paint.HasSource())
}

/// The document element's own background facts. Rechrom's root LayoutObject
/// is the LayoutView carrying the document element's node and style, so this
/// is also document.GetLayoutView()->StyleRef() in BoxFragmentPainter.
fn RootFragmentBackground<'a>(
    root_object: &'a LayoutObject,
    output: crate::layout_engine::PaintFragmentCursor<'a>,
) -> Option<CanvasBackgroundPaintView<'a>> {
    if !root_object.IsDocumentElement() {
        return None;
    }
    SourceFragment(output, root_object).map(SelectedFragmentBackground)
}

fn RootElementPaintOffset(root_object: &LayoutObject) -> Offset {
    let root_box = DynamicTo::<LayoutBox>(root_object as *const LayoutObject);
    if root_box.is_null() {
        return Offset::default();
    }
    let offset = unsafe { &*root_box }.FirstFragment().PaintOffset();
    Offset {
        x: Number(offset.left),
        y: Number(offset.top),
    }
}

/// Source: ViewPainter::PaintBoxDecorationBackground and
/// PaintRootElementGroup (core/paint/view_painter.cc:76-262, 264-449) paint
/// the root element's background, or the body's when it propagates per
/// LayoutBoxModelObject::BackgroundTransfersToView
/// (core/layout/layout_box_model_object.cc:1123-1151), positioned by the
/// document element's border box. `output` is the LayoutView root fragment.
/// The result is returned even when transparent, for LayoutView background
/// invalidation; CanvasBackgroundForRoot applies the painting admission.
pub(crate) fn ViewBackgroundForRoot<'a>(
    root_object: &'a LayoutObject,
    output: crate::layout_engine::PaintFragmentCursor<'a>,
) -> Option<CanvasBackgroundPaintView<'a>> {
    if !root_object.ShouldApplyRootBackgroundBehaviorForLayout() {
        return None;
    }
    let root_geometry = RootFragmentBackground(root_object, output)?;
    let body = unsafe { BodyLayoutObject(root_object).as_ref() };
    let body_transfers = body.is_some_and(|body| {
        let model = DynamicTo::<LayoutBoxModelObject>(body as *const LayoutObject);
        !model.is_null() && unsafe { &*model }.BackgroundTransfersToViewDefault()
    });
    let source = match body {
        Some(body) if body_transfers => body,
        _ => root_object,
    };
    let mut canvas = SelectedFragmentBackground(SourceFragment(output, source)?);
    canvas.document_element_node_id = InputId(root_object);
    canvas.document_element_size = root_geometry.document_element_size;
    // RootBox::FirstFragment().PaintOffset(), in LayoutView contents space.
    canvas.document_element_paint_offset = RootElementPaintOffset(root_object);
    canvas.document_element_border = root_geometry.document_element_border;
    canvas.document_element_padding = root_geometry.document_element_padding;
    canvas.previous_document_element_geometry = root_geometry.previous_document_element_geometry;
    canvas.document_element_content_box = root_geometry.document_element_content_box;
    canvas.document_element_scrollable_overflow =
        root_geometry.document_element_scrollable_overflow;
    canvas.document_element_has_transform = root_object.HasTransform();
    Some(canvas)
}

/// ViewBackgroundForRoot when it paints anything: a transparent canvas
/// background without images records no canvas drawing.
pub(crate) fn CanvasBackgroundForRoot<'a>(
    root_object: &'a LayoutObject,
    output: crate::layout_engine::PaintFragmentCursor<'a>,
) -> Option<CanvasBackgroundPaintView<'a>> {
    ViewBackgroundForRoot(root_object, output).filter(|canvas| {
        canvas.style.background_color().alpha > 0.0 || canvas.style.has_background_images()
    })
}

/// Publish the LayoutView-global background facts that are still retained:
/// - `view_background_invalidation` on the root slot. BoxPaintInvalidator
///   compares the previous slot's selection with the current one, so the
///   value captured at publication must survive into the next lifecycle.
/// - On paginated page border boxes / page containers: LayoutView::StyleRef()
///   and the root's positioning size, paint offset and fixed attachment
///   (view_painter.cc:397-402). The page box types and the fragment-local
///   page geometry are read live through ResidentBoxGeometry.
/// The painted canvas background and BackgroundTransfersToView are read live
/// (PaintFragmentCursor::CanvasBackground, PaintProperties).
#[inline(never)]
pub(super) fn ExportCanvasBackground(
    root_object: &LayoutObject,
    output: crate::layout_engine::PaintFragmentCursor<'_>,
) {
    let Some(root_geometry) = RootFragmentBackground(root_object, output) else {
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
        if input.IsPageBorderBox() {
            input.paint.page_document_background_style = Some(style.clone());
            input.paint.page_document_background_geometry =
                Some(crate::fragment_tree::PageBackgroundPaintData {
                    positioning_size,
                    positioning_area_offset: Offset::default(),
                    border,
                    padding,
                    has_background_fixed_to_viewport,
                });
        }
        if input.IsPageContainer() {
            input.paint.page_container_background_geometry =
                Some(crate::fragment_tree::PageContainerBackgroundPaintData {
                    positioning_size,
                    root_element_paint_offset,
                    has_background_fixed_to_viewport,
                    ..Default::default()
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
    // kPageContainer / kPageBorderBox fragments exist only below a paginated
    // root (PhysicalBoxFragment::IsPaginatedRoot); an ordinary screen layout
    // has none, so do not walk every fragment looking for them.
    if output.IsPaginatedRoot() {
        let style = root_geometry.style.clone();
        let positioning_size = root_geometry.document_element_size;
        unsafe {
            publish_page_document_style(
                output,
                &style,
                positioning_size,
                RootElementPaintOffset(root_object),
                output.Borders(),
                output.Padding(),
                output.paint.IsBackgroundAttachmentFixedObject(),
            )
        };
    }
    let view_background = ViewBackgroundForRoot(root_object, output)
        .map(|canvas| canvas.MapStyle(|style: &PaintStyle| style.clone()));
    unsafe { output.PaintDataMut() }
        .paint
        .view_background_invalidation = view_background;
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
