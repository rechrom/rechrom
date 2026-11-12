/*
 * Copyright (C) 2010 Google Inc. All rights reserved.
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
// Source: third_party/blink/renderer/core/html/forms/button_input_type.h:1-58
// Source: third_party/blink/renderer/core/html/forms/button_input_type.cc:1-53
use super::base_button_input_type::{
    BaseButtonInputType, ButtonInputElementRef, ButtonStyleAdjustment,
};
use super::input_type::Type;
use std::ops::Deref;
// Source: button_input_type.h:38-46,49-54.
pub struct ButtonInputType {
    pub base: BaseButtonInputType,
}
impl ButtonInputType {
    pub fn new(element: ButtonInputElementRef) -> Self {
        Self {
            base: BaseButtonInputType::new(Type::kButton, element),
        }
    }
    pub fn AllowFrom(input_type: Type) -> bool {
        input_type == Type::kButton
    }
    // Source: button_input_type.cc:39-45.
    pub fn CountUsage(&self) { /* TODO: cc:39-41 CountUsageIfVisible. */
    }
    pub fn SupportsValidation(&self) -> bool {
        false
    }
    // Source: button_input_type.cc:47-51.
    pub fn AdjustStyle(&self, style: &mut dyn ButtonStyleAdjustment) {
        style.SetShouldIgnoreOverflowPropertyForInlineBlockBaseline();
        style.SetInlineBlockBaselineEdgeContentBox();
        style.BaseInputAdjustStyle();
    }
}
impl Deref for ButtonInputType {
    type Target = BaseButtonInputType;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
