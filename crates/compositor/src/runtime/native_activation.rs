//! External activation capabilities for the canonical Host. Tree switching,
//! raster-capability selection and cache lifetime remain in the translated stem.
use super::{
    engine_factory::NativeFactoryCapabilities, host_owners::*,
    native_resource_events::NativeResourceEffect,
};
use crate::{
    adapter::{layer_tree_host_impl::*, layer_tree_impl::LayerTreeId},
    cc::trees::layer_tree_impl::LayerTreeImpl,
    NativeCompositorTypes as B,
};
use std::sync::Arc;

pub(crate) struct NativeActivationServices {
    native: Arc<NativeFactoryCapabilities>,
    image_decode: ImageDecodeFactory,
}
impl NativeActivationServices {
    pub(crate) fn new(
        native: Arc<NativeFactoryCapabilities>,
        image_decode: ImageDecodeFactory,
    ) -> Self {
        Self {
            native,
            image_decode,
        }
    }
    fn Trace(&self, event: &str) {
        if self.native.frame_trace_categories.contains("cc") {
            self.native.events.Emit("cc", event.into());
        }
    }
}
impl HostImplFServices<B> for NativeActivationServices {
    // Source: cc/trees/proxy_impl.cc:495-501. The original main-thread signal
    // and scheduler mutation are consumed after this source-call borrow ends.
    fn DelegateDidLoseLayerTreeFrameSink(&mut self, delegate: HostImplDelegateId) {
        self.native
            .resource_effects
            .Push(NativeResourceEffect::DidLoseLayerTreeFrameSink { delegate });
    }
    fn LagTrackingClear(&mut self, lag: &mut AverageLagTrackingManager) {
        lag.0.Clear();
    }
    fn FrameSorterReset(&mut self, sorter: &mut FrameSorter, reset_fcp: bool) {
        sorter.Reset(reset_fcp);
    }
    fn TraceActivateSyncTree(
        &self,
        trace: &mut dyn FnMut() -> crate::adapter::commit_types::BeginMainFrameTraceId,
    ) {
        if self.native.frame_trace_categories.contains("cc") {
            let id = trace();
            self.native
                .events
                .Emit("cc", format!("ActivateSyncTree {id:?}"));
        }
    }
    fn TracePendingTreeActivated(&self, tree: LayerTreeId, local: &super::types::LocalSurfaceId) {
        if self.native.frame_trace_categories.contains("cc") {
            self.native
                .events
                .Emit("cc", format!("PendingTreeActivated {tree:?} {local:?}"));
        }
    }
    fn TraceTreeSynchronization(&self, name: &str, count: Option<usize>) {
        if self.native.frame_trace_categories.contains("cc") {
            self.native.events.Emit("cc", format!("{name} {count:?}"));
        }
    }
    fn PromoteScrollTimelinesPendingToActive(&mut self, mutator: &mut super::input::MutatorHost) {
        assert_eq!(mutator.identity(), self.native.animations.identity());
        self.native
            .animations
            .PromoteScrollTimelinesPendingToActive(mutator);
    }
    fn InvalidPreviousSurfaceScreenshot(&self) {
        self.native.events.Emit(
            "cc",
            "Invalid previous surface screenshot destination".into(),
        );
    }
    // Source: cc/trees/proxy_impl.cc:660-668. The activation-completion pointer
    // only exists in blocking threaded commits; this single-sequence assembly
    // has no such event. The source trace is still emitted when requested.
    fn DelegateDidActivateSyncTree(&mut self, _delegate: HostImplDelegateId) {
        self.Trace("ProxyImpl::DidActivateSyncTreeOnImplThread");
    }
    fn RunTreeActivationCallback(
        &mut self,
        callback: &crate::adapter::task_runner::RepeatingClosure,
    ) {
        callback();
    }
    fn VerboseLogActivatedTree(&self, tree: &LayerTreeImpl<B>) {
        if self
            .native
            .diagnostics
            .maximum_vlog_level
            .is_some_and(|v| v >= 3)
        {
            self.native
                .events
                .Emit("cc", format!("Activated tree {:?}", tree.id));
        }
    }
    fn ReportingSetVisible(
        &mut self,
        reporting: &mut super::host_metrics::ReportingController<B>,
        visible: bool,
    ) {
        reporting.0.ReportingSetVisible(visible);
    }
    fn LayerContextSetVisible(&mut self, context: &mut LayerContext, visible: bool) {
        context.0.SetVisible(visible);
    }
    fn ResourceProviderSetVisible(
        &mut self,
        provider: &mut ClientResourceProvider<B>,
        visible: bool,
    ) {
        provider.0.SetVisible(visible);
    }
    fn ImageDecodeFactory(&self) -> &ImageDecodeFactory {
        &self.image_decode
    }
    // Source: cc/trees/layer_tree_host_impl.cc:4358-4365. CPU has no worker
    // context. Preserve nullptr, rather than inventing a context allocation.
    fn ImageDecodeContextProvider(
        &self,
        provider: Option<HostImplContextProviderId>,
    ) -> Option<RasterContextProvider> {
        assert!(
            provider.is_none(),
            "CPU activation has no GPU worker context"
        );
        None
    }
    fn ImageDecodeDarkModeFilter(
        &self,
        id: Option<HostImplDarkModeFilterId>,
    ) -> Option<&(dyn NativeDarkModeFilter + 'static)> {
        assert!(
            id.is_none(),
            "no dark-mode filter injected into this native assembly"
        );
        None
    }
    fn CreateRasterQueryQueue(
        &mut self,
        _worker: HostImplContextProviderId,
    ) -> Box<RasterQueryQueue> {
        panic!(
            "GPU raster query requires an injected GPU context; CPU source branch never calls this"
        )
    }
    fn CreateSynchronousTaskGraphRunner(
        &mut self,
    ) -> Box<<B as HostImplTypes>::SynchronousTaskGraphRunner> {
        let execution = self.native.tile_execution.lock().expect("tile execution");
        Box::new(Arc::new(
            crate::cc::raster::synchronous_task_graph_runner::SynchronousTaskGraphRunner::new(
                execution.diagnostics.clone(),
            ),
        ))
    }
    fn FrameSinkSharedImageInterface(
        &self,
        sink: HostImplFrameSinkId,
    ) -> HostSharedImageInterfaceId {
        assert_eq!(
            sink,
            self.native.frame_sink.lock().expect("frame sink").id()
        );
        HostSharedImageInterfaceId(sink.0)
    }
    fn ContextProviderSharedImageInterface(
        &self,
        _provider: HostImplContextProviderId,
    ) -> HostSharedImageInterfaceId {
        panic!("CPU source branch has no context provider")
    }
    fn CreateZeroCopyRasterBufferProvider(
        &mut self,
        interface: HostSharedImageInterfaceId,
        is_software: bool,
    ) -> Box<RasterBufferProvider<B>> {
        assert!(is_software, "CPU zero-copy requires software shared images");
        assert_eq!(
            interface.0,
            self.native.frame_sink.lock().expect("frame sink").id().0
        );
        let execution = self.native.tile_execution.lock().expect("tile execution");
        Box::new(RasterBufferProvider(Box::new(
            execution.CreateRasterBufferProvider(),
        )))
    }
    fn CreateGpuRasterBufferProvider(
        &mut self,
        _interface: HostSharedImageInterfaceId,
        _compositor: HostImplContextProviderId,
        _worker: HostImplContextProviderId,
        _overlay: bool,
        _max: crate::adapter::geometry::Size,
        _queries: Option<&RasterQueryQueue>,
    ) -> Box<RasterBufferProvider<B>> {
        panic!("CPU source branch never selects GPU raster")
    }
    fn MissingWorkerContextForcingZeroCopy(&self) {
        self.native.events.Emit(
            "cc",
            "Missing worker context; forcing zero-copy raster".into(),
        );
    }
    fn CreateOneCopyRasterBufferProvider(
        &mut self,
        _interface: HostSharedImageInterfaceId,
        _runner: HostImplTaskRunnerId,
        _compositor: HostImplContextProviderId,
        _worker: HostImplContextProviderId,
        _partial: bool,
        _max: i32,
        _overlay: bool,
    ) -> Box<RasterBufferProvider<B>> {
        panic!("CPU source branch never selects GPU one-copy raster")
    }
}
