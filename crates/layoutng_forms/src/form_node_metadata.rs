#![allow(non_snake_case)]

use foundation::{BlinkString, MakeGarbageCollected, UnsupportedLayout, Visitor};
use layoutng_assembly::internal::form_control_types::{AutofillState, FormControlType as Type};
use layoutng_assembly::internal::form_node_metadata::{
    FormNodeMetadataExternal, HTMLButtonElement, HTMLDivElement, HTMLElement, HTMLFieldSetElement,
    HTMLFormControlElement, HTMLFormControlElementWithState, HTMLInputElement, HTMLLegendElement,
    HTMLMarqueeElement, HTMLOutputElement, HTMLSelectElement, HTMLTextAreaElement,
    SliderThumbElement, SpinButtonElement, TextControlElement, TextControlInnerEditorElement,
};
use layoutng_assembly::internal::layout_input::{NativeNodeConstructionData, NodeKind};
use layoutng_assembly::internal::layout_node_metadata::{Element, ElementType, Node, NodeDowncast};
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_forms/form_node_metadata.cc:14-16
fn Require(condition: bool, message: &'static str) {
    if !condition {
        panic!("{message}");
    }
}

// cpp: layoutng_forms/form_node_metadata.cc:18-34
fn ControlType(input: &NativeNodeConstructionData) -> Type {
    if input.kind == NodeKind::kFieldset {
        Require(
            input.element.as_ref().is_none_or(|element| {
                element.form_control_type.is_none()
                    || element.form_control_type == Some(Type::kFieldset)
            }),
            "Fieldset input has a conflicting control type",
        );
        return Type::kFieldset;
    }
    let data = input.element.as_ref();
    Require(
        (input.kind == NodeKind::kFormControl
            && data.is_some_and(|element| element.form_control_type.is_some()))
            || (input.kind == NodeKind::kReplaced
                && data
                    .is_some_and(|element| element.form_control_type == Some(Type::kInputImage))),
        "Form-control input requires an explicit control type",
    );
    let control_type = data
        .and_then(|element| element.form_control_type)
        .expect("validated explicit control type");
    Require(
        (control_type as i32) >= Type::kButtonButton as i32
            && (control_type as i32) <= Type::kTextArea as i32
            && control_type != Type::kFieldset,
        "Invalid form-control type; fieldsets use the fieldset node kind",
    );
    control_type
}

// cpp: layoutng_forms/form_node_metadata.cc:35-43
fn IsInputType(control_type: Type) -> bool {
    (control_type as i32) >= Type::kInputButton as i32
        && (control_type as i32) <= Type::kInputWeek as i32
}
fn IsButtonType(control_type: Type) -> bool {
    (control_type as i32) >= Type::kButtonButton as i32
        && (control_type as i32) <= Type::kButtonPopover as i32
}
fn IsSelectType(control_type: Type) -> bool {
    control_type == Type::kSelectOne || control_type == Type::kSelectMultiple
}

fn html_base(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
    element_type: ElementType,
    form_control: bool,
) -> HTMLElement {
    let mut html = HTMLElement::new(input, style).expect("valid HTML element metadata");
    html.SetRuntimeIdentity(element_type, form_control);
    html
}

// cpp: layoutng_forms/form_node_metadata.cc:46-67
fn control_base(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
    element_type: ElementType,
) -> HTMLFormControlElement {
    let html = HTMLElement::new(input, style).expect("valid HTML element metadata");
    let control_type = ControlType(input);
    let autofill = input
        .element
        .as_ref()
        .map_or(AutofillState::kNotFilled, |element| element.autofill_state);
    Require(
        matches!(
            autofill,
            AutofillState::kNotFilled | AutofillState::kPreviewed | AutofillState::kAutofilled
        ),
        "Invalid autofill state",
    );
    Require(
        input.element.as_ref().is_none_or(|element| {
            element.selected_file_count == 0 || control_type == Type::kInputFile
        }),
        "Selected file count requires an input-file control",
    );
    if let Some(value) = input
        .element
        .as_ref()
        .and_then(|element| element.range_value_ratio)
    {
        Require(
            control_type == Type::kInputRange && value.is_finite() && (0.0..=1.0).contains(&value),
            "Range ratio requires an input-range control and a value in [0, 1]",
        );
    }
    if control_type == Type::kInputRange {
        Require(
            input
                .element
                .as_ref()
                .is_some_and(|element| element.range_value_ratio.is_some()),
            "Input-range controls require a resolved value ratio",
        );
    }
    let mut html = html;
    html.SetRuntimeIdentity(element_type, true);
    HTMLFormControlElement::from_resolved_parts(html, control_type, autofill)
}

