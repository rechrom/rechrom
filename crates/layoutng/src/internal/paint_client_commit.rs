//! Persistent paint commit at the rooted native layout-tree boundary.
#![allow(non_snake_case)]

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient;

use crate::{
    fragment_tree::{FragmentNode, ScrollbarPaintData},
    internal::{
        layout_box_model_object::LayoutBoxModelObject, layout_object::LayoutObject,
        layout_object_builder::LayoutObjectTree,
    },
};

impl LayoutObjectTree {
    /// Commit only clients reachable from this document's native tree. IDs in
    /// exported snapshots are comparison keys, never pointers to dereference.
    // cpp: platform/graphics/paint/paint_controller.cc:127-143
    // cpp: platform/graphics/paint/paint_chunker.cc:22-28
    pub fn CommitPaintClientState(
        &self,
        used_clients: &HashSet<u64>,
        fragments: &mut Rc<FragmentNode>,
    ) {
        if used_clients.is_empty() {
            return;
        }
        let mut states = HashMap::new();
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
        self.WithObjectFactoryScope(|| {
            let root = self.Root() as *const LayoutObject;
            let mut object = root;
            while !object.is_null() {
                // The tree roots each live object. LayoutObject is repr(C), with
                // DisplayItemClient first, matching the native base subobject.
                let current = unsafe { &*object };
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
                object = current.NextInPreOrder(root) as *const LayoutObject;
            }
        });

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

        fn scrollbars_changed(
            data: &ScrollbarPaintData,
            states: &HashMap<u64, (bool, bool)>,
        ) -> bool {
            flags_changed(
                data.corner_client_id,
                data.corner_client_is_cacheable,
                data.corner_client_is_just_created,
                states,
            ) || [&data.horizontal, &data.vertical]
                .into_iter()
                .flatten()
                .any(|axis| {
                    flags_changed(
                        axis.display_item_client_id,
                        axis.display_item_client_is_cacheable,
                        axis.display_item_client_is_just_created,
                        states,
                    )
                })
        }

        fn changed(node: &FragmentNode, states: &HashMap<u64, (bool, bool)>) -> bool {
            let paint = &node.paint;
            flags_changed(
                paint.display_item_client_id,
                paint.display_item_client_is_cacheable,
                paint.display_item_client_is_just_created,
                states,
            ) || flags_changed(
                paint.paint_layer_client_id,
                paint.paint_layer_client_is_cacheable,
                paint.paint_layer_client_is_just_created,
                states,
            ) || paint
                .scrollbars
                .as_ref()
                .is_some_and(|data| scrollbars_changed(data, states))
                || node.children.iter().any(|child| changed(child, states))
        }

        fn refresh(node: &mut FragmentNode, states: &HashMap<u64, (bool, bool)>) {
            let paint = &mut node.paint;
            if let Some(&(cacheable, just_created)) = states.get(&paint.display_item_client_id) {
                paint.display_item_client_is_cacheable = cacheable;
                paint.display_item_client_is_just_created = just_created;
            }
            if let Some(&(cacheable, just_created)) = states.get(&paint.paint_layer_client_id) {
                paint.paint_layer_client_is_cacheable = cacheable;
                paint.paint_layer_client_is_just_created = just_created;
            }
            if let Some(data) = &mut paint.scrollbars {
                if scrollbars_changed(data, states) {
                    let data = Arc::make_mut(data);
                    if let Some(&(cacheable, just_created)) = states.get(&data.corner_client_id) {
                        data.corner_client_is_cacheable = cacheable;
                        data.corner_client_is_just_created = just_created;
                    }
                    for axis in [&mut data.horizontal, &mut data.vertical]
                        .into_iter()
                        .flatten()
                    {
                        if let Some(&(cacheable, just_created)) =
                            states.get(&axis.display_item_client_id)
                        {
                            axis.display_item_client_is_cacheable = cacheable;
                            axis.display_item_client_is_just_created = just_created;
                        }
                    }
                }
            }
            for child in &mut node.children {
                refresh(child, states);
            }
        }
        // Already validated snapshots need no mutation. In particular, a
        // scroll repaint must retain shared geometry rather than clone it.
        if changed(fragments, &states) {
            refresh(Rc::make_mut(fragments), &states);
        }
    }
}
