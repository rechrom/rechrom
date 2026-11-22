use super::*;

// cpp: layoutng/internal/layout_block_flow.cc:80-82
fn is_inner_editor_child(block: &LayoutBlockFlow) -> bool {
    let parent = block.Parent();
    !parent.is_null() && unsafe { &*parent }.IsTextControlInnerEditor()
}

// cpp: layoutng/internal/layout_block_flow.cc:84-87
fn is_mergeable_anonymous_block(block: &LayoutBlockFlow) -> bool {
    block.IsAnonymousBlockFlow()
        && !block.BeingDestroyed()
        && !block.IsViewTransitionRoot()
        && !is_inner_editor_child(block)
}

// cpp: layoutng/internal/layout_block_flow.cc:98-106
#[inline(never)]
fn reparent_subsequent_floating_or_out_of_flow(
    from: *mut LayoutBlockFlow,
    to: *mut LayoutBlockFlow,
    mut next: *mut LayoutObject,
) {
    while !next.is_null() && unsafe { &*next }.IsFloatingOrOutOfFlowPositioned() {
        let sibling = unsafe { &*next }.NextSibling();
        unsafe { &mut *from }.MoveChildTo(to.cast(), next, std::ptr::null_mut(), true);
        next = sibling;
    }
}

// cpp: layoutng/internal/layout_block_flow.cc:108-116
#[inline(never)]
fn reparent_preceding_floating_or_out_of_flow(
    from: *mut LayoutBlockFlow,
    to: *mut LayoutBlockFlow,
    mut previous: *mut LayoutObject,
) {
    while !previous.is_null() && unsafe { &*previous }.IsFloatingOrOutOfFlowPositioned() {
        let sibling = unsafe { &*previous }.NextSibling();
        let before_child = unsafe { &*to }.FirstChild();
        unsafe { &mut *from }.MoveChildTo(to.cast(), previous, before_child, true);
        previous = sibling;
    }
}

// cpp: layoutng/internal/layout_block_flow.h:153-154
// cpp: layoutng/internal/layout_block_flow.cc:166-212
#[inline(never)]
fn add_child_before_descendant(
    block: &mut LayoutBlockFlow,
    new_child: *mut LayoutObject,
    before_descendant: *mut LayoutObject,
) {
    block.CheckIsNotDestroyed();
    let block_object = block as *mut LayoutBlockFlow as *mut LayoutObject;
    debug_assert_ne!(unsafe { &*before_descendant }.Parent(), block_object);
    let mut container = unsafe { &*before_descendant }.Parent();
    while unsafe { &*container }.Parent() != block_object {
        container = unsafe { &*container }.Parent();
    }
    debug_assert!(!container.is_null());
    assert!(unsafe { &*container }.IsAnonymous());
    if unsafe { &*container }.IsAnonymousBlockFlow() {
        if (unsafe { &*new_child }.IsInline()
            && !unsafe { &*new_child }.IsLayoutOutsideListMarker())
            || unsafe { &*new_child }.IsFloatingOrOutOfFlowPositioned()
            || !unsafe { &*before_descendant }.PreviousSibling().is_null()
        {
            unsafe { &mut *container }.AddChild(new_child, before_descendant);
        } else {
            LayoutBlockFlowAddChildBase(block, new_child, unsafe { &*before_descendant }.Parent());
        }
        return;
    }
    debug_assert!(unsafe { &*container }.IsTable());
    if unsafe { &*new_child }.IsTablePart() {
        unsafe { &mut *container }.AddChild(new_child, before_descendant);
        return;
    }
    let before_child = block.SplitAnonymousBoxesAroundChild(before_descendant);
    assert_eq!(unsafe { &*before_child }.Parent(), block_object);
    LayoutBlockFlowAddChildBase(block, new_child, before_child);
}

