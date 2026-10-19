//! Exact resident Paint/node/value interfaces used by the canonical conversion
//! state machine. This boundary stores no chunks, layers, state stack or cache.
use super::{
    content_layer_client_impl::{
        ContentDisplayItemList, ContentLayerClientServices, ContentLayerClientTypes,
    },
    geometry::{Point, PointF, Rect, RectF, Vector2dF},
    input_types::ElementId,
    layer_tree_impl::SelectionBoundType,
    paint_geometry::{ChunkGeometryMapper, PaintChunkView},
    property_types::BlendMode,
};
use crate::cc::input::{layer_selection_bound::LayerSelection, touch_action::TouchAction};
use std::sync::Arc;

pub trait PaintChunksTypes:
    ContentLayerClientTypes<
    TransformNode: ConversionTransformNode<Self>,
    ClipNode: ConversionClipNode<Self> + PartialEq,
    EffectNode: ConversionEffectNode<Self> + PartialEq,
    PaintChunk: ConversionPaintChunk<Self> + PaintChunkView<Self::ChunkMapper>,
    PaintArtifact: ConversionPaintArtifact<Self>,
    DisplayItem: ConversionDisplayItem<Self>,
    LayerSelection = LayerSelection,
    ContentDisplayItemList: ConversionDisplayItemList<Self>,
    PaintOpBuffer: ConversionPaintOpBuffer<Self>,
    RasterInvalidationTracking: ConversionInvalidationTracking<Self>,
    RegionCaptureBounds: ConversionCaptureBounds<Self::CaptureId>,
    TrackedElementRects: ConversionTrackedElementRects<Self>,
