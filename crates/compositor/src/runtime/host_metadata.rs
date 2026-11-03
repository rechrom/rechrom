//! Native metadata/submission capabilities for the resident CPU HostImpl.
//!
//! Chromium correspondence:
//! - cc/trees/layer_tree_host_impl.cc:2860-2910,3098-3130
//! - cc/metrics/event_metrics.cc:997-1004
//! - cc/metrics/frame_sequence_metrics.h:63-80
//!
//! This facade owns no compositor tree and creates no frame snapshot. It
//! forwards the canonical HostImpl decisions to the Page-local native owners.

use super::{
    frame::CompositorRenderPass,
    host::TraceValue,
    host_lifecycle::HostLifecycle,
    host_owners::{
        ClientResourceProvider, LayerContext, RenderFrameMetadataObserver, ViewTransitionRequest,
    },
    input::{EventMetrics, EventMetricsSet, FrameSequenceTrackerType, LatencyInfo, MutatorHost},
    layer_tree_frame_sink_endpoint::FrameSinkEndpoint,
    metadata::{DelegatedInkMetadata, FrameDeadline, FrameTokenGenerator, TransitionDirective},
    resources::{RasterDisplayColorSpaces, RasterTransferableResource},
    services::EventSink,
    types::FrameSinkId,
};
use crate::{
    adapter::{
        geometry::{Rect, Vector2dF},
        layer_tree_host_impl::{
            HostCheckerboardMode, HostCompositorFrame, HostDrawSequence, HostImplDServices,
            HostImplDelegateId, HostImplFrameSinkId, HostImplRenderingStatsId,
            HostImplVideoFrameControllerId, HostMetadataTransitionRequest,
            HostSharedImageInterfaceId, HostTraceValue,
        },
        time::{TimeDelta, TimeTicks},
    },
    runtime::{host_metrics::FrameSequenceTrackers, layer::CompositorLayer, skia::SkRegion},
    CompositorFrame, CompositorTypes,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

/// Explicit policy selected by the embedding. No feature flag is sampled from
/// a hidden global while drawing a frame.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct HostMetadataConfig {
    pub checkerboard_mode: HostCheckerboardMode,
    pub is_new_trace: bool,
    pub trees_in_viz: bool,
}

pub(crate) struct HostMetadataServices {
    endpoint: Arc<Mutex<FrameSinkEndpoint>>,
    lifecycle: Arc<HostLifecycle>,
    events: Arc<dyn EventSink>,
    config: HostMetadataConfig,
}

impl HostMetadataServices {
    pub(crate) fn new(
        endpoint: Arc<Mutex<FrameSinkEndpoint>>,
        lifecycle: Arc<HostLifecycle>,
        events: Arc<dyn EventSink>,
        config: HostMetadataConfig,
    ) -> Self {
        Self {
            endpoint,
            lifecycle,
            events,
            config,
        }
    }

    fn WithEndpoint<T>(
        &self,
        sink: HostImplFrameSinkId,
        f: impl FnOnce(&mut FrameSinkEndpoint) -> T,
    ) -> T {
        let mut endpoint = self.endpoint.lock().expect("native frame-sink endpoint");
        assert_eq!(
            endpoint.id(),
            sink,
            "HostImpl used a different frame-sink endpoint"
        );
        f(&mut endpoint)
    }

    fn Bit(types: &u16, kind: FrameSequenceTrackerType) -> bool {
        (*types & (1u16 << kind as u8)) != 0
    }
}

impl HostImplDServices<CompositorTypes> for HostMetadataServices {
    fn MakeEventMetricsSet(
        &self,
        main: Vec<EventMetrics>,
        saved: Vec<EventMetrics>,
        raster: Vec<EventMetrics>,
    ) -> EventMetricsSet {
        EventMetricsSet {
            main_event_metrics: main,
            impl_event_metrics: saved,
            raster_event_metrics: raster,
        }
    }

    fn CollectScrollEventsFromFrame(
        &mut self,
        manager: &mut super::host_owners::AverageLagTrackingManager,
        token: u32,
        events: &EventMetricsSet,
    ) {
        manager.CollectScrollEventsFromFrame(token, events)
    }

    fn FrameSinkSetSourceFrameNumber(&mut self, sink: HostImplFrameSinkId, number: i32) {
        self.WithEndpoint(sink, |endpoint| endpoint.SetSourceFrameNumber(number))
    }

    fn PostCompositorFrameDump(
        &mut self,
        frame: &CompositorFrame,
        client: bool,
        viz: bool,
        token: u32,
    ) {
        self.events.Emit(
            "compositor.frame_dump",
            format!(
                "token={token} client={client} viz={viz} passes={} resources={}",
                frame.render_pass_list.len(),
                frame.resource_list.len()
            ),
        )
    }

