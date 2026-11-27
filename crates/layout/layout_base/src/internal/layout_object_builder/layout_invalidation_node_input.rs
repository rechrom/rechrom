//! Targeted source node-input update transaction.
//!
//! The public LayoutObjectTree entry remains in the translated core API, while
//! validation, inherited-text expansion, and mutation are owned here so the
//! existing `layout_invalidation.NodeInput` interval and sampling stack agree.

use super::*;

#[inline(never)]
pub(super) fn update_node_input(
    tree: &mut LayoutObjectTree,
    id: u64,
    style: &ComputedStyle,
    element: Option<ElementData>,
) -> Option<NodeInputUpdate> {
    let _heap = LayoutHeapScope::new();
    let storage = unsafe { &mut *tree.storage_ };
    assert!(!storage.updating);
    let keys = storage.InputKeys(id).to_vec();
    if keys.is_empty() {
        return None;
    }
    // Sub-phase attribution of this targeted SetStyle transaction: input
    // admission (validation, no mutation), then in-place application.
    let admission_trace =
        browser_tracing::span("layout", "layout_invalidation.NodeInput.Admission");
    // cpp: core/layout/layout_object.cc:2991-2995 (LayoutObject::SetStyle).
    // Element::RecalcOwnStyle hands every LayoutObject of the element the
    // same ComputedStyle, and Text::RecalcTextStyle gives its LayoutText the
    // parent's style; neither copies it. Build this restyle's envelope and
    // its resource projection once and share them with every retained input.
    let shared_style = Arc::new(style.clone());
    let shared_resources =
        Arc::new(crate::internal::layout_input::StyleResourceData::FromLegacyInput(&shared_style));
    let mut inputs = Vec::new();
    let mut relations_changed = false;
    for key in &keys {
        let record = &storage.managed[key];
        if record.input.style_generated
            || record.input.first_line_style.is_some()
            || unsafe { &*record.object }.IsSVG()
            || unsafe { &*record.object }.WasDestroyedForTreeUpdate()
            || unsafe { &*record.node }.InputHasFirstLineStyles()
            || record
                .input
                .element
                .as_ref()
                .is_some_and(|e| e.form_control_type.is_some())
        {
            return None;
        }
        // The retained descriptor's scalar and text fields carry over; its
        // style envelope, resources and element data are replaced below, so
        // they are not copied (first-line inputs were declined above).
        let mut input = NodeConstructionData {
            id: record.input.id,
            kind: record.input.kind,
            style_generated: record.input.style_generated,
            computed_style: style.style,
            style: Arc::clone(&shared_style),
            resources: Arc::clone(&shared_resources),
            text: record.input.text.clone(),
            element: None,
            // CreateWithStyleOwner keeps the existing InputKey, whose name is
            // the retained debug name; Create reads the parameter only to
            // mint a new key, so copying the string here is dead work.
            debug_name: String::new(),
            first_line_computed_style: record.input.first_line_computed_style,
            first_line_style: None,
            first_line_resources: None,
        };
        // Resource bindings remain with their owner. A resource completion may
        // refresh this exact input; SVG graph ownership remains resident.
        // Only a retained graph that differs from the restyle's needs an
        // envelope of its own.
        let retained_filter_graphs = &record.effective_style.paint.svg_filter_graphs;
        let shared_filter_graphs = &shared_style.paint.svg_filter_graphs;
        if Arc::ptr_eq(retained_filter_graphs, shared_filter_graphs)
            || (retained_filter_graphs.is_empty() && shared_filter_graphs.is_empty())
        {
            input.style = Arc::clone(&shared_style);
            input.resources = Arc::clone(&shared_resources);
        } else {
            let mut own_style = (*shared_style).clone();
            own_style.paint.svg_filter_graphs = Arc::clone(retained_filter_graphs);
            input.style = Arc::new(own_style);
            input.resources = Arc::new(
                crate::internal::layout_input::StyleResourceData::FromLegacyInput(&input.style),
            );
        }
        input.element = if input.kind == NodeKind::kText {
            None
        } else {
            element.clone()
        };
        if !LayoutObjectTreeStorage::StyleUpdateCompatible(
            record,
            record.parent,
            &input,
            &input.style,
        ) {
            return None;
        }
        let image_resource_binding_changed = unsafe {
            match (
                record.effective_computed_style.as_ref(),
                style.style.as_ref(),
            ) {
                (Some(old), Some(new)) => {
                    layout_fragment_paint_input::StyleImageResourceBindingsChanged(old, new)
                }
                _ => false,
            }
        };
        if image_resource_binding_changed {
            unsafe { &mut *record.object }.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                foundation::PaintInvalidationReason::kImage,
            );
        }
        relations_changed |= record.input.element != input.element
            || record
                .input
                .element
                .as_ref()
                .is_some_and(|data| data.document_role != DocumentRole::kNone)
            || input
                .element
                .as_ref()
                .is_some_and(|data| data.document_role != DocumentRole::kNone);
        inputs.push((
            key.clone(),
            record.parent,
            input,
            NodeInputObjectUpdate {
                object: record.object,
            },
        ));
    }
    // Source: core/dom/text.cc Text::RecalcTextStyle. A resident LayoutText is
    // restyled from its style parent even when StyleForText returned a distinct
    // inherited-equal ComputedStyle allocation. Use tree ownership, not address
    // identity between the old snapshots.
    let parents = keys
        .iter()
        .map(|key| storage.managed[key].object)
        .collect::<Vec<_>>();
    for parent in parents {
        for key in storage.ChildKeys(parent) {
            let record = &storage.managed[key];
            if record.input.kind == NodeKind::kText && !keys.contains(key) {
                if record.input.style_generated || record.input.first_line_style.is_some() {
                    return None;
                }
                let mut input = record.input.clone();
                input.style = Arc::clone(&shared_style);
                input.computed_style = style.style;
                input.resources = Arc::clone(&shared_resources);
                inputs.push((
                    key.clone(),
                    record.parent,
                    input,
                    NodeInputObjectUpdate {
                        object: record.object,
                    },
                ));
            }
        }
    }
    drop(admission_trace);
    let _apply_trace = browser_tracing::span("layout", "layout_invalidation.NodeInput.Apply");
    // Input comparisons and validation finish before any mutation. No sibling
    // topology or partial document transaction is exposed here.
    let mut objects = Vec::with_capacity(inputs.len());
    for (key, parent, input, object) in inputs {
        objects.push(object);
        let record = storage.managed.remove(&key).unwrap();
        unsafe { &mut *record.node }.UnsealForTreeUpdate();
        storage.pending.insert(key.clone(), record);
        let text_style = if input.kind == NodeKind::kText {
            Arc::as_ptr(&input.style)
        } else {
            std::ptr::null()
        };
        storage.CreateWithStyleOwner(
            parent,
            input.id,
            input.kind,
            &input.style,
            input.element,
            input.debug_name,
            input.text,
            text_style,
            key,
            (&input.style, &input.resources),
        );
    }
    if !relations_changed {
        for change in &objects {
            let node = unsafe { &*change.object }.GetNode();
            if !node.is_null() {
                unsafe { &mut *node }.seal_input_tree();
            }
        }
    }
    Some(NodeInputUpdate {
        objects,
        relations_changed,
    })
}

