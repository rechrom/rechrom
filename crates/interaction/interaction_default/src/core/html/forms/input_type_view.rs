/*
 * Copyright (C) 2010 Google Inc. All rights reserved.
 * Copyright (C) 2011 Apple Inc. All rights reserved.
 * Copyright (C) 2012 Samsung Electronics. All rights reserved.
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
// Source: third_party/blink/renderer/core/html/forms/input_type_view.h:65-77,82-85,92-100,108,112,118,177-178
// Source: third_party/blink/renderer/core/html/forms/input_type_view.cc:46-50,57-65,73-77,110-112,122,192-197,254-257

use super::input_type::{CheckableActivationState, InputElementRef};
use interaction_dispatch::core::dom::events::event::EventRef;
use interaction_dispatch::core::dom::events::event_dispatcher::EventDispatchHandlingState;

// Source: input_type_view.h:65-72; input_type_view.cc:254-257.
// Retained Rc holds exactly the source's checked_radio_button Member owner.
pub struct ClickHandlingState<E = InputElementRef> {
    pub checked: bool,
    pub indeterminate: bool,
    pub checked_radio_button: Option<E>,
}
impl<E> Default for ClickHandlingState<E> {
    fn default() -> Self {
        Self {
            checked: false,
            indeterminate: false,
            checked_radio_button: None,
        }
    }
}
impl<E: 'static> EventDispatchHandlingState for ClickHandlingState<E> {}
impl<E: 'static> CheckableActivationState for ClickHandlingState<E> {}

// Source: input_type_view.h:77-85,177-178. Field Drop replaces Trace.
pub struct InputTypeView {
    element_: InputElementRef,
    will_be_destroyed: std::cell::Cell<bool>,
    has_created_shadow_subtree: std::cell::Cell<bool>,
    needs_update_view_in_create_shadow_subtree: std::cell::Cell<bool>,
}
impl InputTypeView {
    // input_type_view.cc:235-244. The value owner preserves WTF-16.
    pub fn SaveFormControlState(&self) -> Vec<std::rc::Rc<[u16]>> {
        Self::SaveFormControlStateWithValues(
            self.element_.ValueForFormState().Owner(),
            self.element_.DefaultValueForFormState().Owner(),
        )
    }
    pub fn SaveFormControlStateWithValues(
        current: std::rc::Rc<[u16]>,
        default: std::rc::Rc<[u16]>,
    ) -> Vec<std::rc::Rc<[u16]>> {
        if current == default {
            Vec::new()
        } else {
            vec![current]
        }
    }
    pub fn RestoreFormControlState(&self, state: &[std::rc::Rc<[u16]>]) {
        self.element_.RestoreFormStateValue(&state[0]);
    }
    pub fn new(element: InputElementRef) -> Self {
        Self {
            element_: element,
            will_be_destroyed: std::cell::Cell::new(false),
            has_created_shadow_subtree: std::cell::Cell::new(false),
            needs_update_view_in_create_shadow_subtree: std::cell::Cell::new(false),
        }
    }
    pub fn HasCreatedShadowSubtree(&self) -> bool {
        self.has_created_shadow_subtree.get()
    }
    /// InputTypeView::CreateShadowSubtreeIfNeeded, cc144-169. These retained
    /// source flags, not the current DOM child count, control initialization.
    pub fn CreateShadowSubtreeIfNeeded(
        &self,
        source: &dyn InputTypeViewShadowOperations,
        is_type_changing: bool,
    ) {
        if self.has_created_shadow_subtree.get() || !source.NeedsShadowSubtree() {
            return;
        }
        source.EnsureUserAgentShadowRoot();
        self.has_created_shadow_subtree.set(true);
        source.CreateShadowSubtree();
        if !is_type_changing {
            if self.needs_update_view_in_create_shadow_subtree.get() {
                source.UpdateView();
            }
            if source.HasSuggestedValueOrPlaceholder() {
                source.UpdatePlaceholderVisibilityAndPseudo();
            }
        }
        self.needs_update_view_in_create_shadow_subtree.set(false);
    }
    // Source: input_type_view.cc:42-44,128. Current translated families inherit
    // the empty popup close virtual; popup-owning subclasses must override it.
    pub fn ClosePopupView(&self) {}
    // Source: input_type_view.h:131-134,180-181; cc:145-171.
    pub fn set_needs_update_view_in_create_shadow_subtree(&self, needs: bool) {
        self.needs_update_view_in_create_shadow_subtree.set(needs);
    }
    pub fn NeedsUpdateViewInCreateShadowSubtree(&self) -> bool {
        self.needs_update_view_in_create_shadow_subtree.get()
    }
    pub fn WillBeDestroyed(&self) {
        self.will_be_destroyed.set(true);
    }
    pub(super) fn IsBeingDestroyed(&self) -> bool {
        self.will_be_destroyed.get()
    }
    // Source: input_type_view.cc:118-120.
    pub fn HasCustomFocusLogic(&self) -> bool {
        true
    }
    // Source: input_type_view.cc:199. The inherited hidden-input view has
    // no view state to update; this is Chromium's actual base virtual body.
    pub fn UpdateView(&self) {}
    pub fn GetElement(&self) -> &InputElementRef {
        &self.element_
    }
    // Source: input_type_view.cc:57-65,122.
    pub fn HandleClickEvent(&self, _event: &EventRef) {}
    pub fn HandleMouseDownEvent(&self, _event: &EventRef) {}
    pub fn HandleKeydownEvent(&self, _event: &EventRef) {}
    pub fn HandleKeypressEvent(&self, _event: &EventRef) {}
    pub fn HandleKeyupEvent(&self, _event: &EventRef) {}
    pub fn HandleBlurEvent(&self) {}
    // Source: input_type_view.cc:67-71. These are the actual inherited virtual
    // defaults; concrete families override them through InputTypeOwner.
    pub fn HandleBeforeTextInsertedEvent(&self, _event: &EventRef) {}
    pub fn HandleDOMActivateEvent(&self, _event: &EventRef) {}
    pub fn ForwardEvent(&self, _event: &EventRef) {}
    // Source: input_type_view.cc:79-85.
    pub fn AccessKeyAction(
        &self,
        scope: interaction_dispatch::core::dom::events::simulated_click_options::SimulatedClickCreationScope,
    ) {
        self.element_.FocusForAccessKey();
        self.element_.DispatchAccessKeySimulatedClick(scope);
    }
    // Source: input_type_view.cc:94-98.
    pub fn ShouldSubmitImplicitly(&self, event: &EventRef) -> bool {
        let event = event.borrow();
        event.BaseEvent().r#type() == "keypress"
            && event
                .AsKeyboardEvent()
                .is_some_and(|keyboard| keyboard.charCode() == '\r' as i32)
    }
    // Source: input_type_view.cc:100-102.
    pub fn FormForSubmission(&self) -> Option<super::html_input_element::HTMLInputFormRef> {
        self.element_.FormForSubmission()
    }
    // Source: input_type_view.cc:73-77. No Event borrow crosses simulated
    // dispatch: a listener may synchronously reenter the same event owner.
    pub fn DispatchSimulatedClickIfActive(&self, event: &EventRef) {
        if self.element_.IsActive() {
            self.element_.DispatchSimulatedClick(event);
        }
        event.borrow_mut().BaseEventMut().SetDefaultHandled();
    }
    // Source: input_type_view.cc:110-112.
    pub fn ComputedTextDirectionIsRtl(&self) -> bool {
        self.element_.IsRightToLeft()
    }
    // Source: input_type_view.cc:192-197.
    pub fn LegacyPreActivationBehavior(&self) -> Option<ClickHandlingState> {
        None
    }
    pub fn RunInputActivationBehavior(&self, _event: &EventRef, _state: &ClickHandlingState) {}
}

/// Individual original virtual/source primitives for the shadow lifecycle.
pub trait InputTypeViewShadowOperations {
    fn NeedsShadowSubtree(&self) -> bool;
    fn EnsureUserAgentShadowRoot(&self);
    fn CreateShadowSubtree(&self);
    fn UpdateView(&self);
    fn HasSuggestedValueOrPlaceholder(&self) -> bool;
    fn UpdatePlaceholderVisibilityAndPseudo(&self);
}
