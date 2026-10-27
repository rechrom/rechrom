//! Native BlockNode traversal policy.
//!
//! Included by `block_node` at the reciprocal assembly boundary. Inline-run
//! skipping, first layout-child selection, and fieldset legend discovery keep
//! separate block-algorithm sampling frames.

use super::*;

// cpp: layoutng/internal/block_node.cc:843-868
#[inline(never)]
pub(super) fn BlockNodeNextBlockSibling(node: &BlockNode) -> BlockNode {
    let mut next_sibling = unsafe { &*node.GetLayoutBox() }.NextSibling();
    while !next_sibling.is_null() && unsafe { &*next_sibling }.IsInline() {
        #[cfg(debug_assertions)]
        {
            if !unsafe { &*next_sibling }.IsText() {
                unsafe { &*next_sibling }.ShowLayoutTreeForThis();
            }
            debug_assert!(unsafe { &*next_sibling }.IsText());
        }
        unsafe { &mut *next_sibling }.ClearNeedsLayout();
        next_sibling = unsafe { &*next_sibling }.NextSibling();
    }
    if next_sibling.is_null() {
        return BlockNode::null();
    }
    BlockNode::new(To::<LayoutBox>(next_sibling))
}

// cpp: layoutng/internal/block_node.cc:870-915
#[inline(never)]
pub(super) fn BlockNodeFirstChild(node: &BlockNode) -> LayoutInputNode {
    if node.ChildLayoutBlockedByDisplayLock() {
        return LayoutInputNode::null();
    }
    let box_ = unsafe { &*node.GetLayoutBox() };
    let block = DynamicTo::<LayoutBlock>(node.GetLayoutBox());
    if block.is_null() {
        return BlockNode::new(To::<LayoutBox>(box_.SlowFirstChild())).base;
    }
    let block_ref = unsafe { &*block };
    let mut child = block_ref.FirstChild();
    if child.is_null() {
        return LayoutInputNode::null();
    }
    if !block_ref.ChildrenInline() {
        return BlockNode::new(To::<LayoutBox>(child)).base;
    }
    let inline_node = InlineNode::new(To::<LayoutBlockFlow>(block));
    if !inline_node.IsBlockLevel() {
        return inline_node.base;
    }
    while !child.is_null() && unsafe { &*child }.IsInline() {
        debug_assert!(unsafe { &*child }.IsText());
        unsafe { &mut *child }.ClearNeedsLayout();
        child = unsafe { &*child }.NextSibling();
    }
    if child.is_null() {
        return LayoutInputNode::null();
    }
    debug_assert!(unsafe { &*child }.IsFloatingOrOutOfFlowPositioned());
    BlockNode::new(To::<LayoutBox>(child)).base
}

// cpp: layoutng/internal/block_node.cc:917-927
#[inline(never)]
pub(super) fn BlockNodeGetRenderedLegend(node: &BlockNode) -> BlockNode {
    if !node.IsFieldsetContainer() {
        return BlockNode::null();
    }
    let mut child = unsafe { &*node.GetLayoutBox() }.SlowFirstChild();
    while !child.is_null() {
        if unsafe { &*child }.IsRenderedLegendCandidate() {
            return BlockNode::new(To::<LayoutBox>(child));
        }
        child = unsafe { &*child }.NextSibling();
    }
    BlockNode::null()
}
