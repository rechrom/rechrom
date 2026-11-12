// Copyright 2014 The Chromium Authors
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/event_dispatch_forbidden_scope.h:1-78
// Source: third_party/blink/renderer/core/dom/events/event_dispatch_forbidden_scope.cc:1-13

use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

// Source: event_dispatch_forbidden_scope.cc:9-11
// The embedding identifies the document main sequence. Keeping this counter
// thread-local preserves worker exclusion without binding an engine to an OS
// thread; guards cannot move between sequences.
thread_local! { static COUNT: Cell<u32> = const { Cell::new(0) }; }

// Source: event_dispatch_forbidden_scope.h:18-40,59-66
pub struct EventDispatchForbiddenScope {
    main_sequence: bool,
    sequence_bound: PhantomData<Rc<()>>,
}

impl EventDispatchForbiddenScope {
    // Source: event_dispatch_forbidden_scope.h:22-28,63-66
    pub fn new(is_main_sequence: bool) -> Self {
        debug_assert!(is_main_sequence);
        if cfg!(debug_assertions) && is_main_sequence {
            COUNT.with(|count| count.set(count.get() + 1));
        }
        Self {
            main_sequence: is_main_sequence,
            sequence_bound: PhantomData,
        }
    }
    // Source: event_dispatch_forbidden_scope.h:36-40
    pub fn IsEventDispatchForbidden(is_main_sequence: bool) -> bool {
        cfg!(debug_assertions) && is_main_sequence && COUNT.with(|count| count.get() != 0)
    }
}

impl Drop for EventDispatchForbiddenScope {
    // Source: event_dispatch_forbidden_scope.h:30-34
    fn drop(&mut self) {
        debug_assert!(self.main_sequence);
        if cfg!(debug_assertions) && self.main_sequence {
            COUNT.with(|count| {
                debug_assert_ne!(count.get(), 0);
                count.set(count.get() - 1);
            });
        }
    }
}

// Source: event_dispatch_forbidden_scope.h:42-51,68-71
pub struct AllowUserAgentEvents {
    previous: u32,
    sequence_bound: PhantomData<Rc<()>>,
}
impl AllowUserAgentEvents {
    // Source: event_dispatch_forbidden_scope.h:46
    pub fn new(is_main_sequence: bool) -> Self {
        debug_assert!(is_main_sequence);
        let previous = if cfg!(debug_assertions) {
            COUNT.with(|count| count.replace(0))
        } else {
            0
        };
        Self {
            previous,
            sequence_bound: PhantomData,
        }
    }
}
impl Drop for AllowUserAgentEvents {
    // Source: event_dispatch_forbidden_scope.h:48-50
    fn drop(&mut self) {
        if cfg!(debug_assertions) {
            COUNT.with(|count| {
                debug_assert_eq!(count.get(), 0);
                count.set(self.previous);
            });
        }
    }
}
