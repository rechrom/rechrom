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
// Source: third_party/blink/renderer/core/html/forms/checkbox_input_type.h:1-67
// Source: third_party/blink/renderer/core/html/forms/checkbox_input_type.cc:1-139

use super::base_checkable_input_type::{BaseCheckableInputType, CheckableKeyboardEvent};
use super::input_type::{
    BaseAppearanceValue, CheckableLocale, InputElementRef, TextFieldEventBehavior, Type,
};
use super::input_type_view::ClickHandlingState;
use interaction_dispatch::core::dom::events::event::EventRef;
use std::ops::Deref;

/// The exact operations used by the source activation transaction. Borrowing
/// these operations permits an opaque DOM identity without a second control
/// model or a retained embedding callback.
pub trait CheckboxActivationOperations {
    fn Checked(&self) -> bool;
    fn Indeterminate(&self) -> bool;
    fn SetChecked(&self, checked: bool, behavior: TextFieldEventBehavior);
    fn SetIndeterminate(&self, indeterminate: bool);
    fn SetInClickHandler(&self, in_handler: bool);
    fn SetUserHasEditedTheField(&self);
    fn DispatchInputAndChangeEventIfNeeded(&self);
}

// Source: checkbox_input_type.h:38-56.
pub struct CheckboxInputType {
    pub base: BaseCheckableInputType,
}
impl CheckboxInputType {
    // Source: checkbox_input_type.h:40-41.
    pub fn new(element: InputElementRef) -> Self {
        Self {
            base: BaseCheckableInputType::new(Type::kCheckbox, element),
        }
    }
    // Source: checkbox_input_type.cc:47-49.
    pub fn CountUsage(&self) { /* TODO: checkbox_input_type.cc:48 UseCounter. */
    }
    // Source: checkbox_input_type.cc:51-53.
    pub fn AutoAppearance(&self) -> CheckableAppearance {
        CheckableAppearance::kCheckbox
    }
    // Source: checkbox_input_type.cc:55-57.
    pub fn ValueMissing(&self, _value: &str) -> bool {
        Self::ValueMissingWithState(self.GetElement().IsRequired(), self.GetElement().Checked())
    }
    pub fn ValueMissingWithState(required: bool, checked: bool) -> bool {
        required && !checked
    }
    // Source: checkbox_input_type.cc:59-61; input_type.cc:745-747.
    pub fn ValueMissingText(&self, locale: &dyn CheckableLocale) -> String {
        locale.ValueMissingCheckbox()
    }
    // Source: checkbox_input_type.cc:63-71.
    pub fn HandleKeyupEvent(
        &self,
        event: &EventRef,
        keyboard: &CheckableKeyboardEvent,
        spatial_navigation: bool,
    ) {
        if keyboard.key == " " || (spatial_navigation && keyboard.key == "Enter") {
            self.base.view.DispatchSimulatedClickIfActive(event);
        }
    }
    // Source: checkbox_input_type.cc:73-97.
    pub fn LegacyPreActivationBehavior(&self) -> ClickHandlingState {
        Self::LegacyPreActivationWithOperations(self)
    }
    // Source: checkbox_input_type.cc:73-97. Same source body for retained
    // element owners and per-call borrowed DOM operations.
    pub fn LegacyPreActivationWithOperations<E>(
        element: &dyn CheckboxActivationOperations,
    ) -> ClickHandlingState<E> {
        let state = ClickHandlingState {
            checked: element.Checked(),
            indeterminate: element.Indeterminate(),
            checked_radio_button: None,
        };
        element.SetChecked(!state.checked, TextFieldEventBehavior::kDispatchChangeEvent);
        element.SetIndeterminate(false);
        element.SetInClickHandler(true);
        state
    }
    // Source: checkbox_input_type.cc:99-125. No Event borrow crosses effects.
    pub fn RunInputActivationBehavior(&self, event: &EventRef, state: &ClickHandlingState) {
        let (canceled, trusted) = {
            let event = event.borrow();
            (
                event.BaseEvent().defaultPrevented() || event.BaseEvent().DefaultHandled(),
                event.BaseEvent().isTrusted(),
            )
        };
        Self::RunActivationWithOperations(self, canceled, trusted, state);
        event.borrow_mut().BaseEventMut().SetDefaultHandled();
    }
    pub fn RunActivationWithOperations<E>(
        element: &dyn CheckboxActivationOperations,
        canceled: bool,
        trusted: bool,
        state: &ClickHandlingState<E>,
    ) {
        if canceled {
            element.SetIndeterminate(state.indeterminate);
            element.SetChecked(state.checked, TextFieldEventBehavior::kDispatchNoEvent);
        } else {
            if trusted {
                element.SetUserHasEditedTheField();
            }
            element.DispatchInputAndChangeEventIfNeeded();
        }
        element.SetInClickHandler(false);
    }
    // Source: checkbox_input_type.cc:127-129.
    pub fn ShouldAppearIndeterminate(&self) -> bool {
        self.GetElement().indeterminate()
    }
    // Source: checkbox_input_type.cc:131-137.
    pub fn SupportsBaseAppearance(
        &self,
        value: BaseAppearanceValue,
        appearance_base_enabled: bool,
    ) -> bool {
        if !appearance_base_enabled {
            return false;
        }
        value == BaseAppearanceValue::kBase
    }
}
// Source: checkbox_input_type.cc:51-53 and radio_input_type.cc:61-63. This
// narrow output carries only the source appearance choice to Style's owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckableAppearance {
    kCheckbox,
    kRadio,
}
impl Deref for CheckboxInputType {
    type Target = BaseCheckableInputType;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl CheckboxActivationOperations for CheckboxInputType {
    fn Checked(&self) -> bool {
        self.GetElement().Checked()
    }
    fn Indeterminate(&self) -> bool {
        self.GetElement().indeterminate()
    }
    fn SetChecked(&self, checked: bool, behavior: TextFieldEventBehavior) {
        self.GetElement().SetChecked(checked, behavior);
    }
    fn SetIndeterminate(&self, value: bool) {
        self.GetElement().setIndeterminate(value);
    }
    fn SetInClickHandler(&self, value: bool) {
        self.base.is_in_click_handler_.set(value);
    }
    fn SetUserHasEditedTheField(&self) {
        self.GetElement().SetUserHasEditedTheField();
    }
    fn DispatchInputAndChangeEventIfNeeded(&self) {
        self.GetElement().DispatchInputAndChangeEventIfNeeded();
    }
}
