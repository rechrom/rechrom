// Copyright 2018 The Chromium Authors. BSD-style license.
// Source: base/allocator/partition_allocator/src/partition_alloc/oom_callback.h:12-20
pub type PartitionAllocOomCallback = unsafe extern "C-unwind" fn();
// Source: base/allocator/partition_allocator/src/partition_alloc/oom_callback.cc:11-13
static mut g_oom_callback: Option<PartitionAllocOomCallback> = None;
// Source: base/allocator/partition_allocator/src/partition_alloc/oom_callback.cc:15-18
/// Safety: native registration is exclusive, before any concurrent OOM callback
/// read, and the callback is allocation-safe with the original static lifetime.
pub unsafe fn SetPartitionAllocOomCallback(callback: Option<PartitionAllocOomCallback>) {
    crate::PA_DCHECK!(@source b"base/allocator/partition_allocator/src/partition_alloc/oom_callback.cc\0".as_ptr().cast(),16; @condition b"!g_oom_callback\0".as_ptr().cast(); g_oom_callback.is_none());
    unsafe {
        g_oom_callback = callback;
    }
}
pub mod internal {
    // Source: base/allocator/partition_allocator/src/partition_alloc/oom_callback.cc:21-25
    /// Safety: no callback registration races this native unchecked callback read.
    pub unsafe fn RunPartitionAllocOomCallback() {
        if let Some(callback) = unsafe { super::g_oom_callback } {
            unsafe {
                callback();
            }
        }
    }
}
