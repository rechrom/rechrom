use super::discardable_memory::DiscardableMemory;
use core::ptr::NonNull;
// Source: base/memory/discardable_memory_allocator.h:20-76
// Native pure virtual allocator. Registration borrows its existing allocation;
// no implementation, allocator ownership, or fallback allocation is supplied.
pub trait DiscardableMemoryAllocator {
    /// Safety: implement the original allocator's synchronization contract;
    /// returned storage is genuinely locked, or allocation failure is None.
    unsafe fn AllocateLockedDiscardableMemory(
        &self,
        size: usize,
    ) -> Option<Box<dyn DiscardableMemory>>;
    fn GetBytesAllocated(&self) -> usize;
    unsafe fn ReleaseFreeMemory(&self);
}
// Source: base/memory/discardable_memory_allocator.cc:15-15
// Trait-pointer Option represents only the native nullable, non-owning pointer.
static mut g_discardable_allocator: Option<NonNull<dyn DiscardableMemoryAllocator>> = None;
// Source: base/memory/discardable_memory_allocator.cc:20-24
/// Safety: native startup/shutdown registration excludes every concurrent
/// access; the caller retains the registered object until it clears the slot.
/// In the native debug configuration, the missing DCHECK diagnostic producer
/// remains pending; do not publish an assertion-free debug implementation.
#[cfg(all(sk_native_ndebug, not(DCHECK_ALWAYS_ON)))]
pub unsafe fn SetInstance(allocator: Option<NonNull<dyn DiscardableMemoryAllocator>>) {
    unsafe {
        g_discardable_allocator = allocator;
    }
}
// Source: base/memory/discardable_memory_allocator.cc:27-30
/// Safety: registration and allocator object remain live; the native nonnull
/// precondition is required before dispatching a virtual allocation operation.
#[cfg(all(sk_native_ndebug, not(DCHECK_ALWAYS_ON)))]
pub unsafe fn GetInstance() -> Option<NonNull<dyn DiscardableMemoryAllocator>> {
    unsafe { g_discardable_allocator }
}
// Source: base/memory/discardable_memory_allocator.cc:32-34
/// Safety: no concurrent SetInstance may race this native non-atomic read.
pub unsafe fn HasInstance() -> bool {
    unsafe { g_discardable_allocator.is_some() }
}
// Source: base/memory/discardable_memory_allocator.cc:37-60
// FnOnce preserves the actual OnceClosure's single consuming Run. The native
// ordinary method reads the global allocator, but ReleaseFreeMemory uses this.
/// Safety: retain this and the globally registered allocator through both
/// allocation attempts and the callback; implement their synchronization leases.
#[cfg(all(sk_native_ndebug, not(DCHECK_ALWAYS_ON)))]
pub unsafe fn AllocateLockedDiscardableMemoryWithRetryOrDie(
    this: NonNull<dyn DiscardableMemoryAllocator>,
    size: usize,
    on_no_memory: impl FnOnce(),
) -> Box<dyn DiscardableMemory> {
    let allocator = unsafe { GetInstance().unwrap_unchecked() };
    let memory = unsafe { allocator.as_ref().AllocateLockedDiscardableMemory(size) };
    if let Some(memory) = memory {
        return memory;
    }
    on_no_memory();
    unsafe {
        this.as_ref().ReleaseFreeMemory();
    }
    let memory = unsafe { allocator.as_ref().AllocateLockedDiscardableMemory(size) };
    if let Some(memory) = memory {
        return memory;
    }
    // Actual unresolved native dependency, not abort/default/None substitution:
    // base/process/memory.h:29-34 selects partition_alloc OOM or RawCheckFailure.
    crate::base::process::memory::TerminateBecauseOutOfMemory(size)
}
