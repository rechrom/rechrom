//! Window presentation executor policy. Page execution belongs to page_runtime;
//! window_runtime only schedules Display from already produced Page frames.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DisplayExecutor {
    #[default]
    DedicatedThread,
    CompositorThread,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Config {
    pub display_executor: DisplayExecutor,
    pub capture_frames: bool,
}