    fn VerboseLogCompositorFrame(&self, frame: &CompositorFrame) {
        self.events.Emit(
            "compositor.frame",
            format!(
                "passes={} resources={}",
                frame.render_pass_list.len(),
                frame.resource_list.len()
            ),
        )
    }

    fn FrameSinkExportFrameTiming(&mut self, sink: HostImplFrameSinkId) {
        assert!(
            self.config.trees_in_viz,
            "ExportFrameTiming requires the TreesInViz transport"
        );
        self.WithEndpoint(sink, |_| ());
        self.events.Emit("compositor.frame_timing", "export".into());
    }

    fn TraceSubmitCompositorFrame(&self, trace_id: i64, latency: &[LatencyInfo]) {
        self.events.Emit(
            "compositor.submit",
            format!("trace_id={trace_id} latency_events={}", latency.len()),
        )
    }

    fn FrameSinkSubmitCompositorFrame(
        &mut self,
        sink: HostImplFrameSinkId,
        frame: CompositorFrame,
        hit_test_data_changed: bool,
    ) {
        self.WithEndpoint(sink, |endpoint| {
            endpoint.SubmitCompositorFrame(frame, hit_test_data_changed)
        })
    }

    fn MutatorNextFrameHasPendingRAF(&self, host: &MutatorHost) -> bool {
        host.NextFrameHasPendingRAF()
    }
    fn MutatorHasCanvasInvalidation(&self, host: &MutatorHost) -> bool {
        host.HasCanvasInvalidation()
    }
    fn MutatorHasJSAnimation(&self, host: &MutatorHost) -> bool {
        host.HasJSAnimation()
    }
    fn MutatorMainThreadAnimationsCount(&self, host: &MutatorHost) -> usize {
        host.MainThreadAnimationsCount()
    }
    fn MutatorHasSmilAnimation(&self, host: &MutatorHost) -> bool {
        host.HasSmilAnimation()
    }

    fn StopDrawSequence(
        &mut self,
        trackers: &mut FrameSequenceTrackers<CompositorTypes>,
        kind: HostDrawSequence,
    ) {
        trackers.StopDrawSequence(kind)
    }

    fn DidDrawDevtoolsFrame(&self, id: i32, sequence: u64) {
        self.events.Emit(
            "compositor.devtools",
            format!("host={id} sequence={sequence}"),
        )
    }

    fn IssueImplThreadRenderingStatsEvent(&mut self, stats: HostImplRenderingStatsId) {
        self.events
            .Emit("compositor.rendering_stats", format!("stats={}", stats.0))
    }

    fn FrameSinksToThrottleUpdated(
        &mut self,
        delegate: HostImplDelegateId,
        ids: &std::collections::BTreeSet<FrameSinkId>,
    ) {
        self.lifecycle.FrameSinksToThrottleUpdated(delegate, ids)
    }

    fn NewContentForCheckerboardedScrollsMode(&self) -> HostCheckerboardMode {
        self.config.checkerboard_mode
    }

