//! cc::ViewTransitionRequest (cc/view_transition/view_transition_request.h),
//! a shared cc owner: Blink creates it on the main thread
//! (LayerTreeHost::AddViewTransitionRequest, carried by
//! CommitState::view_transition_requests), the impl LayerTreeHostImpl reads
//! it and builds its viz::CompositorFrameTransitionDirective. The impl
//! tree / metadata facets are implemented impl-side on this owner.
use super::{
    geometry::RectF, main_callbacks::MainViewTransitionRequest,
    property_types::ViewTransitionElementResourceId,
};
use compositor_values::display_color_spaces::RasterDisplayColorSpaces;
use std::collections::{BTreeMap, HashMap};
use viz_common::{
    compositor_frame_metadata::TransitionDirective, draw_quad::RenderPassId, ViewTransitionToken,
};

/// ViewTransitionRequest::ViewTransitionElementResourceRects
/// (view_transition_request.h:33-34).
pub type ViewTransitionElementResourceRects = HashMap<ViewTransitionElementResourceId, RectF>;
/// ViewTransitionRequest::ViewTransitionCaptureCallback
/// (view_transition_request.h:35-36).
pub type ViewTransitionCaptureCallback =
    Box<dyn FnOnce(&ViewTransitionElementResourceRects) + Send + Sync>;

/// The native request owns its callback and capture parameters. Calls only
/// expose the request API cc consumes (view_transition_request.h:38-104).
pub trait ViewTransitionRequestBackend {
    fn IsSave(&self) -> bool;
    fn IsAnimateRenderer(&self) -> bool;
    fn token(&self) -> ViewTransitionToken;
    fn sequence_id(&self) -> u32;
    fn delay_layer_tree_view_deletion(&self) -> bool;
    // Source: view_transition_request.h:35-36,70-72,89-91.
    fn TakeFinishedCallback(&mut self) -> Option<ViewTransitionCaptureCallback>;
    fn maybe_cross_frame_sink(&self) -> bool;
    fn SetContentRect(&self, id: ViewTransitionElementResourceId, rect: RectF);
    // Source: view_transition_request.h:80-84 ConstructDirective.
    fn ConstructDirective(
        &self,
        elements: &BTreeMap<ViewTransitionElementResourceId, RenderPassId>,
        colors: &RasterDisplayColorSpaces,
        delay: bool,
    ) -> TransitionDirective;
}

pub struct ViewTransitionRequest(pub Box<dyn ViewTransitionRequestBackend + Send + Sync>);
impl ViewTransitionRequest {
    pub fn ConstructDirective(
        &self,
        elements: &BTreeMap<ViewTransitionElementResourceId, RenderPassId>,
        colors: &RasterDisplayColorSpaces,
        delay: bool,
    ) -> TransitionDirective {
        self.0.ConstructDirective(elements, colors, delay)
    }
}

// The main LayerTreeHost's view of the request (runtime/main_host.rs).
impl MainViewTransitionRequest for ViewTransitionRequest {
    type ElementResourceRects = ViewTransitionElementResourceRects;
    type FinishedCallback = ViewTransitionCaptureCallback;
    fn sequence_id(&self) -> u32 {
        self.0.sequence_id()
    }
    fn token(&self) -> [u64; 2] {
        self.0.token()
    }
    fn IsSave(&self) -> bool {
        self.0.IsSave()
    }
    fn IsAnimateRenderer(&self) -> bool {
        self.0.IsAnimateRenderer()
    }
    fn maybe_cross_frame_sink(&self) -> bool {
        self.0.maybe_cross_frame_sink()
    }
    fn TakeFinishedCallback(&mut self) -> Option<ViewTransitionCaptureCallback> {
        self.0.TakeFinishedCallback()
    }
}
