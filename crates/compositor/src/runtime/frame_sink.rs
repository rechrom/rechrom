//! Injected native frame transport. The wrapper is the actual external sink
//! allocation; it contains no Host/Page reference and creates no window or Viz.
use super::{
    frame::{CompositorFrame, CompositorRenderPass, FrameValueTypes},
    resources::{CpuReturnedResource, CpuTransferableResource},
};
use crate::adapter::{
    geometry::Size,
    unbounded::{CompositorFrame as NativeFrame, FrameSink as NativeFrameSink, UnboundedTypes},
};

/// Native Bind registers only the source client identity. Future callbacks are
/// delivered by the Page with a fresh live owner context; this object never
/// stores a callable child-to-owner pointer.
pub struct FrameSink<B: UnboundedTypes> {
    native: Box<dyn NativeFrameSink<B>>,
}
impl<B: UnboundedTypes> FrameSink<B> {
    pub fn new(native: Box<dyn NativeFrameSink<B>>) -> Self {
        Self { native }
    }
}
impl<B: UnboundedTypes> NativeFrameSink<B> for FrameSink<B> {
    fn BindToClient(&mut self, client: B::ClientIdentity) -> bool {
        self.native.BindToClient(client)
    }
    fn SetLocalSurfaceId(&mut self, id: &B::LocalSurfaceId) {
        self.native.SetLocalSurfaceId(id)
    }
    fn DetachFromClient(&mut self) {
        self.native.DetachFromClient()
    }
    fn SubmitCompositorFrame(&mut self, frame: B::Frame, hit_test_data_changed: bool) {
        self.native
            .SubmitCompositorFrame(frame, hit_test_data_changed)
    }
}
impl<M, V: FrameValueTypes> NativeFrame<CpuReturnedResource>
    for CompositorFrame<M, CompositorRenderPass<V>, CpuTransferableResource>
{
    fn render_pass_list_is_empty(&self) -> bool {
        self.render_pass_list.is_empty()
    }
    fn SetLastRenderPassOutputSize(&mut self, size: Size) {
        let last = self
            .render_pass_list
            .last_mut()
            .expect("native nonempty render pass list");
        last.output_rect.width = size.width();
        last.output_rect.height = size.height();
    }
    fn size_in_pixels(&self) -> Size {
        self.render_pass_list
            .last()
            .expect("native nonempty render pass list")
            .output_rect
            .size()
    }
    // Source dependency: cc/trees/unbounded_frame_sink_handler.cc:71-76;
    // components/viz/common/resources/transferable_resource.cc:44-50. Each actual transfer's EndImport is used.
    fn ReturnResources(&self) -> Vec<CpuReturnedResource> {
        self.resource_list
            .iter()
            .map(CpuTransferableResource::ToReturnedResource)
            .collect()
    }
}

/// Independent transport for complete, move-only native frames. The queue owns
/// only submitted output, including its actual resource leases; it never owns
/// a compositor, tree, scheduler or a callback into any producer.
pub(crate) struct FrameOutput<F>(std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<F>>>);
impl<F> Clone for FrameOutput<F> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<F> Default for FrameOutput<F> {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<F> FrameOutput<F> {
    pub(crate) fn Submit(&self, frame: F) {
        self.0
            .lock()
            .expect("native frame transport")
            .push_back(frame);
    }
    pub(crate) fn Take(&self) -> Option<F> {
        self.0.lock().expect("native frame transport").pop_front()
    }
    pub(crate) fn IsEmpty(&self) -> bool {
        self.0.lock().expect("native frame transport").is_empty()
    }
}

/// CPU frame transport endpoint. Binding retains only the native identity, and
/// Submit transfers the original frame allocation into the output transport.
/// There is no alternate frame representation or separate pixel bundle.
pub(crate) struct OutputFrameSink {
    client: Option<super::host_types::UnboundedClientIdentity>,
    local_surface_id: super::types::LocalSurfaceId,
    output: FrameOutput<crate::NativeCompositorFrameSubmission>,
}
impl OutputFrameSink {
    pub(crate) fn new(output: FrameOutput<crate::NativeCompositorFrameSubmission>) -> Self {
        Self {
            client: None,
            local_surface_id: Default::default(),
            output,
        }
    }
}
impl NativeFrameSink<crate::NativeCompositorTypes> for OutputFrameSink {
    fn BindToClient(&mut self, client: super::host_types::UnboundedClientIdentity) -> bool {
        assert!(self.client.is_none(), "native frame sink already bound");
        assert_ne!(client.0, 0, "native client identity is non-null");
        self.client = Some(client);
        true
    }
    fn SetLocalSurfaceId(&mut self, id: &super::types::LocalSurfaceId) {
        self.local_surface_id = *id;
    }
    fn DetachFromClient(&mut self) {
        self.client = None;
    }
    fn SubmitCompositorFrame(
        &mut self,
        frame: crate::NativeCompositorFrame,
        _hit_test_data_changed: bool,
    ) {
        assert!(
            self.client.is_some(),
            "native frame submission requires a bound client"
        );
        self.output.Submit(crate::NativeCompositorFrameSubmission {
            local_surface_id: self.local_surface_id,
            frame,
        });
    }
}
