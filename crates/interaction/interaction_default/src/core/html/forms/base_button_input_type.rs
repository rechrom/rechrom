/*
 * Copyright (C) 2010 Google Inc. All rights reserved.
 * Copyright (C) 2011 Apple Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 *     * Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above
 * copyright notice, this list of conditions and the following disclaimer
 * in the documentation and/or other materials provided with the
 * distribution.
 *     * Neither the name of Google Inc. nor the names of its
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/html/forms/base_button_input_type.h:1-72
// Source: third_party/blink/renderer/core/html/forms/base_button_input_type.cc:1-116

use super::html_form_element::{FormControlRef, HTMLFormElement};
pub use super::input_type::InputTypeFormData as ButtonFormData;
use super::input_type::{
    BaseAppearanceValue, InputElement, InputElementRef, InputType, TextControlSetValueSelection,
    TextFieldEventBehavior, Type, ValueMode,
};
use super::keyboard_clickable_input_type_view::KeyboardClickableInputTypeView;
use std::{ops::Deref, rc::Rc};

pub type ButtonInputElementRef = Rc<dyn ButtonInputElement>;
// Source: base_button_input_type.cc:56-70,95-107; reset_input_type.cc:53-61;
// submit_input_type.cc:53-84. Each projection retains the existing control.
pub trait ButtonInputElement: InputElement {
    fn IsDisabledFormControl(&self) -> bool;
    fn Form(&self) -> Option<Rc<HTMLFormElement>>;
    fn AsFormControl(&self) -> FormControlRef;
    fn ValueOrDefaultLabel(&self) -> String;
    fn ValueOrDefaultLabelForFormData(&self) -> Rc<[u16]>;
    fn AppendShadowText(&self, text: &str);
    fn SetShadowText(&self, text: &str);
    fn IsActivatedSubmit(&self) -> bool;
    fn ResetLabel(&self) -> String;
    fn SubmitLabel(&self) -> String;
}

/// Source form-activation virtuals borrowed over an existing element identity.
pub trait ButtonFormActivationOperations<F> {
    fn Form(&self) -> Option<F>;
    fn IsDisabledFormControl(&self) -> bool;
    fn PrepareForSubmission(&self, form: &F);
    fn ResetForm(&self, form: &F);
}
pub(super) struct SourceButtonFormActivation<'a> {
    pub element: &'a ButtonInputElementRef,
    pub event: &'a interaction_dispatch::core::dom::events::event::EventRef,
}
impl ButtonFormActivationOperations<Rc<HTMLFormElement>> for SourceButtonFormActivation<'_> {
    fn Form(&self) -> Option<Rc<HTMLFormElement>> {
        self.element.Form()
    }
    fn IsDisabledFormControl(&self) -> bool {
        self.element.IsDisabledFormControl()
    }
    fn PrepareForSubmission(&self, form: &Rc<HTMLFormElement>) {
        form.PrepareForSubmission(Some(self.event), Some(self.element.AsFormControl()));
    }
    fn ResetForm(&self, form: &Rc<HTMLFormElement>) {
        form.reset();
    }
}

/// Typed allocation/style values consumed by the owning Layout/Style adapter.
pub enum ButtonLayoutObject {
    LayoutBlockFlow,
}
pub enum ButtonAppearance {
    PushButton,
}
pub trait ButtonStyleAdjustment {
    fn SetShouldIgnoreOverflowPropertyForInlineBlockBaseline(&mut self);
    fn SetInlineBlockBaselineEdgeContentBox(&mut self);
    fn BaseInputAdjustStyle(&mut self);
}

// Source: base_button_input_type.h:39-67. Rc/Drop covers both source Trace bases.
pub struct BaseButtonInputType {
    pub input_type: InputType,
    pub view: KeyboardClickableInputTypeView,
    pub element: ButtonInputElementRef,
}
impl BaseButtonInputType {
    // Source: base_button_input_type.cc:44-50.
    pub fn new(input_type: Type, element: ButtonInputElementRef) -> Self {
        let input: InputElementRef = element.clone();
        Self {
            input_type: InputType::new(input_type, input.clone()),
            view: KeyboardClickableInputTypeView::new(input),
            element,
        }
    }
    // Source: base_button_input_type.cc:52-60.
    pub fn CreateView(&self) -> &KeyboardClickableInputTypeView {
        &self.view
    }
    pub fn CreateShadowSubtree(&self) {
        self.element
            .AppendShadowText(&self.element.ValueOrDefaultLabel());
    }
    // Source: base_button_input_type.cc:62-65.
    pub fn SupportsPopoverTriggering(&self) -> bool {
        true
    }
    // Source: base_button_input_type.cc:67-70.
    pub fn ValueAttributeChanged(&self) {
        self.element
            .SetShadowText(&self.element.ValueOrDefaultLabel());
    }
    // Source: base_button_input_type.cc:72-89.
    pub fn ShouldSaveAndRestoreFormControlState(&self) -> bool {
        false
    }
    pub fn IsAutoDirectionalityFormAssociated(&self) -> bool {
        true
    }
    pub fn AppendToFormData(&self, _data: &mut dyn ButtonFormData) {}
    pub fn AutoAppearance(&self) -> ButtonAppearance {
        ButtonAppearance::PushButton
    }
    pub fn CreateLayoutObject(&self) -> ButtonLayoutObject {
        ButtonLayoutObject::LayoutBlockFlow
    }
    // Source: base_button_input_type.cc:91-101.
    pub fn GetValueMode(&self) -> ValueMode {
        ValueMode::kDefault
    }
    pub fn SetValue(
        &self,
        value: &str,
        _changed: bool,
        _behavior: TextFieldEventBehavior,
        _selection: TextControlSetValueSelection,
    ) {
        self.element.SetValueAttribute(value);
    }
    // Source: base_button_input_type.cc:103-108. The virtual submit-button test
    // is supplied by the concrete source InputType before traversing the form.
    pub fn MatchesDefaultPseudoClass(&self, successful_submit_button: bool) -> bool {
        successful_submit_button
            && self.element.Form().is_some_and(|form| {
                form.FindDefaultButton()
                    .is_some_and(|default| Rc::ptr_eq(&default, &self.element.AsFormControl()))
            })
    }
    // Source: base_button_input_type.cc:110-114.
    pub fn SupportsBaseAppearance(&self, value: BaseAppearanceValue, enabled: bool) -> bool {
        enabled && value == BaseAppearanceValue::kBase
    }
}
impl Deref for BaseButtonInputType {
    type Target = InputType;
    fn deref(&self) -> &Self::Target {
        &self.input_type
    }
}
