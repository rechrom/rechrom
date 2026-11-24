//! Production Paint value capabilities of the main half (moved from
//! compositor/src/runtime/paint_services.rs; the impl half keeps only the
//! value lease `CpuImplPaintServices` and the TreeSynchronizer trace binding).
//! This owner holds independent raster value/diagnostic leases, never a Page,
//! Layer, PAC or PaintArtifact backpointer. The family-generic impls bound
//! only `F: LayerizeFamily<Skia = R>` (MAIN_IMPL_SPLIT.md, option h).
use super::{
    paint::{self, CpuGeometryMapper},
    types::PaintSkiaBackend,
};
use crate::{
    adapter::{
        geometry::Vector2dF, input_types::ElementId, main_layer::MainLayerServices,
        picture_layer::PictureLayerServices, recording_source::RecordingSourceServices,
    },
    cc::layers::recording_source::RecordingSource,
    commit::layerize_family::LayerizeFamily,
};
use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};

pub use crate::commit::main_paint_config::CpuPaintFeatures;

/// Only native diagnostics and the external SkPicture recording target. All
/// recording, invalidation, conversion and layerization policy stays canonical.
pub trait CpuPaintPlatform<R: PaintSkiaBackend>: Send + Sync {
    fn VlogIsOn(&self, level: i32) -> bool;
    fn CrashKey(&self, name: &str, section: &str, value: String) -> paint::PaintTraceScope;
    fn DumpWithoutCrashing(&self);
    fn BeginPacUpdateTrace(&self) -> paint::PaintTraceScope;
    fn ShouldRecordSubsampledMetric(&self, probability: f64) -> bool;
    fn HistogramCounts100(&self, name: &str, value: i32);
    fn ColorNameForLayoutTree(&self, color: crate::adapter::layer_tree_impl::Color4f) -> String;
    fn CreateLoggingCanvas(&self) -> paint::CpuLoggingCanvas<R>;
    fn Diagnostic(&self, message: &str);
    fn TraceLayer(&self, event: &str, element: Option<ElementId>);
    fn Vector2dFToString(&self, value: Vector2dF) -> String;
    fn TraceScope(&self, category: &str, name: &str, count: Option<usize>);
    fn CreateObjectTracker(&self, layer_id: i32) -> paint::ObjectTracker;
    fn BeginPushPropertiesTrace(&self) -> paint::PaintTraceScope;
    fn BeginUpdateTrace(&self, source_frame_number: i32) -> paint::PaintTraceScope;
    fn BeginUpdateLayerTask(&self, layer_id: i32, host_id: i32) -> paint::PaintTraceScope;
    fn CreatePictureRecorder(&self) -> paint::CpuPictureRecorder<R>;
}

