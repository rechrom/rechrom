//! Embedding boundary for the translated CPU Canvas.
//!
//! This module contains no replacement raster algorithms and receives no native
//! source-line credit. Pixels remain exclusively borrowed until Canvas teardown.
//! Browser replay migration can use this boundary without importing the legacy
//! skia crate's Canvas, Pixmap, PixelStorage or compatibility algorithms.

use core::{marker::PhantomData, pin::Pin, ptr::NonNull};

use crate::include::core::{
    SkAlphaType::SkAlphaType, SkClipOp::SkClipOp, SkColorType::SkColorType,
    SkSamplingOptions::SkSamplingOptions, SkSurfaceProps::SkSurfaceProps,
};
use crate::src::core::{
    SkBlendMode::SkBlendMode,
    SkCanvas::{SaveLayerRec, SkCanvas, SrcRectConstraint},
    SkColor::{SkColor, SkColor4f},
    SkColorSpace::SkColorSpace,
    SkFont::SkFont,
    SkImageInfo::SkImageInfo,
    SkM44::SkM44,
    SkMatrix::SkMatrix,
    SkPaint::SkPaint,
    SkPath::SkPath,
    SkPoint::SkPoint,
    SkRRect::SkRRect,
    SkRect::SkRect,
    SkTextBlob::SkTextBlob,
};
use crate::src::image::SkImage::SkImage;
use crate::RasterError;
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

/// Porter-Duff operations with a translated Skia N32 fast path. `Src` and
/// `SrcOver` use the original sprite row blitter; `DstIn` uses Skia's original
/// RasterPipeline stages because the row blitter does not expose that mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum N32BlitMode {
    Src,
    SrcOver,
    DstIn,
}

/// Convert one tightly packed straight-alpha RGBA8888 image into Skia's
/// premultiplied native N32 representation using the selected original SkOpts
/// kernel. This is the PaintImage upload boundary; it owns the returned bytes.
pub fn premultiply_n32_rgba8888(width: u32, height: u32, pixels: &[u8]) -> Option<Vec<u8>> {
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
            if crate::src::core::SkColorPriv::SK_PMCOLOR_IS_BGRA {
                crate::src::opts::SkSwizzler_opts::RGBA_to_bgrA(
                    dst.as_mut_ptr().cast(),
                    src.as_ptr().cast(),
                    width as i32,
                );
            } else {
                crate::src::opts::SkSwizzler_opts::RGBA_to_rgbA(
                    dst.as_mut_ptr().cast(),
                    src.as_ptr().cast(),
                    width as i32,
                );
            }
        }
    }
    Some(output)
}

/// Convert tightly packed premultiplied RGBA8888 pixels into Skia's native
/// N32 byte order without applying alpha a second time. WebP's libwebp-backed
/// decoder publishes this representation directly.
pub fn native_n32_from_premultiplied_rgba8888(
    width: u32,
    height: u32,
    pixels: &[u8],
) -> Option<Vec<u8>> {
    if width == 0 || height == 0 || width > (i32::MAX as u32) / 4 {
        return None;
    }
    let row_bytes = (width as usize).checked_mul(4)?;
    let length = row_bytes.checked_mul(height as usize)?;
    if length > isize::MAX as usize || pixels.len() != length {
        return None;
    }
    if !crate::src::core::SkColorPriv::SK_PMCOLOR_IS_BGRA {
        return Some(pixels.to_vec());
    }
    let mut output = vec![0; length];
    for (dst, src) in output
        .chunks_exact_mut(row_bytes)
        .zip(pixels.chunks_exact(row_bytes))
    {
        unsafe {
            crate::src::opts::SkSwizzler_opts::RGBA_to_BGRA(
                dst.as_mut_ptr().cast(),
                src.as_ptr().cast(),
                width as i32,
            );
        }
    }
    Some(output)
}

