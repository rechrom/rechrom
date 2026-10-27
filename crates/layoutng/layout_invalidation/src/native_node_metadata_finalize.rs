//! Native metadata relation finalization.
//!
//! Included by `boundary::node_metadata_input` at the reciprocal assembly
//! boundary. The transaction performs several whole-tree passes, including
//! implicit-anchor resolution, and therefore owns one invalidation-qualified
//! sampling frame rather than accruing to the boundary facade.

use super::*;

// cpp: layoutng/internal/boundary/node_metadata_input.cc:146-263
#[inline(never)]
pub(super) fn FinalizeTreeRelations(root: &mut Node) -> Result<(), MetadataRelationsError> {
    let mut nodes: Vec<*mut Node> = Vec::new();
    let mut pending: Vec<*mut Node> = vec![root];
    while let Some(node) = pending.pop() {
        nodes.push(node);
        if !unsafe { &*node }.IsContainerNode() {
            continue;
        }
        let container = unsafe { &*(node.cast::<ContainerNode>()) };
        // Walk sibling links backwards to preserve the preorder stack without
        // allocating a temporary vector for every container.
        let mut child = container.lastChild();
        while !child.is_null() {
            pending.push(child);
            child = unsafe { &*child }.previousSibling();
        }
    }

    let mut document_elements = 0u32;
    let mut bodies = 0u32;
    let mut document_element = std::ptr::null_mut::<Node>();
    let mut body_element = std::ptr::null_mut::<Node>();
    let mut viewport_defining_element: *mut Element = std::ptr::null_mut();
    for &node in &nodes {
        match unsafe { &*node }.InputDocumentRole() {
            DocumentRole::kNone => {}
            DocumentRole::kDocumentElement => {
                document_elements += 1;
                document_element = node;
            }
            DocumentRole::kBody => {
                bodies += 1;
                body_element = node;
            }
            DocumentRole::kDocumentElementAndBody => {
                document_elements += 1;
                bodies += 1;
            }
        }
        if document_elements > 1 || bodies > 1 {
            return Err(MetadataRelationsError::DuplicateDocumentRole);
        }
        if unsafe { &*node }.IsElementNode() {
            let element = unsafe { &mut *node.cast::<Element>() };
            element
                .container
                .node
                .set_implicit_anchor(std::ptr::null_mut());
            element.set_may_be_implicit_anchor(false);
            if element.InputIsViewportDefining() {
                if !viewport_defining_element.is_null() {
                    return Err(MetadataRelationsError::DuplicateViewportDefiningElement);
                }
                viewport_defining_element = element;
            }
        }
    }
    // Document::ViewportDefiningElement: primary body overflow propagates only
    // when html is visible on both axes and neither applies containment. At
    // this boundary the complete native tree and canonical styles exist.
    if !document_element.is_null() && !body_element.is_null() {
        let root_object = unsafe { &*document_element }.GetLayoutObject();
        let body_object = unsafe { &*body_element }.GetLayoutObject();
        if !root_object.is_null()
            && !body_object.is_null()
            && unsafe { &*root_object }
                .StyleRef()
                .IsOverflowVisibleAlongBothAxes()
            && !unsafe { &*root_object }.ShouldApplyAnyContainment()
            && !unsafe { &*body_object }.ShouldApplyAnyContainment()
        {
            viewport_defining_element = body_element.cast::<Element>();
        }
    }
    let previous_viewport = root.InputViewportDefiningElement();
    for &node in &nodes {
        unsafe { &mut *node }.set_viewport_defining_element(viewport_defining_element);
    }
    if previous_viewport != viewport_defining_element {
        // The former and new propagated owners change clipping/area ownership
        // even if their authored styles are unchanged.
        for &node in &nodes {
            if node == document_element
                || node == previous_viewport.cast::<Node>()
                || node == viewport_defining_element.cast::<Node>()
            {
                let object = unsafe { &*node }.GetLayoutObject();
                let box_ = foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(object);
                if !box_.is_null() {
                    unsafe { &mut *box_ }.UpdateFromStyle();
                    let axes = unsafe { &*box_ }.ComputeOverflowClipAxes();
                    unsafe { &mut *box_ }.SetOverflowClipAxes(axes);
                    unsafe { &mut *box_ }.SetNeedsLayoutAndIntrinsicWidthsRecalc(
                        &raw const crate::internal::layout_invalidation_reason::kStyleChange,
                    );
                }
            }
        }
    }

    // Resolve stable public IDs only after all nodes exist.
    for &node in &nodes {
        if !unsafe { &*node }.IsElementNode() {
            continue;
        }
        let owner = node.cast::<Element>();
        let anchor_id = unsafe { &*owner }
            .InputElementData()
            .as_ref()
            .map_or(0, |data| data.implicit_anchor_id);
        if anchor_id == 0 {
            continue;
        }
        let mut found: *mut Element = std::ptr::null_mut();
        for &candidate_node in &nodes {
            if unsafe { &*candidate_node }.InputId() != anchor_id {
                continue;
            }
            if !unsafe { &*candidate_node }.IsElementNode() || !found.is_null() {
                return Err(MetadataRelationsError::ImplicitAnchorIdNotUnique);
            }
            found = candidate_node.cast::<Element>();
        }
        if found.is_null() || found == owner {
            return Err(MetadataRelationsError::ImplicitAnchorMustDiffer);
        }
        unsafe { &mut *owner }
            .container
            .node
            .set_implicit_anchor(found);
        unsafe { &mut *found }.set_may_be_implicit_anchor(true);
    }

    // A generated list marker may occur at most once per parent.
    for &node in &nodes {
        if !unsafe { &*node }.IsContainerNode() {
            continue;
        }
        let container = unsafe { &*node.cast::<ContainerNode>() };
        if unsafe { &*node }.InputKind() == NodeKind::kFrameSet {
            let mut child = container.firstChild();
            while !child.is_null() {
                let kind = unsafe { &*child }.InputKind();
                if kind != NodeKind::kFrame && kind != NodeKind::kFrameSet {
                    return Err(MetadataRelationsError::InvalidFramesetChild);
                }
                child = unsafe { &*child }.nextSibling();
            }
        }
        let mut markers = 0u32;
        let mut child = container.firstChild();
        while !child.is_null() {
            markers += u32::from(unsafe { &*child }.InputKind() == NodeKind::kListMarker);
            child = unsafe { &*child }.nextSibling();
        }
        if markers > 1 {
            return Err(MetadataRelationsError::DuplicateListMarker);
        }
    }

    for &node in &nodes {
        unsafe { &mut *node }.set_global_input_owner(root as *mut Node);
        unsafe { &mut *node }.seal_input_tree();
    }
    Ok(())
}