/// Main-thread (Layerize) paint services: PaintArtifactCompositor and
/// PaintChunksToCcLayer conversion, the GeometryMapper, PaintRecord
/// production and the main cc Layer services. Chromium reaches these from
/// LocalFrameView::PushPaintArtifactToCompositor on the main thread
/// (local_frame_view.cc:3348). The impl half never holds this owner; it
/// receives [`CpuImplPaintServices`].
pub struct CpuMainPaintServices<R: PaintSkiaBackend> {
    pub features: CpuPaintFeatures,
    values: Arc<dyn paint::CpuPaintValueServices<R>>,
    mapper: Arc<CpuGeometryMapper<R>>,
    platform: Arc<dyn CpuPaintPlatform<R>>,
}
impl<R: PaintSkiaBackend> CpuMainPaintServices<R> {
    pub fn new(
        features: CpuPaintFeatures,
        values: Arc<dyn paint::CpuPaintValueServices<R>>,
        tracking: Arc<dyn paint::CpuRasterTrackingServices<R>>,
        platform: Arc<dyn CpuPaintPlatform<R>>,
    ) -> Self {
        Self {
            mapper: Arc::new(CpuGeometryMapper::new(
                features.singular_transform_fix,
                tracking,
            )),
            features,
            values,
            platform,
        }
    }
    pub fn PrepareInput(
        &self,
        artifact: Arc<paint::PaintArtifact>,
    ) -> Result<paint::PaintInput, paint::PaintInputError> {
        paint::PaintInput::new(artifact)
    }
    pub fn GeometryMapper(&self) -> &Arc<CpuGeometryMapper<R>> {
        &self.mapper
    }
    pub fn CreatePaintOpBuffer(&self) -> paint::CpuPaintOpBuffer<R> {
        paint::CpuPaintOpBuffer::new(self.values.clone())
    }
    /// The shared cc/paint value capability; the impl half leases it as
    /// `CpuImplPaintServices`.
    pub fn PaintValues(&self) -> Arc<dyn paint::CpuPaintValueServices<R>> {
        self.values.clone()
    }
    /// The embedding's diagnostics/recording platform.
    pub fn Platform(&self) -> &Arc<dyn CpuPaintPlatform<R>> {
        &self.platform
    }
    pub fn CreateDisplayItemList(&self) -> paint::ContentDisplayItemList<R> {
        paint::ContentDisplayItemList::new_with_buffer(self.CreatePaintOpBuffer())
    }
    pub fn CreateRasterInvalidator(&self) -> paint::CpuRasterInvalidator<R> {
        paint::CpuRasterInvalidator::new(self.mapper.clone())
    }
}
// Source: cc/layers/layer.cc:79,97-100. AtomicSequenceNumber starts at zero;
// canonical Layer::GetNextLayerId adds one at its original source call site.
static NEXT_LAYER_ID: AtomicI32 = AtomicI32::new(0);
impl<R: PaintSkiaBackend> MainLayerServices for CpuMainPaintServices<R> {
    fn TraceLayer(&self, event: &str, element: Option<ElementId>) {
        self.platform.TraceLayer(event, element)
    }
    fn Vector2dFToString(&self, value: Vector2dF) -> String {
        self.platform.Vector2dFToString(value)
    }
    fn NextLayerIdSequenceValue(&self) -> i32 {
        NEXT_LAYER_ID.fetch_add(1, Ordering::Relaxed)
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.features.cc_slimming
    }
    fn RasterInducingScrollEnabled(&self) -> bool {
        self.features.raster_inducing_scroll
    }
    fn ScrollingContentsCullRectOnScrollNodeEnabled(&self) -> bool {
        self.features.scrolling_contents_cull_rect
    }
}
impl<R: PaintSkiaBackend> RecordingSourceServices<paint::CpuRecordingTypes<R>>
    for CpuMainPaintServices<R>
{
    fn CreateRasterSource(
        &self,
        recording: &RecordingSource<paint::CpuRecordingTypes<R>>,
    ) -> Arc<paint::RasterSourceStorage<R>> {
        Arc::new(paint::RasterSourceStorage::from_recording(recording))
    }
    fn TraceScope(&self, category: &str, name: &str, count: Option<usize>) {
        self.platform.TraceScope(category, name, count)
    }
}
impl<R: PaintSkiaBackend, F: LayerizeFamily<Skia = R>> PictureLayerServices<F> for CpuMainPaintServices<R> {
    fn CreateObjectTracker(&self, id: i32) -> paint::ObjectTracker {
        self.platform.CreateObjectTracker(id)
    }
    fn BeginPushPropertiesTrace(&self) -> paint::PaintTraceScope {
        self.platform.BeginPushPropertiesTrace()
    }
    fn BeginUpdateTrace(&self, frame: i32) -> paint::PaintTraceScope {
        self.platform.BeginUpdateTrace(frame)
    }
    fn BeginUpdateLayerTask(&self, id: i32, host: i32) -> paint::PaintTraceScope {
        self.platform.BeginUpdateLayerTask(id, host)
    }
    fn CreatePictureRecorder(&self) -> paint::CpuPictureRecorder<R> {
        self.platform.CreatePictureRecorder()
    }
}
impl<R: PaintSkiaBackend, F: LayerizeFamily<Skia = R>>
    crate::adapter::paint_artifact_compositor::SynthesizedClipPaintServices<F>
    for CpuMainPaintServices<R>
{
    fn CreateDisplayItemList(&self) -> paint::ContentDisplayItemList<R> {
        Self::CreateDisplayItemList(self)
    }
    fn NewPaintFlags(&self) -> paint::CpuPaintFlags<R> {
        Default::default()
    }
    fn SetAntiAlias(&self, flags: &mut paint::CpuPaintFlags<R>, value: bool) {
        flags.setAntiAlias(value)
    }
    fn PushDrawRRect(
        &self,
        list: &mut paint::ContentDisplayItemList<R>,
        r: &crate::commit::skia_region::SkRoundedRect,
        flags: paint::CpuPaintFlags<R>,
    ) {
        let rect = crate::commit::skia_region::SkRoundedRectValue(r);
        list.push(paint::CpuPaintOp::DrawRRect(rect, flags));
    }
}
impl<R: PaintSkiaBackend, F: LayerizeFamily<Skia = R>>
    crate::adapter::solid_color_layer::SolidColorLayerServices<F>
    for CpuMainPaintServices<R>
{
    fn CreateSolidPictureRecorder(&self) -> paint::CpuPictureRecorder<R> {
        self.platform.CreatePictureRecorder()
    }
}
impl<R: PaintSkiaBackend, F: LayerizeFamily<Skia = R>> crate::adapter::pending_layer::PendingLayerServices<F>
    for CpuMainPaintServices<R>
{
    fn CreateSolidColorLayer(
        &mut self,
        services: &dyn MainLayerServices,
    ) -> crate::cc::layers::layer::LayerRef<F> {
        crate::cc::layers::solid_color_layer::SolidColorLayer::Create(services)
    }
}

