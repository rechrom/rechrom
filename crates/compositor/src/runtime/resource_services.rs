//! Native resource/feedback forwarding. The source Host remains the sole owner
//! of tile notification, damage, completion, budget and release decisions.
use super::{
    engine_factory::FactoryCapabilities,
    host_frame_sink::HostFrameSinkCallbacks,
    host_metrics::{CustomTrackerResults, EventLatencyData, FrameSequenceTrackers},
    host_owners::{
        AverageLagTrackingManager, ClientResourceProvider, EventsMetricsManager, LayerContext,
    },
    metadata::{FrameTimingDetails, PresentationFeedback},
    resource_events::{ResourceEffect, ResourceEffects},
    resources::RasterReturnedResource,
    types::LocalSurfaceId,
};
use crate::{
    adapter::{
        begin_frame::BeginFrameArgs,
        host_picture::{HostPictureRedrawServices, HostPictureReleaseServices},
        layer_tree_host_impl::*,
        settings::LayerTreeSettingsFeatures,
        task_runner::TaskLocation,
        time::TimeDelta,
    },
    cc::{
        resources::resource_pool::ResourcePool,
        tiles::{picture_layer_tiling::PictureLayerTilingState, tile::Tile},
        trees::presentation_time_callback_buffer::PendingCallbacks,
    },
    CompositorTypes,
};
use std::sync::Arc;
pub(crate) struct ResourceServices {
    capabilities: Arc<FactoryCapabilities>,
    returns: HostFrameSinkCallbacks,
}
impl ResourceServices {
    pub(crate) fn new(capabilities: Arc<FactoryCapabilities>) -> Self {
        let returns = HostFrameSinkCallbacks::new(
            capabilities.frame_sink.clone(),
            capabilities.lifecycle.clone(),
            capabilities.runners.clone(),
            capabilities.commands.clone(),
        );
        Self {
            capabilities,
            returns,
        }
    }
}
impl HostImplTaskRunnerServices for ResourceServices {
    fn MainThreadTaskRunner(&self, p: HostImplTaskRunnerProviderId) -> HostImplTaskRunnerId {
        self.returns.MainThreadTaskRunner(p)
    }
    fn ImplThreadTaskRunner(&self, p: HostImplTaskRunnerProviderId) -> HostImplTaskRunnerId {
        self.returns.ImplThreadTaskRunner(p)
    }
    fn HasImplThread(&self, p: HostImplTaskRunnerProviderId) -> bool {
        self.returns.HasImplThread(p)
    }
}
impl HostImplResourceReturnServices<CompositorTypes> for ResourceServices {
    fn FrameSinkContextProvider(
        &self,
        id: HostImplFrameSinkId,
    ) -> Option<HostImplContextProviderId> {
        self.returns.FrameSinkContextProvider(id)
    }
    fn PostDelayedCompositorTask(
        &mut self,
        runner: HostImplTaskRunnerId,
        location: TaskLocation,
        task: HostImplPostedCompositorTask<FrameTimingDetails>,
        delay: TimeDelta,
    ) {
        self.returns
            .PostDelayedCompositorTask(runner, location, task, delay)
    }
    fn ReceiveReturnsFromParent(
        &mut self,
        p: &mut ClientResourceProvider<CompositorTypes>,
        r: Vec<RasterReturnedResource>,
        pool: Option<&mut ResourcePool<CompositorTypes>>,
        client: &mut dyn HostUIResourceReleaseClient<CompositorTypes>,
    ) {
        self.returns.ReceiveReturnsFromParent(p, r, pool, client)
    }
    fn ResourcePoolReduceResourceUsage(&mut self, pool: &mut ResourcePool<CompositorTypes>) {
        self.returns.ResourcePoolReduceResourceUsage(pool)
    }
    fn ContextHasSupport(&self, id: HostImplContextProviderId) -> bool {
        self.returns.ContextHasSupport(id)
    }
    fn ContextSupportFlushPendingWork(&mut self, id: HostImplContextProviderId) {
        self.returns.ContextSupportFlushPendingWork(id)
    }
}
impl HostImplCServices<CompositorTypes> for ResourceServices {
    fn TraceHostImplCEvent(&mut self, category: &'static str, name: &'static str) {
        self.capabilities.events.Emit(category, name.into());
    }
    fn LayerContextSetTargetLocalSurfaceId(&mut self, c: &mut LayerContext, id: &LocalSurfaceId) {
        c.0.SetTargetLocalSurfaceId(id)
    }
    fn DelegateNotifyReadyToActivate(&mut self, d: HostImplDelegateId) {
        self.capabilities.lifecycle.NotifyReadyToActivate(d)
    }
    fn DelegateNotifyReadyToDraw(&mut self, d: HostImplDelegateId) {
        self.capabilities.lifecycle.NotifyReadyToDraw(d)
    }
    fn UpdateDisplayTile(
        &mut self,
        _: &mut LayerContext,
        _: &dyn PictureLayerDisplayRead,
        _: &dyn HostDisplayTileTiling,
        _: &Tile<CompositorTypes>,
        _: &mut ClientResourceProvider<CompositorTypes>,
        _: HostImplFrameSinkId,
        _: bool,
    ) {
        panic!("TreesInViz display-tile transport is not configured for the CPU compositor")
    }
    fn DelegateDidReceiveCompositorFrameAckOnImplThread(&mut self, d: HostImplDelegateId) {
        self.capabilities.lifecycle.DidReceiveCompositorFrameAck(d)
    }
    fn PresentationFeedbackFailed(&self, d: &FrameTimingDetails) -> bool {
        d.presentation_feedback.flags & (1 << 4) != 0
    }
    fn DelegateDidPresentCompositorFrameOnImplThread(
        &mut self,
        delegate: HostImplDelegateId,
        token: u32,
        mut callbacks: PendingCallbacks<PresentationFeedback, FrameTimingDetails>,
        details: &FrameTimingDetails,
    ) {
        // The original compositor callbacks were just consumed synchronously
        // with the actual Host and live context before entering this facet.
        assert!(callbacks.compositor_successful_callbacks.is_empty());
        self.capabilities
            .resource_effects
            .Push(ResourceEffect::Presented {
                delegate,
                token,
                callbacks,
                details: details.clone(),
            });
    }
    fn PostCompositorTask(
        &mut self,
        r: HostImplTaskRunnerId,
        l: TaskLocation,
        t: HostImplPostedCompositorTask<FrameTimingDetails>,
    ) {
        self.returns
            .PostDelayedCompositorTask(r, l, t, TimeDelta::default())
    }
    fn LagTrackingDidPresentCompositorFrame(
        &mut self,
        m: &mut AverageLagTrackingManager,
        t: u32,
        d: &FrameTimingDetails,
    ) {
        m.0.DidPresentCompositorFrame(t, d)
    }
    fn DelegateNotifyCompositorMetricsTrackerResults(
        &mut self,
        delegate: HostImplDelegateId,
        results: &CustomTrackerResults,
    ) {
        self.capabilities
            .resource_effects
            .Push(ResourceEffect::CustomMetrics {
                delegate,
                results: results.clone(),
            });
    }
    fn FrameTrackersNotifyPauseFrameProduction(
        &mut self,
        t: &mut FrameSequenceTrackers<CompositorTypes>,
    ) {
        t.0.FrameTrackersNotifyPauseFrameProduction()
    }
    fn DelegateOnDrawForLayerTreeFrameSink(&mut self, d: HostImplDelegateId, r: bool, s: bool) {
        self.capabilities
            .lifecycle
            .OnDrawForLayerTreeFrameSink(d, r, s)
    }
    fn DelegateNotifyTransitionRequestFinished(
        &mut self,
        delegate: HostImplDelegateId,
        sequence: u32,
        rects: HostViewTransitionElementResourceRects,
    ) {
        self.capabilities
            .resource_effects
            .Push(ResourceEffect::TransitionFinished {
                delegate,
                sequence,
                rects,
            });
    }
    fn LocalSurfaceIdIsNewerThanOrEmbeddingChanged(
        &self,
        a: &LocalSurfaceId,
        b: &LocalSurfaceId,
    ) -> bool {
        a.IsNewerThanOrEmbeddingChanged(b)
    }
    fn ResourceProviderSetEvicted(
        &mut self,
        p: &mut ClientResourceProvider<CompositorTypes>,
        evicted: bool,
    ) {
        p.0.SetEvicted(evicted)
    }
    // No custom recorder is installed in this capabilities constructor. Source guards
    // ReportEventLatency with this exact optional-owner query.
    fn HasCustomMetricRecorder(&self) -> bool {
        false
    }
    fn CustomMetricRecorderReportEventLatency(
        &mut self,
        _: &BeginFrameArgs,
        _: Vec<EventLatencyData>,
    ) {
        panic!("no custom metric recorder was configured")
    }
}

