//! Source solid-color classification. The backend supplies only no-pixels
//! SkCanvas/gfx value primitives and typed resident op projections.
#![allow(non_snake_case)]
use super::paint_op_buffer_iterator::{
    CompositeIterator, PaintOpBufferView, PaintOpType, PaintOpView,
};
use skia::BlendMode;

pub type Color = [f32; 4];
#[derive(Clone, Copy, Debug)]
pub struct AnalysisRect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
impl AnalysisRect {
    pub fn width(self) -> f32 {
        self.right - self.left
    }
    pub fn height(self) -> f32 {
        self.bottom - self.top
    }
}
pub trait AnalysisShape {
    fn contains(&self, rect: AnalysisRect) -> bool;
}
pub trait AnalysisMatrix: Clone {
    fn Preserves2DAxisAlignment(&self) -> bool;
    fn invert(&self) -> Option<Self>;
    fn map(&self, x: f32, y: f32, z: f32, w: f32) -> [f32; 4];
}
pub trait AnalysisCanvas {
    type Matrix: AnalysisMatrix;
    fn isClipRect(&self) -> bool;
    fn getDeviceClipBounds(&self) -> Option<AnalysisRect>;
    fn getBaseLayerSize(&self) -> (i32, i32);
    fn getLocalToDevice(&self) -> Self::Matrix;
    fn getLocalClipBounds(&self) -> Option<AnalysisRect>;
    fn getSaveCount(&self) -> i32;
    fn restoreToCount(&mut self, count: i32);
    fn translate(&mut self, x: f32, y: f32);
    fn clipRect(&mut self, rect: AnalysisRect, antialias: bool);
}
pub trait AnalysisFlags {
    fn getBlendMode(&self) -> BlendMode;
    fn HasShader(&self) -> bool;
    fn getLooper(&self) -> bool;
    fn getColorFilter(&self) -> bool;
    fn getImageFilter(&self) -> bool;
    fn isFill(&self) -> bool;
    fn getColor4f(&self) -> Color;
    fn nothingToDraw(&self) -> bool;
    fn isFullyTransparent(&self) -> bool;
}
/// Native no-pixels device and field projections. RasterState must execute the
/// supplied state op against this canvas, including original-CTM SetMatrix;
/// it cannot classify solids, skip ops, scan pixels, or traverse a record.
pub trait SolidColorBackend<B: PaintOpBufferView + ?Sized> {
    type Canvas: AnalysisCanvas;
    type Flags<'a>: AnalysisFlags
    where
        Self: 'a,
        B: 'a;
    type RectShape<'a>: AnalysisShape
    where
        Self: 'a,
        B: 'a;
    type RRectShape<'a>: AnalysisShape
    where
        Self: 'a,
        B: 'a;
    fn MakeNoDrawCanvas(&self, width: i32, height: i32) -> Self::Canvas;
    fn Flags<'a>(&'a self, op: B::Op<'a>) -> Self::Flags<'a>;
    fn Rect<'a>(&'a self, op: B::Op<'a>) -> Self::RectShape<'a>;
    fn RRect<'a>(&'a self, op: B::Op<'a>) -> Self::RRectShape<'a>;
    fn DrawColor(&self, op: B::Op<'_>) -> (Color, BlendMode);
    fn ClipIsDifference(&self, op: B::Op<'_>) -> bool;
    fn RasterState(
        &self,
        op: B::Op<'_>,
        canvas: &mut Self::Canvas,
        original: &<Self::Canvas as AnalysisCanvas>::Matrix,
    );
    type TraceScope;
    fn TraceCheckIfSolidShape(&self) -> Self::TraceScope;
    fn TraceRRectClipCoversCanvas(&self) -> Self::TraceScope;
}
// Source: cc/paint/solid_color_analyzer.cc:19-42.
fn DoSrcOverAlphaBlend(src: Color, dst: Color) -> Color {
    if src[3] == 0. {
        return dst;
    }
    if src[3] == 1. {
        return src;
    }
    let out_alpha = src[3] + dst[3] * (1. - src[3]);
    if out_alpha == 0. {
        return [0.; 4];
    }
    let inverse_out_alpha = 1. / out_alpha;
    let src_weight = src[3] * inverse_out_alpha;
    let dst_weight = dst[3] * (1. - src[3]) * inverse_out_alpha;
    [
        src[0] * src_weight + dst[0] * dst_weight,
        src[1] * src_weight + dst[1] * dst_weight,
        src[2] * src_weight + dst[2] * dst_weight,
        out_alpha,
    ]
}
// Source: cc/paint/solid_color_analyzer.cc:44-59.
fn ActsLikeClear(mode: BlendMode, src_alpha: f32) -> bool {
    match mode {
        BlendMode::kClear => true,
        BlendMode::kSrc
        | BlendMode::kSrcIn
        | BlendMode::kDstIn
        | BlendMode::kSrcOut
        | BlendMode::kDstATop => src_alpha == 0.,
        BlendMode::kDstOut => src_alpha == 1.,
        _ => false,
    }
}
// Source: cc/paint/solid_color_analyzer.cc:61-84.
fn IsSolidColorBlendMode(mode: BlendMode) -> bool {
    mode == BlendMode::kSrc || mode == BlendMode::kSrcOver
}
fn IsSolidColorPaint(flags: &impl AnalysisFlags) -> bool {
    let solid = IsSolidColorBlendMode(flags.getBlendMode())
        && !flags.HasShader()
        && !flags.getLooper()
        && !flags.getColorFilter()
        && !flags.getImageFilter()
        && flags.isFill();
    #[cfg(target_os = "macos")]
    let solid = solid && flags.getColor4f()[3] == 1.;
    solid
}
// Source: cc/paint/solid_color_analyzer.cc:86-122.
fn IsFullQuad(canvas: &impl AnalysisCanvas, drawn_shape: &impl AnalysisShape) -> bool {
    if !canvas.isClipRect() {
        return false;
    }
    let Some(clip) = canvas.getDeviceClipBounds() else {
        return false;
    };
    let (width, height) = canvas.getBaseLayerSize();
    if !(clip.left <= 0.
        && clip.top <= 0.
        && clip.right >= width as f32
        && clip.bottom >= height as f32)
    {
        return false;
    }
    let matrix = canvas.getLocalToDevice();
    if !matrix.Preserves2DAxisAlignment() {
        return false;
    }
    let Some(inverse) = matrix.invert() else {
        return false;
    };
    let upper_left = inverse.map(clip.left, clip.top, 0., 1.);
    let lower_right = inverse.map(clip.right, clip.bottom, 0., 1.);
    drawn_shape.contains(AnalysisRect {
        left: upper_left[0],
        top: upper_left[1],
        right: lower_right[0],
        bottom: lower_right[1],
    })
}
struct RectShape(AnalysisRect);
impl AnalysisShape for RectShape {
    fn contains(&self, r: AnalysisRect) -> bool {
        let s = self.0;
        s.left < s.right
            && s.top < s.bottom
            && r.left < r.right
            && r.top < r.bottom
            && s.left <= r.left
            && s.top <= r.top
            && s.right >= r.right
            && s.bottom >= r.bottom
    }
}
// Source: cc/paint/solid_color_analyzer.cc:124-144.
fn CalculateSolidColor(src: Color, blend: BlendMode, dst: &mut Color, is_solid: &mut bool) {
    if blend == BlendMode::kSrc {
        *dst = src;
        *is_solid = true
    } else {
        debug_assert_eq!(blend, BlendMode::kSrcOver);
        if src[3] == 1. || *is_solid {
            *dst = DoSrcOverAlphaBlend(src, *dst);
            *is_solid = true
        }
    }
}
// Source: cc/paint/solid_color_analyzer.cc:146-181.
fn CheckIfSolidColor(
    canvas: &impl AnalysisCanvas,
    color: Color,
    blend: BlendMode,
    is_solid: &mut bool,
    is_transparent: &mut bool,
    out: &mut Color,
) {
    let Some(rect) = canvas.getLocalClipBounds() else {
        *is_transparent = false;
        *is_solid = false;
        return;
    };
    let covers = IsFullQuad(canvas, &RectShape(rect));
    if covers && ActsLikeClear(blend, color[3]) {
        *is_transparent = true
    } else if color[3] != 0. || blend != BlendMode::kSrc {
        *is_transparent = false
    }
    let candidate = covers && IsSolidColorBlendMode(blend);
    #[cfg(target_os = "macos")]
    let candidate = candidate && color[3] == 1.;
    if candidate {
        CalculateSolidColor(color, blend, out, is_solid)
    } else {
        *is_solid = false
    }
}
// Source: cc/paint/solid_color_analyzer.cc:183-210.
fn CheckIfSolidShape(
    canvas: &impl AnalysisCanvas,
    shape: &impl AnalysisShape,
    flags: &impl AnalysisFlags,
    is_solid: &mut bool,
    is_transparent: &mut bool,
    color: &mut Color,
) {
    if flags.nothingToDraw() {
        return;
    }
    let covers = IsFullQuad(canvas, shape);
    let blend = flags.getBlendMode();
    if covers && ActsLikeClear(blend, flags.getColor4f()[3]) {
        *is_transparent = true
    } else if !flags.isFullyTransparent() || blend != BlendMode::kSrc {
        *is_transparent = false
    }
    if covers && IsSolidColorPaint(flags) {
        CalculateSolidColor(flags.getColor4f(), blend, color, is_solid)
    } else {
        *is_solid = false
    }
}
// Source: cc/paint/solid_color_analyzer.cc:212-219.
fn CheckIfRRectClipCoversCanvas(canvas: &impl AnalysisCanvas, rrect: &impl AnalysisShape) -> bool {
    IsFullQuad(canvas, rrect)
}
// Source: cc/paint/solid_color_analyzer.h:17-28.
pub struct SolidColorAnalyzer;
impl SolidColorAnalyzer {
    // Source: cc/paint/solid_color_analyzer.cc:221-256.
    pub fn DetermineIfSolidColor<B: PaintOpBufferView + ?Sized, A: SolidColorBackend<B>>(
        buffer: &B,
        rect: foundation::gfx::Rect,
        max_ops_to_analyze: i32,
        offsets: Option<&[usize]>,
        backend: &A,
    ) -> Option<Color> {
        if buffer.Used() == 0 || offsets.is_some_and(|o| o.is_empty()) {
            return Some([0.; 4]);
        }
        let mut is_solid = true;
        let mut is_transparent = true;
        let mut color = [0.; 4];
        struct Frame<'a, B: PaintOpBufferView + ?Sized, M> {
            iter: CompositeIterator<'a, B>,
            original_ctm: M,
            save_count: i32,
        }
        let mut canvas = backend.MakeNoDrawCanvas(rect.size().width(), rect.size().height());
        canvas.translate(-rect.origin().x() as f32, -rect.origin().y() as f32);
        canvas.clipRect(
            AnalysisRect {
                left: rect.origin().x() as f32,
                top: rect.origin().y() as f32,
                right: (rect.origin().x() + rect.size().width()) as f32,
                bottom: (rect.origin().y() + rect.size().height()) as f32,
            },
            false,
        );
        let mut stack = Vec::with_capacity(2);
        stack.push(Frame {
            iter: CompositeIterator::new(buffer, offsets),
            original_ctm: canvas.getLocalToDevice(),
            save_count: canvas.getSaveCount(),
        });
        let mut num_draw_ops = 0;
        // Source: cc/paint/solid_color_analyzer.cc:257-371.
        while let Some(frame) = stack.last_mut() {
            if !frame.iter.is_valid() {
                canvas.restoreToCount(frame.save_count);
                stack.pop();
                if let Some(parent) = stack.last_mut() {
                    parent.iter.Advance()
                }
                continue;
            }
            let op = frame.iter.get();
            use PaintOpType::*;
            match op.GetType() {
                kDrawRecord => {
                    // Rust's source-empty record has no allocation. This is
                    // exactly the native empty CompositeIterator's zero visits.
                    if let Some(record) = op.RecordBuffer() {
                        stack.push(Frame {
                            iter: CompositeIterator::new(record, None),
                            original_ctm: canvas.getLocalToDevice(),
                            save_count: canvas.getSaveCount(),
                        });
                        continue;
                    }
                }
                kDrawDRRect
                | kDrawImage
                | kDrawImageRect
                | kDrawIRect
                | kDrawLine
                | kDrawLineLite
                | kDrawArc
                | kDrawArcLite
                | kDrawOval
                | kDrawPath
                | kDrawScrollingContents
                | kDrawVertices => return None,
                kDrawRRect => {
                    num_draw_ops += 1;
                    if num_draw_ops > max_ops_to_analyze {
                        return None;
                    }
                    let _trace = backend.TraceCheckIfSolidShape();
                    CheckIfSolidShape(
                        &canvas,
                        &backend.RRect(op),
                        &backend.Flags(op),
                        &mut is_solid,
                        &mut is_transparent,
                        &mut color,
                    )
                }
                kDrawSkottie | kDrawSlug | kDrawTextBlob | kSaveLayer | kSaveLayerAlpha
                | kSaveLayerFilters | kClipPath => return None,
                kClipRRect => {
                    let _trace = backend.TraceRRectClipCoversCanvas();
                    if !CheckIfRRectClipCoversCanvas(&canvas, &backend.RRect(op)) {
                        return None;
                    }
                }
                kDrawRect => {
                    num_draw_ops += 1;
                    if num_draw_ops > max_ops_to_analyze {
                        return None;
                    }
                    let _trace = backend.TraceCheckIfSolidShape();
                    CheckIfSolidShape(
                        &canvas,
                        &backend.Rect(op),
                        &backend.Flags(op),
                        &mut is_solid,
                        &mut is_transparent,
                        &mut color,
                    )
                }
                kDrawColor => {
                    num_draw_ops += 1;
                    if num_draw_ops > max_ops_to_analyze {
                        return None;
                    }
                    let (c, b) = backend.DrawColor(op);
                    CheckIfSolidColor(
                        &canvas,
                        c,
                        b,
                        &mut is_solid,
                        &mut is_transparent,
                        &mut color,
                    )
                }
                kClipRect => {
                    if backend.ClipIsDifference(op) {
                        return None;
                    }
                    backend.RasterState(op, &mut canvas, &frame.original_ctm)
                }
                kAnnotate | kCustomData | kSetNodeId | kNoop => {}
                kConcat | kScale | kSetMatrix | kRestore | kRotate | kSave | kTranslate => {
                    backend.RasterState(op, &mut canvas, &frame.original_ctm)
                }
            }
            // Source: cc/paint/solid_color_analyzer.cc:372-379.
            frame.iter.Advance();
        }
        if is_transparent {
            return Some([0.; 4]);
        }
        if is_solid {
            return Some(color);
        }
        None
    }
}
// Source scaffolding: cc/paint/solid_color_analyzer.h:1-16,30-32;
// cc/paint/solid_color_analyzer.cc:1-18,381: includes/namespaces/guards.
// Translation status: complete
