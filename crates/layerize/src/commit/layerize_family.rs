//! The binding of the main half to the one concrete compositor family
//! (MAIN_IMPL_SPLIT.md, "Plan correction", option h).
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
//! value types and requires the push targets the main layers write, so that
//! `F: LayerizeFamily` alone proves every main trait through those blanket
//! impls (and their associated types normalize).
//!
//! Chromium has one concrete cc::Layer / cc::LayerImpl family; this trait is
//! the template-parameter projection of it, not a second model.
use super::{
    commit_state::CommitStateTypes,
    geometry::Size,
    layer_value_backend::MainLayerCopyOutputRequest,
    main_callbacks::TaskRunner,
    paint::{CpuFilterQuality, DynamicRangeLimit, NodeId},
    push_properties::PushPropertiesTypes,
    skia_value_backend::{MainSkiaBackend, PaintSkiaBackend},
    surface_range::SurfaceRangeTypes,
    transform::Transform,
    viz_metadata::{RegionCaptureBounds, TrackedElementRects},
};
use crate::adapter::{
    painted_scrollbar_layer::PaintedScrollbarLayerImplTarget,
    picture_layer::PictureLayerImplTarget, solid_color_layer::SolidColorLayerImplTarget,
    solid_color_scrollbar_layer::SolidColorScrollbarLayerImplTarget,
    surface_layer::SurfaceLayerImplTarget,
};

pub trait LayerizeFamily:
    PushPropertiesTypes<
        Transform = Transform<<Self as LayerizeFamily>::Skia>,
        Filters = <<Self as LayerizeFamily>::Skia as MainSkiaBackend>::Filters,
        Path = <<Self as LayerizeFamily>::Skia as MainSkiaBackend>::Path,
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
        Layer: PictureLayerImplTarget<Self>
                   + SolidColorLayerImplTarget<Self>
                   + PaintedScrollbarLayerImplTarget<Self>
                   + SolidColorScrollbarLayerImplTarget<Self>
                   + SurfaceLayerImplTarget<Self>,
    > + CommitStateTypes
    + SurfaceRangeTypes
{
    /// The external Skia value backend (the compositor's `R`).
    type Skia: PaintSkiaBackend;

    /// Source: PaintedScrollbarLayer::RasterizeScrollbarPart
    /// (cc/layers/painted_scrollbar_layer.cc). The family allocates the
    /// impl-side UIResourceBitmap from the painted thumb canvas.
    fn RasterizeScrollbarThumb(
        scrollbar: &crate::runtime::paint::Scrollbar,
        logical_size: Size,
        pixel_size: Size,
    ) -> Self::UIResourceBitmap;
}
