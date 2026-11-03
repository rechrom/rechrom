//! Native begin-frame values and external trace/transport effects. Scheduler
//! policy and Host frame management remain in their canonical translated owners.
use super::{
    engine_factory::FactoryCapabilities,
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
    CompositorTypes,
};
use std::sync::Arc;
struct FrameTrace {
    events: Arc<dyn super::services::EventSink>,
    enabled: bool,
}
impl TraceSink for FrameTrace {
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn emit(&mut self, event: TraceEvent) {
        if self.enabled {
            self.events.Emit("cc", format!("{event:?}"));
        }
    }
}
pub(crate) struct BeginFrameServices {
    capabilities: Arc<FactoryCapabilities>,
    trace: FrameTrace,
}
impl BeginFrameServices {
    pub(crate) fn new(capabilities: Arc<FactoryCapabilities>) -> Self {
        use crate::adapter::layer_tree_impl::draw_host::LayerTreeDrawServices;
        let trace = FrameTrace {
            events: capabilities.events.clone(),
            enabled: capabilities.render.TraceCategoryEnabled(),
        };
        Self {
            capabilities,
            trace,
        }
    }
}
impl HostImplEServices<CompositorTypes> for BeginFrameServices {
    fn ScaleToEnclosingRect(&self, rect: Rect, scale: f32) -> Rect {
        ScaleToEnclosingRect(rect, scale)
    }
    fn IsLowEndDevice(&self) -> bool {
        self.capabilities.picture.low_end_device
    }
    fn FrameTokenValue(&self, token: &FrameTokenGenerator) -> u32 {
        token.current()
    }
    fn BeginFrameTrace(&mut self) -> &mut dyn TraceSink {
        &mut self.trace
    }
    fn TrackersNotifyBeginImplFrame(
        &mut self,
        trackers: &mut FrameSequenceTrackers<CompositorTypes>,
        args: &BeginFrameArgs,
    ) {
        trackers.0.TrackersNotifyBeginImplFrame(args);
    }
    fn ReportingSetNeedsRasterPropertiesAnimated(
        &mut self,
        reporter: &mut ReportingController<CompositorTypes>,
        animated: bool,
    ) {
        reporter
            .0
            .ReportingSetNeedsRasterPropertiesAnimated(animated);
    }
    fn DevtoolsDidBeginFrame(&mut self, id: i32, time: TimeTicks, sequence: u64) {
        self.capabilities.events.Emit(
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
        trackers: &mut FrameSequenceTrackers<CompositorTypes>,
        current: &BeginFrameArgs,
        args: &BeginFrameArgs,
    ) {
        trackers.0.TrackersNotifyFrameEnd(current, args);
    }
    fn TraceDidNotProduceFrame(&self, ack: &BeginFrameAck, reason_name: &str) {
        if self.trace.enabled {
            self.capabilities
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
            self.capabilities
                .frame_sink
                .lock()
                .expect("native frame sink")
                .id()
        );
        self.capabilities
            .commands
            .Push(EngineTask::DidNotProduceFrame { ack: *ack, reason });
    }
    fn DropSavedEventMetricsForNoFrameUpdate(&mut self, manager: &mut EventsMetricsManager) {
        manager.0.DropSavedEventMetricsForNoFrameUpdate();
    }
    fn DidChangeBeginFrameSourcePaused(&mut self, _delegate: HostImplDelegateId, paused: bool) {
        self.capabilities
            .commands
            .Push(EngineTask::BeginFrameSourcePaused { paused });
    }
    fn SynchronousTaskGraphRunnerRunUntilIdle(
        &mut self,
        runner: &mut Arc<SynchronousTaskGraphRunner<RasterCompletion<CompositorTypes>>>,
    ) {
        runner.RunUntilIdle();
    }
    fn TraceBuildHitTestData(&self) {
        if self.trace.enabled {
            self.capabilities
                .events
                .Emit("cc", "BuildHitTestData".into());
        }
    }
}
