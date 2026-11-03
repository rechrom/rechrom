//! Commit-owned lookup from stable input identity to a published LayoutObject.

use super::*;

#[inline(always)]
pub(super) fn output_node(tree: &LayoutObjectTree, id: u64) -> Option<&LayoutObject> {
    let object = output_node_ptr(unsafe { &*tree.storage_ }, id)?;
    Some(unsafe { &*object })
}

#[inline(always)]
pub(super) fn output_node_mut(tree: &mut LayoutObjectTree, id: u64) -> Option<&mut LayoutObject> {
    let object = output_node_ptr(unsafe { &*tree.storage_ }, id)?;
    Some(unsafe { &mut *object })
}

#[inline(never)]
fn output_node_ptr(storage: &LayoutObjectTreeStorage, id: u64) -> Option<*mut LayoutObject> {
    storage
        .InputKeys(id)
        .iter()
        .filter_map(|key| storage.managed.get(key))
        .filter(|record| !record.input.style_generated)
        .min_by_key(|record| record.input.kind as i32)
        .map(|record| record.object)
}
