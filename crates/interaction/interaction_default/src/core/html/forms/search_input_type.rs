/*
 * Copyright (C) 2010 Google Inc. All rights reserved.
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the copyright notice and this
 * disclaimer are retained. THIS SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/html/forms/search_input_type.h:39-69
// Source: third_party/blink/renderer/core/html/forms/search_input_type.cc:52-171
// Source: third_party/blink/renderer/core/html/forms/text_field_input_type.cc:830-842

use super::{
    base_text_input_type::{BaseTextInputType, ScriptRegexpFactory},
    input_type::{InputType, Type},
    text_control_inner_elements::{TextControlInputHostRef, TextControlShadowId},
    text_field_input_type::{TextFieldInputType, TextFieldInteraction},
};
use interaction_dispatch::core::dom::events::event::EventRef;
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchAppearance {
    SearchField,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchCancelInlineProperty {
    Opacity,
    PointerEvents,
}

/// The existing shadow DOM/style owners retain element identity. Only values
/// needed by the source DOM calls cross this boundary; no tree is copied.
pub trait SearchShadowElement {
    fn NativeIdentity(&self) -> u64;
    fn NextSibling(&self) -> Option<SearchShadowElementRef>;
    fn InsertBefore(&self, child: SearchShadowElementRef, before: Option<SearchShadowElementRef>);
    fn SetInlineOpacityNumber(&self, value: f64);
    fn SetInlinePointerEventsNone(&self);
    fn RemoveInlineStyleProperty(&self, property: SearchCancelInlineProperty);
}
pub type SearchShadowElementRef = Rc<dyn SearchShadowElement>;

pub trait SearchShadowRoot {
    fn GetElementById(&self, id: TextControlShadowId) -> Option<SearchShadowElementRef>;
}

pub trait SearchInputShadowOwner {
    fn TextFieldCreateShadowSubtree(&self);
    fn ContainerElement(&self) -> Option<SearchShadowElementRef>;
    fn UserAgentShadowRoot(&self) -> Rc<dyn SearchShadowRoot>;
    fn EnsureShadowSubtree(&self) -> Rc<dyn SearchShadowRoot>;
    fn CreateSearchFieldCancelButtonElement(&self) -> SearchShadowElementRef;
}

/// The document's kUserInteraction task runner owns this timer. Its callback
/// invokes SearchEventTimerFired on this SearchInputType owner; dropping the
/// retained timer cancels it, matching HeapTaskRunnerTimer's lifetime. Starting
/// replaces its current one-shot rather than queuing another independent task.
pub trait SearchEventTimer {
    fn Stop(&self);
    fn StartOneShot(&self, timeout: Duration);
}

/// Operations on the same HTMLInputElement as base_text.text_field.value and
/// input_host. PostOnSearchTask retains that element through the posted task,
/// as WrapPersistent does at the source call site.
pub trait SearchInputElement {
    fn HasLayoutObject(&self) -> bool;
    fn HasIncrementalAttribute(&self) -> bool;
    fn PostOnSearchTask(&self);
    fn DispatchBubblingSearchEvent(&self);
    // Nonvirtual base UpdateView inputs/effect, text_field_input_type.cc:830-842.
    fn SuggestedValueIsEmpty(&self) -> bool;
    fn NeedsToUpdateViewValue(&self) -> bool;
    fn SetInnerEditorValue(&self, value: &[u16]);
}

pub trait SearchKeyboardEventProjection {
    fn Key(&self, event: &EventRef) -> String;
}

pub struct SearchInputType {
    pub base_text: BaseTextInputType,
    input_host: TextControlInputHostRef,
    element: Rc<dyn SearchInputElement>,
    shadow: Rc<dyn SearchInputShadowOwner>,
    search_event_timer: Rc<dyn SearchEventTimer>,
}

impl SearchInputType {
    // Source: search_input_type.h:64-69.
    pub fn AllowFrom(input_type: &InputType) -> bool {
        input_type.r#type() == Type::kSearch
    }

    // Source: search_input_type.cc:52-57; h:39-61. The embedding binds the
    // document task runner's timer to this owner. Rc/Drop replaces Trace's
    // retained timer/base ownership at search_input_type.cc:168-171.
    pub fn new(
        mut text_field: TextFieldInputType,
        regexp_factory: Rc<dyn ScriptRegexpFactory>,
        input_host: TextControlInputHostRef,
        element: Rc<dyn SearchInputElement>,
        shadow: Rc<dyn SearchInputShadowOwner>,
        search_event_timer: Rc<dyn SearchEventTimer>,
    ) -> Self {
        text_field.input_type = Type::kSearch;
        Self {
            base_text: BaseTextInputType::new(text_field, regexp_factory),
            input_host,
            element,
            shadow,
            search_event_timer,
        }
    }

    // Source: search_input_type.cc:59-61.
    // TODO: CountUsageIfVisible(WebFeature::kInputTypeSearch) instrumentation.
    pub fn CountUsage(&self) {}

    // Source: search_input_type.cc:63-69.
    pub fn AutoAppearance(&self) -> SearchAppearance {
        SearchAppearance::SearchField
    }
    pub fn NeedsContainer(&self) -> bool {
        true
    }

    // Source: search_input_type.cc:71-81.
    pub fn CreateShadowSubtree(&self) {
        self.shadow.TextFieldCreateShadowSubtree();
        let container = self
            .shadow
            .ContainerElement()
            .expect("search field container");
        let view_port = self
            .shadow
            .UserAgentShadowRoot()
            .GetElementById(TextControlShadowId::EditingViewPort)
            .expect("search editing viewport");
        let cancel = self.shadow.CreateSearchFieldCancelButtonElement();
        container.InsertBefore(cancel, view_port.NextSibling());
    }

    // Source: search_input_type.cc:83-97. Input/change/search dispatch can
    // reenter, so the event is borrowed only for its scalar key/handled state.
    pub fn HandleKeydownEvent(
        &self,
        event: &EventRef,
        keyboard: &dyn SearchKeyboardEventProjection,
        interaction: &dyn TextFieldInteraction,
    ) {
        if self.base_text.text_field.value.IsDisabledOrReadOnly() {
            self.base_text
                .text_field
                .HandleKeydownEvent(event, interaction);
            return;
        }
        if keyboard.Key(event) == "Escape"
            && !self
                .base_text
                .text_field
                .value
                .InnerEditorValue()
                .is_empty()
        {
            self.input_host.SetValueForUser(&[]);
            self.input_host.OnSearch();
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
            return;
        }
        self.base_text
            .text_field
            .HandleKeydownEvent(event, interaction);
    }

    // Source: search_input_type.cc:99-118. Length is WTF::String's UTF-16
    // length. Empty edits post OnSearch instead of firing it synchronously.
    pub fn StartSearchEventTimer(&self) {
        debug_assert!(self.element.HasLayoutObject());
        let length = self.base_text.text_field.value.InnerEditorValue().len();
        if length == 0 {
            self.search_event_timer.Stop();
            self.element.PostOnSearchTask();
            return;
        }
        let step = length.min(4) - 1;
        self.search_event_timer
            .StartOneShot(Duration::from_millis(500 - 100 * step as u64));
    }

    // Source: search_input_type.cc:120-131.
    pub fn DispatchSearchEvent(&self) {
        self.search_event_timer.Stop();
        self.element.DispatchBubblingSearchEvent();
    }
    pub fn SearchEventTimerFired(&self) {
        self.input_host.OnSearch();
    }
    pub fn SearchEventsShouldBeDispatched(&self) -> bool {
        self.element.HasIncrementalAttribute()
    }

    // Source: search_input_type.cc:133-141.
    pub fn DidSetValueByUserEdit(&self) {
        self.UpdateCancelButtonVisibility();
        if self.SearchEventsShouldBeDispatched() {
            self.StartSearchEventTimer();
        }
        self.base_text.text_field.DidSetValueByUserEdit();
    }

    // Source: search_input_type.cc:143-146;
    // text_field_input_type.cc:830-842. BaseTextInputType inherits this
    // nonvirtual base call. Do not call HTMLInputElement::UpdateView here:
    // that would redispatch into this override and recurse.
    pub fn UpdateView(&self) {
        if self.element.SuggestedValueIsEmpty() && self.element.NeedsToUpdateViewValue() {
            self.element
                .SetInnerEditorValue(&self.base_text.text_field.value.VisibleValue());
            self.base_text
                .text_field
                .effects
                .UpdatePlaceholderVisibility();
        }
        self.UpdateCancelButtonVisibility();
    }

    // Source: search_input_type.cc:148-162. EnsureShadowSubtree can create
    // the button; acquire it before reading the current element value.
    pub fn UpdateCancelButtonVisibility(&self) {
        let Some(button) = self
            .shadow
            .EnsureShadowSubtree()
            .GetElementById(TextControlShadowId::SearchClearButton)
        else {
            return;
        };
        if self.base_text.text_field.value.Value().is_empty() {
            button.SetInlineOpacityNumber(0.0);
            button.SetInlinePointerEventsNone();
        } else {
            button.RemoveInlineStyleProperty(SearchCancelInlineProperty::Opacity);
            button.RemoveInlineStyleProperty(SearchCancelInlineProperty::PointerEvents);
        }
    }

    // Source: search_input_type.cc:164-166.
    pub fn SupportsInputModeAttribute(&self) -> bool {
        true
    }
}

// HeapTaskRunnerTimer's destruction cancels a pending callback. Rc retains
// the source Trace edge; explicit cancellation also handles a host-side timer
// registration handle surviving the translated SearchInputType owner.
impl Drop for SearchInputType {
    fn drop(&mut self) {
        self.search_event_timer.Stop();
    }
}
