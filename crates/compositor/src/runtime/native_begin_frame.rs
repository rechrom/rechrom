//! Native begin-frame values and external trace/transport effects. Scheduler
//! policy and Host frame management remain in their canonical translated owners.
use super::{
    engine_factory::NativeFactoryCapabilities,
    execution::EngineTask,
    host_metrics::{FrameSequenceTrackers, ReportingController},
    host_owners::EventsMetricsManager,
    metadata::FrameTokenGenerator,
    types::LocalSurfaceId,
};
use crate::{
    adapter::{
        begin_frame::BeginFrameArgs,
        geometry::{Rect, ScaleToEnclosingRect},
        layer_tree_host_impl::{HostImplDelegateId, HostImplEServices, HostImplFrameSinkId},
        scheduler_host::BeginFrameAck,
        time::TimeTicks,
        trace_sink::{TraceEvent, TraceSink},
    },
    cc::{
        raster::synchronous_task_graph_runner::SynchronousTaskGraphRunner,
        scheduler::scheduler::FrameSkippedReason, tiles::tile_manager::RasterCompletion,
    },
    NativeCompositorTypes,
};
use std::sync::Arc;
struct NativeFrameTrace {
    events: Arc<dyn super::native_services::NativeEventSink>,
    enabled: bool,
}
impl TraceSink for NativeFrameTrace {
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn emit(&mut self, event: TraceEvent) {
        if self.enabled {
            self.events.Emit("cc", format!("{event:?}"));
        }
    }
}
pub(crate) struct NativeBeginFrameServices {
    native: Arc<NativeFactoryCapabilities>,
    trace: NativeFrameTrace,
}
impl NativeBeginFrameServices {
    pub(crate) fn new(native: Arc<NativeFactoryCapabilities>) -> Self {
        use crate::adapter::layer_tree_impl::draw_host::LayerTreeDrawServices;
        let trace = NativeFrameTrace {
            events: native.events.clone(),
            enabled: native.render.TraceCategoryEnabled(),
        };
        Self { native, trace }
    }
}
impl HostImplEServices<NativeCompositorTypes> for NativeBeginFrameServices {
    fn ScaleToEnclosingRect(&self, rect: Rect, scale: f32) -> Rect {
        ScaleToEnclosingRect(rect, scale)
    }
    fn IsLowEndDevice(&self) -> bool {
        self.native.picture.low_end_device
    }
    fn FrameTokenValue(&self, token: &FrameTokenGenerator) -> u32 {
        token.current()
    }
    fn BeginFrameTrace(&mut self) -> &mut dyn TraceSink {
        &mut self.trace
    }
    fn TrackersNotifyBeginImplFrame(
        &mut self,
        trackers: &mut FrameSequenceTrackers<NativeCompositorTypes>,
        args: &BeginFrameArgs,
    ) {
        trackers.0.TrackersNotifyBeginImplFrame(args);
    }
    fn ReportingSetNeedsRasterPropertiesAnimated(
        &mut self,
        reporter: &mut ReportingController<NativeCompositorTypes>,
        animated: bool,
    ) {
        reporter
            .0
            .ReportingSetNeedsRasterPropertiesAnimated(animated);
    }
    fn DevtoolsDidBeginFrame(&mut self, id: i32, time: TimeTicks, sequence: u64) {
        self.native.events.Emit(
            "cc",
            format!("begin frame host={id} time={time:?} sequence={sequence}"),
        );
    }
    fn LocalSurfaceIdIsNewerThan(
        &self,
        target: &LocalSurfaceId,
        upcoming: &LocalSurfaceId,
    ) -> bool {
        target.IsNewerThan(upcoming)
    }
    fn TrackersNotifyFrameEnd(
        &mut self,
        trackers: &mut FrameSequenceTrackers<NativeCompositorTypes>,
        current: &BeginFrameArgs,
        args: &BeginFrameArgs,
    ) {
        trackers.0.TrackersNotifyFrameEnd(current, args);
    }
    fn TraceDidNotProduceFrame(&self, ack: &BeginFrameAck, reason_name: &str) {
        if self.trace.enabled {
            self.native
                .events
                .Emit("cc", format!("did not produce {ack:?}: {reason_name}"));
        }
    }
    fn FrameSinkDidNotProduceFrame(
        &mut self,
        sink: HostImplFrameSinkId,
        ack: &BeginFrameAck,
        reason: FrameSkippedReason,
    ) {
        assert_eq!(
            sink,
            self.native
                .frame_sink
                .lock()
                .expect("native frame sink")
                .id()
        );
        self.native
            .commands
            .Push(EngineTask::DidNotProduceFrame { ack: *ack, reason });
    }
    fn DropSavedEventMetricsForNoFrameUpdate(&mut self, manager: &mut EventsMetricsManager) {
        manager.0.DropSavedEventMetricsForNoFrameUpdate();
    }
    fn DidChangeBeginFrameSourcePaused(&mut self, _delegate: HostImplDelegateId, paused: bool) {
        self.native
            .commands
            .Push(EngineTask::BeginFrameSourcePaused { paused });
    }
    fn SynchronousTaskGraphRunnerRunUntilIdle(
        &mut self,
        runner: &mut Arc<SynchronousTaskGraphRunner<RasterCompletion<NativeCompositorTypes>>>,
    ) {
        runner.RunUntilIdle();
    }
    fn TraceBuildHitTestData(&self) {
        if self.trace.enabled {
            self.native.events.Emit("cc", "BuildHitTestData".into());
        }
    }
}
