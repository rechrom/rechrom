//! GPU memory allocation values carried by LayerTreeSettings
//! (gpu/command_buffer/common/gpu_memory_allocation.h). Moved from
//! compositor_resource, which re-exports them.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum PriorityCutoff {
    CUTOFF_ALLOW_NOTHING,
    CUTOFF_ALLOW_REQUIRED_ONLY,
    CUTOFF_ALLOW_NICE_TO_HAVE,
    CUTOFF_ALLOW_EVERYTHING,
}

impl PriorityCutoff {
    pub const CUTOFF_LAST: Self = Self::CUTOFF_ALLOW_EVERYTHING;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryAllocation {
    pub bytes_limit_when_visible: u64,
    pub priority_cutoff_when_visible: PriorityCutoff,
}

impl Default for MemoryAllocation {
    fn default() -> Self {
        Self {
            bytes_limit_when_visible: 0,
            priority_cutoff_when_visible: PriorityCutoff::CUTOFF_ALLOW_NOTHING,
        }
    }
}

impl MemoryAllocation {
    pub fn new(bytes_limit_when_visible: u64) -> Self {
        Self {
            bytes_limit_when_visible,
            priority_cutoff_when_visible: PriorityCutoff::CUTOFF_ALLOW_EVERYTHING,
        }
    }

    pub fn Equals(&self, other: &Self) -> bool {
        self == other
    }
}
