// Copyright 2021 The Chromium Authors. BSD-style license.
#[cfg(pa_has_allocation_guard)]
use super::partition_alloc_base::immediate_crash::PA_IMMEDIATE_CRASH;
// Source: base/allocator/partition_allocator/src/partition_alloc/allocation_guard.cc:15-15
#[cfg(pa_has_allocation_guard)]
#[thread_local]
static mut g_disallow_allocations: bool = false;
// Source: base/allocator/partition_allocator/src/partition_alloc/allocation_guard.h:19-23
#[cfg(pa_has_allocation_guard)]
pub struct ScopedDisallowAllocations {
    _thread: core::marker::PhantomData<*mut ()>,
}
// Source: base/allocator/partition_allocator/src/partition_alloc/allocation_guard.h:26-35
#[cfg(pa_has_allocation_guard)]
pub struct ScopedAllowAllocations {
    saved_value_: bool,
    _thread: core::marker::PhantomData<*mut ()>,
}
// ZST marker enforces the actual same-thread scope; it adds no state/owner.
#[cfg(pa_has_allocation_guard)]
impl ScopedDisallowAllocations {
    // Source: base/allocator/partition_allocator/src/partition_alloc/allocation_guard.cc:18-24
    pub fn new() -> Self {
        if unsafe { g_disallow_allocations } {
            PA_IMMEDIATE_CRASH!();
        }
        unsafe {
            g_disallow_allocations = true;
        }
        Self {
            _thread: core::marker::PhantomData,
        }
    }
}
#[cfg(pa_has_allocation_guard)]
impl Drop for ScopedDisallowAllocations {
    // Source: base/allocator/partition_allocator/src/partition_alloc/allocation_guard.cc:26-28
    fn drop(&mut self) {
        unsafe {
            g_disallow_allocations = false;
        }
    }
}
#[cfg(pa_has_allocation_guard)]
impl ScopedAllowAllocations {
    // Source: base/allocator/partition_allocator/src/partition_alloc/allocation_guard.cc:30-35
    pub fn new() -> Self {
        let saved = unsafe { g_disallow_allocations };
        unsafe {
            g_disallow_allocations = false;
        }
        Self {
            saved_value_: saved,
            _thread: core::marker::PhantomData,
        }
    }
}
#[cfg(pa_has_allocation_guard)]
impl Drop for ScopedAllowAllocations {
    // Source: base/allocator/partition_allocator/src/partition_alloc/allocation_guard.cc:37-39
    fn drop(&mut self) {
        unsafe {
            g_disallow_allocations = self.saved_value_;
        }
    }
}
// Source: base/allocator/partition_allocator/src/partition_alloc/allocation_guard.h:39-40
#[cfg(not(pa_has_allocation_guard))]
pub struct ScopedDisallowAllocations;
#[cfg(not(pa_has_allocation_guard))]
pub struct ScopedAllowAllocations;
#[cfg(not(pa_has_allocation_guard))]
impl ScopedAllowAllocations {
    pub const fn new() -> Self {
        Self
    }
}
#[cfg(not(pa_has_allocation_guard))]
impl ScopedDisallowAllocations {
    pub const fn new() -> Self {
        Self
    }
}
