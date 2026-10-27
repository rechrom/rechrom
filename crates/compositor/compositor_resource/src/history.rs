// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//! Fixed inline base::RingBuffer dependency, including its source cursor/index
//! semantics (End is the newest element, not the usual Rust sentinel).
// Source dependency: base/containers/ring_buffer.h:27-153.
pub struct RingBuffer<T: Default + Clone, const N: usize> {
    buffer: [T; N],
    current_index: usize,
}
impl<T: Default + Clone, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        assert!(N > 0);
        Self {
            buffer: std::array::from_fn(|_| T::default()),
            current_index: 0,
        }
    }
}
impl<T: Default + Clone, const N: usize> RingBuffer<T, N> {
    pub const fn BufferSize(&self) -> usize {
        N
    }
    pub fn CurrentIndex(&self) -> usize {
        self.current_index
    }
    fn BufferIndex(&self, n: usize) -> usize {
        self.current_index.wrapping_add(n) % N
    }
    fn IsFilledIndexByBufferIndex(&self, index: usize) -> bool {
        index < self.current_index
    }
    pub fn IsFilledIndex(&self, n: usize) -> bool {
        self.IsFilledIndexByBufferIndex(self.BufferIndex(n))
    }
    pub fn ReadBuffer(&self, n: usize) -> &T {
        let index = self.BufferIndex(n);
        assert!(self.IsFilledIndexByBufferIndex(index));
        &self.buffer[index]
    }
    pub fn MutableReadBuffer(&mut self, n: usize) -> &mut T {
        let index = self.BufferIndex(n);
        assert!(self.IsFilledIndexByBufferIndex(index));
        &mut self.buffer[index]
    }
    pub fn SaveToBuffer(&mut self, value: T) {
        let index = self.BufferIndex(0);
        self.buffer[index] = value;
        self.current_index = self.current_index.wrapping_add(1);
    }
    pub fn Clear(&mut self) {
        self.current_index = 0;
    }
    pub fn Begin(&self) -> RingBufferIterator<'_, T, N> {
        RingBufferIterator {
            buffer: self,
            index: if self.current_index < N {
                N - self.current_index
            } else {
                0
            },
            out_of_range: false,
        }
    }
    pub fn End(&self) -> RingBufferIterator<'_, T, N> {
        RingBufferIterator {
            buffer: self,
            index: N - 1,
            out_of_range: false,
        }
    }
}
pub struct RingBufferIterator<'a, T: Default + Clone, const N: usize> {
    buffer: &'a RingBuffer<T, N>,
    index: usize,
    out_of_range: bool,
}
impl<'a, T: Default + Clone, const N: usize> RingBufferIterator<'a, T, N> {
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn IsValid(&self) -> bool {
        !self.out_of_range && self.buffer.IsFilledIndex(self.index)
    }
    pub fn value(&self) -> &'a T {
        self.buffer.ReadBuffer(self.index)
    }
    pub fn Increment(&mut self) {
        self.index = self.index.wrapping_add(1);
        if self.index == N {
            self.out_of_range = true;
        }
    }
    pub fn Decrement(&mut self) {
        if self.index == 0 {
            self.out_of_range = true;
        }
        self.index = self.index.wrapping_sub(1);
    }
}
impl<'a, T: Default + Clone, const N: usize> Iterator for RingBufferIterator<'a, T, N> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.IsValid() {
            return None;
        }
        let value = self.value();
        self.Increment();
        Some(value)
    }
}
