// Translation status: complete
// Copyright 2011 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/base/completion_event.h:1-89.
//
// Used for making blocking calls from one thread to another. Use only when
// absolutely certain that doing so will not lead to a deadlock.
//
// Chromium hands a raw `CompletionEvent*` from ProxyMain (which owns it in
// LayerTreeHost::commit_completion_event_) to ProxyImpl. Rust shares one
// event through `Arc` clones instead; the lifetime argument of the raw
// pointer becomes reference counting. `base::WaitableEvent` is a
// `Mutex<State> + Condvar`. ProxyMain creates the commit completion with
// `ResetPolicy::MANUAL` (cc/trees/proxy_main.cc:471), which is the only
// policy used here: once signaled the event stays signaled.
//
// Rust extension: `SignalAbandoned` releases a waiter when the signaling
// side is dropped without completing its work (a panicking impl
// FinishCommit, or an impl thread that exits with the commit still queued).
// The waiter observes it through `WasAbandoned` instead of touching a
// half-pushed main tree.

use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

#[derive(Default)]
struct State {
    signaled: bool,
    abandoned: bool,
    // Source: completion_event.h:82-85. Used to assert that Wait() and
    // Signal() are each called exactly once (DCHECK builds).
    waited: bool,
    signal_called: bool,
}

#[derive(Default)]
struct Event {
    state: Mutex<State>,
    condition: Condvar,
}

#[derive(Clone, Default)]
pub struct CompletionEvent(Arc<Event>);

impl CompletionEvent {
    // Source: completion_event.h:21-28 (MANUAL reset, NOT_SIGNALED).
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.0
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    // Source: completion_event.h:37-51.
    pub fn Wait(&self) {
        let mut state = self.lock();
        debug_assert!(!state.waited);
        state.waited = true;
        // The event has already been signaled and cannot be re-signaled.
        while !state.signaled {
            state = self
                .0
                .condition
                .wait(state)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }

    // Source: completion_event.h:53-66.
    pub fn TimedWait(&self, max_time: Duration) -> bool {
        let mut state = self.lock();
        debug_assert!(!state.waited);
        state.waited = true;
        let (state_after, _) = self
            .0
            .condition
            .wait_timeout_while(state, max_time, |state| !state.signaled)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state = state_after;
        if state.signaled {
            return true;
        }
        state.waited = false;
        false
    }

    // Source: completion_event.h:68.
    pub fn IsSignaled(&self) -> bool {
        self.lock().signaled
    }

    // Source: completion_event.h:70-76.
    pub fn Signal(&self) {
        let mut state = self.lock();
        debug_assert!(!state.signal_called);
        state.signal_called = true;
        state.signaled = true;
        self.0.condition.notify_all();
    }

    /// Rust extension (see the file comment): signals the event and records
    /// that the signaling side never completed its work.
    pub fn SignalAbandoned(&self) {
        let mut state = self.lock();
        state.abandoned = true;
        state.signal_called = true;
        state.signaled = true;
        self.0.condition.notify_all();
    }

    pub fn WasAbandoned(&self) -> bool {
        self.lock().abandoned
    }

    /// Identity of the shared event (Chromium compares the raw pointer).
    pub fn SameEvent(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
