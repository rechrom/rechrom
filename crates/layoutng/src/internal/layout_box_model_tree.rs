#![allow(non_snake_case)]

use foundation::{DynamicTo, LayoutUnit, MinimumValueForLength, To};
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;

use super::layout_block_flow::LayoutBlockFlow;
use super::layout_box::LayoutBox;
use super::layout_box_model_object::LayoutBoxModelObject;
use super::layout_invalidation_reason;
use super::layout_node_metadata::{ContainerNode, Node};
use super::layout_object::LayoutObject;

// The derived owner calls this when constructing its LayoutObject base.
// Its own fields belong to the pending layout_box_model_object.h record.
// cpp: layoutng/internal/layout_box_model_tree.cc:33-35
pub fn LayoutBoxModelObjectBaseForNode(node: *mut ContainerNode) -> LayoutObject {
    LayoutObject::new_base(node as *mut Node)
}

// cpp: layoutng/internal/layout_box_model_tree.cc:38-41
fn MarkBoxForRelayoutAfterSplit(box_: *mut LayoutBoxModelObject) {
    unsafe { &mut *box_ }.SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(unsafe {
        std::ptr::addr_of!(layout_invalidation_reason::kAnonymousBlockChange)
    });
}

// cpp: layoutng/internal/layout_box_model_tree.cc:43-52
fn CollapseLoneAnonymousBlockChild(parent: *mut LayoutBox, child: *mut LayoutObject) {
    let child_block_flow = DynamicTo::<LayoutBlockFlow>(child);
    let parent_block_flow = DynamicTo::<LayoutBlockFlow>(parent);
    if !unsafe { &*child }.IsAnonymousBlockFlow() || child_block_flow.is_null() {
        return;
    }
    if parent_block_flow.is_null() {
        return;
    }
    unsafe { &mut *parent_block_flow }.CollapseAnonymousBlockChild(child_block_flow);
}

// cpp: layoutng/internal/layout_box_model_object.h:327-355
unsafe extern "Rust" {
    fn DispatchLayoutBoxModelObjectMoveChildrenTo(
        owner: *mut LayoutBoxModelObject,
        to: *mut LayoutBoxModelObject,
        start_child: *mut LayoutObject,
        end_child: *mut LayoutObject,
        before_child: *mut LayoutObject,
        full_remove_insert: bool,
    );
    fn DispatchLayoutBoxModelObjectCanMergeWith(
        owner: *const LayoutBoxModelObject,
        other: &LayoutBoxModelObject,
    ) -> bool;
    fn DispatchLayoutBoxModelObjectContainingBlockLogicalWidthForContent(
        owner: *const LayoutBoxModelObject,
    ) -> LayoutUnit;
}

impl LayoutBoxModelObject {
    // cpp: layoutng/internal/layout_box_model_object.h:298-303
    pub fn MoveChildToDefault(
        &mut self,
        to_box_model_object: *mut LayoutBoxModelObject,
        child: *mut LayoutObject,
        full_remove_insert: bool,
    ) {
        self.CheckIsNotDestroyed();
        self.MoveChildTo(
            to_box_model_object,
            child,
            std::ptr::null_mut(),
            full_remove_insert,
        );
    }

    // cpp: layoutng/internal/layout_box_model_object.h:304-317
    pub fn MoveAllChildrenTo(
        &mut self,
        to_box_model_object: *mut LayoutBoxModelObject,
        full_remove_insert: bool,
    ) {
        self.CheckIsNotDestroyed();
        self.MoveAllChildrenToBefore(
            to_box_model_object,
            std::ptr::null_mut(),
            full_remove_insert,
        );
    }

    pub fn MoveAllChildrenToBefore(
        &mut self,
        to_box_model_object: *mut LayoutBoxModelObject,
        before_child: *mut LayoutObject,
        full_remove_insert: bool,
    ) {
        self.CheckIsNotDestroyed();
        self.MoveChildrenTo(
            to_box_model_object,
            self.SlowFirstChild(),
            std::ptr::null_mut(),
            before_child,
            full_remove_insert,
        );
    }

    pub fn MoveChildrenToDefault(
        &mut self,
        to_box_model_object: *mut LayoutBoxModelObject,
        start_child: *mut LayoutObject,
        end_child: *mut LayoutObject,
        full_remove_insert: bool,
    ) {
        self.CheckIsNotDestroyed();
        self.MoveChildrenTo(
            to_box_model_object,
            start_child,
            end_child,
            std::ptr::null_mut(),
            full_remove_insert,
        );
    }

