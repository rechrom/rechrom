//! The commit family: the one concrete cc type set the main half
//! (LayerizeEngine, ProxyMain, cc main layers) and the commit vocabulary
//! (CommitState, ThreadUnsafeCommitState, the Proxy messages) are
//! instantiated over.
//!
//! Chromium has one cc::Layer / CommitState / Proxy message definition,
//! named by both threads (cc/trees/commit_state.h, cc/trees/proxy_main.h,
//! cc/trees/proxy_impl.h), over concrete value types (gfx::Transform,
//! cc::FilterOperations, viz::SurfaceRange, ...). This type is that set. The
//! impl family (the compositor's LayerImpl family) names it as its `Main`;
//! the bounds of that associated type are the single place that equates the
//! main values with the impl family's, so the impl reads a moved
//! `CommitState<CommitFamily<R>>` while the main half names no impl type.
//! `R` is the Skia value backend.
use super::{
    animation_host::AnimationHostOwner,
    commit_state::{CommitStateTypes, CommitTypes},
    geometry::Size,
    layerize_family::LayerizeFamily,
    paint::{CpuFilterQuality, DrawImage, DynamicRangeLimit, NodeId},
    presentation_time_callback::{PresentationCallback, SuccessfulPresentationCallback},
    proxy::{ProxyImplInit, ProxyTypes},
    push_properties::PushPropertiesTypes,
    skia_value_backend::{MainSkiaBackend, PaintSkiaBackend},
    surface_range::SurfaceRangeTypes,
    swap_promise::SwapPromise,
    transform::Transform,
    ui_resource::{HostUIResourceBitmap, UIBitmap, UIResourceRequest},
    values::{UIResourceId, UIResourceRequestType},
    view_transition_request::ViewTransitionRequest,
    viz_metadata::{RegionCaptureBounds, TrackedElementRects},
};
use crate::cc::layers::layer::{LayerBorrow, LayerRef};
use compositor_values::surface_id::{LocalSurfaceId, SurfaceId, SurfaceRange};
use std::marker::PhantomData;

/// The concrete commit family over the Skia value backend `R`.
pub struct CommitFamily<R: PaintSkiaBackend>(PhantomData<fn() -> R>);

/// The production family over the CPU Skia region backend.
pub type MainFamily = CommitFamily<super::skia_region::SkRegion>;

impl<R: PaintSkiaBackend> CommitTypes for CommitFamily<R> {
    type Transform = Transform<R>;
    type RegionBackend = R;
    type UIResourceBitmap = UIBitmap;
    type Filters = <R as MainSkiaBackend>::Filters;
    type Path = <R as cc_paint::skia_backend::RecordSkiaBackend>::Path;
    type CopyOutputRequest = viz_common::copy_output_request::CopyOutputRequest;
    type DisplayColorSpaces = compositor_values::display_color_spaces::RasterDisplayColorSpaces;
    type LocalSurfaceId = LocalSurfaceId;
    type SurfaceRange = SurfaceRange;
    type SwapPromise = SwapPromise;
    type CompositorFrameMetadata = viz_common::compositor_frame_metadata::CompositorFrameMetadata<
        viz_common::SoftwareMetadataValues,
    >;
    type UIResourceRequest = UIResourceRequest;
    type PresentationCallback = PresentationCallback;
    type SuccessfulPresentationCallback = SuccessfulPresentationCallback;
    type EventMetrics = compositor_values::event_metrics::EventMetrics;
    type DelegatedInkMetadata = viz_common::compositor_frame_metadata::DelegatedInkMetadata;
    type ViewTransitionRequest = ViewTransitionRequest;
}

impl<R: PaintSkiaBackend> PushPropertiesTypes for CommitFamily<R> {
    type NodeId = NodeId;
    type RegionCaptureBounds = RegionCaptureBounds;
    type TrackedElementRects = TrackedElementRects;
    type FilterQuality = CpuFilterQuality;
    type DynamicRangeLimit = DynamicRangeLimit;
}

impl<R: PaintSkiaBackend> CommitStateTypes for CommitFamily<R> {
    type BeginMainFrameMetrics = compositor_values::begin_main_frame_metrics::BeginMainFrameMetrics;
    type DrawImage = DrawImage;
    type MainRootLayer = LayerRef<Self>;
    type MainLayerRef = LayerBorrow<Self>;
    // The main AnimationHost itself (commit_state.h:205 raw_ptr<MutatorHost>);
    // its MutatorEvents are LayerTreeHostImpl::TakeMutatorEvents' value.
    type MutatorHostHandle = AnimationHostOwner;
}

impl<R: PaintSkiaBackend> SurfaceRangeTypes for CommitFamily<R> {
    fn DefaultSurfaceRange() -> SurfaceRange {
        Default::default()
    }
    fn SurfaceRangesEqual(a: &SurfaceRange, b: &SurfaceRange) -> bool {
        a == b
    }
    // Source: components/viz/common/surfaces/surface_range.cc IsValid.
    fn SurfaceRangeIsValid(range: &SurfaceRange) -> bool {
        let valid = |id: &SurfaceId| {
            id.frame_sink_id.client_id != 0
                && id.frame_sink_id.sink_id != 0
                && id.local_surface_id.parent_sequence_number != 0
                && id.local_surface_id.child_sequence_number != 0
                && id.local_surface_id.embed_token != [0, 0]
        };
        valid(&range.end) && range.start.as_ref().is_none_or(valid)
    }
}

impl<R: PaintSkiaBackend> LayerizeFamily for CommitFamily<R> {
    type Skia = R;
    // Source: cc/resources/ui_resource_bitmap.cc UIResourceBitmap(SkBitmap).
    fn CreateUIResourceBitmap(
        pixel_size: Size,
        pixels: Vec<u8>,
        row_bytes: usize,
        opaque: bool,
    ) -> UIBitmap {
        UIBitmap::FromPixels(pixel_size, pixels, row_bytes, opaque)
    }
    // Source: cc/resources/ui_resource_request.h:17-56.
    fn CreateUIResourceRequest(
        kind: UIResourceRequestType,
        id: UIResourceId,
        bitmap: UIBitmap,
    ) -> UIResourceRequest {
        UIResourceRequest { kind, id, bitmap }
    }
    fn UIResourceBitmapSize(bitmap: &UIBitmap) -> Size {
        bitmap.GetSize()
    }
}

// The payloads of the ProxyMain/ProxyImpl messages. The queued DrawImage is
// not posted yet and stays a unit value.
impl<R: PaintSkiaBackend> ProxyTypes for CommitFamily<R> {
    // BeginMainFrameAbortedOnImpl's std::vector<std::unique_ptr<SwapPromise>>.
    type ProxySwapPromise = Box<SwapPromise>;
    // ProxyMain::SetMutator (cc/trees/proxy_main.cc:959-965).
    type LayerTreeMutator = compositor_values::layer_tree_mutator::LayerTreeMutator;
    // ProxyMain::InitializeOnImplThread (proxy_main.cc:80-89).
    type ImplInit = Box<ProxyImplInit>;
    type ProxyDrawImage = ();
}
