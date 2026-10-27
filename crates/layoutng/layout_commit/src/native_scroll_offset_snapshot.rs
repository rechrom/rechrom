// Page's scroll-offset snapshot is post-layout publication, not tree-building
// policy. Keep the managed-record scan behind a commit-owned sampling frame.

use super::*;

#[inline(never)]
pub(super) fn output_scroll_offsets(
    tree: &LayoutObjectTree,
) -> Vec<(u64, crate::internal::layout_input::Offset)> {
    let storage = unsafe { &*tree.storage_ };
    storage
        .managed
        .values()
        .filter_map(|record| {
            if record.input.style_generated {
                return None;
            }
            let element = foundation::DynamicTo::<crate::internal::layout_node_metadata::Element>(
                record.node,
            );
            let data = unsafe { element.as_ref() }?.InputElementData().as_ref()?;
            Some((record.input.id, data.scroll_offset))
        })
        .collect()
}
