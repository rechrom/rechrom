// Translation status: complete
// Copyright 2013 The Chromium Authors
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/resources/memory_history.h:1-52; cc/resources/memory_history.cc:1-24.
use crate::adapter::history::{RingBuffer, RingBufferIterator};
// Source: cc/resources/memory_history.h:27-36.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    pub total_budget_in_bytes: usize,
    pub total_bytes_used: i64,
    pub had_enough_memory: bool,
}
// Source: cc/resources/memory_history.h:40.
pub type RingBufferType = RingBuffer<Entry, 80>;
// Source: cc/resources/memory_history.h:18-47.
// C++ deletes copying, so this owner intentionally has no Clone implementation.
pub struct MemoryHistory {
    ring_buffer_: RingBufferType,
}
impl MemoryHistory {
    // Source: cc/resources/memory_history.h:20; cc/resources/memory_history.cc:13-16.
    pub fn Create() -> Box<Self> {
        Box::new(Self {
            ring_buffer_: RingBuffer::default(),
        })
    }
    // Source: cc/resources/memory_history.h:25.
    pub fn HistorySize(&self) -> usize {
        self.ring_buffer_.BufferSize()
    }
    // Source: cc/resources/memory_history.cc:20-22.
    pub fn SaveEntry(&mut self, entry: &Entry) {
        self.ring_buffer_.SaveToBuffer(entry.clone());
    }
    // Source: cc/resources/memory_history.h:41-42.
    pub fn Begin(&self) -> RingBufferIterator<'_, Entry, 80> {
        self.ring_buffer_.Begin()
    }
    pub fn End(&self) -> RingBufferIterator<'_, Entry, 80> {
        self.ring_buffer_.End()
    }
}
