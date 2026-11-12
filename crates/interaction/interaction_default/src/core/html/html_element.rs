/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 * Copyright (C) 2004-2008, 2013, 2014 Apple Inc. All rights reserved.
 * Copyright (C) 2009 Torch Mobile Inc. All rights reserved.
 * (http://www.torchmobile.com/)
 * Copyright (C) 2011 Motorola Mobility. All rights reserved.
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
// Source: third_party/blink/renderer/core/html/html_element.h:238,243-245,403,503-504
// Source: third_party/blink/renderer/core/html/html_element.cc:3376-3419,4171-4240,4253-4297
// Only activation and keyboard default actions are translated here. DOM,
// command target actions, style lifecycle and ChromeClient retain their owners.

use crate::core::dom::element::{CommandEventType, FocusableState, UpdateBehavior};
use interaction_dispatch::core::dom::events::event::{EventRef, EventTargetRef};
use interaction_dispatch::core::events::keyboard_event::KeyboardEvent;
use std::{cell::RefCell, rc::Rc};

// Source: html_element.cc:3377-3416. The target is the live DOM Element owner;
// dispatch is synchronous so cancellation is visible before its default action.
pub trait HTMLElementCommandTarget {
    fn IsValidBuiltinCommand(&self, invoker: &EventTargetRef, command: CommandEventType) -> bool;
    fn DispatchEvent(&self, event: &EventRef);
    fn HandleCommandInternal(&self, invoker: &EventTargetRef, command: CommandEventType) -> bool;
}

// Source: html_element.cc:4172-4176. The optional shared behavior owner is
// captured before activation, as in the source, rather than looked up again.
pub trait HTMLElementSubmitBehavior {
    fn HandleActivation(&self, event: &EventRef) -> bool;
}

// Source: html_element.cc:4268-4270. This is the existing page's ChromeClient,
// not a second Page or editable-element model.
pub trait HTMLElementChromeClient {
    fn HandleKeyboardEventOnEditableElement(
        &self,
        element: &EventTargetRef,
        event: &Rc<RefCell<KeyboardEvent>>,
    ) -> bool;
}

// Source: html_element.cc:3376-3419,4171-4240,4253-4297. These narrow ports
// execute at the source call points and retain the live DOM/behavior owners.
pub trait HTMLElementEnvironment {
    fn Target(&self) -> EventTargetRef;
    fn CanBeCommandInvoker(&self) -> bool;
    fn commandForElement(&self) -> Option<Rc<dyn HTMLElementCommandTarget>>;
    fn HasPopoverTargetAttribute(&self) -> bool;
    fn AddOtherConsoleWarning(&self, message: &str);
    fn command(&self) -> String;
    fn CommandAttribute(&self) -> String;
    fn GetCommandEventType(&self, action: &str) -> CommandEventType;
    /// Creates the source CommandEvent("command", action, this) owner.
    fn CreateCommandEvent(&self, action: &str) -> EventRef;
    fn ShadowRootReferenceTargetEnabled(&self) -> bool;
    fn SubmitBehavior(&self) -> Option<Rc<dyn HTMLElementSubmitBehavior>>;
    /// Projects exactly the same concrete owner held by EventRef. None is the
    /// source failed DynamicTo; implementations must not copy the event.
    fn KeyboardEvent(&self, event: &EventRef) -> Option<Rc<RefCell<KeyboardEvent>>>;
    fn AutofillKeydownEditableElementEnabled(&self) -> bool;
    fn ElementDefaultEventHandler(&self, event: &EventRef);
    fn SetActive(&self, active: bool);
    fn IsActive(&self) -> bool;
    fn DispatchSimulatedClick(&self, underlying_event: &EventRef);
    fn DispatchAccessKeySimulatedClick(
        &self,
        scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
    );
    fn IsFocusedElementInDocument(&self) -> bool;
    fn UpdateStyleAndLayoutTree(&self);
    fn IsEditable(&self) -> bool;
    /// None represents GetDocument().GetPage() == nullptr.
    fn PageChromeClient(&self) -> Option<Rc<dyn HTMLElementChromeClient>>;
    fn IsSpatialNavigationEnabled(&self) -> bool;
    fn SupportsFocus(&self, update_behavior: UpdateBehavior) -> FocusableState;
    fn IsTextControl(&self) -> bool;
}

pub trait HTMLElementKeyboardActivationOperations {
    fn SetActive(&self, active: bool);
    fn IsActive(&self) -> bool;
    fn DispatchSimulatedClick(&self);
}
/// Source has two independent results: space keydown returns handled from
/// HTMLElement while deliberately leaving Event::DefaultHandled false.
#[derive(Default)]
pub struct KeyboardActivationResult {
    pub matched: bool,
    pub default_handled: bool,
}

pub struct HTMLElement {
    environment: Rc<dyn HTMLElementEnvironment>,
}

pub trait HTMLElementNativeArrowKeyState {
    fn IsContentEditableForBinding(&self) -> bool;
    fn ElementNativeArrowKeyAxes(&self) -> foundation::focusgroup_flags::FocusgroupFlags;
}
// Source: html_element.cc:4163-4169.
pub fn NativeArrowKeyAxes(
    state: &dyn HTMLElementNativeArrowKeyState,
) -> foundation::focusgroup_flags::FocusgroupFlags {
    use foundation::focusgroup_flags::FocusgroupFlags as F;
    if state.IsContentEditableForBinding() {
        F::kInline | F::kBlock
    } else {
        state.ElementNativeArrowKeyAxes()
    }
}

impl HTMLElement {
    // Source: html_element.cc:3697-3699. The inherited action does not focus.
    pub fn AccessKeyAction(
        &self,
        scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
    ) {
        self.environment.DispatchAccessKeySimulatedClick(scope);
    }

    pub fn new(environment: Rc<dyn HTMLElementEnvironment>) -> Self {
        Self { environment }
    }
    // Source: html_element.cc:3433-3450.
    pub fn command(&self) -> String {
        if !self.environment.CanBeCommandInvoker() {
            return String::new();
        }
        let action = self.environment.CommandAttribute();
        match self.environment.GetCommandEventType(&action) {
            CommandEventType::kNone => String::new(),
            CommandEventType::kCustom => action,
            _ => action.to_ascii_lowercase(),
        }
    }

    // Source: html_element.h:403; html_element.cc:3376-3419.
    pub fn HandleCommandForActivation(&self) -> bool {
        if !self.environment.CanBeCommandInvoker() {
            return false;
        }
        let Some(command_target) = self.environment.commandForElement() else {
            return false;
        };
        if self.environment.HasPopoverTargetAttribute() {
            self.environment
                .AddOtherConsoleWarning("popovertarget is ignored on elements with commandfor.");
        }
        let action = self.command();
        if action.is_empty() {
            return false;
        }
        debug_assert_ne!(
            self.environment
                .GetCommandEventType(&self.environment.CommandAttribute()),
            CommandEventType::kNone
        );
        let command_event_type = self.environment.GetCommandEventType(&action);
        let is_valid_builtin =
            command_target.IsValidBuiltinCommand(&self.environment.Target(), command_event_type);
        if !is_valid_builtin && command_event_type != CommandEventType::kCustom {
            return false;
        }
        let command_event = self.environment.CreateCommandEvent(&action);
        if self.environment.ShadowRootReferenceTargetEnabled() {
            command_event.borrow_mut().BaseEventMut().SetComposed(true);
        }
        command_target.DispatchEvent(&command_event);
        let default_prevented = command_event.borrow().BaseEvent().defaultPrevented();
        if !default_prevented && command_event_type != CommandEventType::kCustom {
            command_target.HandleCommandInternal(&self.environment.Target(), command_event_type);
        }
        true
    }

    // Source: html_element.h:238; html_element.cc:4171-4208.
    pub fn DefaultEventHandler(&self, event: &EventRef) {
        let submit_behavior = self.environment.SubmitBehavior();
        let is_dom_activate = event.borrow().BaseEvent().r#type() == "DOMActivate";
        if is_dom_activate {
            if let Some(submit_behavior) = &submit_behavior {
                if submit_behavior.HandleActivation(event) {
                    return;
                }
            }
            if self.HandleCommandForActivation() {
                return;
            }
        }
        if submit_behavior.is_some() && self.HandleKeyboardActivation(event) {
            return;
        }
        if let Some(keyboard_event) = self.environment.KeyboardEvent(event) {
            if self.environment.AutofillKeydownEditableElementEnabled()
                && event.borrow().BaseEvent().r#type() == "keydown"
            {
                self.HandleKeydownEvent(&keyboard_event);
                if event.borrow().BaseEvent().DefaultHandled() {
                    return;
                }
            } else if event.borrow().BaseEvent().r#type() == "keypress" {
                self.HandleKeypressEvent(event, &keyboard_event);
                if event.borrow().BaseEvent().DefaultHandled() {
                    return;
                }
            }
        }
        self.environment.ElementDefaultEventHandler(event);
    }

    // Source: html_element.h:243-245; html_element.cc:4210-4240.
    pub fn HandleKeyboardActivation(&self, event: &EventRef) -> bool {
        let values = self.environment.KeyboardEvent(event).map(|keyboard| {
            let keyboard = keyboard.borrow();
            (
                keyboard.r#type().to_owned(),
                keyboard.key().to_owned(),
                keyboard.charCode(),
            )
        });
        let Some((kind, key, char_code)) = values else {
            return false;
        };
        let result = Self::HandleKeyboardActivationWithOperations(
            &kind,
            &key,
            char_code,
            &SourceHTMLElementKeyboard {
                element: self,
                event,
            },
        );
        if result.default_handled {
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
        }
        result.matched
    }
    // Source: html_element.cc:4210-4240.
    pub fn HandleKeyboardActivationWithOperations(
        kind: &str,
        key: &str,
        char_code: i32,
        operations: &dyn HTMLElementKeyboardActivationOperations,
    ) -> KeyboardActivationResult {
        if kind == "keydown" && key == " " {
            operations.SetActive(true);
            return KeyboardActivationResult {
                matched: true,
                default_handled: false,
            };
        }
        if kind == "keypress" {
            match char_code {
                13 => {
                    operations.DispatchSimulatedClick();
                    return KeyboardActivationResult {
                        matched: true,
                        default_handled: true,
                    };
                }
                32 => {
                    return KeyboardActivationResult {
                        matched: true,
                        default_handled: true,
                    }
                }
                _ => {}
            }
        }
        if kind == "keyup" && key == " " {
            if operations.IsActive() {
                operations.DispatchSimulatedClick();
            }
            return KeyboardActivationResult {
                matched: true,
                default_handled: true,
            };
        }
        KeyboardActivationResult::default()
    }

    // Source: html_element.h:503; html_element.cc:4253-4274.
    fn HandleKeydownEvent(&self, event: &Rc<RefCell<KeyboardEvent>>) {
        let is_focused_contenteditable = if !self.environment.IsFocusedElementInDocument() {
            false
        } else {
            self.environment.UpdateStyleAndLayoutTree();
            self.environment.IsEditable()
        };
        if is_focused_contenteditable {
            self.environment.UpdateStyleAndLayoutTree();
            if let Some(chrome_client) = self.environment.PageChromeClient() {
                if chrome_client
                    .HandleKeyboardEventOnEditableElement(&self.environment.Target(), event)
                {
                    event.borrow_mut().SetDefaultHandled();
                }
            }
        }
    }

    // Source: html_element.h:504; html_element.cc:4276-4297.
    fn HandleKeypressEvent(&self, event: &EventRef, keyboard_event: &Rc<RefCell<KeyboardEvent>>) {
        if !self.environment.IsSpatialNavigationEnabled()
            || self
                .environment
                .SupportsFocus(UpdateBehavior::kStyleAndLayout)
                == FocusableState::kNotFocusable
        {
            return;
        }
        // SupportsFocus overrides do not all ensure clean style and layout.
        self.environment.UpdateStyleAndLayoutTree();
        if self.environment.IsTextControl() || self.environment.IsEditable() {
            return;
        }
        let char_code = keyboard_event.borrow().charCode();
        if char_code == 13 || char_code == 32 {
            self.environment.DispatchSimulatedClick(event);
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
        }
    }
}

struct SourceHTMLElementKeyboard<'a> {
    element: &'a HTMLElement,
    event: &'a EventRef,
}
impl HTMLElementKeyboardActivationOperations for SourceHTMLElementKeyboard<'_> {
    fn SetActive(&self, active: bool) {
        self.element.environment.SetActive(active);
    }
    fn IsActive(&self) -> bool {
        self.element.environment.IsActive()
    }
    fn DispatchSimulatedClick(&self) {
        self.element.environment.DispatchSimulatedClick(self.event);
    }
}
