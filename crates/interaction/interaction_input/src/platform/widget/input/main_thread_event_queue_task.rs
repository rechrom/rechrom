// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task.h:1-44
// Rust modules replace C++ includes, include guards, and the namespace.

use std::any::Any;

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task.h:22-32
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterResult {
    // The new event was consumed by this task and must not be queued.
    CoalescedEvent,
    // Stop filtering tasks older than this one.
    StopIterating,
    // Continue filtering the next older task.
    KeepIterating,
}

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task.h:12-20,34-40
// Q is the source MainThreadEventQueue owner supplied by the caller at
// dispatch. Its generic parameter keeps the task/list independent of the
// queue implementation while preserving the synchronous queue reference.
// Rust Drop supplies the source virtual destructor.
pub trait MainThreadEventQueueTask<Q: ?Sized>: Any + Send {
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task.h:34-37
    fn FilterNewEvent(&mut self, event: &mut dyn MainThreadEventQueueTask<Q>) -> FilterResult;

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task.h:38
    fn IsWebInputEvent(&self) -> bool;

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task.h:39
    fn Dispatch(&mut self, queue: &mut Q);

    // Rust checked projections for the source subclasses' static_cast after
    // IsWebInputEvent. They add no separate task state or dispatch decision.
    fn AsAny(&self) -> &dyn Any;
    fn AsAnyMut(&mut self) -> &mut dyn Any;
}
