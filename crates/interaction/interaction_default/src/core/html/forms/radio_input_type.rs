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
/*
 * Copyright (C) 2005, 2011 Apple Inc. All rights reserved.
 * Copyright (C) 2010 Google Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 *
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/html/forms/radio_input_type.h:1-84
// Source: third_party/blink/renderer/core/html/forms/radio_input_type.cc:1-387

use super::base_checkable_input_type::{BaseCheckableInputType, CheckableKeyboardEvent};
use super::checkbox_input_type::CheckableAppearance;
use super::input_type::{
    BaseAppearanceValue, CheckableLocale, InputElementRef, OwnerIdentity, TextFieldEventBehavior,
    Type, UpdateBehavior,
};
use super::input_type_view::ClickHandlingState;
use interaction_dispatch::core::dom::events::event::EventRef;
use std::ops::Deref;
use std::rc::Rc;

/// Source RadioButtonGroupScope queries, retaining the same element owners.
pub trait RadioButtonGroupScope {
    fn IsInRequiredGroup(&self, input: &InputElementRef) -> bool;
    fn CheckedButtonForGroup(&self, name: &str) -> Option<InputElementRef>;
    fn LastFocusedButtonForGroup(&self, name: &str) -> Option<InputElementRef>;
}
/// Borrowed DOM traversal operations and opaque identities. Traversal remains
/// with the existing DOM tree; selection/group decisions remain in this file.
pub trait RadioInputEnvironment {
    fn Form(&self, input: &InputElementRef) -> Option<OwnerIdentity>;
    fn HasFormOwner(&self, input: &InputElementRef) -> bool;
    fn TreeScope(&self, input: &InputElementRef) -> OwnerIdentity;
    fn TreeRoot(&self, input: &InputElementRef) -> OwnerIdentity;
    fn IsConnected(&self, input: &InputElementRef) -> bool;
    fn IsRadio(&self, input: &InputElementRef) -> bool;
    fn GroupScope(&self, input: &InputElementRef) -> Option<Rc<dyn RadioButtonGroupScope>>;
    // Source: radio_input_type.cc:47-53. `form` carries its listed-elements
    // scope; root is used for the disconnected inclusive-first traversal.
    fn NextInputElement(
        &self,
        input: &InputElementRef,
        form: Option<&OwnerIdentity>,
        root: Option<&OwnerIdentity>,
        forward: bool,
    ) -> Option<InputElementRef>;
    fn InclusiveFirstInputWithin(&self, root: &OwnerIdentity) -> Option<InputElementRef>;
    fn SpatialNavigationEnabled(&self, input: &InputElementRef) -> bool;
    // Source: radio_input_type.cc:166-168. Restore selection, focus type None,
    // and null source capabilities are fixed by this narrow operation.
    fn SetFocusedElementRestoringSelection(&self, input: &InputElementRef);
    fn CSSUserValidAndUserInvalidForRadioEnabled(&self) -> bool;
    fn AppearanceBaseEnabled(&self) -> bool;
    // Calls the existing input type's virtual focusability decision; it does
    // not read copied style or choose another radio in the adapter.
    fn IsKeyboardFocusableSlow(&self, input: &InputElementRef, update: UpdateBehavior) -> bool;
}

fn SameOwner(a: Option<OwnerIdentity>, b: Option<OwnerIdentity>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(&a, &b),
        (None, None) => true,
        _ => false,
    }
}

/// Borrowed source activation primitives; E is the same DOM radio identity
/// before and after listener reentry. The source, rather than the adapter,
/// decides cancellation restoration and event dispatch.
pub trait RadioActivationOperations<E> {
    fn Checked(&self) -> bool;
    fn CheckedRadioButtonForGroup(&self) -> Option<E>;
    fn SetChecked(&self, checked: bool, behavior: TextFieldEventBehavior);
    fn IsSameRadioGroup(&self, checked: &E) -> bool;
    fn SetRadioChecked(&self, checked: &E, behavior: TextFieldEventBehavior);
    fn SetInClickHandler(&self, value: bool);
    fn CSSUserValidAndUserInvalidForRadioEnabled(&self) -> bool;
    fn SetUserHasEditedTheField(&self);
    fn DispatchInputAndChangeEventIfNeeded(&self);
}

/// Native keyboard primitives for the original radio control. Group identity
/// and document traversal remain with their existing DOM owner.
pub trait RadioKeyboardOperations<N> {
    fn Current(&self) -> N;
    fn HasLayoutObject(&self) -> bool;
    fn SetActive(&self, active: bool);
    fn IsRightToLeft(&self) -> bool;
    fn SpatialNavigationEnabled(&self) -> bool;
    fn NextRadioButtonInGroup(&self, current: &N, forward: bool) -> Option<N>;
    fn IsFocusable(&self, input: &N) -> bool;
    fn FocusRestoringSelection(&self, input: &N);
    fn DispatchSimulatedClick(&self, input: &N);
}

// Source: radio_input_type.h:39-73.
pub struct RadioInputType {
    pub base: BaseCheckableInputType,
    environment: Rc<dyn RadioInputEnvironment>,
}
impl RadioInputType {
    // Source: radio_input_type.h:46-47.
    pub fn new(element: InputElementRef, environment: Rc<dyn RadioInputEnvironment>) -> Self {
        Self {
            base: BaseCheckableInputType::new(Type::kRadio, element),
            environment,
        }
    }
    // Source: radio_input_type.cc:57-59.
    pub fn CountUsage(&self) { /* TODO: radio_input_type.cc:58 UseCounter. */
    }
    // Source: radio_input_type.cc:61-63.
    pub fn AutoAppearance(&self) -> CheckableAppearance {
        CheckableAppearance::kRadio
    }
    // Source: radio_input_type.cc:65-94.
    pub fn ValueMissing(&self, _value: &str) -> bool {
        let input = self.GetElement();
        if let Some(scope) = self.environment.GroupScope(input) {
            return Self::ValueMissingWithGroup(
                scope.IsInRequiredGroup(input),
                self.CheckedRadioButtonForGroup().is_some(),
            );
        }
        debug_assert!(!self.environment.IsConnected(input));
        debug_assert!(!self.environment.HasFormOwner(input));
        let name = input.GetName();
        if name.is_empty() {
            return false;
        }
        let mut is_required = false;
        let mut is_checked = false;
        let root = self.environment.TreeRoot(input);
        let mut another = self.environment.InclusiveFirstInputWithin(&root);
        while let Some(current) = another {
            if self.environment.IsRadio(&current)
                && current.GetName() == name
                && !self.environment.HasFormOwner(&current)
            {
                if current.Checked() {
                    is_checked = true;
                }
                if current.HasRequiredAttribute() {
                    is_required = true;
                }
                if is_checked && is_required {
                    return false;
                }
            }
            another = self
                .environment
                .NextInputElement(&current, None, Some(&root), true);
        }
        Self::ValueMissingWithGroup(is_required, is_checked)
    }
    pub fn ValueMissingWithGroup(required: bool, checked: bool) -> bool {
        required && !checked
    }
    // Source: radio_input_type.cc:96-98; input_type.cc:745-747.
    pub fn ValueMissingText(&self, locale: &dyn CheckableLocale) -> String {
        locale.ValueMissingRadio()
    }
    // Source: radio_input_type.cc:100-102.
    pub fn HandleClickEvent(&self, event: &EventRef) {
        event.borrow_mut().BaseEventMut().SetDefaultHandled();
    }
    // Source: radio_input_type.cc:104-115.
    pub fn FindNextFocusableRadioButtonInGroup(
        &self,
        current: &InputElementRef,
        forward: bool,
    ) -> Option<InputElementRef> {
        Self::FindNextFocusableWithOperations(
            &SourceRadioKeyboard {
                radio: self,
                event: None,
            },
            current,
            forward,
        )
    }
    pub fn FindNextFocusableWithOperations<N>(
        operations: &dyn RadioKeyboardOperations<N>,
        current: &N,
        forward: bool,
    ) -> Option<N> {
        let mut next = operations.NextRadioButtonInGroup(current, forward);
        while let Some(input) = next {
            if operations.IsFocusable(&input) {
                return Some(input);
            }
            next = operations.NextRadioButtonInGroup(&input, forward);
        }
        None
    }
    // Source: radio_input_type.cc:117-173.
    pub fn HandleKeydownEvent(&self, event: &EventRef, keyboard: &CheckableKeyboardEvent) {
        let default_handled = event.borrow().BaseEvent().DefaultHandled();
        if Self::HandleKeydownWithOperations(
            &SourceRadioKeyboard {
                radio: self,
                event: Some(event),
            },
            keyboard,
            default_handled,
        ) {
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
        }
    }
    pub fn HandleKeydownWithOperations<N>(
        operations: &dyn RadioKeyboardOperations<N>,
        keyboard: &CheckableKeyboardEvent,
        default_handled: bool,
    ) -> bool {
        if !operations.HasLayoutObject() {
            return false;
        }
        if keyboard.key == " " {
            operations.SetActive(true);
        }
        if default_handled {
            return false;
        }
        let key = keyboard.key.as_str();
        if !matches!(key, "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight") {
            return false;
        }
        if keyboard.ctrl_key
            || keyboard.meta_key
            || keyboard.alt_key
            || operations.SpatialNavigationEnabled()
        {
            return false;
        }
        let current = operations.Current();
        let mut forward = if operations.IsRightToLeft() {
            key == "ArrowDown" || key == "ArrowLeft"
        } else {
            key == "ArrowDown" || key == "ArrowRight"
        };
        let mut input = Self::FindNextFocusableWithOperations(operations, &current, forward);
        if input.is_none() {
            forward = !forward;
            let mut next = Self::FindNextFocusableWithOperations(operations, &current, forward);
            while let Some(current) = next {
                next = Self::FindNextFocusableWithOperations(operations, &current, forward);
                input = Some(current);
            }
        }
        if let Some(input) = input {
            operations.FocusRestoringSelection(&input);
            operations.DispatchSimulatedClick(&input);
            return true;
        }
        false
    }
    // Source: radio_input_type.cc:175-193.
    pub fn HandleKeyupEvent(&self, event: &EventRef, keyboard: &CheckableKeyboardEvent) {
        if keyboard.key == " "
            || (self.environment.SpatialNavigationEnabled(self.GetElement())
                && keyboard.key == "Enter")
        {
            if self.GetElement().Checked() {
                self.GetElement().SetActive(false);
            } else {
                self.base.view.DispatchSimulatedClickIfActive(event);
            }
        }
    }
    // Source: radio_input_type.cc:195-243.
    pub fn IsKeyboardFocusableSlow(&self, update: UpdateBehavior) -> bool {
        if !self.base.input_type.IsKeyboardFocusableSlow(update) {
            return false;
        }
        if self.environment.SpatialNavigationEnabled(self.GetElement()) {
            return true;
        }
        if self.GetElement().Checked() {
            return true;
        }
        let checked = self.CheckedRadioButtonForGroup();
        if let Some(checked) = checked {
            assert!(!Rc::ptr_eq(&checked, self.GetElement()));
            if self.environment.IsKeyboardFocusableSlow(&checked, update) {
                return false;
            }
        }
        if let Some(scope) = self.environment.GroupScope(self.GetElement()) {
            if let Some(last) = scope.LastFocusedButtonForGroup(&self.GetElement().GetName()) {
                if !Rc::ptr_eq(&last, self.GetElement()) {
                    return false;
                }
            }
        }
        true
    }
    // Source: radio_input_type.cc:245-249.
    pub fn ShouldSendChangeEventAfterCheckedChanged(&self) -> bool {
        self.GetElement().Checked()
    }
    // Source: radio_input_type.cc:251-276.
    pub fn LegacyPreActivationBehavior(&self) -> ClickHandlingState {
        Self::LegacyPreActivationWithOperations(self)
    }
    // Source: radio_input_type.cc:251-276.
    pub fn LegacyPreActivationWithOperations<E>(
        element: &dyn RadioActivationOperations<E>,
    ) -> ClickHandlingState<E> {
        let state = ClickHandlingState {
            checked: element.Checked(),
            checked_radio_button: element.CheckedRadioButtonForGroup(),
            indeterminate: false,
        };
        element.SetChecked(true, TextFieldEventBehavior::kDispatchChangeEvent);
        element.SetInClickHandler(true);
        state
    }
    // Source: radio_input_type.cc:278-315.
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
        element: &dyn RadioActivationOperations<E>,
        canceled: bool,
        trusted: bool,
        state: &ClickHandlingState<E>,
    ) {
        if canceled {
            if let Some(checked) = &state.checked_radio_button {
                if element.IsSameRadioGroup(checked) {
                    element.SetRadioChecked(checked, TextFieldEventBehavior::kDispatchNoEvent);
                }
            } else {
                element.SetChecked(false, TextFieldEventBehavior::kDispatchNoEvent);
            }
        } else if state.checked != element.Checked() {
            if element.CSSUserValidAndUserInvalidForRadioEnabled() && trusted {
                element.SetUserHasEditedTheField();
            }
            element.DispatchInputAndChangeEventIfNeeded();
        }
        element.SetInClickHandler(false);
    }
    // Source: radio_input_type.cc:317-319.
    pub fn ShouldAppearIndeterminate(&self) -> bool {
        self.CheckedRadioButtonForGroup().is_none()
    }
    // Source: radio_input_type.cc:321-338,47-53.
    pub fn NextRadioButtonInGroup(
        current: &InputElementRef,
        forward: bool,
        environment: &dyn RadioInputEnvironment,
    ) -> Option<InputElementRef> {
        let mut next = environment.NextInputElement(
            current,
            environment.Form(current).as_ref(),
            None,
            forward,
        );
        while let Some(input) = next {
            if SameOwner(environment.Form(current), environment.Form(&input))
                && Rc::ptr_eq(
                    &environment.TreeScope(&input),
                    &environment.TreeScope(current),
                )
                && environment.IsRadio(&input)
                && input.GetName() == current.GetName()
            {
                return Some(input);
            }
            next = environment.NextInputElement(
                &input,
                environment.Form(current).as_ref(),
                None,
                forward,
            );
        }
        None
    }
    // Source: radio_input_type.cc:340-365.
    pub fn CheckedRadioButtonForGroup(&self) -> Option<InputElementRef> {
        let input = self.GetElement();
        if input.Checked() {
            return Some(input.clone());
        }
        if let Some(scope) = self.environment.GroupScope(input) {
            return scope.CheckedButtonForGroup(&input.GetName());
        }
        debug_assert!(!self.environment.IsConnected(input));
        debug_assert!(!self.environment.HasFormOwner(input));
        let name = input.GetName();
        if name.is_empty() {
            return None;
        }
        let root = self.environment.TreeRoot(input);
        let mut another = self.environment.InclusiveFirstInputWithin(&root);
        while let Some(current) = another {
            if self.environment.IsRadio(&current)
                && current.GetName() == name
                && !self.environment.HasFormOwner(&current)
                && current.Checked()
            {
                return Some(current);
            }
            another = self
                .environment
                .NextInputElement(&current, None, Some(&root), true);
        }
        None
    }
    // Source: radio_input_type.cc:367-377.
    pub fn WillUpdateCheckedness(&self, new_checked: bool) {
        if !new_checked {
            return;
        }
        if self.environment.GroupScope(self.GetElement()).is_some() {
            return;
        }
        if let Some(input) = self.CheckedRadioButtonForGroup() {
            input.SetChecked(false, TextFieldEventBehavior::kDispatchNoEvent);
        }
    }
    // Source: radio_input_type.cc:379-385.
    pub fn SupportsBaseAppearance(&self, value: BaseAppearanceValue) -> bool {
        if !self.environment.AppearanceBaseEnabled() {
            return false;
        }
        value == BaseAppearanceValue::kBase
    }
}
impl Deref for RadioInputType {
    type Target = BaseCheckableInputType;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl RadioActivationOperations<InputElementRef> for RadioInputType {
    fn Checked(&self) -> bool {
        self.GetElement().Checked()
    }
    fn CheckedRadioButtonForGroup(&self) -> Option<InputElementRef> {
        RadioInputType::CheckedRadioButtonForGroup(self)
    }
    fn SetChecked(&self, value: bool, behavior: TextFieldEventBehavior) {
        self.GetElement().SetChecked(value, behavior);
    }
    fn IsSameRadioGroup(&self, checked: &InputElementRef) -> bool {
        self.environment.IsRadio(checked)
            && SameOwner(
                self.environment.Form(checked),
                self.environment.Form(self.GetElement()),
            )
            && checked.GetName() == self.GetElement().GetName()
    }
    fn SetRadioChecked(&self, checked: &InputElementRef, behavior: TextFieldEventBehavior) {
        checked.SetChecked(true, behavior);
    }
    fn SetInClickHandler(&self, value: bool) {
        self.base.is_in_click_handler_.set(value);
    }
    fn CSSUserValidAndUserInvalidForRadioEnabled(&self) -> bool {
        self.environment.CSSUserValidAndUserInvalidForRadioEnabled()
    }
    fn SetUserHasEditedTheField(&self) {
        self.GetElement().SetUserHasEditedTheField();
    }
    fn DispatchInputAndChangeEventIfNeeded(&self) {
        self.GetElement().DispatchInputAndChangeEventIfNeeded();
    }
}

struct SourceRadioKeyboard<'a> {
    radio: &'a RadioInputType,
    event: Option<&'a EventRef>,
}
impl RadioKeyboardOperations<InputElementRef> for SourceRadioKeyboard<'_> {
    fn Current(&self) -> InputElementRef {
        self.radio.GetElement().clone()
    }
    fn HasLayoutObject(&self) -> bool {
        self.radio.GetElement().HasLayoutObject()
    }
    fn SetActive(&self, active: bool) {
        self.radio.GetElement().SetActive(active);
    }
    fn IsRightToLeft(&self) -> bool {
        self.radio.GetElement().IsRightToLeft()
    }
    fn SpatialNavigationEnabled(&self) -> bool {
        self.radio
            .environment
            .SpatialNavigationEnabled(self.radio.GetElement())
    }
    fn NextRadioButtonInGroup(
        &self,
        current: &InputElementRef,
        forward: bool,
    ) -> Option<InputElementRef> {
        RadioInputType::NextRadioButtonInGroup(current, forward, self.radio.environment.as_ref())
    }
    fn IsFocusable(&self, input: &InputElementRef) -> bool {
        input.IsFocusable(UpdateBehavior::kStyleAndLayout)
    }
    fn FocusRestoringSelection(&self, input: &InputElementRef) {
        self.radio
            .environment
            .SetFocusedElementRestoringSelection(input);
    }
    fn DispatchSimulatedClick(&self, input: &InputElementRef) {
        input.DispatchSimulatedClick(
            self.event
                .expect("keyboard dispatch retains original event"),
        );
    }
}
