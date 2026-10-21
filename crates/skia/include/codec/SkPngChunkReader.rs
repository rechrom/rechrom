// Copyright 2015 Google Inc. BSD-style license in LICENSE.
use crate::include::core::SkRefCnt::SkRefCntObject;
use core::ffi::{c_char, c_void};
// Source: include/codec/SkPngChunkReader.h:21-44.
// Native abstract SkRefCnt-derived interface: concrete owner supplies its sole actual parent.
// The decoder may invoke this repeatedly, and on a different externally synchronized thread.
// No blanket Send/Sync is imposed on an arbitrary implementation.
pub trait SkPngChunkReader: SkRefCntObject {
    /// Safety: tag and data retain the native decoder's live buffers for the call;
    /// data is readable for length bytes, with external synchronization of the complete owner.
    unsafe fn readChunk(&mut self, tag: *const c_char, data: *const c_void, length: usize) -> bool;
}
