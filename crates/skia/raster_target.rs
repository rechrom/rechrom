//! Embedding boundary for the translated CPU Canvas.
//!
//! This module contains no replacement raster algorithms and receives no native
//! source-line credit. Pixels remain exclusively borrowed until Canvas teardown.
//! Browser replay migration can use this boundary without importing the legacy
//! skia crate's Canvas, Pixmap, PixelStorage or compatibility algorithms.

use core::{marker::PhantomData, pin::Pin, ptr::NonNull};

use crate::RasterError;
use crate::include::core::{
    SkAlphaType::SkAlphaType, SkClipOp::SkClipOp, SkColorType::SkColorType,
    SkSamplingOptions::SkSamplingOptions, SkSurfaceProps::SkSurfaceProps,
};
use crate::src::core::{
    SkBlendMode::SkBlendMode,
    SkCanvas::{SaveLayerRec, SkCanvas, SrcRectConstraint},
    SkColor::{SkColor, SkColor4f},
    SkFont::SkFont,
    SkImageInfo::SkImageInfo,
    SkM44::SkM44,
    SkMatrix::SkMatrix,
    SkPaint::SkPaint,
    SkPath::SkPath,
    SkPoint::SkPoint,
    SkRRect::SkRRect,
    SkRect::SkRect,
};
use crate::src::image::SkImage::SkImage;
use std::sync::Arc;

/// Byte order in the host mapping, independent of the native N32 configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PixelFormat {
    Rgba8888,
    Bgra8888,
    Bgrx8888,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetError {
    InvalidLayout,
    BufferTooSmall,
    CanvasCreationFailed,
}

/// Convert one tightly packed straight-alpha RGBA8888 image into Skia's
/// premultiplied RGBA8888 representation using the selected original SkOpts
/// kernel. This is the PaintImage upload boundary; it owns the returned bytes.
pub fn premultiply_rgba8888(width: u32, height: u32, pixels: &[u8]) -> Option<Vec<u8>> {
    if width == 0 || height == 0 || width > (i32::MAX as u32) / 4 {
        return None;
    }
    let row_bytes = (width as usize).checked_mul(4)?;
    let length = row_bytes.checked_mul(height as usize)?;
    if length > isize::MAX as usize || pixels.len() != length {
        return None;
    }
    let mut output = vec![0; length];
    for (dst, src) in output
        .chunks_exact_mut(row_bytes)
        .zip(pixels.chunks_exact(row_bytes))
    {
        unsafe {
            crate::src::opts::SkSwizzler_opts::RGBA_to_rgbA(
                dst.as_mut_ptr().cast(),
                src.as_ptr().cast(),
                width as i32,
            );
        }
    }
    Some(output)
}

/// A pinned, concrete CPU Canvas over initialized, host-owned 32-bit pixels.
///
/// The Canvas/device may retain the raw address but never owns the host buffer.
/// No pixels, Canvas pointer, snapshot or device reference escapes this owner.
/// Dropping an unfinished target cancels layers; call `finish` and propagate its
/// result before publishing a rendered frame.
pub struct RasterTarget<'pixels> {
    canvas: Pin<Box<SkCanvas<'pixels>>>,
    _pixels: PhantomData<&'pixels mut [u8]>,
}

impl<'pixels> RasterTarget<'pixels> {
    /// Borrow the entire visible target, including any padding between rows.
    /// `row_bytes` is the physical stride; `width` remains the visible width.
    pub fn new(
        pixels: &'pixels mut [u8],
        width: i32,
        height: i32,
        row_bytes: usize,
        format: PixelFormat,
        props: Option<&SkSurfaceProps>,
    ) -> Result<Self, TargetError> {
        let (color_type, alpha_type) = match format {
            PixelFormat::Rgba8888 => (
                SkColorType::kRGBA_8888_SkColorType,
                SkAlphaType::kPremul_SkAlphaType,
            ),
            PixelFormat::Bgra8888 => (
                SkColorType::kBGRA_8888_SkColorType,
                SkAlphaType::kPremul_SkAlphaType,
            ),
            PixelFormat::Bgrx8888 => (
                SkColorType::kBGRA_8888_SkColorType,
                SkAlphaType::kOpaque_SkAlphaType,
            ),
        };
        let info = SkImageInfo::MakeWithColorSpace(width, height, color_type, alpha_type, None);
        if !crate::src::image::SkSurface_Raster::SkSurfaceValidateRasterInfo(&info, row_bytes) {
            return Err(TargetError::InvalidLayout);
        }
        let required = info.computeByteSize(row_bytes);
        let available = pixels.len();
        if required == usize::MAX || required > available {
            return Err(TargetError::BufferTooSmall);
        }
        // The u32 slice establishes initialization, alignment, allocation length
        // and exclusivity. Its borrow is retained by the returned target. The
        // original MakeRasterDirect installs nonowning pixels; no release
        // callback or replacement pixel owner is introduced here.
        let canvas = unsafe {
            SkCanvas::MakeRasterDirect(&info, pixels.as_mut_ptr().cast(), row_bytes, props)
        }
        .ok_or(TargetError::CanvasCreationFailed)?;
        Ok(Self {
            canvas,
            _pixels: PhantomData,
        })
    }