>
{
    type ChunkMapper: ChunkGeometryMapper<
        PropertyTreeState = Self::PropertyTreeState,
        Transform: ConversionTransform,
    >;
    type RoundedRect: Clone;
    type PaintPath: Clone;
    type ScrollNode: ConversionScrollNode<Self> + Clone + PartialEq;
    type PaintFlags;
    type PaintFilter;
    type FilterOperations: Clone;
    type HitTestData: ConversionHitTestData<Self>;
    type RegionCaptureData: ConversionRegionCaptureData<Self>;
    type TrackedData: ConversionTrackedData<Self>;
    type SelectionData: ConversionSelectionData;
    type CaptureId: Copy;
    type TrackingFeature: Copy;
    type TrackedId: Copy;
    type FrameToken: Copy;
    type PaintRecorder: ConversionPaintRecorder<Self>;
    type UnderInvalidationPicture;
}
pub trait ConversionTransform: Clone {
    fn IsIdentity(&self) -> bool;
    fn IsIdentityOr2dTranslation(&self) -> bool;
    fn To2dTranslation(&self) -> Vector2dF;
    fn MapPoint(&self, point: PointF) -> PointF;
}
pub trait ConversionTransformNode<B: PaintChunksTypes> {
    fn ScrollNode(&self) -> Option<B::ScrollNode>;
    fn NearestScrollTranslationNode(&self) -> B::TransformNode;
    fn ParentScrollTranslationNode(&self) -> Option<B::TransformNode>;
    fn IsAncestorOf(&self, other: &B::TransformNode) -> bool;
    fn IsIdentity(&self) -> bool;
    /// Exact Parent() pointer equality before unaliasing, as used by CombineClip.
    fn ParentIs(&self, node: &B::TransformNode) -> bool;
}
pub trait ConversionClipNode<B: PaintChunksTypes> {
    fn UnaliasedParent(&self) -> Option<B::ClipNode>;
    fn PaintClipRect(&self) -> B::RoundedRect;
    fn ClipPath(&self) -> Option<B::PaintPath>;
}
pub trait ConversionEffectNode<B: PaintChunksTypes> {
    fn UnaliasedParent(&self) -> Option<B::EffectNode>;
    fn HasDirectCompositingReasons(&self) -> bool;
    fn RequiresCompositingForCanvasChild(&self) -> bool;
    fn HasRealEffects(&self) -> bool;
    fn Filter(&self) -> Option<B::FilterOperations>;
    fn BackdropFilter(&self) -> Option<B::FilterOperations>;
    fn BackdropFilterBounds(&self) -> B::PaintPath;
    fn HasReferenceFilter(&self) -> bool;
    fn FilterOutputBounds(&self) -> Rect;
    fn MapRect(&self, rect: Rect) -> Rect;
}
pub trait ConversionScrollNode<B: PaintChunksTypes> {
    fn Parent(&self) -> Option<B::ScrollNode>;
    fn ContentsRect(&self) -> Rect;
    fn ContainerRect(&self) -> Rect;
    fn UserScrollableHorizontal(&self) -> bool;
    fn PropagatesXScroll(&self) -> bool;
    fn CompositedScrollingNotPreferred(&self) -> bool;
    fn GetCompositorElementId(&self) -> ElementId;
    fn OverflowClipNode(&self) -> Option<B::ClipNode>;
}
pub trait ConversionPaintChunk<B: PaintChunksTypes> {
    fn effectively_invisible(&self) -> bool;
    fn drawable_bounds(&self) -> Rect;
    fn bounds_rect(&self) -> Rect;
    fn IsScrollHitTest(&self) -> bool;
    fn IsScrollbarHitTest(&self) -> bool;
    fn IsResizerScrollHitTest(&self) -> bool;
    fn hit_test_data(&self) -> Option<&B::HitTestData>;
    fn region_capture_data(&self) -> Option<&B::RegionCaptureData>;
    fn tracked_element_rects(&self) -> Option<&B::TrackedData>;
    fn layer_selection_data(&self) -> Option<&B::SelectionData>;
}
/// PaintChunk stores only source display-item indices. The real immutable
/// artifact resolves that range, matching native PaintChunkIterator::DisplayItems
/// without adding a chunk-to-artifact backpointer or copying item payloads.
pub trait ConversionPaintArtifact<B: PaintChunksTypes> {
    fn DisplayItemsForChunk<'a>(
        &'a self,
        chunk: &'a B::PaintChunk,
    ) -> impl ExactSizeIterator<Item = &'a B::DisplayItem>;
}
pub trait ConversionDisplayItem<B: PaintChunksTypes> {
    fn VisualRect(&self) -> Rect;
    fn IsDrawing(&self) -> bool;
    fn IsScrollbar(&self) -> bool;
    fn GetPaintRecord(
        &self,
        artifact: &Arc<B::PaintArtifact>,
        services: &dyn PaintChunksServices<B>,
    ) -> B::PaintRecord;
    fn PaintScrollbar(
        &self,
        artifact: &Arc<B::PaintArtifact>,
        services: &dyn PaintChunksServices<B>,
    ) -> B::PaintRecord;
    fn ScrollbarElementId(&self) -> ElementId;
}
pub trait ConversionHitTestData<B: PaintChunksTypes> {
    fn touch_action_rects(&self) -> impl Iterator<Item = (Rect, TouchAction)>;
    fn wheel_event_rects(&self) -> impl Iterator<Item = Rect>;
    fn xr_regions(&self) -> impl Iterator<Item = ElementId>;
    fn scroll_hit_test_rect(&self) -> Rect;
    fn scroll_translation(&self) -> Option<B::TransformNode>;
    fn scrolling_contents_cull_rect(&self) -> Rect;
}
pub trait ConversionRegionCaptureData<B: PaintChunksTypes> {
    fn iter(&self) -> impl Iterator<Item = (B::CaptureId, Rect)>;
}
pub trait ConversionCaptureBounds<I> {
    fn Set(&mut self, id: I, rect: Rect);
}
pub struct TrackedRect<I, F> {
    pub bounds: Rect,
    pub id: I,
    pub should_add_to_compositor_frame_metadata: bool,
    pub should_exclude_fixed_and_sticky_occlusions: bool,
    pub frame_token: F,
    pub parent_frame_token: F,
}
pub trait ConversionTrackedData<B: PaintChunksTypes> {
    fn iter(
        &self,
    ) -> impl Iterator<Item = (B::TrackingFeature, TrackedRect<B::TrackedId, B::FrameToken>)>;
}
pub trait ConversionTrackedElementRects<B: PaintChunksTypes> {
    fn Push(&mut self, feature: B::TrackingFeature, rect: TrackedRect<B::TrackedId, B::FrameToken>);
}
#[derive(Clone, Copy)]
pub struct PaintedSelectionBound {
    pub r#type: SelectionBoundType,
    pub edge_start: Point,
    pub edge_end: Point,
}
pub trait ConversionSelectionData {
    fn start(&self) -> Option<PaintedSelectionBound>;
    fn end(&self) -> Option<PaintedSelectionBound>;
    fn any_selection_was_painted(&self) -> bool;
}
/// Exact paint operation command values. Backends append to the actual native
/// op buffer; no replay, pixels, filter execution or recording strategy here.
pub enum ConversionOp<B: PaintChunksTypes> {
    Save,
    Restore,
    Translate(f32, f32),
    Concat(<B::ChunkMapper as ChunkGeometryMapper>::Transform),
    ClipRect {
        rect: RectF,
        difference: bool,
        antialias: bool,
    },
    ClipRRect(B::RoundedRect, bool),
    ClipPath {
        path: B::PaintPath,
        difference: bool,
        antialias: bool,
    },
    SaveLayer(B::PaintFlags),
    SaveLayerAlpha(f32),
    SaveLayerFilters {
        bounds: RectF,
        filters: Vec<B::PaintFilter>,
        backdrop_filter: B::PaintFilter,
        flags: B::PaintFlags,
    },
    DrawTransparentColorSrc,
    DrawRecord(B::PaintRecord),
}
pub trait ConversionDisplayItemList<B: PaintChunksTypes>: ContentDisplayItemList<B> {
    fn Push(&mut self, op: ConversionOp<B>) -> usize;
    fn EndPaintOfPairedBegin(&mut self);
    fn EndPaintOfPairedEnd(&mut self);
    fn UpdateSaveLayerBounds(&mut self, id: usize, rect: RectF);
    fn PushDrawScrollingContentsOp(
        &mut self,
        element: ElementId,
        list: Arc<B::ContentDisplayItemList>,
        rect: Rect,
    );
}
pub trait ConversionPaintOpBuffer<B: PaintChunksTypes> {
    fn next_op_offset(&self) -> usize;
    fn Push(&mut self, op: ConversionOp<B>);
    fn UpdateSaveLayerBounds(&mut self, id: usize, bounds: RectF);
}
pub trait ConversionPaintRecorder<B: PaintChunksTypes> {
    fn beginRecording(&mut self);
    fn DrawPicture(&mut self, record: B::PaintRecord);
    fn finishRecordingAsPicture(self) -> B::UnderInvalidationPicture;
}
pub trait ConversionInvalidationTracking<B: PaintChunksTypes> {
    fn CheckUnderInvalidations(
        &mut self,
        name: &str,
        picture: B::UnderInvalidationPicture,
        interest: Rect,
    );
    fn UnderInvalidationRecord(&self) -> B::PaintRecord;
}
pub trait PaintChunksServices<B: PaintChunksTypes>: ContentLayerClientServices<B> {
    /// Retains the artifact's native record range with the independent value
    /// analysis capability owned by the services. No item/owner backpointer.
    fn NativePaintRecord(
        &self,
        artifact: &Arc<B::PaintArtifact>,
        item: &B::DisplayItem,
    ) -> B::PaintRecord;
    fn ChunkMapper(&self) -> &B::ChunkMapper;
    fn MakeState(
        &self,
        transform: B::TransformNode,
        clip: B::ClipNode,
        effect: B::EffectNode,
    ) -> B::PropertyTreeState;
    fn IdentityTransform(&self) -> <B::ChunkMapper as ChunkGeometryMapper>::Transform;
    fn SourceToDestinationProjection(
        &self,
        source: &B::TransformNode,
        destination: &B::TransformNode,
        result: &mut <B::ChunkMapper as ChunkGeometryMapper>::Transform,
    ) -> bool;
    fn SourceToDestinationRect(
        &self,
        source: &B::TransformNode,
        destination: &B::TransformNode,
        rect: &mut RectF,
    );
    fn RoundedRectRect(&self, rect: &B::RoundedRect) -> RectF;
    fn RoundedRectIsRounded(&self, rect: &B::RoundedRect) -> bool;
    fn MakeRoundedRect(&self, rect: RectF) -> B::RoundedRect;
    fn PathBounds(&self, path: &B::PaintPath) -> RectF;
    fn PathIsRect(&self, path: &B::PaintPath) -> Option<RectF>;
    fn InfiniteIntRect(&self) -> Rect;
    fn NewPaintFlags(&self) -> B::PaintFlags;
    fn SetBlendMode(&self, flags: &mut B::PaintFlags, mode: BlendMode);
    fn SetAlphaf(&self, flags: &mut B::PaintFlags, alpha: f32);
    fn BuildImageFilter(&self, ops: B::FilterOperations) -> B::PaintFilter;
    fn SetImageFilter(&self, flags: &mut B::PaintFlags, filter: B::PaintFilter);
    fn RecordIsEmpty(&self, record: &B::PaintRecord) -> bool;
    fn NewPaintRecorder(&self) -> B::PaintRecorder;
    fn LogClipHierarchyIssue(&self, target: &B::ClipNode, current: &B::ClipNode);
    fn LogEffectHierarchyIssue(&self, target: &B::EffectNode, current: &B::EffectNode);
    fn RemoveScrollNodeWorkaroundEnabled(&self) -> bool;
    fn ScrollingContentsCullRectOnScrollNodeEnabled(&self) -> bool;
    fn SelectionEdgeVisibilityUsesFullEdge(&self) -> bool;
    fn ClipIsTight(&self, clip: &<B::ChunkMapper as ChunkGeometryMapper>::FloatClipRect) -> bool;
    fn BoundingRect(&self, first: Point, second: Point) -> Rect;
    fn ToRoundedPoint(&self, point: PointF) -> Point;
    fn OffsetRect(&self, rect: &mut Rect, x: i32, y: i32);
    fn OutsetRect(&self, rect: &mut Rect, outset: i32);
    fn IntersectRects(&self, first: RectF, second: RectF) -> RectF;
    fn UnionRects(&self, first: RectF, second: RectF) -> RectF;
}
