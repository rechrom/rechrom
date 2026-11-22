//! Chromium input-triggered invalidation for the resident LayoutObject tree.
//!
//! Included by `layout_object_builder` so the standalone package remains free
//! of concrete source object types while the traversals keep invalidation
//! sampling and trace ownership.

use super::*;

type StyleFetchedImage = layout_style::style::style_fetched_image::StyleFetchedImage;

#[inline(always)]
fn style_image_matches(
    image: *mut layout_style::style::style_image::StyleImage,
    matches: &dyn Fn(&StyleFetchedImage) -> bool,
) -> bool {
    !image.is_null()
        && layout_style::style::style_fetched_image::FetchedImage(unsafe { &*image })
            .is_some_and(|image| matches(image))
}

#[inline(always)]
fn fill_layers_match(
    first: &layout_style::style::fill_layer::FillLayer,
    matches: &dyn Fn(&StyleFetchedImage) -> bool,
) -> bool {
    let mut layer = first as *const layout_style::style::fill_layer::FillLayer;
    while !layer.is_null() {
        let current = unsafe { &*layer };
        if style_image_matches(current.GetImage(), matches) {
            return true;
        }
        layer = current.Next();
    }
    false
}

/// ImageResourceObserver::ImageChanged delivered to one resident client for
/// the style images selected by `matches`; `replaced` reports that the
/// client's own content image (its LayoutImageResource) changed.
///
/// Source: core/layout/layout_box_model_object.cc:479-494;
///         core/layout/layout_box.cc:2102-2142;
///         core/layout/layout_image.cc:139-194,281-289;
///         core/layout/layout_inline.cc:854-864.
fn notify_image_changed(
    root: *mut LayoutObject,
    object: *mut LayoutObject,
    matches: &dyn Fn(&StyleFetchedImage) -> bool,
    replaced: bool,
) -> bool {
    let current = unsafe { &*object };
    let style = current.StyleRef();
    let background = fill_layers_match(style.BackgroundLayers(), matches);
    let mask = fill_layers_match(style.MaskLayers(), matches);
    let border = style_image_matches(style.BorderImage().GetImage(), matches);
    let mask_box = style_image_matches(style.MaskBoxImage().GetImage(), matches);
    let reflection = {
        let reflect = style.BoxReflect();
        !reflect.is_null() && style_image_matches(unsafe { &*reflect }.Mask().GetImage(), matches)
    };

    let box_model =
        DynamicTo::<crate::internal::layout_box_model_object::LayoutBoxModelObject>(object);
    let box_ = DynamicTo::<crate::internal::layout_box::LayoutBox>(object);
    let inline = DynamicTo::<crate::internal::layout_inline::LayoutInline>(object);
    let mut changed = false;

    if !box_model.is_null() && mask {
        let object = unsafe { &mut *object };
        object.SetNeedsPaintPropertyUpdate();
        object.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
            foundation::PaintInvalidationReason::kImage,
        );
        changed = true;
    }
    if !box_.is_null() {
        let box_ = unsafe { &mut *box_ };
        if reflection && box_.HasLayer() {
            box_.SetNeedsPaintPropertyUpdate();
        }
        if border || mask_box || reflection {
            box_.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                foundation::PaintInvalidationReason::kImage,
            );
            changed = true;
        }
        if background {
            if box_.BackgroundTransfersToViewDefault() {
                // LayoutView owns the canvas-background display item after
                // LayoutBoxModelObject::BackgroundTransfersToView().
                unsafe { &mut *root }.SetBackgroundNeedsFullPaintInvalidation();
            } else {
                box_.SetBackgroundNeedsFullPaintInvalidation();
            }
            changed = true;
        }
    } else if !inline.is_null() && (background || border || mask_box || reflection) {
        unsafe { &mut *object }.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
            foundation::PaintInvalidationReason::kImage,
        );
        changed = true;
    }
    if replaced {
        let object = unsafe { &mut *object };
        object.SetNeedsPaintPropertyUpdate();
        object.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
            foundation::PaintInvalidationReason::kImage,
        );
        changed = true;
    }
    changed
}

