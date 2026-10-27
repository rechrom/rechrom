//! Native retained-fragment and PrePaint input mutation.
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
        for (index, layer) in current.mask_images.iter_mut().enumerate() {
            let Some(reference) = layer
                .svg_reference
                .as_mut()
                .filter(|r| r.target_id.is_some())
            else {
                continue;
            };
            let Some(previous) = fragment
                .paint
                .style
                .mask_images
                .get(index)
                .and_then(|l| l.svg_reference.as_ref())
                .filter(|p| {
                    p.target_id == reference.target_id
                        && p.scope == reference.scope
                        && p.id == reference.id
                })
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
pub(super) fn VisitNativePaintInputsMut(
    fragments: &LayoutResultHandle,
    objects: &LayoutObjectFactorySet,
    visitor: &mut impl FnMut(&mut FragmentPaintData),
) {
    // Physical fragments retain their native LayoutObject owners. Rust's
    // explicit virtual dispatch needs the same factory table while walking
    // those owners that Chromium's C++ vtables provide unconditionally.
    let _object_factory_scope = LayoutObjectFactoryScope::new(objects);
    unsafe fn visit(
        cursor: NativePaintFragmentCursor<'_>,
        visitor: &mut impl FnMut(&mut FragmentPaintData),
    ) {
        let children = cursor.Children();
        visitor(unsafe { cursor.PaintDataMut() });
        for child in children {
            unsafe { visit(child, visitor) };
        }
    }
    unsafe { visit(NativePaintFragmentCursor::FromHandle(fragments), visitor) };
}

#[inline(never)]
pub(super) fn UpdateNativePrePaintInputs(
    tracker: &mut PrePaintRevisionTracker,
    fragments: &LayoutResultHandle,
    objects: &LayoutObjectFactorySet,
) {
    // Keep native child traversal inside the engine's virtual-dispatch scope.
    // This is a direct pointer walk; it does not construct a fragment copy.
    let _object_factory_scope = LayoutObjectFactoryScope::new(objects);
    fn collect(
        cursor: NativePaintFragmentCursor<'_>,
        parent: Option<usize>,
        output: &mut Vec<FlatPrePaintInput>,
    ) {
        let index = output.len();
        output.push(FlatPrePaintInput {
            node: cursor.PaintData() as *const FragmentPaintData as *mut FragmentPaintData,
            parent,
        });
        for child in cursor.Children() {
            collect(child, Some(index), output);
        }
    }
    let mut inputs = Vec::new();
    collect(
        NativePaintFragmentCursor::FromHandle(fragments),
        None,
        &mut inputs,
    );
    tracker.update_flat(&mut inputs);
}
