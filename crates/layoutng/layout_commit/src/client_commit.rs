//! Chromium DisplayItemClient commit adapter.
//!
//! This source is compiled by LayoutNG at its reciprocal assembly boundary.
//! Keeping the two hot traversals here gives sampling profiles the same
//! `layout_commit_package` owner as their package-owned trace phases without
//! adding concrete LayoutObject dependencies to the standalone crate.

#![allow(non_snake_case)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient;
use foundation::PaintInvalidationReason;

type CommittedClientState = (PaintInvalidationReason, bool, bool);

use crate::{
    fragment_tree::{FragmentPaintData, LayoutResultHandle, ScrollbarPaintData},
    internal::{
        layout_box::LayoutBox, layout_box_model_object::LayoutBoxModelObject,
        layout_object::LayoutObject, layout_object_builder::LayoutObjectTree,
        layout_text::LayoutText,
    },
    layout_engine::PaintFragmentCursor,
};

fn flags_changed(
    id: u64,
    invalidation_reason: Option<PaintInvalidationReason>,
    cacheable: bool,
    just_created: bool,
    states: &HashMap<u64, CommittedClientState>,
) -> bool {
    states.get(&id).is_some_and(|&state| {
        state
            != (
                invalidation_reason.unwrap_or(PaintInvalidationReason::kNone),
                cacheable,
                just_created,
            )
    })
}

fn scrollbars_changed(
    data: &ScrollbarPaintData,
    states: &HashMap<u64, CommittedClientState>,
) -> bool {
    flags_changed(
        data.corner_client_id,
        data.corner_client_invalidation_reason,
        data.corner_client_is_cacheable,
        data.corner_client_is_just_created.get(),
        states,
    ) || [&data.horizontal, &data.vertical]
        .into_iter()
        .flatten()
        .any(|axis| {
            flags_changed(
                axis.display_item_client_id,
                axis.display_item_client_invalidation_reason,
                axis.display_item_client_is_cacheable,
                axis.display_item_client_is_just_created.get(),
                states,
            )
        })
}

fn refresh(node: &mut FragmentPaintData, states: &HashMap<u64, CommittedClientState>) -> usize {
    let mut refreshed_clients = 0;
    let paint = &mut node.paint;
    if let Some(&(reason, cacheable, just_created)) = states.get(&paint.display_item_client_id) {
        paint.display_item_client_invalidation_reason = Some(reason);
        paint.display_item_client_is_cacheable = cacheable;
        paint.display_item_client_is_just_created.set(just_created);
        refreshed_clients += 1;
    }
    if let Some(&(reason, cacheable, just_created)) = states.get(&paint.selection_client_id) {
        paint.selection_client_invalidation_reason = Some(reason);
        paint.selection_client_is_cacheable = cacheable;
        paint.selection_client_is_just_created.set(just_created);
        refreshed_clients += 1;
    }
    if let Some(&(reason, cacheable, just_created)) = states.get(&paint.paint_layer_client_id) {
        paint.paint_layer_client_invalidation_reason = Some(reason);
        paint.paint_layer_client_is_cacheable = cacheable;
        paint.paint_layer_client_is_just_created.set(just_created);
        refreshed_clients += 1;
    }
    if let Some(data) = &mut paint.scrollbars {
        if scrollbars_changed(data, states) {
            let data = Arc::make_mut(data);
            if let Some(&(reason, cacheable, just_created)) = states.get(&data.corner_client_id) {
                data.corner_client_invalidation_reason = Some(reason);
                data.corner_client_is_cacheable = cacheable;
                data.corner_client_is_just_created.set(just_created);
                refreshed_clients += 1;
            }
            for axis in [&mut data.horizontal, &mut data.vertical]
                .into_iter()
                .flatten()
            {
                if let Some(&(reason, cacheable, just_created)) =
                    states.get(&axis.display_item_client_id)
                {
                    axis.display_item_client_invalidation_reason = Some(reason);
                    axis.display_item_client_is_cacheable = cacheable;
                    axis.display_item_client_is_just_created.set(just_created);
                    refreshed_clients += 1;
                }
            }
        }
    }
    if let Some(data) = &mut paint.scroll_container {
        if let Some(&(reason, cacheable, just_created)) =
            states.get(&data.scrolling_background_client_id)
        {
            data.scrolling_background_client_invalidation_reason = Some(reason);
            data.scrolling_background_client_is_cacheable = cacheable;
            data.scrolling_background_client_is_just_created
                .set(just_created);
            refreshed_clients += 1;
        }
    }
    refreshed_clients
}

