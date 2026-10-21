use crate::src::sksl::SkSLMemoryPool::MemoryPool;
use core::{cell::Cell, pin::Pin, ptr};
// Selected process allocator adaptation for native unsized global new/delete.
// malloc/free are paired; zero-size global new must still return storage. Failure
// aborts across this Rust boundary instead of permitting a C++ exception unwind.
unsafe extern "C" {
    fn malloc(size: usize) -> *mut core::ffi::c_void;
    fn free(ptr: *mut core::ffi::c_void);
}
// Source: src/sksl/SkSLPool.h:55-55
// Adapted native owning unique_ptr state; raw TLS never owns the MemoryPool.
pub struct Pool {
    fMemPool: Option<Pin<Box<MemoryPool>>>,
}
// Source: src/sksl/SkSLPool.h:64-70
pub struct Poolable;
impl Poolable {
    /// Safety: memory is initialized/destroyed by its concrete IR owner and is
    /// freed under the same allocation pool attachment, before the pool drops.
    pub unsafe fn operator_new(size: usize) -> *mut u8 {
        Pool::AllocMemory(size)
    }
    /// Safety: ptr follows AllocMemory's exact current-thread allocator protocol.
    pub unsafe fn operator_delete(ptr: *mut u8) {
        Pool::FreeMemory(ptr)
    }
}
// Source: src/sksl/SkSLPool.h:78-82
impl<'a> AutoAttachPoolToThread<'a> {
    /// Safety: no pool is already attached. Pool stays on this thread for the
    /// guard, and callbacks cannot detach/attach another pool before guard drop.
    pub unsafe fn new(p: Option<&'a Pool>) -> Self {
        let value = Self { fPool: p };
        if let Some(p) = value.fPool {
            p.attachToThread();
        }
        value
    }
}
// Source: src/sksl/SkSLPool.h:83-87
impl Drop for AutoAttachPoolToThread<'_> {
    fn drop(&mut self) {
        if let Some(p) = self.fPool {
            unsafe {
                p.detachFromThread();
            }
        }
    }
}
// Source: src/sksl/SkSLPool.h:90-90
// Adapted borrowed scope guard; lifetime enforces pool-live-before-detach order.
pub struct AutoAttachPoolToThread<'a> {
    fPool: Option<&'a Pool>,
}
// Source: src/sksl/SkSLPool.cpp:16-16
thread_local! { static sMemPool:Cell<*mut MemoryPool>=const {Cell::new(ptr::null_mut())}; }
// Source: src/sksl/SkSLPool.cpp:18-20
fn get_thread_local_memory_pool() -> *mut MemoryPool {
    sMemPool.with(Cell::get)
}
// Source: src/sksl/SkSLPool.cpp:22-24
fn set_thread_local_memory_pool(memPool: *mut MemoryPool) {
    sMemPool.with(|p| p.set(memPool));
}
// Source: src/sksl/SkSLPool.cpp:26-26
impl Pool {
    fn new() -> Self {
        Self { fMemPool: None }
    }
}
// Source: src/sksl/SkSLPool.cpp:28-35
impl Drop for Pool {
    fn drop(&mut self) {
        if get_thread_local_memory_pool() == self.mem_ptr() {
            crate::SkDEBUGFAIL!(@source b"src/sksl/SkSLPool.cpp\0".as_ptr(),30;
                b"SkSL pool is being destroyed while it is still attached to the thread\0".as_ptr());
            set_thread_local_memory_pool(ptr::null_mut());
        }
    }
}
impl Pool {
    // Source: src/sksl/SkSLPool.cpp:37-42
    pub fn Create() -> Box<Self> {
        let mut pool = Box::new(Self::new());
        pool.fMemPool = Some(MemoryPool::Make());
        pool
    }
    // Source: src/sksl/SkSLPool.cpp:44-46
    pub fn IsAttached() -> bool {
        !get_thread_local_memory_pool().is_null()
    }
    // Source: src/sksl/SkSLPool.cpp:48-52
    /// Safety: keep pool and all its pointers live on this thread until detach;
    /// no other pool may attach or this pool drop while allocations are live.
    pub unsafe fn attachToThread(&self) {
        crate::SkASSERT!(@source b"src/sksl/SkSLPool.cpp\0".as_ptr(),50; @condition b"get_thread_local_memory_pool() == nullptr\0".as_ptr(); get_thread_local_memory_pool().is_null());
        set_thread_local_memory_pool(self.mem_ptr());
    }
    // Source: src/sksl/SkSLPool.cpp:54-58
    /// Safety: all allocation/free operations complete before detaching; later
    /// freeing requires reattaching this same pool first, exactly as native.
    pub unsafe fn detachFromThread(&self) {
        crate::SkASSERT!(@source b"src/sksl/SkSLPool.cpp\0".as_ptr(),56; @condition b"get_thread_local_memory_pool() == fMemPool.get()\0".as_ptr(); get_thread_local_memory_pool()==self.mem_ptr());
        set_thread_local_memory_pool(ptr::null_mut());
    }
    // Source: src/sksl/SkSLPool.cpp:60-73
    /// Safety: allocation operations on one attached pool are serialized, and
    /// its owner remains pinned/live through all concrete object destruction.
    pub unsafe fn AllocMemory(size: usize) -> *mut u8 {
        let memPool = get_thread_local_memory_pool();
        if !memPool.is_null() {
            let p = Pin::new_unchecked(&*memPool).allocate(size);
            return p;
        }
        let p = malloc(size.max(1)).cast::<u8>();
        if p.is_null() {
            std::process::abort();
        }
        p
    }
    // Source: src/sksl/SkSLPool.cpp:75-87
    /// Safety: ptr was allocated under the same current pool attachment, or by
    /// the no-pool fallback if currently unattached; concrete destructor already ran.
    pub unsafe fn FreeMemory(ptr: *mut u8) {
        let memPool = get_thread_local_memory_pool();
        if !memPool.is_null() {
            (*memPool).release(ptr);
            return;
        }
        free(ptr.cast());
    }
    // Uncounted nullable unique_ptr::get language projection.
    fn mem_ptr(&self) -> *mut MemoryPool {
        self.fMemPool.as_ref().map_or(ptr::null_mut(), |p| {
            p.as_ref().get_ref() as *const MemoryPool as *mut MemoryPool
        })
    }
}
