// Source: base/memory/discardable_memory.h:12-18
// Genuine forward declarations; the ProcessMemoryDump retains its dump objects.
unsafe extern "C" {
    pub type MemoryAllocatorDump;
    pub type ProcessMemoryDump;
}
// Source: base/memory/discardable_memory.h:44-81
// Box<dyn DiscardableMemory> owns the actual polymorphic allocation. Send is
// the existing SkDiscardableMemory CPU cache handoff requirement, not Sync.
// The implementation must provide the native unlocked-destruction protocol.
pub trait DiscardableMemory: Send {
    fn Lock(&mut self) -> bool;
    fn Unlock(&mut self);
    fn data(&self) -> *mut u8;
    fn DiscardForTesting(&mut self);
    unsafe fn CreateMemoryAllocatorDump(
        &self,
        name: *const core::ffi::c_char,
        pmd: *mut ProcessMemoryDump,
    ) -> *mut MemoryAllocatorDump;
}
// Source: base/memory/discardable_memory.cc:9-11
// Native default ordinary constructor/destructor add no state or algorithm.
// Only real implementations can construct the abstract interface above.
