//! Chromium metadata-tree append transaction.
//!
//! Included by `layout_node_metadata` at the reciprocal assembly boundary.
//! Cycle admission may walk the ancestor chain, and link mutation belongs to
//! invalidation rather than the metadata value facade.

use super::*;

fn TreeRoot(mut node: *mut Node) -> *mut Node {
    debug_assert!(!node.is_null());
    loop {
        let parent = unsafe { &*node }.parent_.Get();
        if parent.is_null() {
            return node;
        }
        node = unsafe { &mut (*parent).node };
    }
}

#[inline(never)]
pub(super) fn DetachForTreeUpdate(node: &mut Node) {
    // Clear the control's direct native UA-part aliases before unlinking.
    // A replacement part can then attach within this same transaction; hit
    // queries cannot retain a detached inner-editor/container identity.
    let host = foundation::DynamicTo::<crate::internal::form_node_metadata::TextControlElement>(
        node.OwnerShadowHost().cast::<Node>(),
    );
    if let Some(host) = unsafe { host.as_mut() } {
        if host.InnerEditorElement().cast::<Node>() == node as *mut Node {
            host.inner_editor_ = Member::default();
        }
        if host.HitContainerElement().cast::<Node>() == node as *mut Node {
            host.hit_container_ = Member::default();
        }
    }
    let parent = node.parent_.Get();
    if parent.is_null() {
        return;
    }
    let previous = node.previous_.Get();
    let next = node.next_.Get();
    unsafe {
        if previous.is_null() {
            (*parent).first_child_ = Member::from_ptr(next);
        } else {
            (*previous).next_ = Member::from_ptr(next);
        }
        if next.is_null() {
            (*parent).last_child_ = Member::from_ptr(previous);
        } else {
            (*next).previous_ = Member::from_ptr(previous);
        }
    }
    node.parent_ = Member::default();
    node.previous_ = Member::default();
    node.next_ = Member::default();
}

#[inline(never)]
pub(super) fn InsertForTreeUpdate(parent: &mut ContainerNode, child: &mut Node, before: *mut Node) {
    assert!(before.is_null() || unsafe { &*before }.parentNode() == parent);
    let global_owner = TreeRoot(&mut parent.node);
    DetachForTreeUpdate(child);
    let previous = if before.is_null() {
        parent.last_child_.Get()
    } else {
        unsafe { &*before }.previousSibling()
    };
    child.parent_ = Member::from_ptr(parent);
    // The native Document/LocalFrameView identity is available as soon as a
    // node joins its metadata tree.  PaintLayer may create its scrollable area
    // during the subsequent SetStyle call, before the final relation pass.
    child.global_input_owner_ = Member::from_ptr(global_owner);
    child.previous_ = Member::from_ptr(previous);
    child.next_ = Member::from_ptr(before);
    unsafe {
        if previous.is_null() {
            parent.first_child_ = Member::from_ptr(child);
        } else {
            (*previous).next_ = Member::from_ptr(child);
        }
        if before.is_null() {
            parent.last_child_ = Member::from_ptr(child);
        } else {
            (*before).previous_ = Member::from_ptr(child);
        }
    }
}

// cpp: layoutng/internal/layout_node_metadata.cc:202-215
#[inline(never)]
pub(super) fn AppendChild(
    parent: &mut ContainerNode,
    child: &mut Node,
) -> Result<(), AppendChildError> {
    if parent.node.InputTreeIsSealed() || child.InputTreeIsSealed() {
        return Err(AppendChildError::SealedTree);
    }
    if !child.parent_.Get().is_null()
        || !child.previous_.Get().is_null()
        || !child.next_.Get().is_null()
    {
        return Err(AppendChildError::AlreadyAttached);
    }
    let child_ptr = child as *mut Node;
    let mut ancestor = &mut parent.node as *mut Node;
    let mut global_owner = ancestor;
    while !ancestor.is_null() {
        global_owner = ancestor;
        if ancestor == child_ptr {
            return Err(AppendChildError::Cycle);
        }
        // SAFETY: metadata tree links are valid for the owning tree's
        // lifetime; only the root owner destroys nodes.
        ancestor = unsafe {
            let ancestor_parent = (*ancestor).parent_.Get();
            if ancestor_parent.is_null() {
                std::ptr::null_mut()
            } else {
                &mut (*ancestor_parent).node
            }
        };
    }
    child.parent_ = Member::from_ptr(parent);
    // Match Node's source owner at attachment time.  Delaying this identity
    // until FinalizeTreeRelations makes PaintLayerScrollableArea registration
    // miss LocalFrameView while the initial LayoutObject tree is constructed.
    child.global_input_owner_ = Member::from_ptr(global_owner);
    child.previous_ = Member::from_ptr(parent.last_child_.Get());
    if !parent.last_child_.Get().is_null() {
        // SAFETY: last_child_ is a linked node owned by the same tree.
        unsafe {
            (*parent.last_child_.Get()).next_ = Member::from_ptr(child_ptr);
        }
    } else {
        parent.first_child_ = Member::from_ptr(child_ptr);
    }
    parent.last_child_ = Member::from_ptr(child_ptr);
    Ok(())
}