use crate::adapter::{
    geometry::{Point, PointF, RRectF, Rect, RectF},
    layer_tree_impl::Color4f,
    main_layer::LayerContext,
    paint_geometry::MaskGeometryMapper,
    pending_layer::{PendingLayerColor, PendingLayerGeometry, PendingMappedVisualRect},
};
use crate::third_party::blink::renderer::platform::graphics::paint::{
    float_clip_rect::FloatClipRect, geometry_mapper::GeometryMapper,
    geometry_mapper_clip_cache::OverlayScrollbarClipBehavior,
};
// Source primitive: ui/gfx/color_utils.cc:577-614. Native SkColor channels are
// quantized before blending and ClampRound'ed afterwards, as in the source.
impl<R: PaintSkiaBackend> PendingLayerColor for CpuMainPaintServices<R> {
    fn GetResultingPaintColor(&self, f: Color4f, b: Color4f) -> Color4f {
        let q = |v: f32| (v * 255.).round().clamp(0., 255.);
        let (fa, ba) = (q(f.alpha), q(b.alpha));
        let alpha = fa / 255.;
        let f = [q(f.red), q(f.green), q(f.blue), 255.];
        let b = [q(b.red), q(b.green), q(b.blue), ba];
        let value = if alpha == 0. {
            b
        } else if alpha == 1. {
            f
        } else {
            let n = 255. * alpha + ba * (1. - alpha);
            if n == 0. {
                [0.; 4]
            } else {
                let fw = 255. * alpha / n;
                let bw = ba * (1. - alpha) / n;
                [
                    (f[0] * fw + b[0] * bw).round(),
                    (f[1] * fw + b[1] * bw).round(),
                    (f[2] * fw + b[2] * bw).round(),
                    n.round(),
                ]
            }
        };
        Color4f {
            red: value[0] / 255.,
            green: value[1] / 255.,
            blue: value[2] / 255.,
            alpha: value[3] / 255.,
        }
    }
}
impl<R: PaintSkiaBackend> MaskGeometryMapper for CpuMainPaintServices<R> {
    type TransformPaintPropertyNode = paint::TransformNode<R>;
    fn SourceToDestinationRectFromRoot(&self, d: &Self::TransformPaintPropertyNode, r: &mut RectF) {
        GeometryMapper::<CpuGeometryMapper<R>>::SourceToDestinationRect(
            &paint::PropertyTreeState::<R>::default().transform(),
            d,
            r,
        )
    }
}
impl<R: PaintSkiaBackend, F: LayerizeFamily<Skia = R>>
    crate::adapter::content_layer_client_impl::ContentLayerClientServices<F>
    for CpuMainPaintServices<R>
{
    fn CreateDisplayItemList(&self) -> paint::ContentDisplayItemList<R> {
        Self::CreateDisplayItemList(self)
    }
    fn CreateRasterInvalidator(&self) -> paint::CpuRasterInvalidator<R> {
        Self::CreateRasterInvalidator(self)
    }
    fn RasterInducingScrollEnabled(&self) -> bool {
        self.features.raster_inducing_scroll
    }
    fn PaintUnderInvalidationCheckingEnabled(&self) -> bool {
        self.features.paint_under_invalidation_checking
    }
    fn ExpensiveDChecksEnabled(&self) -> bool {
        self.features.expensive_dchecks
    }
    fn NewJsonArray(&self) -> Vec<serde_json::Value> {
        Vec::new()
    }
    fn AppendChunkDebugData(
        &self,
        array: &mut Vec<serde_json::Value>,
        chunks: &paint::PaintChunkSubset,
        index: usize,
    ) {
        array.push(serde_json::Value::String(format!(
            "{:?}",
            chunks.chunk(index).id
        )))
    }
    fn CanvasChildPaintState(
        &self,
        state: &paint::PropertyTreeState<R>,
    ) -> Option<Arc<::paint::paint_property_tree::PaintCanvasChildState>> {
        state.effect().node().compositor.canvas_child_state.clone()
    }
    fn ScrollContentsRect(&self, state: &paint::PropertyTreeState<R>) -> Option<Rect> {
        use crate::adapter::pending_layer::PendingTransformNode;
        state
            .transform()
            .PropertyTreeNearestScrollTranslationNode()
            .node()
            .scroll
            .as_ref()
            .map(|s| {
                let contents = paint_model::property_cell::with_access(|a| s.ContentsRect(a));
                crate::adapter::geometry::ToEnclosingRect(RectF::new(
                    contents.x as f32,
                    contents.y as f32,
                    contents.width as f32,
                    contents.height as f32,
                ))
            })
    }
    fn RectContains(&self, a: RectF, b: RectF) -> bool {
        a.Contains(b)
    }
    fn CreatePaintOpBuffer(&self) -> paint::CpuPaintOpBuffer<R> {
        Self::CreatePaintOpBuffer(self)
    }
    fn CreateCanvasChildPaintRecord(
        &self,
        state: Arc<::paint::paint_property_tree::PaintCanvasChildState>,
        record: paint::CpuPaintRecord<R>,
    ) -> paint::CpuCanvasChildPaintRecord<R> {
        paint::CpuCanvasChildPaintRecord { state, record }
    }
    fn CreateLoggingCanvas(&self) -> paint::CpuLoggingCanvas<R> {
        self.platform.CreateLoggingCanvas()
    }
    fn CurrentScrollOffset(
        &self,
        layer: &crate::cc::layers::layer::Layer<F>,
        id: ElementId,
        c: &LayerContext<'_, F>,
    ) -> PointF {
        let trees = layer
            .layer_tree_host(c)
            .expect("source attached layer")
            .property_trees();
        trees.scroll_tree().current_scroll_offset(id, trees.state())
    }
}
impl<R: PaintSkiaBackend, F: LayerizeFamily<Skia = R>> crate::adapter::paint_chunks_to_cc_layer::PaintChunksServices<F>
    for CpuMainPaintServices<R>
{
    fn PaintRecord(
        &self,
        a: &Arc<paint::PaintArtifact>,
        item: &paint::DisplayItem,
    ) -> paint::CpuPaintRecord<R> {
        paint::CpuPaintRecord::from_source(
            paint::PaintRecord::from_item(a, item).expect("validated native record range"),
            self.values.clone(),
        )
    }
    fn ChunkMapper(&self) -> &CpuGeometryMapper<R> {
        &self.mapper
    }
    fn MakeState(
        &self,
        t: paint::TransformNode<R>,
        c: paint::ClipNode<R>,
        e: paint::EffectNode<R>,
    ) -> paint::PropertyTreeState<R> {
        let mut s = paint::PropertyTreeState::default();
        s.set_transform(t);
        s.set_clip(c);
        s.set_effect(e);
        s
    }
    fn IdentityTransform(&self) -> super::geometry::Transform<R> {
        Default::default()
    }
    fn SourceToDestinationProjection(
        &self,
        s: &paint::TransformNode<R>,
        d: &paint::TransformNode<R>,
        r: &mut super::geometry::Transform<R>,
    ) -> bool {
        GeometryMapper::<CpuGeometryMapper<R>>::SourceToDestinationProjectionTo(
            self.mapper.as_ref(),
            s,
            d,
            r,
        )
    }
    fn SourceToDestinationRect(
        &self,
        s: &paint::TransformNode<R>,
        d: &paint::TransformNode<R>,
        r: &mut RectF,
    ) {
        GeometryMapper::<CpuGeometryMapper<R>>::SourceToDestinationRect(s, d, r)
    }
    fn RoundedRectRect(&self, r: &RRectF) -> RectF {
        r.rect
    }
    fn RoundedRectIsRounded(&self, r: &RRectF) -> bool {
        r.corner_radii.iter().any(|v| v.x() != 0. || v.y() != 0.)
    }
    fn MakeRoundedRect(&self, r: RectF) -> RRectF {
        RRectF {
            rect: r,
            corner_radii: [Vector2dF::default(); 4],
        }
    }
    fn PathBounds(&self, p: &Arc<::paint::paint_clip_path::PaintClipPath>) -> RectF {
        let mut rect = RectF::default();
        for p in p.paths() {
            let b = p.bounds();
            rect.Union(RectF::new(
                b.x as f32,
                b.y as f32,
                b.width as f32,
                b.height as f32,
            ));
        }
        rect
    }
    fn PathIsRect(&self, p: &Arc<::paint::paint_clip_path::PaintClipPath>) -> Option<RectF> {
        let p = p.single_path()?;
        let b = p.is_rect()?;
        Some(RectF::new(
            b.x as f32,
            b.y as f32,
            b.width as f32,
            b.height as f32,
        ))
    }
    fn InfiniteIntRect(&self) -> Rect {
        let r = foundation::infinite_int_rect::InfiniteIntRect();
        Rect::new(r.x(), r.y(), r.width(), r.height())
    }
    fn NewPaintFlags(&self) -> paint::CpuPaintFlags<R> {
        Default::default()
    }
    fn SetBlendMode(
        &self,
        f: &mut paint::CpuPaintFlags<R>,
        m: crate::adapter::property_types::BlendMode,
    ) {
        f.setBlendMode(m)
    }
    fn SetAlphaf(&self, f: &mut paint::CpuPaintFlags<R>, a: f32) {
        f.setAlphaf(a)
    }
    fn BuildImageFilter(
        &self,
        o: ::paint::cc::paint::filter_operations::FilterOperations,
    ) -> paint::CpuPaintFilter {
        Arc::new(paint::CpuOperationsFilter(o))
    }
    fn SetImageFilter(&self, f: &mut paint::CpuPaintFlags<R>, v: paint::CpuPaintFilter) {
        f.image_filter = Some(v)
    }
    fn RecordIsEmpty(&self, r: &paint::CpuPaintRecord<R>) -> bool {
        r.is_empty()
    }
    fn NewPaintRecorder(&self) -> paint::CpuConversionRecorder<R> {
        paint::CpuConversionRecorder {
            buffer: self.CreatePaintOpBuffer(),
        }
    }
    fn LogClipHierarchyIssue(&self, t: &paint::ClipNode<R>, c: &paint::ClipNode<R>) {
        self.platform
            .Diagnostic(&format!("clip hierarchy target={t:?} current={c:?}"))
    }
    fn LogEffectHierarchyIssue(&self, t: &paint::EffectNode<R>, c: &paint::EffectNode<R>) {
        self.platform
            .Diagnostic(&format!("effect hierarchy target={t:?} current={c:?}"))
    }
    fn RemoveScrollNodeWorkaroundEnabled(&self) -> bool {
        self.features.remove_scroll_node_workaround
    }
    fn ScrollingContentsCullRectOnScrollNodeEnabled(&self) -> bool {
        self.features.scrolling_contents_cull_rect
    }
    fn SelectionEdgeVisibilityUsesFullEdge(&self) -> bool {
        self.features.selection_edge_visibility_uses_full_edge
    }
    fn ClipIsTight(&self, r: &FloatClipRect) -> bool {
        r.IsTight()
    }
    fn BoundingRect(&self, a: Point, b: Point) -> Rect {
        Rect::FromBounds(
            a.x().min(b.x()),
            a.y().min(b.y()),
            a.x().max(b.x()),
            a.y().max(b.y()),
        )
    }
    fn ToRoundedPoint(&self, p: PointF) -> Point {
        Point::new(p.x().round() as i32, p.y().round() as i32)
    }
    fn OffsetRect(&self, r: &mut Rect, x: i32, y: i32) {
        r.x = r.x.saturating_add(x);
        r.y = r.y.saturating_add(y)
    }
    fn OutsetRect(&self, r: &mut Rect, n: i32) {
        *r = Rect::FromBounds(
            r.x.saturating_sub(n),
            r.y.saturating_sub(n),
            r.right().saturating_add(n),
            r.bottom().saturating_add(n),
        );
    }
    fn IntersectRects(&self, mut a: RectF, b: RectF) -> RectF {
        a.Intersect(b);
        a
    }
    fn UnionRects(&self, mut a: RectF, b: RectF) -> RectF {
        a.Union(b);
        a
    }
}

