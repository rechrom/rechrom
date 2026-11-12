//! Chromium input-triggered invalidation for the resident LayoutObject tree.
//!
//! Included by `layout_object_builder` so the standalone package remains free
//! of concrete source object types while the traversals keep invalidation
//! sampling and trace ownership.

use super::*;

#[inline(always)]
fn is_document_style_image(image: *mut layoutng_style::style::style_image::StyleImage) -> bool {
    !image.is_null()
        && layoutng_style::style::style_fetched_image::FetchedImage(unsafe { &*image })
            .is_some_and(|image| image.IsDocumentImage())
}

#[inline(always)]
fn fill_layers_use_document_image(first: &layoutng_style::style::fill_layer::FillLayer) -> bool {
    let mut layer = first as *const layoutng_style::style::fill_layer::FillLayer;
    while !layer.is_null() {
        let current = unsafe { &*layer };
        if is_document_style_image(current.GetImage()) {
            return true;
        }
        layer = current.Next();
    }
    false
}

/// Notify the resident clients whose document-image rendering environment
/// changed without changing their CSS style or geometry.
///
/// Source: core/layout/layout_box_model_object.cc:479-493;
///         core/layout/layout_box.cc:2173-2227;
///         core/layout/layout_image.cc:139-194,281-289;
///         core/layout/layout_inline.cc:857-866.
///
/// Chromium reaches these branches through ImageResourceObserver::ImageChanged.
/// The extracted resource owner publishes immutable image catalogs instead, so
/// the host/constraint boundary delivers the same notification before retained
/// Paint inputs are republished.
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
        let style = current.StyleRef();
        let background = fill_layers_use_document_image(style.BackgroundLayers());
        let mask = fill_layers_use_document_image(style.MaskLayers());
        let border = is_document_style_image(style.BorderImage().GetImage());
        let mask_box = is_document_style_image(style.MaskBoxImage().GetImage());
        let reflection = {
            let reflect = style.BoxReflect();
            !reflect.is_null() && is_document_style_image(unsafe { &*reflect }.Mask().GetImage())
        };
        let node = current.GetNode();
        let replaced = current.IsLayoutReplaced()
            && !node.is_null()
            && unsafe { &*node }
                .InputStyleResources()
                .image_resource_id
                .is_some_and(|id| document_ids.contains(&id));

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
        invalidated += usize::from(changed);
        object = next;
    }
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
