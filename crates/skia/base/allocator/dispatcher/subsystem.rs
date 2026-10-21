// Source: base/allocator/dispatcher/subsystem.h:13-24
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AllocationSubsystem {
    kPartitionAllocator = 1,
    kAllocatorShim = 2,
    kManualForTesting = 3,
}
