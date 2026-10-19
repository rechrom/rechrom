//! Native resource/feedback forwarding. The source Host remains the sole owner
//! of tile notification, damage, completion, budget and release decisions.
use super::{
    engine_factory::NativeFactoryCapabilities,
    host_metrics::{CustomTrackerResults, EventLatencyData, FrameSequenceTrackers},
    host_owners::{
        AverageLagTrackingManager, ClientResourceProvider, EventsMetricsManager, LayerContext,
    },
    metadata::{FrameTimingDetails, PresentationFeedback},
    native_host_frame_sink::NativeHostFrameSinkCallbacks,
    native_resource_events::{NativeResourceEffect, NativeResourceEffects},
    resources::CpuReturnedResource,
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
    NativeCompositorTypes,
};
use std::sync::Arc;
pub(crate) struct NativeResourceServices {
    native: Arc<NativeFactoryCapabilities>,
    returns: NativeHostFrameSinkCallbacks,
}
impl NativeResourceServices {
    pub(crate) fn new(native: Arc<NativeFactoryCapabilities>) -> Self {
        let returns = NativeHostFrameSinkCallbacks::new(
            native.frame_sink.clone(),
            native.lifecycle.clone(),
            native.runners.clone(),
            native.commands.clone(),
        );
        Self { native, returns }
    }
}
impl HostImplTaskRunnerServices for NativeResourceServices {
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
impl HostImplResourceReturnServices<NativeCompositorTypes> for NativeResourceServices {
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
        p: &mut ClientResourceProvider<NativeCompositorTypes>,
        r: Vec<CpuReturnedResource>,
        pool: Option<&mut ResourcePool<NativeCompositorTypes>>,
        client: &mut dyn HostUIResourceReleaseClient<NativeCompositorTypes>,
    ) {
        self.returns.ReceiveReturnsFromParent(p, r, pool, client)
    }
    fn ResourcePoolReduceResourceUsage(&mut self, pool: &mut ResourcePool<NativeCompositorTypes>) {
        self.returns.ResourcePoolReduceResourceUsage(pool)
    }
    fn ContextHasSupport(&self, id: HostImplContextProviderId) -> bool {
        self.returns.ContextHasSupport(id)
    }
    fn ContextSupportFlushPendingWork(&mut self, id: HostImplContextProviderId) {
        self.returns.ContextSupportFlushPendingWork(id)
    }
}
impl HostImplCServices<NativeCompositorTypes> for NativeResourceServices {
    fn TraceHostImplCEvent(&mut self, category: &'static str, name: &'static str) {
        self.native.events.Emit(category, name.into());
    }
    fn LayerContextSetTargetLocalSurfaceId(&mut self, c: &mut LayerContext, id: &LocalSurfaceId) {
        c.0.SetTargetLocalSurfaceId(id)
    }
    fn DelegateNotifyReadyToActivate(&mut self, d: HostImplDelegateId) {
        self.native.lifecycle.NotifyReadyToActivate(d)
    }
    fn DelegateNotifyReadyToDraw(&mut self, d: HostImplDelegateId) {
        self.native.lifecycle.NotifyReadyToDraw(d)
    }
    fn UpdateDisplayTile(
        &mut self,
        _: &mut LayerContext,
        _: &dyn PictureLayerDisplayRead,
        _: &dyn HostDisplayTileTiling,
        _: &Tile<NativeCompositorTypes>,
        _: &mut ClientResourceProvider<NativeCompositorTypes>,
        _: HostImplFrameSinkId,
        _: bool,
    ) {
        panic!("TreesInViz display-tile transport is not configured for the CPU compositor")
    }
    fn DelegateDidReceiveCompositorFrameAckOnImplThread(&mut self, d: HostImplDelegateId) {
        self.native.lifecycle.DidReceiveCompositorFrameAck(d)
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
        self.native
            .resource_effects
            .Push(NativeResourceEffect::Presented {
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
        self.native
            .resource_effects
            .Push(NativeResourceEffect::CustomMetrics {
                delegate,
                results: results.clone(),
            });
    }
    fn FrameTrackersNotifyPauseFrameProduction(
        &mut self,
        t: &mut FrameSequenceTrackers<NativeCompositorTypes>,
    ) {
        t.0.FrameTrackersNotifyPauseFrameProduction()
    }
    fn DelegateOnDrawForLayerTreeFrameSink(&mut self, d: HostImplDelegateId, r: bool, s: bool) {
        self.native.lifecycle.OnDrawForLayerTreeFrameSink(d, r, s)
    }
    fn DelegateNotifyTransitionRequestFinished(
        &mut self,
        delegate: HostImplDelegateId,
        sequence: u32,
        rects: HostViewTransitionElementResourceRects,
    ) {
        self.native
            .resource_effects
            .Push(NativeResourceEffect::TransitionFinished {
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
        p: &mut ClientResourceProvider<NativeCompositorTypes>,
        evicted: bool,
    ) {
        p.0.SetEvicted(evicted)
    }
    // No custom recorder is installed in this native constructor. Source guards
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

pub(crate) struct NativePictureReleaseServices {
    native: Arc<NativeFactoryCapabilities>,
}
impl NativePictureReleaseServices {
    pub(crate) fn new(native: Arc<NativeFactoryCapabilities>) -> Self {
        Self { native }
    }
}
impl HostPictureRedrawServices<NativeCompositorTypes> for NativePictureReleaseServices {
    fn SettingsFeatures(&self) -> &dyn LayerTreeSettingsFeatures {
        self.native.picture.as_ref()
    }
    fn DelegateIsInsideDraw(&self, _: HostImplDelegateId) -> bool {
        self.native.lifecycle.IsInsideDraw()
    }
    fn NotifyLatencyInfoSwapPromiseMonitor(
        &mut self,
        id: HostImplLatencyMonitorId,
    ) -> Option<Box<super::host_owners::SwapPromise<super::skia::SkRegion>>> {
        self.native.latency_monitors.Notify(id)
    }
    fn SaveActiveEventMetrics(&mut self, m: &mut EventsMetricsManager) {
        m.0.SaveActiveEventMetrics()
    }
    fn DelegateSetNeedsRedrawOnImplThread(&mut self, d: HostImplDelegateId) {
        self.native.lifecycle.SetNeedsRedraw(d)
    }
}
impl HostPictureReleaseServices<NativeCompositorTypes> for NativePictureReleaseServices {
    fn TraceTileStateNotification(&mut self) {
        self.native
            .events
            .Emit("cc", "NotifyTileStateChanged".into());
    }
    fn UpdateDisplayTile(
        &mut self,
        _: &mut LayerContext,
        _: &dyn PictureLayerDisplayRead,
        _: &PictureLayerTilingState<NativeCompositorTypes>,
        _: &Tile<NativeCompositorTypes>,
        _: &mut ClientResourceProvider<NativeCompositorTypes>,
        _: HostImplFrameSinkId,
        _: bool,
    ) {
        panic!("TreesInViz display-tile transport is not configured for the CPU compositor")
    }
}
