//! Field/native-value binding for the sole canonical discardable-image map.
//! The map owns its generated occurrences and spatial index; the backend and
//! canvas exist only for this Generate call and never retain compositor owners.
use super::*;
use crate::adapter::image_animation::AnimatedImageMetadata as CpuAnimatedImageMetadata;
use crate::{
    adapter::{
        geometry::{Path as _, Rect, RectF, ToEnclosingRect, Vector2dF},
        image_animation::*,
        prioritized_tile::ScrollOffsetMap,
    },
    cc::base::rtree::RTree,
    runtime::skia::SkRegion,
};
use paint::cc::paint::{
    discardable_image_map::*,
    paint_op_buffer_iterator::{PaintOpType, PaintOpView},
};
use skia::{
    include::{core::SkClipOp::SkClipOp, utils::SkNoDrawCanvas::SkNoDrawCanvas},
    src::core::{
        SkCanvas::SkCanvas,
        SkM44::SkM44,
        SkRRect::SkRRect,
        SkRect::{SkIRect as SkRegionIRect, SkRect as SkRRectBounds},
    },
};
use std::{collections::BTreeMap, sync::Arc};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ImageRectValue(Rect);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ImageFloatRectValue(RectF);

impl ImageRect for ImageRectValue {
    fn IsEmpty(self) -> bool {
        self.0.IsEmpty()
    }
    fn Union(&mut self, r: Self) {
        self.0.Union(r.0)
    }
}
impl ImageFloatRect for ImageFloatRectValue {
    type Rect = ImageRectValue;
    fn MakeWH(w: i32, h: i32) -> Self {
        Self(RectF::new(0., 0., w as f32, h as f32))
    }
    fn FromRect(r: ImageRectValue) -> Self {
        Self(RectF::new(
            r.0.x as f32,
            r.0.y as f32,
            r.0.width as f32,
            r.0.height as f32,
        ))
    }
    fn roundOut(self) -> ImageRectValue {
        ImageRectValue(ToEnclosingRect(self.0))
    }
}
#[derive(Clone)]
pub(crate) struct ImageMapMatrix(SkM44);
fn matrix_col_major(values: [f32; 16]) -> SkM44 {
    SkM44::ColMajor(&values)
}
fn skrect(r: RectF) -> SkRRectBounds {
    {
        SkRRectBounds::MakeLTRB(r.x(), r.y(), r.right(), r.bottom())
    }
}
fn frect(r: SkRRectBounds) -> RectF {
    {
        RectF::new(r.fLeft, r.fTop, r.fRight - r.fLeft, r.fBottom - r.fTop)
    }
}
fn matrix_map_rect(matrix: &SkM44, rect: SkRRectBounds) -> SkRRectBounds {
    {
        matrix.asM33().mapRect_value(&rect)
    }
}
fn scalar_length(x: f32, y: f32) -> f32 {
    {
        skia::src::core::SkPoint::SkPoint::Length(x, y)
    }
}
fn matrix_pre_scale(matrix: &mut SkM44, x: f32, y: f32) {
    {
        matrix.preScale(x, y);
    }
}
fn toggle_path_inverse(path: &mut skia::src::core::SkPath::SkPath) {
    path.toggleInverseFillType();
}
#[derive(Clone, Copy)]
enum StrokeBoundsCap {
    Butt,
    Round,
    Square,
}
#[derive(Clone, Copy)]
enum StrokeBoundsJoin {
    Miter,
    Round,
    Bevel,
}
fn stroke_inflation_radius(
    join: StrokeBoundsJoin,
    miter: f32,
    cap: StrokeBoundsCap,
    width: f32,
) -> f32 {
    {
        use skia::src::core::{
            SkPaint::{Cap, Join},
            SkStrokeRec::SkStrokeRec,
        };
        let join = match join {
            StrokeBoundsJoin::Miter => Join::kMiter_Join,
            StrokeBoundsJoin::Round => Join::kRound_Join,
            StrokeBoundsJoin::Bevel => Join::kBevel_Join,
        };
        let cap = match cap {
            StrokeBoundsCap::Butt => Cap::kButt_Cap,
            StrokeBoundsCap::Round => Cap::kRound_Cap,
            StrokeBoundsCap::Square => Cap::kSquare_Cap,
        };
        SkStrokeRec::GetInflationRadius(join, miter, cap, width)
    }
}
impl ImageMatrix<ImageFloatRectValue> for ImageMapMatrix {
    fn Identity() -> Self {
        Self(SkM44::default())
    }
    // Source: SkMatrix.cpp:559-599; SkMatrix.h:1126-1147. The legacy
    // RectToRect entry used by cc returns identity for an empty source rect.
    fn RectToRect(s: ImageFloatRectValue, d: ImageFloatRectValue, center: bool) -> Self {
        let s = s.0;
        let d = d.0;
        if s.IsEmpty() {
            return Self::Identity();
        }
        let mut sx = d.width() / s.width();
        let mut sy = d.height() / s.height();
        let mut x_larger = false;
        if center {
            if sx > sy {
                x_larger = true;
                sx = sy;
            } else {
                sy = sx;
            }
        }
        let mut tx = d.x() - s.x() * sx;
        let mut ty = d.y() - s.y() * sy;
        if center {
            let diff = if x_larger {
                d.width() - s.width() * sy
            } else {
                d.height() - s.height() * sy
            } / 2.;
            if x_larger {
                tx += diff;
            } else {
                ty += diff;
            }
        }
        Self(matrix_col_major([
            sx, 0., 0., 0., 0., sy, 0., 0., 0., 0., 1., 0., tx, ty, 0., 1.,
        ]))
    }
    fn Multiply(&self, r: &Self) -> Self {
        let mut m = self.0;
        m.preConcat(&r.0);
        Self(m)
    }
}
pub(crate) struct MapSpatialIndex(RTree<usize>);
impl ImageSpatialIndex<ImageRectValue> for MapSpatialIndex {
    fn Build(n: usize, rect: impl Fn(usize) -> ImageRectValue) -> Self {
        let mut t = RTree::new();
        t.Build(n, |i| rect(i).0, |i| i);
        Self(t)
    }
    fn Search(&self, rect: &ImageRectValue, out: &mut Vec<usize>) {
        self.0.Search(&rect.0, |i, _| out.push(*i));
    }
}
pub(crate) struct MapSequence(std::thread::ThreadId);
impl ImageMapSequenceChecker for MapSequence {
    fn Check(&self) {
        debug_assert_eq!(
            self.0,
            std::thread::current().id(),
            "source DCHECK_CALLED_ON_VALID_SEQUENCE"
        );
    }
}
pub(crate) struct NativeDiscardableBackend {
    pub trace: Arc<dyn CpuPaintAnalysisTrace>,
}
impl DiscardableImageTypes for NativeDiscardableBackend {
    type Id = PaintImageId;
    type ScrollOffsets = ScrollOffsetMap;
    type Rect = ImageRectValue;
    type FloatRect = ImageFloatRectValue;
    type Matrix = ImageMapMatrix;
    type FilterQuality = CpuFilterQuality;
    type DecodingMode = ImageDecodingMode;
    type WorkletInput = Arc<CpuWorkletInput>;
    type CompletionState = CompletionState;
    type FrameMetadata = FrameMetadata;
    type AnimationSequenceId = u32;
    type Image = CpuPaintImage;
    type DrawImage = CpuDrawImage;
    type SpatialIndex = MapSpatialIndex;
    type SequenceChecker = MapSequence;
    fn NewSequenceChecker() -> MapSequence {
        MapSequence(std::thread::current().id())
    }
    fn NewDrawImage(
        image: CpuPaintImage,
        dark: bool,
        source: ImageRectValue,
        quality: CpuFilterQuality,
        m: ImageMapMatrix,
    ) -> CpuDrawImage {
        let source = source.0;
        // Source: cc/paint/draw_image.cc:14-27,50-61; native scale extraction
        // preserves signed diagonal scale until an affine component appears.
        let mut scale = Vector2dF::new(m.0.rc(0, 0), m.0.rc(1, 1));
        let mut decomposable = true;
        if m.0.rc(0, 1) != 0.
            || m.0.rc(1, 0) != 0.
            || m.0.rc(3, 0) != 0.
            || m.0.rc(3, 1) != 0.
            || m.0.rc(3, 3) != 1.
        {
            // Source external primitive: SkMatrix.cpp:1479-1499 and
            // SkScalar.h:98-104. No projection/approximate decomposition.
            let sx = scalar_length(m.0.rc(0, 0), m.0.rc(1, 0));
            let sy = scalar_length(m.0.rc(0, 1), m.0.rc(1, 1));
            if m.0.rc(3, 0) != 0.
                || m.0.rc(3, 1) != 0.
                || m.0.rc(3, 3) != 1.
                || !sx.is_finite()
                || !sy.is_finite()
                || sx.abs() <= 1. / 4096.
                || sy.abs() <= 1. / 4096.
            {
                scale = Vector2dF::new(1., 1.);
                decomposable = false;
            } else {
                scale = Vector2dF::new(sx, sy);
            }
        }
        CpuDrawImage {
            image,
            source_rect: source,
            filter_quality: quality,
            use_dark_mode: dark,
            scale,
            matrix_is_decomposable: decomposable,
            frame_index: kDefaultFrameIndex,
            target: None,
        }
    }
    // Source: cc/paint/paint_image.cc:91-102.
    fn ConservativeDecodingMode(a: ImageDecodingMode, b: ImageDecodingMode) -> ImageDecodingMode {
        if a == ImageDecodingMode::kSync || b == ImageDecodingMode::kSync {
            return ImageDecodingMode::kSync;
        }
        if a == ImageDecodingMode::kUnspecified || b == ImageDecodingMode::kUnspecified {
            return ImageDecodingMode::kUnspecified;
        }
        debug_assert_eq!(a, ImageDecodingMode::kAsync);
        debug_assert_eq!(b, ImageDecodingMode::kAsync);
        ImageDecodingMode::kAsync
    }
}
impl DiscardablePaintImage<NativeDiscardableBackend> for CpuPaintImage {
    fn IsTextureBacked(&self) -> bool {
        false
    } // This concrete owner is a CPU bitmap.
    fn stable_id(&self) -> PaintImageId {
        self.0.stable_id()
    }
    fn width(&self) -> i32 {
        self.0.size().width()
    }
    fn height(&self) -> i32 {
        self.0.size().height()
    }
    fn IsPaintWorklet(&self) -> bool {
        self.0.WorkletInput().is_some()
    }
    fn GetPaintWorkletInput(&self) -> Arc<CpuWorkletInput> {
        Arc::new(
            self.0
                .WorkletInput()
                .expect("native worklet subtype")
                .clone(),
        )
    }
    fn IsLazyGenerated(&self) -> bool {
        self.0.IsLazyGenerated()
    }
    fn decoding_mode(&self) -> ImageDecodingMode {
        ImageDecodingMode::kSync
    } // Source paint_image.h:505; this producer has no use-site override.
    fn ShouldAnimate(&self) -> bool {
        self.0.ShouldAnimate()
    }
    fn completion_state(&self) -> CompletionState {
        if self.0.IsComplete() {
            CompletionState::kDone
        } else {
            CompletionState::kPartiallyDone
        }
    }
    fn GetFrameMetadata(&self) -> Vec<FrameMetadata> {
        self.0
            .animation_metadata()
            .expect("animated image must own a real timeline")
            .frames
            .clone()
    }
    fn repetition_count(&self) -> i32 {
        self.0
            .animation_metadata()
            .expect("animated subtype")
            .repetition_count
    }
    fn reset_animation_sequence_id(&self) -> u32 {
        self.0
            .animation_metadata()
            .expect("animated subtype")
            .reset_animation_sequence_id
    }
    fn sync_animation_target_id(&self) -> PaintImageId {
        self.0
            .animation_metadata()
            .expect("animated subtype")
            .sync_animation_target_id
    }
    fn sync_animation_sequence_id(&self) -> u32 {
        self.0
            .animation_metadata()
            .expect("animated subtype")
            .sync_animation_sequence_id
    }
}
pub(crate) struct MapCanvas(core::pin::Pin<Box<SkNoDrawCanvas<'static>>>);
pub(crate) fn make_canvas(r: Rect) -> MapCanvas {
    {
        MapCanvas(SkNoDrawCanvas::with_bounds(&SkRegionIRect::make_ltrb(
            r.x,
            r.y,
            r.right(),
            r.bottom(),
        )))
    }
}
impl MapCanvas {
    pub(crate) fn LocalClipBounds(&self) -> Option<Rect> {
        {
            let r = self.0.as_ref().canvas_ref().getLocalClipBounds();
            (!r.isEmpty()).then(|| ToEnclosingRect(frect(r)))
        }
    }
    fn save(&mut self) {
        unsafe {
            self.0.as_mut().canvas().save();
        }
    }
    fn restore(&mut self) {
        unsafe {
            let mut canvas = self.0.as_mut().canvas();
            let canvas = core::ptr::NonNull::from(canvas.as_mut().get_unchecked_mut());
            SkCanvas::restore_raw(canvas)
                .expect("SkNoDrawCanvas restore cannot execute raster stages");
        }
    }
    fn restore_to_count(&mut self, count: i32) {
        unsafe {
            let mut canvas = self.0.as_mut().canvas();
            let canvas = core::ptr::NonNull::from(canvas.as_mut().get_unchecked_mut());
            SkCanvas::restoreToCount_raw(canvas, count)
                .expect("SkNoDrawCanvas restore cannot execute raster stages");
        }
    }
    fn translate(&mut self, x: f32, y: f32) {
        unsafe {
            self.0.as_mut().canvas().translate(x, y);
        }
    }
    fn concat(&mut self, matrix: &ImageMapMatrix) {
        unsafe {
            self.0.as_mut().canvas().concat44(&matrix.0);
        }
    }
    fn clip_rect(&mut self, rect: RectF, difference: bool, aa: bool) {
        unsafe {
            self.0.as_mut().canvas().clipRect(
                &skrect(rect),
                if difference {
                    SkClipOp::kDifference
                } else {
                    SkClipOp::kIntersect
                },
                aa,
            );
        }
    }
    fn clip_rrect_bounds(&mut self, rect: RectF, is_rect: bool, difference: bool, aa: bool) {
        unsafe {
            let bounds = skrect(rect);
            // SkNoPixelsDevice consumes a conservative bound and rect-ness for
            // this analysis. A nonzero radius preserves the source non-rect
            // classification without approximating any emitted pixels.
            let rrect = if is_rect {
                SkRRect::MakeRect(&bounds)
            } else {
                SkRRect::MakeRectXY(&bounds, 1.0, 1.0)
            };
            self.0.as_mut().canvas().clipRRect(
                &rrect,
                if difference {
                    SkClipOp::kDifference
                } else {
                    SkClipOp::kIntersect
                },
                aa,
            );
        }
    }
    fn clip_path(
        &mut self,
        path: &<SkRegion as crate::runtime::types::SkiaValueBackend>::Path,
        difference: bool,
        aa: bool,
    ) {
        unsafe {
            self.0.as_mut().canvas().clipPath(
                path,
                if difference {
                    SkClipOp::kDifference
                } else {
                    SkClipOp::kIntersect
                },
                aa,
            );
        }
    }
}
impl ImageGatheringCanvas<NativeDiscardableBackend> for MapCanvas {
    fn getSaveCount(&self) -> i32 {
        {
            self.0.as_ref().canvas_ref().getSaveCount()
        }
    }
    fn save(&mut self) {
        MapCanvas::save(self)
    }
    fn restore(&mut self) {
        MapCanvas::restore(self)
    }
    fn restoreToCount(&mut self, c: i32) {
        self.restore_to_count(c)
    }
    fn translate(&mut self, x: f32, y: f32) {
        MapCanvas::translate(self, x, y)
    }
    fn getLocalToDevice(&self) -> ImageMapMatrix {
        {
            ImageMapMatrix(self.0.as_ref().canvas_ref().getLocalToDevice())
        }
    }
    fn getDeviceClipBounds(&self) -> ImageRectValue {
        let r = self.0.as_ref().canvas_ref().getDeviceClipBounds();
        ImageRectValue(Rect::FromBounds(r.fLeft, r.fTop, r.fRight, r.fBottom))
    }
}
#[derive(Clone, Copy)]
pub(crate) enum MapShader<'a> {
    Converted(&'a CpuPaintShader),
    Native(&'a paint::paint_engine::PaintShader, &'a PaintRecord),
}
fn actual_shader_images(s: MapShader<'_>, visit: &mut dyn FnMut(&PaintRecord, u64)) {
    match s {
        MapShader::Converted(s) => {
            if let Some((r, id)) = super::native_metadata::shader_image_id(s) {
                visit(r, id)
            } else {
                let Some((source, mask)) = s.source() else {
                    return;
                };
                let shader = mask.map_or(source.op().paint_shader.as_ref(), |i| {
                    source.op().mask_layers[i].paint_shader.as_ref()
                });
                if let Some(s) = shader {
                    super::native_metadata::visit_shader_images(s, &mut |id| {
                        visit(source.record(), id)
                    });
                }
            }
        }
        MapShader::Native(s, r) => {
            super::native_metadata::visit_shader_images(s, &mut |id| visit(r, id))
        }
    }
}
impl DiscardableImageBackend<CpuPaintOpBuffer<SkRegion>> for NativeDiscardableBackend {
    type TraceScope = PaintTraceScope;
    type Canvas = MapCanvas;
    type Flags<'a> = CpuPaintOpRef<'a, SkRegion>;
    type Shader<'a> = MapShader<'a>;
    type Filter<'a> = &'a dyn paint::cc::paint::filter_operation::PaintFilter;
    type SkottieImages<'a> = std::iter::Empty<(CpuPaintImage, CpuFilterQuality)>;
    fn MakeNoDrawCanvas(&self, r: ImageRectValue) -> MapCanvas {
        make_canvas(r.0)
    }
    fn HasDiscardableImages(&self, b: &CpuPaintOpBuffer<SkRegion>) -> bool {
        b.canonical_metadata().has_discardable_images()
    }
    fn OpHasDiscardableImages(&self, op: CpuPaintOpRef<'_, SkRegion>) -> bool {
        match op {
            CpuPaintOpRef::Converted(CpuPaintOp::DrawRecord(r)) => {
                r.buffer().is_some_and(|b| self.HasDiscardableImages(b))
            }
            CpuPaintOpRef::Converted(CpuPaintOp::DrawScrollingContents { list, .. }) => {
                self.HasDiscardableImages(list.paint_op_buffer())
            }
            CpuPaintOpRef::Native { op, .. }
                if op.r#type == paint::paint_engine::DisplayItemType::kDrawImageRect =>
            {
                true
            }
            _ => super::native_metadata::flag_images(
                op,
                &mut crate::adapter::layer_tree_host_impl::ContentColorUsage::kSRGB,
            ),
        }
    }
    fn RasterState(
        &self,
        op: CpuPaintOpRef<'_, SkRegion>,
        c: &mut MapCanvas,
        original: &ImageMapMatrix,
    ) {
        raster_state(op, c, original)
    }
    fn ComputePaintRect(
        &self,
        op: CpuPaintOpRef<'_, SkRegion>,
        clip: ImageRectValue,
        ctm: &ImageMapMatrix,
    ) -> ImageRectValue {
        ImageRectValue(paint::cc::paint::paint_op::ComputePaintRect(
            &PaintRectProjection {
                op,
                clip: clip.0,
                ctm,
            },
        ))
    }
    fn Flags<'a>(&'a self, op: CpuPaintOpRef<'a, SkRegion>) -> Self::Flags<'a> {
        op
    }
    fn FlagShader<'a>(&'a self, op: Self::Flags<'a>) -> Option<MapShader<'a>> {
        match op {
            CpuPaintOpRef::Native { op, record } => op
                .paint_shader
                .as_ref()
                .map(|s| MapShader::Native(s, record)),
            _ => super::native_metadata::flags(op)
                .and_then(|f| f.shader.as_deref())
                .map(MapShader::Converted),
        }
    }
    fn FlagFilter<'a>(&'a self, op: Self::Flags<'a>) -> Option<Self::Filter<'a>> {
        super::native_metadata::flags(op).and_then(|f| f.image_filter.as_deref())
    }
    fn FlagQuality(&self, op: Self::Flags<'_>) -> CpuFilterQuality {
        self.ImageQuality(op)
    }
    fn FlagUseDarkMode(&self, op: Self::Flags<'_>) -> bool {
        super::native_metadata::flags(op).is_some_and(|f| f.useDarkModeForImage())
    }
    fn PreserveDiscardableImageMapQuality(&self) -> bool {
        true
    }
    fn Image<'a>(&'a self, op: CpuPaintOpRef<'a, SkRegion>) -> CpuPaintImage {
        match op {
            CpuPaintOpRef::Native { op, record } => {
                CpuPaintImage::FromPaintResource(record, op.resource_id)
                    .expect("validated native image")
            }
            _ => unreachable!("source image subtype"),
        }
    }
    fn ImageQuality(&self, op: CpuPaintOpRef<'_, SkRegion>) -> CpuFilterQuality {
        if let Some(f) = super::native_metadata::flags(op) {
            return match f.getFilterQuality() {
                paint::cc::paint::paint_flags::FilterQuality::kNone => CpuFilterQuality::kNone,
                paint::cc::paint::paint_flags::FilterQuality::kLow => CpuFilterQuality::kLow,
                paint::cc::paint::paint_flags::FilterQuality::kMedium => CpuFilterQuality::kMedium,
                paint::cc::paint::paint_flags::FilterQuality::kHigh => CpuFilterQuality::kHigh,
            };
        }
        match op {
            CpuPaintOpRef::Native { op, .. } => match op.image_interpolation_quality {
                foundation::graphics_enums::InterpolationQuality::kInterpolationNone => {
                    CpuFilterQuality::kNone
                }
                foundation::graphics_enums::InterpolationQuality::kInterpolationLow => {
                    CpuFilterQuality::kLow
                }
                _ => CpuFilterQuality::kMedium,
            },
            _ => unreachable!("source flags subtype"),
        }
    }
    fn ImageSourceRect(&self, op: CpuPaintOpRef<'_, SkRegion>) -> ImageFloatRectValue {
        ImageFloatRectValue(match op {
            CpuPaintOpRef::Native { op, .. } => RectF::new(
                op.source_rect.x as f32,
                op.source_rect.y as f32,
                op.source_rect.width as f32,
                op.source_rect.height as f32,
            ),
            _ => unreachable!("source image subtype"),
        })
    }
    fn ImageDestinationRect(&self, op: CpuPaintOpRef<'_, SkRegion>) -> ImageFloatRectValue {
        ImageFloatRectValue(match op {
            CpuPaintOpRef::Native { op, .. } => RectF::new(
                op.rect.x as f32,
                op.rect.y as f32,
                op.rect.width as f32,
                op.rect.height as f32,
            ),
            _ => unreachable!("source image subtype"),
        })
    }
    fn SkottieImages<'a>(&'a self, _: CpuPaintOpRef<'a, SkRegion>) -> Self::SkottieImages<'a> {
        unreachable!("this CPU family has no Skottie op")
    }
    fn SkottieNativeSize(&self, _: CpuPaintOpRef<'_, SkRegion>) -> ImageFloatRectValue {
        unreachable!("no Skottie subtype")
    }
    fn ScrollingBuffer<'a>(
        &'a self,
        op: CpuPaintOpRef<'a, SkRegion>,
    ) -> &'a CpuPaintOpBuffer<SkRegion> {
        match op {
            CpuPaintOpRef::Converted(CpuPaintOp::DrawScrollingContents { list, .. }) => {
                list.paint_op_buffer()
            }
            _ => unreachable!("scrolling subtype"),
        }
    }
    fn ScrollingOffset(
        &self,
        op: CpuPaintOpRef<'_, SkRegion>,
        offsets: &ScrollOffsetMap,
    ) -> (f32, f32) {
        match op {
            CpuPaintOpRef::Converted(CpuPaintOp::DrawScrollingContents { element, .. }) => {
                let v = offsets
                    .get(element)
                    .expect("source CHECK(scroll offset present)");
                (v.x(), v.y())
            }
            _ => unreachable!("scrolling subtype"),
        }
    }
    fn ShaderHasDiscardableImages(&self, s: MapShader<'_>) -> bool {
        let mut any = false;
        actual_shader_images(s, &mut |_, _| any = true);
        any
    }
    fn ShaderType(&self, s: MapShader<'_>) -> ShaderType {
        if matches!(s, MapShader::Converted(CpuPaintShader::ImagePattern { .. }))
            || matches!(s, MapShader::Native(shader, _) if shader.resource_id != 0 && shader.pattern.is_none())
        {
            ShaderType::Image
        } else {
            ShaderType::PaintRecord
        }
    }
    fn ShaderImage(&self, s: MapShader<'_>) -> CpuPaintImage {
        match s {
            MapShader::Converted(s) => {
                let (r, id) =
                    super::native_metadata::shader_image_id(s).expect("image shader subtype");
                CpuPaintImage::FromPaintResource(r, id).expect("validated image")
            }
            MapShader::Native(shader, record)
                if shader.pattern.is_none() && shader.resource_id != 0 =>
            {
                CpuPaintImage::FromPaintResource(record, shader.resource_id)
                    .expect("validated native image shader")
            }
            _ => unreachable!("image shader subtype"),
        }
    }
    fn ShaderLocalMatrix(&self, s: MapShader<'_>) -> ImageMapMatrix {
        match s {
            MapShader::Converted(s) => {
                let Some((source, mask)) = s.source() else {
                    if let CpuPaintShader::PaintRecord { local_matrix, .. } = s {
                        return ImageMapMatrix(matrix_col_major(local_matrix.as_ref().map_or(
                            [
                                1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
                            ],
                            |m| std::array::from_fn(|i| m.native().rc(i % 4, i / 4) as f32),
                        )));
                    }
                    unreachable!("exhaustive shader subtype");
                };
                let op = source.op();
                let (sx, sy, px, py) = mask.map_or(
                    (
                        op.tile_scale.x,
                        op.tile_scale.y,
                        op.tile_phase.x,
                        op.tile_phase.y,
                    ),
                    |i| {
                        let l = &op.mask_layers[i];
                        (l.tile_scale.x, l.tile_scale.y, l.tile_rect.x, l.tile_rect.y)
                    },
                );
                let mut m = SkM44::default();
                m.preTranslate(px as f32, py as f32, 0.);
                matrix_pre_scale(&mut m, sx as f32, sy as f32);
                ImageMapMatrix(m)
            }
            MapShader::Native(s, _) => {
                ImageMapMatrix(matrix_col_major(s.transform.values.map(|v| v as f32)))
            }
        }
    }
    fn ShaderHasNoAnimatedImages(&self, s: MapShader<'_>) -> bool {
        let mut animated = false;
        actual_shader_images(s, &mut |r, id| {
            animated |= CpuPaintImage::FromPaintResource(r, id)
                .expect("validated actual image")
                .0
                .ShouldAnimate()
        });
        !animated
    }
    fn ShaderRasterizationTileRect(
        &self,
        _: MapShader<'_>,
        _: &ImageMapMatrix,
    ) -> Option<ImageFloatRectValue> {
        unreachable!("animated pattern records require their actual timeline/record owner and are rejected before PAC")
    }
    fn ShaderRecord<'a>(&'a self, _: MapShader<'a>) -> &'a CpuPaintOpBuffer<SkRegion> {
        unreachable!("no accepted animated pattern owner")
    }
    fn ShaderSetHasAnimatedImages(&self, _: MapShader<'_>, _: bool) {
        unreachable!("no accepted animated pattern owner")
    }
    fn FilterHasDiscardableImages(&self, f: Self::Filter<'_>) -> bool {
        super::native_metadata::filter_has_images(f)
    }
    fn FilterHasNoAnimatedImages(&self, f: Self::Filter<'_>) -> bool {
        !super::native_metadata::filter_has_images(f)
    }
    fn FilterSnapshotWithImages(
        &self,
        _: Self::Filter<'_>,
        _: &mut dyn FnMut(CpuPaintImage, ImageFloatRectValue, CpuFilterQuality),
    ) {
        unreachable!("reference/reflection image owner is a typed PaintInput blocker")
    }
    fn FilterSetHasAnimatedImages(&self, _: Self::Filter<'_>, _: bool) {
        unreachable!("no accepted image-input filter")
    }
    fn TraceGenerate(&self) -> PaintTraceScope {
        self.trace.Scope("DiscardableImageMap::Generate")
    }
}
pub(crate) struct NativeImageMap {
    map: DiscardableImageMap<NativeDiscardableBackend>,
    animated: BTreeMap<PaintImageId, CpuAnimatedImageMetadata>,
}

