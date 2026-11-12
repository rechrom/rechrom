// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/add_event_listener_options_resolved.h:1-49
// Source: third_party/blink/renderer/core/dom/events/add_event_listener_options_resolved.cc:1-32

use std::rc::Rc;

// Source: add_event_listener_options.idl:10
// EventTarget owns registration/removal of the signal's abort algorithm.
pub trait AbortAlgorithmHandle: std::any::Any {}
// Source: core/dom/abort_signal.h:103,115-122
pub trait AbortSignal {
    fn aborted(&self) -> bool;
    fn AddAlgorithm(&self, callback: Box<dyn FnOnce()>) -> Option<Rc<dyn AbortAlgorithmHandle>>;
}
pub type AbortSignalRef = Rc<dyn AbortSignal>;

// Source: event_listener_options.idl:7-9; add_event_listener_options.idl:7-11
// The optional entries retain generated dictionary presence alongside values.
#[derive(Clone)]
pub struct AddEventListenerOptions {
    capture: Option<bool>,
    passive: Option<bool>,
    once: Option<bool>,
    signal: Option<AbortSignalRef>,
}

impl Default for AddEventListenerOptions {
    fn default() -> Self {
        Self {
            capture: Some(false),
            passive: None,
            once: Some(false),
            signal: None,
        }
    }
}

impl AddEventListenerOptions {
    pub fn hasCapture(&self) -> bool {
        self.capture.is_some()
    }
    pub fn capture(&self) -> bool {
        self.capture.unwrap_or(false)
    }
    pub fn setCapture(&mut self, value: bool) {
        self.capture = Some(value);
    }
    pub fn hasPassive(&self) -> bool {
        self.passive.is_some()
    }
    pub fn passive(&self) -> bool {
        self.passive.unwrap_or(false)
    }
    pub fn setPassive(&mut self, value: bool) {
        self.passive = Some(value);
    }
    pub fn hasOnce(&self) -> bool {
        self.once.is_some()
    }
    pub fn once(&self) -> bool {
        self.once.unwrap_or(false)
    }
    pub fn setOnce(&mut self, value: bool) {
        self.once = Some(value);
    }
    pub fn hasSignal(&self) -> bool {
        self.signal.is_some()
    }
    pub fn signal(&self) -> Option<AbortSignalRef> {
        self.signal.clone()
    }
    pub fn setSignal(&mut self, value: AbortSignalRef) {
        self.signal = Some(value);
    }
}

// Source: add_event_listener_options_resolved.h:13-22,41-44
#[derive(Clone, Default)]
pub struct AddEventListenerOptionsResolved {
    options: AddEventListenerOptions,
    passive_forced_for_document_target_: bool,
    passive_specified_: bool,
    animation_trigger_: bool,
}

impl AddEventListenerOptionsResolved {
    // Source: add_event_listener_options_resolved.cc:9
    pub fn new() -> Self {
        Self::default()
    }

    // Source: add_event_listener_options_resolved.cc:11-24
    pub fn FromOptions(options: &AddEventListenerOptions) -> Self {
        let mut result = Self::new();
        if options.hasPassive() {
            result.setPassive(options.passive());
        }
        if options.hasOnce() {
            result.setOnce(options.once());
        }
        if let Some(signal) = options.signal() {
            result.setSignal(signal);
        }
        if options.hasCapture() {
            result.setCapture(options.capture());
        }
        result
    }

    // Source: add_event_listener_options_resolved.h:24-29
    pub fn SetPassiveForcedForDocumentTarget(&mut self, forced: bool) {
        self.passive_forced_for_document_target_ = forced;
    }
    pub fn PassiveForcedForDocumentTarget(&self) -> bool {
        self.passive_forced_for_document_target_
    }

    // Source: add_event_listener_options_resolved.h:31-34
    pub fn SetPassiveSpecified(&mut self, specified: bool) {
        self.passive_specified_ = specified;
    }
    pub fn PassiveSpecified(&self) -> bool {
        self.passive_specified_
    }

    // Source: add_event_listener_options_resolved.h:36-37
    pub fn SetAnimationTrigger(&mut self, value: bool) {
        self.animation_trigger_ = value;
    }
    pub fn IsAnimationTrigger(&self) -> bool {
        self.animation_trigger_
    }
}

// Source: add_event_listener_options_resolved.h:17-18
impl std::ops::Deref for AddEventListenerOptionsResolved {
    type Target = AddEventListenerOptions;
    fn deref(&self) -> &Self::Target {
        &self.options
    }
}
impl std::ops::DerefMut for AddEventListenerOptionsResolved {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.options
    }
}
// Source: add_event_listener_options_resolved.cc:26-30
// Rc retains signal identity; Rust field destruction replaces GC tracing.
