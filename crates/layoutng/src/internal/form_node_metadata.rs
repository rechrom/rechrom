#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{Member, String as BlinkString, Visitor};
use layoutng_style::style::computed_style::ComputedStyle;

use super::form_control_types::{AutofillState, FormControlType};
use super::layout_input::NativeNodeConstructionData;
use super::layout_node_metadata::{Element, ElementMetadataError, ElementType, Node, NodeDowncast};

// Every form metadata subclass embeds its C++ public base as its first field.
// cpp: layoutng/internal/form_node_metadata.h:15-163
macro_rules! impl_form_base {
    ($type:ty, $field:ident, $base:ty) => {
        impl Deref for $type {
            type Target = $base;
            fn deref(&self) -> &$base {
                &self.$field
            }
        }
        impl DerefMut for $type {
            fn deref_mut(&mut self) -> &mut $base {
                &mut self.$field
            }
        }
        const _: () = assert!(std::mem::offset_of!($type, $field) == 0);
    };
}

impl_form_base!(HTMLElement, element, Element);
impl_form_base!(HTMLAreaElement, html, HTMLElement);
impl_form_base!(HTMLImageElement, html, HTMLElement);
impl_form_base!(HTMLFormControlElement, html, HTMLElement);
impl_form_base!(
    HTMLFormControlElementWithState,
    control,
    HTMLFormControlElement
);
impl_form_base!(TextControlElement, state, HTMLFormControlElementWithState);
impl_form_base!(HTMLInputElement, text_control, TextControlElement);
impl_form_base!(HTMLSelectElement, state, HTMLFormControlElementWithState);
impl_form_base!(HTMLTextAreaElement, text_control, TextControlElement);
impl_form_base!(HTMLButtonElement, control, HTMLFormControlElement);
impl_form_base!(HTMLFieldSetElement, control, HTMLFormControlElement);
impl_form_base!(HTMLOutputElement, control, HTMLFormControlElement);
impl_form_base!(HTMLLegendElement, html, HTMLElement);
impl_form_base!(HTMLMarqueeElement, html, HTMLElement);
impl_form_base!(HTMLDivElement, html, HTMLElement);
impl_form_base!(TextControlInnerEditorElement, div, HTMLDivElement);
impl_form_base!(SpinButtonElement, div, HTMLDivElement);
impl_form_base!(SliderThumbElement, div, HTMLDivElement);

// The form classes embed Element at offset zero. Match the source's
// FormClassTraits tags before making a checked base-to-derived cast.
// cpp: layoutng/internal/form_node_metadata.h:165-200
macro_rules! impl_form_downcast {
    ($type:ty; $($tag:path)|+ $(|)?) => {
        impl foundation::DowncastFrom<Element> for $type {
            fn AllowFrom(element: &Element) -> bool {
                matches!(element.GetElementType(), $($tag)|+)
            }
        }
        impl foundation::DowncastFrom<Node> for $type {
            fn AllowFrom(node: &Node) -> bool {
                if !node.IsHTMLElement() {
                    return false;
                }
                let element = unsafe { &*(node as *const Node).cast::<Element>() };
                <Self as foundation::DowncastFrom<Element>>::AllowFrom(element)
            }
        }
    };
}

// cpp: layoutng/internal/form_node_metadata.h:165-166
impl foundation::DowncastFrom<Node> for HTMLElement {
    fn AllowFrom(node: &Node) -> bool {
        node.IsHTMLElement()
    }
}

impl foundation::DowncastFrom<Element> for HTMLElement {
    fn AllowFrom(element: &Element) -> bool {
        let node = &element.container.node;
        node.IsHTMLElement()
    }
}

