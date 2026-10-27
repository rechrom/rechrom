//! Native generated list-marker lookup.
//!
//! Included by `layout_node_metadata` at the reciprocal assembly boundary.
//! The child scan remains a list-algorithm sampling frame while the element
//! API stays source-compatible.

use super::*;

// cpp: layoutng/internal/layout_node_metadata.cc:463-473
#[inline(never)]
pub(super) fn InputListMarker(element: &Element) -> *mut Element {
    let mut child = element.container.firstChild();
    while !child.is_null() {
        // SAFETY: child links are owned by the native metadata tree and
        // list-marker nodes have element identity.
        let child_ref = unsafe { &*child };
        if child_ref.InputKind() == NodeKind::kListMarker {
            assert!(child_ref.IsElementNode());
            return child.cast::<Element>();
        }
        child = child_ref.nextSibling();
    }
    std::ptr::null_mut()
}
