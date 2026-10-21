#![allow(non_snake_case)]

use foundation::{
    DynamicTo, Member, PaintInvalidationReason, SubtreePaintPropertyUpdateReason, Visitor,
};
use layoutng_fragment_tree::fragment_items::FragmentItems;

use super::layout_inline::LayoutInline;
use super::layout_invalidation_reason;
use super::layout_object::LayoutObject;
use super::layout_text::LayoutText;

// cpp: layoutng/internal/layout_object_child_list.cc:44-66
fn InvalidateInlineItems(object: *mut LayoutObject) {
    debug_assert!(unsafe { &*object }.IsInLayoutNGInlineFormattingContext());

    let layout_text = DynamicTo::<LayoutText>(object);
    if !layout_text.is_null() {
        unsafe { &mut *layout_text }.InvalidateInlineItems();
    } else {
        let layout_inline = DynamicTo::<LayoutInline>(object);
        if !layout_inline.is_null() {
            let mut child = unsafe { &*layout_inline }.FirstChild();
            while !child.is_null() {
                if unsafe { &*child }.IsInLayoutNGInlineFormattingContext() {
                    InvalidateInlineItems(child);
                }
                child = unsafe { &*child }.NextSibling();
            }
        }
    }

    if unsafe { &*object }.FirstInlineFragmentItemIndex() != 0 {
        FragmentItems::LayoutObjectWillBeMoved(unsafe { &*object });
    }
    unsafe { &mut *object }.SetIsInLayoutNGInlineFormattingContext(false);
}

// cpp: layoutng/internal/layout_object_child_list.h:38-73
pub struct LayoutObjectChildList {
    first_child_: Member<LayoutObject>,
    last_child_: Member<LayoutObject>,
}

impl Default for LayoutObjectChildList {
    // cpp: layoutng/internal/layout_object_child_list.h:42-42
    fn default() -> Self {
        Self {
            first_child_: Member::default(),
            last_child_: Member::default(),
        }
    }
}

