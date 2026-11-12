/*
 * Copyright (C) 2001 Peter Kelly (pmk@post.com)
 * Copyright (C) 2001 Tobias Anton (anton@stud.fbi.fh-darmstadt.de)
 * Copyright (C) 2006 Samuel Weinig (sam.weinig@gmail.com)
 * Copyright (C) 2003, 2004, 2005, 2006, 2008, 2009 Apple Inc. All rights
 * reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB. If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/registered_event_listener.h:1-123
// Source: third_party/blink/renderer/core/dom/events/registered_event_listener.cc:1-109

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::add_event_listener_options_resolved::AddEventListenerOptionsResolved;
use super::event::{Event, PhaseType};
use super::event_listener::{EventListener, EventListenerRef};

pub type RegisteredEventListenerRef = Rc<RegisteredEventListener>;

// Source: registered_event_listener.h:79-89
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OptionsForMatching {
    pub use_capture_: bool,
}

impl OptionsForMatching {
    // Source: registered_event_listener.h:83-85
    pub fn new(use_capture: bool) -> Self {
        Self {
            use_capture_: use_capture,
        }
    }
}

// Source: registered_event_listener.h:37-47,105-115
// Snapshots retain the same registration object. Cell flags let removal be
// visible immediately while a dispatch snapshot still retains this owner.
pub struct RegisteredEventListener {
    callback_: RefCell<Option<EventListenerRef>>,
    use_capture_: bool,
    passive_: bool,
    once_: bool,
    blocked_event_warning_emitted_: Cell<bool>,
    passive_forced_for_document_target_: bool,
    passive_specified_: bool,
    removed_: Cell<bool>,
    animation_trigger_: bool,
}

impl RegisteredEventListener {
    // Source: registered_event_listener.h:42
    // Source: registered_event_listener.cc:33-41
    pub fn new() -> Self {
        Self {
            callback_: RefCell::new(None),
            use_capture_: false,
            passive_: false,
            once_: false,
            blocked_event_warning_emitted_: Cell::new(false),
            passive_forced_for_document_target_: false,
            passive_specified_: false,
            removed_: Cell::new(false),
            animation_trigger_: false,
        }
    }

    // Source: registered_event_listener.h:43-44
    // Source: registered_event_listener.cc:43-55
    pub fn WithOptions(
        listener: EventListenerRef,
        options: &AddEventListenerOptionsResolved,
    ) -> Self {
        Self {
            callback_: RefCell::new(Some(listener)),
            use_capture_: options.capture(),
            passive_: options.passive(),
            once_: options.once(),
            blocked_event_warning_emitted_: Cell::new(false),
            passive_forced_for_document_target_: options.PassiveForcedForDocumentTarget(),
            passive_specified_: options.PassiveSpecified(),
            removed_: Cell::new(false),
            animation_trigger_: options.IsAnimationTrigger(),
        }
    }

    // Source: registered_event_listener.h:49
    // Source: registered_event_listener.cc:57-59
    // Rc retains callback identity and replaces GC tracing.

    // Source: registered_event_listener.h:51
    // Source: registered_event_listener.cc:61-71
    pub fn Options(&self) -> AddEventListenerOptionsResolved {
        let mut result = AddEventListenerOptionsResolved::new();
        result.setCapture(self.use_capture_);
        result.setPassive(self.passive_);
        result.SetPassiveForcedForDocumentTarget(self.passive_forced_for_document_target_);
        result.setOnce(self.once_);
        result.SetPassiveSpecified(self.passive_specified_);
        result.SetAnimationTrigger(self.animation_trigger_);
        result
    }

    // Source: registered_event_listener.h:53-55
    pub fn Callback(&self) -> Option<EventListenerRef> {
        self.callback_.borrow().clone()
    }

    // Source: registered_event_listener.h:57
    // Source: registered_event_listener.cc:73-75
    pub fn SetCallback(&self, listener: Option<EventListenerRef>) {
        *self.callback_.borrow_mut() = listener;
    }

    // Source: registered_event_listener.h:59-63
    pub fn Passive(&self) -> bool {
        self.passive_
    }
    pub fn Once(&self) -> bool {
        self.once_
    }
    pub fn Capture(&self) -> bool {
        self.use_capture_
    }

    // Source: registered_event_listener.h:65-67
    pub fn BlockedEventWarningEmitted(&self) -> bool {
        self.blocked_event_warning_emitted_.get()
    }

    // Source: registered_event_listener.h:69-73
    pub fn PassiveForcedForDocumentTarget(&self) -> bool {
        self.passive_forced_for_document_target_
    }
    pub fn PassiveSpecified(&self) -> bool {
        self.passive_specified_
    }

    // Source: registered_event_listener.h:75-77
    pub fn SetBlockedEventWarningEmitted(&self) {
        self.blocked_event_warning_emitted_.set(true);
    }

    // Source: registered_event_listener.h:91-93
    pub fn GetOptionsForMatching(&self) -> OptionsForMatching {
        OptionsForMatching::new(self.use_capture_)
    }

    // Source: registered_event_listener.h:94-95
    // Source: registered_event_listener.cc:77-83
    pub fn Matches(&self, listener: &dyn EventListener, options: &OptionsForMatching) -> bool {
        let callback = self
            .Callback()
            .expect("registered listener callback is non-null");
        callback.Matches(listener) && *options == self.GetOptionsForMatching()
    }

    // Source: registered_event_listener.h:97
    // Source: registered_event_listener.cc:85-99
    pub fn ShouldFire(&self, event: &Event) -> bool {
        if event.FireOnlyCaptureListenersAtTarget() {
            debug_assert_eq!(event.eventPhase(), PhaseType::kAtTarget);
            return self.Capture();
        }
        if event.FireOnlyNonCaptureListenersAtTarget() {
            debug_assert_eq!(event.eventPhase(), PhaseType::kAtTarget);
            return !self.Capture();
        }
        if event.eventPhase() == PhaseType::kCapturingPhase {
            return self.Capture();
        }
        if event.eventPhase() == PhaseType::kBubblingPhase {
            return !self.Capture();
        }
        true
    }

    // Source: registered_event_listener.h:99-103
    pub fn Removed(&self) -> bool {
        self.removed_.get()
    }
    pub fn SetRemoved(&self) {
        self.removed_.set(true);
    }
    pub fn IsAnimationTrigger(&self) -> bool {
        self.animation_trigger_
    }
}

impl Default for RegisteredEventListener {
    fn default() -> Self {
        Self::new()
    }
}

// Source: registered_event_listener.h:117
// Source: registered_event_listener.cc:101-107
impl PartialEq for RegisteredEventListener {
    fn eq(&self, rhs: &Self) -> bool {
        let lhs_callback = self
            .Callback()
            .expect("registered listener callback is non-null");
        let rhs_callback = rhs
            .Callback()
            .expect("registered listener callback is non-null");
        lhs_callback.Matches(rhs_callback.as_ref()) && self.Capture() == rhs.Capture()
    }
}