/// Notify the resident clients whose document-image rendering environment
/// changed without changing their CSS style or geometry.
///
/// Chromium reaches these branches through ImageResourceObserver::ImageChanged
/// (see `notify_image_changed`). The extracted resource owner publishes
/// immutable image catalogs instead, so the host/constraint boundary delivers
/// the same notification before retained Paint inputs are republished.
#[inline(never)]
pub(super) fn invalidate_document_image_clients(
    tree: &mut LayoutObjectTree,
    images: &[crate::internal::layout_input::PaintImage],
) -> usize {
    if !tree.HasRoot() {
        return 0;
    }
    let document_ids = images
        .iter()
        .filter(|image| image.IsDocumentImage())
        .map(|image| image.id)
        .collect::<std::collections::HashSet<_>>();
    if document_ids.is_empty() {
        return 0;
    }

    let root = tree.RootMut() as *mut LayoutObject;
    let mut object = root;
    let mut invalidated = 0;
    while !object.is_null() {
        let current = unsafe { &*object };
        let next = current.NextInPreOrder(root);
        let node = current.GetNode();
        let replaced = current.IsLayoutReplaced()
            && !node.is_null()
            && unsafe { &*node }
                .InputStyleResources()
                .image_resource_id
                .is_some_and(|id| document_ids.contains(&id));
        let changed =
            notify_image_changed(root, object, &|image| image.IsDocumentImage(), replaced);
        invalidated += usize::from(changed);
        object = next;
    }
    invalidated
}

/// The facts of one immutable document paint-resource catalog that paint
/// clients observe. Layout's ConstraintSpace and Page's published
/// PaintResources carry the same catalog; `base_background` is present only
/// where the source owns LocalFrameView's base background facts.
pub(crate) struct PaintResourceFacts<'a> {
    pub fonts: &'a font_engine::FontFaceCatalog,
    pub images: &'a [crate::internal::layout_input::PaintImage],
    pub canvases: &'a [canvas::CanvasPaintResource],
    pub device_pixel_ratio: f64,
    pub preferred_color_scheme: u8,
    pub layout_zoom_factor: f64,
    pub viewport: Option<crate::internal::layout_input::ViewportGeometry>,
    pub base_background: Option<(foundation::PaintColor, bool, bool)>,
}

impl<'a> PaintResourceFacts<'a> {
    pub(crate) fn FromConstraintSpace(
        space: &'a crate::internal::layout_input::ConstraintSpace,
    ) -> Self {
        Self {
            fonts: &space.fonts,
            images: &space.images,
            canvases: &space.canvases,
            device_pixel_ratio: space.device_pixel_ratio,
            preferred_color_scheme: space.preferred_color_scheme,
            layout_zoom_factor: space.layout_zoom_factor,
            viewport: space.viewport,
            base_background: None,
        }
    }

    pub(crate) fn FromPaintResources(
        resources: &'a layout_fragment::fragment_tree::PaintResources,
    ) -> Self {
        Self {
            fonts: &resources.fonts,
            images: &resources.images,
            canvases: &resources.canvases,
            device_pixel_ratio: resources.device_pixel_ratio,
            preferred_color_scheme: resources.preferred_color_scheme,
            layout_zoom_factor: resources.layout_zoom_factor,
            viewport: resources.viewport,
            base_background: Some((
                resources.base_background_color,
                resources.paints_opaque_base_background,
                resources.should_clear_document_background,
            )),
        }
    }
}

/// Resource ids whose immutable catalog entry was added, replaced or removed.
fn changed_image_ids(
    before: &[crate::internal::layout_input::PaintImage],
    after: &[crate::internal::layout_input::PaintImage],
) -> std::collections::HashSet<u64> {
    let mut changed = std::collections::HashSet::new();
    if before.len() == after.len() && before.iter().zip(after).all(|(a, b)| a == b) {
        return changed;
    }
    let previous = before
        .iter()
        .map(|image| (image.id, image))
        .collect::<std::collections::HashMap<_, _>>();
    for image in after {
        if previous.get(&image.id).is_none_or(|old| **old != *image) {
            changed.insert(image.id);
        }
    }
    let current = after
        .iter()
        .map(|image| image.id)
        .collect::<std::collections::HashSet<_>>();
    changed.extend(
        before
            .iter()
            .map(|image| image.id)
            .filter(|id| !current.contains(id)),
    );
    changed
}

