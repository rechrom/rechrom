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

#[path = "../../../../layout_invalidation/src/native_node_metadata_finalize.rs"]
mod layout_invalidation_native_node_metadata_finalize;
#[path = "../../../../layout_invalidation/src/native_node_metadata_incremental_relations.rs"]
mod layout_invalidation_native_node_metadata_incremental_relations;

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
        layout_invalidation_native_node_metadata_incremental_relations::AttachIncrementalRelations(
            node,
        )
    }

    // cpp: layoutng/internal/boundary/node_metadata_input.cc:146-263
    pub fn FinalizeTreeRelations(root: &mut Node) -> Result<(), MetadataRelationsError> {
        layout_invalidation_native_node_metadata_finalize::FinalizeTreeRelations(root)
    }
}
