//! The client-only commit lifecycle's narrow external values/capabilities.
//! No callback can dereference an owner; worker completions return typed values
//! and are applied by the Engine to the current resident client host.
use super::layer_tree_host_impl::{
    HostImplContext, HostImplFrameSinkId, HostImplServices, HostImplTypes,
};
use super::layer_tree_impl::{LayerTreeId, LayerTreeTypes};
use super::picture_layer_impl::PictureLayerImplTypes;

pub trait ClientBatchResourceRelease {}
pub trait ClientHostCapabilities<B: HostImplTypes> {
    fn ClientFirstContentfulPaintReceived(&self, sorter: &B::FrameSorter) -> bool;
    fn ClientOnFirstContentfulPaintReceived(&mut self, sorter: &mut B::FrameSorter);
    fn ClientScopedBatchResourceRelease(
        &mut self,
        provider: &mut B::ClientResourceProvider,
    ) -> Box<dyn ClientBatchResourceRelease>;
    fn ClientHandleRemovedScrollAnimatingElements(
        &mut self,
        owner: super::layer_tree_impl::LayerTreeHostId,
        commits_to_active: bool,
        target: &mut dyn super::animation_host::AnimationMutationTarget,
    );
    fn ClientVerboseLog(&self, message: &str);
    fn TracePendingTreeWaiting(
        &self,
        pending: LayerTreeId,
        active_lsid: &<B as LayerTreeTypes>::LocalSurfaceId,
    );
    fn ClientInvalidateFrameSink(&mut self, sink: HostImplFrameSinkId, needs_redraw: bool);
    fn ClientHistogramBoolean(&self, name: &'static str, value: bool);
}
pub trait ClientHostServices<B: HostImplTypes + PictureLayerImplTypes>:
    HostImplServices<B> + ClientHostCapabilities<B> + ClientWorkletCapabilities<B>
{
}
impl<
        B: HostImplTypes + PictureLayerImplTypes,
        T: HostImplServices<B> + ClientHostCapabilities<B> + ClientWorkletCapabilities<B>,
    > ClientHostServices<B> for T
{
}
pub struct ClientHostContext<'a, B: HostImplTypes + PictureLayerImplTypes> {
    pub services: &'a mut dyn ClientHostServices<B>,
}
impl<B: HostImplTypes + PictureLayerImplTypes> ClientHostContext<'_, B> {
    pub fn Base(&mut self) -> HostImplContext<'_, B> {
        HostImplContext {
            services: self.services,
        }
    }
}

use super::{
    image_animation::{PaintImageId, PaintImageIdFlatSet},
    paint_worklet::PaintWorkletTypes,
};
use std::{collections::BTreeMap, sync::Arc};
// External cc/paint job values. Inputs/records retain their actual producer
// allocations; this is the native worklet dispatch/result payload, not a tree.
pub struct PaintWorkletJob<B: super::picture_layer_impl::PictureLayerImplTypes> {
    pub layer_id: i32,
    pub input: Arc<B::Input>,
    pub animated_property_values: BTreeMap<B::PropertyKey, B::PropertyValue>,
    pub output: Option<B::PaintRecord>,
}
pub type PaintWorkletJobMap<B> = BTreeMap<u64, Vec<PaintWorkletJob<B>>>;
#[derive(Clone, Copy)]
pub enum ClientSequence {
    RAF,
    CanvasAnimation,
    JSAnimation,
    MainThreadAnimation,
    SETMainThreadAnimation,
}
pub struct TrackedAnimationSequenceInfo {
    pub id: usize,
    pub start: bool,
}

/// A benchmark call can borrow every actual resource it reads without lending
/// the controller that is executing the callback. The source owner is never
/// retained or made available through a reverse-reference capability.
pub struct BenchmarkContext<'a, B: HostImplTypes> {
    pub owner: super::layer_tree_impl::LayerTreeHostId,
    pub active: &'a mut super::super::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    pub pending: Option<&'a mut super::super::cc::trees::layer_tree_impl::LayerTreeImpl<B>>,
    pub recycle: Option<&'a mut super::super::cc::trees::layer_tree_impl::LayerTreeImpl<B>>,
    pub settings: &'a crate::cc::trees::layer_tree_settings::LayerTreeSettings,
    pub raster_caps: &'a crate::cc::trees::raster_capabilities::RasterCapabilities,
    pub task_provider: super::layer_tree_host_impl::HostImplTaskRunnerProviderId,
    pub services: &'a mut dyn HostImplServices<B>,
}
impl<B: HostImplTypes> super::micro_benchmark::HostHandle for BenchmarkContext<'_, B> {
    fn IsNull(&self) -> bool {
        self.owner.0 == 0
    }
}
// Keep worklet-specific capabilities separate from the base Host services. They
// contain only independent external controller/value/painter operations.
pub trait ClientWorkletCapabilities<
    B: HostImplTypes + super::picture_layer_impl::PictureLayerImplTypes,
>
{
    fn ClientTraceCommitComplete(&self, trace: crate::adapter::commit_types::BeginMainFrameTraceId);
    fn ClientScrollingThreadIsRaster(&self, trackers: &B::FrameSequenceTrackerCollection) -> bool;
    fn ClientCurrentFrameHadRAF(&self, mutator: &<B as LayerTreeTypes>::MutatorHost) -> bool;
    fn ClientMutatorHasViewTransition(&self, mutator: &<B as LayerTreeTypes>::MutatorHost) -> bool;
    fn ClientTakePendingTrackerInfos(
        &mut self,
        mutator: &mut <B as LayerTreeTypes>::MutatorHost,
    ) -> Vec<TrackedAnimationSequenceInfo>;
    fn ClientStartSequence(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        sequence: ClientSequence,
    );
    fn ClientStartCustomSequence(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        id: usize,
    );
    fn ClientStopCustomSequence(
        &mut self,
        trackers: &mut B::FrameSequenceTrackerCollection,
        id: usize,
    );
    fn ClientTakeImagesToInvalidate(
        &mut self,
        tracker: super::tile_manager::CheckerImageTrackerId,
    ) -> PaintImageIdFlatSet;
    fn ClientHasOngoingWorkletDispatch(&self, painter: &B::PaintWorkletLayerPainter) -> bool;
    fn ClientNotifyPaintWorkletStateChange(
        &mut self,
        delegate: super::layer_tree_host_impl::HostImplDelegateId,
        state: crate::cc::scheduler::scheduler_state_machine::PaintWorkletState,
    );
    /// A synchronous backend returns its completion here. An async backend
    /// emits the same typed completion to Engine, tagged only with weak owner
    /// identity; it cannot call the host or retain its call-scoped context.
    fn ClientDispatchWorklets(
        &mut self,
        painter: &mut B::PaintWorkletLayerPainter,
        jobs: PaintWorkletJobMap<B>,
        owner: super::layer_tree_host_impl::HostImplWeak,
    ) -> Option<PaintWorkletJobMap<B>>;
}
