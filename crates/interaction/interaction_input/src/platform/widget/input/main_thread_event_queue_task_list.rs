// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.h:1-53
// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.cc:1-54
// Rust modules replace C++ includes, include guards, and namespaces.

use std::collections::VecDeque;

use super::main_thread_event_queue_task::{FilterResult, MainThreadEventQueueTask};

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.h:19
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnqueueResult {
    kCoalesced,
    kEnqueued,
}

// Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.h:15-17,45-48
// VecDeque and Box preserve circular_deque and unique_ptr task ownership.
// The absent Clone/Copy implementations preserve deleted source copying.
pub struct MainThreadEventQueueTaskList<Q: ?Sized> {
    queue_: VecDeque<Box<dyn MainThreadEventQueueTask<Q>>>,
}

impl<Q: ?Sized + 'static> MainThreadEventQueueTaskList<Q> {
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.h:21-25
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.cc:13-15
    // Rust field destruction drops every remaining uniquely owned task.
    pub fn new() -> Self {
        Self {
            queue_: VecDeque::new(),
        }
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.h:27-29
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.cc:17-33
    pub fn Enqueue(&mut self, mut event: Box<dyn MainThreadEventQueueTask<Q>>) -> EnqueueResult {
        for last_event in self.queue_.iter_mut().rev() {
            match last_event.FilterNewEvent(event.as_mut()) {
                FilterResult::CoalescedEvent => return EnqueueResult::kCoalesced,
                FilterResult::StopIterating => break,
                FilterResult::KeepIterating => continue,
            }
        }
        self.queue_.push_back(event);
        EnqueueResult::kEnqueued
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.h:30
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.cc:35-42
    pub fn Pop(&mut self) -> Option<Box<dyn MainThreadEventQueueTask<Q>>> {
        self.queue_.pop_front()
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.h:32-34
    pub fn front(&self) -> &dyn MainThreadEventQueueTask<Q> {
        self.queue_
            .front()
            .expect("front requires a nonempty task queue")
            .as_ref()
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.h:35-37
    pub fn at(&self, pos: usize) -> &dyn MainThreadEventQueueTask<Q> {
        self.queue_[pos].as_ref()
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.h:35-37
    // A const unique_ptr still exposes a mutable pointee in the source. Rust
    // makes that borrow explicit for the queue's touch-unblock pass.
    pub fn at_mut(&mut self, pos: usize) -> &mut dyn MainThreadEventQueueTask<Q> {
        self.queue_[pos].as_mut()
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.h:39
    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.cc:44-52
    pub fn remove(&mut self, pos: usize) -> Option<Box<dyn MainThreadEventQueueTask<Q>>> {
        if self.queue_.is_empty() {
            return None;
        }
        // The source accesses queue_.at(pos) before erasing; an invalid
        // position in a nonempty queue must not silently become a null task.
        Some(
            self.queue_
                .remove(pos)
                .expect("task queue position out of range"),
        )
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.h:41
    pub fn empty(&self) -> bool {
        self.queue_.is_empty()
    }

    // Source: third_party/blink/renderer/platform/widget/input/main_thread_event_queue_task_list.h:43
    pub fn size(&self) -> usize {
        self.queue_.len()
    }
}

impl<Q: ?Sized + 'static> Default for MainThreadEventQueueTaskList<Q> {
    fn default() -> Self {
        Self::new()
    }
}
