/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 *           (C) 2001 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2004, 2005, 2006, 2007, 2012 Apple Inc. All rights reserved.
 * Copyright (C) 2006 Alexey Proskuryakov (ap@webkit.org)
 *           (C) 2007, 2008 Nikolas Zimmermann <zimmermann@kde.org>
 * Copyright (C) 2011 Andreas Kling (kling@webkit.org)
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE COMPUTER, INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE COMPUTER, INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/event_listener_map.h:1-108
// Source: third_party/blink/renderer/core/dom/events/event_listener_map.cc:1-233

use std::cell::RefCell;
use std::rc::Rc;

use super::add_event_listener_options_resolved::AddEventListenerOptionsResolved;
use super::event_listener::{EventListener, EventListenerRef};
use super::registered_event_listener::{
    OptionsForMatching, RegisteredEventListener, RegisteredEventListenerRef,
};

// Source: event_listener_map.h:49-51
pub type EventListenerVector = Rc<RefCell<Vec<RegisteredEventListenerRef>>>;
pub type EventListenerVectorSnapshot = Vec<RegisteredEventListenerRef>;

// Source: event_listener_map.cc:208-218
// This is precisely the receiving target operation used by source copying.
// The target owns registration, signal handling and mutation notification.
pub trait EventListenerCopyTarget {
    fn addEventListener(
        &self,
        event_type: &str,
        listener: EventListenerRef,
        options: &AddEventListenerOptionsResolved,
    );
}

// Source: event_listener_map.h:53-59,96-104
// Preserve the small vector of event types and each live vector owner.
// Copying this map is deliberately unsupported, as in the source.
#[derive(Default)]
pub struct EventListenerMap {
    entries_: Vec<(String, EventListenerVector)>,
}

impl EventListenerMap {
    // Source: event_listener_map.h:57
    // Source: event_listener_map.cc:54
    pub fn new() -> Self {
        Self::default()
    }

    // Source: event_listener_map.h:61
    // Source: event_listener_map.cc:93-100
    pub fn Clear(&mut self) {
        for (_, listeners) in &self.entries_ {
            for listener in listeners.borrow().iter() {
                listener.SetRemoved();
            }
        }
        self.entries_.clear();
    }

    // Source: event_listener_map.h:62
    pub fn IsEmpty(&self) -> bool {
        self.entries_.is_empty()
    }

    // Source: event_listener_map.h:63
    // Source: event_listener_map.cc:56-62
    pub fn Contains(&self, event_type: &str) -> bool {
        for (kind, _) in &self.entries_ {
            if kind == event_type {
                return true;
            }
        }
        false
    }

    // Source: event_listener_map.h:64
    // Source: event_listener_map.cc:64-76
    pub fn ContainsAnyCapturing(&self) -> bool {
        self.entries_
            .iter()
            .any(|(_, listeners)| listeners.borrow().iter().any(|listener| listener.Capture()))
    }

    pub fn ContainsCapturing(&self, event_type: &str) -> bool {
        for (kind, listeners) in &self.entries_ {
            if kind == event_type {
                for listener in listeners.borrow().iter() {
                    if listener.Capture() {
                        return true;
                    }
                }
                return false;
            }
        }
        false
    }

    // Source: event_listener_map.h:65
    // Source: event_listener_map.cc:78-91
    pub fn ContainsJSBasedEventListeners(&self, event_type: &str) -> bool {
        for (kind, listeners) in &self.entries_ {
            if kind == event_type {
                for listener in listeners.borrow().iter() {
                    if listener
                        .Callback()
                        .is_some_and(|callback| callback.IsJSBasedEventListener())
                    {
                        return true;
                    }
                }
                return false;
            }
        }
        false
    }

    // Source: event_listener_map.h:67-73
    // Source: event_listener_map.cc:129-154
    pub fn Add(
        &mut self,
        event_type: &str,
        listener: EventListenerRef,
        options: &AddEventListenerOptionsResolved,
        registered_listener: &mut Option<RegisteredEventListenerRef>,
    ) -> bool {
        for (kind, listeners) in &self.entries_ {
            if kind == event_type {
                // Source: event_listener_map.cc:135-144
                // TODO: report listener_count_log2 crash key for >= 8 listeners.
                return AddListenerToVector(listeners, listener, options, registered_listener);
            }
        }
        let listeners = Rc::new(RefCell::new(Vec::new()));
        self.entries_
            .push((event_type.to_owned(), listeners.clone()));
        AddListenerToVector(&listeners, listener, options, registered_listener)
    }