/// Canvas element node ids whose published front buffer changed.
fn changed_canvas_nodes(
    before: &[canvas::CanvasPaintResource],
    after: &[canvas::CanvasPaintResource],
) -> std::collections::HashSet<u64> {
    let mut changed = std::collections::HashSet::new();
    if before.len() == after.len() && before.iter().zip(after).all(|(a, b)| a == b) {
        return changed;
    }
    let previous = before
        .iter()
        .map(|canvas| (canvas.dom_node_id, canvas))
        .collect::<std::collections::HashMap<_, _>>();
    for canvas in after {
        if previous
            .get(&canvas.dom_node_id)
            .is_none_or(|old| **old != *canvas)
        {
            changed.insert(canvas.dom_node_id);
        }
    }
    let current = after
        .iter()
        .map(|canvas| canvas.dom_node_id)
        .collect::<std::collections::HashSet<_>>();
    changed.extend(
        before
            .iter()
            .map(|canvas| canvas.dom_node_id)
            .filter(|id| !current.contains(id)),
    );
    changed
}

/// The face identity proof PrePaint's resource comparison uses: shared font
/// bytes plus the face descriptor.
fn font_catalog_changed(
    before: &font_engine::FontFaceCatalog,
    after: &font_engine::FontFaceCatalog,
) -> bool {
    !(before.len() == after.len()
        && (before.ptr_eq(after)
            || before.iter().zip(after.iter()).all(|(a, b)| {
                font_engine::SharedFontBytes::ptr_eq(&a.bytes, &b.bytes) && a == b
            })))
}

/// ImageResourceObserver::ImageChanged for every client kind a changed image
/// id reaches on one resident object.
fn notify_resource_image_clients(
    root: *mut LayoutObject,
    object: *mut LayoutObject,
    images: &std::collections::HashSet<u64>,
) -> bool {
    let matches = |image: &StyleFetchedImage| images.contains(&image.ResourceId());
    let current = unsafe { &*object };
    if current.IsLayoutTableCol() {
        // Source: core/layout/table/layout_table_column.cc:94-100
        // (translated in layout_algo_table/src/layout_table_column.rs). A
        // column observes its background images but paints nothing itself;
        // its table is invalidated instead.
        if !fill_layers_match(current.StyleRef().BackgroundLayers(), &matches) {
            return false;
        }
        let mut ancestor = current.Parent();
        while let Some(table) = unsafe { ancestor.as_mut() } {
            if table.IsTable() {
                table.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                    foundation::PaintInvalidationReason::kImage,
                );
                return true;
            }
            ancestor = table.Parent();
        }
        return false;
    }
    let node = current.GetNode();
    // LayoutImage / LayoutSVGImage observe their LayoutImageResource; a list
    // marker image's resource is its list-style-image.
    // Source: core/layout/layout_image.cc:139-194;
    //         core/layout/svg/layout_svg_image.cc:249-262.
    let content = !node.is_null()
        && (current.IsLayoutReplaced() || current.IsSVGImage())
        && unsafe { &*node }
            .InputStyleResources()
            .image_resource_id
            .is_some_and(|id| images.contains(&id));
    // Rechrom has no separate LayoutListMarkerImage class (its dispatch is
    // untranslated); the list marker object carries the list-style-image
    // that ListMarker::ListStyleImageChanged repaints.
    let marker = current.IsListMarker()
        && style_image_matches(current.StyleRef().ListStyleImage().Get(), &matches);
    let mut changed = notify_image_changed(root, object, &matches, content || marker);
    if !node.is_null() && unsafe { &*node }.IsElementNode() {
        // Source: core/html/media/html_media_element.cc:4142-4149
        // HTMLMediaElement::Repaint. A new video frame is a new catalog entry
        // named by the element's current frame resource.
        let element =
            unsafe { &*(node as *const crate::internal::layout_node_metadata::Element) };
        if element
            .InputElementData()
            .as_ref()
            .and_then(|data| data.video_frame_resource_id)
            .is_some_and(|id| images.contains(&id))
        {
            unsafe { &mut *object }.SetShouldDoFullPaintInvalidation();
            changed = true;
        }
    }
    if !node.is_null() {
        // An image inside an SVG resource container (pattern content, mask
        // content, feImage) reaches the container through
        // LayoutSVGImage::ImageChanged ->
        // LayoutSVGResourceContainer::MarkForLayoutAndParentResourceInvalidation
        // (core/layout/svg/layout_svg_image.cc:249-262,
        // layout_svg_resource_container.cc:344-385), whose SVGResource then
        // notifies each referencing element through
        // SVGElementResourceClient::ResourceContentChanged
        // (core/layout/svg/svg_resources.cc:279-318). Rechrom's resolved SVG
        // resource bindings carry no per-container image observer, so a
        // changed image reaches every client referencing such a container.
        let resources = unsafe { &*node }.InputStyleResources();
        if resources.svg_fill_server.is_some()
            || resources.svg_stroke_server.is_some()
            || resources.svg_pattern_content_target.is_some()
        {
            // LayoutSVGInlineTexts paint with their parent's servers.
            unsafe { &mut *object }.SetSubtreeShouldDoFullPaintInvalidation(
                foundation::PaintInvalidationReason::kSVGResource,
            );
            changed = true;
        }
        if !resources.svg_filter_graphs.is_empty()
            || resources.svg_mask_resource.is_some()
            || !resources.svg_markers.is_empty()
        {
            unsafe { &mut *object }.SetShouldDoFullPaintInvalidation();
            changed = true;
        }
    }
    changed
}