#[cfg(test)]
mod image_upload_tests {
    #[test]
    fn premultiplied_upload_only_changes_native_channel_order() {
        let input = [10, 20, 30, 40, 50, 60, 70, 80];
        let actual = super::native_n32_from_premultiplied_rgba8888(2, 1, &input)
            .expect("valid premultiplied bitmap");
        let expected = if crate::src::core::SkColorPriv::SK_PMCOLOR_IS_BGRA {
            vec![30, 20, 10, 40, 70, 60, 50, 80]
        } else {
            input.to_vec()
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn premultiplied_upload_rejects_invalid_storage() {
        assert!(super::native_n32_from_premultiplied_rgba8888(0, 1, &[]).is_none());
        assert!(super::native_n32_from_premultiplied_rgba8888(1, 1, &[0; 3]).is_none());
    }
}

/// A pinned, concrete CPU Canvas over initialized, host-owned 32-bit pixels.
///
/// The Canvas/device may retain the raw address but never owns the host buffer.
/// No pixels, Canvas pointer, snapshot or device reference escapes this owner.
/// Dropping an unfinished target cancels layers; call `finish` and propagate its
/// result before publishing a rendered frame.
pub struct RasterTarget<'pixels> {
    canvas: Pin<Box<SkCanvas<'pixels>>>,
    pixels: NonNull<u8>,
    pixel_len: usize,
    width: i32,
    height: i32,
    row_bytes: usize,
    format: PixelFormat,
    _pixels: PhantomData<&'pixels mut [u8]>,
}

impl<'pixels> RasterTarget<'pixels> {
    /// Preserve the tile's position in picture device space for analytic AA.
    pub fn set_raster_phase_origin(&mut self, x: i32, y: i32) {
        self.canvas.as_mut().setRasterPhaseOrigin(x, y);
    }

    /// Source: SkCanvas::imageInfo / SkCanvas::getBaseProps. Picture shaders
    /// choose their cached raster tile from the actual destination color and
    /// surface identities, not from an embedding-wide default.
    pub fn image_info(&self) -> SkImageInfo {
        self.canvas.as_ref().get_ref().imageInfo()
    }

    pub fn surface_props(&self) -> SkSurfaceProps {
        self.canvas.as_ref().get_ref().getBaseProps()
    }

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
        Self::new_with_color_space(pixels, width, height, row_bytes, format, None, props)
    }

    /// Wraps pixels with the exact color identity selected by the embedding.
    /// This is the Rust form of `SkSurfaces::WrapPixels(SkImageInfo, ...)` used
    /// by Chromium's `RasterBufferProvider::PlaybackToMemory`.
    pub fn new_with_color_space(
        pixels: &'pixels mut [u8],
        width: i32,
        height: i32,
        row_bytes: usize,
        format: PixelFormat,
        color_space: Option<Arc<SkColorSpace>>,
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
        let info =
            SkImageInfo::MakeWithColorSpace(width, height, color_type, alpha_type, color_space);
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
        let pixel_address = NonNull::new(pixels.as_mut_ptr()).ok_or(TargetError::BufferTooSmall)?;
        let canvas = unsafe {
            SkCanvas::MakeRasterDirect(&info, pixels.as_mut_ptr().cast(), row_bytes, props)
        }
        .ok_or(TargetError::CanvasCreationFailed)?;
        Ok(Self {
            canvas,
            pixels: pixel_address,
            pixel_len: available,
            width,
            height,
            row_bytes,
            format,
            _pixels: PhantomData,
        })
    }