// cpp: layoutng/internal/form_node_metadata.h:31-36,46-51,174-200
impl_form_downcast!(HTMLAreaElement; ElementType::kHTMLAreaElement);
impl_form_downcast!(HTMLImageElement; ElementType::kHTMLImageElement);
impl_form_downcast!(HTMLInputElement; ElementType::kHTMLInputElement);
impl_form_downcast!(HTMLSelectElement; ElementType::kHTMLSelectElement);
impl_form_downcast!(HTMLTextAreaElement; ElementType::kHTMLTextAreaElement);
impl_form_downcast!(HTMLButtonElement; ElementType::kHTMLButtonElement);
impl_form_downcast!(HTMLFieldSetElement; ElementType::kHTMLFieldSetElement);
impl_form_downcast!(HTMLOutputElement; ElementType::kHTMLOutputElement);
impl_form_downcast!(HTMLLegendElement; ElementType::kHTMLLegendElement);
impl_form_downcast!(HTMLMarqueeElement; ElementType::kHTMLMarqueeElement);
impl_form_downcast!(TextControlInnerEditorElement; ElementType::kTextControlInnerEditorElement);
impl_form_downcast!(SpinButtonElement; ElementType::kSpinButtonElement);
impl_form_downcast!(SliderThumbElement; ElementType::kSliderThumbElement);
impl_form_downcast!(HTMLDivElement; ElementType::kHTMLDivElement | ElementType::kSliderThumbElement | ElementType::kTextControlInnerEditorElement | ElementType::kSpinButtonElement);
impl_form_downcast!(HTMLFormControlElement; ElementType::kHTMLInputElement | ElementType::kHTMLSelectElement | ElementType::kHTMLTextAreaElement | ElementType::kHTMLButtonElement | ElementType::kHTMLFieldSetElement | ElementType::kHTMLOutputElement);
impl_form_downcast!(HTMLFormControlElementWithState; ElementType::kHTMLInputElement | ElementType::kHTMLSelectElement | ElementType::kHTMLTextAreaElement);
impl_form_downcast!(TextControlElement; ElementType::kHTMLInputElement | ElementType::kHTMLTextAreaElement);

// cpp: layoutng/internal/form_node_metadata.h:12-13
pub mod mojom {
    pub mod blink {
        pub type FormControlType = crate::internal::form_control_types::FormControlType;
    }
}
pub type WebAutofillState = AutofillState;

// Each first field embeds the C++ base at offset zero. Methods whose bodies
// live in //src/layoutng_forms remain external declarations below.
// cpp: layoutng/internal/form_node_metadata.h:15-22
#[repr(C)]
pub struct HTMLElement {
    pub element: Element,
}

impl HTMLElement {
    pub fn new(
        input: &NativeNodeConstructionData,
        style: *const ComputedStyle,
    ) -> Result<Self, ElementMetadataError> {
        let mut element = Element::new(input, style)?;
        element.set_runtime_identity(ElementType::kHTMLElement, false);
        Ok(Self { element })
    }

    pub fn IsHTMLElement(&self) -> bool {
        true
    }

    pub fn GetElementType(&self) -> ElementType {
        ElementType::kHTMLElement
    }

    // Public assembly seam for the constructors defined in layoutng_forms.
    // Their source validation stays in that package; this only records the
    // virtual identity in the shared metadata base.
    pub fn SetRuntimeIdentity(&mut self, kind: ElementType, form_control: bool) {
        self.element.set_runtime_identity(kind, form_control);
    }
}

// cpp: layoutng/internal/form_node_metadata.h:24-36
#[repr(C)]
pub struct HTMLAreaElement {
    pub html: HTMLElement,
}
impl HTMLAreaElement {
    pub fn new(
        input: &NativeNodeConstructionData,
        style: *const ComputedStyle,
    ) -> Result<Self, ElementMetadataError> {
        let mut html = HTMLElement::new(input, style)?;
        html.element
            .set_runtime_identity(ElementType::kHTMLAreaElement, false);
        Ok(Self { html })
    }
    pub fn GetElementType(&self) -> ElementType {
        ElementType::kHTMLAreaElement
    }
}

