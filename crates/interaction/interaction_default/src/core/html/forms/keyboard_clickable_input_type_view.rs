/*
 * Copyright (C) 2010, 2012 Google Inc. All rights reserved.
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
// Source: third_party/blink/renderer/core/html/forms/keyboard_clickable_input_type_view.h:1-54
// Source: third_party/blink/renderer/core/html/forms/keyboard_clickable_input_type_view.cc:1-70

use super::input_type::InputElementRef;
use super::input_type_view::InputTypeView;
use interaction_dispatch::core::dom::events::event::EventRef;
use std::ops::Deref;

pub trait KeyboardClickableOperations {
    fn SetActive(&self, active: bool);
    fn IsActive(&self) -> bool;
    fn DispatchSimulatedClick(&self);
}

// Source: keyboard_clickable_input_type_view.h:40-49.
pub struct KeyboardClickableInputTypeView {
    pub view: InputTypeView,
}
impl KeyboardClickableInputTypeView {
    // Source: keyboard_clickable_input_type_view.h:43-44.
    pub fn new(element: InputElementRef) -> Self {
        Self {
            view: InputTypeView::new(element),
        }
    }
    // Source: keyboard_clickable_input_type_view.cc:41-48.
    pub fn HandleKeydownEvent(&self, key: &str) {
        Self::HandleKeydownWithOperations(
            key,
            &SourceKeyboardClickable {
                view: self,
                event: None,
            },
        );
    }
    pub fn HandleKeydownWithOperations(key: &str, operations: &dyn KeyboardClickableOperations) {
        if key == " " {
            operations.SetActive(true);
        }
    }
    // Source: keyboard_clickable_input_type_view.cc:50-61.
    pub fn HandleKeypressEvent(&self, event: &EventRef, key: &str) {
        if Self::HandleKeypressWithOperations(
            key,
            &SourceKeyboardClickable {
                view: self,
                event: Some(event),
            },
        ) {
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
        }
    }
    pub fn HandleKeypressWithOperations(
        key: &str,
        operations: &dyn KeyboardClickableOperations,
    ) -> bool {
        if key == "Enter" {
            operations.DispatchSimulatedClick();
            return true;
        }
        key == " "
    }
    // Source: keyboard_clickable_input_type_view.cc:63-68;
    // input_type_view.cc:73-77 supplies the inherited active/default-handled body.
    pub fn HandleKeyupEvent(&self, event: &EventRef, key: &str) {
        if Self::HandleKeyupWithOperations(
            key,
            &SourceKeyboardClickable {
                view: self,
                event: Some(event),
            },
        ) {
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
        }
    }
    pub fn HandleKeyupWithOperations(
        key: &str,
        operations: &dyn KeyboardClickableOperations,
    ) -> bool {
        if key != " " {
            return false;
        }
        if operations.IsActive() {
            operations.DispatchSimulatedClick();
        }
        true
    }
}
impl Deref for KeyboardClickableInputTypeView {
    type Target = InputTypeView;
    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

struct SourceKeyboardClickable<'a> {
    view: &'a KeyboardClickableInputTypeView,
    event: Option<&'a EventRef>,
}
impl KeyboardClickableOperations for SourceKeyboardClickable<'_> {
    fn SetActive(&self, active: bool) {
        self.view.GetElement().SetActive(active);
    }
    fn IsActive(&self) -> bool {
        self.view.GetElement().IsActive()
    }
    fn DispatchSimulatedClick(&self) {
        self.view
            .GetElement()
            .DispatchSimulatedClick(self.event.expect("keypress/keyup retains its source event"));
    }
}