    // Source: event_listener_map.h:74-80
    // Source: event_listener_map.cc:175-191
    pub fn Remove(
        &mut self,
        event_type: &str,
        listener: &dyn EventListener,
        options: &OptionsForMatching,
        registered_listener: &mut Option<RegisteredEventListenerRef>,
    ) -> bool {
        for i in 0..self.entries_.len() {
            if self.entries_[i].0 == event_type {
                let was_removed = RemoveListenerFromVector(
                    &self.entries_[i].1,
                    listener,
                    options,
                    registered_listener,
                );
                if self.entries_[i].1.borrow().is_empty() {
                    self.entries_.remove(i);
                }
                return was_removed;
            }
        }
        false
    }

    // Source: event_listener_map.h:81-82
    // Source: event_listener_map.cc:193-206
    // Cloning the Rc retains the same live vector for const/mutable access.
    pub fn Find(&self, event_type: &str) -> Option<EventListenerVector> {
        for (kind, listeners) in &self.entries_ {
            if kind == event_type {
                return Some(listeners.clone());
            }
        }
        None
    }

    // Source: event_listener_map.h:83
    // Source: event_listener_map.cc:102-110
    pub fn EventTypes(&self) -> Vec<String> {
        let mut types = Vec::with_capacity(self.entries_.len());
        for (kind, _) in &self.entries_ {
            types.push(kind.clone());
        }
        types
    }

    // Source: event_listener_map.h:85-90
    pub fn ForAllEventListenerTypes(&self, mut callback: impl FnMut(&str, usize)) {
        for (kind, listeners) in &self.entries_ {
            callback(kind, listeners.borrow().len());
        }
    }

    // Source: event_listener_map.h:92
    // Source: event_listener_map.cc:221-227
    pub fn CopyEventListenersNotCreatedFromMarkupToTarget(
        &self,
        target: &dyn EventListenerCopyTarget,
    ) {
        for (event_type, listeners) in &self.entries_ {
            CopyListenersNotCreatedFromMarkupToTarget(event_type, listeners, target);
        }
    }

    // Source: event_listener_map.h:94; event_listener_map.cc:229-231
    // Rc owns vectors and registered listeners; Drop replaces GC tracing.
}

// Source: event_listener_map.cc:112-127
fn AddListenerToVector(
    listeners: &EventListenerVector,
    listener: EventListenerRef,
    options: &AddEventListenerOptionsResolved,
    registered_listener: &mut Option<RegisteredEventListenerRef>,
) -> bool {
    let mut vector = listeners.borrow_mut();
    for item in vector.iter() {
        if item.Matches(
            listener.as_ref(),
            &OptionsForMatching::new(options.capture()),
        ) {
            return false;
        }
    }
    let registered = Rc::new(RegisteredEventListener::WithOptions(listener, options));
    *registered_listener = Some(registered.clone());
    vector.push(registered);
    true
}

// Source: event_listener_map.cc:156-173
fn RemoveListenerFromVector(
    listeners: &EventListenerVector,
    listener: &dyn EventListener,
    options: &OptionsForMatching,
    registered_listener: &mut Option<RegisteredEventListenerRef>,
) -> bool {
    let mut removed = 0;
    listeners.borrow_mut().retain(|current| {
        if current.Matches(listener, options) {
            current.SetRemoved();
            *registered_listener = Some(current.clone());
            removed += 1;
            false
        } else {
            true
        }
    });
    debug_assert!(removed <= 1, "listener vector contains duplicate listeners");
    removed > 0
}

// Source: event_listener_map.cc:208-219
fn CopyListenersNotCreatedFromMarkupToTarget(
    event_type: &str,
    listeners: &EventListenerVector,
    target: &dyn EventListenerCopyTarget,
) {
    let mut index = 0;
    loop {
        let listener = listeners.borrow().get(index).cloned();
        let Some(listener) = listener else { break };
        index += 1;
        let callback = listener
            .Callback()
            .expect("registered listener callback is non-null");
        if callback.IsEventHandlerForContentAttribute() {
            continue;
        }
        let options = listener.Options();
        target.addEventListener(event_type, callback, &options);
    }
}
