// Source: base/allocator/dispatcher/notification_data.h:22-119
use super::{memory_tagging::MTEMode, subsystem::AllocationSubsystem};
use core::ffi::{c_char, c_void};
pub struct AllocationNotificationData {
    address_: *mut c_void,
    size_: usize,
    type_name_: *const c_char,
    #[cfg(pa_has_memory_tagging)]
    mte_reporting_mode_: MTEMode,
    allocation_subsystem_: AllocationSubsystem,
}
impl AllocationNotificationData {
    pub const fn new(
        address: *mut c_void,
        size: usize,
        type_name: *const c_char,
        allocation_subsystem: AllocationSubsystem,
    ) -> Self {
        Self {
            address_: address,
            size_: size,
            type_name_: type_name,
            #[cfg(pa_has_memory_tagging)]
            mte_reporting_mode_: MTEMode::kUndefined,
            allocation_subsystem_: allocation_subsystem,
        }
    }
    pub const fn address(&self) -> *mut c_void {
        self.address_
    }
    pub const fn size(&self) -> usize {
        self.size_
    }
    pub const fn type_name(&self) -> *const c_char {
        self.type_name_
    }
    pub const fn allocation_subsystem(&self) -> AllocationSubsystem {
        self.allocation_subsystem_
    }
    #[cfg(pa_has_memory_tagging)]
    pub const fn SetMteReportingMode(&mut self, mode: MTEMode) -> &mut Self {
        self.mte_reporting_mode_ = mode;
        self
    }
    pub const fn mte_reporting_mode(&self) -> MTEMode {
        #[cfg(pa_has_memory_tagging)]
        {
            self.mte_reporting_mode_
        }
        #[cfg(not(pa_has_memory_tagging))]
        {
            MTEMode::kUndefined
        }
    }
}
pub struct FreeNotificationData {
    address_: *mut c_void,
    #[cfg(pa_has_memory_tagging)]
    mte_reporting_mode_: MTEMode,
    allocation_subsystem_: AllocationSubsystem,
}
impl FreeNotificationData {
    pub const fn new(address: *mut c_void, allocation_subsystem: AllocationSubsystem) -> Self {
        Self {
            address_: address,
            #[cfg(pa_has_memory_tagging)]
            mte_reporting_mode_: MTEMode::kUndefined,
            allocation_subsystem_: allocation_subsystem,
        }
    }
    pub const fn address(&self) -> *mut c_void {
        self.address_
    }
    pub const fn allocation_subsystem(&self) -> AllocationSubsystem {
        self.allocation_subsystem_
    }
    #[cfg(pa_has_memory_tagging)]
    pub const fn SetMteReportingMode(&mut self, mode: MTEMode) -> &mut Self {
        self.mte_reporting_mode_ = mode;
        self
    }
    pub const fn mte_reporting_mode(&self) -> MTEMode {
        #[cfg(pa_has_memory_tagging)]
        {
            self.mte_reporting_mode_
        }
        #[cfg(not(pa_has_memory_tagging))]
        {
            MTEMode::kUndefined
        }
    }
}