/// Deliver Chromium's paint-client notifications for the difference between
/// two immutable document paint-resource catalogs, so PrePaint observes the
/// change through LayoutObject dirty bits rather than a catalog comparison.
///
/// - Image entries: ImageResourceObserver::ImageChanged on their observers
///   (`notify_resource_image_clients`).
/// - Canvas front buffers: HTMLCanvasElement::DidDraw
///   (core/html/canvas/html_canvas_element.cc:671-689) followed by
///   LayoutHTMLCanvas::InvalidatePaint (core/layout/layout_html_canvas.cc:
///   99-112), which fully invalidates the canvas box.
/// - Base background: LocalFrameView::SetBaseBackgroundColor
///   (core/frame/local_frame_view.cc:1754-1765).
/// - Viewport geometry: LocalFrameView::FrameRectsChanged and
///   SetNeedsPaintPropertyUpdate (core/frame/local_frame_view.cc:602-612,
///   937-940), plus the fixed-attachment background repaint of
///   InvalidateBackgroundAttachmentFixedDescendantsOnScroll (:1485-1496).
/// - Device scale, color scheme or zoom: LayoutView::
///   InvalidatePaintForViewAndDescendants (core/layout/layout_view.cc:420-423).
/// - Font faces: text paints shaped glyphs from the catalog faces. Chromium
///   re-shapes through InvalidateSubtreeLayoutForFontUpdates, which Rechrom
///   delivers through style/FontFallbackChanged layout invalidation; a
///   catalog-only change asks PrePaint to check each LayoutText.
#[inline(never)]
pub(super) fn invalidate_paint_resource_clients(
    tree: &mut LayoutObjectTree,
    before: &PaintResourceFacts<'_>,
    after: &PaintResourceFacts<'_>,
) -> usize {
    if !tree.HasRoot() {
        return 0;
    }
    let images = changed_image_ids(before.images, after.images);
    let canvases = changed_canvas_nodes(before.canvases, after.canvases);
    let fonts = font_catalog_changed(before.fonts, after.fonts);
    let environment = before.device_pixel_ratio != after.device_pixel_ratio
        || before.preferred_color_scheme != after.preferred_color_scheme
        || before.layout_zoom_factor != after.layout_zoom_factor;
    let viewport = before.viewport != after.viewport;
    let base_background = matches!(
        (before.base_background, after.base_background),
        (Some(a), Some(b)) if a != b
    );
    if images.is_empty()
        && canvases.is_empty()
        && !fonts
        && !environment
        && !viewport
        && !base_background
    {
        return 0;
    }
    let mut trace =
        layout_invalidation::trace_span(layout_invalidation::TracePhase::ImageInvalidation);
    trace.set("changed_images", images.len() as f64);
    trace.set("changed_canvases", canvases.len() as f64);
    trace.set("fonts_changed", fonts as u8 as f64);
    trace.set("environment_changed", environment as u8 as f64);
    trace.set("viewport_changed", viewport as u8 as f64);
    trace.set("base_background_changed", base_background as u8 as f64);

    let root = tree.RootMut() as *mut LayoutObject;
    let mut invalidated = 0usize;
    if environment {
        unsafe { &mut *root }
            .SetSubtreeShouldDoFullPaintInvalidation(foundation::PaintInvalidationReason::kSubtree);
        invalidated += 1;
    }
    if base_background {
        unsafe { &mut *root }.SetBackgroundNeedsFullPaintInvalidation();
        invalidated += 1;
    }
    if viewport {
        let view = unsafe { &mut *root };
        view.SetShouldCheckForPaintInvalidation();
        view.SetNeedsPaintPropertyUpdate();
        invalidated += 1;
    }
    for &node_id in &canvases {
        if let Some(object) = tree.GetLayoutObjectForNodeMut(node_id) {
            if object.IsCanvas() {
                object.SetShouldDoFullPaintInvalidation();
                invalidated += 1;
            }
        }
    }
    if images.is_empty() && !fonts && !viewport {
        trace.set("clients", invalidated as f64);
        return invalidated;
    }
    let mut object = root;
    while !object.is_null() {
        let current = unsafe { &*object };
        let next = current.NextInPreOrder(root);
        let mut changed = false;
        if !images.is_empty() {
            changed |= notify_resource_image_clients(root, object, &images);
        }
        if fonts && current.IsText() {
            unsafe { &mut *object }.SetShouldCheckForPaintInvalidationWithoutLayoutChange();
            changed = true;
        }
        if viewport && current.StyleRef().HasFixedAttachmentBackgroundImage() {
            let box_model =
                DynamicTo::<crate::internal::layout_box_model_object::LayoutBoxModelObject>(object);
            if let Some(box_model) = unsafe { box_model.as_ref() } {
                if box_model.BackgroundTransfersToViewDefault() {
                    unsafe { &mut *root }.SetBackgroundNeedsFullPaintInvalidation();
                } else {
                    unsafe { &mut *object }.SetBackgroundNeedsFullPaintInvalidation();
                }
                changed = true;
            }
        }
        invalidated += usize::from(changed);
        object = next;
    }
    trace.set("clients", invalidated as f64);
    invalidated
}

