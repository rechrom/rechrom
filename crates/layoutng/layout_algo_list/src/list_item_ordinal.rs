use std::cell::Cell;

use layoutng_assembly::internal::layout_node_metadata::{ContainerNode, Node};
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};

use crate::layout_inline_list_item::LayoutInlineListItem;
use crate::layout_list_item::LayoutListItem;

// cpp: core/html/list_item_ordinal.h:112-119
#[derive(Default)]
pub struct ListItemOrdinal {
    value: Cell<i32>,
    needs_update: Cell<bool>,
}

impl ListItemOrdinal {
    // cpp: layoutng_list/list_item_ordinal_core.cc:9-9
    pub fn new() -> Self {
        Self {
            needs_update: Cell::new(true),
            ..Self::default()
        }
    }

    // Source: core/html/list_item_ordinal.cc:211-280. Rechrom's matching
    // RuntimeEnabledFeatures branch has CSSListCounterAccounting disabled, so
    // counter-set/reset accounting is intentionally excluded here just as it
    // is by Chromium in that configuration.
    pub fn Value(&self, node: &Node) -> i32 {
        if !self.needs_update.get() {
            return self.value.get();
        }
        let value = CalculateValue(node);
        self.value.set(value);
        self.needs_update.set(false);
        value
    }

    // cpp: layoutng_list/list_item_ordinal.h:62-63
    pub fn ItemInsertedOrRemoved(
        item: *const layoutng_assembly::internal::layout_object::LayoutObject,
    ) {
        unsafe { ListItemOrdinalItemInsertedOrRemoved(item) }
    }
}

unsafe extern "Rust" {
    fn ListItemOrdinalItemInsertedOrRemoved(
        item: *const layoutng_assembly::internal::layout_object::LayoutObject,
    );
}

fn IsListOwner(node: &Node) -> bool {
    if node.InputIsHTMLListOwner() && !node.GetLayoutObject().is_null() {
        return true;
    }
    let object = node.GetLayoutObject();
    !object.is_null() && unsafe { &*object }.ShouldApplyStyleContainment()
}

fn EnclosingList(item: &Node) -> *mut Node {
    let mut first: *mut Node = std::ptr::null_mut();
    let mut parent = item.parentNode();
    while !parent.is_null() {
        let node = unsafe { &mut (*parent).node } as *mut Node;
        if first.is_null() {
            first = node;
        }
        if IsListOwner(unsafe { &*node }) {
            return node;
        }
        parent = unsafe { &*node }.parentNode();
    }
    first
}

fn Previous(node: *mut Node, boundary: *mut Node) -> *mut Node {
    if node.is_null() || node == boundary {
        return std::ptr::null_mut();
    }
    let previous = unsafe { &*node }.previousSibling();
    if !previous.is_null() {
        let mut candidate = previous;
        loop {
            if !unsafe { &*candidate }.IsContainerNode() {
                return candidate;
            }
            let last = unsafe { &*candidate.cast::<ContainerNode>() }.lastChild();
            if last.is_null() {
                return candidate;
            }
            candidate = last;
        }
    }
    let parent = unsafe { &*node }.parentNode();
    if parent.is_null() {
        std::ptr::null_mut()
    } else {
        let parent = unsafe { &mut (*parent).node } as *mut Node;
        (parent != boundary).then_some(parent).unwrap_or(std::ptr::null_mut())
    }
}

fn Next(node: *mut Node, boundary: *mut Node, skip_children: bool) -> *mut Node {
    if node.is_null() {
        return std::ptr::null_mut();
    }
    if !skip_children && unsafe { &*node }.IsContainerNode() {
        let child = unsafe { &*node.cast::<ContainerNode>() }.firstChild();
        if !child.is_null() {
            return child;
        }
    }
    let mut current = node;
    while current != boundary {
        let sibling = unsafe { &*current }.nextSibling();
        if !sibling.is_null() {
            return sibling;
        }
        let parent = unsafe { &*current }.parentNode();
        if parent.is_null() {
            break;
        }
        current = unsafe { &mut (*parent).node } as *mut Node;
    }
    std::ptr::null_mut()
}

fn Ordinal(node: *mut Node) -> *mut ListItemOrdinal {
    if node.is_null() {
        return std::ptr::null_mut();
    }
    let object = unsafe { &*node }.GetLayoutObject();
    if object.is_null() {
        return std::ptr::null_mut();
    }
    match unsafe { &*object }.RuntimeClass() {
        LayoutObjectClass::ListItem => unsafe { &mut *object.cast::<LayoutListItem>() }.Ordinal(),
        LayoutObjectClass::InlineListItem => {
            unsafe { &mut *object.cast::<LayoutInlineListItem>() }.Ordinal()
        }
        _ => std::ptr::null_mut(),
    }
}

fn PreviousListItem(list: *mut Node, item: *mut Node) -> (*mut Node, *mut ListItemOrdinal) {
    let mut current = Previous(item, list);
    while !current.is_null() {
        let ordinal = Ordinal(current);
        if !ordinal.is_null() {
            let other_list = EnclosingList(unsafe { &*current });
            if other_list == list {
                return (current, ordinal);
            }
            if !other_list.is_null() {
                current = Previous(other_list, list);
                continue;
            }
        }
        current = Previous(current, list);
    }
    (std::ptr::null_mut(), std::ptr::null_mut())
}

fn ListItemCount(list: *mut Node) -> i32 {
    let mut count = 0i64;
    let mut current = Next(list, list, false);
    while !current.is_null() {
        if IsListOwner(unsafe { &*current }) {
            current = Next(current, list, true);
            continue;
        }
        if !Ordinal(current).is_null() && EnclosingList(unsafe { &*current }) == list {
            count += 1;
        }
        current = Next(current, list, false);
    }
    count.min(i64::from(i32::MAX)) as i32
}

fn CalculateValue(node: &Node) -> i32 {
    if let Some(explicit) = node.InputListItemExplicitValue() {
        return explicit;
    }
    let list = EnclosingList(node);
    let reversed = !list.is_null() && unsafe { &*list }.InputOrderedListReversed();
    let step = if reversed { -1i64 } else { 1i64 };
    let (previous_node, previous_ordinal) =
        PreviousListItem(list, node as *const Node as *mut Node);
    let base = if !previous_ordinal.is_null() {
        i64::from(unsafe { &*previous_ordinal }.Value(unsafe { &*previous_node }))
    } else if list.is_null() {
        0
    } else if let Some(start) = unsafe { &*list }.InputOrderedListStart() {
        i64::from(start) - step
    } else if reversed {
        i64::from(ListItemCount(list)) - step
    } else {
        0
    };
    (base + step).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}