// cpp: layoutng/internal/layout_block_flow.cc:214-291
#[inline(never)]
pub(super) fn LayoutBlockFlowAddChildBase(
    block: &mut LayoutBlockFlow,
    new_child: *mut LayoutObject,
    mut before_child: *mut LayoutObject,
) {
    block.CheckIsNotDestroyed();
    let block_object = block as *mut LayoutBlockFlow as *mut LayoutObject;
    if !before_child.is_null() && unsafe { &*before_child }.Parent() != block_object {
        add_child_before_descendant(block, new_child, before_child);
        return;
    }
    let mut made_boxes_non_inline = false;
    let child_is_block_level = !unsafe { &*new_child }.IsInline()
        && !unsafe { &*new_child }.IsFloatingOrOutOfFlowPositioned();
    if block.ChildrenInline() {
        if child_is_block_level {
            block.MakeChildrenNonInline(before_child);
            made_boxes_non_inline = true;
            if !before_child.is_null() && unsafe { &*before_child }.Parent() != block_object {
                before_child = unsafe { &*before_child }.Parent();
                debug_assert!(unsafe { &*before_child }.IsAnonymousBlockFlow());
                debug_assert_eq!(unsafe { &*before_child }.Parent(), block_object);
            }
        }
    } else if !child_is_block_level {
        let after_child = if before_child.is_null() {
            block.LastChild()
        } else {
            unsafe { &*before_child }.PreviousSibling()
        };
        if !after_child.is_null() && unsafe { &*after_child }.IsAnonymousBlockFlow() {
            unsafe { &mut *after_child }.AddChild(new_child, std::ptr::null_mut());
            return;
        }
        if unsafe { &*new_child }.IsInline() && !unsafe { &*new_child }.IsLayoutOutsideListMarker()
        {
            let new_block_flow = To::<LayoutBlockFlow>(block.CreateAnonymousBlock());
            block
                .box_base_mut()
                .AddChildBase(new_block_flow.cast(), before_child);
            reparent_preceding_floating_or_out_of_flow(
                block,
                new_block_flow,
                unsafe { &*new_block_flow }.PreviousSibling(),
            );
            LayoutBlockFlowAddChildBase(
                unsafe { &mut *new_block_flow },
                new_child,
                std::ptr::null_mut(),
            );
            reparent_subsequent_floating_or_out_of_flow(
                block,
                new_block_flow,
                unsafe { &*new_block_flow }.NextSibling(),
            );
            return;
        }
    }
    block.box_base_mut().AddChildBase(new_child, before_child);
    let parent_block_flow = DynamicTo::<LayoutBlockFlow>(block.Parent());
    if !parent_block_flow.is_null() && made_boxes_non_inline && is_mergeable_anonymous_block(block)
    {
        let next_sibling = block.NextSibling();
        unsafe { &mut *parent_block_flow }
            .ChildrenMut()
            .RemoveChildNodeDefault(parent_block_flow.cast(), block_object);
        block.MoveAllChildrenToBefore(parent_block_flow.cast(), next_sibling, true);
        block.Destroy();
    }
}

// cpp: layoutng/internal/layout_block_flow.h:79-79
// cpp: layoutng/internal/layout_block_flow.cc:293-348
#[inline(never)]
pub(super) fn LayoutBlockFlowRemoveChildBase(
    block: &mut LayoutBlockFlow,
    old_child: *mut LayoutObject,
) {
    block.CheckIsNotDestroyed();
    let previous = unsafe { &*old_child }.PreviousSibling();
    let next = unsafe { &*old_child }.NextSibling();
    if !previous.is_null() && !next.is_null() && !unsafe { &*old_child }.IsInline() {
        let previous_block_flow = DynamicTo::<LayoutBlockFlow>(previous);
        if !previous_block_flow.is_null()
            && is_mergeable_anonymous_block(unsafe { &*previous_block_flow })
        {
            reparent_subsequent_floating_or_out_of_flow(block, previous_block_flow, next);
        }
        let next_block_flow = DynamicTo::<LayoutBlockFlow>(unsafe { &*previous }.NextSibling());
        if !next_block_flow.is_null() && is_mergeable_anonymous_block(unsafe { &*next_block_flow })
        {
            reparent_preceding_floating_or_out_of_flow(block, next_block_flow, previous);
        }
    }
    block.block_.RemoveChildBase(old_child);
    if block.IsAnonymous() && is_inner_editor_child(block) && !block.BeingDestroyed() {
        let factories = LayoutObjectFactoryScope::Objects();
        let remove = if factories.is_null() {
            None
        } else {
            unsafe { &*factories }.remove_inner_editor_child
        };
        let Some(remove) = remove else {
            std::panic::panic_any(UnsupportedLayout::new(
                "forms layout module is not installed",
            ));
        };
        remove(block, unsafe { &mut *old_child });
        return;
    }
    if block.FirstChild() == block.LastChild() {
        let child_block_flow = DynamicTo::<LayoutBlockFlow>(block.FirstChild());
        if !child_block_flow.is_null()
            && is_mergeable_anonymous_block(unsafe { &*child_block_flow })
        {
            block.CollapseAnonymousBlockChild(child_block_flow);
        }
    }
    if !block.FirstChild().is_null()
        && !block.BeingDestroyed()
        && !unsafe { &*old_child }.IsFloatingOrOutOfFlowPositioned()
        && !unsafe { &*old_child }.IsAnonymousBlockFlow()
    {
        block.MakeChildrenInlineIfPossible();
    }
    if block.FirstChild().is_null() && is_mergeable_anonymous_block(block) {
        block.Destroy();
    }
}

// cpp: layoutng/internal/layout_block_flow.h:81-81
// cpp: layoutng/internal/layout_block_flow.cc:350-358
pub(super) fn LayoutBlockFlowCanMergeWithBase(
    block: &LayoutBlockFlow,
    other: &LayoutBoxModelObject,
) -> bool {
    let other_block_flow = DynamicTo::<LayoutBlockFlow>(other as *const LayoutBoxModelObject);
    if other_block_flow.is_null() {
        return false;
    }
    is_mergeable_anonymous_block(block)
        && is_mergeable_anonymous_block(unsafe { &*other_block_flow })
}
