//! Image::ResizeAndOrientImage, image.cc:125-186, used by DragImage::Create
//! (core/page/drag_image.cc:110-142), fixed Chromium revision
//! 6c1d401fcca5e1b0030563a90c2f2bba168e0c15. Input pixels retain their original
//! resource owner; only the source's actual resize/orientation/alpha operation
//! allocates an output. The native drag packet uses straight RGBA8.
use super::*;

pub fn ResizeAndOrientDragImage(
    image: &image_resource::PaintImage,
    respect_orientation: bool,
    scale: (f32, f32),
    opacity: f32,
    nearest: bool,
) -> io::Result<Option<image_resource::ImmutableImageBitmap>> {
    let pixels = image
        .BitmapPixels()
        .ok_or_else(|| io::Error::other("drag image requires a decoded bitmap frame"))?;
    let orientation = if image.ForceOrientationIfNecessary(respect_orientation) {
        image.orientation
    } else {
        image_resource::ImageOrientation::kDefault
    };
    ResizeAndOrientDragPixels(
        image.width,
        image.height,
        pixels,
        image.alpha_type,
        orientation,
        scale,
        opacity,
        nearest,
        true,
    )
}

pub fn ResizeDragImageBitmap(
    bitmap: &image_resource::ImmutableImageBitmap,
    scale: (f32, f32),
    opacity: f32,
    nearest: bool,
) -> io::Result<Option<image_resource::ImmutableImageBitmap>> {
    ResizeAndOrientDragPixels(
        bitmap.width,
        bitmap.height,
        &bitmap.rgba8,
        image_resource::PaintImageAlphaType::Unpremultiplied,
        image_resource::ImageOrientation::kDefault,
        scale,
        opacity,
        nearest,
        true,
    )
}

fn ResizeAndOrientDragPixels(
    width: u32,
    height: u32,
    pixels: &Arc<Vec<u8>>,
    alpha: image_resource::PaintImageAlphaType,
    orientation: image_resource::ImageOrientation,
    scale: (f32, f32),
    opacity: f32,
    nearest: bool,
    straight: bool,
) -> io::Result<Option<image_resource::ImmutableImageBitmap>> {
    use image_resource::{ImageOrientation as O, PaintImageAlphaType as A};
    if !scale.0.is_finite()
        || !scale.1.is_finite()
        || !opacity.is_finite()
        || !(0.0..=1.0).contains(&opacity)
    {
        return Err(io::Error::other("invalid drag-image transform/opacity"));
    }
    let mut out_width = f64::from((width as f32 * scale.0).floor());
    let mut out_height = f64::from((height as f32 * scale.1).floor());
    if orientation.uses_width_as_height() {
        std::mem::swap(&mut out_width, &mut out_height);
    }
    if out_width <= 0. || out_height <= 0. {
        return Ok(None);
    }
    if out_width > i32::MAX as f64
        || out_height > i32::MAX as f64
        || width > i32::MAX as u32
        || height > i32::MAX as u32
    {
        return Err(io::Error::other("drag-image dimensions exceed Skia limits"));
    }
    if orientation == O::kDefault && scale == (1., 1.) && opacity == 1. {
        return Ok(Some(if !straight || alpha == A::Unpremultiplied {
            image_resource::ImmutableImageBitmap {
                width,
                height,
                rgba8: pixels.clone(),
            }
        } else {
            StraightDragBitmap(width, height, pixels)?
        }));
    }
    let (w, h) = (out_width as u32, out_height as u32);
    let row_bytes = (w as usize)
        .checked_mul(4)
        .ok_or_else(|| io::Error::other("drag-image row overflow"))?;
    let len = row_bytes
        .checked_mul(h as usize)
        .ok_or_else(|| io::Error::other("drag-image allocation overflow"))?;
    let mut output = Vec::new();
    output.try_reserve_exact(len).map_err(io::Error::other)?;
    output.resize(len, 0);
    let source_info = SkImageInfo::Make(
        width as i32,
        height as i32,
        SkColorType::kRGBA_8888_SkColorType,
        match alpha {
            A::Premultiplied => SkAlphaType::kPremul_SkAlphaType,
            A::Unpremultiplied => SkAlphaType::kUnpremul_SkAlphaType,
        },
    );
    let source_row = (width as usize)
        .checked_mul(4)
        .ok_or_else(|| io::Error::other("drag-image source row overflow"))?;
    if source_row
        .checked_mul(height as usize)
        .is_none_or(|len| len > pixels.len())
    {
        return Err(io::Error::other("drag-image source pixels are incomplete"));
    }
    let pixmap = SkPixmap::new(&source_info, pixels.as_ptr(), source_row);
    let image = unsafe {
        skia::src::image::SkImage_RasterFactories::RasterFromPixmap(
            &pixmap,
            None,
            std::ptr::null_mut(),
        )
    }
    .ok_or_else(|| io::Error::other("drag-image source image allocation failed"))?;
    {
        let mut target = RasterTarget::new(
            &mut output,
            w as i32,
            h as i32,
            row_bytes,
            PixelFormat::Rgba8888,
            None,
        )
        .map_err(|e| raster_error("drag-image target failed", e))?;
        // ImageOrientation::TransformFromDefault, image_orientation.cc:35-60.
        let (a, b, c, d, tx, ty) = match orientation {
            O::kDefault => (1., 0., 0., 1., 0., 0.),
            O::kFlipHorizontal => (-1., 0., 0., 1., w as f32, 0.),
            O::kRotate180 => (-1., 0., 0., -1., w as f32, h as f32),
            O::kFlipVertical => (1., 0., 0., -1., 0., h as f32),
            O::kTranspose => (0., 1., 1., 0., 0., 0.),
            O::kRotate90 => (0., 1., -1., 0., w as f32, 0.),
            O::kTransverse => (0., -1., -1., 0., w as f32, h as f32),
            O::kRotate270 => (0., -1., 1., 0., 0., h as f32),
        };
        target.concat44(&SkM44::ColMajor(&[
            a, b, 0., 0., c, d, 0., 0., 0., 0., 1., 0., tx, ty, 0., 1.,
        ]));
        target.scale(scale.0, scale.1);
        let mut paint = SkPaint::default();
        paint.setAlpha((opacity * 255.) as u32);
        let sampling = if nearest {
            SkSamplingOptions::default()
        } else {
            SkSamplingOptions::with_cubic(SkCubicResampler::CatmullRom())
        };
        let rect = SkRect::MakeWH(width as f32, height as f32);
        unsafe {
            target.draw_image_rect(
                &image,
                Some(&rect),
                &rect,
                &sampling,
                Some(&paint),
                SrcRectConstraint::kStrict_SrcRectConstraint,
            )
        }
        .map_err(|e| raster_error("drag-image draw failed", e))?;
        target
            .finish()
            .map_err(|e| raster_error("drag-image finish failed", e))?;
    }
    Ok(Some(if straight {
        StraightDragBitmap(w, h, &output)?
    } else {
        image_resource::ImmutableImageBitmap {
            width: w,
            height: h,
            rgba8: Arc::new(output),
        }
    }))
}

