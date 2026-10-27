//! Targeted native node-input update transaction.
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
        let mut input = record.input.clone();
        input.style = Arc::new(style.clone());
        // Resource bindings remain with their owner. A resource completion may
        // refresh this exact input; SVG graph ownership remains resident.
        Arc::make_mut(&mut input.style).paint.svg_filter_graphs =
            record.effective_style.paint.svg_filter_graphs.clone();
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
    // Propagate the canonical parent style to existing inherited text,
    // matching Create's effective text style and Blink's anonymous children.
    let parents = keys
        .iter()
        .map(|key| {
            let record = &storage.managed[key];
            (record.object, record.effective_style.native_style)
        })
        .collect::<Vec<_>>();
    for (parent, native) in parents {
        for key in storage.ChildKeys(parent) {
            let record = &storage.managed[key];
            if record.input.kind == NodeKind::kText
                && !keys.contains(key)
                && !native.is_null()
                && record.effective_style.native_style == native
            {
                if record.input.style_generated || record.input.first_line_style.is_some() {
                    return None;
                }
                let mut input = record.input.clone();
                input.style = Arc::new(style.clone());
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
        storage.Create(
            parent,
            input.id,
            input.kind,
            &input.style,
            input.element,
            input.debug_name,
            input.text,
            text_style,
            false,
            std::ptr::null(),
            std::ptr::null(),
            Some(key),
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
