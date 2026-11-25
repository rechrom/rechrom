//! The binding of the main half to the one concrete compositor family.
//!
//! The family type stays in the compositor, because it also implements impl
//! traits of the compositor sub-crates (orphan rule). The compositor
//! implements this trait for it. Main traits that move into layerize are
//! implemented by blanket impls over `F: LayerizeFamily`
//! (`runtime/main_family.rs`), and main code that named the concrete family
//! becomes generic over `F`, using `F::Skia` where it used the Skia backend
//! parameter `R`.
//!
//! The supertrait fixes the commit values the main half names to its own
//! value types, so that
//! `F: LayerizeFamily` alone proves every main trait through those blanket
//! impls (and their associated types normalize).
//!
//! Chromium has one concrete cc::Layer / cc::LayerImpl family; this trait is
//! the template-parameter projection of it, not a second model.
use super::{
    commit_state::CommitStateTypes,
    geometry::Size,
    layer_value_backend::MainLayerCopyOutputRequest,
    main_callbacks::{MainViewTransitionRequest, TaskRunner},
    main_effects::LocalSurfaceId,
    values::{UIResourceId, UIResourceRequestType},
    paint::{CpuFilterQuality, DynamicRangeLimit, NodeId},
    push_properties::PushPropertiesTypes,
    skia_value_backend::{MainSkiaBackend, PaintSkiaBackend},
    surface_range::SurfaceRangeTypes,
    transform::Transform,
    viz_metadata::{RegionCaptureBounds, TrackedElementRects},
};

pub trait LayerizeFamily:
    PushPropertiesTypes<
        Transform = Transform<<Self as LayerizeFamily>::Skia>,
        Filters = <<Self as LayerizeFamily>::Skia as MainSkiaBackend>::Filters,
        Path = <<Self as LayerizeFamily>::Skia as cc_paint::skia_backend::RecordSkiaBackend>::Path,
        RegionBackend = <Self as LayerizeFamily>::Skia,
        CopyOutputRequest: MainLayerCopyOutputRequest<TaskRunner = TaskRunner>
                               + Send
                               + Sync
                               + 'static,
        NodeId = NodeId,
        RegionCaptureBounds = RegionCaptureBounds,
        TrackedElementRects = TrackedElementRects,
        FilterQuality = CpuFilterQuality,
        DynamicRangeLimit = DynamicRangeLimit,
        LocalSurfaceId = LocalSurfaceId,
        ViewTransitionRequest: MainViewTransitionRequest,
    > + CommitStateTypes<
        MainRootLayer = crate::cc::layers::layer::LayerRef<Self>,
        MainLayerRef = crate::cc::layers::layer::LayerBorrow<Self>,
    > + SurfaceRangeTypes
{
    /// The external Skia value backend (the compositor's `R`).
    type Skia: PaintSkiaBackend;

    /// Source: cc/resources/ui_resource_bitmap.h UIResourceBitmap(SkBitmap).
    /// The impl family allocates its UIResourceBitmap from the pixels the
    /// main half painted (PaintedScrollbarLayer::RasterizeScrollbarPart,
    /// cc/layers/painted_scrollbar_layer.cc).
    fn CreateUIResourceBitmap(
        pixel_size: Size,
        pixels: Vec<u8>,
        row_bytes: usize,
        opaque: bool,
    ) -> Self::UIResourceBitmap;

    /// Source: cc/resources/ui_resource_request.h:17-56. The impl family owns
    /// the request and bitmap types LayerTreeHost::CreateUIResource /
    /// DeleteUIResource queue (cc/trees/layer_tree_host.cc).
    fn CreateUIResourceRequest(
        kind: UIResourceRequestType,
        id: UIResourceId,
        bitmap: Self::UIResourceBitmap,
    ) -> Self::UIResourceRequest;
    /// UIResourceBitmap::GetSize.
    fn UIResourceBitmapSize(bitmap: &Self::UIResourceBitmap) -> Size;
    /// The empty native bitmap a DELETE request carries (its payload is not
    /// read by LayerTreeHostImpl for that request type).
    fn EmptyUIResourceBitmap() -> Self::UIResourceBitmap {
        Self::CreateUIResourceBitmap(Size::default(), Vec::new(), 0, false)
    }
}
