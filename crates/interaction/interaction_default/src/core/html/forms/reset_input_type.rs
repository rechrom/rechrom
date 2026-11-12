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
// Source: third_party/blink/renderer/core/html/forms/reset_input_type.h:1-60
// Source: third_party/blink/renderer/core/html/forms/reset_input_type.cc:1-70
use super::base_button_input_type::{
    BaseButtonInputType, ButtonInputElementRef, ButtonStyleAdjustment,
};
use super::input_type::Type;
use interaction_dispatch::core::dom::events::event::EventRef;
use std::ops::Deref;
// Source: reset_input_type.h:38-55.
pub struct ResetInputType {
    pub base: BaseButtonInputType,
}
impl ResetInputType {
    pub fn new(element: ButtonInputElementRef) -> Self {
        Self {
            base: BaseButtonInputType::new(Type::kReset, element),
        }
    }
    pub fn AllowFrom(input_type: Type) -> bool {
        input_type == Type::kReset
    }
    // Source: reset_input_type.cc:45-51.
    pub fn CountUsage(&self) { /* TODO: cc:45-47 CountUsageIfVisible. */
    }
    pub fn SupportsValidation(&self) -> bool {
        false
    }
    // Source: reset_input_type.cc:53-58.
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
        operations.ResetForm(&form);
        true
    }
    // Source: reset_input_type.cc:60-68.
    pub fn DefaultLabel(&self) -> String {
        self.element.ResetLabel()
    }
    pub fn AdjustStyle(&self, style: &mut dyn ButtonStyleAdjustment) {
        style.SetShouldIgnoreOverflowPropertyForInlineBlockBaseline();
        style.SetInlineBlockBaselineEdgeContentBox();
        style.BaseInputAdjustStyle();
    }
}
impl Deref for ResetInputType {
    type Target = BaseButtonInputType;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