struct PaintRectProjection<'a> {
    op: CpuPaintOpRef<'a, SkRegion>,
    clip: Rect,
    ctm: &'a ImageMapMatrix,
}
impl paint::cc::paint::paint_op::PaintRectBackend for PaintRectProjection<'_> {
    type Rect = Rect;
    type FloatRect = RectF;
    fn GetBounds(&self) -> Option<RectF> {
        let r = match self.op {
            CpuPaintOpRef::Native { op, .. } => {
                use paint::paint_engine::DisplayItemType as N;
                if matches!(op.r#type, N::kDrawPath | N::kStrokePath) {
                    let p = paint::paint_clip_path::PaintClipPath::from_authored(
                        &op.path,
                        &[],
                        op.even_odd,
                        Default::default(),
                    );
                    let b = p.single_path().expect("one native path").bounds();
                    RectF::new(b.x as f32, b.y as f32, b.width as f32, b.height as f32)
                } else if op.r#type == N::kDrawGlyphRun {
                    let r = op.glyph_ink_rect?;
                    RectF::new(r.x as f32, r.y as f32, r.width as f32, r.height as f32)
                } else {
                    RectF::new(
                        op.rect.x as f32,
                        op.rect.y as f32,
                        op.rect.width as f32,
                        op.rect.height as f32,
                    )
                }
            }
            CpuPaintOpRef::Converted(CpuPaintOp::DrawPath { path, .. }) => path.bounds(),
            CpuPaintOpRef::Converted(CpuPaintOp::DrawRect(r, ..)) => *r,
            CpuPaintOpRef::Converted(CpuPaintOp::DrawRRect(r, ..)) => r.rect,
            CpuPaintOpRef::Converted(CpuPaintOp::DrawDRRect { outer, .. }) => outer.rect,
            _ => return None,
        };
        Some(RectF::new(
            r.x().min(r.right()),
            r.y().min(r.bottom()),
            r.width().abs(),
            r.height().abs(),
        ))
    }
    fn Clip(&self) -> RectF {
        RectF::from(self.clip)
    }
    fn MapRect(&self, r: RectF) -> RectF {
        frect(matrix_map_rect(&self.ctm.0, skrect(r)))
    }
    fn HasFlags(&self) -> bool {
        self.op.IsPaintOpWithFlags()
    }
    fn IsFinite(&self, r: RectF) -> bool {
        [r.x(), r.y(), r.right(), r.bottom()]
            .iter()
            .all(|v| v.is_finite())
    }
    fn ComputeFastBounds(&self, mut r: RectF) -> Option<RectF> {
        use paint::cc::paint::paint_flags::{Cap, Join, Style};
        let (width, miter, cap, join) = if let Some(f) = super::native_metadata::flags(self.op) {
            // A native filter without a published fast-bounds primitive cannot
            // promise one. The canonical source takes its full-clip branch.
            if f.image_filter.is_some() {
                return None;
            }
            let width = if f.getStyle() == Style::kFill_Style {
                -1.
            } else {
                f.getStrokeWidth()
            };
            (
                width,
                f.getStrokeMiter(),
                match f.getStrokeCap() {
                    Cap::kButt_Cap => StrokeBoundsCap::Butt,
                    Cap::kRound_Cap => StrokeBoundsCap::Round,
                    Cap::kSquare_Cap => StrokeBoundsCap::Square,
                },
                match f.getStrokeJoin() {
                    Join::kMiter_Join => StrokeBoundsJoin::Miter,
                    Join::kRound_Join => StrokeBoundsJoin::Round,
                    Join::kBevel_Join => StrokeBoundsJoin::Bevel,
                },
            )
        } else if let CpuPaintOpRef::Native { op, .. } = self.op {
            if !op.filters.is_empty() {
                return None;
            }
            use paint::paint_engine::{
                DisplayItemType as N, SvgStrokeLineCap as C, SvgStrokeLineJoin as J,
            };
            let stroke = matches!(
                op.r#type,
                N::kStrokePath
                    | N::kStrokeRect
                    | N::kStrokeEllipse
                    | N::kStrokeLine
                    | N::kStrokeWavyLine
            );
            (
                if stroke { op.stroke_width as f32 } else { -1. },
                op.miter_limit as f32,
                if op.round_cap {
                    StrokeBoundsCap::Round
                } else {
                    match op.svg_line_cap {
                        C::kButt => StrokeBoundsCap::Butt,
                        C::kRound => StrokeBoundsCap::Round,
                        C::kSquare => StrokeBoundsCap::Square,
                    }
                },
                match op.svg_line_join {
                    J::kMiter => StrokeBoundsJoin::Miter,
                    J::kRound => StrokeBoundsJoin::Round,
                    J::kBevel => StrokeBoundsJoin::Bevel,
                },
            )
        } else {
            return Some(r);
        };
        let radius = stroke_inflation_radius(join, miter, cap, width);
        r.Inset(-radius, -radius, -radius, -radius);
        Some(r)
    }
    fn Intersect(&self, a: &mut RectF, b: RectF) -> bool {
        let l = a.x().max(b.x());
        let t = a.y().max(b.y());
        let r = a.right().min(b.right());
        let bottom = a.bottom().min(b.bottom());
        if !(l < r && t < bottom) {
            return false;
        }
        *a = RectF::new(l, t, r - l, bottom - t);
        true
    }
    fn ToEnclosingRect(&self, r: RectF) -> Rect {
        ToEnclosingRect(r)
    }
    fn Empty(&self) -> Rect {
        Rect::default()
    }
    fn OutsetOne(&self, r: &mut Rect) {
        r.Inset(-1, -1, -1, -1);
    }
}
impl NativeImageMap {
    pub fn Generate(
        buffer: &CpuPaintOpBuffer<SkRegion>,
        bounds: Rect,
        offsets: &ScrollOffsetMap,
        decoding: Option<&mut BTreeMap<PaintImageId, ImageDecodingMode>>,
        worklets: Option<&mut CpuPaintWorkletInputs>,
        trace: Arc<dyn CpuPaintAnalysisTrace>,
    ) -> CpuDiscardableImageMap {
        let mut map = DiscardableImageMap::Generate(
            buffer,
            ImageRectValue(bounds),
            offsets,
            decoding,
            worklets,
            &NativeDiscardableBackend { trace },
        );
        // One-way ownership conversion. The canonical field is consumed;
        // there is exactly one resident metadata map after this typed boundary.
        let animated = map
            .TakeAnimatedImagesMetadata()
            .into_iter()
            .map(|(id, m)| {
                (
                    id,
                    CpuAnimatedImageMetadata {
                        paint_image_id: m.paint_image_id,
                        sync_animation_target_id: m.sync_animation_target_id,
                        sync_animation_sequence_id: m.sync_animation_sequence_id,
                        completion_state: m.completion_state,
                        frames: m.frames,
                        repetition_count: m.repetition_count,
                        reset_animation_sequence_id: m.reset_animation_sequence_id,
                    },
                )
            })
            .collect();
        CpuDiscardableImageMap(Arc::new(Self { map, animated }))
    }
}
impl CpuDiscardableImageMapValue for NativeImageMap {
    fn empty(&self) -> bool {
        self.map.empty()
    }
    fn animated_images_metadata(&self) -> &BTreeMap<PaintImageId, CpuAnimatedImageMetadata> {
        let _ = self.map.animated_images_metadata();
        &self.animated
    }
    fn GetRectsForImage(&self, id: PaintImageId) -> &[Rect] {
        let values = self.map.GetRectsForImage(id);
        // ImageRectValue is a transparent ownership-boundary wrapper.
        unsafe { std::slice::from_raw_parts(values.as_ptr().cast::<Rect>(), values.len()) }
    }
    fn GetDiscardableImagesInRect(&self, rect: &Rect) -> Vec<&CpuDrawImage> {
        self.map.GetDiscardableImagesInRect(&ImageRectValue(*rect))
    }
}

