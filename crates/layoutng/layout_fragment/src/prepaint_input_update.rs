//! Chromium retained-fragment and PrePaint input mutation.
//!
//! Paint-style replacement, mutable fragment traversal, and flat PrePaint
//! input collection are fragment lifecycle work with separate sampling frames.

#![allow(non_snake_case)]

use super::*;

#[inline(never)]
pub(super) fn UpdateFragmentPaintStyle(
    fragment: &mut FragmentPaintData,
    node_id: u64,
    style: &crate::internal::layout_input::PaintStyle,
    found: &mut bool,
) {
    if fragment.node_id == node_id {
        let mut current = style.clone();
        if let Some(clip) = current
            .clip_path
            .as_mut()
            .filter(|clip| clip.reference_target_id.is_some())
        {
            let Some(previous) = fragment.paint.style.clip_path.as_ref().filter(|previous| {
                previous.reference_target_id == clip.reference_target_id
                    && previous.reference_scope == clip.reference_scope
            }) else {
                // A newly selected resource needs ordinary fragment-local
                // Paint-data population to establish its own viewport.
                return;
            };
            clip.reference_viewport = previous.reference_viewport;
        }
        for reference in &mut current.svg_mask_bindings {
            if reference.target_id.is_none() {
                continue;
            }
            let Some(previous) = fragment
                .paint
                .style
                .svg_mask_bindings
                .iter()
                .filter(|p| {
                    p.target_id == reference.target_id
                        && p.scope == reference.scope
                        && p.id == reference.id
                })
                .next()
            else {
                return;
            };
            reference.viewport = previous.viewport;
        }
        fragment.paint.style = current;
        *found = true;
    }
}

#[inline(never)]
pub(super) fn VisitMutablePaintInputs(
    fragments: &LayoutResultHandle,
    objects: &LayoutObjectFactorySet,
    visitor: &mut impl FnMut(&mut FragmentPaintData),
) {
    // Physical fragments retain their source LayoutObject owners. Rust's
    // explicit virtual dispatch needs the same factory table while walking
    // those owners that Chromium's C++ vtables provide unconditionally.
    let _object_factory_scope = LayoutObjectFactoryScope::new(objects);
    unsafe fn visit(
        cursor: PaintFragmentCursor<'_>,
        visitor: &mut impl FnMut(&mut FragmentPaintData),
    ) {
        let children = cursor.Children();
        visitor(unsafe { cursor.PaintDataMut() });
        for child in children {
            unsafe { visit(child, visitor) };
        }
    }
    unsafe { visit(PaintFragmentCursor::FromHandle(fragments), visitor) };
}

#[inline(never)]
pub(super) fn UpdatePrePaintInputs(
    tracker: &mut PrePaintRevisionTracker,
    fragments: &LayoutResultHandle,
    objects: &LayoutObjectFactorySet,
) {
    // Keep source child traversal inside the engine's virtual-dispatch scope.
    // Certify each canonical sidecar in place. Do not flatten the source
    // topology into another document-sized Vec first.
    let _object_factory_scope = LayoutObjectFactoryScope::new(objects);
    fn certify(cursor: PaintFragmentCursor<'_>, tracker: &mut PrePaintRevisionTracker) {
        tracker.ObserveProductionNode(unsafe { cursor.PaintDataMut() });
        for child in cursor.Children() {
            certify(child, tracker);
        }
    }
    tracker.BeginProductionUpdate();
    certify(PaintFragmentCursor::FromHandle(fragments), tracker);
    tracker.FinishProductionUpdate();
}
