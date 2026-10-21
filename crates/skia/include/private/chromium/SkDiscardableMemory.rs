// Copyright 2013 Google Inc. BSD-style license.
// Source: include/private/chromium/SkDiscardableMemory.h:18-18
// Native outer class name refers to the same sole virtual allocation interface.
pub use DiscardableMemory as SkDiscardableMemory;
// Source: include/private/chromium/SkDiscardableMemory.h:30-30
// The embedder supplies this actual abstract allocation capability. No allocator implementation is substituted.
pub trait Factory: Send + Sync {
    // Source: include/private/chromium/SkDiscardableMemory.h:32-32
    fn create(&self, bytes: usize) -> Option<Box<dyn SkDiscardableMemory>>;
}
// Dynamic Box ownership substitutes the C++ noncopyable virtual destructor contract.
// Drop must execute while unlocked. data() is usable only while the allocation is locked.
pub trait DiscardableMemory: Send {
    // Source: include/private/chromium/SkDiscardableMemory.h:50-50
    fn lock(&mut self) -> bool;
    // Source: include/private/chromium/SkDiscardableMemory.h:56-56
    fn data(&mut self) -> *mut u8;
    // Source: include/private/chromium/SkDiscardableMemory.h:62-62
    fn unlock(&mut self);
}

// Source: include/private/chromium/SkDiscardableMemory.h:24-24
// Actual Chromium factory TU, not the separate native none/pool configuration.
#[cfg(all(sk_native_ndebug, not(DCHECK_ALWAYS_ON)))]
pub use crate::skia::ext::SkDiscardableMemory_chrome::Create;