// cpp: layoutng/internal/form_node_metadata.h:38-51
#[repr(C)]
pub struct HTMLImageElement {
    pub html: HTMLElement,
}
impl HTMLImageElement {
    pub fn new(
        input: &NativeNodeConstructionData,
        style: *const ComputedStyle,
    ) -> Result<Self, ElementMetadataError> {
        let mut html = HTMLElement::new(input, style)?;
        html.element
            .set_runtime_identity(ElementType::kHTMLImageElement, false);
        Ok(Self { html })
    }
    pub fn GetElementType(&self) -> ElementType {
        ElementType::kHTMLImageElement
    }
}

// cpp: layoutng/internal/form_node_metadata.h:53-63
#[repr(C)]
pub struct HTMLFormControlElement {
    pub html: HTMLElement,
    pub(crate) type_: FormControlType,
    pub(crate) autofill_state_: AutofillState,
}
impl HTMLFormControlElement {
    pub fn from_resolved_parts(
        html: HTMLElement,
        control_type: FormControlType,
        autofill_state: AutofillState,
    ) -> Self {
        Self {
            html,
            type_: control_type,
            autofill_state_: autofill_state,
        }
    }

    pub fn IsFormControlElement(&self) -> bool {
        true
    }
    pub fn FormControlType(&self) -> FormControlType {
        self.type_
    }
    pub fn GetAutofillState(&self) -> AutofillState {
        self.autofill_state_
    }
}

// cpp: layoutng/internal/form_node_metadata.h:64-67
#[repr(C)]
pub struct HTMLFormControlElementWithState {
    pub control: HTMLFormControlElement,
}

// cpp: layoutng/internal/form_node_metadata.h:68-81
#[repr(C)]
pub struct TextControlElement {
    pub state: HTMLFormControlElementWithState,
    pub(crate) inner_editor_: Member<TextControlInnerEditorElement>,
}
impl TextControlElement {
    pub fn from_resolved_state(state: HTMLFormControlElementWithState) -> Self {
        Self {
            state,
            inner_editor_: Member::default(),
        }
    }

    pub fn InnerEditorElement(&self) -> *mut TextControlInnerEditorElement {
        self.inner_editor_.Get()
    }

    pub fn InnerEditorMember(&self) -> &Member<TextControlInnerEditorElement> {
        &self.inner_editor_
    }
}

// cpp: layoutng/internal/form_node_metadata.h:82-99
#[repr(C)]
pub struct HTMLInputElement {
    pub text_control: TextControlElement,
    pub(crate) upload_button_: Member<HTMLInputElement>,
    pub(crate) spin_button_: Member<SpinButtonElement>,
}
impl HTMLInputElement {
    pub fn from_resolved_text_control(text_control: TextControlElement) -> Self {
        Self {
            text_control,
            upload_button_: Member::default(),
            spin_button_: Member::default(),
        }
    }

    pub fn GetElementType(&self) -> ElementType {
        ElementType::kHTMLInputElement
    }
    // C++ inherits this accessor through TextControlElement and
    // HTMLFormControlElement; Rust keeps the embedded base chain explicit.
    // cpp: layoutng/internal/form_node_metadata.h:59-59,82-99
    pub fn FormControlType(&self) -> FormControlType {
        self.text_control.state.control.FormControlType()
    }
    pub fn SpinButtonForLayout(&self) -> *mut SpinButtonElement {
        self.spin_button_.Get()
    }
    pub fn UploadButton(&self) -> *mut HTMLInputElement {
        self.upload_button_.Get()
    }
    pub fn UploadButtonMember(&self) -> &Member<HTMLInputElement> {
        &self.upload_button_
    }
    pub fn SpinButtonMember(&self) -> &Member<SpinButtonElement> {
        &self.spin_button_
    }
}

// cpp: layoutng/internal/form_node_metadata.h:100-104
#[repr(C)]
pub struct HTMLSelectElement {
    pub state: HTMLFormControlElementWithState,
}
impl HTMLSelectElement {
    pub fn GetElementType(&self) -> ElementType {
        ElementType::kHTMLSelectElement
    }
}

