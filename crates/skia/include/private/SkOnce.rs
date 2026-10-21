// Copyright 2013 Google Inc. BSD-style license.
use core::sync::atomic::{AtomicU8, Ordering};

// Source: include/private/SkOnce.h:51-51
#[repr(u8)]
enum State {
    NotStarted,
    Claimed,
    Done,
}

// Source: include/private/SkOnce.h:22-24
// Source: include/private/SkOnce.h:52-52
pub struct SkOnce {
    fState: AtomicU8,
}
impl SkOnce {
    pub const fn new() -> Self {
        Self {
            fState: AtomicU8::new(State::NotStarted as u8),
        }
    }

    // Source: include/private/SkOnce.h:26-48
    /// Calls the native three-state protocol on the actual allocation.
    ///
    /// Safety: the allocation and atomic must be initialized and remain alive
    /// through each atomic access. Concurrent access must use this protocol;
    /// resetting, moving, or destroying a Claimed instance is forbidden. A
    /// caller may publish ownership allowing destruction at the final Done
    /// release: this implementation does not access the allocation afterward.
    /// Callback/capture destruction and all caller accesses must independently
    /// respect that boundary. If initialization unwinds, native Claimed remains
    /// Claimed; there is no invented poison, retry, or success transition.
    pub unsafe fn call_once<F: FnOnce()>(this: *const Self, init: F) {
        let state = unsafe { (*this).fState.load(Ordering::Acquire) };
        if state == State::Done as u8 {
            return;
        }
        if state == State::NotStarted as u8
            && unsafe {
                (*this)
                    .fState
                    .compare_exchange(
                        State::NotStarted as u8,
                        State::Claimed as u8,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    )
                    .is_ok()
            }
        {
            init();
            unsafe {
                (*this).fState.store(State::Done as u8, Ordering::Release);
            }
            return;
        }
        // Native potentially-blocking annotations have no runtime operation.
        // This is the original empty spin, without yield/backoff/pause changes.
        while unsafe { (*this).fState.load(Ordering::Acquire) } != State::Done as u8 {}
    }

    // Source: include/private/SkOnce.h:26-27,39-39
    /// The native forwarding argument pack is represented by a typed value or
    /// tuple; the callable destructures that tuple into its real parameters.
    /// Safety: the same raw allocation/lifecycle contract as call_once applies.
    pub unsafe fn call_once_with<F: FnOnce(A), A>(this: *const Self, init: F, args: A) {
        unsafe {
            Self::call_once(this, || init(args));
        }
    }
}
impl Default for SkOnce {
    fn default() -> Self {
        Self::new()
    }
}