impl<R: PaintSkiaBackend, F: LayerizeFamily<Skia = R>> PendingLayerGeometry<F> for CpuMainPaintServices<R> {
    fn VisibilityLimit(&self, s: &paint::PropertyTreeState<R>) -> Option<RectF> {
        GeometryMapper::<CpuGeometryMapper<R>>::VisibilityLimit(
            &self.mapper.with_trees(s.transform().trees().clone()),
            s,
        )
    }
    fn MightOverlapForCompositing(
        &self,
        a: RectF,
        sa: &paint::PropertyTreeState<R>,
        b: RectF,
        sb: &paint::PropertyTreeState<R>,
    ) -> bool {
        GeometryMapper::<CpuGeometryMapper<R>>::MightOverlapForCompositing(
            &self.mapper.with_trees(sa.transform().trees().clone()),
            &a,
            sa,
            &b,
            sb,
        )
    }
    fn MapLocalToAncestorVisualRect(
        &self,
        s: &paint::PropertyTreeState<R>,
        d: &paint::PropertyTreeState<R>,
        r: RectF,
    ) -> PendingMappedVisualRect {
        let mut value = FloatClipRect::FromRect(r);
        GeometryMapper::<CpuGeometryMapper<R>>::LocalToAncestorVisualRect(
            &self.mapper.with_trees(s.transform().trees().clone()),
            s,
            d,
            &mut value,
            OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
            Default::default(),
        );
        PendingMappedVisualRect {
            rect: *value.Rect(),
            is_tight: value.IsTight(),
        }
    }
    // Source: ui/gfx/geometry/rect_f.cc:338-370. Preserve float area/tie order.
    fn MaximumCoveredRect(&self, a: RectF, b: RectF) -> RectF {
        let mut max = a;
        let area = |r: RectF| r.width() * r.height();
        let mut ma = area(a);
        if area(b) > ma {
            max = b;
            ma = area(b);
        }
        let x = a.x().max(b.x());
        let y = a.y().max(b.y());
        let r = a.right().min(b.right());
        let bot = a.bottom().min(b.bottom());
        let intersection = if x <= r && y <= bot {
            RectF::new(x, y, r - x, bot - y)
        } else {
            RectF::default()
        };
        if intersection.width() != 0. || intersection.height() != 0. {
            let vy = a.y().min(b.y());
            let vertical = RectF::new(
                intersection.x(),
                vy,
                intersection.width(),
                a.bottom().max(b.bottom()) - vy,
            );
            if area(vertical) > ma {
                max = vertical;
                ma = area(vertical);
            }
            let hx = a.x().min(b.x());
            let horizontal = RectF::new(
                hx,
                intersection.y(),
                a.right().max(b.right()) - hx,
                intersection.height(),
            );
            if area(horizontal) > ma {
                max = horizontal;
            }
        }
        max
    }
    fn UnionRects(&self, mut a: RectF, b: RectF) -> RectF {
        a.Union(b);
        a
    }
    // Source: ui/gfx/geometry/rect_conversions.cc:113-120.
    fn ToRoundedRect(&self, r: RectF) -> Rect {
        Rect::FromBounds(
            r.x().round() as i32,
            r.y().round() as i32,
            r.right().round() as i32,
            r.bottom().round() as i32,
        )
    }
    fn MergeFixedLayersEnabled(&self) -> bool {
        self.features.merge_fixed_layers
    }
    fn MergeStickyLayersEnabled(&self) -> bool {
        self.features.merge_sticky_layers
    }
}
impl<R: PaintSkiaBackend, F: LayerizeFamily<Skia = R>> crate::adapter::paint_artifact_compositor::SynthesizedClipServices<F>
    for CpuMainPaintServices<R>
{
    fn NewUniqueObjectElementId(&self) -> ElementId {
        <F as crate::adapter::property_tree_manager::PropertyTreeManagerTypes>::NewUniqueElementId()
    }
    fn Projection(
        &self,
        s: &paint::TransformNode<R>,
        d: &paint::TransformNode<R>,
    ) -> super::geometry::Transform<R> {
        GeometryMapper::<CpuGeometryMapper<R>>::SourceToDestinationProjection(s, d)
    }
    fn ToSkRoundedRect(&self, r: &RRectF) -> crate::commit::skia_region::SkRoundedRect {
        crate::commit::skia_region::MakeSkRoundedRect(r)
    }
    fn OffsetSkRoundedRect(&self, r: &mut crate::commit::skia_region::SkRoundedRect, v: Vector2dF) {
        crate::commit::skia_region::OffsetSkRoundedRect(r, v)
    }
    fn MapRect(&self, t: &super::geometry::Transform<R>, r: Rect) -> Rect {
        crate::adapter::geometry::ToEnclosingRect(t.MapRect(RectF::from(r)))
    }
    fn PostTranslate(&self, t: &mut super::geometry::Transform<R>, v: Vector2dF) {
        crate::adapter::property_tree::PropertyTreeTransform::PostTranslate(t, v)
    }
    fn MakeIdentity(&self, t: &mut super::geometry::Transform<R>) {
        *t = Default::default()
    }
    fn ClipPathEquals(
        &self,
        c: &paint::ClipNode<R>,
        previous: &Option<Arc<::paint::paint_clip_path::PaintClipPath>>,
    ) -> bool {
        paint_model::property_cell::with_access(|a| c.node().ClipPathEquals(a, previous.as_ref()))
    }
}

