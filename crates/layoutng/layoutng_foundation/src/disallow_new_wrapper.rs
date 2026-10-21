#![allow(non_snake_case)]

use crate::{Traceable, Visitor};

// Rust's traced heap allocates the wrapper, while T remains an inline value.
// The C++ forwarding constructors map to one owned constructor; TakeValue's
// rvalue-reference contract is not exposed until a caller requires it.
// cpp: foundation/blink_base/heap/disallow_new_wrapper.h:13-33
pub struct DisallowNewWrapper<T: Traceable> {
    value_: T,
}

impl<T: Traceable> DisallowNewWrapper<T> {
    pub fn new(value: T) -> Self {
        Self { value_: value }
    }

    // cpp: foundation/blink_base/heap/disallow_new_wrapper.h:34-35
    pub fn Value(&mut self) -> &mut T {
        &mut self.value_
    }
}

// cpp: foundation/blink_base/heap/disallow_new_wrapper.h:38-38
impl<T: Traceable> Traceable for DisallowNewWrapper<T> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.value_);
    }
}