fn state_base(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
    element_type: ElementType,
) -> HTMLFormControlElementWithState {
    HTMLFormControlElementWithState {
        control: control_base(input, style, element_type),
    }
}

fn text_control_base(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
    element_type: ElementType,
) -> TextControlElement {
    TextControlElement::from_resolved_state(state_base(input, style, element_type))
}

// cpp: layoutng_forms/form_node_metadata.cc:69-71
fn input_element(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
) -> HTMLInputElement {
    let input_node = HTMLInputElement::from_resolved_text_control(text_control_base(
        input,
        style,
        ElementType::kHTMLInputElement,
    ));
    Require(
        IsInputType(input_node.FormControlType()),
        "HTMLInputElement requires an input control",
    );
    input_node
}

// cpp: layoutng_forms/form_node_metadata.cc:72-75,83-86
fn TraceTextControl(element: &TextControlElement, visitor: &mut Visitor) {
    visitor.Trace(element.InnerEditorMember());
    element.state.control.html.element.Trace(visitor);
}
fn TraceInput(input: &HTMLInputElement, visitor: &mut Visitor) {
    visitor.Trace(input.UploadButtonMember());
    visitor.Trace(input.SpinButtonMember());
    TraceTextControl(&input.text_control, visitor);
}

// cpp: layoutng_forms/form_node_metadata.cc:76-82
fn GetSizeWithDecoration(input: &HTMLInputElement, preferred_size: &mut i32) -> bool {
    let sizing = input
        .text_control
        .state
        .control
        .html
        .element
        .InputElementData()
        .as_ref()
        .and_then(|element| element.text_field_sizing.as_ref())
        .unwrap_or_else(|| {
            std::panic::panic_any(UnsupportedLayout::new(
                "Text-field intrinsic width requires ElementData::text_field_sizing",
            ))
        });
    *preferred_size = sizing.preferred_size;
    sizing.includes_decoration
}

// cpp: layoutng_forms/form_node_metadata.cc:87-94
fn FileNoFileLabelForLayout(input: &HTMLInputElement) -> BlinkString {
    assert_eq!(input.FormControlType(), Type::kInputFile);
    let label = input
        .text_control
        .state
        .control
        .html
        .element
        .InputElementData()
        .as_ref()
        .and_then(|element| element.file_no_file_label.as_ref())
        .unwrap_or_else(|| {
            std::panic::panic_any(UnsupportedLayout::new(
                "File control intrinsic width requires ElementData::file_no_file_label",
            ))
        });
    BlinkString::FromUtf8(label.as_bytes())
}

// cpp: layoutng_forms/form_node_metadata.cc:96-108
fn LayoutValueRatio(input: &HTMLInputElement) -> f64 {
    assert_eq!(input.FormControlType(), Type::kInputRange);
    input
        .text_control
        .state
        .control
        .html
        .element
        .InputElementData()
        .as_ref()
        .and_then(|element| element.range_value_ratio)
        .expect("input-range requires a resolved value ratio")
}
fn ShouldApplyMiddleEllipsis(input: &HTMLInputElement) -> bool {
    input.FormControlType() == Type::kInputFile
        && input
            .text_control
            .state
            .control
            .html
            .element
            .InputElementData()
            .as_ref()
            .is_some_and(|element| element.selected_file_count <= 1)
}