// Only the native packet boundary normalizes SkBitmap alpha representation.
fn StraightDragBitmap(
    w: u32,
    h: u32,
    output: &[u8],
) -> io::Result<image_resource::ImmutableImageBitmap> {
    let row_bytes = (w as usize)
        .checked_mul(4)
        .ok_or_else(|| io::Error::other("drag packet row overflow"))?;
    let len = row_bytes
        .checked_mul(h as usize)
        .ok_or_else(|| io::Error::other("drag packet size overflow"))?;
    if len > output.len() {
        return Err(io::Error::other("drag packet pixels are incomplete"));
    }
    let premul = SkImageInfo::Make(
        w as i32,
        h as i32,
        SkColorType::kRGBA_8888_SkColorType,
        SkAlphaType::kPremul_SkAlphaType,
    );
    let straight = SkImageInfo::Make(
        w as i32,
        h as i32,
        SkColorType::kRGBA_8888_SkColorType,
        SkAlphaType::kUnpremul_SkAlphaType,
    );
    let mut converted = Vec::new();
    converted.try_reserve_exact(len).map_err(io::Error::other)?;
    converted.resize(len, 0);
    if !unsafe {
        SkConvertPixels(
            &straight,
            converted.as_mut_ptr(),
            row_bytes,
            &premul,
            output.as_ptr(),
            row_bytes,
        )
    }
    .map_err(|e| raster_error("drag-image alpha normalization failed", e))?
    {
        return Err(io::Error::other(
            "drag-image alpha conversion rejected layout",
        ));
    }
    Ok(image_resource::ImmutableImageBitmap {
        width: w,
        height: h,
        rgba8: Arc::new(converted),
    })
}

