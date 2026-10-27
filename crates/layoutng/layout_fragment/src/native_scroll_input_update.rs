//! Native scroll-input mutation for retained fragment paint state.
//!
//! Included by `layout_object_builder` at the reciprocal assembly boundary.
//! The id-to-record fan-out and node metadata synchronization are fragment
//! input work, so their samples should not accrue to the generic tree facade.

use super::*;

#[inline(never)]
pub(super) fn update_scroll_offset(
    tree: &mut LayoutObjectTree,
    id: u64,
    offset: crate::internal::layout_input::Offset,
) -> bool {
    let storage = unsafe { &mut *tree.storage_ };
    assert!(!storage.updating);
    let mut found = false;
    for key in storage.InputKeys(id).to_vec() {
        let record = storage.managed.get_mut(&key).unwrap();
        if let Some(element) = record.input.element.as_mut() {
            element.scroll_offset = offset;
            unsafe { &mut *record.node }.UpdateInputScrollOffset(offset);
            found = true;
        }
    }
    found
}
