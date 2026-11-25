//! The main-half traits of the one concrete compositor family, implemented
//! once for every `F: LayerizeFamily`.
//! Moved from compositor/src/runtime/{picture_types,owner_types}.rs, where
//! they were impls for `CpuCompositorTypes<R>`; `F::Skia` is that `R`.
//!
//! There is exactly one family, so these blanket impls never overlap, and
//! `<F as MainTrait>::Item` normalizes through them for every caller that
//! only knows `F: LayerizeFamily`.
use super::{
    geometry::Transform,
    metadata,
    paint,
    types::NodeId,
};
use crate::{
    adapter::{
        geometry::Size,
        input_types::ElementId,
        main_layer::MainLayerTypes,
        paint_artifact_compositor::{PaintArtifactCompositorTypes, SynthesizedClipTypes},
        paint_chunks_to_cc_layer::PaintChunksTypes,
        painted_scrollbar_layer::PaintedScrollbarLayerTypes,
        pending_layer::PendingLayerTypes,
        picture_layer::PictureLayerTypes,
        property_tree_manager::PropertyTreeManagerTypes,
        scrollbar_layer_base::{ScrollbarConstructor, ScrollbarLayerTypes},
        solid_color_layer::SolidColorLayerTypes,
        solid_color_scrollbar_layer::SolidColorScrollbarLayerTypes,
    },
    commit::{layerize_family::LayerizeFamily, main_callbacks},
};

type Sk<F> = <F as LayerizeFamily>::Skia;

