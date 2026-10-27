use super::*;

// cpp: layoutng/internal/layout_block_flow.cc:432-480
#[inline(never)]
fn get_inline_run(
    start: *mut LayoutObject,
    boundary: *mut LayoutObject,
    inline_run_start: &mut *mut LayoutObject,
    inline_run_end: &mut *mut LayoutObject,
) {
    let mut curr = start;
    if !curr.is_null() && unsafe { &*curr }.IsLayoutOutsideListMarker() {
        curr = unsafe { &*curr }.NextSibling();
    }
    loop {
        while !curr.is_null()
            && !(unsafe { &*curr }.IsInline()
                || unsafe { &*curr }.IsFloatingOrOutOfFlowPositioned())
        {
            curr = unsafe { &*curr }.NextSibling();
        }
        *inline_run_start = curr;
        *inline_run_end = curr;
        if curr.is_null() {
            return;
        }
        let mut saw_inline = unsafe { &*curr }.IsInline();
        curr = unsafe { &*curr }.NextSibling();
        while !curr.is_null()
            && (unsafe { &*curr }.IsInline() || unsafe { &*curr }.IsFloatingOrOutOfFlowPositioned())
            && curr != boundary
        {
            *inline_run_end = curr;
            if unsafe { &*curr }.IsInline() {
                saw_inline = true;
            }
            curr = unsafe { &*curr }.NextSibling();
        }
        if saw_inline {
            break;
        }
    }
}

// cpp: layoutng/internal/layout_block_flow.cc:360-373
fn allows_collapse_anonymous_block_child(block: &LayoutBlockFlow, child: &LayoutBlockFlow) -> bool {
    if child.BeingDestroyed() {
        return false;
    }
    if child.IsViewTransitionRoot() {
        return false;
    }
    !child.ChildrenInline() || block.AllowsInlineChildren()
}

// cpp: layoutng/internal/layout_block_flow.h:83-83
// cpp: layoutng/internal/layout_block_flow.cc:375-387
#[inline(never)]
pub(super) fn LayoutBlockFlowCollapseAnonymousBlockChild(
    block: &mut LayoutBlockFlow,
    child: *mut LayoutBlockFlow,
) {
    block.CheckIsNotDestroyed();
    if !allows_collapse_anonymous_block_child(block, unsafe { &*child }) {
        return;
    }
    block.SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(unsafe {
        std::ptr::addr_of!(layout_invalidation_reason::kChildAnonymousBlockChanged)
    });
    let next_sibling = unsafe { &*child }.NextSibling();
    let has_layer = unsafe { &*child }.HasLayer();
    unsafe { &mut *child }.MoveAllChildrenToBefore(
        block as *mut _ as *mut LayoutBoxModelObject,
        next_sibling,
        has_layer,
    );
    block.SetChildrenInline(unsafe { &*child }.ChildrenInline());
    let block_object = block as *mut LayoutBlockFlow as *mut LayoutObject;
    block
        .ChildrenMut()
        .RemoveChildNode(block_object, child.cast(), has_layer);
    unsafe { &mut *child }.Destroy();
}

// cpp: layoutng/internal/layout_block_flow.h:156-156
// cpp: layoutng/internal/layout_block_flow.cc:389-430
#[inline(never)]
pub(super) fn LayoutBlockFlowMakeChildrenInlineIfPossible(block: &mut LayoutBlockFlow) {
    block.CheckIsNotDestroyed();
    if !block.AllowsInlineChildren() {
        return;
    }
    if block.IsAnonymousBlockFlow() {
        return;
    }
    let mut blocks_to_remove = HeapVector::<Member<LayoutBlockFlow>>::default();
    let mut child = block.FirstChild();
    while !child.is_null() {
        let current = unsafe { &*child };
        if current.IsFloating() || current.IsOutOfFlowPositioned() {
            child = current.NextSibling();
            continue;
        }
        let child_block_flow = DynamicTo::<LayoutBlockFlow>(child);
        if !current.IsAnonymousBlockFlow() || child_block_flow.is_null() {
            return;
        }
        if unsafe { &*child_block_flow }.BeingDestroyed() {
            return;
        }
        if !current.ChildrenInline() {
            return;
        }
        blocks_to_remove.push_back(Member::from_ptr(child_block_flow));
        child = current.NextSibling();
    }
    for child in blocks_to_remove.iter() {
        LayoutBlockFlowCollapseAnonymousBlockChild(block, child.Get());
    }
    block.SetChildrenInline(true);
}

// cpp: layoutng/internal/layout_block_flow.cc:482-523
#[inline(never)]
pub(super) fn LayoutBlockFlowMakeChildrenNonInline(
    block: &mut LayoutBlockFlow,
    insertion_point: *mut LayoutObject,
) {
    block.CheckIsNotDestroyed();
    let block_object = block as *mut LayoutBlockFlow as *mut LayoutObject;
    debug_assert!(
        insertion_point.is_null() || unsafe { &*insertion_point }.Parent() == block_object
    );
    block.SetChildrenInline(false);
    block.ClearInlineNodeData();
    let mut child = block.FirstChild();
    if child.is_null() {
        return;
    }
    while !child.is_null() {
        let mut inline_run_start = std::ptr::null_mut();
        let mut inline_run_end = std::ptr::null_mut();
        get_inline_run(
            child,
            insertion_point,
            &mut inline_run_start,
            &mut inline_run_end,
        );
        if inline_run_start.is_null() {
            break;
        }
        child = unsafe { &*inline_run_end }.NextSibling();
        let anonymous_block = block.CreateAnonymousBlock();
        block.ChildrenMut().InsertChildNodeDefault(
            block_object,
            anonymous_block.cast(),
            inline_run_start,
        );
        block.MoveChildrenTo(
            anonymous_block.cast(),
            inline_run_start,
            child,
            std::ptr::null_mut(),
            false,
        );
    }
    #[cfg(debug_assertions)]
    {
        let mut current = block.FirstChild();
        while !current.is_null() {
            debug_assert!(
                !unsafe { &*current }.IsInline()
                    || unsafe { &*current }.IsLayoutOutsideListMarker()
            );
            current = unsafe { &*current }.NextSibling();
        }
    }
    block.SetShouldDoFullPaintInvalidation();
}

// cpp: layoutng/internal/layout_block_flow.h:150-150
// cpp: layoutng/internal/layout_block_flow.cc:525-534
#[inline(never)]
pub(super) fn LayoutBlockFlowDirtyLinesFromChangedChild(
    block: &mut LayoutBlockFlow,
    child: *mut LayoutObject,
) {
    block.CheckIsNotDestroyed();
    if unsafe { &*child }.IsInLayoutNGInlineFormattingContext() {
        FragmentItems::DirtyLinesFromChangedChild(unsafe { &*child }, block);
    }
}