// cpp: layoutng/internal/form_node_metadata.h:105-114
#[repr(C)]
pub struct HTMLTextAreaElement {
    pub text_control: TextControlElement,
    rows_: u32,
    cols_: u32,
}
impl HTMLTextAreaElement {
    pub fn from_resolved_text_control(
        text_control: TextControlElement,
        rows: u32,
        cols: u32,
    ) -> Self {
        Self {
            text_control,
            rows_: rows,
            cols_: cols,
        }
    }

    pub fn GetElementType(&self) -> ElementType {
        ElementType::kHTMLTextAreaElement
    }
    pub fn cols(&self) -> u32 {
        self.cols_
    }
    pub fn rows(&self) -> u32 {
        self.rows_
    }
}

// cpp: layoutng/internal/form_node_metadata.h:115-129
#[repr(C)]
pub struct HTMLButtonElement {
    pub control: HTMLFormControlElement,
}
impl HTMLButtonElement {
    pub fn GetElementType(&self) -> ElementType {
        ElementType::kHTMLButtonElement
    }
}
#[repr(C)]
pub struct HTMLFieldSetElement {
    pub control: HTMLFormControlElement,
}
impl HTMLFieldSetElement {
    pub fn GetElementType(&self) -> ElementType {
        ElementType::kHTMLFieldSetElement
    }
}
#[repr(C)]
pub struct HTMLOutputElement {
    pub control: HTMLFormControlElement,
}
impl HTMLOutputElement {
    pub fn GetElementType(&self) -> ElementType {
        ElementType::kHTMLOutputElement
    }
}

// cpp: layoutng/internal/form_node_metadata.h:130-146
#[repr(C)]
pub struct HTMLLegendElement {
    pub html: HTMLElement,
}
impl HTMLLegendElement {
    pub fn GetElementType(&self) -> ElementType {
        ElementType::kHTMLLegendElement
    }
}
#[repr(C)]
pub struct HTMLMarqueeElement {
    pub html: HTMLElement,
}
impl HTMLMarqueeElement {
    pub fn GetElementType(&self) -> ElementType {
        ElementType::kHTMLMarqueeElement
    }
}
#[repr(C)]
pub struct HTMLDivElement {
    pub html: HTMLElement,
}
impl HTMLDivElement {
    pub fn GetElementType(&self) -> ElementType {
        ElementType::kHTMLDivElement
    }
}

// cpp: layoutng/internal/form_node_metadata.h:147-163
#[repr(C)]
pub struct TextControlInnerEditorElement {
    pub div: HTMLDivElement,
}
impl TextControlInnerEditorElement {
    pub fn GetElementType(&self) -> ElementType {
        ElementType::kTextControlInnerEditorElement
    }
}
#[repr(C)]
pub struct SpinButtonElement {
    pub div: HTMLDivElement,
}
impl SpinButtonElement {
    pub fn GetElementType(&self) -> ElementType {
        ElementType::kSpinButtonElement
    }
}
#[repr(C)]
pub struct SliderThumbElement {
    pub div: HTMLDivElement,
}
impl SliderThumbElement {
    pub fn GetElementType(&self) -> ElementType {
        ElementType::kSliderThumbElement
    }
}

// cpp: layoutng/internal/form_node_metadata.h:168-173
pub struct FormClassTraits;
impl FormClassTraits {
    pub fn AllowFrom(node: &Node, types: &[ElementType]) -> bool {
        if !node.IsHTMLElement() {
            return false;
        }
        let element = unsafe { &*(node as *const Node as *const Element) };
        types.contains(&element.GetElementType())
    }
}

// cpp: layoutng/internal/form_node_metadata.h:31-51
// cpp: layoutng/internal/form_node_metadata.h:165-198
impl NodeDowncast for HTMLElement {
    fn AllowFrom(node: &Node) -> bool {
        node.IsHTMLElement()
    }
}

macro_rules! form_downcast {
    ($type:ty, $($kind:expr),+ $(,)?) => {
        impl NodeDowncast for $type {
            fn AllowFrom(node: &Node) -> bool {
                FormClassTraits::AllowFrom(node, &[$($kind),+])
            }
        }
    };
}

