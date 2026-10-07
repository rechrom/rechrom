#![allow(non_snake_case)]

use foundation::{HeapVector, Traceable, Visitor, WtfSizeT};
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_style::style::computed_style_initial_values::ComputedStyleInitialValues;

// cpp: layoutng_flex/flex_child_iterator.h:36-48
pub struct ChildWithOrder {
    pub child: BlockNode,
    pub order: i32,
}

impl ChildWithOrder {
    pub fn new(child: BlockNode, order: i32) -> Self {
        Self { child, order }
    }

    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.child.GetLayoutBox());
    }
}

impl Traceable for ChildWithOrder {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        ChildWithOrder::Trace(self, visitor);
    }
}

// cpp: layoutng_flex/flex_child_iterator.h:21-58
pub struct FlexChildIterator {
    children_: HeapVector<ChildWithOrder, 4>,
    position_: usize,
}

impl FlexChildIterator {
    // cpp: layoutng_flex/flex_child_iterator.cc:12-37
    pub fn new(node: BlockNode) -> Self {
        let is_deprecated_webkit_box = unsafe { &*node.GetLayoutBox() }
            .StyleRef()
            .IsDeprecatedFlexbox();
        let initial_order = if is_deprecated_webkit_box {
            ComputedStyleInitialValues::InitialBoxOrdinalGroup() as i32
        } else {
            ComputedStyleInitialValues::InitialOrder()
        };
        let mut children = HeapVector::<ChildWithOrder, 4>::new();
        let mut needs_sort = false;
        let mut child = node.FirstChild();
        while child.is_non_null() {
            let style = unsafe { &*child.GetLayoutBox() }.StyleRef();
            let order = if is_deprecated_webkit_box {
                style.BoxOrdinalGroup() as i32
            } else {
                style.Order()
            };
            needs_sort |= order != initial_order;
            children.push(ChildWithOrder::new(BlockNode::from(child.clone()), order));
            child = child.NextSibling();
        }
        if needs_sort {
            children.sort_by_key(|entry| entry.order);
        }
        Self {
            children_: children,
            position_: 0,
        }
    }

    // cpp: layoutng_flex/flex_child_iterator.h:29-34
    pub fn NextChild(&mut self) -> BlockNode {
        debug_assert!(self.position_ <= self.children_.len());
        if self.position_ == self.children_.len() {
            return BlockNode::null();
        }
        let child = self.children_[self.position_].child.clone();
        self.position_ += 1;
        child
    }

    // cpp: layoutng_flex/flex_child_iterator.h:36
    pub fn size(&self) -> WtfSizeT {
        self.children_.len() as WtfSizeT
    }
}
