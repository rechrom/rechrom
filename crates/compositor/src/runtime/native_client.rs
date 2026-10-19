//! Native values consumed by the canonical ClientLayerTreeHostImpl. No commit,
//! tree lifetime, animation or resource-release policy is implemented here.
use super::{
    engine_factory::NativeFactoryCapabilities,
    host_owners::{ClientResourceProvider, FrameSorter},
};
use crate::{
    adapter::{
        animation_host::AnimationMutationTarget,
        client_layer_tree_host_impl::{ClientBatchResourceRelease, ClientHostCapabilities},
        layer_tree_host_impl::HostImplFrameSinkId,
        layer_tree_impl::{LayerTreeHostId, LayerTreeId},
    },
    NativeCompositorTypes,
};
use std::sync::Arc;
pub(crate) struct NativeClientServices {
    native: Arc<NativeFactoryCapabilities>,
}
impl NativeClientServices {
    pub(crate) fn new(native: Arc<NativeFactoryCapabilities>) -> Self {
        Self { native }
    }
}
impl ClientHostCapabilities<NativeCompositorTypes> for NativeClientServices {
    fn ClientFirstContentfulPaintReceived(&self, sorter: &FrameSorter) -> bool {
        sorter.FirstContentfulPaintReceived()
    }
    fn ClientOnFirstContentfulPaintReceived(&mut self, sorter: &mut FrameSorter) {
        sorter.OnFirstContentfulPaintReceived();
    }
    fn ClientScopedBatchResourceRelease(
        &mut self,
        provider: &mut ClientResourceProvider<NativeCompositorTypes>,
    ) -> Box<dyn ClientBatchResourceRelease> {
        // Source: components/viz/common/features.cc:357. The pinned CPU feature
        // configuration has no override for the disabled-by-default batch feature.
        provider.0.CreateScopedBatchResourcesRelease(false)
    }
    fn ClientHandleRemovedScrollAnimatingElements(
        &mut self,
        owner: LayerTreeHostId,
        active: bool,
        target: &mut dyn AnimationMutationTarget,
    ) {
        assert_eq!(owner, target.AnimationHostOwner());
        self.native
            .animations
            .HandleRemovedScrollAnimatingElements(active, target);
    }
    fn ClientVerboseLog(&self, message: &str) {
        self.native.events.Emit("cc", message.into());
    }
    fn TracePendingTreeWaiting(&self, pending: LayerTreeId, active: &super::types::LocalSurfaceId) {
        self.native.events.Emit(
            "cc",
            format!("pending tree {pending:?} waiting; active surface {active:?}"),
        );
    }
    fn ClientInvalidateFrameSink(&mut self, sink: HostImplFrameSinkId, _needs_redraw: bool) {
        assert_eq!(
            sink,
            self.native
                .frame_sink
                .lock()
                .expect("native frame sink")
                .id()
        );
        // Source: cc/trees/layer_tree_frame_sink.h:116. The production CPU output
        // sink inherits this empty base hook; it is not the source redraw request.
    }
    fn ClientHistogramBoolean(&self, name: &'static str, value: bool) {
        self.native
            .events
            .HistogramCount(name, if value { 1.0 } else { 0.0 });
    }
}

use super::{
    host_metrics::FrameSequenceTrackers,
    host_owners::PaintWorkletLayerPainter,
    input::{MutatorHost, SmoothEffectDrivingThread},
};
use crate::adapter::client_layer_tree_host_impl::{
    ClientSequence, ClientWorkletCapabilities, PaintWorkletJobMap, TrackedAnimationSequenceInfo,
};
pub(crate) struct NativeClientWorkletServices {
    native: Arc<NativeFactoryCapabilities>,
}
impl NativeClientWorkletServices {
    pub(crate) fn new(native: Arc<NativeFactoryCapabilities>) -> Self {
        Self { native }
    }
    fn CheckMutator(&self, mutator: &MutatorHost) {
        assert_eq!(mutator.0.identity(), self.native.animations.identity());
    }
}
impl ClientWorkletCapabilities<NativeCompositorTypes> for NativeClientWorkletServices {
    fn ClientTraceCommitComplete(
        &self,
        trace: crate::adapter::commit_types::BeginMainFrameTraceId,
    ) {
        self.native
            .events
            .Emit("cc", format!("commit complete trace={trace:?}"));
    }
    fn ClientScrollingThreadIsRaster(
        &self,
        trackers: &FrameSequenceTrackers<NativeCompositorTypes>,
    ) -> bool {
        trackers.0.GetScrollingThread() == SmoothEffectDrivingThread::kRaster
    }
    fn ClientCurrentFrameHadRAF(&self, mutator: &MutatorHost) -> bool {
        self.CheckMutator(mutator);
        self.native.animations.CurrentFrameHadRAF()
    }
    fn ClientMutatorHasViewTransition(&self, mutator: &MutatorHost) -> bool {
        self.CheckMutator(mutator);
        self.native.animations.HasViewTransition()
    }
    fn ClientTakePendingTrackerInfos(
        &mut self,
        mutator: &mut MutatorHost,
    ) -> Vec<TrackedAnimationSequenceInfo> {
        self.CheckMutator(mutator);
        self.native
            .animations
            .TakePendingCompositorMetricsTrackerInfos()
    }
    fn ClientStartSequence(
        &mut self,
        trackers: &mut FrameSequenceTrackers<NativeCompositorTypes>,
        sequence: ClientSequence,
    ) {
        trackers.0.StartMainSequence(sequence);
    }
    fn ClientStartCustomSequence(
        &mut self,
        trackers: &mut FrameSequenceTrackers<NativeCompositorTypes>,
        id: usize,
    ) {
        trackers.0.StartCustomSequence(id);
    }
    fn ClientStopCustomSequence(
        &mut self,
        trackers: &mut FrameSequenceTrackers<NativeCompositorTypes>,
        id: usize,
    ) {
        trackers.0.StopCustomSequence(id);
    }
    fn ClientTakeImagesToInvalidate(
        &mut self,
        tracker: crate::adapter::tile_manager::CheckerImageTrackerId,
    ) -> crate::adapter::image_animation::PaintImageIdFlatSet {
        self.native
            .tile_execution
            .lock()
            .expect("tile execution")
            .images
            .CheckerImageTracker(tracker)
            .TakeImagesToInvalidateOnSyncTree()
            .clone()
    }
    fn ClientHasOngoingWorkletDispatch(&self, painter: &PaintWorkletLayerPainter) -> bool {
        painter.0.HasOngoingDispatch()
    }
    fn ClientNotifyPaintWorkletStateChange(
        &mut self,
        delegate: crate::adapter::layer_tree_host_impl::HostImplDelegateId,
        state: crate::cc::scheduler::scheduler_state_machine::PaintWorkletState,
    ) {
        self.native
            .commands
            .Push(super::execution::EngineTask::PaintWorkletState { delegate, state });
    }
    fn ClientDispatchWorklets(
        &mut self,
        painter: &mut PaintWorkletLayerPainter,
        jobs: PaintWorkletJobMap<NativeCompositorTypes>,
        owner: crate::adapter::layer_tree_host_impl::HostImplWeak,
    ) -> Option<PaintWorkletJobMap<NativeCompositorTypes>> {
        painter.0.DispatchWorklets(jobs, owner)
    }
}
