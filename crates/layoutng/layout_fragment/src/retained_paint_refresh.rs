//! Fragment-owned refresh of retained paint data after a paint-only input update.
//!
//! Included by the source layout boundary so reciprocal translated types remain
//! available while sampling symbols resolve to `layout_fragment`.

use super::*;

/// Refresh local paint inputs from existing source objects, retaining layout
/// geometry, shaping, physical fragment identities and resource ownership.
/// Special geometry clients retain ordinary export until its proof is available.
#[inline(never)]
pub(super) fn refresh_retained_paint_inputs(
    root: &mut LayoutObject,
    fragments: crate::layout_engine::PaintFragmentCursor<'_>,
    changed: &HashSet<u64>,
    inline_ink_changes: &HashSet<u64>,
) -> bool {
    let mut sources = SourceMap::default();
    let mut style_sources = StyleSourceMap::default();
    let mut clients = HashMap::new();
    let mut objects = Vec::new();
    let root_ptr: *mut LayoutObject = root;
    let mut object = root_ptr;
    let mut logical_order = 0;
    while !object.is_null() {
        objects.push(object);
        let current = unsafe { &*object };
        let node = current.GetNode();
        if !node.is_null() {
            logical_order += 1;
            sources.insert(
                object as *const _,
                SourceEntry {
                    node,
                    logical_tree_order: logical_order,
                },
            );
            style_sources.insert(
                current.StyleRef() as *const _,
                StyleSourceEntry {
                    resources: unsafe { &*node }.InputStyleResources(),
                },
            );
            if changed.contains(&unsafe { &*node }.InputId()) {
                clients.insert(
                    unsafe { &*(object as *const DisplayItemClient) }.Id() as u64,
                    object,
                );
            }
        }
        object = current.NextInPreOrder(root_ptr);
    }
    // FragmentItem::RecalcInkOverflow keeps shaping resident when decorations
    // change. Invalidate the existing items, then use the same boundary ink
    // export as ordinary layout. Unsupported decoration overflow remains
    // explicitly unset; never publish the old plain-text rect as current ink.
    let mut ink_items: HashMap<(u64, u32), *const FragmentItem> = HashMap::new();
    let mut ink_clients = HashSet::new();
    for &object in &objects {
        if inline_ink_changes.contains(&InputId(unsafe { &*object })) {
            let mut ancestor = object;
            while !ancestor.is_null() {
                ink_clients.insert(unsafe { &*(ancestor as *const DisplayItemClient) }.Id() as u64);
                ancestor = unsafe { &*ancestor }.Parent();
            }
        }
        let box_ = DynamicTo::<LayoutBox>(object);
        let Some(box_) = (unsafe { box_.as_ref() }) else {
            continue;
        };
        for index in 0..box_.PhysicalFragmentCount() {
            let fragment = box_.GetPhysicalFragment(index);
            let Some(fragment) = (unsafe { fragment.as_ref() }) else {
                continue;
            };
            let Some(items) = (unsafe { fragment.Items().as_ref() }) else {
                continue;
            };
            for item in items.Items() {
                let source = item.GetLayoutObject();
                if source.is_null()
                    || !item.IsText()
                    || !inline_ink_changes.contains(&InputId(unsafe { &*source }))
                {
                    continue;
                }
                let client = unsafe { &*(source as *const DisplayItemClient) }.Id() as u64;
                let key = (client, item.FragmentId());
                if ink_items
                    .insert(key, item)
                    .is_some_and(|old| !std::ptr::eq(old, item))
                {
                    return false;
                }
            }
        }
    }
    for &item in ink_items.values() {
        unsafe { &*item }.InvalidateInkOverflow();
    }
    fn validate(
        node: crate::layout_engine::PaintFragmentCursor<'_>,
        clients: &HashMap<u64, *mut LayoutObject>,
    ) -> bool {
        if clients.contains_key(&node.paint.display_item_client_id)
            && (node.paint.has_collapsed_borders
                || node.paint.svg_text.is_some()
                || node.paint.text_combine.is_some())
        {
            return false;
        }
        node.Children()
            .into_iter()
            .all(|child| validate(child, clients))
    }
    if !validate(fragments, &clients) {
        return false;
    }
    unsafe fn refresh(
        cursor: crate::layout_engine::PaintFragmentCursor<'_>,
        clients: &HashMap<u64, *mut LayoutObject>,
        sources: &SourceMap,
        styles: &StyleSourceMap,
        ink_clients: &HashSet<u64>,
        ink_items: &HashMap<(u64, u32), *const FragmentItem>,
    ) {
        let children = cursor.Children();
        let node = unsafe { cursor.PaintDataMut() };
        node.paint.background_transfers_to_view = false;
        node.paint.canvas_background = None;
        node.paint.view_background_invalidation = None;
        if ink_clients.contains(&node.paint.display_item_client_id) {
            // Ancestor contents ink depends on the invalidated text item.
            // Keep that proof unavailable until the source overflow owner
            // completes it, as ExportBoxInkOverflow does for unset ink.
            node.paint.contents_ink_overflow = None;
            node.paint.ink_overflow = None;
        }
        if let Some(&object) = clients.get(&node.paint.display_item_client_id) {
            let resources = node
                .paint
                .resources
                .as_ref()
                .expect("completed fragment has resources");
            let current =
                ExportPaintProperties(object, sources, styles, std::ptr::null(), resources);
            node.paint.BindSourceObject(object);
            node.paint.pre_paint_invalidation = current.pre_paint_invalidation;
            node.paint.style = current.style;
            node.paint.form_control = current.form_control;
            node.paint.display_item_client_is_cacheable = current.display_item_client_is_cacheable;
            node.paint.display_item_client_invalidation_reason =
                current.display_item_client_invalidation_reason;
            // FragmentData identity is owned by this retained physical
            // fragment. ExportPaintProperties has no physical-fragment
            // context and therefore reports the LayoutObject's first
            // FragmentData; do not collapse fragmented boxes back to it.
            node.paint
                .svg_descendant_may_have_transform_related_operations =
                current.svg_descendant_may_have_transform_related_operations;
            node.paint.svg_view_box_is_empty = current.svg_view_box_is_empty;
            node.paint.display_item_client_debug_name =
                current.display_item_client_debug_name.clone();
            node.paint.display_item_client_owner_node_id =
                current.display_item_client_owner_node_id;
            node.paint
                .display_item_client_is_just_created
                .set(current.display_item_client_is_just_created.get());
            node.paint.selection_client_id = current.selection_client_id;
            node.paint.selection_client_invalidation_reason =
                current.selection_client_invalidation_reason;
            node.paint.selection_client_is_cacheable = current.selection_client_is_cacheable;
            node.paint
                .selection_client_is_just_created
                .set(current.selection_client_is_just_created.get());
            node.paint.display_item_raster_effect_outset =
                current.display_item_raster_effect_outset;
            node.paint.paint_layer_client_id = current.paint_layer_client_id;
            node.paint.paint_layer_client_invalidation_reason =
                current.paint_layer_client_invalidation_reason;
            node.paint.paint_layer_client_is_cacheable = current.paint_layer_client_is_cacheable;
            node.paint
                .paint_layer_client_is_just_created
                .set(current.paint_layer_client_is_just_created.get());
            node.paint.paint_layer_first_fragment_visual_rect =
                current.paint_layer_first_fragment_visual_rect;
            node.paint.paint_layer_contents_visual_rect = current.paint_layer_contents_visual_rect;
            node.paint.paint_layer_filter_reference_box = current.paint_layer_filter_reference_box;
            node.paint.paint_layer_filter_viewport = current.paint_layer_filter_viewport;
            node.paint.paint_layer_is_in_top_or_view_transition_layer =
                current.paint_layer_is_in_top_or_view_transition_layer;
            node.paint.paint_layer_is_direct_layout_view_child =
                current.paint_layer_is_direct_layout_view_child;
            node.paint.paint_layer_can_contain_absolute_position_objects =
                current.paint_layer_can_contain_absolute_position_objects;
            node.paint.paint_layer_can_contain_fixed_position_objects =
                current.paint_layer_can_contain_fixed_position_objects;
            node.paint.paint_layer_has_overlay_overflow_controls =
                current.paint_layer_has_overlay_overflow_controls;
            node.paint.paint_layer_order_parent_unique_id =
                current.paint_layer_order_parent_unique_id;
            node.paint.paint_layer_order = current.paint_layer_order;
            node.paint.background_paint_location = current.background_paint_location;
            node.paint.background_fill_layer_count = current.background_fill_layer_count;
            node.paint.has_background = current.has_background;
            node.paint.background_should_always_be_clipped =
                current.background_should_always_be_clipped;
            node.paint.background_any_local_attachment_image =
                current.background_any_local_attachment_image;
            node.paint.background_any_default_attachment_image =
                current.background_any_default_attachment_image;
            node.paint.background_any_layer_uses_content_box =
                current.background_any_layer_uses_content_box;
            node.paint.scrollable_overflow = current.scrollable_overflow;
            node.paint.view_background_rect = current.view_background_rect;
            node.paint.focused_image_map_area = current.focused_image_map_area;
            node.paint.image_decoding_mode = current.image_decoding_mode;
        }
        if let Some(&item) = ink_items.get(&(
            node.paint.display_item_client_id,
            node.paint.display_item_fragment,
        )) {
            node.paint.self_ink_overflow = None;
            node.paint.ink_overflow = None;
            ExportInlineInkOverflow(unsafe { &*item }, &mut node.paint);
        }
        for child in children {
            unsafe { refresh(child, clients, sources, styles, ink_clients, ink_items) };
        }
    }
    unsafe {
        refresh(
            fragments,
            &clients,
            &sources,
            &style_sources,
            &ink_clients,
            &ink_items,
        )
    };
    ExportCanvasBackground(root, &objects, fragments);
    ExportGlobalMainThreadScrollFacts(root, &objects, fragments);
    true
}
