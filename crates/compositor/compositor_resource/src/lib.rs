//! Compositor resource identity, budgeting, pooling, and export lifecycle.
//!
//! Actual pixel storage and replay are external raster capabilities.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod client_resource_lifecycle;
pub mod frame_resource_export;
pub mod history;
pub use compositor_values::managed_memory_policy;
pub mod memory_history;
pub mod resource_pool;
pub mod resource_pool_boundary;
pub mod returned_resource;
pub mod resource_types;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);

// gpu/command_buffer/common/gpu_memory_allocation.h values shared with
// LayerTreeSettings.
pub use compositor_values::gpu_memory_allocation::{MemoryAllocation, PriorityCutoff};
