/*
 * Copyright (C) 2010, 2011 Nokia Corporation and/or its subsidiary(-ies)
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
// Source: third_party/blink/renderer/core/html/html_details_element.h:34,43,54-58,65-81
// Source: third_party/blink/renderer/core/html/html_details_element.cc:58-70,156-239,251-294

use crate::core::dom::element::AttributeModificationReason;
use interaction_dispatch::core::dom::events::event::EventTargetRef;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone)]
pub struct PendingToggleEvent {
    pub old_state: &'static str,
    pub new_state: &'static str,
    pub source: Option<EventTargetRef>,
    pub composed: bool,
}
pub trait DetailsTaskHandle {
    fn Cancel(&self);
}
pub trait DetailsEnvironment {
    fn Target(&self) -> EventTargetRef;
    fn MainSummary(&self) -> EventTargetRef;
    fn HasParentNode(&self) -> bool;
    fn NameAttribute(&self) -> String;
    fn SetOpenAttribute(&self, present: bool);
    fn DetailsInTreeOrder(&self) -> Box<dyn Iterator<Item = Rc<HTMLDetailsElement>> + '_>;
    fn PostToggleTask(&self, task: Box<dyn FnOnce()>) -> Box<dyn DetailsTaskHandle>;
    fn DispatchToggle(&self, event: PendingToggleEvent);
    fn SetToggleDuringParsing(&self, value: bool);
    fn SetContentVisibilityHidden(&self, hidden: bool);
    fn SetContentDisplayLockIsDetailsSlotElement(&self);
    fn PseudoOpenChanged(&self);
}

// Source: html_details_element.h:79-83. The slot owners stay in DOM; retained
// pending event/task and is_open mirror the source interaction-owned state.
pub struct HTMLDetailsElement {
    environment: Rc<dyn DetailsEnvironment>,
    is_open_: Cell<bool>,
    pending_event_task_: RefCell<Option<Box<dyn DetailsTaskHandle>>>,
    pending_toggle_event_: RefCell<Option<PendingToggleEvent>>,
}
impl HTMLDetailsElement {
    pub fn new(environment: Rc<dyn DetailsEnvironment>) -> Self {
        Self::FromParsedState(environment, false)
    }
    /// Attaches the translated owner to an already-parsed canonical Element.
    /// This is its existing open-attribute state, not another parsing pass or a
    /// synthetic toggle. Subsequent mutations call ParseAttribute normally.
    pub fn FromParsedState(environment: Rc<dyn DetailsEnvironment>, is_open: bool) -> Self {
        Self {
            environment,
            is_open_: Cell::new(is_open),
            pending_event_task_: RefCell::new(None),
            pending_toggle_event_: RefCell::new(None),
        }
    }
    pub fn Target(&self) -> EventTargetRef {
        self.environment.Target()
    }
    pub fn MainSummary(&self) -> EventTargetRef {
        self.environment.MainSummary()
    }
    pub fn IsOpen(&self) -> bool {
        self.is_open_.get()
    }
    pub fn GetName(&self) -> String {
        self.environment.NameAttribute()
    }
    // Source: html_details_element.cc:58-70.
    pub fn DispatchPendingEvent(&self, reason: AttributeModificationReason) {
        let event = self
            .pending_toggle_event_
            .borrow_mut()
            .take()
            .expect("pending toggle event");
        if reason == AttributeModificationReason::kByParser {
            self.environment.SetToggleDuringParsing(true);
        }
        self.environment.DispatchToggle(event);
        if reason == AttributeModificationReason::kByParser {
            self.environment.SetToggleDuringParsing(false);
        }
    }
    // Source: html_details_element.cc:156-222. Attribute storage and slot
    // mutation are synchronous DOM effects at these same source points.
    pub fn ParseAttribute(
        self: &Rc<Self>,
        name: &str,
        new_value_present: bool,
        reason: AttributeModificationReason,
    ) {
        assert!(
            reason != AttributeModificationReason::kByParser || !self.environment.HasParentNode()
        );
        if name != "open" {
            // Generic HTMLElement parsing already ran on the canonical DOM
            // owner before it published this subclass attribute change.
            return;
        }
        let old_value = self.is_open_.replace(new_value_present);
        if self.is_open_.get() == old_value {
            return;
        }
        let mut old_state = if self.is_open_.get() {
            "closed"
        } else {
            "open"
        };
        let new_state = if self.is_open_.get() {
            "open"
        } else {
            "closed"
        };
        if let Some(pending) = self.pending_toggle_event_.borrow().as_ref() {
            old_state = pending.old_state;
        }
        *self.pending_toggle_event_.borrow_mut() = Some(PendingToggleEvent {
            old_state,
            new_state,
            source: None,
            composed: false,
        });
        let owner = self.clone();
        let task = self
            .environment
            .PostToggleTask(Box::new(move || owner.DispatchPendingEvent(reason)));
        let old_task = self.pending_event_task_.borrow_mut().replace(task);
        if let Some(old_task) = old_task {
            old_task.Cancel();
        }
        if self.is_open_.get() {
            self.environment.SetContentVisibilityHidden(false);
            assert_ne!(
                reason,
                AttributeModificationReason::kBySynchronizationOfLazyAttribute
            );
            if !self.GetName().is_empty() && reason == AttributeModificationReason::kDirectly {
                // Source explicitly snapshots group owners before attr mutation.
                let group = self.OtherElementsInNameGroup();
                for other in group {
                    assert!(!Rc::ptr_eq(&other.Target(), &self.Target()));
                    // TODO: cc:208-210 UseCounter.
                    other.environment.SetOpenAttribute(false);
                }
            }
        } else {
            self.environment.SetContentVisibilityHidden(true);
            self.environment.SetContentDisplayLockIsDetailsSlotElement();
        }
    }
    // Source: html_details_element.cc:224-239.
    pub fn AttributeChanged(
        &self,
        name: &str,
        old_present: bool,
        new_present: bool,
        _reason: AttributeModificationReason,
    ) {
        if name == "name" {
            // TODO: cc:227-229 UseCounter.
            self.MaybeCloseForExclusivity();
        }
        if name == "open" && old_present != new_present {
            self.environment.PseudoOpenChanged();
        }
        // HTMLElement::AttributeChanged is the canonical DOM mutation stage,
        // completed before this subclass notification (no second base pass).
    }
    // Source: html_details_element.cc:241-249. DOM completes HTMLElement's
    // insertion stage before invoking this retained subclass notification.
    pub fn InsertedInto(&self) {
        self.MaybeCloseForExclusivity();
    }
    // Source: html_details_element.cc:251-269.
    pub fn MaybeCloseForExclusivity(&self) {
        if self.GetName().is_empty() || !self.is_open_.get() {
            return;
        }
        let group = self.OtherElementsInNameGroup();
        for other in group {
            assert!(!Rc::ptr_eq(&other.Target(), &self.Target()));
            if other.is_open_.get() {
                // TODO: cc:263-264 UseCounter.
                self.ToggleOpen();
                break;
            }
        }
    }
    // Source: html_details_element.cc:271-294.
    pub fn ToggleOpen(&self) {
        self.environment.SetOpenAttribute(!self.is_open_.get());
    }
    // Source: html_details_element.cc:330-343. Preserve the pending task and
    // original old/new states; only the event's source/composed state changes.
    pub fn SetPendingToggleEventSource(&self, source: &EventTargetRef, composed: bool) {
        if let Some(event) = self.pending_toggle_event_.borrow_mut().as_mut() {
            event.source = Some(source.clone());
            event.composed = composed;
        }
    }
    pub fn OtherElementsInNameGroup(&self) -> Vec<Rc<HTMLDetailsElement>> {
        let name = self.GetName();
        if name.is_empty() {
            return Vec::new();
        }
        self.environment
            .DetailsInTreeOrder()
            .filter(|details| {
                !Rc::ptr_eq(&details.Target(), &self.Target()) && details.GetName() == name
            })
            .collect()
    }
    pub fn IsInteractiveContent(&self) -> bool {
        true
    }
}
impl Drop for HTMLDetailsElement {
    fn drop(&mut self) {
        if let Some(task) = self.pending_event_task_.get_mut().take() {
            task.Cancel();
        }
    }
}
