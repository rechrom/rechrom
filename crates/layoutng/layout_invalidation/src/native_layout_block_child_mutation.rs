//! Native LayoutBlock child-tree mutation traversal.
//!
//! Included by `layout_block` at the reciprocal assembly boundary. Locating
//! the direct anonymous child for a descendant insertion keeps a dedicated
//! invalidation sampling frame rather than accruing as LayoutBlock self time.

use super::*;

// cpp: layoutng/internal/layout_block.cc:76-100
#[inline(never)]
pub(super) fn LayoutBlockAddChildBeforeDescendant(
    block: &mut LayoutBlock,
    new_child: *mut LayoutObject,
    before_descendant: *mut LayoutObject,
) {
    block.CheckIsNotDestroyed();
    debug_assert!(!block.IsLayoutBlockFlow());
    let block_object = block as *mut LayoutBlock as *mut LayoutObject;
    debug_assert_ne!(unsafe { &*before_descendant }.Parent(), block_object);

    if !unsafe { &*new_child }.IsInline() && !unsafe { &*new_child }.IsTablePart() {
        let before_child = block.SplitAnonymousBoxesAroundChild(before_descendant);
        debug_assert_eq!(unsafe { &*before_child }.Parent(), block_object);
        block.AddChild(new_child, before_child);
        return;
    }

    let mut before_descendant_container = unsafe { &*before_descendant }.Parent();
    while unsafe { &*before_descendant_container }.Parent() != block_object {
        before_descendant_container = unsafe { &*before_descendant_container }.Parent();
    }
    assert!(unsafe { &*before_descendant_container }.IsAnonymous());
    assert!(unsafe { &*before_descendant_container }.IsLayoutBlockFlow());
    unsafe { &mut *before_descendant_container }.AddChild(new_child, before_descendant);
}
