//! Chromium DisplayItemClient commit adapter.
//!
//! A base module mounted as `layout_commit_package`. Keeping the two hot
//! traversals here gives sampling profiles the same commit owner as their
//! `layout_commit` trace phases without adding LayoutObject dependencies to
//! the `layout_commit` vocabulary crate.

#![allow(non_snake_case)]

use std::collections::HashSet;

use foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient;

use crate::{
    fragment_tree::LayoutResultHandle,
    internal::{
        layout_box_model_object::LayoutBoxModelObject, layout_object::LayoutObject,
        layout_object_builder::LayoutObjectTree, layout_text::LayoutText,
    },
};

/// Commit clients reachable from this document's source tree.
///
/// Source: platform/graphics/paint/paint_controller.cc:127-143. Paint reads
/// the LayoutObject, selection, PaintLayer and scrollable-area clients live
/// (PaintProperties::*ClientState), so validating them here is the whole
/// commit: no fragment mirror of their state remains to be republished. The
/// traversal stays out of line so a sampling profiler assigns its work to
/// `layout_commit_package`.
#[inline(never)]
pub fn commit_paint_client_state(
    tree: &LayoutObjectTree,
    used_clients: &HashSet<u64>,
    _fragments: &mut LayoutResultHandle,
) {
    let mut matched_clients = 0usize;
    let mut object_count = 0usize;
    {
        let mut trace = layout_commit::trace_span(layout_commit::TracePhase::CollectClients);
        let mut commit = |client: &DisplayItemClient| {
            let id = client.Id() as u64;
            if used_clients.contains(&id) {
                client.ValidateForCommittedPaint();
                matched_clients += 1;
            }
        };
        // SVG and table virtual child lists belong to the tree's assembly.
        // Paint commit runs after layout's temporary scope has ended, so use
        // the same persistent factory set while traversing all source clients.
        tree.WithObjectFactoryScope(|| {
            let root = tree.Root() as *const LayoutObject;
            let mut object = root.cast_mut();
            while !object.is_null() {
                object_count += 1;
                // The tree roots each live object. LayoutObject is repr(C),
                // with DisplayItemClient first, matching the source base.
                let current = unsafe { &*object };
                let next = current.NextInPreOrder(root);
                commit(unsafe { &*(object.cast::<DisplayItemClient>()) });
                let layout_text = foundation::DynamicTo::<LayoutText>(object);
                if !layout_text.is_null() {
                    let selection_client = unsafe { &*layout_text }.GetSelectionDisplayItemClient();
                    if let Some(selection_client) = unsafe { selection_client.as_ref() } {
                        commit(selection_client);
                    }
                }
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
                        commit(area.GetScrollingBackgroundDisplayItemClient());
                    }
                }
                // PrePaint owns the per-object paint-flag transition: it clears
                // each visited LayoutObject's flags, saves its previous box
                // geometry and visibility, and re-arms delayed full
                // invalidations after the walk (pre_paint_tree_walk.cc:107,
                // 1584-1585). The commit only validates recorded clients.
                object = next;
            }
        });
        trace.set("objects", object_count as f64);
        trace.set("matched_clients", matched_clients as f64);
    }
}
