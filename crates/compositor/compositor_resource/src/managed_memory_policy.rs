//! Resource-budget policy chosen by the compositor host.

use crate::{MemoryAllocation, PriorityCutoff};
use compositor_tile::TileMemoryLimitPolicy;

#[derive(Clone, Copy, Debug)]
pub struct ManagedMemoryPolicy {
    pub bytes_limit_when_visible: usize,
    pub priority_cutoff_when_visible: PriorityCutoff,
    pub num_resources_limit: usize,
}

impl ManagedMemoryPolicy {
    pub const kDefaultNumResourcesLimit: usize = 10 * 1000 * 1000;

    pub fn new(bytes_limit_when_visible: usize) -> Self {
        Self {
            bytes_limit_when_visible,
            priority_cutoff_when_visible: PriorityCutoff::CUTOFF_ALLOW_EVERYTHING,
            num_resources_limit: Self::kDefaultNumResourcesLimit,
        }
    }

    pub fn FromAllocation(allocation: &MemoryAllocation) -> Self {
        Self {
            bytes_limit_when_visible: allocation.bytes_limit_when_visible as usize,
            priority_cutoff_when_visible: allocation.priority_cutoff_when_visible,
            num_resources_limit: Self::kDefaultNumResourcesLimit,
        }
    }

    pub fn WithLimits(
        bytes_limit_when_visible: usize,
        priority_cutoff_when_visible: PriorityCutoff,
        num_resources_limit: usize,
    ) -> Self {
        Self {
            bytes_limit_when_visible,
            priority_cutoff_when_visible,
            num_resources_limit,
        }
    }

    pub fn PriorityCutoffToTileMemoryLimitPolicy(
        priority_cutoff: PriorityCutoff,
    ) -> TileMemoryLimitPolicy {
        match priority_cutoff {
            PriorityCutoff::CUTOFF_ALLOW_NOTHING => TileMemoryLimitPolicy::ALLOW_NOTHING,
            PriorityCutoff::CUTOFF_ALLOW_REQUIRED_ONLY => {
                TileMemoryLimitPolicy::ALLOW_ABSOLUTE_MINIMUM
            }
            PriorityCutoff::CUTOFF_ALLOW_NICE_TO_HAVE => TileMemoryLimitPolicy::ALLOW_PREPAINT_ONLY,
            PriorityCutoff::CUTOFF_ALLOW_EVERYTHING => TileMemoryLimitPolicy::ALLOW_ANYTHING,
        }
    }
}

impl PartialEq for ManagedMemoryPolicy {
    fn eq(&self, other: &Self) -> bool {
        self.bytes_limit_when_visible == other.bytes_limit_when_visible
            && self.priority_cutoff_when_visible == other.priority_cutoff_when_visible
            && self.num_resources_limit == other.num_resources_limit
    }
}

impl Eq for ManagedMemoryPolicy {}
