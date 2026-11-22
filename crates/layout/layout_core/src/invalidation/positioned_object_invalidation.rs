// Removing positioned descendants is an invalidation transaction: scan the
// resident fragments, mark affected OOF objects, then propagate dirtiness to
// the containing block.

use super::*;

#[inline(never)]
pub(super) fn remove_positioned_objects(block: &mut LayoutBlock, stay_within: *mut LayoutObject) {
    block.CheckIsNotDestroyed();
    let process_positioned_object_removal = |positioned_object: *mut LayoutObject| -> bool {
        if !stay_within.is_null()
            && (!unsafe { &*positioned_object }.IsDescendantOf(stay_within)
                || stay_within == positioned_object)
        {
            return false;
        }
        unsafe { &mut *positioned_object }
            .SetChildNeedsLayoutWithBehavior(MarkingBehavior::kMarkOnlyThis);
        unsafe { &mut *positioned_object }.MarkParentForSpannerOrOutOfFlowPositionedChange();
        true
    };

    let mut has_positioned_children_in_fragment_tree = false;
    for fragment in block.PhysicalFragments() {
        if !fragment.HasOutOfFlowFragmentChild() {
            continue;
        }
        for fragment_child in fragment.Children() {
            if !fragment_child.IsOutOfFlowPositioned() {
                continue;
            }
            let child = fragment_child.GetMutableLayoutObject();
            if !child.is_null() && process_positioned_object_removal(child) {
                has_positioned_children_in_fragment_tree = true;
            }
        }
    }
    if has_positioned_children_in_fragment_tree {
        let containing_block = block.ContainingBlockForAbsolutePosition();
        if !containing_block.is_null() {
            unsafe { &mut *containing_block }
                .SetChildNeedsLayoutWithBehavior(MarkingBehavior::kMarkContainerChain);
        }
    }
}
