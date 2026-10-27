//! Raster task selection, dependency graphs, and completion accounting.
//!
//! Pixel playback and storage remain injected services owned by `raster`.

#![allow(non_camel_case_types)]

pub mod completion_stats;
pub mod completion_task;
pub mod raster_capabilities;
pub mod raster_task;
pub mod synchronous_task_graph_runner;
pub mod task;
pub mod task_graph_runner;
pub mod task_graph_work_queue;
pub mod tile_task;
pub mod tile_task_graph;
pub mod worker_dispatch;

#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskCategory {
    TASK_CATEGORY_NONCONCURRENT_FOREGROUND = 0,
    TASK_CATEGORY_FOREGROUND = 1,
    TASK_CATEGORY_BACKGROUND = 2,
    TASK_CATEGORY_BACKGROUND_WITH_NORMAL_THREAD_PRIORITY = 3,
}

pub const LAST_TASK_CATEGORY: TaskCategory =
    TaskCategory::TASK_CATEGORY_BACKGROUND_WITH_NORMAL_THREAD_PRIORITY;