// cpp: layoutng_forms/form_node_metadata.cc:109-116
fn select_element(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
) -> HTMLSelectElement {
    let select = HTMLSelectElement {
        state: state_base(input, style, ElementType::kHTMLSelectElement),
    };
    Require(
        IsSelectType(select.state.control.FormControlType()),
        "HTMLSelectElement requires a select control",
    );
    select
}

// cpp: layoutng_forms/form_node_metadata.cc:117-125
fn text_area_element(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
) -> HTMLTextAreaElement {
    let text_control = text_control_base(input, style, ElementType::kHTMLTextAreaElement);
    let sizing = input
        .element
        .as_ref()
        .and_then(|element| element.text_area_sizing.as_ref());
    let textarea = HTMLTextAreaElement::from_resolved_text_control(
        text_control,
        sizing.map_or(2, |value| value.rows),
        sizing.map_or(20, |value| value.columns),
    );
    Require(
        textarea.text_control.state.control.FormControlType() == Type::kTextArea,
        "HTMLTextAreaElement requires a textarea",
    );
    textarea
}

// cpp: layoutng_forms/form_node_metadata.cc:126-140
fn button_element(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
) -> HTMLButtonElement {
    let button = HTMLButtonElement {
        control: control_base(input, style, ElementType::kHTMLButtonElement),
    };
    Require(
        IsButtonType(button.control.FormControlType()),
        "HTMLButtonElement requires a button control",
    );
    button
}
fn fieldset_element(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
) -> HTMLFieldSetElement {
    let fieldset = HTMLFieldSetElement {
        control: control_base(input, style, ElementType::kHTMLFieldSetElement),
    };
    Require(
        fieldset.control.FormControlType() == Type::kFieldset,
        "HTMLFieldSetElement requires a fieldset",
    );
    fieldset
}
fn output_element(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
) -> HTMLOutputElement {
    let output = HTMLOutputElement {
        control: control_base(input, style, ElementType::kHTMLOutputElement),
    };
    Require(
        output.control.FormControlType() == Type::kOutput,
        "HTMLOutputElement requires an output control",
    );
    output
}

// cpp: layoutng_forms/form_node_metadata.cc:141-178
fn legend_element(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
) -> HTMLLegendElement {
    let legend = HTMLLegendElement {
        html: html_base(input, style, ElementType::kHTMLLegendElement, false),
    };
    Require(
        input.kind == NodeKind::kLegend,
        "HTMLLegendElement requires a legend",
    );
    legend
}
fn marquee_element(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
) -> HTMLMarqueeElement {
    let marquee = HTMLMarqueeElement {
        html: html_base(input, style, ElementType::kHTMLMarqueeElement, false),
    };
    Require(
        input.kind == NodeKind::kMarquee,
        "HTMLMarqueeElement requires a marquee",
    );
    marquee
}
fn IsHorizontal(marquee: &HTMLMarqueeElement) -> bool {
    marquee
        .html
        .element
        .InputElementData()
        .as_ref()
        .is_none_or(|element| element.marquee_horizontal)
}
fn slider_thumb_element(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
) -> SliderThumbElement {
    let thumb = SliderThumbElement {
        div: HTMLDivElement {
            html: html_base(input, style, ElementType::kSliderThumbElement, false),
        },
    };
    Require(
        input.kind == NodeKind::kSliderThumb,
        "SliderThumbElement requires a slider thumb",
    );
    thumb
}
fn inner_editor_element(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
) -> TextControlInnerEditorElement {
    let editor = TextControlInnerEditorElement {
        div: HTMLDivElement {
            html: html_base(
                input,
                style,
                ElementType::kTextControlInnerEditorElement,
                false,
            ),
        },
    };
    Require(
        input.kind == NodeKind::kBox
            && input
                .element
                .as_ref()
                .is_some_and(|element| element.text_control_inner_editor),
        "Inner editor requires an explicit box part",
    );
    editor
}
fn spin_button_element(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
) -> SpinButtonElement {
    let button = SpinButtonElement {
        div: HTMLDivElement {
            html: html_base(input, style, ElementType::kSpinButtonElement, false),
        },
    };
    Require(
        input.kind == NodeKind::kBox
            && input
                .element
                .as_ref()
                .is_some_and(|element| element.text_control_spin_button),
        "Spin button requires an explicit box part",
    );
    button
}