/// DataTransfer::CreateDragImageForFrame cc:455-491. The record has already
/// passed the original PaintChunksToCcLayer conversion against its actual
/// border-box properties. Replay never scans/crops the ordinary Page artifact.
pub(super) fn RasterDragImage(
    backend: &TranslatedSkiaBackend,
    record: &compositor_core::engine::ImmutablePaintRecord,
    layout_size: (f64, f64),
    paint_offset: (f64, f64),
    scale: f32,
    opacity: f32,
    snapshots: &std::collections::BTreeMap<
        u32,
        canvas::child_paint_record::CanvasSnapshotReplacement,
    >,
) -> io::Result<Option<image_resource::ImmutableImageBitmap>> {
    if !scale.is_finite()
        || scale <= 0.
        || !layout_size.0.is_finite()
        || !layout_size.1.is_finite()
        || !paint_offset.0.is_finite()
        || !paint_offset.1.is_finite()
    {
        return Err(io::Error::other("invalid source drag-record geometry"));
    }
    let width = (layout_size.0 * f64::from(scale)).floor();
    let height = (layout_size.1 * f64::from(scale)).floor();
    if width <= 0. || height <= 0. {
        return Ok(None);
    }
    if width > i32::MAX as f64 || height > i32::MAX as f64 {
        return Err(io::Error::other("drag-record surface exceeds Skia limits"));
    }
    let (width, height) = (width as u32, height as u32);
    let row = (width as usize)
        .checked_mul(4)
        .ok_or_else(|| io::Error::other("drag-record row overflow"))?;
    let len = row
        .checked_mul(height as usize)
        .ok_or_else(|| io::Error::other("drag-record allocation overflow"))?;
    let mut pixels = Vec::new();
    pixels.try_reserve_exact(len).map_err(io::Error::other)?;
    pixels.resize(len, 0);
    {
        let mut target = RasterTarget::new(
            &mut pixels,
            width as i32,
            height as i32,
            row,
            PixelFormat::Rgba8888,
            None,
        )
        .map_err(|e| raster_error("drag-record surface failed", e))?;
        target.translate(
            (-paint_offset.0 * f64::from(scale)) as f32,
            (-paint_offset.1 * f64::from(scale)) as f32,
        );
        target.scale(scale, scale);
        if let Some(buffer) = record.buffer() {
            let mut receiver = TranslatedCanvas::new(&mut target, false, 1.)
                .with_typeface_cache(&backend.typeface_cache);
            receiver.canvas_snapshots = Some(snapshots);
            let mut provider = canvas_2d::CanvasImageProvider {
                frames: None,
                error: None,
                values: buffer.values().clone(),
            };
            receiver.Playback(
                buffer,
                &mut RasterPlaybackParams {
                    raster_inducing_scroll_offsets: None,
                    image_provider: Some(&mut provider),
                },
                true,
                None,
            );
            receiver.unwind_to_depth(0);
            receiver.finish_replay()?;
            if let Some(error) = provider.error {
                return Err(error);
            }
        }
        target
            .finish()
            .map_err(|e| raster_error("drag-record surface completion failed", e))?;
    }
    ResizeAndOrientDragPixels(
        width,
        height,
        &Arc::new(pixels),
        image_resource::PaintImageAlphaType::Premultiplied,
        image_resource::ImageOrientation::kDefault,
        (1., 1.),
        opacity,
        false,
        true,
    )
}