    fn TileManagerMemoryStats<'a>(
        &self,
        manager: &'a crate::cc::tiles::tile_manager::TileManager<CompositorTypes>,
    ) -> &'a crate::cc::resources::memory_history::Entry {
        manager.memory_stats_from_last_assign()
    }

    fn TraceScrollDeltaThisFrame(&self, delta: Vector2dF) {
        self.events.Emit(
            "compositor.scroll_delta",
            format!("x={} y={}", delta.x(), delta.y()),
        )
    }

    fn IsNewTrace(&self) -> bool {
        self.config.is_new_trace
    }

    fn TraceFrameViewerSnapshot(&self, id: i32, value: &TraceValue) {
        self.events.Emit(
            "compositor.frame_viewer",
            format!("host={id} {}", value.ToFormattedJSON()),
        )
    }

    fn UpdateHudTexture(
        &mut self,
        _layer: &mut CompositorLayer<CompositorTypes>,
        _mode: crate::cc::layers::draw_mode::DrawMode,
        _sink: HostImplFrameSinkId,
        _provider: Option<&mut ClientResourceProvider<CompositorTypes>>,
        _caps: &crate::cc::trees::raster_capabilities::RasterCapabilities,
        _passes: &mut Vec<Box<CompositorRenderPass<CompositorTypes>>>,
    ) {
        panic!("CPU layer family has no HeadsUpDisplayLayerImpl owner")
    }

    fn ConstructTransitionDirective(
        &self,
        request: &ViewTransitionRequest,
        elements: &BTreeMap<
            crate::adapter::property_types::ViewTransitionElementResourceId,
            super::frame::RenderPassId,
        >,
        colors: &RasterDisplayColorSpaces,
        delay: bool,
    ) -> TransitionDirective {
        request.ConstructDirective(elements, colors, delay)
    }

    fn MakeFrameDeadline(
        &self,
        time: TimeTicks,
        frames: u32,
        interval: TimeDelta,
        lower_bound: bool,
    ) -> FrameDeadline {
        FrameDeadline::new(time, frames, interval, lower_bound)
    }

    fn DelegatedInkIsHovering(&self, metadata: &DelegatedInkMetadata) -> bool {
        metadata.is_hovering
    }

    fn ObserveRenderFrameSubmission(
        &mut self,
        observer: &mut RenderFrameMetadataObserver<SkRegion>,
        render_metadata: &crate::cc::trees::render_frame_metadata::RenderFrameMetadata<
            CompositorTypes,
        >,
        metadata: &mut super::metadata::CompositorFrameMetadata<CompositorTypes>,
        force: bool,
    ) {
        observer.ObserveRenderFrameSubmission(render_metadata, metadata, force)
    }

    fn AddRendererSwapLatencyTimestamp(&self, latency: &mut LatencyInfo, time: TimeTicks) {
        latency.AddRendererSwapTimestamp(time)
    }

    fn PrepareSendResourcesToParent(
        &mut self,
        provider: &mut ClientResourceProvider<CompositorTypes>,
        resources: &[crate::adapter::layer_tree_impl::ResourceId],
        output: &mut Vec<RasterTransferableResource>,
        interface: HostSharedImageInterfaceId,
    ) {
        provider.0.PrepareSendToParent(resources, output, interface)
    }

    fn RecordCompositorFrameQuadCount(&self, client: &str, count: usize) {
        self.events.Emit(
            "compositor.quad_count",
            format!("client={client} count={count}"),
        )
    }

    fn TraceNoDamageEarlyOut(&self) {
        self.events.Emit("compositor.no_damage", "early_out".into())
    }

    fn FrameSinkSetCurrentLocalSurfaceId(
        &mut self,
        sink: HostImplFrameSinkId,
        id: &super::types::LocalSurfaceId,
    ) {
        self.WithEndpoint(sink, |endpoint| endpoint.SetCurrentLocalSurfaceId(id))
    }

    fn IncrementFrameToken(&self, token: &mut FrameTokenGenerator) -> u32 {
        token.Next()
    }

    fn HasMainThreadAnimation(&self, types: &u16) -> bool {
        Self::Bit(types, FrameSequenceTrackerType::kMainThreadAnimation)
            || Self::Bit(types, FrameSequenceTrackerType::kCanvasAnimation)
            || Self::Bit(types, FrameSequenceTrackerType::kJSAnimation)
            || Self::Bit(types, FrameSequenceTrackerType::kRAF)
    }

    fn HasCompositorThreadAnimation(&self, types: &u16) -> bool {
        Self::Bit(types, FrameSequenceTrackerType::kCompositorAnimation)
            || Self::Bit(types, FrameSequenceTrackerType::kCompositorRasterAnimation)
            || Self::Bit(types, FrameSequenceTrackerType::kCompositorAnimation)
    }

    fn CloneDelegatedInkMetadata(
        &self,
        metadata: &DelegatedInkMetadata,
    ) -> Box<DelegatedInkMetadata> {
        Box::new(metadata.clone())
    }
    fn SetDelegatedInkFrameTime(&self, metadata: &mut DelegatedInkMetadata, time: TimeTicks) {
        metadata.frame_time = time
    }
    fn TraceDelegatedInkOnCompositorFrame(&self, metadata: &DelegatedInkMetadata) {
        self.events.Emit(
            "compositor.delegated_ink",
            format!(
                "hovering={} frame_time={:?}",
                metadata.is_hovering, metadata.frame_time
            ),
        )
    }

    fn VideoFrameControllerDidDrawFrame(&mut self, controller: HostImplVideoFrameControllerId) {
        self.lifecycle.VideoFrameControllerDidDrawFrame(controller)
    }

    fn LayerContextUpdateDisplayTreeFrom(
        &mut self,
        _layer_context: &mut LayerContext,
        _tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<CompositorTypes>,
        _resource_provider: &mut ClientResourceProvider<CompositorTypes>,
        _shared_image_interface: HostSharedImageInterfaceId,
        _damage: Rect,
        _has_damage: bool,
        _is_flush: bool,
        _latency_info: Vec<LatencyInfo>,
        _tracked: super::metadata::TrackedElementRects,
    ) -> TimeTicks {
        panic!("TreesInViz display-tree transport is not configured for the CPU compositor")
    }
}
