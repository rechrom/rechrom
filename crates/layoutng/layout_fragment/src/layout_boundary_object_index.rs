//! Chromium object-index adapter for fragment publication.
//!
//! This file is compiled by LayoutNG at the reciprocal assembly boundary. It
//! keeps the document-order traversal under a fragment-qualified sampling
//! module without adding LayoutObject dependencies to the standalone
//! `layout_fragment` crate.

use std::sync::Arc;

use crate::internal::layout_object::LayoutObject;
use crate::layout_boundary_support::{
    LayoutBoundaryCache, LayoutBoundarySourceEntry, LayoutBoundaryStyleSourceEntry,
};

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
            cache.style_sources.entry(current.StyleRef()).or_insert(
                LayoutBoundaryStyleSourceEntry {
                    resources: input_node.InputStyleResources(),
                },
            );
            let first_line_style = input_node.InputFirstLineStyle();
            if !first_line_style.is_null() {
                cache.style_sources.entry(first_line_style).or_insert(
                    LayoutBoundaryStyleSourceEntry {
                        resources: input_node
                            .InputFirstLineStyleResources()
                            .as_deref()
                            .unwrap_or_else(|| input_node.InputStyleResources()),
                    },
                );
            }
        }
        if current.StyleRef().HasStickyConstrainedPosition() {
            sticky_objects.push(object);
        }
        object = current.NextInPreOrder(root_ptr);
    }
    cache.source_objects = Arc::new(source_objects);
    cache.sticky_objects = Arc::new(sticky_objects);
}