impl<R: PaintSkiaBackend, F: LayerizeFamily<Skia = R>>
    crate::adapter::paint_artifact_compositor::PaintArtifactCompositorServices<F>
    for CpuMainPaintServices<R>
{
    fn VlogIsOn(&self, l: i32) -> bool {
        self.platform.VlogIsOn(l)
    }
    fn NullPointerForDebug(&self) -> String {
        format!("{:p}", std::ptr::null::<()>())
    }
    fn CrashKey(&self, name: &str, section: &str, value: String) -> paint::PaintTraceScope {
        self.platform.CrashKey(name, section, value)
    }
    fn DumpWithoutCrashing(&self) {
        self.platform.DumpWithoutCrashing()
    }
    fn CanvasDrawElementEnabledByRuntimeFlag(&self) -> bool {
        self.features.canvas_draw_element
    }
    fn LayerMergeDistanceLimit(&self) -> i32 {
        self.features.layer_merge_distance_limit
    }
    // Source: property_tree_manager.cc:372-376. The source static is one immutable
    // native identity, not a mutable owner registry or a Page-specific cache.
    fn RootEffectElementId(&self) -> ElementId {
        static ID: std::sync::OnceLock<ElementId> = std::sync::OnceLock::new();
        *ID.get_or_init(||<F as crate::adapter::property_tree_manager::PropertyTreeManagerTypes>::NewUniqueElementId())
    }
    // Source: platform/graphics/compositor_element_id.cc:51-56.
    fn CompositorElementIdFromDomNodeId(&self, id: super::types::NodeId) -> ElementId {
        assert!(id.0 >= 0);
        let ns = foundation::CompositorElementIdNamespace::kDOMNodeId as u64;
        ElementId::new(
            (((id.0 as u64) << foundation::compositor_element_id::kCompositorNamespaceBitCount)
                + ns)
                << foundation::compositor_element_id::kCompositorReservedBitCount,
        )
    }
    fn NewUpdateTrace(&self) -> paint::PaintTraceScope {
        self.platform.BeginPacUpdateTrace()
    }
    fn ShouldRecordSubsampledMetric(&self, p: f64) -> bool {
        self.platform.ShouldRecordSubsampledMetric(p)
    }
    fn HistogramCounts100(&self, name: &str, value: i32) {
        self.platform.HistogramCounts100(name, value)
    }
    fn CanvasSnapshot(
        &self,
        callback: &Arc<dyn paint::CpuCanvasSnapshot<R>>,
        child: super::types::NodeId,
    ) -> Option<paint::CpuPaintRecord<R>> {
        callback.Snapshot(child)
    }
    fn DomNodeIdFromCustomData(&self, id: u32) -> super::types::NodeId {
        super::types::NodeId(id as i32)
    }
    fn AppendPendingLayerJson(
        &self,
        a: &mut Vec<serde_json::Value>,
        v: crate::adapter::pending_layer::PendingLayerJson,
    ) {
        a.push(serde_json::json!({"debug_name":v.debug_name,"bounds":[v.bounds.x(),v.bounds.y(),v.bounds.width(),v.bounds.height()],"rect_known_to_be_opaque":[v.rect_known_to_be_opaque.x(),v.rect_known_to_be_opaque.y(),v.rect_known_to_be_opaque.width(),v.rect_known_to_be_opaque.height()],"text_known_to_be_on_opaque_background":v.text_known_to_be_on_opaque_background,"property_tree_state":v.property_tree_state,"offset_of_decomposited_transforms":[v.offset_of_decomposited_transforms.x(),v.offset_of_decomposited_transforms.y()],"paint_chunks":v.paint_chunks,"draws_content":v.draws_content,"is_solid_color":v.is_solid_color,"hit_test_opaqueness":v.hit_test_opaqueness}));
    }
    fn NewLayersAsJson(
        &self,
        f:crate::third_party::blink::renderer::platform::graphics::compositing::content_layer_client_impl::LayerTreeFlags,
    ) -> paint::CpuLayersAsJson<F> {
        paint::CpuLayersAsJson(crate::third_party::blink::renderer::platform::graphics::compositing::layers_as_json::LayersAsJSON::new(f))
    }
    fn Descriptions(&self, r: &paint::CpuCompositingReasons) -> Vec<&'static str> {
        r.descriptions()
    }
    fn ShortNames(&self, r: &paint::CpuCompositingReasons) -> Vec<&'static str> {
        r.short_names()
    }
    fn IsTracingRasterInvalidations(&self) -> bool {
        self.mapper.IsTracingRasterInvalidations()
    }
    // Source: platform/graphics/compositor_element_id.cc:63-68.
    fn IsScrollbarElementId(&self, id: ElementId) -> bool {
        let ns = (id.GetInternalValue()
            >> foundation::compositor_element_id::kCompositorReservedBitCount)
            % foundation::CompositorElementIdNamespace::kMaxRepresentable;
        ns == foundation::CompositorElementIdNamespace::kVerticalScrollbar as u64
            || ns == foundation::CompositorElementIdNamespace::kHorizontalScrollbar as u64
    }
    fn DebugShowLayers(&self, json: serde_json::Value) {
        self.platform.Diagnostic(&json.to_string())
    }
    fn DebugVlogLayers(&self, event: &str, json: serde_json::Value) {
        self.platform.Diagnostic(&format!("{event}: {json}"))
    }
}

