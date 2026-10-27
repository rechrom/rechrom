//! Native input-triggered invalidation for the resident LayoutObject tree.
//!
//! Included by `layout_object_builder` so the standalone package remains free
//! of concrete native object types while the traversals keep invalidation
//! sampling and trace ownership.

use super::*;

#[inline(never)]
pub(super) fn invalidate_editing(tree: &mut LayoutObjectTree, id: u64) -> bool {
    let _trace =
        layout_invalidation::trace_span(layout_invalidation::TracePhase::EditingInvalidation);
    let storage = unsafe { &mut *tree.storage_ };
    assert!(!storage.updating);
    let mut found = false;
    for key in storage.InputKeys(id).to_vec() {
        let record = storage.managed.get_mut(&key).unwrap();
        unsafe { &mut *record.object }.SetNeedsLayout(std::ptr::addr_of!(
            crate::internal::layout_invalidation_reason::kTextControlChanged
        ));
        found = true;
    }
    found
}

#[inline(never)]
pub(super) fn invalidate_selection_paint(tree: &mut LayoutObjectTree, id: u64) -> bool {
    let mut trace = layout_invalidation::trace_span(
        layout_invalidation::TracePhase::SelectionPaintInvalidation,
    );
    let storage = unsafe { &mut *tree.storage_ };
    assert!(!storage.updating);
    let roots = storage
        .InputKeys(id)
        .iter()
        .map(|key| storage.managed[key].object)
        .collect::<Vec<_>>();
    let mut stack = roots.clone();
    let mut visited = std::collections::HashSet::new();
    let mut leaves = Vec::new();
    while let Some(object) = stack.pop() {
        if !visited.insert(object) {
            continue;
        }
        // Text-control selections are painted by their generated LayoutText
        // descendants. Invalidating unrelated generated boxes would discard
        // more recordings than Chromium's selection client invalidation.
        if unsafe { &*object }.IsText() {
            leaves.push(object);
        }
        for key in storage.ChildKeys(object).to_vec() {
            stack.push(storage.managed[&key].object);
        }
    }
    trace.set("roots", roots.len() as f64);
    trace.set("visited", visited.len() as f64);
    trace.set("leaves", leaves.len() as f64);
    for object in leaves {
        unsafe { &mut *object }.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
            foundation::PaintInvalidationReason::kSelection,
        );
    }
    !roots.is_empty()
}
