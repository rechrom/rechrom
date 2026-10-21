// Copyright 2013 Google Inc. BSD-style license.
use super::SkStream::{SkStream, SkWStream};
use core::{marker::PhantomPinned, pin::Pin};
// Source: src/core/SkStreamPriv.h:30-30
pub use super::SkStream::SkStreamPriv::CopyStreamToData;
// Source: src/core/SkStreamPriv.h:36-36
pub use super::SkStream::SkStreamPriv::Copy;
// Source: src/core/SkStreamPriv.h:39-46
pub struct DebugfStream {
    pub(super) fBytesWritten: usize,
    _pin: PhantomPinned,
}
impl DebugfStream {
    // C++ implicit constructor applies the native in-class zero initializer.
    pub fn new() -> Self {
        Self {
            fBytesWritten: 0,
            _pin: PhantomPinned,
        }
    }
}
// Source: src/core/SkStreamPriv.h:51-54
pub fn WriteU16BE(mut s: Pin<&mut dyn SkWStream>, mut value: u16) -> bool {
    value = value.to_be();
    unsafe {
        s.as_mut().write(
            (&value as *const u16).cast(),
            core::mem::size_of_val(&value),
        )
    }
}
// Source: src/core/SkStreamPriv.h:56-59
pub fn WriteU32BE(mut s: Pin<&mut dyn SkWStream>, mut value: u32) -> bool {
    value = value.to_be();
    unsafe {
        s.as_mut().write(
            (&value as *const u32).cast(),
            core::mem::size_of_val(&value),
        )
    }
}
// Source: src/core/SkStreamPriv.h:61-64
pub fn WriteS32BE(mut s: Pin<&mut dyn SkWStream>, mut value: i32) -> bool {
    value = value.to_be();
    unsafe {
        s.as_mut().write(
            (&value as *const i32).cast(),
            core::mem::size_of_val(&value),
        )
    }
}
// Source: src/core/SkStreamPriv.h:66-72
pub fn ReadU16BE(mut s: Pin<&mut dyn SkStream>, value: &mut u16) -> bool {
    if !s.as_mut().readU16(value) {
        return false;
    }
    *value = u16::from_be(*value);
    true
}
// Source: src/core/SkStreamPriv.h:74-80
pub fn ReadU32BE(mut s: Pin<&mut dyn SkStream>, value: &mut u32) -> bool {
    if !s.as_mut().readU32(value) {
        return false;
    }
    *value = u32::from_be(*value);
    true
}
// Source: src/core/SkStreamPriv.h:82-88
pub fn ReadS32BE(mut s: Pin<&mut dyn SkStream>, value: &mut i32) -> bool {
    if !s.as_mut().readS32(value) {
        return false;
    }
    *value = i32::from_be(*value);
    true
}
// Source: src/core/SkStreamPriv.h:95-95
pub use super::SkStream::SkStreamPriv::RemainingLengthIsBelow;
