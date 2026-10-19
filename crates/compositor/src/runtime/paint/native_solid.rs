//! Thin actual Skia no-pixels/value binding for canonical Paint solid analysis.
//! Classification remains entirely in paint::cc::paint::solid_color_analyzer.
#![allow(non_snake_case)]
use crate::runtime::paint::{CpuPaintOp, CpuPaintOpBuffer, CpuPaintOpRef};
use paint::cc::paint::solid_color_analyzer::*;
use skia::{
    include::core::{
        SkRRect::{SkRRect, SkRRectBounds, SkRRectVector},
        SkRegion::SkRegion,
    },
    src::{core::SkM44::SkM44, utils::SkNoDrawCanvas::SkNoDrawCanvas},
    BlendMode,
};
use std::sync::Arc;
pub trait CpuPaintAnalysisTrace: Send + Sync {
    fn Scope(&self, name: &'static str) -> crate::runtime::paint::PaintTraceScope;
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
impl crate::adapter::geometry::Matrix44 for MatrixRef<'_> {
    fn rc(&self, r: usize, c: usize) -> f32 {
        self.0.rc(r, c)
    }
}
impl AnalysisMatrix for Matrix {
    fn Preserves2DAxisAlignment(&self) -> bool {
        crate::cc::base::math_util::MathUtil::SkM44Preserves2DAxisAlignment(&MatrixRef(&self.0))
    }
    fn invert(&self) -> Option<Self> {
        self.0.invert().map(Self)
    }
    fn map(&self, x: f32, y: f32, z: f32, w: f32) -> [f32; 4] {
        self.0.map(x, y, z, w)
    }
}
pub struct NoDrawCanvas(pub(crate) SkNoDrawCanvas);
fn bounds(r: AnalysisRect) -> SkRRectBounds {
    SkRRectBounds::MakeLTRB(r.left, r.top, r.right, r.bottom)
}
fn analysis(r: SkRRectBounds) -> AnalysisRect {
    AnalysisRect {
        left: r.left,
        top: r.top,
        right: r.right,
        bottom: r.bottom,
    }
}
impl AnalysisCanvas for NoDrawCanvas {
    type Matrix = Matrix;
    fn isClipRect(&self) -> bool {
        self.0.isClipRect()
    }
    fn getDeviceClipBounds(&self) -> Option<AnalysisRect> {
        let r = self.0.getDeviceClipBounds();
        (!r.isEmpty()).then(|| AnalysisRect {
            left: r.fLeft as f32,
            top: r.fTop as f32,
            right: r.fRight as f32,
            bottom: r.fBottom as f32,
        })
    }
    fn getBaseLayerSize(&self) -> (i32, i32) {
        self.0.getBaseLayerSize()
    }
    fn getLocalToDevice(&self) -> Matrix {
        Matrix(self.0.getLocalToDevice())
    }
    fn getLocalClipBounds(&self) -> Option<AnalysisRect> {
        self.0.getLocalClipBounds().map(analysis)
    }
    fn getSaveCount(&self) -> i32 {
        self.0.getSaveCount()
    }
    fn restoreToCount(&mut self, count: i32) {
        self.0.restoreToCount(count)
    }
    fn translate(&mut self, x: f32, y: f32) {
        self.0.translate(x, y)
    }
    fn clipRect(&mut self, r: AnalysisRect, aa: bool) {
        self.0.clipRect(bounds(r), false, aa)
    }
}
pub struct RectShape(SkRRectBounds);
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
pub struct RRectShape(SkRRect);
impl AnalysisShape for RRectShape {
    fn contains(&self, r: AnalysisRect) -> bool {
        self.0.contains(bounds(r))
    }
}
pub struct Flags<'a>(CpuPaintOpRef<'a, SkRegion>);
fn native_blend(m: paint::paint_property_tree::PaintBlendMode) -> BlendMode {
    use paint::paint_property_tree::PaintBlendMode::*;
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
            CpuPaintOpRef::Native { op, .. } => native_blend(op.blend_mode),
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
            CpuPaintOpRef::Native { op, .. } => {
                op.paint_shader.is_some()
                    || matches!(
                        op.r#type,
                        paint::paint_engine::DisplayItemType::kDrawGradientRect
                            | paint::paint_engine::DisplayItemType::kDrawTiledGradient
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
            CpuPaintOpRef::Native { op, .. } => op.is_shadow,
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
            CpuPaintOpRef::Native { op, .. } => !op.filters.is_empty(),
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
            CpuPaintOpRef::Native { op, .. } => !matches!(
                op.r#type,
                paint::paint_engine::DisplayItemType::kStrokeRect
                    | paint::paint_engine::DisplayItemType::kStrokeEllipse
                    | paint::paint_engine::DisplayItemType::kStrokePath
                    | paint::paint_engine::DisplayItemType::kStrokeLine
                    | paint::paint_engine::DisplayItemType::kStrokeWavyLine
            ),
            CpuPaintOpRef::Converted(
                CpuPaintOp::DrawRect(_, f)
                | CpuPaintOp::DrawPath { flags: f, .. }
                | CpuPaintOp::DrawRRect(_, f)
                | CpuPaintOp::DrawDRRect { flags: f, .. },
            ) => f.getStyle() == paint::cc::paint::paint_flags::Style::kFill_Style,
            _ => panic!("source flagged shape required"),
        }
    }
    fn getColor4f(&self) -> Color {
        match self.0 {
            CpuPaintOpRef::Native { op, .. } => {
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
                [c.fR, c.fG, c.fB, c.fA]
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
    type TraceScope = crate::runtime::paint::PaintTraceScope;
    fn MakeNoDrawCanvas(&self, w: i32, h: i32) -> NoDrawCanvas {
        NoDrawCanvas(SkNoDrawCanvas::new(w, h))
    }
    fn Flags<'a>(&'a self, op: CpuPaintOpRef<'a, SkRegion>) -> Flags<'a> {
        Flags(op)
    }
    fn Rect<'a>(&'a self, op: CpuPaintOpRef<'a, SkRegion>) -> RectShape {
        match op {
            CpuPaintOpRef::Native { op, .. } => RectShape(SkRRectBounds::MakeLTRB(
                op.rect.x as f32,
                op.rect.y as f32,
                (op.rect.x + op.rect.width) as f32,
                (op.rect.y + op.rect.height) as f32,
            )),
            CpuPaintOpRef::Converted(CpuPaintOp::DrawRect(r, _)) => {
                RectShape(SkRRectBounds::MakeLTRB(r.x(), r.y(), r.right(), r.bottom()))
            }
            _ => panic!("source DrawRect required"),
        }
    }
    fn RRect<'a>(&'a self, op: CpuPaintOpRef<'a, SkRegion>) -> RRectShape {
        let mut r = SkRRect::default();
        match op {
            CpuPaintOpRef::Native { op, .. } => {
                let radii = op.corner_radii;
                r.setRectRadii(
                    SkRRectBounds::MakeLTRB(
                        op.rect.x as f32,
                        op.rect.y as f32,
                        (op.rect.x + op.rect.width) as f32,
                        (op.rect.y + op.rect.height) as f32,
                    ),
                    [
                        radii.top_left,
                        radii.top_right,
                        radii.bottom_right,
                        radii.bottom_left,
                    ]
                    .map(|r| SkRRectVector::new(r.x as f32, r.y as f32)),
                );
            }
            CpuPaintOpRef::Converted(
                CpuPaintOp::DrawRRect(value, _) | CpuPaintOp::ClipRRect(value, _),
            ) => r.setRectRadii(
                SkRRectBounds::MakeLTRB(
                    value.rect.x(),
                    value.rect.y(),
                    value.rect.right(),
                    value.rect.bottom(),
                ),
                value.corner_radii.map(|p| SkRRectVector::new(p.x(), p.y())),
            ),
            _ => panic!("source rounded rect op required"),
        }
        RRectShape(r)
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
            CpuPaintOpRef::Native { op, .. } => matches!(
                op.r#type,
                paint::paint_engine::DisplayItemType::kClipOutRect
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
        use paint::paint_engine::DisplayItemType as T;
        match op {
            CpuPaintOpRef::Native { op, .. } => match op.r#type {
                T::kSave => canvas.0.save(),
                T::kRestore | T::kEndMask => canvas.0.restore(),
                T::kConcat => canvas
                    .0
                    .concat(&SkM44::ColMajor(op.transform.values.map(|v| v as f32))),
                T::kClipRect | T::kClipOutRect => canvas.0.clipRect(
                    SkRRectBounds::MakeLTRB(
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
                CpuPaintOp::Save => canvas.0.save(),
                CpuPaintOp::Restore => canvas.0.restore(),
                CpuPaintOp::Translate(x, y) => canvas.0.translate(*x, *y),
                CpuPaintOp::Concat(m) => canvas
                    .0
                    .concat(&SkM44::ColMajor(m.native().GetColMajor().map(|v| v as f32))),
                CpuPaintOp::ClipRect {
                    rect,
                    difference,
                    antialias,
                } => canvas.0.clipRect(
                    SkRRectBounds::MakeLTRB(rect.x(), rect.y(), rect.right(), rect.bottom()),
                    *difference,
                    *antialias,
                ),
                _ => panic!("source analyzer state op required"),
            },
        }
    }
    fn TraceCheckIfSolidShape(&self) -> crate::runtime::paint::PaintTraceScope {
        self.trace.Scope("SolidColorAnalyzer::CheckIfSolidShape")
    }
    fn TraceRRectClipCoversCanvas(&self) -> crate::runtime::paint::PaintTraceScope {
        self.trace
            .Scope("SolidColorAnalyzer::CheckIfRRectClipCoversCanvas")
    }
}