/// Actual SkNoDrawCanvas state operations shared by analysis and recording.
pub(crate) fn raster_state(
    op: CpuPaintOpRef<'_, SkRegion>,
    c: &mut MapCanvas,
    _original: &ImageMapMatrix,
) {
    use paint::paint_engine::DisplayItemType as N;
    match op {
        CpuPaintOpRef::Native { op, .. } => match op.r#type {
            N::kSave
            | N::kSaveLayer
            | N::kSaveLayerAlpha
            | N::kSaveLayerBlend
            | N::kSaveLayerDstIn
            | N::kSaveLayerFilter
            | N::kBeginMask => c.save(),
            N::kRestore | N::kEndMask => c.restore(),
            N::kConcat => c.concat(&ImageMapMatrix(matrix_col_major(
                op.transform.values.map(|v| v as f32),
            ))),
            N::kClipRect | N::kClipOutRect => c.clip_rect(
                RectF::new(
                    op.rect.x as f32,
                    op.rect.y as f32,
                    op.rect.width as f32,
                    op.rect.height as f32,
                ),
                op.r#type == N::kClipOutRect,
                op.antialias,
            ),
            N::kClipRoundedRect | N::kClipOutRoundedRect => c.clip_rrect_bounds(
                RectF::new(
                    op.rect.x as f32,
                    op.rect.y as f32,
                    op.rect.width as f32,
                    op.rect.height as f32,
                ),
                false,
                op.r#type == N::kClipOutRoundedRect,
                op.antialias,
            ),
            N::kClipPath => {
                let p = paint::paint_clip_path::PaintClipPath::from_authored(
                    &op.path,
                    &[],
                    op.even_odd,
                    Default::default(),
                );
                let mut p = <SkRegion as crate::runtime::types::SkiaValueBackend>::FromPaintPath(
                    p.single_path().unwrap(),
                );
                if p.is_inverse_fill_type() != op.inverse_winding {
                    toggle_path_inverse(&mut p);
                }
                c.clip_path(&p, false, op.antialias)
            }
            _ => unreachable!("source native state subtype"),
        },
        CpuPaintOpRef::Converted(op) => match op {
            CpuPaintOp::Save
            | CpuPaintOp::SaveLayer { .. }
            | CpuPaintOp::SaveLayerAlpha { .. }
            | CpuPaintOp::SaveLayerFilters { .. } => c.save(),
            CpuPaintOp::Restore => c.restore(),
            CpuPaintOp::Translate(x, y) => c.translate(*x, *y),
            CpuPaintOp::Concat(m) => c.concat(&ImageMapMatrix(matrix_col_major(
                m.native().GetColMajor().map(|v| v as f32),
            ))),
            CpuPaintOp::ClipRect {
                rect,
                difference,
                antialias,
            } => c.clip_rect(*rect, *difference, *antialias),
            CpuPaintOp::ClipRRect(r, aa) => c.clip_rrect_bounds(
                r.rect,
                r.corner_radii.iter().all(|v| v.x() == 0. && v.y() == 0.),
                false,
                *aa,
            ),
            CpuPaintOp::ClipPath {
                path,
                difference,
                antialias,
            } => {
                let p = path.single_path().expect("validated path union capability");
                c.clip_path(p, *difference, *antialias)
            }
            CpuPaintOp::Noop => {}
            _ => unreachable!("draw op is not a state op"),
        },
    }
}
