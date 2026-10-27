//! Facade specialization of scheduler-owned host boundaries.

pub use compositor_scheduler::scheduler_host::*;

pub type SubmitInfo =
    crate::cc::metrics::submit_info::SubmitInfo<crate::runtime::input::EventMetricsSet>;
pub type FrameTimingDetails = crate::runtime::metadata::FrameTimingDetails;

pub struct FacadeSchedulerTypes;

impl SchedulerTypes for FacadeSchedulerTypes {
    type SubmitInfo = SubmitInfo;
    type FrameTimingDetails = FrameTimingDetails;
}
