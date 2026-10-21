// Copyright 2018 Google LLC. BSD-style license.
use core::mem::{needs_drop, ManuallyDrop};
// Source: src/core/SkNoDestructor.h:84-92
// Rust needs_drop is a monomorphized language counterpart of the native
// trivial-destructor assertion, not a C++ ABI-trait claim. No Copy/Clone
// implementation is provided for the wrapper. OnceLock at each static call
// site supplies the original thread-safe function-local initialization.
impl<T> SkNoDestructor<T> {
    // Source: src/core/SkNoDestructor.h:95-97
    // Variadic construction is expressed by a constructor closure. The
    // concrete native singleton type does not contain self-relative pointers.
    pub fn from_constructor(constructor: impl FnOnce() -> T) -> Self {
        Self::new(constructor())
    }
    // Source: src/core/SkNoDestructor.h:101-101
    pub fn from_copy(value: &T) -> Self
    where
        T: Clone,
    {
        Self::new(value.clone())
    }
    // Source: src/core/SkNoDestructor.h:102-102
    pub fn new(value: T) -> Self {
        assert!(
            needs_drop::<T>(),
            "use an ordinary static for a trivially destructible type"
        );
        Self {
            storage: ManuallyDrop::new(value),
        }
    }
}
// Source: src/core/SkNoDestructor.h:107-107
// The wrapper has no Drop implementation. ManuallyDrop retains T in its inline
// storage without invoking the contained destructor when the wrapper dies.
// Source: src/core/SkNoDestructor.h:109-113
impl<T> core::ops::Deref for SkNoDestructor<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
    }
}
impl<T> core::ops::DerefMut for SkNoDestructor<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}
// Source: src/core/SkNoDestructor.h:115-116
impl<T> SkNoDestructor<T> {
    pub fn get(&self) -> &T {
        &self.storage
    }
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.storage
    }
}
// Source: src/core/SkNoDestructor.h:119-119
// ManuallyDrop has exactly T's alignment/size; no payload heap allocation.
pub struct SkNoDestructor<T> {
    storage: ManuallyDrop<T>,
}