/// DragImage::Create(KURL,label,DSF), drag_image.cc:225-291. FontEngine has
/// already produced positioned, immutable glyph resources; this source Canvas
/// receiver performs the rounded background/text paint at the requested DSF.
pub(super) fn RasterLinkDragImage(
    cache: &TypefaceCache,
    size: (i32, i32),
    scale: f32,
    text: &[((f32, f32), font_engine::plain_text_painter::PlainTextPaint)],
) -> io::Result<Option<image_resource::ImmutableImageBitmap>> {
    let w = (size.0 as f32 * scale).floor() as u32;
    let h = (size.1 as f32 * scale).floor() as u32;
    if w == 0 || h == 0 {
        return Ok(None);
    }
    let len = (w as usize)
        .checked_mul(h as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| io::Error::other("link drag bitmap overflow"))?;
    if w > i32::MAX as u32 || h > i32::MAX as u32 {
        return Ok(None);
    }
    let mut pixels = Vec::new();
    pixels.try_reserve_exact(len).map_err(io::Error::other)?;
    pixels.resize(len, 0);
    {
        let mut target = RasterTarget::new(
            &mut pixels,
            w as i32,
            h as i32,
            w as usize * 4,
            PixelFormat::Rgba8888,
            None,
        )
        .map_err(|e| raster_error("link drag target", e))?;
        target.scale(scale, scale);
        let mut paint = SkPaint::default();
        paint.setColor(0xff8c8c8c);
        paint.setAntiAlias(true);
        let rect = SkRRect::MakeRectXY(&SkRect::MakeWH(size.0 as f32, size.1 as f32), 5., 5.);
        unsafe { target.draw_rrect(&rect, &paint) }
            .map_err(|e| raster_error("link drag background", e))?;
        let text_paint = SkPaint::default();
        for (origin, text) in text {
            if text.skip_drawing {
                continue;
            }
            for run in &text.runs {
                if run.glyphs.is_empty() {
                    continue;
                }
                let variations = run
                    .variations
                    .iter()
                    .map(|v| command::FontVariation {
                        tag: v.tag,
                        value: v.value,
                    })
                    .collect::<Vec<_>>();
                let face = command::FontFace {
                    family: &run.face.family,
                    resolved_family: &run.face.resolved_family,
                    weight: run.face.weight,
                    italic: run.face.italic,
                    bytes: &run.face.bytes,
                    bytes_owner: Some(Arc::new(run.face.bytes.clone())),
                    face_index: run.face.face_index,
                    variations: variations.clone(),
                };
                let item = DrawCommand {
                    font_size: run.size as f64,
                    font_variations: variations,
                    synthetic_bold: run.synthetic_bold,
                    synthetic_italic: run.synthetic_italic,
                    ..Default::default()
                };
                #[cfg(target_vendor = "apple")]
                let typeface = cache.GetOrCreate(&item, &face)?;
                #[cfg(not(target_vendor = "apple"))]
                let typeface = create_raster_typeface(&item, &face, &item.font_variations)?;
                let mut font = SkFont::with_size(Some(typeface), run.size);
                font.setEmbolden(run.synthetic_bold);
                font.setSkewX(if run.synthetic_italic { -0.25 } else { 0. });
                let count = i32::try_from(run.glyphs.len())
                    .map_err(|_| io::Error::other("link drag glyph count"))?;
                let mut builder = SkTextBlobBuilder::new();
                unsafe {
                    let b = builder.allocRunPos(&font, count, None);
                    for (i, glyph) in run.glyphs.iter().enumerate() {
                        *b.glyphs.add(i) = glyph.id;
                        *b.points().add(i) = SkPoint {
                            fX: glyph.x,
                            fY: glyph.y,
                        };
                    }
                }
                let blob = unsafe { builder.make() };
                unsafe {
                    target.draw_text_blob(
                        &blob,
                        SkPoint {
                            fX: origin.0,
                            fY: origin.1,
                        },
                        &text_paint,
                    )
                }
                .map_err(|e| raster_error("link drag text", e))?;
            }
        }
        target
            .finish()
            .map_err(|e| raster_error("link drag completion", e))?;
    }
    Ok(Some(StraightDragBitmap(w, h, &pixels)?))
}

/// DataTransfer::CreateDragImage -> DragImage::Create -> DragImage::Scale,
/// data_transfer.cc:500-515, drag_image.cc:301-312. Do not normalize to the
/// native packet's straight alpha before the source CPU convolution.
pub fn ScaleCachedDragImage(
    image: &image_resource::PaintImage,
    scale: f32,
    nearest: bool,
) -> io::Result<Option<image_resource::ImmutableImageBitmap>> {
    let Some(pixels) = image.BitmapPixels() else {
        return Err(io::Error::other(
            "cached bitmap drag requires its source pixels",
        ));
    };
    let orientation = image.orientation; // Create uses kRespectImageOrientation.
    let Some(bitmap) = ResizeAndOrientDragPixels(
        image.width,
        image.height,
        pixels,
        image.alpha_type,
        orientation,
        (1., 1.),
        1.,
        false,
        false,
    )?
    else {
        return Ok(None);
    };
    let alpha = if orientation == image_resource::ImageOrientation::kDefault {
        image.alpha_type
    } else {
        image_resource::PaintImageAlphaType::Premultiplied
    };
    ScaleDragPixels(&bitmap, alpha, scale, nearest)
}
pub(super) fn ScaleDragPixels(
    bitmap: &image_resource::ImmutableImageBitmap,
    alpha: image_resource::PaintImageAlphaType,
    scale: f32,
    nearest: bool,
) -> io::Result<Option<image_resource::ImmutableImageBitmap>> {
    if !scale.is_finite() {
        return Err(io::Error::other("invalid DragImage::Scale"));
    }
    let width = (bitmap.width as f32 * scale) as i32;
    let height = (bitmap.height as f32 * scale) as i32;
    if width <= 0 || height <= 0 {
        return Ok(None);
    }
    let stride = (bitmap.width as usize)
        .checked_mul(4)
        .ok_or_else(|| io::Error::other("cached drag stride overflow"))?;
    let Some(pixels) = super::super::image_operations::Resize(
        &bitmap.rgba8,
        (bitmap.width, bitmap.height),
        stride,
        true,
        (width as u32, height as u32),
        if nearest {
            super::super::image_operations::ResizeMethod::Box
        } else {
            super::super::image_operations::ResizeMethod::Lanczos3
        },
    )?
    else {
        return Ok(None);
    };
    Ok(Some(
        if alpha == image_resource::PaintImageAlphaType::Premultiplied {
            StraightDragBitmap(width as u32, height as u32, &pixels)?
        } else {
            image_resource::ImmutableImageBitmap {
                width: width as u32,
                height: height as u32,
                rgba8: Arc::new(pixels),
            }
        },
    ))
}
