/*
 * Copyright (C) 2006, 2008, 2009 Apple Inc. All rights reserved.
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
// Source: third_party/blink/renderer/core/dom/events/event_listener.h:1-98

use std::rc::Rc;

use super::event::EventRef;

// Source: event_listener.h:31-32,52-53,66-69
// The runtime owns the execution context and its current world. Listener
// invocation receives that existing owner without borrowing a document tree.
pub trait ExecutionContext {
    // Source: execution_context.h:150; event_target.cc:598-620
    // These facts are consumed by the target's registration checks; their
    // values come from the same execution context passed to listener Invoke.
    fn IsContextDestroyed(&self) -> bool;
    fn IsWindow(&self) -> bool;
    fn IsInFencedFrameTree(&self) -> bool;
    fn DeprecateUnloadOptOutEnabled(&self) -> bool;
    fn UnloadFeatureEnabled(&self) -> bool;
    fn IsParsingDocument(&self) -> bool {
        false
    }
}
pub type EventListenerRef = Rc<dyn EventListener>;

// Source: event_listener.h:34-50,87-94
// A trait object supplies the virtual callback. Rc retains the GC member's
// identity; listener objects themselves have no Clone/Copy implementation.
pub trait EventListener: std::any::Any {
    /// Opaque binding identity of this source listener's traced native cell.
    /// Native callbacks without script storage have no binding cell.
    fn ScriptBindingIdentity(&self) -> Option<u64> {
        None
    }
    // Source: event_listener.h:52-53
    // EventRef retains the same event owner while permitting synchronous
    // listener reentry without a held mutable event borrow.
    fn Invoke(&self, context: Option<&dyn ExecutionContext>, event: &EventRef);

    // Source: event_listener.h:55-56
    fn IsEventHandler(&self) -> bool {
        false
    }

    // Source: event_listener.h:58-60
    fn IsEventHandlerForContentAttribute(&self) -> bool {
        false
    }

    // Source: event_listener.h:62-64
    fn ScriptBody(&self) -> &str {
        ""
    }

    // Source: event_listener.h:66-69
    fn BelongsToTheCurrentWorld(&self, _context: Option<&dyn ExecutionContext>) -> bool {
        false
    }

    // Source: event_listener.h:71-77
    // Implementations must retain the source's symmetric match relation.
    fn Matches(&self, other: &dyn EventListener) -> bool;

    // Source: event_listener.h:79
    // Rc field ownership replaces tracing and default destruction.

    // Source: event_listener.h:81
    fn GetHumanReadableName(&self) -> &'static str {
        "EventListener"
    }

    // Source: event_listener.h:83-85
    fn IsJSBasedEventListener(&self) -> bool {
        false
    }
    fn IsNativeEventListener(&self) -> bool {
        false
    }

    // A data address expresses C++ `this == &other` for native listeners.
    // Both callbacks remain alive during comparison; the address is never
    // dereferenced and does not become a separate callback identity owner.
    fn Identity(&self) -> *const () {
        self as *const Self as *const ()
    }
}
