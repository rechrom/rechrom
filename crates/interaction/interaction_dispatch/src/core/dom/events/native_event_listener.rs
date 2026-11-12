// Copyright 2018 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/native_event_listener.h:1-42

use super::event::EventRef;
use super::event_listener::{EventListener, ExecutionContext};

// Source: native_event_listener.h:14-18,27-30
// The blanket EventListener implementation supplies the native base overrides.
pub trait NativeEventListener {
    fn Invoke(&self, context: Option<&dyn ExecutionContext>, event: &EventRef);
    fn IsImageEventListener(&self) -> bool {
        false
    }
    // Source: event_listener.h:55-69,81
    // Native subclasses can still override inherited virtual operations.
    fn IsEventHandler(&self) -> bool {
        false
    }
    fn IsEventHandlerForContentAttribute(&self) -> bool {
        false
    }
    fn ScriptBody(&self) -> &str {
        ""
    }
    fn BelongsToTheCurrentWorld(&self, _context: Option<&dyn ExecutionContext>) -> bool {
        false
    }
    fn GetHumanReadableName(&self) -> &'static str {
        "EventListener"
    }
}

impl<T: NativeEventListener + 'static> EventListener for T {
    fn Invoke(&self, context: Option<&dyn ExecutionContext>, event: &EventRef) {
        NativeEventListener::Invoke(self, context, event);
    }
    fn IsEventHandler(&self) -> bool {
        NativeEventListener::IsEventHandler(self)
    }
    fn IsEventHandlerForContentAttribute(&self) -> bool {
        NativeEventListener::IsEventHandlerForContentAttribute(self)
    }
    fn ScriptBody(&self) -> &str {
        NativeEventListener::ScriptBody(self)
    }
    fn BelongsToTheCurrentWorld(&self, context: Option<&dyn ExecutionContext>) -> bool {
        NativeEventListener::BelongsToTheCurrentWorld(self, context)
    }
    fn GetHumanReadableName(&self) -> &'static str {
        NativeEventListener::GetHumanReadableName(self)
    }

    // Source: native_event_listener.h:20-23
    fn Matches(&self, other: &dyn EventListener) -> bool {
        std::ptr::eq(self as *const T as *const (), other.Identity())
    }

    // Source: native_event_listener.h:25-26
    fn IsNativeEventListener(&self) -> bool {
        true
    }
}

// Source: native_event_listener.h:33-38
pub fn AllowFrom(event_listener: &dyn EventListener) -> bool {
    event_listener.IsNativeEventListener()
}