    /// Composite a clipped, pixel-aligned N32 sprite using Skia's original
    /// `SkBlitRow` or `SkRasterPipeline` implementation.
    ///
    /// This is the same terminal row operation used by Skia's sprite blitter;
    /// it avoids rebuilding an image shader for the common 1:1 tile case. The
    /// caller supplies device-space coordinates after applying Canvas' matrix
    /// and clip. Returning `false` means the validated N32 fast path does not
    /// apply and Canvas must perform the draw.
    pub fn blit_n32(
        &mut self,
        source: &[u8],
        source_width: i32,
        source_height: i32,
        source_row_bytes: usize,
        source_x: i32,
        source_y: i32,
        destination_x: i32,
        destination_y: i32,
        width: i32,
        height: i32,
        mode: N32BlitMode,
        alpha: u8,
        source_opaque: bool,
    ) -> bool {
        let mut trace = browser_tracing::span("renderer", "Renderer.Skia.N32Blit");
        trace.set("mode", mode as u8 as f64);
        trace.set("pixels", width.max(0) as f64 * height.max(0) as f64);
        if !matches!(self.format, PixelFormat::Bgra8888 | PixelFormat::Bgrx8888)
            || source_width <= 0
            || source_height <= 0
            || width <= 0
            || height <= 0
            || source_x < 0
            || source_y < 0
            || destination_x < 0
            || destination_y < 0
            || source_x.saturating_add(width) > source_width
            || source_y.saturating_add(height) > source_height
            || destination_x.saturating_add(width) > self.width
            || destination_y.saturating_add(height) > self.height
            || source_row_bytes < source_width as usize * 4
            || self.row_bytes < self.width as usize * 4
            || source_row_bytes % 4 != 0
            || self.row_bytes % 4 != 0
            || source_row_bytes
                .checked_mul(source_height as usize)
                .is_none_or(|required| required > source.len())
            || self
                .row_bytes
                .checked_mul(self.height as usize)
                .is_none_or(|required| required > self.pixel_len)
            || (mode == N32BlitMode::Src && alpha != 255)
            || (mode == N32BlitMode::DstIn && self.format != PixelFormat::Bgra8888)
        {
            return false;
        }

        let source_stride = source_row_bytes / 4;
        let destination_stride = self.row_bytes / 4;
        let source_start = source_y as usize * source_stride + source_x as usize;
        let destination_start =
            destination_y as usize * destination_stride + destination_x as usize;

        if mode == N32BlitMode::DstIn {
            if alpha == 255 && source_opaque {
                return true;
            }
            if alpha == 255 {
                // Source: src/opts/SkRasterPipeline_opts.h:6239-6270.
                // The lowp DstIn pipeline is four independent
                // div255_accurate(destination * source_alpha) channels. Keep
                // that exact arithmetic while specializing the common N32
                // mask row: fully opaque mask pixels are a no-op, and fully
                // transparent pixels clear without entering the general
                // RasterPipeline stage chain. CSS mask gradients commonly
                // contain a wide opaque plateau, so this also avoids writing
                // most of the retained offscreen pass.
                #[inline(always)]
                fn div255_accurate_pair(value: u32) -> u32 {
                    const PAIR_MASK: u32 = 0x00ff_00ff;
                    let biased = value.wrapping_add(0x0080_0080);
                    (biased.wrapping_add((biased >> 8) & PAIR_MASK) >> 8) & PAIR_MASK
                }

                unsafe {
                    let mut src_row = source.as_ptr().cast::<u32>().add(source_start);
                    let mut dst_row = self.pixels.as_ptr().cast::<u32>().add(destination_start);
                    for _ in 0..height {
                        let mut src = src_row;
                        let mut dst = dst_row;
                        for _ in 0..width {
                            let source_alpha = src.read() >> 24;
                            if source_alpha == 0 {
                                dst.write(0);
                            } else if source_alpha != 255 {
                                let pixel = dst.read();
                                let rb = div255_accurate_pair(
                                    (pixel & 0x00ff_00ff).wrapping_mul(source_alpha),
                                );
                                let ag = div255_accurate_pair(
                                    ((pixel >> 8) & 0x00ff_00ff).wrapping_mul(source_alpha),
                                );
                                dst.write(rb | (ag << 8));
                            }
                            src = src.add(1);
                            dst = dst.add(1);
                        }
                        src_row = src_row.add(source_stride);
                        dst_row = dst_row.add(destination_stride);
                    }
                }
                return true;
            }
            let Ok(source_stride) = i32::try_from(source_stride) else {
                return false;
            };
            let Ok(destination_stride) = i32::try_from(destination_stride) else {
                return false;
            };
            use crate::src::core::SkRasterPipelineOpContexts::MemoryCtx;
            let mut source_context = MemoryCtx {
                pixels: unsafe {
                    source
                        .as_ptr()
                        .cast::<u32>()
                        .add(source_start)
                        .cast_mut()
                        .cast()
                },
                stride: source_stride,
            };
            let mut destination_context = MemoryCtx {
                pixels: unsafe {
                    self.pixels
                        .as_ptr()
                        .cast::<u32>()
                        .add(destination_start)
                        .cast()
                },
                stride: destination_stride,
            };
            let mut paint_alpha = alpha as f32 / 255.;
            let mut storage = core::pin::pin!(
                crate::src::core::SkArenaAlloc::SkSTArenaAlloc::<256>::new(0)
            );
            // This is the same source -> paint alpha -> destination -> blend ->
            // store program assembled by SkRasterPipelineBlitter. Validation
            // above proves both disjoint MemoryCtx spans remain live through
            // the synchronous run.
            unsafe {
                let mut pipeline = crate::src::core::SkRasterPipeline::SkRasterPipeline::new(
                    storage.as_mut().arena(),
                );
                // DstIn reads only source alpha and multiplies every retained
                // destination channel by that same value. Viewing native BGRA
                // bytes as RGBA is therefore identical for this operation and
                // lets Skia omit three otherwise redundant swap_rb stages.
                let color_type = SkColorType::kRGBA_8888_SkColorType;
                pipeline.appendLoad(color_type, &mut source_context);
                if alpha != 255 {
                    pipeline.append(
                        crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp::scale_1_float,
                        (&mut paint_alpha as *mut f32).cast(),
                    );
                }
                pipeline.appendLoadDst(color_type, &mut destination_context);
                crate::src::core::SkBlendMode::SkBlendMode_AppendStages(
                    SkBlendMode::kDstIn,
                    &mut pipeline,
                );
                pipeline.appendStore(color_type, &mut destination_context);
                return pipeline.run(0, 0, width as usize, height as usize).is_ok();
            }
        }

        let mut flags = 0;
        if mode == N32BlitMode::SrcOver && !source_opaque {
            flags |= crate::src::core::SkBlitRow::SkBlitRow::kSrcPixelAlpha_Flag32;
        }
        if alpha != 255 {
            flags |= crate::src::core::SkBlitRow::SkBlitRow::kGlobalAlpha_Flag32;
        }
        let procedure = crate::src::core::SkBlitRow::SkBlitRow::Factory32_raw(flags);
        // Validation above proves every addressed row lies inside its live
        // allocation. Raster resources and render-pass targets are disjoint
        // owners, satisfying SkBlitRow's source/destination alias contract.
        unsafe {
            let mut src = source.as_ptr().cast::<u32>().add(source_start);
            let mut dst = self.pixels.as_ptr().cast::<u32>().add(destination_start);
            for _ in 0..height {
                procedure(dst, src, width, alpha as u32);
                src = src.add(source_stride);
                dst = dst.add(destination_stride);
            }
        }
        true
    }

