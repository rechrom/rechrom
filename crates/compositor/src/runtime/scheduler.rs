//! Production construction of the canonical scheduler. This module creates no
//! clock, task runner, thread or metrics owner. The Page supplies independent
//! external capabilities and a fresh borrowed client for every synchronous call.
use crate::{
    adapter::scheduler_host::{
        CompositorFrameReportingController, CompositorTimingHistory, SchedulerHost,
    },
    cc::scheduler::{
        scheduler::{DefaultSchedulerStateMachineFactory, Scheduler, SchedulerClient},
        scheduler_settings::SchedulerSettings,
    },
};
use std::sync::Arc;

/// The canonical factory preserves the Classic/Headless/Webview/Slim source
/// branch. No scheduler/client pointer is retained by an embedding wrapper.
pub fn CreateScheduler(
    client: &mut dyn SchedulerClient,
    settings: &SchedulerSettings,
    layer_tree_host_id: i32,
    host: Arc<dyn SchedulerHost>,
    timing_history: Arc<dyn CompositorTimingHistory>,
    reporting: Arc<dyn CompositorFrameReportingController>,
) -> Scheduler {
    Scheduler::new(
        client,
        settings,
        layer_tree_host_id,
        host,
        timing_history,
        reporting,
        &DefaultSchedulerStateMachineFactory,
    )
}

// A production, non-test type assertion for this actual constructor. Keeping
// the function pointer typed makes changes to native ownership/call-scoped
// client signatures fail normal library compilation.
const _: fn(
    &mut dyn SchedulerClient,
    &SchedulerSettings,
    i32,
    Arc<dyn SchedulerHost>,
    Arc<dyn CompositorTimingHistory>,
    Arc<dyn CompositorFrameReportingController>,
) -> Scheduler = CreateScheduler;
