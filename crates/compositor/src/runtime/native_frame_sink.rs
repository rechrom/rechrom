//! Page-local native frame-sink endpoint for the CPU compositor.
//!
//! Chromium correspondence:
//! - cc/trees/layer_tree_frame_sink.cc:24-50
//! - cc/mojo_embedder/async_layer_tree_frame_sink.cc:128-134,162
//! - components/viz/client/client_resource_provider.cc:275-407
//!
//! A submitted frame is moved once into `FrameOutput`. The endpoint and the
//! ResourcePool share one `RasterClientResources` allocation, so there is one
//! import/export table and no resource snapshot or parallel bundle.

use super::{
    client_resources::RasterClientResources,
    frame_sink::{FrameOutput, FrameSink, OutputFrameSink},
    host_owners::ClientResourceProvider,
    host_types::UnboundedClientIdentity,
    types::LocalSurfaceId,
};
use crate::{
    adapter::{
        layer_tree_host_impl::{
            HostFrameSinkClientFacade, HostFrameSinkOperations, HostImplFrameSinkId, HostImplWeak,
        },
        unbounded::FrameSink as NativeFrameSink,
    },
    NativeCompositorFrame, NativeCompositorFrameSubmission, NativeCompositorTypes,
};
use std::sync::Arc;
use std::sync::Mutex;

type BeginFrameSource = Arc<dyn crate::adapter::scheduler_host::BeginFrameSource>;

pub(crate) struct NativeFrameSinkEndpoint {
    id: HostImplFrameSinkId,
    sink: FrameSink<NativeCompositorTypes>,
    output: FrameOutput<NativeCompositorFrameSubmission>,
    resources: RasterClientResources,
    begin_frame_source: Option<BeginFrameSource>,
    source_frame_number: i32,
    current_local_surface_id: LocalSurfaceId,
}

impl NativeFrameSinkEndpoint {
    pub(crate) fn new(id: HostImplFrameSinkId, resources: RasterClientResources) -> Self {
        assert_ne!(id.0, 0, "native frame-sink identity is non-null");
        let output = FrameOutput::default();
        let sink = FrameSink::new(Box::new(OutputFrameSink::new(output.clone())));
        Self {
            id,
            sink,
            output,
            resources,
            begin_frame_source: None,
            source_frame_number: 0,
            current_local_surface_id: LocalSurfaceId::default(),
        }
    }

    pub(crate) fn id(&self) -> HostImplFrameSinkId {
        self.id
    }

    pub(crate) fn SetBeginFrameSource(&mut self, source: Option<BeginFrameSource>) {
        self.begin_frame_source = source;
    }

    pub(crate) fn LayerTreeFrameSink(&self) -> &FrameSink<NativeCompositorTypes> {
        &self.sink
    }

    pub(crate) fn SetSourceFrameNumber(&mut self, number: i32) {
        self.source_frame_number = number;
    }

    pub(crate) fn source_frame_number(&self) -> i32 {
        self.source_frame_number
    }

    pub(crate) fn SetCurrentLocalSurfaceId(&mut self, id: &LocalSurfaceId) {
        self.current_local_surface_id = *id;
        self.sink.SetLocalSurfaceId(id);
    }

    pub(crate) fn current_local_surface_id(&self) -> LocalSurfaceId {
        self.current_local_surface_id
    }

    pub(crate) fn SubmitCompositorFrame(
        &mut self,
        frame: NativeCompositorFrame,
        hit_test_data_changed: bool,
    ) {
        self.sink
            .SubmitCompositorFrame(frame, hit_test_data_changed);
    }

    pub(crate) fn TakeFrame(&self) -> Option<NativeCompositorFrameSubmission> {
        self.output.Take()
    }

    pub(crate) fn HasFrame(&self) -> bool {
        !self.output.IsEmpty()
    }

    pub(crate) fn CreateClientResourceProvider(
        &self,
    ) -> ClientResourceProvider<NativeCompositorTypes> {
        ClientResourceProvider(Box::new(self.resources.clone()))
    }

    pub(crate) fn resources(&self) -> &RasterClientResources {
        &self.resources
    }
}

impl HostFrameSinkOperations<NativeCompositorTypes> for NativeFrameSinkEndpoint {
    fn BindToClient(
        &mut self,
        target: HostImplWeak,
        client: &mut dyn HostFrameSinkClientFacade<NativeCompositorTypes>,
    ) -> bool {
        if !target.IsValid() {
            return false;
        }
        if !self
            .sink
            .BindToClient(UnboundedClientIdentity(target.owner.0))
        {
            return false;
        }
        client.SetBeginFrameSource(self.begin_frame_source.as_ref());
        true
    }

    fn DetachFromClient(
        &mut self,
        client: &mut dyn HostFrameSinkClientFacade<NativeCompositorTypes>,
    ) {
        client.SetBeginFrameSource(None);
        self.sink.DetachFromClient();
    }
}

/// Borrowable operation facet for `HostFrameSinkAccess`. Metadata submission
/// and bind/detach lock the same endpoint allocation; no second sink exists.
pub(crate) struct NativeFrameSinkOperationsHandle(pub(crate) Arc<Mutex<NativeFrameSinkEndpoint>>);
impl NativeFrameSinkOperationsHandle {
    pub(crate) fn new(endpoint: Arc<Mutex<NativeFrameSinkEndpoint>>) -> Self {
        Self(endpoint)
    }
}
impl HostFrameSinkOperations<NativeCompositorTypes> for NativeFrameSinkOperationsHandle {
    fn BindToClient(
        &mut self,
        target: HostImplWeak,
        client: &mut dyn HostFrameSinkClientFacade<NativeCompositorTypes>,
    ) -> bool {
        self.0
            .lock()
            .expect("native frame-sink endpoint")
            .BindToClient(target, client)
    }

    fn DetachFromClient(
        &mut self,
        client: &mut dyn HostFrameSinkClientFacade<NativeCompositorTypes>,
    ) {
        self.0
            .lock()
            .expect("native frame-sink endpoint")
            .DetachFromClient(client)
    }
}