/// This adapter has no resident state: the native factory takes all value
/// capabilities from its call argument. It prevents overlapping mutable
/// PendingServices/shared PACServices borrows of the same capability owner.
struct CpuPendingLayerServices<R: PaintSkiaBackend>(std::marker::PhantomData<fn() -> R>);
impl<R: PaintSkiaBackend, F: LayerizeFamily<Skia = R>> crate::adapter::pending_layer::PendingLayerServices<F>
    for CpuPendingLayerServices<R>
{
    fn CreateSolidColorLayer(
        &mut self,
        services: &dyn MainLayerServices,
    ) -> crate::cc::layers::layer::LayerRef<F> {
        crate::cc::layers::solid_color_layer::SolidColorLayer::Create(services)
    }
}
impl<R: PaintSkiaBackend> CpuMainPaintServices<R> {
    /// Engine-internal construction of the actual canonical PAC. The caller
    /// attaches RootLayer to CpuMainLayerHost using its live main context.
    pub fn CreateCompositor<F: LayerizeFamily<Skia = R>>(&self,callbacks:std::sync::Weak<dyn crate::cc::trees::property_tree::ScrollCallbacks>,c:&mut LayerContext<'_,F>)->crate::third_party::blink::renderer::platform::graphics::compositing::paint_artifact_compositor::PaintArtifactCompositor<F>{
        crate::third_party::blink::renderer::platform::graphics::compositing::paint_artifact_compositor::PaintArtifactCompositor::new(callbacks,c,self)
    }
    /// Source: PaintArtifactCompositor::DirectlyUpdateTransform and
    /// DirectlyUpdateCompositedOpacityValue (paint_artifact_compositor.cc:
    /// 1458-1469,1479-1495), called per node as in
    /// paint_property_tree_builder.cc (FragmentPaintPropertyTreeBuilder::
    /// UpdateTransform / UpdateEffect; CompositorSimpleValuesUpdated,
    /// paint_property_node.h:82-85). Page has already run Blink's deferred
    /// property builder and admitted only simple animated values. Returns
    /// whether every node was updated directly; otherwise the caller runs
    /// the full Paint, whose PaintArtifactCompositor::Update rewrites the
    /// rejected nodes (local_frame_view.cc:2965-2975,3348).
    pub fn ApplyCompositedAnimationProperties<F: LayerizeFamily<Skia = R>>(
        &self,
        pac: &crate::third_party::blink::renderer::platform::graphics::compositing::paint_artifact_compositor::PaintArtifactCompositor<F>,
        update: &::paint::CompositedAnimationPropertyUpdate,
        context: &mut LayerContext<'_, F>,
    ) -> bool {
        let mut trace = browser_tracing::span(
            "compositor.layerize",
            "Compositor.Layerize.DirectAnimationPropertyUpdate",
        );
        trace.set("transform_updates", update.transforms.len() as f64);
        trace.set("effect_updates", update.effects.len() as f64);
        if let Some(node) = update.transforms.first() {
            trace.set("transform_x", node.Matrix().values[12]);
            trace.set("transform_y", node.Matrix().values[13]);
        }
        if let Some(node) = update.effects.first() {
            trace.set("opacity", node.opacity as f64);
        }
        // Per node, as FragmentPaintPropertyTreeBuilder::UpdateTransform /
        // UpdateEffect call PAC: an accepted node becomes
        // kChangedOnlyCompositedValues; a rejected one keeps
        // kChangedOnlySimpleValues and reaches PAC's full Update through the
        // same lifecycle's Paint.
        let mut rejected = 0usize;
        for node in &update.transforms {
            let transform = paint::TransformNode::<R>::from_arc(node.clone());
            if pac.DirectlyUpdateTransform(&transform, context) {
                paint_model::property_cell::with_access(|a| node.lifecycle.CompositorSimpleValuesUpdated(a));
            } else {
                rejected += 1;
            }
        }
        for node in &update.effects {
            let effect = paint::EffectNode::<R>::from_arc(node.clone());
            if pac.DirectlyUpdateCompositedOpacityValue(&effect, context) {
                paint_model::property_cell::with_access(|a| node.lifecycle.CompositorSimpleValuesUpdated(a));
            } else {
                rejected += 1;
            }
        }
        trace.set("rejected", rejected as f64);
        trace.set("applied", (rejected == 0) as u8 as f64);
        rejected == 0
    }