impl LayoutObjectChildList {
    // cpp: layoutng/internal/layout_object_child_list.h:43-43
    // cpp: layoutng/internal/layout_node_data.cc:203-206
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.first_child_);
        visitor.Trace(&self.last_child_);
    }

    // cpp: layoutng/internal/layout_object_child_list.h:45-45
    pub fn FirstChild(&self) -> *mut LayoutObject {
        self.first_child_.Get()
    }

    // cpp: layoutng/internal/layout_object_child_list.h:46-46
    pub fn LastChild(&self) -> *mut LayoutObject {
        self.last_child_.Get()
    }

    // cpp: layoutng/internal/layout_object_child_list.h:48-48
    // cpp: layoutng/internal/layout_object_fragment_data.cc:104-113
    pub fn DestroyLeftoverChildren(&mut self) {
        while !self.FirstChild().is_null() {
            let child = self.FirstChild();
            let child_node = unsafe { &*child }.GetNode();
            if !child_node.is_null() {
                unsafe { &mut *child_node }.SetLayoutObject(std::ptr::null_mut());
            }
            unsafe { &mut *child }.Destroy();
        }
    }

    // cpp: layoutng/internal/layout_object_child_list.h:63-66
    // cpp: layoutng/internal/layout_object_child_list.cc:70-86
    pub fn AppendInitialChildNode(
        &mut self,
        owner: *mut LayoutObject,
        new_child: *mut LayoutObject,
    ) {
        assert!(!owner.is_null());
        assert!(!new_child.is_null());
        assert_eq!(
            self as *const _,
            unsafe { &*owner }.VirtualChildren() as *const _
        );
        assert!(unsafe { &*new_child }.Parent().is_null());
        assert!(unsafe { &*new_child }.PreviousSibling().is_null());
        assert!(unsafe { &*new_child }.NextSibling().is_null());

        unsafe { &mut *new_child }.SetParent(owner);
        unsafe { &mut *new_child }.SetPreviousSibling(self.LastChild());
        if !self.LastChild().is_null() {
            unsafe { &mut *self.LastChild() }.SetNextSibling(new_child);
        } else {
            self.first_child_ = Member::from_ptr(new_child);
        }
        self.last_child_ = Member::from_ptr(new_child);
    }

    // cpp: layoutng/internal/layout_object_child_list.h:50-52
    // cpp: layoutng/internal/layout_object_child_list.cc:90-143
    pub fn RemoveChildNode(
        &mut self,
        owner: *mut LayoutObject,
        old_child: *mut LayoutObject,
        notify_layout_object: bool,
    ) -> *mut LayoutObject {
        debug_assert_eq!(unsafe { &*old_child }.Parent(), owner);
        debug_assert_eq!(
            self as *const _,
            unsafe { &*owner }.VirtualChildren() as *const _
        );

        if notify_layout_object && unsafe { &*old_child }.EverHadLayout() {
            unsafe { &mut *old_child }.SetNeedsLayoutAndIntrinsicWidthsRecalc(unsafe {
                std::ptr::addr_of!(layout_invalidation_reason::kRemovedFromLayout)
            });
            if unsafe { &*old_child }.IsOutOfFlowPositioned()
                || unsafe { &*old_child }.IsColumnSpanAll()
            {
                unsafe { &mut *old_child }.MarkParentForSpannerOrOutOfFlowPositionedChange();
            }
        }
        self.InvalidatePaintOnRemoval(unsafe { &*old_child });

        if notify_layout_object {
            unsafe { &mut *old_child }.WillBeRemovedFromTree();
        }
        if unsafe { &*old_child }.IsInLayoutNGInlineFormattingContext() {
            unsafe { &mut *owner }.SetChildNeedsCollectInlines();
            InvalidateInlineItems(old_child);
        }

        let previous = unsafe { &*old_child }.PreviousSibling();
        let next = unsafe { &*old_child }.NextSibling();
        if !previous.is_null() {
            unsafe { &mut *previous }.SetNextSibling(next);
        }
        if !next.is_null() {
            unsafe { &mut *next }.SetPreviousSibling(previous);
        }
        if self.FirstChild() == old_child {
            self.first_child_ = Member::from_ptr(next);
        }
        if self.LastChild() == old_child {
            self.last_child_ = Member::from_ptr(previous);
        }
        unsafe { &mut *old_child }.SetPreviousSibling(std::ptr::null_mut());
        unsafe { &mut *old_child }.SetNextSibling(std::ptr::null_mut());
        unsafe { &mut *old_child }.SetParent(std::ptr::null_mut());
        let consumes_notification = unsafe { &*old_child }.ConsumesSubtreeChangeNotification();
        unsafe { &mut *old_child }
            .RegisterSubtreeChangeListenerOnDescendants(consumes_notification);
        old_child
    }

    // cpp: layoutng/internal/layout_object_child_list.h:53-56
    // cpp: layoutng/internal/layout_object_child_list.cc:145-230
    pub fn InsertChildNode(
        &mut self,
        owner: *mut LayoutObject,
        new_child: *mut LayoutObject,
        mut before_child: *mut LayoutObject,
        notify_layout_object: bool,
    ) {
        debug_assert!(unsafe { &*new_child }.Parent().is_null());
        debug_assert_eq!(
            self as *const _,
            unsafe { &*owner }.VirtualChildren() as *const _
        );
        debug_assert!(
            !unsafe { &*owner }.IsLayoutBlockFlow()
                || (!unsafe { &*new_child }.IsTableSection()
                    && !unsafe { &*new_child }.IsTableRow()
                    && !unsafe { &*new_child }.IsTableCell())
        );

        while !before_child.is_null() {
            let parent = unsafe { &*before_child }.Parent();
            if parent.is_null() || parent == owner {
                break;
            }
            before_child = parent;
        }
        if !before_child.is_null() && unsafe { &*before_child }.Parent() != owner {
            panic!("before-child is outside the owner's subtree");
        }

        if unsafe { &*new_child }.IsInLayoutNGInlineFormattingContext() {
            InvalidateInlineItems(new_child);
        }
        unsafe { &mut *new_child }.SetParent(owner);

        if self.FirstChild() == before_child {
            self.first_child_ = Member::from_ptr(new_child);
        }
        if !before_child.is_null() {
            let previous_sibling = unsafe { &*before_child }.PreviousSibling();
            if !previous_sibling.is_null() {
                unsafe { &mut *previous_sibling }.SetNextSibling(new_child);
            }
            unsafe { &mut *new_child }.SetPreviousSibling(previous_sibling);
            unsafe { &mut *new_child }.SetNextSibling(before_child);
            unsafe { &mut *before_child }.SetPreviousSibling(new_child);
        } else {
            if !self.LastChild().is_null() {
                unsafe { &mut *self.LastChild() }.SetNextSibling(new_child);
            }
            unsafe { &mut *new_child }.SetPreviousSibling(self.LastChild());
            self.last_child_ = Member::from_ptr(new_child);
        }

        if unsafe { &*new_child }.IsInLayoutNGInlineFormattingContext() {
            InvalidateInlineItems(new_child);
        }
        if notify_layout_object {
            unsafe { &mut *new_child }.InsertedIntoTree();
        }
        if unsafe { &*owner }.IsInLayoutNGInlineFormattingContext()
            || (unsafe { &*owner }.EverHadLayout() && unsafe { &*owner }.ChildrenInline())
        {
            unsafe { &mut *owner }.SetChildNeedsCollectInlines();
        }
        if unsafe { &*owner }.HasSubtreeChangeListenerRegistered() {
            unsafe { &mut *new_child }.RegisterSubtreeChangeListenerOnDescendants(true);
        }
        unsafe { &mut *owner }.SetShouldCheckForPaintInvalidation();

        unsafe { &mut *new_child }.SetNeedsLayoutAndIntrinsicWidthsRecalc(unsafe {
            std::ptr::addr_of!(layout_invalidation_reason::kAddedToLayout)
        });
        if unsafe { &*new_child }.IsOutOfFlowPositioned()
            || unsafe { &*new_child }.IsColumnSpanAll()
        {
            unsafe { &mut *new_child }.MarkParentForSpannerOrOutOfFlowPositionedChange();
        }
        unsafe { &mut *new_child }
            .SetShouldDoFullPaintInvalidationWithReason(PaintInvalidationReason::kAppeared);
        unsafe { &mut *new_child }.AddSubtreePaintPropertyUpdateReason(
            SubtreePaintPropertyUpdateReason::kContainerChainMayChange,
        );
        unsafe { &mut *new_child }.SetNeedsOverflowRecalc();
        if !unsafe { &*owner }.ChildNeedsFullLayout() {
            unsafe { &mut *owner }.SetChildNeedsLayout();
        }
    }

    // cpp: layoutng/internal/layout_object_child_list.h:57-61
    pub fn AppendChildNode(
        &mut self,
        owner: *mut LayoutObject,
        new_child: *mut LayoutObject,
        notify_layout_object: bool,
    ) {
        self.InsertChildNode(owner, new_child, std::ptr::null_mut(), notify_layout_object);
    }

    // cpp: layoutng/internal/layout_object_child_list.h:52-52
    pub fn RemoveChildNodeDefault(
        &mut self,
        owner: *mut LayoutObject,
        old_child: *mut LayoutObject,
    ) -> *mut LayoutObject {
        self.RemoveChildNode(owner, old_child, true)
    }

    // cpp: layoutng/internal/layout_object_child_list.h:56-56
    pub fn InsertChildNodeDefault(
        &mut self,
        owner: *mut LayoutObject,
        new_child: *mut LayoutObject,
        before_child: *mut LayoutObject,
    ) {
        self.InsertChildNode(owner, new_child, before_child, true);
    }

    // cpp: layoutng/internal/layout_object_child_list.h:59-59
    pub fn AppendChildNodeDefault(
        &mut self,
        owner: *mut LayoutObject,
        new_child: *mut LayoutObject,
    ) {
        self.AppendChildNode(owner, new_child, true);
    }

    // cpp: layoutng/internal/layout_object_child_list.h:69-69
    // cpp: layoutng/internal/layout_object_child_list.cc:232-240
    fn InvalidatePaintOnRemoval(&self, old_child: &LayoutObject) {
        if !old_child.IsRooted() {
            return;
        }
        let view = old_child.View();
        if !view.is_null() && (old_child.IsBody() || old_child.IsDocumentElement()) {
            unsafe { &mut *view }.SetShouldDoFullPaintInvalidation();
            unsafe { &mut *view }.SetBackgroundNeedsFullPaintInvalidation();
        }
    }
}
