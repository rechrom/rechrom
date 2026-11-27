//! Thin actual Skia no-pixels/value binding for canonical Paint solid analysis.
//! Classification remains entirely in paint_model::cc::paint::solid_color_analyzer.
#![allow(non_snake_case)]
use super::{CpuPaintOp, CpuPaintOpBuffer, CpuPaintOpRef};
use crate::skia_region::SkRegion;
use paint_model::cc::paint::blend_mode::BlendMode;
use paint_model::cc::paint::solid_color_analyzer::*;
use skia::{
    include::{core::SkClipOp::SkClipOp, utils::SkNoDrawCanvas::SkNoDrawCanvas},
    src::core::{SkCanvas::SkCanvas, SkM44::SkM44, SkRect::SkRect as Rect},
};
use std::sync::Arc;
pub trait CpuPaintAnalysisTrace: Send + Sync {
    fn Scope(&self, name: &'static str) -> super::PaintTraceScope;
}
pub struct CpuSolidColorBackend {
    trace: Arc<dyn CpuPaintAnalysisTrace>,
}
impl CpuSolidColorBackend {
    pub fn new(trace: Arc<dyn CpuPaintAnalysisTrace>) -> Self {
        Self { trace }
    }
}
#[derive(Clone)]
pub struct Matrix(pub(crate) SkM44);
struct MatrixRef<'a>(&'a SkM44);
impl crate::geometry::Matrix44 for MatrixRef<'_> {
    fn rc(&self, r: usize, c: usize) -> f32 {
        self.0.rc(r, c)
    }
}
impl AnalysisMatrix for Matrix {
    fn Preserves2DAxisAlignment(&self) -> bool {
        compositor_tree::math_util::MathUtil::SkM44Preserves2DAxisAlignment(&MatrixRef(&self.0))
    }
    fn invert(&self) -> Option<Self> {
        {
            let mut inverse = SkM44::default();
            self.0.invert(&mut inverse).then_some(Self(inverse))
        }
    }
    fn map(&self, x: f32, y: f32, z: f32, w: f32) -> [f32; 4] {
        {
            let mapped = self.0.map(x, y, z, w);
            [mapped.x, mapped.y, mapped.z, mapped.w]
        }
    }
}
/// The analysis canvas of one SolidColorAnalyzer::DetermineIfSolidColor call.
///
/// cc/paint/solid_color_analyzer.cc:232-233 constructs an `SkNoDrawCanvas`
/// on the stack for every analysis; that is a few stores in C++. The
/// translated canvas is a pinned heap owner whose construction allocates the
/// canvas, its MCRec storage and its SkNoPixelsDevice, and whose drop
/// releases them again. Each thread therefore keeps its one idle canvas and
/// re-initializes it with Skia's own reuse entry point,
/// `SkNoDrawCanvas::resetCanvas` (include/utils/SkNoDrawCanvas.h:30;
/// src/core/SkCanvas.cpp:287-302: restore to the base save, reset the root
/// no-pixels device to the new bounds, reset the MCRec matrix and clip).
/// The analysis observes exactly the state of a newly constructed canvas.
pub struct NoDrawCanvas(core::mem::ManuallyDrop<core::pin::Pin<Box<SkNoDrawCanvas<'static>>>>);
thread_local! {
    static IDLE_NO_DRAW_CANVAS: core::cell::Cell<Option<core::pin::Pin<Box<SkNoDrawCanvas<'static>>>>> =
        const { core::cell::Cell::new(None) };
}
impl NoDrawCanvas {
    fn with_size(width: i32, height: i32) -> Self {
        let idle: Option<core::pin::Pin<Box<SkNoDrawCanvas<'static>>>> =
            IDLE_NO_DRAW_CANVAS.try_with(core::cell::Cell::take).ok().flatten();
        if let Some(mut canvas) = idle {
            // Safety: the idle canvas is the original pinned SkNoDrawCanvas
            // allocation built by SkNoDrawCanvas::new, so its root device is
            // the complete SkNoPixelsDevice, and no device or canvas
            // reference to it is outstanding while it sits idle.
            if unsafe { canvas.as_mut().resetCanvas(width, height) }.is_ok() {
                return Self(core::mem::ManuallyDrop::new(canvas));
            }
        }
        Self(core::mem::ManuallyDrop::new(SkNoDrawCanvas::new(width, height)))
    }
    fn pinned(&self) -> core::pin::Pin<&SkNoDrawCanvas<'static>> {
        (*self.0).as_ref()
    }
    fn pinned_mut(&mut self) -> core::pin::Pin<&mut SkNoDrawCanvas<'static>> {
        (*self.0).as_mut()
    }
}
impl Drop for NoDrawCanvas {
    fn drop(&mut self) {
        // Safety: the field is taken exactly once, here, and never read again.
        let canvas = unsafe { core::mem::ManuallyDrop::take(&mut self.0) };
        // A canvas that cannot be parked (thread-local storage already torn
        // down) is dropped with the closure.
        let _ = IDLE_NO_DRAW_CANVAS.try_with(move |idle| drop(idle.replace(Some(canvas))));
    }
}
fn bounds(r: AnalysisRect) -> Rect {
    Rect::MakeLTRB(r.left, r.top, r.right, r.bottom)
}
fn analysis(r: Rect) -> AnalysisRect {
    let (left, top, right, bottom) = (r.fLeft, r.fTop, r.fRight, r.fBottom);
    AnalysisRect {
        left,
        top,
        right,
        bottom,
    }
}
fn matrix_col_major(values: [f32; 16]) -> SkM44 {
    return SkM44::ColMajor(&values);
}
impl NoDrawCanvas {
    fn as_source(&self) -> &SkCanvas<'static> {
        self.pinned().canvas_ref()
    }
    fn save(&mut self) {
        unsafe {
            self.pinned_mut().canvas().save();
        }
    }
    fn restore(&mut self) {
        unsafe {
            let mut canvas = self.pinned_mut().canvas();
            let address = core::ptr::NonNull::from(canvas.as_mut().get_unchecked_mut());
            SkCanvas::restore_raw(address).expect("SkNoDrawCanvas restore");
        }
    }
    fn restore_to_count(&mut self, count: i32) {
        while self.as_source().getSaveCount() > count.max(1) {
            self.restore();
        }
    }
    fn translate_by(&mut self, x: f32, y: f32) {
        unsafe {
            self.pinned_mut().canvas().translate(x, y);
        }
    }
    fn concat_m44(&mut self, matrix: &SkM44) {
        unsafe {
            self.pinned_mut().canvas().concat44(matrix);
        }
    }
    fn clip_rect(&mut self, rect: Rect, difference: bool, aa: bool) {
        unsafe {
            self.pinned_mut().canvas().clipRect(
                &rect,
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
impl AnalysisCanvas for NoDrawCanvas {
    type Matrix = Matrix;
    fn isClipRect(&self) -> bool {
        self.as_source().isClipRect()
    }
    fn getDeviceClipBounds(&self) -> Option<AnalysisRect> {
        let r = self.as_source().getDeviceClipBounds();
        let empty = r.is_empty();
        (!empty).then(|| AnalysisRect {
            left: r.fLeft as f32,
            top: r.fTop as f32,
            right: r.fRight as f32,
            bottom: r.fBottom as f32,
        })
    }
    fn getBaseLayerSize(&self) -> (i32, i32) {
        {
            let size = self.as_source().getBaseLayerSize();
            (size.width(), size.height())
        }
    }
    fn getLocalToDevice(&self) -> Matrix {
        Matrix(self.as_source().getLocalToDevice())
    }
    fn getLocalClipBounds(&self) -> Option<AnalysisRect> {
        {
            let rect = self.as_source().getLocalClipBounds();
            (!rect.isEmpty()).then(|| analysis(rect))
        }
    }
    fn getSaveCount(&self) -> i32 {
        self.as_source().getSaveCount()
    }
    fn restoreToCount(&mut self, count: i32) {
        self.restore_to_count(count)
    }
    fn translate(&mut self, x: f32, y: f32) {
        self.translate_by(x, y)
    }
    fn clipRect(&mut self, r: AnalysisRect, aa: bool) {
        self.clip_rect(bounds(r), false, aa)
    }
}
pub struct RectShape(AnalysisRect);
impl AnalysisShape for RectShape {
    fn contains(&self, r: AnalysisRect) -> bool {
        let a = self.0;
        a.left < a.right
            && a.top < a.bottom
            && r.left < r.right
            && r.top < r.bottom
            && a.left <= r.left
            && a.top <= r.top
            && a.right >= r.right
            && a.bottom >= r.bottom
    }
}
pub struct RRectShape(crate::skia_region::SkRoundedRect);
impl AnalysisShape for RRectShape {
    fn contains(&self, r: AnalysisRect) -> bool {
        {
            self.0.contains(&bounds(r))
        }
    }
}
pub struct Flags<'a>(CpuPaintOpRef<'a, SkRegion>);
fn blend(m: paint_model::paint_property_tree::PaintBlendMode) -> BlendMode {
    use paint_model::paint_property_tree::PaintBlendMode::*;
    match m {
        kNormal => BlendMode::kSrcOver,
        kMultiply => BlendMode::kMultiply,
        kScreen => BlendMode::kScreen,
        kOverlay => BlendMode::kOverlay,
        kDarken => BlendMode::kDarken,
        kLighten => BlendMode::kLighten,
        kColorDodge => BlendMode::kColorDodge,
        kColorBurn => BlendMode::kColorBurn,
        kHardLight => BlendMode::kHardLight,
        kSoftLight => BlendMode::kSoftLight,
        kDifference => BlendMode::kDifference,
        kExclusion => BlendMode::kExclusion,
        kHue => BlendMode::kHue,
        kSaturation => BlendMode::kSaturation,
        kColor => BlendMode::kColor,
        kLuminosity => BlendMode::kLuminosity,
        kPlusLighter => BlendMode::kPlus,
    }
}
impl AnalysisFlags for Flags<'_> {
    fn getBlendMode(&self) -> BlendMode {
        match self.0 {
            CpuPaintOpRef::PaintRecord { op, .. } => blend(op.blend_mode),
            CpuPaintOpRef::Converted(
                CpuPaintOp::DrawPath { flags: f, .. }
                | CpuPaintOp::DrawRRect(_, f)
                | CpuPaintOp::DrawRect(_, f)
                | CpuPaintOp::DrawDRRect { flags: f, .. },
            ) => f.getBlendMode(),
            _ => panic!("source flagged draw required"),
        }
    }
    fn HasShader(&self) -> bool {
        match self.0 {
            CpuPaintOpRef::PaintRecord { op, .. } => {
                op.paint_shader.is_some()
                    || matches!(
                        op.r#type,
                        paint_model::paint_op::PaintOpType::kDrawGradientRect
                            | paint_model::paint_op::PaintOpType::kDrawTiledGradient
                    )
            }
            CpuPaintOpRef::Converted(
                CpuPaintOp::DrawPath { flags: f, .. }
                | CpuPaintOp::DrawRRect(_, f)
                | CpuPaintOp::DrawRect(_, f)
                | CpuPaintOp::DrawDRRect { flags: f, .. },
            ) => f.shader.is_some(),
            _ => panic!("source flagged draw required"),
        }
    }
    fn getLooper(&self) -> bool {
        match self.0 {
            CpuPaintOpRef::PaintRecord { op, .. } => op.is_shadow,
            CpuPaintOpRef::Converted(
                CpuPaintOp::DrawPath { flags: f, .. }
                | CpuPaintOp::DrawRRect(_, f)
                | CpuPaintOp::DrawRect(_, f)
                | CpuPaintOp::DrawDRRect { flags: f, .. },
            ) => f.looper.is_some(),
            _ => false,
        }
    }
    fn getColorFilter(&self) -> bool {
        match self.0 {
            CpuPaintOpRef::Converted(
                CpuPaintOp::DrawPath { flags: f, .. }
                | CpuPaintOp::DrawRRect(_, f)
                | CpuPaintOp::DrawRect(_, f)
                | CpuPaintOp::DrawDRRect { flags: f, .. },
            ) => f.color_filter.is_some(),
            _ => false,
        }
    }
    fn getImageFilter(&self) -> bool {
        match self.0 {
            CpuPaintOpRef::PaintRecord { op, .. } => !op.canonical_filters.is_empty(),
            CpuPaintOpRef::Converted(
                CpuPaintOp::DrawPath { flags: f, .. }
                | CpuPaintOp::DrawRRect(_, f)
                | CpuPaintOp::DrawRect(_, f)
                | CpuPaintOp::DrawDRRect { flags: f, .. },
            ) => f.image_filter.is_some(),
            _ => panic!("source flagged draw required"),
        }
    }
    fn isFill(&self) -> bool {
        match self.0 {
            CpuPaintOpRef::PaintRecord { op, .. } => !matches!(
                op.r#type,
                paint_model::paint_op::PaintOpType::kStrokeRect
                    | paint_model::paint_op::PaintOpType::kStrokeEllipse
                    | paint_model::paint_op::PaintOpType::kStrokePath
                    | paint_model::paint_op::PaintOpType::kStrokeLine
                    | paint_model::paint_op::PaintOpType::kStrokeWavyLine
            ),
            CpuPaintOpRef::Converted(
                CpuPaintOp::DrawRect(_, f)
                | CpuPaintOp::DrawPath { flags: f, .. }
                | CpuPaintOp::DrawRRect(_, f)
                | CpuPaintOp::DrawDRRect { flags: f, .. },
            ) => f.getStyle() == paint_model::cc::paint::paint_flags::Style::kFill_Style,
            _ => panic!("source flagged shape required"),
        }
    }
    fn getColor4f(&self) -> Color {
        match self.0 {
            CpuPaintOpRef::PaintRecord { op, .. } => {
                let c = op.color;
                [c.red, c.green, c.blue, c.alpha]
            }
            CpuPaintOpRef::Converted(
                CpuPaintOp::DrawPath { flags: f, .. }
                | CpuPaintOp::DrawRRect(_, f)
                | CpuPaintOp::DrawRect(_, f)
                | CpuPaintOp::DrawDRRect { flags: f, .. },
            ) => {
                let c = f.getColor4f();
                [c.red, c.green, c.blue, c.alpha]
            }
            _ => panic!("source flagged draw required"),
        }
    }
    // Source: cc/paint/paint_flags.cc:93-115; paint_flags.h:52.
    fn nothingToDraw(&self) -> bool {
        if self.getLooper() {
            return false;
        }
        match self.getBlendMode() {
            BlendMode::kSrcOver
            | BlendMode::kSrcATop
            | BlendMode::kDstOut
            | BlendMode::kDstOver
            | BlendMode::kPlus => {
                if self.isFullyTransparent() {
                    return !self.getColorFilter() && !self.getImageFilter();
                }
            }
            BlendMode::kDst => return true,
            _ => {}
        }
        false
    }
    fn isFullyTransparent(&self) -> bool {
        self.getColor4f()[3] == 0.
    }
}
impl SolidColorBackend<CpuPaintOpBuffer<SkRegion>> for CpuSolidColorBackend {
    type Canvas = NoDrawCanvas;
    type Flags<'a> = Flags<'a>;
    type RectShape<'a> = RectShape;
    type RRectShape<'a> = RRectShape;
    type TraceScope = super::PaintTraceScope;
    fn MakeNoDrawCanvas(&self, w: i32, h: i32) -> NoDrawCanvas {
        NoDrawCanvas::with_size(w, h)
    }
    fn Flags<'a>(&'a self, op: CpuPaintOpRef<'a, SkRegion>) -> Flags<'a> {
        Flags(op)
    }
    fn Rect<'a>(&'a self, op: CpuPaintOpRef<'a, SkRegion>) -> RectShape {
        match op {
            CpuPaintOpRef::PaintRecord { op, .. } => RectShape(AnalysisRect {
                left: op.rect.x as f32,
                top: op.rect.y as f32,
                right: (op.rect.x + op.rect.width) as f32,
                bottom: (op.rect.y + op.rect.height) as f32,
            }),
            CpuPaintOpRef::Converted(CpuPaintOp::DrawRect(r, _)) => RectShape(AnalysisRect {
                left: r.x(),
                top: r.y(),
                right: r.right(),
                bottom: r.bottom(),
            }),
            _ => panic!("source DrawRect required"),
        }
    }
    fn RRect<'a>(&'a self, op: CpuPaintOpRef<'a, SkRegion>) -> RRectShape {
        use crate::geometry::{RRectF, RectF, Vector2dF};
        let value = match op {
            CpuPaintOpRef::PaintRecord { op, .. } => {
                let radii = op.corner_radii;
                RRectF {
                    rect: RectF::new(
                        op.rect.x as f32,
                        op.rect.y as f32,
                        op.rect.width as f32,
                        op.rect.height as f32,
                    ),
                    corner_radii: [
                        radii.top_left,
                        radii.top_right,
                        radii.bottom_right,
                        radii.bottom_left,
                    ]
                    .map(|radius| Vector2dF::new(radius.x as f32, radius.y as f32)),
                }
            }
            CpuPaintOpRef::Converted(
                CpuPaintOp::DrawRRect(value, _) | CpuPaintOp::ClipRRect(value, _),
            ) => *value,
            _ => panic!("source rounded rect op required"),
        };
        RRectShape(crate::skia_region::MakeSkRoundedRect(&value))
    }
    fn DrawColor(&self, op: CpuPaintOpRef<'_, SkRegion>) -> (Color, BlendMode) {
        match op {
            CpuPaintOpRef::Converted(CpuPaintOp::DrawColor { color, blend }) => {
                ([color.red, color.green, color.blue, color.alpha], *blend)
            }
            CpuPaintOpRef::Converted(CpuPaintOp::DrawTransparentColorSrc) => {
                ([0.; 4], BlendMode::kSrc)
            }
            _ => panic!("source DrawColor required"),
        }
    }
    fn ClipIsDifference(&self, op: CpuPaintOpRef<'_, SkRegion>) -> bool {
        match op {
            CpuPaintOpRef::PaintRecord { op, .. } => matches!(
                op.r#type,
                paint_model::paint_op::PaintOpType::kClipOutRect
            ),
            CpuPaintOpRef::Converted(CpuPaintOp::ClipRect { difference, .. }) => *difference,
            _ => panic!("source ClipRect required"),
        }
    }
    fn RasterState(
        &self,
        op: CpuPaintOpRef<'_, SkRegion>,
        canvas: &mut NoDrawCanvas,
        _original: &Matrix,
    ) {
        use paint_model::paint_op::PaintOpType as T;
        match op {
            CpuPaintOpRef::PaintRecord { op, .. } => match op.r#type {
                T::kSave => canvas.save(),
                T::kRestore | T::kEndMask => canvas.restore(),
                T::kConcat => {
                    canvas.concat_m44(&matrix_col_major(op.transform.values.map(|v| v as f32)))
                }
                T::kClipRect | T::kClipOutRect => canvas.clip_rect(
                    Rect::MakeLTRB(
                        op.rect.x as f32,
                        op.rect.y as f32,
                        (op.rect.x + op.rect.width) as f32,
                        (op.rect.y + op.rect.height) as f32,
                    ),
                    op.r#type == T::kClipOutRect,
                    op.antialias,
                ),
                _ => panic!("source analyzer state op required"),
            },
            CpuPaintOpRef::Converted(op) => match op {
                CpuPaintOp::Save => canvas.save(),
                CpuPaintOp::Restore => canvas.restore(),
                CpuPaintOp::Translate(x, y) => canvas.translate_by(*x, *y),
                CpuPaintOp::Concat(m) => canvas.concat_m44(&matrix_col_major(
                    m.as_source().GetColMajor().map(|v| v as f32),
                )),
                CpuPaintOp::ClipRect {
                    rect,
                    difference,
                    antialias,
                } => canvas.clip_rect(
                    Rect::MakeLTRB(rect.x(), rect.y(), rect.right(), rect.bottom()),
                    *difference,
                    *antialias,
                ),
                _ => panic!("source analyzer state op required"),
            },
        }
    }
    fn TraceCheckIfSolidShape(&self) -> super::PaintTraceScope {
        self.trace.Scope("SolidColorAnalyzer::CheckIfSolidShape")
    }
    fn TraceRRectClipCoversCanvas(&self) -> super::PaintTraceScope {
        self.trace
            .Scope("SolidColorAnalyzer::CheckIfRRectClipCoversCanvas")
    }
}
