//! Independent native environment used by the canonical tree-host facade.
//! Returned values/capabilities belong to the external runner/sink allocations;
//! no Host, tree, Page, or owner callback is retained here.
use super::{
    engine_factory::NativeFactoryCapabilities,
    frame_sink::FrameSink,
    host_owners::{RasterContextProvider, TaskRunnerProvider, VideoFrameControllerClient},
    input::MutatorHost,
};
use crate::{
    adapter::{
        input_types::ElementId,
        layer_tree_host_impl::*,
        layer_tree_impl::{LayerTreeHostId, LayerTreeId, PropertyTreesFor},
    },
    cc::{
        layers::render_surface_impl::RenderSurfaceImpl,
        trees::commit_state::ThreadUnsafeCommitState,
    },
    NativeCompositorTypes,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
#[derive(Clone, Copy)]
pub(crate) struct NativeTreeEnvironmentConfig {
    pub selection_edge_visibility_uses_full_edge: bool,
    pub view_transition_capture_and_display: bool,
}
pub(crate) struct NativeTreeEnvironmentServices {
    native: Arc<NativeFactoryCapabilities>,
    video_client: VideoFrameControllerClient,
    provider_id: HostImplTaskRunnerProviderId,
    provider: TaskRunnerProvider,
}
impl NativeTreeEnvironmentServices {
    pub(crate) fn new(
        native: Arc<NativeFactoryCapabilities>,
        owner: LayerTreeHostId,
        provider_id: HostImplTaskRunnerProviderId,
    ) -> Self {
        let main = Arc::new(super::execution::EngineSequencedTaskRunner::new(
            native
                .runners
                .Resolve(native.runners.MainRunnerId())
                .expect("main runner")
                .clone(),
            native.commands.clone(),
        ));
        let compositor =
            if crate::adapter::layer_tree_host_impl::HostImplTaskRunnerServices::HasImplThread(
                native.runners.as_ref(),
                provider_id,
            ) {
                Some(Arc::new(super::execution::EngineSequencedTaskRunner::new(
                    native
                        .runners
                        .Resolve(native.runners.CompositorRunnerId())
                        .expect("compositor runner")
                        .clone(),
                    native.commands.clone(),
                ))
                    as Arc<dyn crate::adapter::task_runner::SequencedTaskRunner>)
            } else {
                None
            };
        Self {
            native,
            video_client: VideoFrameControllerClient { owner },
            provider_id,
            provider: TaskRunnerProvider::new(main, compositor),
        }
    }
    fn CheckSink(&self, id: HostImplFrameSinkId) {
        assert_eq!(
            id,
            self.native
                .frame_sink
                .lock()
                .expect("native frame sink")
                .id()
        )
    }
}
impl HostImplTreeHostServices<NativeCompositorTypes> for NativeTreeEnvironmentServices {
    fn SelectionEdgeVisibilityUsesFullEdge(&self) -> bool {
        self.native
            .tree_environment
            .selection_edge_visibility_uses_full_edge
    }
    fn ViewTransitionCaptureAndDisplay(&self) -> bool {
        self.native
            .tree_environment
            .view_transition_capture_and_display
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.native.picture.cc_slimming
    }
    fn VideoFrameControllerClient(&self, owner: LayerTreeHostId) -> &VideoFrameControllerClient {
        assert_eq!(owner, self.video_client.owner);
        &self.video_client
    }
    fn TaskRunnerProvider(&self, id: HostImplTaskRunnerProviderId) -> &TaskRunnerProvider {
        assert_eq!(id, self.provider_id);
        &self.provider
    }
    fn WithLayerTreeFrameSink(
        &self,
        id: HostImplFrameSinkId,
        callback: &mut dyn FnMut(&FrameSink<NativeCompositorTypes>),
    ) {
        let endpoint = self.native.frame_sink.lock().expect("native frame sink");
        assert_eq!(id, endpoint.id());
        callback(endpoint.LayerTreeFrameSink())
    }
    // This is the same specifically CPU endpoint exposed by Core/G; it has no
    // GPU RasterContextProvider. No substitute lock or context is manufactured.
    fn FrameSinkRasterContext(&self, id: HostImplFrameSinkId) -> Option<&RasterContextProvider> {
        self.CheckSink(id);
        None
    }
    fn TraceDidActivateLayerTree(&self, owner: LayerTreeHostId, frame: i32) {
        self.native.events.Emit(
            "cc",
            format!(
                "DidActivateLayerTree owner={} source_frame_number={frame}",
                owner.0
            ),
        )
    }
    fn MissingPropertyNodeForElementId(
        &self,
        mutation: crate::cc::trees::layer_tree_impl::PropertyMutation,
    ) {
        self.native.events.Emit(
            "compositor.property_tree",
            format!("MissingPropertyNodeForElementId mutation={mutation:?}"),
        )
    }
    fn TraceDelegatedInkMetadataPush(&self, metadata: &super::metadata::DelegatedInkMetadata) {
        self.native.events.Emit(
            "delegated_ink_trails",
            format!("PushDelegatedInkMetadata {metadata:?}"),
        )
    }
    fn TraceLayersUpdatedCount(&self, count: usize) {
        self.native
            .events
            .Emit("cc", format!("LayersUpdated count={count}"))
    }
    fn TraceImageInvalidationCounts(
        &self,
        no_images: usize,
        no_invalidation: usize,
        invalidated: usize,
    ) {
        self.native.events.Emit("cc",format!("ImageInvalidation no_images={no_images} no_invalidation={no_invalidation} invalidated={invalidated}"))
    }
    fn PushMutatorPropertiesTo(
        &mut self,
        state: &ThreadUnsafeCommitState<NativeCompositorTypes>,
        target: &mut MutatorHost,
        trees: &PropertyTreesFor<NativeCompositorTypes>,
    ) {
        self.native
            .main_animations
            .PushMutatorPropertiesTo(state.mutator_host, target, trees)
    }
    fn CreateRenderSurface(
        &mut self,
        tree: LayerTreeId,
        id: ElementId,
    ) -> Box<RenderSurfaceImpl<NativeCompositorTypes>> {
        Box::new(RenderSurfaceImpl::new_with_stable_id_sequence_value(
            Some(tree),
            id,
            super::native_render::NextStableIdSequenceValue(),
        ))
    }
    // Source: cc/trees/layer_tree_impl.cc:1437-1447. The source once flag is
    // process-wide; only diagnostic state is shared between unrelated Pages.
    fn ReportUnexpectedOopifPageScaleOnce(&mut self, scale: f32) {
        static HAS_DUMPED: AtomicBool = AtomicBool::new(false);
        if !HAS_DUMPED.swap(true, Ordering::Relaxed) {
            self.native
                .events
                .Emit("compositor.check", format!("psf_oopif_error={scale:.6}"))
        }
    }
}
