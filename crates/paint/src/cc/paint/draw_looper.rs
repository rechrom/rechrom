//! Native shadow-layer owner and painter-order application.
#![allow(non_snake_case, non_upper_case_globals)]
use super::{blend_mode::BlendMode, color::Color4f};
use std::sync::Arc;
// Source: cc/paint/draw_looper.h:27-39,72-89.
pub const kPostTransformFlag: u32 = 1;
pub const kOverrideAlphaFlag: u32 = 2;
pub const kDontModifyPaintFlag: u32 = 4;
const ALL_FLAGS: u32 = kPostTransformFlag | kOverrideAlphaFlag | kDontModifyPaintFlag;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layer {
    pub offset: Point,
    pub blur_sigma: f32,
    pub color: Color4f,
    pub flags: u32,
}
pub trait LooperCanvas {
    type Matrix;
    fn getSaveCount(&self) -> i32;
    fn save(&mut self);
    fn restoreToCount(&mut self, count: i32);
    fn getLocalToDevice(&self) -> Self::Matrix;
    fn setMatrixPostTranslate(&mut self, matrix: Self::Matrix, x: f32, y: f32);
    fn translate(&mut self, x: f32, y: f32);
}
pub trait LooperPaint: Clone {
    fn setAlpha(&mut self, alpha: u8);
    fn setMaskFilterNormalBlur(&mut self, sigma: f32, respect_ctm: bool);
    fn setColorFilterSRGBBlend(&mut self, color: Color4f, mode: BlendMode);
}
impl Layer {
    // Source: cc/paint/draw_looper.cc:22-42. The native backend performs only
    // SkMaskFilter::MakeBlur and SkColorFilters::Blend pure value operations.
    fn Apply<C: LooperCanvas, P: LooperPaint>(&self, canvas: &mut C, paint: &mut P) {
        if self.flags & kDontModifyPaintFlag == 0 {
            if self.flags & kOverrideAlphaFlag != 0 {
                paint.setAlpha(255)
            }
            if self.blur_sigma > 0. {
                paint.setMaskFilterNormalBlur(self.blur_sigma, self.flags & kPostTransformFlag == 0)
            }
            paint.setColorFilterSRGBBlend(self.color, BlendMode::kSrcIn);
        }
        if self.flags & kPostTransformFlag != 0 {
            let matrix = canvas.getLocalToDevice();
            canvas.setMatrixPostTranslate(matrix, self.offset.x, self.offset.y)
        } else {
            canvas.translate(self.offset.x, self.offset.y)
        }
    }
}
// Source: cc/paint/draw_looper.h:24-26,89-98; cc:19-20.
pub struct DrawLooper {
    layers_: Vec<Layer>,
}
impl DrawLooper {
    // Source: cc/paint/draw_looper.h:41-66.
    pub fn Apply<C: LooperCanvas, P: LooperPaint>(
        &self,
        canvas: &mut C,
        paint: &P,
        mut proc: impl FnMut(&mut C, P),
    ) {
        for layer in self.layers_.iter().rev() {
            let count = canvas.getSaveCount();
            canvas.save();
            let restore = CanvasRestore { canvas, count };
            let mut p = paint.clone();
            layer.Apply(&mut *restore.canvas, &mut p);
            proc(&mut *restore.canvas, p);
        }
    }
    // Source: cc/paint/draw_looper.cc:44-46.
    pub fn EqualsForTesting(&self, other: &Self) -> bool {
        self.layers_ == other.layers_
    }
    /// Borrow the actual native modifier list for a field-only Skia value
    /// conversion. This never records or traverses layers in the compositor.
    pub fn layers(&self) -> &[Layer] {
        &self.layers_
    }
}
impl super::paint_flags::PaintValueEquals for DrawLooper {
    fn EqualsForTesting(&self, other: &Self) -> bool {
        Self::EqualsForTesting(self, other)
    }
}
// Source: cc/paint/draw_looper.h:100-113; cc:48-49.
#[derive(Default)]
pub struct DrawLooperBuilder {
    layers_: Vec<Layer>,
}
impl DrawLooperBuilder {
    // Source: cc/paint/draw_looper.cc:51-54.
    pub fn AddUnmodifiedContent(&mut self, add_on_top: bool) {
        self.AddShadow(
            Point::default(),
            0.,
            Color4f::new(0., 0., 0., 1.),
            kDontModifyPaintFlag,
            add_on_top,
        )
    }
    // Source: cc/paint/draw_looper.cc:56-69.
    pub fn AddShadow(
        &mut self,
        offset: Point,
        blur_sigma: f32,
        color: Color4f,
        flags: u32,
        add_on_top: bool,
    ) {
        let layer = Layer {
            offset,
            blur_sigma,
            color,
            flags: flags & ALL_FLAGS,
        };
        if add_on_top {
            self.layers_.insert(0, layer)
        } else {
            self.layers_.push(layer)
        }
    }
    // Source: cc/paint/draw_looper.cc:71-73.
    pub fn Detach(&mut self) -> Arc<DrawLooper> {
        Arc::new(DrawLooper {
            layers_: std::mem::take(&mut self.layers_),
        })
    }
}
// Source scaffolding: cc/paint/draw_looper.h:1-23,115-117; .cc:1-18,75.
// Translation status: complete

// Native SkAutoCanvasRestore lifetime: restoration also happens during Rust
// unwinding, while all drawing accesses reborrow the same actual canvas.
struct CanvasRestore<'a, C: LooperCanvas> {
    canvas: &'a mut C,
    count: i32,
}
impl<C: LooperCanvas> Drop for CanvasRestore<'_, C> {
    fn drop(&mut self) {
        self.canvas.restoreToCount(self.count)
    }
}