form_downcast!(HTMLAreaElement, ElementType::kHTMLAreaElement);
form_downcast!(HTMLImageElement, ElementType::kHTMLImageElement);
form_downcast!(
    HTMLFormControlElement,
    ElementType::kHTMLInputElement,
    ElementType::kHTMLSelectElement,
    ElementType::kHTMLTextAreaElement,
    ElementType::kHTMLButtonElement,
    ElementType::kHTMLFieldSetElement,
    ElementType::kHTMLOutputElement
);
form_downcast!(
    HTMLFormControlElementWithState,
    ElementType::kHTMLInputElement,
    ElementType::kHTMLSelectElement,
    ElementType::kHTMLTextAreaElement
);
form_downcast!(
    TextControlElement,
    ElementType::kHTMLInputElement,
    ElementType::kHTMLTextAreaElement
);
form_downcast!(HTMLInputElement, ElementType::kHTMLInputElement);
form_downcast!(HTMLSelectElement, ElementType::kHTMLSelectElement);
form_downcast!(HTMLTextAreaElement, ElementType::kHTMLTextAreaElement);
form_downcast!(HTMLButtonElement, ElementType::kHTMLButtonElement);
form_downcast!(HTMLFieldSetElement, ElementType::kHTMLFieldSetElement);
form_downcast!(HTMLOutputElement, ElementType::kHTMLOutputElement);
form_downcast!(HTMLLegendElement, ElementType::kHTMLLegendElement);
form_downcast!(HTMLMarqueeElement, ElementType::kHTMLMarqueeElement);
form_downcast!(
    HTMLDivElement,
    ElementType::kHTMLDivElement,
    ElementType::kSliderThumbElement,
    ElementType::kTextControlInnerEditorElement,
    ElementType::kSpinButtonElement
);
form_downcast!(
    TextControlInnerEditorElement,
    ElementType::kTextControlInnerEditorElement
);
form_downcast!(SpinButtonElement, ElementType::kSpinButtonElement);
form_downcast!(SliderThumbElement, ElementType::kSliderThumbElement);

// These methods and constructors are defined in //src/layoutng_forms, outside
// the selected package. A provider local to that future crate can implement
// this trait: Rust's orphan rule forbids implementing a trait and type both
// owned by this crate from layoutng_forms. The receiver is therefore explicit.
#[allow(non_snake_case)]
pub trait FormNodeMetadataExternal {
    // cpp: layoutng/internal/form_node_metadata.h:59-59
    // cpp: layoutng/internal/form_node_metadata.h:84-84
    // cpp: layoutng/internal/form_node_metadata.h:102-102
    // cpp: layoutng/internal/form_node_metadata.h:107-107
    // cpp: layoutng/internal/form_node_metadata.h:117-127
    // cpp: layoutng/internal/form_node_metadata.h:132-137
    // cpp: layoutng/internal/form_node_metadata.h:149-161
    // The provider's CreateFormElementMetadata dispatches these constructors.

    // cpp: layoutng/internal/form_node_metadata.h:75-75
    fn TraceTextControl(element: &TextControlElement, visitor: &mut Visitor);
    // cpp: layoutng/internal/form_node_metadata.h:88-94
    fn LayoutValueRatio(input: &HTMLInputElement) -> f64;
    fn GetSizeWithDecoration(input: &HTMLInputElement, preferred_size: &mut i32) -> bool;
    fn ShouldApplyMiddleEllipsis(input: &HTMLInputElement) -> bool;
    fn FileNoFileLabelForLayout(input: &HTMLInputElement) -> BlinkString;
    fn TraceInput(input: &HTMLInputElement, visitor: &mut Visitor);
    // cpp: layoutng/internal/form_node_metadata.h:139-139
    fn IsHorizontal(marquee: &HTMLMarqueeElement) -> bool;
    // cpp: layoutng/internal/form_node_metadata.h:200-201
    fn CreateFormElementMetadata(
        input: &NativeNodeConstructionData,
        style: *const ComputedStyle,
    ) -> *mut HTMLElement;
    fn IsSliderThumb(node: *const Node) -> bool;
}
