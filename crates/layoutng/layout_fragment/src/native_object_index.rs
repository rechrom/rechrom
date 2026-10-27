//! Native object-index adapter for fragment publication.
//!
//! This file is compiled by LayoutNG at the reciprocal assembly boundary. It
//! keeps the document-order and uncullable-ancestor traversals under a
//! fragment-qualified sampling module without adding LayoutObject dependencies
//! to the standalone `layout_fragment` crate.

use std::sync::Arc;

use crate::internal::layout_object::LayoutObject;
use crate::layout_boundary_support::{
    LayoutBoundaryCache, LayoutBoundaryHashSet, LayoutBoundarySourceEntry,
};

/// Mark every ancestor whose inline paint must not be rejected by a local
/// cull rect. Kept out of line because pathological out-of-flow trees can make
/// this walk visible in sampling profiles.
#[inline(never)]
pub(crate) fn mark_uncullable_paint_ancestors(
    objects: &[*mut LayoutObject],
) -> LayoutBoundaryHashSet<*const LayoutObject> {
    let mut result = LayoutBoundaryHashSet::default();
    for &object in objects {
        let current = unsafe { &*object };
        let style = current.StyleRef();
        let escapes_normal_flow = current.IsFixedPositioned()
            || current.IsStickyPositioned()
            || current.IsOutOfFlowPositioned()
            || current.IsSVG()
            || current.HasTransformRelatedProperty()
            || current.HasPerspective()
            || style.Filter().HasFilterThatMovesPixels()
            || style.HasBackdropFilter();
        if !escapes_normal_flow {
            continue;
        }
        let mut ancestor = object;
        while !ancestor.is_null() {
            if !result.insert(ancestor as *const LayoutObject) {
                break;
            }
            ancestor = unsafe { &*ancestor }.Parent();
        }
    }
    result
}

/// Rebuild stable document-order indices used by fragment paint publication.
/// The core boundary retains only an ABI-shaped forwarding function.
#[inline(never)]
pub(crate) fn ensure_layout_boundary_object_index(
    root: &mut LayoutObject,
    cache: &mut LayoutBoundaryCache,
) {
    if !cache.objects.is_empty() {
        return;
    }

    let mut logical_order = 0;
    let mut source_objects = Vec::new();
    let mut sticky_objects = Vec::new();
    let root_ptr: *mut LayoutObject = root;
    let mut object = root_ptr;
    while !object.is_null() {
        // This runs inside LayoutHeapScope; the resident tree owns the objects.
        // Topology mutations clear every cached pointer before rebuilding.
        cache.objects.push(object);
        let current = unsafe { &*object };
        let node = current.GetNode();
        if !node.is_null() {
            source_objects.push(object);
            let input_node = unsafe { &*node };
            logical_order += 1;
            cache
                .sources
                .entry(object)
                .or_insert(LayoutBoundarySourceEntry {
                    node,
                    logical_tree_order: logical_order,
                });
            cache
                .style_sources
                .entry(current.StyleRef())
                .or_insert(input_node.InputStyle());
            let first_line_style = input_node.InputFirstLineStyle();
            if !first_line_style.is_null() {
                if let Some(first_line_data) = input_node.InputFirstLineStyleData() {
                    cache
                        .style_sources
                        .entry(first_line_style)
                        .or_insert(first_line_data.as_ref());
                }
            }
        }
        if current.StyleRef().HasStickyConstrainedPosition() {
            sticky_objects.push(object);
        }
        object = current.NextInPreOrder(root_ptr);
    }
    cache.source_objects = Arc::new(source_objects);
    cache.sticky_objects = Arc::new(sticky_objects);
    cache.uncullable_objects = Arc::new(mark_uncullable_paint_ancestors(&cache.objects));
}
