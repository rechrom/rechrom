pub use compositor_scheduler::scheduler::{
    DefaultSchedulerStateMachineFactory, FrameSkippedReason, SchedulerClient,
};

pub type Scheduler =
    compositor_scheduler::scheduler::Scheduler<crate::adapter::scheduler_host::FacadeSchedulerTypes>;
