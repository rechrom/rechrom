//! Native DisplayItemClient commit adapter.
//!
//! This source is compiled by LayoutNG at its reciprocal assembly boundary.
//! Keeping the two hot traversals here gives sampling profiles the same
//! `layout_commit_package` owner as their package-owned trace phases without
//! adding concrete LayoutObject dependencies to the standalone crate.

#![allow(non_snake_case)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient;

use crate::{
    fragment_tree::{FragmentPaintData, LayoutResultHandle, ScrollbarPaintData},
    internal::{
        layout_box_model_object::LayoutBoxModelObject, layout_object::LayoutObject,
        layout_object_builder::LayoutObjectTree,
    },
    layout_engine::NativePaintFragmentCursor,
};

fn flags_changed(
    id: u64,
    cacheable: bool,
    just_created: bool,
    states: &HashMap<u64, (bool, bool)>,
) -> bool {
    states
        .get(&id)
        .is_some_and(|&state| state != (cacheable, just_created))
}

fn scrollbars_changed(data: &ScrollbarPaintData, states: &HashMap<u64, (bool, bool)>) -> bool {
    flags_changed(
        data.corner_client_id,
        data.corner_client_is_cacheable,
        data.corner_client_is_just_created.get(),
        states,
    ) || [&data.horizontal, &data.vertical]
        .into_iter()
        .flatten()
        .any(|axis| {
            flags_changed(
                axis.display_item_client_id,
                axis.display_item_client_is_cacheable,
                axis.display_item_client_is_just_created.get(),
                states,
            )
        })
}

fn refresh(node: &mut FragmentPaintData, states: &HashMap<u64, (bool, bool)>) -> usize {
    let mut refreshed_clients = 0;
    let paint = &mut node.paint;
    if let Some(&(cacheable, just_created)) = states.get(&paint.display_item_client_id) {
        paint.display_item_client_is_cacheable = cacheable;
        paint.display_item_client_is_just_created.set(just_created);
        refreshed_clients += 1;
    }
    if let Some(&(cacheable, just_created)) = states.get(&paint.paint_layer_client_id) {
        paint.paint_layer_client_is_cacheable = cacheable;
        paint.paint_layer_client_is_just_created.set(just_created);
        refreshed_clients += 1;
    }
    if let Some(data) = &mut paint.scrollbars {
        if scrollbars_changed(data, states) {
            let data = Arc::make_mut(data);
            if let Some(&(cacheable, just_created)) = states.get(&data.corner_client_id) {
                data.corner_client_is_cacheable = cacheable;
                data.corner_client_is_just_created.set(just_created);
                refreshed_clients += 1;
            }
            for axis in [&mut data.horizontal, &mut data.vertical]
                .into_iter()
                .flatten()
            {
                if let Some(&(cacheable, just_created)) = states.get(&axis.display_item_client_id) {
                    axis.display_item_client_is_cacheable = cacheable;
                    axis.display_item_client_is_just_created.set(just_created);
                    refreshed_clients += 1;
                }
            }
        }
    }
    refreshed_clients
}

unsafe fn refresh_native(
    cursor: NativePaintFragmentCursor<'_>,
    states: &HashMap<u64, (bool, bool)>,
    fragment_nodes: &mut usize,
    refreshed_clients: &mut usize,
) {
    let children = cursor.Children();
    *fragment_nodes += 1;
    *refreshed_clients += refresh(unsafe { cursor.PaintDataMut() }, states);
    for child in children {
        unsafe { refresh_native(child, states, fragment_nodes, refreshed_clients) };
    }
}

/// Commit clients reachable from this document's native tree, then publish
/// their new flags into the retained fragment graph.
///
/// IDs in exported snapshots are comparison keys, never pointers to
/// dereference. Both traversals are out of line so a sampling profiler assigns
/// their work to `layout_commit_package`.
#[inline(never)]
pub fn commit_paint_client_state(
    tree: &LayoutObjectTree,
    used_clients: &HashSet<u64>,
    fragments: &mut LayoutResultHandle,
) {
    let mut states = HashMap::new();
    let mut object_count = 0usize;
    {
        let mut trace = layout_commit::trace_span(layout_commit::TracePhase::CollectNativeClients);
        let mut commit = |client: &DisplayItemClient| {
            let id = client.Id() as u64;
            if used_clients.contains(&id) {
                client.ValidateForCommittedPaint();
                states.insert(id, (client.IsCacheable(), client.IsJustCreated()));
            }
        };
        // SVG and table virtual child lists belong to the tree's assembly.
        // Paint commit runs after layout's temporary scope has ended, so use
        // the same persistent factory set while traversing all native clients.
        tree.WithObjectFactoryScope(|| {
            let root = tree.Root() as *const LayoutObject;
            let mut object = root.cast_mut();
            while !object.is_null() {
                object_count += 1;
                // The tree roots each live object. LayoutObject is repr(C),
                // with DisplayItemClient first, matching the native base.
                let current = unsafe { &*object };
                let next = current.NextInPreOrder(root);
                commit(unsafe { &*(object.cast::<DisplayItemClient>()) });
                let box_model = foundation::DynamicTo::<LayoutBoxModelObject>(object);
                if !box_model.is_null() {
                    let box_model = unsafe { &*box_model };
                    let layer = box_model.Layer();
                    if !layer.is_null() {
                        commit(unsafe { &*layer }.DisplayItemClient());
                    }
                    let area = box_model.GetScrollableArea();
                    if !area.is_null() {
                        let area = unsafe { &*area };
                        for scrollbar in [area.HorizontalScrollbar(), area.VerticalScrollbar()] {
                            if !scrollbar.is_null() {
                                commit(unsafe { &*scrollbar }.DisplayItemClient());
                            }
                        }
                        if let Some(client) = area.ScrollCornerDisplayItemClient() {
                            commit(client);
                        }
                    }
                }
                // Chromium clears both paint-invalidation flags and the
                // object/descendant paint-property-update bits after PrePaint.
                // The immutable fragment crosses the Layout/Paint boundary,
                // so perform that transition after the artifact commits.
                unsafe { &mut *object }.ClearPaintFlags();
                object = next;
            }
        });
        trace.set("objects", object_count as f64);
        trace.set("matched_clients", states.len() as f64);
    }

    let mut fragment_nodes = 0usize;
    let mut refreshed_clients = 0usize;
    {
        let mut trace =
            layout_commit::trace_span(layout_commit::TracePhase::RefreshFragmentClients);
        unsafe {
            refresh_native(
                NativePaintFragmentCursor::FromHandle(fragments),
                &states,
                &mut fragment_nodes,
                &mut refreshed_clients,
            )
        };
        trace.set("fragment_nodes", fragment_nodes as f64);
        trace.set("refreshed_clients", refreshed_clients as f64);
    }
}