#[inline(never)]
pub(super) fn invalidate_editing(tree: &mut LayoutObjectTree, id: u64) -> bool {
    let _trace =
        layout_invalidation::trace_span(layout_invalidation::TracePhase::EditingInvalidation);
    let storage = unsafe { &mut *tree.storage_ };
    assert!(!storage.updating);
    let mut found = false;
    for key in storage.InputKeys(id).to_vec() {
        let record = storage.managed.get_mut(&key).unwrap();
        unsafe { &mut *record.object }.SetNeedsLayout(std::ptr::addr_of!(
            crate::internal::layout_invalidation_reason::kTextControlChanged
        ));
        found = true;
    }
    found
}

#[inline(never)]
pub(super) fn invalidate_selection_paint(
    tree: &mut LayoutObjectTree,
    id: u64,
) -> Vec<*mut LayoutObject> {
    invalidate_text_paint(tree, id, foundation::PaintInvalidationReason::kSelection).1
}

pub(super) fn invalidate_caret_text_paint(
    tree: &mut LayoutObjectTree,
    id: u64,
) -> Vec<*mut LayoutObject> {
    invalidate_text_paint(tree, id, foundation::PaintInvalidationReason::kCaret).1
}

fn invalidate_text_paint(
    tree: &mut LayoutObjectTree,
    id: u64,
    reason: foundation::PaintInvalidationReason,
) -> (bool, Vec<*mut LayoutObject>) {
    let mut trace = layout_invalidation::trace_span(
        layout_invalidation::TracePhase::SelectionPaintInvalidation,
    );
    let storage = unsafe { &mut *tree.storage_ };
    assert!(!storage.updating);
    let roots = storage
        .InputKeys(id)
        .iter()
        .map(|key| storage.managed[key].object)
        .collect::<Vec<_>>();
    let mut stack = roots.clone();
    let mut visited = std::collections::HashSet::new();
    let mut leaves = Vec::new();
    while let Some(object) = stack.pop() {
        if !visited.insert(object) {
            continue;
        }
        // Text-control selections are painted by their generated LayoutText
        // descendants. Invalidating unrelated generated boxes would discard
        // more recordings than Chromium's selection client invalidation.
        if unsafe { &*object }.IsText() || unsafe { &*object }.IsBR() {
            leaves.push(object);
        }
        for key in storage.ChildKeys(object).to_vec() {
            stack.push(storage.managed[&key].object);
        }
    }
    trace.set("roots", roots.len() as f64);
    trace.set("visited", visited.len() as f64);
    trace.set("leaves", leaves.len() as f64);
    for &object in &leaves {
        unsafe { &mut *object }.SetShouldDoFullPaintInvalidationWithoutLayoutChange(reason);
    }
    (!roots.is_empty(), leaves)
}
