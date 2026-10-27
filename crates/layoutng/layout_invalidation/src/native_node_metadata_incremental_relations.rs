//! Incremental native metadata relation attachment.
//!
//! Included by `boundary::node_metadata_input` at the reciprocal assembly
//! boundary. Control-host ancestry and generated form-control relationships
//! are mutation policy, so the complete admission path owns an invalidation
//! sampling frame.

use super::*;

// cpp: layoutng/internal/boundary/node_metadata_input.cc:74-144
#[inline(never)]
pub(super) fn AttachIncrementalRelations(node: &mut Node) -> Result<(), MetadataRelationsError> {
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
        let host =
            unsafe { dynamic_node::<TextControlElement>(node.OwnerShadowHost().cast::<Node>()) };
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
        let host =
            unsafe { dynamic_node::<TextControlElement>(node.OwnerShadowHost().cast::<Node>()) };
        if element.is_null() || host.is_null() {
            return Err(MetadataRelationsError::InvalidIncrementalRelation(
                "Text-control placeholder requires an actual text-control host",
            ));
        }
    }
    Ok(())
}
