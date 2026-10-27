#![allow(non_snake_case)]

use super::{Element, Node};

impl Node {
    #[inline(never)]
    pub fn parentElement(&self) -> *mut Element {
        let mut parent = self.parent_.Get();
        while !parent.is_null() {
            // SAFETY: parent links point to live metadata nodes owned by the
            // native tree. Element begins with ContainerNode at offset zero.
            let node = unsafe { &(*parent).node };
            if node.IsElementNode() {
                return parent.cast::<Element>();
            }
            parent = node.parent_.Get();
        }
        std::ptr::null_mut()
    }
}
