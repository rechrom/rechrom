//! Native CompositorFrame surface aggregation and display deadlines.
mod display_scheduler;
pub mod native;
pub use display_scheduler::{DisplayScheduler, FrameDeadlines, FrameTiming, ScheduledFrame};
pub use native::*;
