#![allow(non_snake_case)]

use foundation::{MakeGarbageCollected, Member};
use layoutng_style::style::computed_style::ComputedStyle;

use super::super::form_control_types::{AutofillState, FormControlType};
use super::super::form_node_metadata::{
    HTMLAreaElement, HTMLFormControlElement, HTMLImageElement, HTMLInputElement,
    SliderThumbElement, SpinButtonElement, TextControlElement, TextControlInnerEditorElement,
};
use super::super::layout_input::{DocumentRole, NativeNodeConstructionData, NodeKind};
use super::super::layout_node_metadata::{
    ContainerNode, Element, ElementMetadataError, NativeNodeMetadataRelations, Node, NodeDowncast,
};
use super::super::layout_object_factory_set::LayoutObjectFactorySet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataFactoryError {
    InvalidInput(&'static str),
    UnsupportedLayout(&'static str),
    RejectedFrameset,
    Element(ElementMetadataError),
}

// cpp: layoutng/internal/boundary/node_metadata_input.cc:17-69
pub fn CreateNativeElementMetadata(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
    factories: &LayoutObjectFactorySet,
) -> Result<*mut Element, MetadataFactoryError> {
    let data = input.element.as_ref();
    if data.is_some_and(|data| data.image_map_area) && input.kind != NodeKind::kBox {
        return Err(MetadataFactoryError::InvalidInput(
            "Image-map area identity requires a box node",
        ));
    }
    let is_image_input = input.kind == NodeKind::kReplaced
        && data.is_some_and(|data| data.form_control_type == Some(FormControlType::kInputImage));
    let is_control =
        input.kind == NodeKind::kFormControl || input.kind == NodeKind::kFieldset || is_image_input;
    let uses_forms_metadata = is_control
        || input.kind == NodeKind::kLegend
        || input.kind == NodeKind::kMarquee
        || input.kind == NodeKind::kSliderThumb
        || data.is_some_and(|data| data.text_control_inner_editor || data.text_control_spin_button);
    if !is_control
        && data.is_some_and(|data| {
            data.form_control_type.is_some()
                || data.range_value_ratio.is_some()
                || data.selected_file_count != 0
                || data.autofill_state != AutofillState::kNotFilled
        })
    {
        return Err(MetadataFactoryError::InvalidInput(
            "Control state requires a form-control node",
        ));
    }
    if data.is_some_and(|data| data.html_image) {
        if (input.kind != NodeKind::kBox && input.kind != NodeKind::kReplaced)
            || data.is_some_and(|data| data.image_map_area || data.first_letter_pseudo)
        {
            return Err(MetadataFactoryError::InvalidInput(
                "HTML image identity requires a box or replaced node without another element identity",
            ));
        }
        let image = HTMLImageElement::new(input, style).map_err(MetadataFactoryError::Element)?;
        return Ok(MakeGarbageCollected(image).cast::<Element>());
    }
    if data.is_some_and(|data| data.image_map_area) {
        let area = HTMLAreaElement::new(input, style).map_err(MetadataFactoryError::Element)?;
        return Ok(MakeGarbageCollected(area).cast::<Element>());
    }
    if uses_forms_metadata {
        let factory = factories
            .forms_metadata
            .ok_or(MetadataFactoryError::UnsupportedLayout(
                "forms metadata module is not installed",
            ))?;
        let form = factory(input, style);
        if !form.is_null() {
            return Ok(form);
        }
    }
    let is_math = (input.kind as i32) >= NodeKind::kMathRow as i32
        && (input.kind as i32) <= NodeKind::kMathTable as i32;
    if is_math {
        let factory = factories
            .mathml_metadata
            .ok_or(MetadataFactoryError::UnsupportedLayout(
                "mathml metadata module is not installed",
            ))?;
        let math = factory(input, style);
        if !math.is_null() {
            return Ok(math);
        }
    }
    if input.kind == NodeKind::kFrameSet || input.kind == NodeKind::kFrame {
        let factory =
            factories
                .frameset_metadata
                .ok_or(MetadataFactoryError::UnsupportedLayout(
                    "frameset layout module is not installed",
                ))?;
        let frame = factory(input, style);
        if !frame.is_null() {
            return Ok(frame);
        }
        return Err(MetadataFactoryError::RejectedFrameset);
    }
    let element = Element::new(input, style).map_err(MetadataFactoryError::Element)?;
    Ok(MakeGarbageCollected(element))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataRelationsError {
    InvalidIncrementalRelation(&'static str),
    DuplicateDocumentRole,
    DuplicateViewportDefiningElement,
    ImplicitAnchorIdNotUnique,
    ImplicitAnchorMustDiffer,
    InvalidFramesetChild,
    DuplicateListMarker,
}

unsafe fn dynamic_node<T: NodeDowncast>(node: *mut Node) -> *mut T {
    if node.is_null() || !T::AllowFrom(unsafe { &*node }) {
        std::ptr::null_mut()
    } else {
        node.cast::<T>()
    }
}

impl NativeNodeMetadataRelations {
    pub(crate) fn ResetForTreeUpdate(node: &mut Node) {
        node.set_control_host(std::ptr::null_mut());
        let text_control = foundation::DynamicTo::<TextControlElement>(&mut *node);
        if !text_control.is_null() {
            unsafe { &mut *text_control }.inner_editor_ = Member::default();
        }
        let input = foundation::DynamicTo::<HTMLInputElement>(&mut *node);
        if !input.is_null() {
            unsafe { &mut *input }.upload_button_ = Member::default();
            unsafe { &mut *input }.spin_button_ = Member::default();
        }
    }

    // cpp: layoutng/internal/boundary/node_metadata_input.cc:74-144
    pub fn AttachIncrementalRelations(node: &mut Node) -> Result<(), MetadataRelationsError> {
        let node_ptr = node as *mut Node;
        let parent = node.parentNode();
        if !parent.is_null() {
            node.set_control_host(unsafe { &(*parent).node }.OwnerShadowHost());
        }
        let element = unsafe { dynamic_node::<Element>(node_ptr) };
        let (control_host_ancestor, file_upload_button, inner_editor, spin_button, placeholder) =
            if element.is_null() {
                (None, false, false, false, false)
            } else {
                unsafe { &*element }.InputElementData().as_ref().map_or(
                    (None, false, false, false, false),
                    |data| {
                        (
                            data.control_host_ancestor,
                            data.file_upload_button,
                            data.text_control_inner_editor,
                            data.text_control_spin_button,
                            data.text_control_placeholder,
                        )
                    },
                )
            };
        if let Some(mut distance) = control_host_ancestor {
            if distance == 0 {
                return Err(MetadataRelationsError::InvalidIncrementalRelation(
                    "Control host must be a strict ancestor",
                ));
            }
            let mut host = node_ptr;
            while distance != 0 && !host.is_null() {
                host = unsafe { &*host }.parentNode().cast::<Node>();
                distance -= 1;
            }
            let control = unsafe { dynamic_node::<HTMLFormControlElement>(host) };
            if control.is_null() {
                return Err(MetadataRelationsError::InvalidIncrementalRelation(
                    "Control host ancestor is not a form control",
                ));
            }
            node.set_control_host(control.cast::<Element>());
        }
        if file_upload_button {
            let button = unsafe { dynamic_node::<HTMLInputElement>(node_ptr) };
            let host =
                unsafe { dynamic_node::<HTMLInputElement>(node.OwnerShadowHost().cast::<Node>()) };
            if button.is_null()
                || unsafe { &*button }
                    .text_control
                    .state
                    .control
                    .FormControlType()
                    != FormControlType::kInputButton
                || host.is_null()
                || unsafe { &*host }
                    .text_control
                    .state
                    .control
                    .FormControlType()
                    != FormControlType::kInputFile
            {
                return Err(MetadataRelationsError::InvalidIncrementalRelation(
                    "File upload button requires input-button identity and an input-file control owner",
                ));
            }
            if !unsafe { &*host }.upload_button_.Get().is_null()
                && unsafe { &*host }.upload_button_.Get() != button
            {
                return Err(MetadataRelationsError::InvalidIncrementalRelation(
                    "A file control has at most one upload button",
                ));
            }
            unsafe { &mut *host }.upload_button_ = Member::from_ptr(button);
        }
        if unsafe { SliderThumbElement::AllowFrom(&*node_ptr) } {
            let range =
                unsafe { dynamic_node::<HTMLInputElement>(node.OwnerShadowHost().cast::<Node>()) };
            if range.is_null()
                || unsafe { &*range }
                    .text_control
                    .state
                    .control
                    .FormControlType()
                    != FormControlType::kInputRange
            {
                return Err(MetadataRelationsError::InvalidIncrementalRelation(
                    "Slider thumb requires an input-range control owner",
                ));
            }
        }
        if inner_editor {
            let editor = unsafe { dynamic_node::<TextControlInnerEditorElement>(node_ptr) };
            let host = unsafe {
                dynamic_node::<TextControlElement>(node.OwnerShadowHost().cast::<Node>())
            };
            if editor.is_null()
                || host.is_null()
                || (!unsafe { &*host }.inner_editor_.Get().is_null()
                    && unsafe { &*host }.inner_editor_.Get() != editor)
            {
                return Err(MetadataRelationsError::InvalidIncrementalRelation(
                    "Inner editor requires one actual text-control host",
                ));
            }
            unsafe { &mut *host }.inner_editor_ = Member::from_ptr(editor);
        }
        if spin_button {
            let button = unsafe { dynamic_node::<SpinButtonElement>(node_ptr) };
            let host =
                unsafe { dynamic_node::<HTMLInputElement>(node.OwnerShadowHost().cast::<Node>()) };
            if button.is_null()
                || host.is_null()
                || (!unsafe { &*host }.spin_button_.Get().is_null()
                    && unsafe { &*host }.spin_button_.Get() != button)
            {
                return Err(MetadataRelationsError::InvalidIncrementalRelation(
                    "Spin button requires one actual input host",
                ));
            }
            unsafe { &mut *host }.spin_button_ = Member::from_ptr(button);
        }
        if placeholder {
            let host = unsafe {
                dynamic_node::<TextControlElement>(node.OwnerShadowHost().cast::<Node>())
            };
            if element.is_null() || host.is_null() {
                return Err(MetadataRelationsError::InvalidIncrementalRelation(
                    "Text-control placeholder requires an actual text-control host",
                ));
            }
        }
        Ok(())
    }

    // cpp: layoutng/internal/boundary/node_metadata_input.cc:146-263
    pub fn FinalizeTreeRelations(root: &mut Node) -> Result<(), MetadataRelationsError> {
        let mut nodes: Vec<*mut Node> = Vec::new();
        let mut pending: Vec<*mut Node> = vec![root];
        while let Some(node) = pending.pop() {
            nodes.push(node);
            if !unsafe { &*node }.IsContainerNode() {
                continue;
            }
            let container = unsafe { &*(node.cast::<ContainerNode>()) };
            // Walk sibling links backwards to preserve the preorder stack
            // without allocating a temporary vector for every container.
            let mut child = container.lastChild();
            while !child.is_null() {
                pending.push(child);
                child = unsafe { &*child }.previousSibling();
            }
        }

        let mut document_elements = 0u32;
        let mut bodies = 0u32;
        let mut viewport_defining_element: *mut Element = std::ptr::null_mut();
        for &node in &nodes {
            match unsafe { &*node }.InputDocumentRole() {
                DocumentRole::kNone => {}
                DocumentRole::kDocumentElement => document_elements += 1,
                DocumentRole::kBody => bodies += 1,
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
        for &node in &nodes {
            unsafe { &mut *node }.set_viewport_defining_element(viewport_defining_element);
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
            unsafe { &mut *node }.seal_input_tree();
        }
        Ok(())
    }
}
