// Page's scroll-offset snapshot is post-layout publication, not tree-building
// policy. Keep the managed-record scan behind a commit-owned sampling frame.

use super::*;

/// Keep the extracted retained input descriptor on the same value as Blink's
/// resident ScrollableArea after layout-time clamping or scroll anchoring.
///
/// Chromium has one live owner here: `PaintLayerScrollableArea`.  Rechrom's
/// `ManagedObject::input` is only the next incremental-attachment descriptor,
/// but leaving its pre-layout value intact makes a later style update write an
/// obsolete offset back into that owner.  Commit the scalar after all native
/// layout scroll adjustments, before either fragments or Page can observe the
/// result.
#[inline(never)]
pub(super) fn commit_resident_scroll_offsets(tree: &mut LayoutObjectTree) {
    let storage = unsafe { &mut *tree.storage_ };
    for record in storage.managed.values_mut() {
        if record.input.style_generated {
            continue;
        }
        let Some(input) = record.input.element.as_mut() else {
            continue;
        };
        let element = foundation::DynamicTo::<crate::internal::layout_node_metadata::Element>(
            record.node,
        );
        let Some(data) = (unsafe { element.as_ref() })
            .and_then(|element| element.InputElementData().as_ref())
        else {
            continue;
        };
        input.scroll_offset = data.scroll_offset;
    }
}

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
