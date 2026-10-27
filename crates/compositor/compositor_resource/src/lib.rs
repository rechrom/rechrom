//! Compositor resource identity, budgeting, pooling, and export lifecycle.
//!
//! Actual pixel storage and replay are external raster capabilities.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod client_resource_lifecycle;
pub mod frame_resource_export;
pub mod history;
pub mod managed_memory_policy;
pub mod memory_history;
pub mod resource_pool;
pub mod resource_pool_boundary;
pub mod resource_types;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);

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
