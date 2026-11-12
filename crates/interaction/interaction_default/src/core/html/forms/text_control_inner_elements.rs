/*
 * Copyright (C) 2006, 2008, 2010 Apple Inc. All rights reserved.
 * Copyright (C) 2010 Google Inc. All rights reserved.
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the copyright notice and this
 * disclaimer are retained. THIS SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/html/forms/text_control_inner_elements.h:49-92
// Source: third_party/blink/renderer/core/html/forms/text_control_inner_elements.cc:75-110,224-299
// Style/layout callbacks remain with their respective owners. These methods
// borrow the resident shadow element and dispatch the original event, including
// synchronous reentry; they never retain a projected shadow tree.

use crate::public::web::web_autofill_state::WebAutofillState;
use interaction_dispatch::core::dom::events::event::EventRef;
use std::rc::Rc;

// Source: text_control_inner_elements.cc:227-228,266-268;
// html/shadow/shadow_element_names.json5:44-46,60-66.
// Values identify existing UA-shadow elements; the DOM port resolves them on
// its own live tree rather than translating them into a second tree here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextControlShadowId {
    EditingViewPort,
    SearchClearButton,
    PasswordRevealButton,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextControlShadowPseudoId {
    WebkitSearchCancelButton,
    InternalReveal,
}

/// The existing HTMLDivElement owner supplies its inherited behavior and
/// constructor mutations. Neither call holds an EventRef borrow across reentry.
pub trait TextControlInnerElement {
    fn SetShadowPseudoId(&self, pseudo: TextControlShadowPseudoId);
    fn SetIdAttribute(&self, id: TextControlShadowId);
    fn SetHasCustomStyleCallbacks(&self);
    fn HTMLDivDefaultEventHandler(&self, event: &EventRef);
    fn HTMLDivWillRespondToMouseClickEvents(&self) -> bool;
}

/// These projections refer to the very same resident HTMLInputElement used by
/// InputType/TextControlElement. Values and password/autofill state remain on
/// that owner, including while SetValueForUser synchronously dispatches input.
pub trait TextControlInputHost {
    fn IsDisabledOrReadOnly(&self) -> bool;
    fn SetValueForUser(&self, value: &[u16]);
    fn SetAutofillState(&self, state: WebAutofillState);
    fn OnSearch(&self);
    fn ShouldRevealPassword(&self) -> bool;
    fn SetShouldRevealPassword(&self, reveal: bool);
    fn UpdateView(&self);
}
/// Search-button primitives, borrowed for a single default-handler call.
/// The wider password/input host interface is unnecessary at this boundary.
pub trait SearchCancelInputOperations {
    fn IsDisabledOrReadOnly(&self) -> bool;
    fn SetValueForUser(&self, value: &[u16]);
    fn SetAutofillState(&self, state: WebAutofillState);
    fn OnSearch(&self);
}
impl<T: TextControlInputHost + ?Sized> SearchCancelInputOperations for T {
    fn IsDisabledOrReadOnly(&self) -> bool {
        TextControlInputHost::IsDisabledOrReadOnly(self)
    }
    fn SetValueForUser(&self, value: &[u16]) {
        TextControlInputHost::SetValueForUser(self, value);
    }
    fn SetAutofillState(&self, state: WebAutofillState) {
        TextControlInputHost::SetAutofillState(self, state);
    }
    fn OnSearch(&self) {
        TextControlInputHost::OnSearch(self);
    }
}

pub type TextControlInputHostRef = Rc<dyn TextControlInputHost>;

pub trait TextControlInputShadowHost {
    fn OwnerShadowHost(&self) -> Option<TextControlInputHostRef>;
}

pub trait TextControlInnerEventProjection {
    // DynamicTo<MouseEvent> with button(), not merely IsMouseEvent().
    fn MouseButton(&self, event: &EventRef) -> Option<i16>;
}

pub trait InnerEditorShadowHost {
    fn DefaultEventHandler(&self, event: &EventRef);
    fn DispatchEvent(&self, event: &EventRef);
}
pub trait InnerEditorStyleInvalidation {
    fn SetNeedsLocalControlStyleRecalc(&self);
}
// Source: text_control_inner_elements.cc:121-126.
pub fn InnerEditorFocusChanged(element: &dyn InnerEditorStyleInvalidation) {
    element.SetNeedsLocalControlStyleRecalc();
}
pub trait InnerEditorShadowOwner {
    fn OwnerShadowHost(&self) -> Option<Rc<dyn InnerEditorShadowHost>>;
}

pub struct TextControlInnerEditorElement<'a> {
    element: &'a dyn TextControlInnerElement,
    shadow: &'a dyn InnerEditorShadowOwner,
}
impl<'a> TextControlInnerEditorElement<'a> {
    // Source: text_control_inner_elements.cc:75-78.
    pub fn new(
        element: &'a dyn TextControlInnerElement,
        shadow: &'a dyn InnerEditorShadowOwner,
    ) -> Self {
        element.SetHasCustomStyleCallbacks();
        Self { element, shadow }
    }

    // Source: text_control_inner_elements.cc:80-110. A detached editor can be
    // retained by EditCommand; absence of a host must not forward recursively.
    pub fn DefaultEventHandler(&self, event: &EventRef) {
        Self::HandleDefaultEvent(self.shadow, event, |event| {
            self.element.HTMLDivDefaultEventHandler(event)
        });
    }
    /// The operation borrows the canonical DOM subclass; no additional object
    /// or copied shadow tree is required by an embedding with retained nodes.
    pub fn HandleDefaultEvent(
        shadow: &dyn InnerEditorShadowOwner,
        event: &EventRef,
        inherited_default: impl FnOnce(&EventRef),
    ) {
        let forward_edit = {
            let event = event.borrow();
            event.IsBeforeTextInsertedEvent()
                || event.BaseEvent().r#type() == "webkitEditableContentChanged"
        };
        if forward_edit {
            if let Some(host) = shadow.OwnerShadowHost() {
                host.DefaultEventHandler(event);
            }
        }
        let forward_scroll = matches!(event.borrow().BaseEvent().r#type(), "scroll" | "scrollend");
        if forward_scroll {
            if let Some(host) = shadow.OwnerShadowHost() {
                host.DispatchEvent(event);
            }
        }
        if !event.borrow().BaseEvent().DefaultHandled() {
            inherited_default(event);
        }
    }
}

pub struct SearchFieldCancelButtonElement<'a> {
    element: &'a dyn TextControlInnerElement,
    shadow: &'a dyn TextControlInputShadowHost,
    projection: &'a dyn TextControlInnerEventProjection,
}
impl<'a> SearchFieldCancelButtonElement<'a> {
    // Source: text_control_inner_elements.cc:224-229.
    pub fn new(
        element: &'a dyn TextControlInnerElement,
        shadow: &'a dyn TextControlInputShadowHost,
        projection: &'a dyn TextControlInnerEventProjection,
    ) -> Self {
        element.SetShadowPseudoId(TextControlShadowPseudoId::WebkitSearchCancelButton);
        element.SetIdAttribute(TextControlShadowId::SearchClearButton);
        Self {
            element,
            shadow,
            projection,
        }
    }

    // Source: text_control_inner_elements.cc:231-252.
    pub fn DefaultEventHandler(&self, event: &EventRef) {
        let input = self.shadow.OwnerShadowHost();
        let click = event.borrow().BaseEvent().r#type() == "click";
        let button = self.projection.MouseButton(event);
        if input
            .as_ref()
            .is_some_and(|input| Self::HandleDefaultEventWithInput(input.as_ref(), click, button))
        {
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
        }
        if !event.borrow().BaseEvent().DefaultHandled() {
            self.element.HTMLDivDefaultEventHandler(event);
        }
    }

    // Source: text_control_inner_elements.cc:231-247. The same body accepts
    // borrowed source operations when an embedding retains opaque DOM ids.
    pub fn HandleDefaultEventWithInput<T: SearchCancelInputOperations + ?Sized>(
        input: &T,
        click: bool,
        button: Option<i16>,
    ) -> bool {
        if input.IsDisabledOrReadOnly() {
            return false;
        }
        if click && button == Some(0) {
            input.SetValueForUser(&[]);
            input.SetAutofillState(WebAutofillState::kNotFilled);
            input.OnSearch();
            return true;
        }
        false
    }

    // Source: text_control_inner_elements.cc:254-260.
    pub fn WillRespondToMouseClickEvents(&self) -> bool {
        if self
            .shadow
            .OwnerShadowHost()
            .is_some_and(|input| !input.IsDisabledOrReadOnly())
        {
            return true;
        }
        self.element.HTMLDivWillRespondToMouseClickEvents()
    }
}

pub struct PasswordRevealButtonElement<'a> {
    element: &'a dyn TextControlInnerElement,
    shadow: &'a dyn TextControlInputShadowHost,
}
impl<'a> PasswordRevealButtonElement<'a> {
    // Source: text_control_inner_elements.cc:264-269.
    pub fn new(
        element: &'a dyn TextControlInnerElement,
        shadow: &'a dyn TextControlInputShadowHost,
    ) -> Self {
        element.SetShadowPseudoId(TextControlShadowPseudoId::InternalReveal);
        element.SetIdAttribute(TextControlShadowId::PasswordRevealButton);
        Self { element, shadow }
    }

    // Source: text_control_inner_elements.cc:271-291. Unlike the cancel
    // button, this source branch accepts a MouseEvent with any button.
    pub fn DefaultEventHandler(&self, event: &EventRef) {
        let input = self.shadow.OwnerShadowHost();
        let enabled = input
            .as_ref()
            .is_some_and(|input| !input.IsDisabledOrReadOnly());
        if !enabled {
            if !event.borrow().BaseEvent().DefaultHandled() {
                self.element.HTMLDivDefaultEventHandler(event);
            }
            return;
        }
        let click = {
            let event = event.borrow();
            event.BaseEvent().r#type() == "click" && event.IsMouseEvent()
        };
        if click {
            let input = input.unwrap();
            input.SetShouldRevealPassword(!input.ShouldRevealPassword());
            input.UpdateView();
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
        }
        if !event.borrow().BaseEvent().DefaultHandled() {
            self.element.HTMLDivDefaultEventHandler(event);
        }
    }

    // Source: text_control_inner_elements.cc:293-299.
    pub fn WillRespondToMouseClickEvents(&self) -> bool {
        if self
            .shadow
            .OwnerShadowHost()
            .is_some_and(|input| !input.IsDisabledOrReadOnly())
        {
            return true;
        }
        self.element.HTMLDivWillRespondToMouseClickEvents()
    }
}