impl<F: LayerizeFamily> PaintArtifactCompositorTypes for F {
    type CanvasSnapshotCallback = std::sync::Arc<dyn paint::CpuCanvasSnapshot<Sk<F>>>;
    type CompositingReasons = paint::CpuCompositingReasons;
    type LayersAsJson = paint::CpuLayersAsJson<Self>;
    type CrashKeyScope = paint::PaintTraceScope;
    type UpdateTraceScope = paint::PaintTraceScope;
}
impl<F: LayerizeFamily> PaintChunksTypes for F {
    type ChunkMapper = paint::CpuGeometryMapper<Sk<F>>;
    type RoundedRect = crate::adapter::geometry::RRectF;
    type PaintPath = std::sync::Arc<::paint::paint_clip_path::PaintClipPath>;
    type ScrollNode = paint::ScrollNode<Sk<F>>;
    type PaintFlags = paint::CpuPaintFlags<Sk<F>>;
    type PaintFilter = paint::CpuPaintFilter;
    type FilterOperations = ::paint::cc::paint::filter_operations::FilterOperations;
    type HitTestData = ::paint::paint_engine::PaintHitTestData;
    type RegionCaptureData = Vec<([u64; 2], ::paint::paint_engine::PaintRect)>;
    type TrackedData = Vec<::paint::paint_engine::PaintTrackedElementRect>;
    type SelectionData = ::paint::paint_engine::PaintLayerSelectionData;
    type CaptureId = metadata::RegionCaptureCropId;
    type TrackingFeature = metadata::TrackedElementFeature;
    type TrackedId = [u64; 2];
    type FrameToken = Option<metadata::FrameToken>;
    type PaintRecorder = paint::CpuConversionRecorder<Sk<F>>;
    type UnderInvalidationPicture = paint::CpuPaintRecord<Sk<F>>;
}
impl<F: LayerizeFamily> SynthesizedClipTypes for F {
    type SkRoundedRect = crate::commit::skia_region::SkRoundedRect;
}
impl<F: LayerizeFamily> PropertyTreeManagerTypes for F {
    fn RootTransformNode() -> paint::TransformNode<Sk<F>> {
        paint::PropertyTreeState::<Sk<F>>::default().transform()
    }
    fn RootClipNode() -> paint::ClipNode<Sk<F>> {
        paint::PropertyTreeState::<Sk<F>>::default().clip()
    }
    fn RootEffectNode() -> paint::EffectNode<Sk<F>> {
        paint::PropertyTreeState::<Sk<F>>::default().effect()
    }
    fn RootScrollNode() -> paint::ScrollNode<Sk<F>> {
        paint::ScrollNode::from_arc(paint_model::property_cell::with_access(|a| {
            ::paint::paint_property_tree::PropertyTreeState::default()
                .transform
                .ScrollNode(a)
                .expect("native ScrollRoot")
                .clone()
        }))
    }
    fn ScrollNodeForTransform(
        t: &paint::TransformNode<Sk<F>>,
    ) -> Option<paint::ScrollNode<Sk<F>>> {
        paint_model::property_cell::with_access(|a| {
            t.node()
                .ScrollNode(a)
                .map(|s| paint::ScrollNode::from_arc(s.clone()))
        })
    }
    fn UniformScaleTransform(scale: f32) -> Transform<Sk<F>> {
        let mut t = Transform::default();
        t.transform_mut().Scale(scale, scale);
        t
    }
    fn SourceToDestinationProjection(
        s: &paint::TransformNode<Sk<F>>,
        d: &paint::TransformNode<Sk<F>>,
    ) -> Transform<Sk<F>> {
        crate::third_party::blink::renderer::platform::graphics::paint::geometry_mapper::GeometryMapper::<paint::CpuGeometryMapper<Sk<F>>>::SourceToDestinationProjection(s,d)
    }
    // Source: platform/graphics/compositor_element_id.cc:41-49. Replace only
    // namespace bits, preserving the original unique object identity.
    fn EffectMaskElementId(effect: ElementId) -> ElementId {
        let n = foundation::CompositorElementIdNamespace::kEffectMask as u64;
        let bits = foundation::compositor_element_id::kCompositorNamespaceBitCount
            + foundation::compositor_element_id::kCompositorReservedBitCount;
        ElementId::new(
            (effect.GetInternalValue() & !((1u64 << bits) - 1))
                | (n << foundation::compositor_element_id::kCompositorReservedBitCount),
        )
    }
    fn NewUniqueElementId() -> ElementId {
        ElementId::new(
            foundation::CompositorElementIdFromUniqueObjectId(
                foundation::NewUniqueObjectId(),
                foundation::CompositorElementIdNamespace::kUniqueObjectId,
            )
            .GetInternalValue(),
        )
    }
}
impl<F: LayerizeFamily> PendingLayerTypes for F {
    type ContentDisplayItemList = paint::ContentDisplayItemList<Sk<F>>;
    type RasterInvalidator = paint::CpuRasterInvalidator<Sk<F>>;
    type RasterInvalidationTracking=crate::third_party::blink::renderer::platform::graphics::paint::raster_invalidation_tracking::RasterInvalidationTracking<paint::CpuRasterTrackingTypes<Sk<F>>>;
    type PaintRecord = paint::CpuPaintRecord<Sk<F>>;
    type PaintOpBuffer = paint::CpuPaintOpBuffer<Sk<F>>;
    type Json = serde_json::Value;
    type JsonArray = Vec<serde_json::Value>;
    type LoggingCanvas = paint::CpuLoggingCanvas<Sk<F>>;
    type PaintArtifact = paint::PaintArtifact;
    type PaintChunk = paint::PaintChunk;
    type DisplayItem = paint::DisplayItem;
    type PaintChunkSubset = paint::PaintChunkSubset;
    type PropertyTreeState = paint::PropertyTreeState<Sk<F>>;
    type TransformNode = paint::TransformNode<Sk<F>>;
    type ClipNode = paint::ClipNode<Sk<F>>;
    type EffectNode = paint::EffectNode<Sk<F>>;
    type PropertyNodeIdentity = usize;
    type PaintPropertyChange = paint::PaintPropertyChange;
    type DomNodeId = NodeId;
    type ClientId = usize;
    type CanvasChildPaintRecord = paint::CpuCanvasChildPaintRecord<Sk<F>>;
    type CanvasChildPaintState =
        std::sync::Arc<::paint::paint_property_tree::PaintCanvasChildState>;
    fn UnchangedPaintPropertyChange() -> Self::PaintPropertyChange {
        Self::PaintPropertyChange::Unchanged
    }
    fn ChangedOnlySimpleValues() -> Self::PaintPropertyChange {
        Self::PaintPropertyChange::ChangedOnlySimpleValues
    }
    fn InvalidDomNodeId() -> Self::DomNodeId {
        NodeId(0)
    }
}
impl<F: LayerizeFamily> PictureLayerTypes for F {
    type Recording = paint::CpuRecordingTypes<Sk<F>>;
    type ObjectTracker = paint::ObjectTracker;
    type TraceScope = paint::PaintTraceScope;
    type LayerTreeTask = paint::PaintTraceScope;
    type PictureRecorder = paint::CpuPictureRecorder<Sk<F>>;
}
impl<F: LayerizeFamily> SolidColorLayerTypes for F {
    type SolidPictureRecorder = paint::CpuPictureRecorder<Sk<F>>;
}
impl<F: LayerizeFamily> MainLayerTypes for F {
    type DidScrollCallback = main_callbacks::DidScrollCallback;
    type NodeInfo = crate::commit::paint::node_info::NodeInfo;
    type Picture = paint::Picture<Sk<F>>;
    type LayerSelection = crate::cc::input::layer_selection_bound::LayerSelection;
    type MicroBenchmark = paint::MicroBenchmark<Self>;
    type TaskRunner = main_callbacks::TaskRunner;
    type DerivedServices = dyn paint::CpuPaintServices<Self>;
}
impl<F: LayerizeFamily> ScrollbarLayerTypes for F {
    type ScrollbarCanvas = paint::ScrollbarCanvas;
    type Scrollbar = paint::Scrollbar;
    fn RasterizeScrollbarThumb(
        scrollbar: &Self::Scrollbar,
        logical_size: Size,
        pixel_size: Size,
    ) -> <Self as crate::commit::CommitTypes>::UIResourceBitmap {
        // Source: cc/layers/painted_scrollbar_layer.cc RasterizeScrollbarPart.
        // The thumb is painted on the main thread; the family allocates the
        // impl-side UIResourceBitmap from the painted pixels.
        use crate::cc::input::scrollbar::Scrollbar as _;
        let mut canvas = paint::ScrollbarCanvas::new(logical_size, pixel_size);
        let mut adapter = scrollbar.clone();
        adapter.PaintThumb(
            &mut canvas,
            &crate::adapter::geometry::Rect::new(0, 0, logical_size.width(), logical_size.height()),
        );
        <F as LayerizeFamily>::CreateUIResourceBitmap(
            canvas.pixel_size(),
            canvas.pixels().to_vec(),
            canvas.row_bytes(),
            false,
        )
    }
    fn WithPaintedScrollbarAnimation(
        scrollbar: &Self::Scrollbar,
        opacity: f32,
        thumb_width: i32,
        needs_repaint: bool,
    ) -> std::sync::Arc<Self::Scrollbar> {
        std::sync::Arc::new(scrollbar.WithPaintedAnimation(opacity, thumb_width, needs_repaint))
    }
    fn PaintedScrollbarConstructor() -> Option<ScrollbarConstructor<Self>> {
        Some(crate::cc::layers::painted_scrollbar_layer::PaintedScrollbarLayer::<Self>::Constructor)
    }
}
impl<F: LayerizeFamily> SolidColorScrollbarLayerTypes for F {}
impl<F: LayerizeFamily> PaintedScrollbarLayerTypes for F {}
