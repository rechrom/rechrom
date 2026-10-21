// Additional direct dependency, not one of the eight authorized original sources.
// Source: base/allocator/dispatcher/memory_tagging.h:14-38
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MTEMode {
    kUndefined,
    kDisabled,
    kSynchronous,
    kAsynchronous,
}
pub const fn ConvertToMTEMode(
    mode: crate::partition_alloc::tagging::TagViolationReportingMode,
) -> MTEMode {
    use crate::partition_alloc::tagging::TagViolationReportingMode as P;
    match mode {
        P::kUndefined => MTEMode::kUndefined,
        P::kDisabled => MTEMode::kDisabled,
        P::kSynchronous => MTEMode::kSynchronous,
        P::kAsynchronous => MTEMode::kAsynchronous,
    }
}