unsafe fn refresh_clients(
    cursor: PaintFragmentCursor<'_>,
    states: &HashMap<u64, CommittedClientState>,
    fragment_nodes: &mut usize,
    refreshed_clients: &mut usize,
) {
    let children = cursor.Children();
    *fragment_nodes += 1;
    *refreshed_clients += refresh(unsafe { cursor.PaintDataMut() }, states);
    for child in children {
        unsafe { refresh_clients(child, states, fragment_nodes, refreshed_clients) };
    }
}

/// Commit clients reachable from this document's source tree, then publish
/// their new flags into the retained fragment graph.
///
/// IDs in exported snapshots are comparison keys, never pointers to
/// dereference. Both traversals are out of line so a sampling profiler assigns
/// their work to `layout_commit_package`.
#[inline(never)]
pub fn commit_paint_client_state(
    tree: &LayoutObjectTree,
    used_clients: &HashSet<u64>,
    pending_delayed_paint_invalidations: &HashSet<u64>,
    fragments: &mut LayoutResultHandle,
) {
    let mut states = HashMap::new();
    let mut object_count = 0usize;
    {
        let mut trace = layout_commit::trace_span(layout_commit::TracePhase::CollectClients);
        let mut commit = |client: &DisplayItemClient| {
            let id = client.Id() as u64;
            if used_clients.contains(&id) {
                client.ValidateForCommittedPaint();
                states.insert(
                    id,
                    (
                        client.GetPaintInvalidationReason(),
                        client.IsCacheable(),
                        client.IsJustCreated(),
                    ),
                );
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
                // Chromium clears both paint-invalidation flags and the
                // object/descendant paint-property-update bits after PrePaint.
                // The immutable fragment crosses the Layout/Paint boundary,
                // so perform that transition after the artifact commits.
                // cpp: BoxPaintInvalidator::SavePreviousBoxGeometriesIfNeeded.
                // Keep the baseline in its native LayoutBox owner. Resident
                // fragment references already contain the new geometry.
                let layout_box = foundation::DynamicTo::<LayoutBox>(object);
                if !layout_box.is_null() && current.ShouldCheckForPaintInvalidation() {
                    let box_ = unsafe { &mut *layout_box };
                    let style = box_.StyleRef();
                    let save_content = box_.IsLayoutReplaced()
                        || ((style.BackgroundLayers().AnyLayerUsesContentBox()
                            || style.MaskLayers().AnyLayerUsesContentBox())
                            && box_.PhysicalContentBoxRect().size != box_.StitchedSize());
                    let save_overflow = box_.HasVisualOverflow()
                        || box_.HasScrollableOverflow()
                        || ((style.BackgroundLayers().AnyLayerHasLocalAttachmentImage()
                            || box_.GetBackgroundPaintLocation()
                                != crate::internal::layout_object::kBackgroundPaintInBorderBoxSpace
                            || box_.IsDocumentElement())
                            && box_.ScrollableOverflowRect() != box_.PhysicalBorderBoxRect());
                    let mut mutable = box_.GetMutableForPainting();
                    mutable.SavePreviousSize();
                    if save_content {
                        mutable.SavePreviousContentBoxRect();
                    } else {
                        mutable.ClearPreviousContentBoxRect();
                    }
                    if save_overflow {
                        mutable.SavePreviousOverflowData();
                    } else {
                        mutable.ClearPreviousOverflowData();
                    }
                    let view =
                        foundation::DynamicTo::<crate::internal::layout_view::LayoutView>(object);
                    if !view.is_null() {
                        let view = unsafe { &*view };
                        view.SetPreviousBackgroundRect(view.BackgroundRect());
                    }
                }
                let current_mut = unsafe { &mut *object };
                current_mut
                    .GetMutableForPainting()
                    .UpdatePreviousVisibilityVisible();
                current_mut.ClearPaintFlags();
                let object_client = unsafe { &*(object.cast::<DisplayItemClient>()) };
                if pending_delayed_paint_invalidations.contains(&(object_client.Id() as u64)) {
                    // Source: pre_paint_tree_walk.cc:107 and
                    // paint_invalidator.cc:291-294. Chromium clears each
                    // object's current-pass flags during the walk, then
                    // schedules pending delayed targets for the next pass.
                    current_mut
                        .GetMutableForPainting()
                        .SetShouldDelayFullPaintInvalidation();
                }
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
            refresh_clients(
                PaintFragmentCursor::FromHandle(fragments),
                &states,
                &mut fragment_nodes,
                &mut refreshed_clients,
            )
        };
        trace.set("fragment_nodes", fragment_nodes as f64);
        trace.set("refreshed_clients", refreshed_clients as f64);
    }
}