    pub fn MoveChildrenTo(
        &mut self,
        to_box_model_object: *mut LayoutBoxModelObject,
        start_child: *mut LayoutObject,
        end_child: *mut LayoutObject,
        before_child: *mut LayoutObject,
        full_remove_insert: bool,
    ) {
        self.CheckIsNotDestroyed();
        unsafe {
            DispatchLayoutBoxModelObjectMoveChildrenTo(
                self,
                to_box_model_object,
                start_child,
                end_child,
                before_child,
                full_remove_insert,
            )
        };
    }

    // cpp: layoutng/internal/layout_box_model_object.h:350-353
    pub fn CanMergeWithBase(&self, _other: &LayoutBoxModelObject) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn CanMergeWith(&self, other: &LayoutBoxModelObject) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutBoxModelObjectCanMergeWith(self, other) }
    }

    pub fn ContainingBlockLogicalWidthForContent(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutBoxModelObjectContainingBlockLogicalWidthForContent(self) }
    }

    // cpp: layoutng/internal/layout_box_model_tree.cc:54-75
    pub fn MoveChildTo(
        &mut self,
        to_box_model_object: *mut LayoutBoxModelObject,
        child: *mut LayoutObject,
        before_child: *mut LayoutObject,
        full_remove_insert: bool,
    ) {
        self.CheckIsNotDestroyed();
        debug_assert_eq!(
            self as *mut LayoutBoxModelObject as *mut LayoutObject,
            unsafe { &*child }.Parent()
        );
        debug_assert!(
            before_child.is_null()
                || to_box_model_object as *mut LayoutObject == unsafe { &*before_child }.Parent()
        );
        let children = self.VirtualChildren();
        if full_remove_insert
            && (unsafe { &*to_box_model_object }.IsLayoutBlock()
                || unsafe { &*to_box_model_object }.IsLayoutInline())
        {
            let removed = unsafe { &mut *children }
                .RemoveChildNodeDefault(self as *mut _ as *mut LayoutObject, child);
            unsafe { &mut *to_box_model_object }.AddChild(removed, before_child);
        } else {
            let removed = unsafe { &mut *children }.RemoveChildNode(
                self as *mut _ as *mut LayoutObject,
                child,
                full_remove_insert,
            );
            let target_children = unsafe { &*to_box_model_object }.VirtualChildren();
            unsafe { &mut *target_children }.InsertChildNode(
                to_box_model_object as *mut LayoutObject,
                removed,
                before_child,
                full_remove_insert,
            );
        }
    }

    // Base virtual implementation.
    // cpp: layoutng/internal/layout_box_model_tree.cc:77-92
    pub fn MoveChildrenToBase(
        &mut self,
        to_box_model_object: *mut LayoutBoxModelObject,
        start_child: *mut LayoutObject,
        end_child: *mut LayoutObject,
        before_child: *mut LayoutObject,
        full_remove_insert: bool,
    ) {
        self.CheckIsNotDestroyed();
        debug_assert!(
            before_child.is_null()
                || to_box_model_object as *mut LayoutObject == unsafe { &*before_child }.Parent()
        );
        let mut child = start_child;
        while !child.is_null() && child != end_child {
            let next_sibling = unsafe { &*child }.NextSibling();
            self.MoveChildTo(to_box_model_object, child, before_child, full_remove_insert);
            child = next_sibling;
        }
    }

    // cpp: layoutng/internal/layout_box_model_tree.cc:94-141
    pub fn SplitAnonymousBoxesAroundChild(
        &mut self,
        mut before_child: *mut LayoutObject,
    ) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        let mut box_at_top_of_new_branch: *mut LayoutBox = std::ptr::null_mut();
        while unsafe { &*before_child }.Parent() != self as *mut _ as *mut LayoutObject {
            let box_to_split = To::<LayoutBox>(unsafe { &*before_child }.Parent());
            if unsafe { &*box_to_split }.SlowFirstChild() != before_child
                && unsafe { &*box_to_split }.IsAnonymous()
            {
                let post_box = self.CreateAnonymousBoxToSplit(box_to_split);
                unsafe { &mut *post_box }
                    .SetChildrenInline(unsafe { &*box_to_split }.ChildrenInline());
                let parent_box = To::<LayoutBoxModelObject>(unsafe { &*box_to_split }.Parent());
                MarkBoxForRelayoutAfterSplit(parent_box);
                let parent_children = unsafe { &*parent_box }.VirtualChildren();
                unsafe { &mut *parent_children }.InsertChildNodeDefault(
                    parent_box as *mut LayoutObject,
                    post_box as *mut LayoutObject,
                    unsafe { &*box_to_split }.NextSibling(),
                );
                unsafe { &mut *box_to_split }.MoveChildrenTo(
                    post_box as *mut LayoutBoxModelObject,
                    before_child,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    true,
                );

                let mut child = unsafe { &*post_box }.SlowFirstChild();
                debug_assert!(!child.is_null());
                if !child.is_null() && unsafe { &*child }.NextSibling().is_null() {
                    CollapseLoneAnonymousBlockChild(post_box, child);
                }
                child = unsafe { &*box_to_split }.SlowFirstChild();
                debug_assert!(!child.is_null());
                if !child.is_null() && unsafe { &*child }.NextSibling().is_null() {
                    CollapseLoneAnonymousBlockChild(box_to_split, child);
                }
                MarkBoxForRelayoutAfterSplit(box_to_split as *mut LayoutBoxModelObject);
                MarkBoxForRelayoutAfterSplit(post_box as *mut LayoutBoxModelObject);
                box_at_top_of_new_branch = post_box;
                before_child = post_box as *mut LayoutObject;
            } else {
                before_child = box_to_split as *mut LayoutObject;
            }
        }
        if !box_at_top_of_new_branch.is_null() {
            MarkBoxForRelayoutAfterSplit(self);
        }
        debug_assert_eq!(
            unsafe { &*before_child }.Parent(),
            self as *mut _ as *mut LayoutObject
        );
        before_child
    }

    // cpp: layoutng/internal/layout_box_model_tree.cc:143-147
    pub fn CreateAnonymousBoxToSplit(&self, box_to_split: *const LayoutBox) -> *mut LayoutBox {
        self.CheckIsNotDestroyed();
        unsafe { &*box_to_split }.CreateAnonymousBoxWithSameTypeAs(
            self as *const LayoutBoxModelObject as *const LayoutObject,
        )
    }

    // cpp: layoutng/internal/layout_box_model_tree.cc:149-179
    pub fn AttemptToMerge(prev: *mut LayoutBoxModelObject, next: *mut LayoutBoxModelObject) {
        if prev.is_null() || !unsafe { &*prev }.IsAnonymous() {
            return;
        }
        if next.is_null() || !unsafe { &*next }.IsAnonymous() {
            return;
        }
        debug_assert_eq!(unsafe { &*prev }.NextSibling(), next as *mut LayoutObject);
        debug_assert_eq!(
            unsafe { &*prev }.CanMergeWith(unsafe { &*next }),
            unsafe { &*next }.CanMergeWith(unsafe { &*prev })
        );
        if !unsafe { &*prev }.CanMergeWith(unsafe { &*next }) {
            return;
        }
        let last_child = DynamicTo::<LayoutBoxModelObject>(unsafe { &*prev }.SlowLastChild());
        let first_child = DynamicTo::<LayoutBoxModelObject>(unsafe { &*next }.SlowFirstChild());
        unsafe { &mut *next }.MoveAllChildrenTo(prev, true);
        unsafe { &mut *next }.Destroy();
        Self::AttemptToMerge(last_child, first_child);
    }

    // cpp: layoutng/internal/layout_box_model_tree.cc:181-198
    pub fn ComputedPaddingOutsets(&self) -> PhysicalBoxStrut {
        self.CheckIsNotDestroyed();
        let style = self.StyleRef();
        if !style.MayHavePadding() {
            return PhysicalBoxStrut::default();
        }
        let percentage_size = if style.PaddingTop().HasPercent()
            || style.PaddingRight().HasPercent()
            || style.PaddingBottom().HasPercent()
            || style.PaddingLeft().HasPercent()
        {
            self.ContainingBlockLogicalWidthForContent()
        } else {
            LayoutUnit::default()
        };
        PhysicalBoxStrut::new(
            MinimumValueForLength(style.PaddingTop(), percentage_size),
            MinimumValueForLength(style.PaddingRight(), percentage_size),
            MinimumValueForLength(style.PaddingBottom(), percentage_size),
            MinimumValueForLength(style.PaddingLeft(), percentage_size),
        )
    }

    // cpp: layoutng/internal/layout_box_model_tree.cc:200-203
    pub fn ContainingBlockLogicalWidthForContentBase(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        unsafe { &*self.ContainingBlock() }.ContentLogicalWidth()
    }
}
