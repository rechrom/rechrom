// Copyright 2015 Google Inc. BSD-style license.
// Source: include/private/SkThreadID.h:16-16
pub type SkThreadID = i64;
// Source: include/private/SkThreadID.h:21-21
pub const kIllegalThreadID: SkThreadID = 0;

// Source: src/core/SkThreadID.cpp:10-12
#[cfg(target_os = "windows")]
pub fn SkGetThreadID() -> SkThreadID {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentThreadId() -> u32;
    }
    unsafe { GetCurrentThreadId() as SkThreadID }
}
// Source: src/core/SkThreadID.cpp:13-15
#[cfg(not(target_os = "windows"))]
pub fn SkGetThreadID() -> SkThreadID {
    unsafe { libc::pthread_self() as SkThreadID }
}
