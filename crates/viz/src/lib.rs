#![allow(non_snake_case, non_camel_case_types, dead_code, non_upper_case_globals)]

//! Chromium Viz surface management, aggregation and display scheduling core.

pub mod adapter;
pub mod common;
pub mod service;

pub use service::display::display_scheduler::{
    BeginFrameId, DisplayScheduler, FrameDeadlines, FrameTiming, PendingSwapParams, ScheduledFrame,
};