    /// Composite a strictly clamped N32 subset through Skia's translated
    /// bilinear image-shader and `DstIn` RasterPipeline stages.
    ///
    /// Chromium's software renderer frequently positions an otherwise 1:1
    /// mask tile at a fractional device origin. That is not a sprite (each
    /// destination pixel samples two source rows and columns), but it also
    /// does not require constructing a general SkCanvas image shader. The
    /// caller supplies the device-to-subset translation and the non-AA clip
    /// spans. Returning `false` leaves the general Canvas draw as the
    /// authoritative fallback.
    pub fn blit_n32_linear_dst_in_spans(
        &mut self,
        source: &[u8],
        source_width: i32,
        source_height: i32,
        source_row_bytes: usize,
        subset_x: i32,
        subset_y: i32,
        subset_width: i32,
        subset_height: i32,
        source_translate_x: f32,
        source_translate_y: f32,
        spans: &[(i32, i32, i32)],
        alpha: u8,
    ) -> bool {
        let mut trace = browser_tracing::span("renderer", "Renderer.Skia.N32Blit");
        trace.set("mode", 3.);
        trace.set(
            "pixels",
            spans
                .iter()
                .map(|&(_, _, width)| f64::from(width.max(0)))
                .sum(),
        );
        if self.format != PixelFormat::Bgra8888
            || source_width <= 0
            || source_height <= 0
            || subset_width <= 0
            || subset_height <= 0
            || spans.is_empty()
            || subset_x < 0
            || subset_y < 0
            || subset_x.saturating_add(subset_width) > source_width
            || subset_y.saturating_add(subset_height) > source_height
            || !source_translate_x.is_finite()
            || !source_translate_y.is_finite()
            || spans.iter().any(|&(x, y, width)| {
                x < 0
                    || y < 0
                    || width <= 0
                    || x.saturating_add(width) > self.width
                    || y >= self.height
            })
            || source_row_bytes < source_width as usize * 4
            || self.row_bytes < self.width as usize * 4
            || source_row_bytes % 4 != 0
            || self.row_bytes % 4 != 0
            || source_row_bytes
                .checked_mul(source_height as usize)
                .is_none_or(|required| required > source.len())
            || self
                .row_bytes
                .checked_mul(self.height as usize)
                .is_none_or(|required| required > self.pixel_len)
        {
            return false;
        }

        let source_stride = source_row_bytes / 4;
        let destination_stride = self.row_bytes / 4;
        let source_start = subset_y as usize * source_stride + subset_x as usize;
        let source_alpha = unsafe { source.as_ptr().add(source_start * 4 + 3) };
        let destination = self.pixels.as_ptr().cast::<u32>();
        let source_limit_x = subset_width - 1;
        let source_limit_y = subset_height - 1;
        let paint_scale = alpha as f32 / (255. * 255.);
        let source_floor_x = source_translate_x.floor() as i32;
        let source_floor_y = source_translate_y.floor() as i32;
        let fx = (source_translate_x + 1.).rem_euclid(1.);
        let fy = (source_translate_y + 1.).rem_euclid(1.);
        let wx0 = 1. - fx;
        let wy0 = 1. - fy;

        // This is the constant-translation specialization of Skia's original
        // bilerp_clamp_8888 + DstIn program (SkRasterPipeline_opts.h). Unlike
        // the general gather stage it reads only the alpha channel required by
        // DstIn. Pixel-center coordinates, clamp sampling, bilerp weights and
        // round-to-N32 storage remain the same as that pipeline.
        unsafe {
            for &(x, y, width) in spans {
                let source_y = y.saturating_add(source_floor_y);
                let y0 = source_y.clamp(0, source_limit_y) as usize;
                let y1 = source_y.saturating_add(1).clamp(0, source_limit_y) as usize;
                let row0 = source_alpha.add(y0 * source_row_bytes);
                let row1 = source_alpha.add(y1 * source_row_bytes);
                let mut source_x = x.saturating_add(source_floor_x);
                let mut destination_pixel =
                    destination.add(y as usize * destination_stride + x as usize);
                for _ in 0..width {
                    let x0 = source_x.clamp(0, source_limit_x) as usize;
                    let x1 = source_x.saturating_add(1).clamp(0, source_limit_x) as usize;
                    let a00 = row0.add(x0 * 4).read() as f32;
                    let a10 = row0.add(x1 * 4).read() as f32;
                    let a01 = row1.add(x0 * 4).read() as f32;
                    let a11 = row1.add(x1 * 4).read() as f32;
                    let sampled_alpha =
                        a00 * wx0 * wy0 + a10 * fx * wy0 + a01 * wx0 * fy + a11 * fx * fy;
                    let scale = sampled_alpha * paint_scale;
                    let pixel = destination_pixel.read();
                    let scale_channel = |shift: u32| {
                        ((((pixel >> shift) & 0xffu32) as f32 * scale + 0.5) as u32).min(0xff)
                    };
                    destination_pixel.write(
                        scale_channel(0)
                            | (scale_channel(8) << 8)
                            | (scale_channel(16) << 16)
                            | (scale_channel(24) << 24),
                    );
                    source_x = source_x.saturating_add(1);
                    destination_pixel = destination_pixel.add(1);
                }
            }
            true
        }
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

    /// Draw the immutable positioned-glyph representation produced by
    /// ShapeResultBloberizer.  Keeping the blob intact preserves Skia's
    /// horizontal-positioning path instead of expanding every glyph to an
    /// independent two-dimensional position at the embedding boundary.
    ///
    /// # Safety
    /// Blob font/typeface/scaler backing and paint resources must remain alive
    /// for the synchronous draw and must not alias the destination pixels.
    pub unsafe fn draw_text_blob(
        &mut self,
        blob: &crate::include::core::SkRefCnt::sk_sp<SkTextBlob>,
        origin: SkPoint,
        paint: &SkPaint,
    ) -> Result<(), RasterError> {
        unsafe {
            SkCanvas::drawTextBlob_shared_raw(
                self.canvas_address(),
                blob,
                origin.fX,
                origin.fY,
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
