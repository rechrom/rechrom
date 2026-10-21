#![allow(non_snake_case)]

use foundation::{MakeGarbageCollected, To};
use layoutng_assembly::internal::algorithm_entry::NativeAlgorithmEntry;
use layoutng_assembly::internal::form_control_types::FormControlType;
use layoutng_assembly::internal::layout_input::{NativeNodeConstructionData, NodeKind};
use layoutng_assembly::internal::layout_node_metadata::{Element, Node};
use layoutng_assembly::internal::layout_object::LayoutObject;
use layoutng_assembly::layout_assembly::LayoutAssembly;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::control_intrinsic_size::{ControlIntrinsicBlockSize, ControlIntrinsicInlineSize};
use crate::fieldset_layout_algorithm::FieldsetLayoutAlgorithm;
use crate::form_node_metadata::CreateFormElementMetadata;
use crate::inner_editor_tree::RemoveInnerEditorChild;
use crate::layout_fieldset::{
    FieldsetAddChild, FieldsetContentLayoutBox, FieldsetInsertedIntoTree, FieldsetScrollHeight,
    FieldsetScrollWidth, FieldsetUpdateAnonymousChildStyle, LayoutFieldset,
};
use crate::layout_text_control_inner_editor::LayoutTextControlInnerEditor;
use crate::layout_text_control_inner_editor::{
    InnerEditorAddChild, InnerEditorAllowsInlineChildren, InnerEditorStyleDidChange,
};
use crate::layout_text_control_multi_line::LayoutTextControlMultiLine;
use crate::layout_text_control_single_line::LayoutTextControlSingleLine;

// cpp: layoutng_forms/assembly.cc:17-44
fn CreateFormsObject(node: &mut Node, _style: &ComputedStyle) -> *mut LayoutObject {
    let element = To::<Element>(&mut *node);
    if unsafe { &*element }
        .InputElementData()
        .as_ref()
        .is_some_and(|data| data.text_control_inner_editor)
    {
        return MakeGarbageCollected(LayoutTextControlInnerEditor::new(element)).cast();
    }
    if node.InputKind() == NodeKind::kFieldset {
        return MakeGarbageCollected(LayoutFieldset::new(element)).cast();
    }
    assert_eq!(node.InputKind(), NodeKind::kFormControl);
    let data = unsafe { &*element }.InputElementData().as_ref();
    assert!(data.is_some_and(|data| data.form_control_type.is_some()));
    match data.unwrap().form_control_type.unwrap() {
        FormControlType::kTextArea => {
            MakeGarbageCollected(LayoutTextControlMultiLine::new(element)).cast()
        }
        FormControlType::kInputText
        | FormControlType::kInputEmail
        | FormControlType::kInputPassword
        | FormControlType::kInputSearch
        | FormControlType::kInputTelephone
        | FormControlType::kInputUrl
        | FormControlType::kInputNumber => {
            MakeGarbageCollected(LayoutTextControlSingleLine::new(element)).cast()
        }
        _ => std::ptr::null_mut(),
    }
}

// cpp: layoutng_forms/assembly.cc:45-49
fn CreateFormsMetadata(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
) -> *mut Element {
    CreateFormElementMetadata(input, style).cast()
}

// cpp: layoutng_forms/assembly.h:3-3
// cpp: layoutng_forms/assembly.cc:52-61
pub fn InstallFormsAlgorithm(assembly: &mut LayoutAssembly) {
    assembly.objects.forms = Some(CreateFormsObject);
    assembly.objects.forms_metadata = Some(CreateFormsMetadata);
    assembly.objects.remove_inner_editor_child = Some(RemoveInnerEditorChild);
    assembly.objects.inner_editor_allows_inline_children = Some(InnerEditorAllowsInlineChildren);
    assembly.objects.inner_editor_add_child = Some(InnerEditorAddChild);
    assembly.objects.inner_editor_style_did_change = Some(InnerEditorStyleDidChange);
    assembly.objects.fieldset_add_child = Some(FieldsetAddChild);
    assembly.objects.fieldset_inserted_into_tree = Some(FieldsetInsertedIntoTree);
    assembly.objects.fieldset_update_anonymous_child_style =
        Some(FieldsetUpdateAnonymousChildStyle);
    assembly.objects.fieldset_content_layout_box = Some(FieldsetContentLayoutBox);
    assembly.objects.fieldset_scroll_width = Some(FieldsetScrollWidth);
    assembly.objects.fieldset_scroll_height = Some(FieldsetScrollHeight);
    assembly.algorithms.fieldset = NativeAlgorithmEntry::<FieldsetLayoutAlgorithm>();
    assembly.algorithms.forms_support.intrinsic_inline_size = Some(ControlIntrinsicInlineSize);
    assembly.algorithms.forms_support.intrinsic_block_size = Some(ControlIntrinsicBlockSize);
}
