use core::ffi::CStr;
// Source: include/core/SkTraceMemoryDump.h:25-31
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LevelOfDetail {
    kLight_LevelOfDetail,
    kObjectsBreakdowns_LevelOfDetail,
}
// Source: include/core/SkTraceMemoryDump.h:20-20
// Native embedder-provided polymorphic interface; ownership stays with caller.
pub trait SkTraceMemoryDump {
    // Source: include/core/SkTraceMemoryDump.h:48-51
    fn dumpNumericValue(&mut self, dumpName: &CStr, valueName: &CStr, units: &CStr, value: u64);
    // Source: include/core/SkTraceMemoryDump.h:53-55
    fn dumpStringValue(&mut self, _dumpName: &CStr, _valueName: &CStr, _value: &CStr) {}
    // Source: include/core/SkTraceMemoryDump.h:62-64
    fn setMemoryBacking(
        &mut self,
        dumpName: &CStr,
        backingType: &CStr,
        backingObjectId: Option<&CStr>,
    );
    // Source: include/core/SkTraceMemoryDump.h:69-71
    fn setDiscardableMemoryBacking(
        &mut self,
        dumpName: &CStr,
        discardableMemoryObject:&dyn crate::include::private::chromium::SkDiscardableMemory::SkDiscardableMemory,
    );
    // Source: include/core/SkTraceMemoryDump.h:78-78
    fn getRequestedDetails(&self) -> LevelOfDetail;
    // Source: include/core/SkTraceMemoryDump.h:84-84
    fn shouldDumpWrappedObjects(&self) -> bool {
        true
    }
    // Source: include/core/SkTraceMemoryDump.h:90-90
    fn dumpWrappedState(&mut self, _dumpName: &CStr, _isWrappedObject: bool) {}
    // Source: include/core/SkTraceMemoryDump.h:98-98
    fn shouldDumpUnbudgetedObjects(&self) -> bool {
        true
    }
    // Source: include/core/SkTraceMemoryDump.h:105-105
    fn dumpBudgetedState(&mut self, _dumpName: &CStr, _isBudgeted: bool) {}
    // Source: include/core/SkTraceMemoryDump.h:112-112
    fn shouldDumpSizelessObjects(&self) -> bool {
        false
    }
}
// Source: include/core/SkTraceMemoryDump.h:114-118
// Rust trait-object ownership supplies the virtual destructor; no copy/clone
// implementation is added to this native noncopyable interface.
