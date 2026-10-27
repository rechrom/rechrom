// Post-layout sticky projection walks native objects but publishes
// fragment-local paint geometry, so both its trace and sampling frame belong
// to layout_fragment.

use super::*;

#[inline(never)]
pub(super) fn update_sticky_offsets_after_layout(root: &mut LayoutObject) {
    let _phase = layout_fragment_boundary::trace::span(
        layout_fragment_boundary::trace::TracePhase::UpdateStickyOffsets,
    );
    let mut objects = Vec::new();
    let mut object = root as *mut LayoutObject;
    while !object.is_null() {
        objects.push(object);
        object = unsafe { &*object }.NextInPreOrder(root);
    }
    crate::internal::boundary::layout_boundary::UpdateStickyOffsets(objects, false);
}
