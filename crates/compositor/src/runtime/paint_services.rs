//! Production Paint value capabilities. This owner holds independent raster
//! value/diagnostic leases, never a Page, Layer, PAC or PaintArtifact backpointer.
use super::{
    paint::{self, CpuGeometryMapper},
    types::{CpuCompositorTypes, CpuSkiaBackend},
};
use crate::{
    adapter::{
        geometry::Vector2dF, input_types::ElementId, main_layer::MainLayerServices,
        picture_layer::PictureLayerServices, recording_source::RecordingSourceServices,
        tree_sync::SynchronizationTrace,
    },
    cc::layers::recording_source::RecordingSource,
};
use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};

/// Explicit embedding feature values. There is deliberately no Default: the
/// embedding supplies the actual enabled runtime/feature-list state.
#[derive(Clone, Copy, Debug)]
pub struct CpuPaintFeatures {
    pub cc_slimming: bool,
    pub singular_transform_fix: bool,
    pub merge_fixed_layers: bool,
    pub merge_sticky_layers: bool,
    pub raster_inducing_scroll: bool,
    pub scrolling_contents_cull_rect: bool,
    pub remove_scroll_node_workaround: bool,
    pub selection_edge_visibility_uses_full_edge: bool,
    pub canvas_draw_element: bool,
    pub expensive_dchecks: bool,
    pub paint_under_invalidation_checking: bool,
    pub layer_merge_distance_limit: i32,
}

