//! Document-global paint facts read live from the resident source tree.
//!
//! Canvas background selection (ViewPainter / BackgroundTransfersToView),
//! the document background painted on paginated pages and their
//! LayoutView-global positioning facts are read at paint time through a
//! fragment's LayoutObject View(), as Chromium's painters read the LayoutView
//! and document element. Nothing here is published into per-fragment slots.

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
        .ViewStitchedDecoration()
        .map_or(data.size, |decoration| decoration.size);
    let border = data.Borders();
    let padding = data.Padding();
    CanvasBackgroundPaintView {
        style: &data.paint.style,
        effective_zoom: data.paint.effective_zoom,
        document_element_node_id: 0,
        // LayoutBox::PreviousSize() and the saved previous content box and
        // scrollable overflow (box_paint_invalidator.cc ComputeViewBackground-
        // Invalidation reads them from the root box).
        previous_document_element_geometry: data.paint.PreviousBoxGeometry(),
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
    if output.ViewNodeId() == InputId(object) && output.paint.HasSource() {
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

/// The element whose background the canvas paints: the body when it
/// propagates per LayoutBoxModelObject::BackgroundTransfersToView
/// (core/layout/layout_box_model_object.cc:1123-1151), else the root.
fn CanvasBackgroundSource(root_object: &LayoutObject) -> &LayoutObject {
    let body = unsafe { BodyLayoutObject(root_object).as_ref() };
    let body_transfers = body.is_some_and(|body| {
        let model = DynamicTo::<LayoutBoxModelObject>(body as *const LayoutObject);
        !model.is_null() && unsafe { &*model }.BackgroundTransfersToViewDefault()
    });
    match body {
        Some(body) if body_transfers => body,
        _ => root_object,
    }
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
    let source = CanvasBackgroundSource(root_object);
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

/// The root geometry ViewBackgroundForRoot records (document element size,
/// borders, padding, content box, scrollable overflow, previous geometry and
/// transform), without the canvas background source selection. Its style
/// and background-layer facts are the document element's own, not the
/// selected source's; LayoutView invalidation reads those from
/// ViewBackgroundForRoot only when this geometry changed.
pub(crate) fn ViewBackgroundRootGeometryForRoot<'a>(
    root_object: &'a LayoutObject,
    output: crate::layout_engine::PaintFragmentCursor<'a>,
) -> Option<CanvasBackgroundPaintView<'a>> {
    if !root_object.ShouldApplyRootBackgroundBehaviorForLayout() {
        return None;
    }
    let mut root = RootFragmentBackground(root_object, output)?;
    root.document_element_has_transform = root_object.HasTransform();
    Some(root)
}

/// ViewBackgroundForRoot when it paints anything: a transparent canvas
/// background without images records no canvas drawing.
pub(crate) fn CanvasBackgroundForRoot<'a>(
    root_object: &'a LayoutObject,
    output: crate::layout_engine::PaintFragmentCursor<'a>,
) -> Option<CanvasBackgroundPaintView<'a>> {
    // ViewPainter reads the propagated background straight from the
    // selected element's ComputedStyle (view_painter.cc:76-262). Decide the
    // painting admission from that live style first, so a transparent canvas
    // (the common case) does not assemble the root/body background view.
    if !root_object.ShouldApplyRootBackgroundBehaviorForLayout()
        || !root_object.IsDocumentElement()
    {
        return None;
    }
    let style = CanvasBackgroundSource(root_object).StyleRef();
    if !(crate::internal::boundary::paint_style_export::StyleColor(style, style.BackgroundColor())
        .alpha
        > 0.0
        || style.HasBackgroundImage())
    {
        return None;
    }
    ViewBackgroundForRoot(root_object, output).filter(|canvas| {
        canvas.style.background_color().alpha > 0.0 || canvas.style.has_background_images()
    })
}

/// The document element's own background facts for an arbitrary fragment of
/// the result, reached through its LayoutObject's View(): LayoutView::StyleRef()
/// as a kPageBorderBox paints it (BoxFragmentPainter, document canvas on each
/// page). `output` is any cursor of the same result.
pub(crate) fn DocumentElementBackgroundForObject<'a>(
    object: &'a LayoutObject,
    output: crate::layout_engine::PaintFragmentCursor<'a>,
) -> Option<CanvasBackgroundPaintView<'a>> {
    let view = unsafe { (object.View() as *const LayoutObject).as_ref() }?;
    RootFragmentBackground(view, output)
}

/// LayoutView-global BoxBackgroundPaintContext facts of a paginated document
/// background (view_painter.cc:397-402): the document element's positioning
/// size and borders/padding, its first fragment's paint offset, and whether
/// it has a background fixed to the viewport. Read live from the LayoutView
/// of `object` and its resident root fragment; the page-local link and page
/// area offsets are read by ResidentBoxGeometry from the page fragment.
pub(crate) fn PageBackgroundViewFacts(
    object: &LayoutObject,
) -> Option<(crate::fragment_tree::PageBackgroundPaintData, Offset)> {
    let view = unsafe { (object.View() as *const LayoutObject).as_ref() }?;
    if !view.IsDocumentElement() {
        return None;
    }
    let root_box = unsafe { DynamicTo::<LayoutBox>(view as *const LayoutObject).as_ref() }?;
    if root_box.PhysicalFragmentCount() == 0 {
        return None;
    }
    let fragment = unsafe { &*root_box.GetPhysicalFragment(0) };
    // The root slot's stitched decoration size, else its border-box size.
    let positioning_size = ExportStitchedDecorationData(fragment).map_or_else(
        || {
            let size = fragment.Size();
            Size {
                width: size.width.ToDouble(),
                height: size.height.ToDouble(),
            }
        },
        |decoration| decoration.size,
    );
    let border = fragment.Borders();
    let padding = fragment.Padding();
    Some((
        crate::fragment_tree::PageBackgroundPaintData {
            positioning_size,
            positioning_area_offset: Offset::default(),
            border: Edges {
                top: border.top.ToDouble(),
                right: border.right.ToDouble(),
                bottom: border.bottom.ToDouble(),
                left: border.left.ToDouble(),
            },
            padding: Edges {
                top: padding.top.ToDouble(),
                right: padding.right.ToDouble(),
                bottom: padding.bottom.ToDouble(),
                left: padding.left.ToDouble(),
            },
            has_background_fixed_to_viewport: view.IsBackgroundAttachmentFixedObject(),
        },
        RootElementPaintOffset(view),
    ))
}
