/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 *           (C) 2001 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2004, 2005, 2006, 2007, 2010 Apple Inc. All rights reserved.
 *           (C) 2006 Alexey Proskuryakov (ap@nypop.com)
 * Copyright (C) 2007 Samuel Weinig (sam@webkit.org)
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
// Source: third_party/blink/renderer/core/html/forms/html_button_element.h:47-60,67-75,89-108,121,127-128
// Source: third_party/blink/renderer/core/html/forms/html_button_element.cc:206-309,316-337

use super::base_button_input_type::ButtonFormData;
use super::html_form_element::{FormControlRef, HTMLFormElement};
use crate::public::mojom::forms::form_control_type::FormControlType;
use interaction_dispatch::core::dom::events::event::EventRef;
use interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope;
use std::{cell::Cell, rc::Rc};

// Source: html_button_element.h:67-75.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Type {
    kSubmit = FormControlType::kButtonSubmit as i32,
    kReset = FormControlType::kButtonReset as i32,
    kButton = FormControlType::kButtonButton as i32,
}
pub trait ButtonFormDefaultOperations<F>:
    super::base_button_input_type::ButtonFormActivationOperations<F>
{
    fn ControlType(&self) -> Type;
    fn TypeAttribute(&self) -> String;
    fn HasCommandOrCommandForAttribute(&self) -> bool;
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonFormDefaultResult {
    Continue,
    Handled,
    Stop,
}

pub trait ButtonElementEnvironment {
    fn Form(&self) -> Option<Rc<HTMLFormElement>>;
    fn AsFormControl(&self) -> FormControlRef;
    fn IsDisabledFormControl(&self) -> bool;
    fn HasTypeAttribute(&self) -> bool;
    fn TypeAttribute(&self) -> String;
    fn HasCommandOrCommandForAttribute(&self) -> bool;
    fn HandleCommandForActivation(&self) -> bool;
    fn HandleKeyboardActivation(&self, event: &EventRef) -> bool;
    fn BaseDefaultEventHandler(&self, event: &EventRef);
    fn BaseWillRespondToMouseClickEvents(&self) -> bool;
    fn HasOwnerSelect(&self) -> bool;
    fn GetName(&self) -> String;
    fn ValueAttribute(&self) -> String;
    fn NameForFormData(&self) -> Rc<[u16]>;
    fn ValueForFormData(&self) -> Rc<[u16]>;
    fn FocusFromUserGesture(&self);
    fn DispatchSimulatedClick(&self, scope: SimulatedClickCreationScope);
    fn BaseRecalcWillValidate(&self) -> bool;
}
pub struct HTMLButtonElement {
    environment: Rc<dyn ButtonElementEnvironment>,
    type_: Cell<Type>,
    is_activated_submit_: Cell<bool>,
}
impl HTMLButtonElement {
    pub fn new(environment: Rc<dyn ButtonElementEnvironment>) -> Self {
        Self {
            environment,
            type_: Cell::new(Type::kSubmit),
            is_activated_submit_: Cell::new(false),
        }
    }
    pub fn SetType(&self, value: Type) {
        self.type_.set(value);
    }
    // Source: html_button_element.cc:82-84.
    pub fn FormControlType(&self) -> FormControlType {
        match self.type_.get() {
            Type::kSubmit => FormControlType::kButtonSubmit,
            Type::kReset => FormControlType::kButtonReset,
            Type::kButton => FormControlType::kButtonButton,
        }
    }
    // Source: html_button_element.cc:206-217.
    pub fn CanBeCommandInvoker(&self) -> bool {
        !self.IsFormAssociatedSubmitButton()
    }
    pub fn IsValidInterestInvoker(&self) -> bool {
        !self.environment.IsDisabledFormControl()
    }
    pub fn IsFormAssociatedSubmitButton(&self) -> bool {
        self.environment.Form().is_some()
            && self.environment.HasTypeAttribute()
            && self.type_.get() == Type::kSubmit
    }
    // Source: html_button_element.cc:219-273.
    pub fn DefaultEventHandler(&self, event: &EventRef) {
        let dom_activate = event.borrow().BaseEvent().r#type() == "DOMActivate";
        if dom_activate {
            match Self::HandleFormDOMActivateWithOperations(&SourceButtonDefault {
                button: self,
                event,
            }) {
                ButtonFormDefaultResult::Handled => {
                    event.borrow_mut().BaseEventMut().SetDefaultHandled();
                    return;
                }
                ButtonFormDefaultResult::Stop => return,
                ButtonFormDefaultResult::Continue => {}
            }
            if self.environment.HandleCommandForActivation() {
                return;
            }
        }
        if self.environment.HandleKeyboardActivation(event) {
            return;
        }
        self.environment.BaseDefaultEventHandler(event);
    }
    // Source: html_button_element.cc:116-127,131-155,177-194. This is the
    // source attribute-state decision for embeddings with opaque DOM owners.
    pub fn TypeForAttributes(value: &str, has_command: bool) -> Type {
        if value.eq_ignore_ascii_case("reset") {
            Type::kReset
        } else if value.eq_ignore_ascii_case("button") {
            Type::kButton
        } else if value.eq_ignore_ascii_case("submit") {
            Type::kSubmit
        } else if has_command {
            Type::kButton
        } else {
            Type::kSubmit
        }
    }
    // Source: html_button_element.cc:219-260. Command execution and keyboard
    // handling remain subsequent source stages, never forced into form submit.
    pub fn HandleFormDOMActivateWithOperations<F>(
        operations: &dyn ButtonFormDefaultOperations<F>,
    ) -> ButtonFormDefaultResult {
        if let Some(form) = operations
            .Form()
            .filter(|_| !operations.IsDisabledFormControl())
        {
            if operations.HasCommandOrCommandForAttribute()
                && operations.ControlType() == Type::kButton
                && !operations.TypeAttribute().eq_ignore_ascii_case("button")
            {
                return ButtonFormDefaultResult::Stop;
            }
            if operations.ControlType() == Type::kSubmit {
                operations.PrepareForSubmission(&form);
                return ButtonFormDefaultResult::Handled;
            }
            if operations.ControlType() == Type::kReset {
                operations.ResetForm(&form);
                return ButtonFormDefaultResult::Handled;
            }
        }
        ButtonFormDefaultResult::Continue
    }
    // Source: html_button_element.cc:275-309.
    pub fn HasActivationBehavior(&self) -> bool {
        true
    }
    pub fn WillRespondToMouseClickEvents(&self) -> bool {
        if !self.environment.IsDisabledFormControl()
            && self.environment.Form().is_some()
            && matches!(self.type_.get(), Type::kSubmit | Type::kReset)
        {
            return true;
        }
        self.environment.BaseWillRespondToMouseClickEvents()
    }
    pub fn CanBeSuccessfulSubmitButton(&self) -> bool {
        Self::CanBeSuccessfulSubmitButtonWithFacts(
            self.type_.get(),
            self.environment.HasOwnerSelect(),
        )
    }
    // Source html_button_element.cc:287-289. OwnerSelect is the canonical
    // parent's slotted-button identity, not general form association.
    pub fn CanBeSuccessfulSubmitButtonWithFacts(button_type: Type, has_owner_select: bool) -> bool {
        button_type == Type::kSubmit && !has_owner_select
    }
    pub fn IsActivatedSubmit(&self) -> bool {
        self.is_activated_submit_.get()
    }
    pub fn SetActivatedSubmit(&self, flag: bool) {
        self.is_activated_submit_.set(flag);
    }
    pub fn AppendToFormData(&self, data: &mut dyn ButtonFormData) {
        Self::AppendWithOperations(
            self.type_.get(),
            self.is_activated_submit_.get(),
            self.environment.NameForFormData(),
            self.environment.ValueForFormData(),
            data,
        );
    }
    // Source html_button_element.cc:295-302.
    pub fn AppendWithOperations(
        button_type: Type,
        activated: bool,
        name: Rc<[u16]>,
        value: Rc<[u16]>,
        data: &mut dyn ButtonFormData,
    ) {
        if button_type == Type::kSubmit && !name.is_empty() && activated {
            data.AppendFromElement(name, value);
        }
    }
    pub fn AccessKeyAction(&self, scope: SimulatedClickCreationScope) {
        self.environment.FocusFromUserGesture();
        self.environment.DispatchSimulatedClick(scope);
    }
    // Source: html_button_element.cc:316-337.
    pub fn Value(&self) -> String {
        self.environment.ValueAttribute()
    }
    pub fn RecalcWillValidate(&self) -> bool {
        self.type_.get() == Type::kSubmit && self.environment.BaseRecalcWillValidate()
    }
    pub fn DefaultTabIndex(&self) -> i32 {
        0
    }
    pub fn IsInteractiveContent(&self) -> bool {
        true
    }
    pub fn MatchesDefaultPseudoClass(&self) -> bool {
        self.CanBeSuccessfulSubmitButton()
            && self.environment.Form().is_some_and(|form| {
                form.FindDefaultButton()
                    .is_some_and(|button| Rc::ptr_eq(&button, &self.environment.AsFormControl()))
            })
    }
}

struct SourceButtonDefault<'a> {
    button: &'a HTMLButtonElement,
    event: &'a EventRef,
}
impl super::base_button_input_type::ButtonFormActivationOperations<Rc<HTMLFormElement>>
    for SourceButtonDefault<'_>
{
    fn Form(&self) -> Option<Rc<HTMLFormElement>> {
        self.button.environment.Form()
    }
    fn IsDisabledFormControl(&self) -> bool {
        self.button.environment.IsDisabledFormControl()
    }
    fn PrepareForSubmission(&self, form: &Rc<HTMLFormElement>) {
        form.PrepareForSubmission(
            Some(self.event),
            Some(self.button.environment.AsFormControl()),
        );
    }
    fn ResetForm(&self, form: &Rc<HTMLFormElement>) {
        form.reset();
    }
}
impl ButtonFormDefaultOperations<Rc<HTMLFormElement>> for SourceButtonDefault<'_> {
    fn ControlType(&self) -> Type {
        self.button.type_.get()
    }
    fn TypeAttribute(&self) -> String {
        self.button.environment.TypeAttribute()
    }
    fn HasCommandOrCommandForAttribute(&self) -> bool {
        self.button.environment.HasCommandOrCommandForAttribute()
    }
}
