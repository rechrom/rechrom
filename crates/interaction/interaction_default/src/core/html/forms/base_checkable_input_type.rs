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
// Source: third_party/blink/renderer/core/html/forms/base_checkable_input_type.h:1-85
// Source: third_party/blink/renderer/core/html/forms/base_checkable_input_type.cc:1-139
// Rust modules replace guards/includes; shared Rc element handles and Drop
// replace the two GC bases without copying their retained element owner.

use super::input_type::{
    InputElementRef, InputType, TextControlSetValueSelection, TextFieldEventBehavior, Type,
    ValueMode,
};
use super::input_type_view::InputTypeView;
use interaction_dispatch::core::dom::events::event::EventRef;
use interaction_dispatch::core::events::keyboard_event::KeyboardEvent;
use std::cell::Cell;
use std::{ops::Deref, rc::Rc};

// Source: base_checkable_input_type.cc:82-96 and radio_input_type.cc:117-193.
// A short-lived scalar projection is captured before releasing the concrete
// KeyboardEvent borrow. The EventRef retains the original event for dispatch.
pub struct CheckableKeyboardEvent {
    pub key: String,
    pub char_code: i32,
    pub ctrl_key: bool,
    pub meta_key: bool,
    pub alt_key: bool,
}
impl CheckableKeyboardEvent {
    pub fn FromEvent(event: &KeyboardEvent) -> Self {
        Self {
            key: event.key().to_owned(),
            char_code: event.charCode(),
            ctrl_key: event.ctrlKey(),
            meta_key: event.metaKey(),
            alt_key: event.altKey(),
        }
    }
}

/// Source layout allocation branch, returned to the owning Layout adapter.
/// Interaction chooses no Layout lifecycle and retains no LayoutObject.
pub enum CheckableLayoutObject {
    InputTypeView,
    GenericObject,
}

/// The existing form-data owner consumes the source name/value append.
pub use super::input_type::InputTypeFormData as CheckableFormData;

// Source: base_checkable_input_type.h:40-57.
pub struct BaseCheckableInputType {
    pub input_type: InputType,
    pub view: InputTypeView,
    pub(crate) is_in_click_handler_: Cell<bool>,
}
impl BaseCheckableInputType {
    // Source: base_checkable_input_type.h:50-53.
    pub fn new(input_type: Type, element: InputElementRef) -> Self {
        Self {
            input_type: InputType::new(input_type, element.clone()),
            view: InputTypeView::new(element),
            is_in_click_handler_: Cell::new(false),
        }
    }
    // Source: base_checkable_input_type.cc:48-55. Rc/Drop covers both Trace
    // calls; CreateView returns this same object's UI base.
    pub fn CreateView(&self) -> &Self {
        self
    }
    // Source: base_checkable_input_type.cc:57-65. The embedding forwards this
    // allocation choice to Layout with its existing style owner.
    pub fn CreateLayoutObject(
        &self,
        inline_layout_enabled: bool,
        display_list_item: bool,
    ) -> CheckableLayoutObject {
        if !inline_layout_enabled || display_list_item {
            CheckableLayoutObject::InputTypeView
        } else {
            CheckableLayoutObject::GenericObject
        }
    }
    // Source: base_checkable_input_type.cc:67-70. Only the source's single
    // saved control-state string is returned; the form controller owns storage.
    pub fn SaveFormControlState(&self) -> String {
        Self::SaveFormControlStateWithChecked(self.GetElement().Checked()).to_owned()
    }
    pub fn SaveFormControlStateWithChecked(checked: bool) -> &'static str {
        if checked {
            "on"
        } else {
            "off"
        }
    }
    // Source: base_checkable_input_type.cc:72-75.
    pub fn RestoreFormControlState(&self, state: &[Rc<[u16]>]) {
        self.GetElement().SetChecked(
            state[0].as_ref() == [111, 110],
            TextFieldEventBehavior::kDispatchNoEvent,
        );
    }
    // Source: base_checkable_input_type.cc:77-80.
    pub fn AppendToFormData(&self, data: &mut dyn CheckableFormData) {
        Self::AppendWithOperations(
            self.GetElement().Checked(),
            self.GetElement().NameForFormData(),
            self.GetElement().ValueForFormData(),
            data,
        );
    }
    // Source base_checkable_input_type.cc:77-80.
    pub fn AppendWithOperations(
        checked: bool,
        name: std::rc::Rc<[u16]>,
        value: std::rc::Rc<[u16]>,
        data: &mut dyn CheckableFormData,
    ) {
        if checked {
            data.AppendFromElement(name, value);
        }
    }
    // Source: base_checkable_input_type.cc:82-89. Space intentionally does
    // not mark keydown handled: the source still dispatches keypress afterward.
    pub fn HandleKeydownEvent(&self, keyboard: &CheckableKeyboardEvent) {
        if keyboard.key == " " {
            self.GetElement().SetActive(true);
        }
    }
    // Source: base_checkable_input_type.cc:91-96.
    pub fn HandleKeypressEvent(&self, event: &EventRef, keyboard: &CheckableKeyboardEvent) {
        if keyboard.char_code == ' ' as i32 {
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
        }
    }
    // Source: base_checkable_input_type.cc:98-108.
    pub fn CanSetStringValue(&self) -> bool {
        false
    }
    pub fn MatchesDefaultPseudoClass(&self) -> bool {
        self.GetElement().HasCheckedAttribute()
    }
    pub fn GetValueMode(&self) -> ValueMode {
        ValueMode::kDefaultOn
    }
    // Source: base_checkable_input_type.cc:110-116.
    pub fn SetValue(
        &self,
        value: &str,
        _changed: bool,
        _behavior: TextFieldEventBehavior,
        _selection: TextControlSetValueSelection,
    ) {
        self.GetElement().SetValueAttribute(value);
    }
    // Source: base_checkable_input_type.cc:118-123.
    pub fn ReadingChecked(&self) {
        if self.is_in_click_handler_.get() {
            // TODO: base_checkable_input_type.cc:120-121 UseCounter.
        }
    }
    // Source: base_checkable_input_type.cc:125-127.
    pub fn IsCheckable(&self) -> bool {
        true
    }
    // Source: base_checkable_input_type.cc:129-137.
    pub fn HandleBlurEvent(&self) {
        if !self.GetElement().HasActiveLabel() {
            self.GetElement().SetActive(false);
        }
    }
    // Source: base_checkable_input_type.h:55.
    pub fn NeedsShadowSubtree(&self) -> bool {
        false
    }
}
impl Deref for BaseCheckableInputType {
    type Target = InputType;
    fn deref(&self) -> &InputType {
        &self.input_type
    }
}