/// Source HTMLMediaElement::UpdateLayoutObject scalar admission. Keep the
/// retained input descriptor in sync with its original metadata node, so the
/// next ordinary tree reconciliation cannot rediscover this same update as an
/// unrelated geometry change. This uses the existing per-node attachment index.
pub(super) fn update_video_layout_state(
    tree: &mut LayoutObjectTree,
    id: u64,
    state: super::super::layout_input::VideoLayoutState,
) -> Option<*mut LayoutObject> {
    let storage = unsafe { &mut *tree.storage_ };
    assert!(!storage.updating);
    let keys = storage.InputKeys(id).to_vec();
    let mut result = None;
    for key in keys {
        let record = storage
            .managed
            .get_mut(&key)
            .expect("native attachment key");
        if record.input.style_generated
            || unsafe { &*record.object }.WasDestroyedForTreeUpdate()
            || !unsafe { &*record.object }.IsVideo()
        {
            continue;
        }
        assert!(
            result.is_none(),
            "one canonical video LayoutObject per element"
        );
        let input = record.input.element.as_mut().expect("video element input");
        input.video = Some(state);
        input.media_is_fullscreen = state.is_fullscreen;
        unsafe { &mut *record.node }.SetVideoLayoutState(state);
        result = Some(record.object);
    }
    result
}
