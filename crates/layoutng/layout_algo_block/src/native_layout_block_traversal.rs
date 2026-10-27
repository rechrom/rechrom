//! Native LayoutBlock first-line traversal policy.
//!
//! Included by `layout_block` at the reciprocal assembly boundary. Sibling
//! skipping and nested block descent retain block-algorithm sampling frames.

use super::*;

// cpp: layoutng/internal/layout_block.cc:208-238
#[inline(never)]
pub(super) fn LayoutBlockFirstLineStyleParentBlock(block: &LayoutBlock) -> *const LayoutBlock {
    block.CheckIsNotDestroyed();
    let first_line_block = block as *const LayoutBlock;
    if block.IsInline() {
        return std::ptr::null();
    }
    if block.IsFloatingOrOutOfFlowPositioned() {
        return std::ptr::null();
    }
    let parent_block = block.Parent();
    if parent_block.is_null() || !unsafe { &*parent_block }.BehavesLikeBlockContainer() {
        return std::ptr::null();
    }
    let parent_layout_block = To::<LayoutBlock>(parent_block);
    let mut first_child = unsafe { &*parent_layout_block }.FirstChild();
    while unsafe { &*first_child }.IsFloatingOrOutOfFlowPositioned()
        || (RuntimeEnabledFeatures::FirstLineOnListItemEnabled()
            && unsafe { &*first_child }.IsListMarker())
    {
        first_child = unsafe { &*first_child }.NextSibling();
    }
    if first_child != first_line_block as *mut LayoutObject {
        return std::ptr::null();
    }
    parent_layout_block
}

// cpp: layoutng/internal/layout_block.cc:240-252
#[inline(never)]
pub(super) fn LayoutBlockNearestInnerBlockWithFirstLine(
    block: &mut LayoutBlock,
) -> *mut LayoutBlockFlow {
    block.CheckIsNotDestroyed();
    if block.ChildrenInline() {
        return To::<LayoutBlockFlow>(block as *mut LayoutBlock);
    }
    let mut child = block.FirstChild();
    while !child.is_null()
        && !unsafe { &*child }.IsFloatingOrOutOfFlowPositioned()
        && unsafe { &*child }.IsLayoutBlockFlow()
    {
        if unsafe { &*child }.ChildrenInline() {
            return To::<LayoutBlockFlow>(child);
        }
        child = unsafe { &*To::<LayoutBlock>(child) }.FirstChild();
    }
    std::ptr::null_mut()
}
