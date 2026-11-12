/*
 * Copyright (C) 2011 Nokia Corporation and/or its subsidiary(-ies)
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
// Source: third_party/blink/renderer/core/html/html_summary_element.h:41-42,48-53
// Source: third_party/blink/renderer/core/html/html_summary_element.cc:161-212

use super::html_details_element::HTMLDetailsElement;
use crate::core::dom::element::UpdateBehavior;
use interaction_dispatch::core::dom::events::event::{EventRef, EventTargetRef};
use std::rc::Rc;

pub trait SummaryEnvironment {
    fn Target(&self) -> EventTargetRef;
    fn ParentDetails(&self) -> Option<Rc<HTMLDetailsElement>>;
    fn OwnerShadowHostDetails(&self) -> Option<Rc<HTMLDetailsElement>>;
    fn IsClickableControl(&self, target: Option<EventTargetRef>) -> bool;
    fn HandleKeyboardActivation(&self, event: &EventRef) -> bool;
    fn BaseDefaultEventHandler(&self, event: &EventRef);
    fn BaseSupportsFocus(&self, update_behavior: UpdateBehavior) -> bool;
    fn BaseWillRespondToMouseClickEvents(&self) -> bool;
}
pub struct HTMLSummaryElement {
    environment: Rc<dyn SummaryEnvironment>,
}
impl HTMLSummaryElement {
    pub fn new(environment: Rc<dyn SummaryEnvironment>) -> Self {
        Self { environment }
    }
    // Source: html_summary_element.cc:161-174.
    pub fn DetailsElement(&self) -> Option<Rc<HTMLDetailsElement>> {
        self.environment
            .ParentDetails()
            .or_else(|| self.environment.OwnerShadowHostDetails())
    }
    pub fn IsMainSummary(&self) -> bool {
        self.DetailsElement()
            .is_some_and(|details| Rc::ptr_eq(&details.MainSummary(), &self.environment.Target()))
    }
    // Source: html_summary_element.cc:176-186.
    pub fn SupportsFocus(&self, update_behavior: UpdateBehavior) -> bool {
        self.IsMainSummary() || self.environment.BaseSupportsFocus(update_behavior)
    }
    pub fn DefaultTabIndex(&self) -> i32 {
        if self.IsMainSummary() {
            0
        } else {
            -1
        }
    }
    // Source: html_summary_element.cc:188-204.
    pub fn DefaultEventHandler(&self, event: &EventRef) {
        if self.IsMainSummary() {
            let (dom_activate, target) = {
                let event = event.borrow();
                (
                    event.BaseEvent().r#type() == "DOMActivate",
                    event.BaseEvent().RawTarget(),
                )
            };
            if Self::HandleDOMActivate(
                true,
                dom_activate,
                || self.environment.IsClickableControl(target),
                || {
                    if let Some(details) = self.DetailsElement() {
                        details.ToggleOpen();
                    }
                },
            ) {
                event.borrow_mut().BaseEventMut().SetDefaultHandled();
                return;
            }
            if self.environment.HandleKeyboardActivation(event) {
                return;
            }
        }
        self.environment.BaseDefaultEventHandler(event);
    }
    // Source: html_summary_element.cc:188-197. Both retained and borrowed
    // native owners execute this same activation decision.
    pub fn HandleDOMActivate(
        is_main: bool,
        dom_activate: bool,
        target_is_clickable: impl FnOnce() -> bool,
        toggle_open: impl FnOnce(),
    ) -> bool {
        if is_main && dom_activate && !target_is_clickable() {
            toggle_open();
            return true;
        }
        false
    }
    // Source: html_summary_element.cc:206-212.
    pub fn HasActivationBehavior(&self) -> bool {
        true
    }
    pub fn WillRespondToMouseClickEvents(&self) -> bool {
        self.IsMainSummary() || self.environment.BaseWillRespondToMouseClickEvents()
    }
}
