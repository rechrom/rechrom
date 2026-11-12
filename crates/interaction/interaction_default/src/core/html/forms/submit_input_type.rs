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
// Source: third_party/blink/renderer/core/html/forms/submit_input_type.h:1-61
// Source: third_party/blink/renderer/core/html/forms/submit_input_type.cc:1-93
use super::base_button_input_type::{
    BaseButtonInputType, ButtonFormData, ButtonInputElementRef, ButtonStyleAdjustment,
};
use super::input_type::Type;
use interaction_dispatch::core::dom::events::event::EventRef;
use std::ops::Deref;
use std::rc::Rc;
// Source: submit_input_type.h:38-56.
pub struct SubmitInputType {
    pub base: BaseButtonInputType,
}
impl SubmitInputType {
    // Source: submit_input_type.cc:48-51.
    pub fn new(element: ButtonInputElementRef) -> Self {
        // TODO: cc:50 UseCounter.
        Self {
            base: BaseButtonInputType::new(Type::kSubmit, element),
        }
    }
    pub fn AllowFrom(input_type: Type) -> bool {
        input_type == Type::kSubmit
    }
    // Source: submit_input_type.cc:53-59.
    pub fn AppendToFormData(&self, data: &mut dyn ButtonFormData) {
        self.input_type
            .AppendToFormData(data, self.element.DirnameAndDirectionForFormData());
        Self::AppendActivatedValue(
            self.element.IsActivatedSubmit(),
            self.element.NameForFormData(),
            self.element.ValueOrDefaultLabelForFormData(),
            data,
        );
    }
    // Source submit_input_type.cc:55-58. Base InputType append precedes this.
    pub fn AppendActivatedValue(
        activated: bool,
        name: Rc<[u16]>,
        value: Rc<[u16]>,
        data: &mut dyn ButtonFormData,
    ) {
        if activated {
            data.AppendFromElement(name, value);
        }
    }
    // Source: submit_input_type.cc:61-79.
    pub fn SupportsRequired(&self) -> bool {
        false
    }
    pub fn HandleDOMActivateEvent(&self, event: &EventRef) {
        if Self::HandleDOMActivateWithOperations(
            &super::base_button_input_type::SourceButtonFormActivation {
                element: &self.element,
                event,
            },
        ) {
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
        }
    }
    pub fn HandleDOMActivateWithOperations<F>(
        operations: &dyn super::base_button_input_type::ButtonFormActivationOperations<F>,
    ) -> bool {
        if operations.IsDisabledFormControl() {
            return false;
        }
        let Some(form) = operations.Form() else {
            return false;
        };
        operations.PrepareForSubmission(&form);
        true
    }
    pub fn CanBeSuccessfulSubmitButton(&self) -> bool {
        true
    }
    pub fn DefaultLabel(&self) -> String {
        self.element.SubmitLabel()
    }
    // Source: submit_input_type.cc:81-91.
    pub fn ValueAttributeChanged(&self) {
        // TODO: cc:82-83 UseCounter.
        self.base.ValueAttributeChanged();
    }
    pub fn AdjustStyle(&self, style: &mut dyn ButtonStyleAdjustment) {
        style.SetShouldIgnoreOverflowPropertyForInlineBlockBaseline();
        style.SetInlineBlockBaselineEdgeContentBox();
        style.BaseInputAdjustStyle();
    }
}
impl Deref for SubmitInputType {
    type Target = BaseButtonInputType;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
