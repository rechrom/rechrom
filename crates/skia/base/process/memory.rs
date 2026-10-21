// Copyright 2013 The Chromium Authors. BSD-style license.
// Source: base/process/memory.h:29-30
#[cfg(pa_use_partition_alloc)]
pub use crate::base::allocator::partition_allocator::src::partition_alloc::oom::TerminateBecauseOutOfMemory;
// Source: base/process/memory.h:40-40
#[cfg(any(
    chromium_is_linux,
    chromium_is_chromeos,
    chromium_is_android,
    chromium_is_aix
))]
pub const kMaxOomScore: i32 = 1000;
// Actual ordinary declarations without admitted platform definitions remain
// pending, not Rust trait providers or fabricated implementations.
// Source: base/process/memory.h:32-34
#[cfg(not(pa_use_partition_alloc))]
pub unsafe fn TerminateBecauseOutOfMemory(_size: usize) -> ! {
    unsafe { crate::base::check::RawCheckFailure(b"Out of memory\0".as_ptr().cast()) }
}
// Source: base/process/memory.h:61-61
#[cfg(pa_is_win)]
pub mod win {
    pub use crate::base::allocator::partition_allocator::src::partition_alloc::oom::win::kOomExceptionCode;
}