pub(crate) struct PictureReleaseServices {
    capabilities: Arc<FactoryCapabilities>,
}
impl PictureReleaseServices {
    pub(crate) fn new(capabilities: Arc<FactoryCapabilities>) -> Self {
        Self { capabilities }
    }
}
impl HostPictureRedrawServices<CompositorTypes> for PictureReleaseServices {
    fn SettingsFeatures(&self) -> &dyn LayerTreeSettingsFeatures {
        self.capabilities.picture.as_ref()
    }
    fn DelegateIsInsideDraw(&self, _: HostImplDelegateId) -> bool {
        self.capabilities.lifecycle.IsInsideDraw()
    }
    fn NotifyLatencyInfoSwapPromiseMonitor(
        &mut self,
        id: HostImplLatencyMonitorId,
    ) -> Option<Box<super::host_owners::SwapPromise<super::skia::SkRegion>>> {
        self.capabilities.latency_monitors.Notify(id)
    }
    fn SaveActiveEventMetrics(&mut self, m: &mut EventsMetricsManager) {
        m.0.SaveActiveEventMetrics()
    }
    fn DelegateSetNeedsRedrawOnImplThread(&mut self, d: HostImplDelegateId) {
        self.capabilities.lifecycle.SetNeedsRedraw(d)
    }
}
impl HostPictureReleaseServices<CompositorTypes> for PictureReleaseServices {
    fn TraceTileStateNotification(&mut self) {
        self.capabilities
            .events
            .Emit("cc", "NotifyTileStateChanged".into());
    }
    fn UpdateDisplayTile(
        &mut self,
        _: &mut LayerContext,
        _: &dyn PictureLayerDisplayRead,
        _: &PictureLayerTilingState<CompositorTypes>,
        _: &Tile<CompositorTypes>,
        _: &mut ClientResourceProvider<CompositorTypes>,
        _: HostImplFrameSinkId,
        _: bool,
    ) {
        panic!("TreesInViz display-tile transport is not configured for the CPU compositor")
    }
}