/// Only native diagnostics and the external SkPicture recording target. All
/// recording, invalidation, conversion and layerization policy stays canonical.
pub trait CpuPaintPlatform<R: CpuSkiaBackend>: Send + Sync {
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

pub struct CpuPaintServices<R: CpuSkiaBackend> {
    pub features: CpuPaintFeatures,
    values: Arc<dyn paint::CpuPaintValueServices<R>>,
    mapper: Arc<CpuGeometryMapper<R>>,
    platform: Arc<dyn CpuPaintPlatform<R>>,
}
impl<R: CpuSkiaBackend> CpuPaintServices<R> {
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
    pub(crate) fn PaintValues(&self) -> Arc<dyn paint::CpuPaintValueServices<R>> {
        self.values.clone()
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
impl<R: CpuSkiaBackend> MainLayerServices for CpuPaintServices<R> {
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
impl<R: CpuSkiaBackend> RecordingSourceServices<paint::CpuRecordingTypes<R>>
    for CpuPaintServices<R>
{
    fn CreateRasterSource(
        &self,
        recording: &RecordingSource<paint::CpuRecordingTypes<R>>,
    ) -> Arc<paint::CpuRasterSource<R>> {
        Arc::new(paint::CpuRasterSource::from_recording(recording))
    }
    fn TraceScope(&self, category: &str, name: &str, count: Option<usize>) {
        self.platform.TraceScope(category, name, count)
    }
}
impl<R: CpuSkiaBackend> PictureLayerServices<CpuCompositorTypes<R>> for CpuPaintServices<R> {
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
impl<R: CpuSkiaBackend> SynchronizationTrace for CpuPaintServices<R> {
    fn TraceScope(&self, name: &str, count: Option<usize>) {
        self.platform.TraceScope("cc", name, count)
    }
}
impl<R: CpuSkiaBackend>
    crate::adapter::paint_artifact_compositor::SynthesizedClipPaintServices<CpuCompositorTypes<R>>
    for CpuPaintServices<R>
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
        r: &skia::include::core::SkRRect::SkRRect,
        flags: paint::CpuPaintFlags<R>,
    ) {
        use crate::adapter::display_item_list::DisplayPaintOpBuffer;
        let b = r.getBounds();
        let rect = crate::adapter::geometry::RRectF {
            rect: crate::adapter::geometry::RectF::new(
                b.left,
                b.top,
                b.right - b.left,
                b.bottom - b.top,
            ),
            corner_radii: r.radii().map(|p| Vector2dF::new(p.x, p.y)),
        };
        list.push(paint::CpuPaintOp::DrawRRect(rect, flags));
    }
}
impl<R: CpuSkiaBackend>
    crate::adapter::solid_color_layer::SolidColorLayerServices<CpuCompositorTypes<R>>
    for CpuPaintServices<R>
{
    fn CreateSolidPictureRecorder(&self) -> paint::CpuPictureRecorder<R> {
        self.platform.CreatePictureRecorder()
    }
}
impl<R: CpuSkiaBackend> crate::adapter::pending_layer::PendingLayerServices<CpuCompositorTypes<R>>
    for CpuPaintServices<R>
where
    CpuCompositorTypes<R>: crate::adapter::pending_layer::PendingLayerTypes,
{
    fn CreateSolidColorLayer(
        &mut self,
        services: &dyn MainLayerServices,
    ) -> crate::cc::layers::layer::LayerRef<CpuCompositorTypes<R>> {
        crate::cc::layers::solid_color_layer::SolidColorLayer::Create(services)
    }
}

use crate::adapter::{
    geometry::{Point, PointF, RRectF, Rect, RectF},
    layer_tree_impl::Color4f,
    main_layer::LayerContext,
    paint_geometry::{ChunkGeometryMapper, MaskGeometryMapper},
    pending_layer::{PendingLayerColor, PendingLayerGeometry, PendingMappedVisualRect},
};
use crate::third_party::blink::renderer::platform::graphics::paint::{
    float_clip_rect::FloatClipRect, geometry_mapper::GeometryMapper,
    geometry_mapper_clip_cache::OverlayScrollbarClipBehavior,
};
type Root<R> = CpuCompositorTypes<R>;
// Source primitive: ui/gfx/color_utils.cc:577-614. Native SkColor channels are
// quantized before blending and ClampRound'ed afterwards, as in the source.
impl<R: CpuSkiaBackend> PendingLayerColor for CpuPaintServices<R> {
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
impl<R: CpuSkiaBackend> MaskGeometryMapper for CpuPaintServices<R> {
    type TransformPaintPropertyNode = paint::TransformNode<R>;
    fn SourceToDestinationRectFromRoot(&self, d: &Self::TransformPaintPropertyNode, r: &mut RectF) {
        GeometryMapper::<CpuGeometryMapper<R>>::SourceToDestinationRect(
            &paint::PropertyTreeState::<R>::default().transform(),
            d,
            r,
        )
    }
}
impl<R: CpuSkiaBackend>
    crate::adapter::content_layer_client_impl::ContentLayerClientServices<Root<R>>
    for CpuPaintServices<R>
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
                crate::adapter::geometry::ToEnclosingRect(RectF::new(
                    s.contents_rect.x as f32,
                    s.contents_rect.y as f32,
                    s.contents_rect.width as f32,
                    s.contents_rect.height as f32,
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
        layer: &crate::cc::layers::layer::Layer<Root<R>>,
        id: ElementId,
        c: &LayerContext<'_, Root<R>>,
    ) -> PointF {
        let trees = layer
            .layer_tree_host(c)
            .expect("source attached layer")
            .property_trees();
        trees.scroll_tree().current_scroll_offset(id, trees.state())
    }
}
impl<R: CpuSkiaBackend> crate::adapter::paint_chunks_to_cc_layer::PaintChunksServices<Root<R>>
    for CpuPaintServices<R>
{
    fn NativePaintRecord(
        &self,
        a: &Arc<paint::PaintArtifact>,
        item: &paint::DisplayItem,
    ) -> paint::CpuPaintRecord<R> {
        paint::CpuPaintRecord::from_native(
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
            let b = p.getBounds();
            rect.Union(RectF::new(
                b.left,
                b.top,
                b.right - b.left,
                b.bottom - b.top,
            ));
        }
        rect
    }
    fn PathIsRect(&self, p: &Arc<::paint::paint_clip_path::PaintClipPath>) -> Option<RectF> {
        let p = p.single_path()?;
        if p.isInverseFillType() {
            return None;
        }
        let b = p.isRect()?.rect;
        Some(RectF::new(
            b.left,
            b.top,
            b.right - b.left,
            b.bottom - b.top,
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
        f.setBlendMode(m.into())
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

impl<R: CpuSkiaBackend> PendingLayerGeometry<Root<R>> for CpuPaintServices<R> {
    fn VisibilityLimit(&self, s: &paint::PropertyTreeState<R>) -> Option<RectF> {
        GeometryMapper::<CpuGeometryMapper<R>>::VisibilityLimit(self.mapper.as_ref(), s)
    }
    fn MightOverlapForCompositing(
        &self,
        a: RectF,
        sa: &paint::PropertyTreeState<R>,
        b: RectF,
        sb: &paint::PropertyTreeState<R>,
    ) -> bool {
        GeometryMapper::<CpuGeometryMapper<R>>::MightOverlapForCompositing(
            self.mapper.as_ref(),
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
            self.mapper.as_ref(),
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
impl<R: CpuSkiaBackend> crate::adapter::paint_artifact_compositor::SynthesizedClipServices<Root<R>>
    for CpuPaintServices<R>
{
    fn NewUniqueObjectElementId(&self) -> ElementId {
        <Root<R> as crate::adapter::property_tree_manager::PropertyTreeManagerTypes>::NewUniqueElementId()
    }
    fn Projection(
        &self,
        s: &paint::TransformNode<R>,
        d: &paint::TransformNode<R>,
    ) -> super::geometry::Transform<R> {
        GeometryMapper::<CpuGeometryMapper<R>>::SourceToDestinationProjection(s, d)
    }
    fn ToSkRoundedRect(&self, r: &RRectF) -> skia::include::core::SkRRect::SkRRect {
        use skia::include::core::SkRRect::{SkRRect, SkRRectBounds, SkRRectVector};
        let mut result = SkRRect::default();
        result.setRectRadii(
            SkRRectBounds::MakeLTRB(r.rect.x(), r.rect.y(), r.rect.right(), r.rect.bottom()),
            r.corner_radii.map(|p| SkRRectVector::new(p.x(), p.y())),
        );
        result
    }
    fn OffsetSkRoundedRect(&self, r: &mut skia::include::core::SkRRect::SkRRect, v: Vector2dF) {
        r.offset(v.x(), v.y())
    }
    fn MapRect(&self, t: &super::geometry::Transform<R>, r: Rect) -> Rect {
        crate::adapter::draw_property::DrawTransform::MapRectInt(t, r)
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
        c.node().clip_path == *previous
    }
}

impl<R: CpuSkiaBackend>
    crate::adapter::paint_artifact_compositor::PaintArtifactCompositorServices<Root<R>>
    for CpuPaintServices<R>
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
        *ID.get_or_init(||<Root<R> as crate::adapter::property_tree_manager::PropertyTreeManagerTypes>::NewUniqueElementId())
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
    ) -> paint::CpuLayersAsJson<R> {
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
struct CpuPendingLayerServices<R: CpuSkiaBackend>(std::marker::PhantomData<fn() -> R>);
impl<R: CpuSkiaBackend> crate::adapter::pending_layer::PendingLayerServices<Root<R>>
    for CpuPendingLayerServices<R>
{
    fn CreateSolidColorLayer(
        &mut self,
        services: &dyn MainLayerServices,
    ) -> crate::cc::layers::layer::LayerRef<Root<R>> {
        crate::cc::layers::solid_color_layer::SolidColorLayer::Create(services)
    }
}
impl<R: CpuSkiaBackend> CpuPaintServices<R> {
    /// Engine-internal construction of the actual canonical PAC. The caller
    /// attaches RootLayer to CpuMainLayerHost using its live main context.
    pub(crate) fn CreateCompositor(&self,callbacks:std::rc::Weak<dyn crate::cc::trees::property_tree::ScrollCallbacks>,c:&mut LayerContext<'_,Root<R>>)->crate::third_party::blink::renderer::platform::graphics::compositing::paint_artifact_compositor::PaintArtifactCompositor<Root<R>>{
        crate::third_party::blink::renderer::platform::graphics::compositing::paint_artifact_compositor::PaintArtifactCompositor::new(callbacks,c,self)
    }
    /// Validation happens before any canonical owner is mutated. Input retains
    /// exactly the supplied artifact Arc; all layerization stays inside PAC.
    pub(crate) fn UpdateCompositor(
        &self,
        pac:&mut crate::third_party::blink::renderer::platform::graphics::compositing::paint_artifact_compositor::PaintArtifactCompositor<Root<R>>,
        artifact: Arc<paint::PaintArtifact>,
        previous_artifact: Option<&Arc<paint::PaintArtifact>>,
        requests: Vec<Box<super::host_owners::ViewTransitionRequest>>,
        c: &mut LayerContext<'_, Root<R>>,
        sequence: &mut i32,
    ) -> Result<(), paint::PaintInputError> {
        let input = self.PrepareInput(artifact)?;
        let viewport = input.ViewportProperties::<R>();
        let mut pending = CpuPendingLayerServices(std::marker::PhantomData);
        let mut context =
            crate::adapter::paint_artifact_compositor::PaintArtifactCompositorContext {
                layer: c,
                pending: &mut pending,
                services: self,
                property_tree_sequence_number: sequence,
            };
        if let Some(previous) = previous_artifact {
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
            let property_trees_changed = previous.chunks.len() == input.artifact().chunks.len()
                && previous
                    .chunks
                    .iter()
                    .zip(&input.artifact().chunks)
                    .any(|(old, new)| !old.properties.same_nodes(&new.properties));
            if property_trees_changed {
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
        Ok(())
    }
}
// A non-test compile assertion proves the actual default family, rather than
// proving only a generic where-clause that no production type implements.
const _: fn() = || {
    fn require<
        B: crate::adapter::paint_artifact_compositor::PaintArtifactCompositorTypes
            + crate::adapter::picture_layer_impl::PictureLayerImplTypes
            + crate::adapter::picture_layer_tiling::PictureLayerTilingTypes
            + crate::adapter::tile_manager::TileManagerTypes,
    >() {
    }
    require::<Root<skia::include::core::SkRegion::SkRegion>>();
};

impl<R:CpuSkiaBackend> crate::third_party::blink::renderer::platform::graphics::compositing::layers_as_json::LayersAsJsonColor for CpuPaintServices<R>{fn ColorNameForLayoutTree(&self,color:Color4f)->String{self.platform.ColorNameForLayoutTree(color)}}
