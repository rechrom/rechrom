// Translation status: complete
// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/unbounded_frame_sink_handler.h:1-73;
//         cc/trees/unbounded_frame_sink_handler.cc:1-15.
use super::managed_memory_policy::ManagedMemoryPolicy;
use crate::adapter::{
    geometry::{Rect, Size},
    unbounded::*,
};
// Source: cc/trees/unbounded_frame_sink_handler.h:27-69.
pub struct UnboundedFrameSinkHandler<B: UnboundedTypes> {
    frame_sink_: Option<Box<B::FrameSink>>,
    local_surface_id_: B::LocalSurfaceId,
    last_submitted_local_surface_id_: B::LocalSurfaceId,
    last_submitted_size_in_pixels_: Size,
    // Rust's stable identity for the C++ `this` used when binding callbacks.
    client_identity_: B::ClientIdentity,
}
impl<B: UnboundedTypes> UnboundedFrameSinkHandler<B> {
    // Source: cc/trees/unbounded_frame_sink_handler.cc:17-21.
    // The caller keeps the source enable_unbounded_element assertion at the
    // owning HostImpl boundary; host-dependent calls receive a live borrow.
    pub fn new(client_identity: B::ClientIdentity) -> Self {
        Self {
            frame_sink_: None,
            local_surface_id_: B::LocalSurfaceId::default(),
            last_submitted_local_surface_id_: B::LocalSurfaceId::default(),
            last_submitted_size_in_pixels_: Size::default(),
            client_identity_: client_identity,
        }
    }
    // Source: cc/trees/unbounded_frame_sink_handler.cc:27-36.
    pub fn SetFrameSink(&mut self, sink: Option<Box<B::FrameSink>>, id: &B::LocalSurfaceId) {
        self.frame_sink_ = sink;
        self.local_surface_id_ = id.clone();
        if let Some(sink) = &mut self.frame_sink_ {
            let _ = sink.BindToClient(self.client_identity_);
            sink.SetLocalSurfaceId(id);
        }
    }
    // Source: cc/trees/unbounded_frame_sink_handler.cc:38-46.
    pub fn DismissFrameSink(&mut self) {
        if let Some(sink) = &mut self.frame_sink_ {
            sink.DetachFromClient();
            self.frame_sink_ = None;
        }
        self.local_surface_id_ = B::LocalSurfaceId::default();
        self.last_submitted_local_surface_id_ = B::LocalSurfaceId::default();
        self.last_submitted_size_in_pixels_ = Size::default();
    }
    // Source: cc/trees/unbounded_frame_sink_handler.cc:48-54.
    pub fn SetLocalSurfaceId(&mut self, id: &B::LocalSurfaceId) {
        self.local_surface_id_ = id.clone();
        if let Some(sink) = &mut self.frame_sink_ {
            sink.SetLocalSurfaceId(id);
        }
    }
    // Source: cc/trees/unbounded_frame_sink_handler.h:37-40.
    pub fn HasUnsubmittedLocalSurfaceId(&self) -> bool {
        self.local_surface_id_.is_valid()
            && self.local_surface_id_ != self.last_submitted_local_surface_id_
    }
    // Source: cc/trees/unbounded_frame_sink_handler.cc:56-77.
    pub fn SubmitFrame(
        &mut self,
        mut frame: B::Frame,
        host: &mut (impl UnboundedResourceHost<B> + ?Sized),
    ) {
        if self.frame_sink_.is_some() && self.local_surface_id_.is_valid() {
            if self.local_surface_id_ == self.last_submitted_local_surface_id_
                && !self.last_submitted_size_in_pixels_.IsEmpty()
                && !frame.render_pass_list_is_empty()
            {
                frame.SetLastRenderPassOutputSize(self.last_submitted_size_in_pixels_);
            }
            self.last_submitted_size_in_pixels_ = frame.size_in_pixels();
            self.last_submitted_local_surface_id_ = self.local_surface_id_.clone();
            self.frame_sink_
                .as_mut()
                .unwrap()
                .SubmitCompositorFrame(frame, false);
        } else {
            let resources = frame.ReturnResources();
            if !resources.is_empty() {
                self.ReclaimResources(resources, host);
            }
        }
    }
    // Source: cc/trees/unbounded_frame_sink_handler.cc:79-82.
    pub fn BuildHitTestData(&self) -> Option<B::HitTestRegionList> {
        None
    }
    // Source: cc/trees/unbounded_frame_sink_handler.cc:84-88.
    pub fn ReclaimResources(
        &mut self,
        resources: Vec<B::ReturnedResource>,
        host: &mut (impl UnboundedResourceHost<B> + ?Sized),
    ) {
        host.ReceiveReturnsFromParent(resources);
    }
    // Source: cc/trees/unbounded_frame_sink_handler.cc:90-92.
    pub fn DidLoseLayerTreeFrameSink(&mut self, host: &mut (impl UnboundedHost<B> + ?Sized)) {
        host.DismissUnboundedFrameSink();
    }
    // Source: cc/trees/unbounded_frame_sink_handler.h:45,48-60.
    // Empty virtual overrides still drop their by-value callback arguments.
    pub fn SetBeginFrameSource(&mut self, _source: Option<&B::BeginFrameSource>) {}
    pub fn SetTreeActivationCallback(&mut self, _callback: B::RepeatingClosure) {}
    pub fn DidReceiveCompositorFrameAck(&mut self) {}
    pub fn DidPresentCompositorFrame(
        &mut self,
        _frame_token: u32,
        _details: &B::FrameTimingDetails,
    ) {
    }
    pub fn OnDraw(
        &mut self,
        _transform: &B::Transform,
        _viewport: &Rect,
        _resourceless_software_draw: bool,
        _skip_draw: bool,
    ) {
    }
    pub fn SetMemoryPolicy(&mut self, _policy: &ManagedMemoryPolicy) {}
    pub fn SetExternalTilePriorityConstraints(
        &mut self,
        _viewport: &Rect,
        _transform: &B::Transform,
    ) {
    }
}
// Source: cc/trees/unbounded_frame_sink_handler.cc:23-25.
impl<B: UnboundedTypes> Drop for UnboundedFrameSinkHandler<B> {
    fn drop(&mut self) {
        self.DismissFrameSink();
    }
}