    // End the temporary Pin borrow before calling any operation that can restore
    // a layer or invoke a device callback. The Box continues to pin the same
    // complete Canvas; no whole-Canvas reference crosses the raw call.
    fn canvas_address(&mut self) -> NonNull<SkCanvas<'pixels>> {
        unsafe { NonNull::from(self.canvas.as_mut().get_unchecked_mut()) }
    }

    pub fn clear(&mut self, color: SkColor) -> Result<(), RasterError> {
        unsafe { SkCanvas::clear_raw(self.canvas_address(), color) }
    }

    pub fn save(&mut self) -> i32 {
        unsafe { self.canvas.as_mut().save() }
    }

    pub fn save_count(&self) -> i32 {
        self.canvas.as_ref().get_ref().getSaveCount()
    }

    pub fn restore(&mut self) -> Result<(), RasterError> {
        unsafe { SkCanvas::restore_raw(self.canvas_address()) }
    }

    pub fn restore_to_count(&mut self, count: i32) -> Result<(), RasterError> {
        unsafe { SkCanvas::restoreToCount_raw(self.canvas_address(), count) }
    }

    pub fn set_matrix(&mut self, matrix: &SkMatrix) {
        unsafe { self.canvas.as_mut().setMatrix(matrix) }
    }

    pub fn set_matrix44(&mut self, matrix: &SkM44) {
        unsafe { self.canvas.as_mut().setMatrix44(matrix) }
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        unsafe { self.canvas.as_mut().translate(x, y) }
    }

    pub fn scale(&mut self, x: f32, y: f32) {
        unsafe { self.canvas.as_mut().scale(x, y) }
    }

    pub fn concat(&mut self, matrix: &SkMatrix) {
        unsafe { self.canvas.as_mut().concat(matrix) }
    }

    pub fn concat44(&mut self, matrix: &SkM44) {
        unsafe { self.canvas.as_mut().concat44(matrix) }
    }

    pub fn local_to_device(&self) -> SkM44 {
        self.canvas.as_ref().get_ref().getLocalToDevice()
    }

    pub fn local_clip_bounds(&self) -> SkRect {
        self.canvas.as_ref().get_ref().getLocalClipBounds()
    }

    /// Copy a device-space rectangle into caller-owned storage while preserving
    /// the target's exclusive Canvas/device lease. The destination layout is
    /// validated here so the embedding never needs the raw Canvas pointer.
    pub fn read_pixels(
        &mut self,
        dst_info: &SkImageInfo,
        pixels: &mut [u8],
        row_bytes: usize,
        x: i32,
        y: i32,
    ) -> Result<bool, RasterError> {
        if !dst_info.validRowBytes(row_bytes) {
            return Ok(false);
        }
        let required = dst_info.computeByteSize(row_bytes);
        if required == usize::MAX || required > pixels.len() {
            return Ok(false);
        }
        unsafe {
            self.canvas
                .as_mut()
                .readPixels_raw(dst_info, pixels.as_mut_ptr(), row_bytes, x, y)
        }
    }

    /// Copy initialized source pixels through SkCanvas::writePixels. Canvas
    /// intentionally ignores its matrix for this device-space operation.
    ///
    /// # Safety
    /// The source slice must not overlap the exclusively borrowed destination.
    pub unsafe fn write_pixels(
        &mut self,
        src_info: &SkImageInfo,
        pixels: &[u8],
        row_bytes: usize,
        x: i32,
        y: i32,
    ) -> Result<bool, RasterError> {
        if !src_info.validRowBytes(row_bytes) {
            return Ok(false);
        }
        let required = src_info.computeByteSize(row_bytes);
        if required == usize::MAX || required > pixels.len() {
            return Ok(false);
        }
        unsafe {
            SkCanvas::writePixels_raw(
                self.canvas_address(),
                src_info,
                pixels.as_ptr(),
                row_bytes,
                x,
                y,
            )
        }
    }

    pub fn clip_rect(&mut self, rect: &SkRect, op: SkClipOp, antialias: bool) {
        unsafe { self.canvas.as_mut().clipRect(rect, op, antialias) }
    }

    pub fn clip_rrect(&mut self, rect: &SkRRect, op: SkClipOp, antialias: bool) {
        unsafe { self.canvas.as_mut().clipRRect(rect, op, antialias) }
    }

    pub fn clip_path(&mut self, path: &SkPath, op: SkClipOp, antialias: bool) {
        unsafe { self.canvas.as_mut().clipPath(path, op, antialias) }
    }

    pub fn save_layer(
        &mut self,
        bounds: Option<&SkRect>,
        paint: Option<&SkPaint>,
    ) -> Result<i32, RasterError> {
        unsafe {
            self.canvas
                .as_mut()
                .saveLayer(&SaveLayerRec::new(bounds, paint, 0))
        }
    }

    pub fn save_layer_rec(&mut self, record: &SaveLayerRec<'_>) -> Result<i32, RasterError> {
        unsafe { self.canvas.as_mut().saveLayer(record) }
    }

    pub fn draw_color(&mut self, color: &SkColor4f, mode: SkBlendMode) -> Result<(), RasterError> {
        unsafe { SkCanvas::drawColor_raw(self.canvas_address(), color, mode) }
    }

    /// # Safety
    /// Paint resources obey the same synchronous lifetime and alias contract as
    /// the other draw methods on this target.
    pub unsafe fn draw_paint(&mut self, paint: &SkPaint) -> Result<(), RasterError> {
        unsafe { SkCanvas::drawPaint_raw(self.canvas_address(), paint) }
    }

    /// Draw through the original Canvas -> Device -> Draw -> Blitter chain.
    ///
    /// # Safety
    /// Paint's shader, filter and image resources must meet the translated
    /// Canvas draw contract, including their backing lifetimes and alias rules.
    /// The exclusive destination pixel lease is supplied by this target.
    pub unsafe fn draw_rect(&mut self, rect: &SkRect, paint: &SkPaint) -> Result<(), RasterError> {
        unsafe { SkCanvas::drawRect_raw(self.canvas_address(), rect, paint) }
    }

    /// # Safety
    /// Same resource and alias requirements as `draw_rect`; path and paint must
    /// remain valid throughout the synchronous draw and its callbacks.
    pub unsafe fn draw_path(&mut self, path: &SkPath, paint: &SkPaint) -> Result<(), RasterError> {
        unsafe { SkCanvas::drawPath_raw(self.canvas_address(), path, paint) }
    }

    /// # Safety
    /// Same resource and alias requirements as `draw_rect`.
    pub unsafe fn draw_rrect(
        &mut self,
        rect: &SkRRect,
        paint: &SkPaint,
    ) -> Result<(), RasterError> {
        unsafe { SkCanvas::drawRRect_raw(self.canvas_address(), rect, paint) }
    }

    /// # Safety
    /// Same resource and alias requirements as `draw_rect`.
    pub unsafe fn draw_drrect(
        &mut self,
        outer: &SkRRect,
        inner: &SkRRect,
        paint: &SkPaint,
    ) -> Result<(), RasterError> {
        unsafe { SkCanvas::drawDRRect_raw(self.canvas_address(), outer, inner, paint) }
    }

    /// # Safety
    /// Same resource and alias requirements as `draw_rect`.
    pub unsafe fn draw_oval(&mut self, oval: &SkRect, paint: &SkPaint) -> Result<(), RasterError> {
        unsafe { SkCanvas::drawOval_raw(self.canvas_address(), oval, paint) }
    }

    /// # Safety
    /// Same resource and alias requirements as `draw_rect`.
    pub unsafe fn draw_line(
        &mut self,
        start: SkPoint,
        end: SkPoint,
        paint: &SkPaint,
    ) -> Result<(), RasterError> {
        unsafe {
            SkCanvas::drawLine_raw(
                self.canvas_address(),
                start.fX,
                start.fY,
                end.fX,
                end.fY,
                paint,
            )
        }
    }

    /// Draw already decoded image resources through the original image shader
    /// and sampling pipeline; this boundary does not add a decoder or resampler.
    ///
    /// # Safety
    /// Image pixels must remain initialized/readable and disjoint from this
    /// target throughout drawing. Paint resources obey `draw_rect`'s contract.
    pub unsafe fn draw_image_rect(
        &mut self,
        image: &Arc<dyn SkImage>,
        source: Option<&SkRect>,
        destination: &SkRect,
        sampling: &SkSamplingOptions,
        paint: Option<&SkPaint>,
        constraint: SrcRectConstraint,
    ) -> Result<(), RasterError> {
        unsafe {
            SkCanvas::drawImageRect_raw(
                self.canvas_address(),
                image,
                source,
                destination,
                sampling,
                paint,
                constraint,
            )
        }
    }

    /// Replay glyph IDs and positions supplied by the browser's shaping owner.
    /// No text reshaping or alternate glyph rasterizer runs in this boundary.
    ///
    /// # Safety
    /// Positions must contain one entry per glyph. Font/typeface/scaler backing
    /// and paint resources obey the native drawGlyphs resource contract and must
    /// not alias the exclusively borrowed destination pixels.
    pub unsafe fn draw_glyphs(
        &mut self,
        glyphs: &[u16],
        positions: &[SkPoint],
        origin: SkPoint,
        font: &SkFont,
        paint: &SkPaint,
    ) -> Result<(), RasterError> {
        unsafe {
            SkCanvas::drawGlyphs_raw(
                self.canvas_address(),
                glyphs,
                positions,
                origin,
                font,
                paint,
            )
        }
    }

    /// Complete deferred restores before releasing the host buffer borrow.
    /// A failure is returned to the embedding; no legacy backend is retried.
    pub fn finish(mut self) -> Result<(), RasterError> {
        unsafe { SkCanvas::try_finish_raw(self.canvas_address()) }
    }
}
