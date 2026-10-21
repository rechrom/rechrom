use crate::base::memory::discardable_memory::{
    DiscardableMemory, MemoryAllocatorDump, ProcessMemoryDump,
};
// Source: skia/ext/SkDiscardableMemory_chrome.h:24-43
pub struct SkDiscardableMemoryChrome {
    discardable_: Box<dyn DiscardableMemory>,
}
// Source: skia/ext/SkDiscardableMemory_chrome.cc:15-15
// The default destructor releases its one real owned polymorphic allocation.
impl crate::include::private::chromium::SkDiscardableMemory::DiscardableMemory
    for SkDiscardableMemoryChrome
{
    // Source: skia/ext/SkDiscardableMemory_chrome.cc:17-19
    fn lock(&mut self) -> bool {
        self.discardable_.Lock()
    }
    // Source: skia/ext/SkDiscardableMemory_chrome.cc:21-23
    fn data(&mut self) -> *mut u8 {
        self.discardable_.data()
    }
    // Source: skia/ext/SkDiscardableMemory_chrome.cc:25-27
    fn unlock(&mut self) {
        self.discardable_.Unlock();
    }
}
impl SkDiscardableMemoryChrome {
    // Source: skia/ext/SkDiscardableMemory_chrome.cc:29-31
    fn new(memory: Box<dyn DiscardableMemory>) -> Self {
        Self {
            discardable_: memory,
        }
    }
    // Source: skia/ext/SkDiscardableMemory_chrome.cc:33-38
    /// Safety: name is a valid native C string; pmd and its owned dump remain
    /// live through the operation and returned non-owning pointer use.
    pub unsafe fn CreateMemoryAllocatorDump(
        &self,
        name: *const core::ffi::c_char,
        pmd: *mut ProcessMemoryDump,
    ) -> *mut MemoryAllocatorDump {
        unsafe { self.discardable_.CreateMemoryAllocatorDump(name, pmd) }
    }
}
// Source: skia/ext/SkDiscardableMemory_chrome.cc:40-47
/// Safety: the embedder has registered and retains its actual allocator;
/// registration and allocation calls satisfy the original synchronization lease.
#[cfg(all(sk_native_ndebug, not(DCHECK_ALWAYS_ON)))]
pub unsafe fn Create(
    bytes: usize,
) -> Option<Box<dyn crate::include::private::chromium::SkDiscardableMemory::SkDiscardableMemory>> {
    use crate::base::memory::discardable_memory_allocator as allocator;
    let instance = unsafe { allocator::GetInstance().unwrap_unchecked() };
    let discardable =
        unsafe { allocator::AllocateLockedDiscardableMemoryWithRetryOrDie(instance, bytes, || {}) };
    Some(Box::new(SkDiscardableMemoryChrome::new(discardable)))
}