    pub fn ApplyScrollbarPaintMutation<F: LayerizeFamily<Skia = R>>(
        &self,
        pac: &crate::third_party::blink::renderer::platform::graphics::compositing::paint_artifact_compositor::PaintArtifactCompositor<F>,
        element: ElementId,
        opacity: f32,
        thumb_width: i32,
        needs_repaint: bool,
        context: &mut LayerContext<'_, F>,
    ) -> bool {
        let mut trace = browser_tracing::span(
            "compositor.layerize",
            "Compositor.Layerize.ScrollbarPaintMutation",
        );
        trace.set("element", element.GetInternalValue() as f64);
        trace.set("opacity", f64::from(opacity));
        trace.set("thumb_width", thumb_width as f64);
        let applied = pac.UpdateScrollbarPaintedAnimation(
            element,
            opacity,
            thumb_width,
            needs_repaint,
            context,
            self,
        );
        trace.set("applied", applied as u8 as f64);
        applied
    }

    /// Validation happens before any canonical owner is mutated. Input retains
    /// exactly the supplied artifact Arc; all layerization stays inside PAC.
    pub fn UpdateCompositor<F: LayerizeFamily<Skia = R>>(
        &self,
        pac:&mut crate::third_party::blink::renderer::platform::graphics::compositing::paint_artifact_compositor::PaintArtifactCompositor<F>,
        input: paint::PaintInput,
        previous_artifact: Option<&Arc<paint::PaintArtifact>>,
        requests: Vec<Box<<F as crate::commit::CommitTypes>::ViewTransitionRequest>>,
        c: &mut LayerContext<'_, F>,
        sequence: &mut i32,
    ) -> Result<(), paint::PaintInputError> {
        let viewport = input.ViewportProperties::<R, F>();
        let mut pending = CpuPendingLayerServices(std::marker::PhantomData);
        let mut context =
            crate::adapter::paint_artifact_compositor::PaintArtifactCompositorContext {
                layer: c,
                pending: &mut pending,
                services: self,
                property_tree_sequence_number: sequence,
            };
        if previous_artifact.is_some() {
            // PaintPropertyTreeBuilder::UpdateScrollAndScrollTranslation pushes
            // a composited scroll offset straight into the resident cc
            // transform node during PrePaint and, when PAC accepts it,
            // downgrades the node's marker to kChangedOnlyCompositedValues.
            // A scroll step therefore never reaches PAC as a property-tree
            // change, and LocalFrameView requests only the repaint update.
            // Paint's immutable artifact reaches this boundary after PrePaint,
            // so perform that same direct update here, before the full/repaint
            // classification below reads the markers.
            //
            // Source: paint_property_tree_builder.cc:3801-3836;
            //         paint_property_node.h:82-85;
            //         paint_artifact_compositor.cc:1470-1478.
            let mut direct_scroll_updates = 0usize;
            for node in &input
                .artifact()
                .compositor_viewport
                .as_ref()
                .expect("validated native viewport")
                .scroll_translation_nodes
            {
                if paint_model::property_cell::with_access(|a| node.lifecycle.node_changed(a))
                    !=::paint::paint_property_tree::PaintPropertyChangeType::ChangedOnlySimpleValues
                    || !node.HasDirectCompositingReasons()
                {
                    continue;
                }
                let transform = paint::TransformNode::<R>::from_arc(node.clone());
                if pac.DirectlyUpdateScrollOffsetTransform(&transform, context.layer) {
                    paint_model::property_cell::with_access(|a| node.lifecycle.CompositorSimpleValuesUpdated(a));
                    direct_scroll_updates += 1;
                }
            }
            browser_tracing::instant(
                "compositor.layerize",
                "Layerize.DirectScrollOffsetUpdate",
                &[("updated", direct_scroll_updates as f64)],
            );
        }
        if let Some(previous) = previous_artifact {
            // Chromium's ScrollbarDisplayItem retains a live cc::Scrollbar
            // adapter. PaintedScrollbarLayer can therefore reuse its layer on
            // the repaint fast path and still observe changing geometry and
            // opacity through that adapter (scrollbar_display_item.h:93-106,
            // painted_scrollbar_layer.cc:185-283). The paint_record package
            // boundary retains the same inputs as an immutable payload
            // instead. Reusing layerization when that payload changed would
            // leave PaintedScrollbarLayer attached to the old snapshot.
            // Promote only those repaints to the canonical full layer update;
            // all other repaint and resize fast paths remain unchanged.
            let scrollbar_inputs_changed = previous
                .display_items
                .iter()
                .filter_map(|item| item.scrollbar.as_ref())
                .ne(input
                    .artifact()
                    .display_items
                    .iter()
                    .filter_map(|item| item.scrollbar.as_ref()));
            // Blink's LocalFrameView/PaintArtifactCompositor owner calls
            // SetNeedsUpdate before SetNeedsUpdateAfterRepaint when the paint
            // property trees changed. Page publishes both immutable artifacts
            // here, so compare their resident node identities directly and
            // preserve that caller contract. Otherwise PAC treats this known
            // lifecycle change as an under-invalidation and builds a full
            // crash-key dump before falling back to the same full update.
            //
            // Source: local_frame_view.cc:3545-3563;
            // paint_artifact_compositor.cc:468-515,646-671.
            // LocalFrameView also reports an in-place value change on any
            // ancestor of a chunk's direct property state. Comparing only the
            // direct state values misses that case: retained nodes keep their
            // identity while their shared lifecycle marker is raised. This is
            // the same ChangedToRoot threshold asserted by PAC's fast path.
            let changed_to_root = input.artifact().chunks.iter().any(|chunk| {
                paint_model::property_cell::with_access(|a| {
                    ::paint::property_node_geometry::PropertyStateChanged(
                        a,
                        &chunk.properties,
                    ::paint::paint_property_tree::PaintPropertyChangeType::ChangedOnlySimpleValues,
                    &::paint::paint_property_tree::PropertyTreeState::default(),
                    )
                })
            });
            let property_trees_changed = changed_to_root
                || (previous.chunks.len() == input.artifact().chunks.len()
                    && previous
                        .chunks
                        .iter()
                        .zip(&input.artifact().chunks)
                        .any(|(old, new)| !old.properties.same_nodes(&new.properties)));
            if property_trees_changed || scrollbar_inputs_changed {
                pac.SetNeedsUpdate();
            } else {
                pac.SetNeedsUpdateAfterRepaint(previous, input.artifact(), self);
            }
        }
        if !pac.TryFastPathUpdate(input.artifact().clone(), &mut context) {
            pac.Update(
                input.artifact().clone(),
                &viewport,
                input.ScrollTranslationNodes::<R>().collect(),
                requests,
                &mut context,
            );
        }
        // Blink's LocalFrameView sends simple scroll-value changes through
        // PaintArtifactCompositor's direct scroll API. Our immutable artifact
        // boundary carries that value together with repaint data, so apply it
        // after PAC has retained or rebuilt the scroll node and before the
        // main tree is committed. This prevents PAC's retained node value
        // from replacing the newer main-thread absolute offset.
        //
        // Source: local_frame_view.cc:3545-3563;
        //         paint_artifact_compositor.cc:1500-1521.
        for scroll_translation in &input
            .artifact()
            .compositor_viewport
            .as_ref()
            .expect("validated native viewport")
            .scroll_translation_nodes
        {
            let Some(scroll) = &scroll_translation.scroll else {
                continue;
            };
            let value = paint_model::property_cell::with_access(|a| scroll.GetCompositorElementId(a))
                .GetInternalValue();
            if !ElementId::IsValidInternalValue(value) {
                continue;
            }
            let offset = PointF::new(
                -scroll_translation.matrix.values[12] as f32,
                -scroll_translation.matrix.values[13] as f32,
            );
            // PaintPropertyTreeBuilder updates the cc transform even when the
            // retained Paint node reports no effective change. This matters
            // after ApplyCompositorChanges has reflected the impl delta into
            // the main scroll map: PushMainToPending can then see equal
            // SyncedScrollOffset values, so the copied transform itself must
            // already carry the absolute offset before activation.
            //
            // Source: paint_property_tree_builder.cc:3801-3827;
            //         property_tree_manager.cc:138-177.
            let transform = paint::TransformNode::<R>::from_arc(scroll_translation.clone());
            let direct = pac.DirectlyUpdateScrollOffsetTransform(&transform, context.layer);
            if !direct {
                // The direct transform path intentionally declines nodes that
                // must be realized on main. Keep ScrollingCoordinator's
                // independent scroll-map update for that source case.
                pac.DirectlySetScrollOffset(ElementId::new(value), &offset, context.layer);
            }
        }
        Ok(())
    }
}

impl<R: PaintSkiaBackend> crate::third_party::blink::renderer::platform::graphics::compositing::layers_as_json::LayersAsJsonColor for CpuMainPaintServices<R>{fn ColorNameForLayoutTree(&self,color:Color4f)->String{self.platform.ColorNameForLayoutTree(color)}}

// Source: cc/trees/tree_synchronizer.cc TRACE_EVENT scopes, which
// FinishCommit runs on impl with the main services lent to the commit.
// Moved from compositor/src/runtime/paint_services.rs.
impl<R: PaintSkiaBackend> crate::commit::main_callbacks::SynchronizationTrace
    for CpuMainPaintServices<R>
{
    fn TraceScope(&self, name: &str, count: Option<usize>) {
        self.Platform().TraceScope("cc", name, count)
    }
}