// cpp: layoutng_forms/form_node_metadata.cc:180-184
pub fn IsSliderThumb(node: *const Node) -> bool {
    !node.is_null()
        && unsafe { &*node }.IsInUserAgentShadowRoot()
        && <SliderThumbElement as NodeDowncast>::AllowFrom(unsafe { &*node })
}

// cpp: layoutng_forms/form_node_metadata.cc:186-199
pub fn CreateFormElementMetadata(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
) -> *mut HTMLElement {
    let data = input.element.as_ref();
    if data.is_some_and(|element| element.text_control_inner_editor) {
        return MakeGarbageCollected(inner_editor_element(input, style)).cast();
    }
    if data.is_some_and(|element| element.text_control_spin_button) {
        return MakeGarbageCollected(spin_button_element(input, style)).cast();
    }
    match input.kind {
        NodeKind::kFieldset => return MakeGarbageCollected(fieldset_element(input, style)).cast(),
        NodeKind::kLegend => return MakeGarbageCollected(legend_element(input, style)).cast(),
        NodeKind::kMarquee => return MakeGarbageCollected(marquee_element(input, style)).cast(),
        NodeKind::kSliderThumb => {
            return MakeGarbageCollected(slider_thumb_element(input, style)).cast()
        }
        NodeKind::kFormControl => {}
        _ => return std::ptr::null_mut(),
    }
    let control_type = ControlType(input);
    if IsInputType(control_type) {
        return MakeGarbageCollected(input_element(input, style)).cast();
    }
    if IsButtonType(control_type) {
        return MakeGarbageCollected(button_element(input, style)).cast();
    }
    if IsSelectType(control_type) {
        return MakeGarbageCollected(select_element(input, style)).cast();
    }
    if control_type == Type::kTextArea {
        return MakeGarbageCollected(text_area_element(input, style)).cast();
    }
    assert_eq!(control_type, Type::kOutput);
    MakeGarbageCollected(output_element(input, style)).cast()
}

pub struct FormNodeMetadataProvider;

impl FormNodeMetadataExternal for FormNodeMetadataProvider {
    fn TraceTextControl(element: &TextControlElement, visitor: &mut Visitor) {
        TraceTextControl(element, visitor)
    }
    fn LayoutValueRatio(input: &HTMLInputElement) -> f64 {
        LayoutValueRatio(input)
    }
    fn GetSizeWithDecoration(input: &HTMLInputElement, preferred_size: &mut i32) -> bool {
        GetSizeWithDecoration(input, preferred_size)
    }
    fn ShouldApplyMiddleEllipsis(input: &HTMLInputElement) -> bool {
        ShouldApplyMiddleEllipsis(input)
    }
    fn FileNoFileLabelForLayout(input: &HTMLInputElement) -> BlinkString {
        FileNoFileLabelForLayout(input)
    }
    fn TraceInput(input: &HTMLInputElement, visitor: &mut Visitor) {
        TraceInput(input, visitor)
    }
    fn IsHorizontal(marquee: &HTMLMarqueeElement) -> bool {
        IsHorizontal(marquee)
    }
    fn CreateFormElementMetadata(
        input: &NativeNodeConstructionData,
        style: *const ComputedStyle,
    ) -> *mut HTMLElement {
        CreateFormElementMetadata(input, style)
    }
    fn IsSliderThumb(node: *const Node) -> bool {
        IsSliderThumb(node)
    }
}

// The callback stored in LayoutObjectFactorySet uses Element*, while the
// source implementation returns the covariant HTMLElement*.
pub fn CreateFormsMetadata(
    input: &NativeNodeConstructionData,
    style: *const ComputedStyle,
) -> *mut Element {
    CreateFormElementMetadata(input, style).cast()
}
