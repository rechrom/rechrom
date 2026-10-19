#![allow(non_snake_case, non_camel_case_types, dead_code)]

//! Chromium Viz surface management, aggregation and display scheduling core.

pub mod common;
pub mod native;
pub mod service;

pub use service::display::display_scheduler::{
    DisplayScheduler, FrameDeadlines, FrameTiming, ScheduledFrame,
};
