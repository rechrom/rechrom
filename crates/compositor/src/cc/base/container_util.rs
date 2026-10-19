// Translation status: complete
// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/base/container_util.h:1-30

// Source: cc/base/container_util.h:7-25. Rust expresses the C++ container
// method requirements as traits; callers can provide the same front/back
// removal contract for source-aligned container types.
pub trait PopFrontContainer {
    type Value;
    fn PopFrontValue(&mut self) -> Self::Value;
}

pub trait PopBackContainer {
    type Value;
    fn PopBackValue(&mut self) -> Self::Value;
}

impl<T> PopFrontContainer for std::collections::VecDeque<T> {
    type Value = T;
    fn PopFrontValue(&mut self) -> T {
        self.pop_front()
            .expect("cc::PopFront requires a non-empty container")
    }
}

impl<T> PopBackContainer for std::collections::VecDeque<T> {
    type Value = T;
    fn PopBackValue(&mut self) -> T {
        self.pop_back()
            .expect("cc::PopBack requires a non-empty container")
    }
}

impl<T> PopFrontContainer for std::collections::LinkedList<T> {
    type Value = T;
    fn PopFrontValue(&mut self) -> T {
        self.pop_front()
            .expect("cc::PopFront requires a non-empty container")
    }
}

impl<T> PopBackContainer for std::collections::LinkedList<T> {
    type Value = T;
    fn PopBackValue(&mut self) -> T {
        self.pop_back()
            .expect("cc::PopBack requires a non-empty container")
    }
}

impl<T> PopBackContainer for Vec<T> {
    type Value = T;
    fn PopBackValue(&mut self) -> T {
        self.pop()
            .expect("cc::PopBack requires a non-empty container")
    }
}

// Source: cc/base/container_util.h:11-16
pub fn PopFront<Container: PopFrontContainer>(container: &mut Container) -> Container::Value {
    container.PopFrontValue()
}

// Source: cc/base/container_util.h:19-24
pub fn PopBack<Container: PopBackContainer>(container: &mut Container) -> Container::Value {
    container.PopBackValue()
}

// Source: cc/base/container_util.h:26-30. Namespace and include-guard closure
// are represented by this Rust module and one-time module inclusion.
