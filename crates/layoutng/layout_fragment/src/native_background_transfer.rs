//! Root-canvas background transfer policy and retained metadata traversal.

use layoutng_style::style::computed_style::ComputedStyle;

use super::super::layout_box_model_object::LayoutBoxModelObject;
use super::super::layout_input::DocumentRole;
use super::super::layout_node_metadata::{ContainerNode, Node};

// Source: third_party/blink/renderer/core/layout/layout_box_model_object.cc:1123-1151.
#[unsafe(no_mangle)]
#[inline(never)]
pub(super) fn LayoutBoxModelObjectBackgroundTransfersToView(
    this: &LayoutBoxModelObject,
    document_style: *const ComputedStyle,
) -> bool {
    if this.IsDocumentElement() {
        return true;
    }
    if !this.IsBody() {
        return false;
    }

    // NativeNodeMetadataRelations validates a single document element and a
    // single primary body role. Walk the retained metadata tree rather than
    // reaching back into DOM ownership.
    unsafe fn FindDocumentElement(node: *mut Node) -> *mut Node {
        if node.is_null() {
            return std::ptr::null_mut();
        }
        let node_ref = unsafe { &*node };
        if matches!(
            node_ref.InputDocumentRole(),
            DocumentRole::kDocumentElement | DocumentRole::kDocumentElementAndBody
        ) {
            return node;
        }
        if !node_ref.IsContainerNode() {
            return std::ptr::null_mut();
        }
        let mut child = unsafe { &*node.cast::<ContainerNode>() }.firstChild();
        while !child.is_null() {
            let found = unsafe { FindDocumentElement(child) };
            if !found.is_null() {
                return found;
            }
            child = unsafe { &*child }.nextSibling();
        }
        std::ptr::null_mut()
    }

    let mut root = this.GetNode();
    if root.is_null() {
        return false;
    }
    loop {
        let parent = unsafe { &*root }.parentNode();
        if parent.is_null() {
            break;
        }
        root = unsafe { &mut (*parent).node };
    }
    let document_element = unsafe { FindDocumentElement(root) };
    if document_element.is_null() || !unsafe { &*document_element }.IsHTMLElement() {
        return false;
    }

    let document_object = unsafe { &*document_element }.GetLayoutObject();
    if document_object.is_null() {
        return false;
    }
    let document_style = if document_style.is_null() {
        unsafe { &*document_element }.GetComputedStyle()
    } else {
        document_style
    };
    if document_style.is_null() || unsafe { &*document_style }.HasBackground() {
        return false;
    }
    let document_object = unsafe { &*document_object };
    let document_style = unsafe { &*document_style };
    let document_has_containment = document_object
        .ShouldApplyPaintContainmentWithStyle(document_style)
        || document_object.ShouldApplyLayoutContainmentWithStyle(document_style)
        || document_style.ContainsStyle()
        || ((document_style.ContainsBlockSize() || document_style.ContainsInlineSize())
            && document_object.IsEligibleForSizeContainment());
    if document_has_containment {
        return false;
    }
    !this.ShouldApplyAnyContainment()
}
