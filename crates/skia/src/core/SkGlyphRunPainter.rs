use crate::include::core::SkColorType::SkColorType;
use crate::include::core::SkSurfaceProps::{SkPixelGeometry, SkSurfaceProps};
use crate::src::core::SkColorSpace::SkColorSpace;
use crate::src::core::SkScalerContext::SkScalerContextFlags;
// Source: src/core/SkGlyphRunPainter.h:29-30
// Source: src/core/SkGlyphRunPainter.h:37-37
// Source: src/core/SkGlyphRunPainter.h:39-39
// Source: src/core/SkGlyphRunPainter.h:42-45
#[derive(Clone, Copy)]
pub struct GlyphRunListPainter {
    fDeviceProps: SkSurfaceProps,
    fBitmapFallbackProps: SkSurfaceProps,
    fColorType: SkColorType,
    fScalerContextFlags: SkScalerContextFlags,
}
// Source: src/core/SkGlyphRunPainter.cpp:47-47
// Source: src/core/SkGlyphRunPainter.cpp:51-56
fn compute_scaler_context_flags(cs: Option<&SkColorSpace>) -> SkScalerContextFlags {
    if cs.is_some_and(SkColorSpace::gammaIsLinear) {
        SkScalerContextFlags::BOOST_CONTRAST
    } else {
        SkScalerContextFlags::FAKE_GAMMA_AND_BOOST_CONTRAST
    }
}
// Source: src/core/SkGlyphRunPainter.cpp:60-87
// The actual Strike guard prepares each digest/glyph under one native lock.
// Accepted arena pointers retain their strike owner through the later draw.
unsafe fn prepare_for_path_drawing(
    strike: &crate::src::core::SkStrike::SkStrike,
    source: &[(u16, crate::src::core::SkPoint::SkPoint)],
    accepted: &mut Vec<(
        core::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph>,
        crate::src::core::SkPoint::SkPoint,
    )>,
    rejected: &mut Vec<(u16, crate::src::core::SkPoint::SkPoint)>,
) -> Result<(), crate::RasterError> {
    use crate::src::core::{
        SkGlyph::{ActionType, GlyphAction, SkPackedGlyphID},
        SkPoint::SkPoint,
    };
    accepted.clear();
    rejected.clear();
    let mut locked = unsafe { strike.lockForGlyphRun() };
    for &(id, pos) in source {
        if !pos.fX.is_finite() || !pos.fY.is_finite() {
            continue;
        }
        let packed = SkPackedGlyphID::from_glyph(id);
        let digest = unsafe { locked.digestFor(ActionType::kPath, packed) }?;
        match digest.actionFor(ActionType::kPath) {
            GlyphAction::kAccept => accepted.push((unsafe { locked.glyph(digest) }, pos)),
            GlyphAction::kReject => rejected.push((id, pos)),
            _ => {}
        }
    }
    drop(locked);
    Ok(())
}

// Source: src/core/SkGlyphRunPainter.cpp:91-118
// The actual Strike guard prepares each digest/glyph under one native lock.
// Accepted arena pointers retain their strike owner through the later draw.
unsafe fn prepare_for_drawable_drawing(
    strike: &crate::src::core::SkStrike::SkStrike,
    source: &[(u16, crate::src::core::SkPoint::SkPoint)],
    accepted: &mut Vec<(
        core::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph>,
        crate::src::core::SkPoint::SkPoint,
    )>,
    rejected: &mut Vec<(u16, crate::src::core::SkPoint::SkPoint)>,
) -> Result<(), crate::RasterError> {
    use crate::src::core::{
        SkGlyph::{ActionType, GlyphAction, SkPackedGlyphID},
        SkPoint::SkPoint,
    };
    accepted.clear();
    rejected.clear();
    let mut locked = unsafe { strike.lockForGlyphRun() };
    for &(id, pos) in source {
        if !pos.fX.is_finite() || !pos.fY.is_finite() {
            continue;
        }
        let packed = SkPackedGlyphID::from_glyph(id);
        let digest = unsafe { locked.digestFor(ActionType::kDrawable, packed) }?;
        match digest.actionFor(ActionType::kDrawable) {
            GlyphAction::kAccept => accepted.push((unsafe { locked.glyph(digest) }, pos)),
            GlyphAction::kReject => rejected.push((id, pos)),
            _ => {}
        }
    }
    drop(locked);
    Ok(())
}

// Source: src/core/SkGlyphRunPainter.cpp:120-163
// The actual Strike guard prepares each digest/glyph under one native lock.
// Accepted arena pointers retain their strike owner through the later draw.
unsafe fn prepare_for_direct_mask_drawing(
    strike: &crate::src::core::SkStrike::SkStrike,
    creation: &crate::src::core::SkMatrix::SkMatrix,
    source: &[(u16, crate::src::core::SkPoint::SkPoint)],
    accepted: &mut Vec<(
        core::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph>,
        crate::src::core::SkPoint::SkPoint,
    )>,
    rejected: &mut Vec<(u16, crate::src::core::SkPoint::SkPoint)>,
) -> Result<(), crate::RasterError> {
    use crate::src::core::{
        SkGlyph::{ActionType, GlyphAction, SkPackedGlyphID},
        SkPoint::SkPoint,
    };
    let mask = strike.roundingSpec().ignore_position_field_mask;
    let half = strike.roundingSpec().half_axis_sample_freq;
    let mut rounded = creation.clone();
    rounded.postTranslate(half.fX, half.fY);
    accepted.clear();
    rejected.clear();
    let mut locked = unsafe { strike.lockForGlyphRun() };
    for &(id, pos) in source {
        if !pos.fX.is_finite() || !pos.fY.is_finite() {
            continue;
        }
        let mapped = rounded.mapPoint(pos);
        let packed = SkPackedGlyphID::from_point(id, mapped, mask);
        let digest = unsafe { locked.digestFor(ActionType::kDirectMaskCPU, packed) }?;
        match digest.actionFor(ActionType::kDirectMaskCPU) {
            GlyphAction::kAccept => accepted.push((
                unsafe { locked.glyph(digest) },
                SkPoint::make(mapped.fX.floor(), mapped.fY.floor()),
            )),
            GlyphAction::kReject => rejected.push((id, pos)),
            _ => {}
        }
    }
    drop(locked);
    Ok(())
}

// Source: src/core/SkGlyphRunPainter.cpp:166-207
// The actual Strike guard prepares each digest/glyph under one native lock.
// Accepted arena pointers retain their strike owner through the later draw.
unsafe fn prepare_for_direct_bitmap_drawing(
    strike: &crate::src::core::SkStrike::SkStrike,
    creation: &crate::src::core::SkMatrix::SkMatrix,
    source: &[(u16, crate::src::core::SkPoint::SkPoint)],
    accepted: &mut Vec<(
        core::ptr::NonNull<crate::src::core::SkGlyph::SkGlyph>,
        crate::src::core::SkPoint::SkPoint,
    )>,
    rejected: &mut Vec<(u16, crate::src::core::SkPoint::SkPoint)>,
) -> Result<(), crate::RasterError> {
    use crate::src::core::{
        SkGlyph::{ActionType, GlyphAction, SkPackedGlyphID},
        SkPoint::SkPoint,
    };
    let mask = strike.roundingSpec().ignore_position_field_mask;
    let half = strike.roundingSpec().half_axis_sample_freq;
    let mut rounded = creation.clone();
    rounded.postTranslate(half.fX, half.fY);
    accepted.clear();
    rejected.clear();
    let mut locked = unsafe { strike.lockForGlyphRun() };
    for &(id, pos) in source {
        if !pos.fX.is_finite() || !pos.fY.is_finite() {
            continue;
        }
        let mapped = rounded.mapPoint(pos);
        let packed = SkPackedGlyphID::from_point(id, mapped, mask);
        let digest = unsafe { locked.digestFor(ActionType::kDirectMaskCPU, packed) }?;
        match digest.actionFor(ActionType::kDirectMaskCPU) {
            GlyphAction::kAccept => accepted.push((unsafe { locked.glyph(digest) }, pos)),
            GlyphAction::kReject => rejected.push((id, pos)),
            _ => {}
        }
    }
    drop(locked);
    Ok(())
}

impl GlyphRunListPainter {
    // Source: src/core/SkGlyphRunPainter.cpp:211-217
    // Source: src/core/SkGlyphRunPainter.h:31-31
    pub fn new(props: &SkSurfaceProps, color_type: SkColorType, cs: Option<&SkColorSpace>) -> Self {
        Self {
            fDeviceProps: *props,
            fBitmapFallbackProps: props
                .cloneWithPixelGeometry(SkPixelGeometry::kUnknown_SkPixelGeometry),
            fColorType: color_type,
            fScalerContextFlags: compute_scaler_context_flags(cs),
        }
    }
}

impl GlyphRunListPainter {
    // Source: src/core/SkGlyphRunPainter.h:33-36
    // Source: src/core/SkGlyphRunPainter.cpp:219-423
    /// Safety: original Canvas/pixels and strike allocations retain their native
    /// leases. Draw CTM/clip/props and this painter are independent value projections
    /// so no complete BitmapDevice borrow survives a Canvas callback.
    pub unsafe fn drawForBitmapDevice<'canvas>(
        &self,
        canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'canvas>>,
        bitmap: &dyn crate::src::core::SkDraw::BitmapDevicePainter,
        list: &crate::src::text::GlyphRun::GlyphRunList,
        paint: &crate::src::core::SkPaint::SkPaint,
        draw_matrix: &crate::src::core::SkMatrix::SkMatrix,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::{
            SkBitmap::SkBitmap,
            SkCanvas::SkCanvas,
            SkImageInfo::SkImageInfo,
            SkMask::Format,
            SkMatrix::SkMatrix,
            SkPaint::Style,
            SkPath::AddPathMode,
            SkPathBuilder::SkPathBuilder,
            SkPoint::SkPoint,
            SkStrikeSpec::{SkBulkGlyphMetrics, SkStrikeSpec},
        };
        let count = unsafe { list.maxGlyphRunSize() };
        // Native four parallel scratch arrays become two ordered tuple buffers.
        // Capacity is allocated once; no glyph pointer is initialized before acceptance.
        let mut accepted = Vec::with_capacity(count);
        let mut rejected = Vec::with_capacity(count);
        let props = if self.fColorType == crate::include::core::SkColorType::kN32_SkColorType
            && paint.isSrcOver()
        {
            &self.fDeviceProps
        } else {
            &self.fBitmapFallbackProps
        };
        let origin = list.origin();
        let mut position_matrix = draw_matrix.clone();
        position_matrix.preTranslate(origin.fX, origin.fY);
        for run in unsafe { list.runs() } {
            let font = run.font();
            let mut source: Vec<(u16, SkPoint)> = unsafe { run.glyphsIDs() }
                .iter()
                .copied()
                .zip(unsafe { run.positions() }.iter().copied())
                .collect();
            if SkStrikeSpec::ShouldDrawAsPath(paint, font, &position_matrix) {
                let (spec, scale) =
                    SkStrikeSpec::MakePath(font, paint, props, self.fScalerContextFlags)?;
                let strike = unsafe { spec.findOrCreateStrike() };
                unsafe {
                    prepare_for_path_drawing(&strike, &source, &mut accepted, &mut rejected)
                }?;
                core::mem::swap(&mut source, &mut rejected);
                let mut path_paint = paint.clone();
                path_paint.setAntiAlias(font.hasSomeAntiAliasing());
                let stroking = path_paint.getStyle() != Style::kFill_Style;
                let hairline = path_paint.getStrokeWidth() == 0.0;
                let exact = path_paint.getShader().is_some()
                    || path_paint.getPathEffect().is_some()
                    || path_paint.getMaskFilter().is_some()
                    || (stroking && !hairline);
                if !exact {
                    for &(glyph, pos) in &accepted {
                        let path = unsafe { glyph.as_ref().path() }
                            .expect("accepted path glyph owns a path");
                        let translate = origin + pos;
                        let mut matrix = SkMatrix::default();
                        matrix.setScaleTranslate(scale, scale, translate.fX, translate.fY);
                        let count = unsafe { canvas.as_ref() }.getSaveCount();
                        unsafe { core::pin::Pin::new_unchecked(&mut *canvas.as_ptr()).save() };
                        unsafe {
                            core::pin::Pin::new_unchecked(&mut *canvas.as_ptr()).concat(&matrix)
                        };
                        let result = unsafe { SkCanvas::drawPath_raw(canvas, path, &path_paint) };
                        let restore = unsafe { SkCanvas::restoreToCount_raw(canvas, count) };
                        restore?;
                        result?;
                    }
                } else {
                    for &(glyph, pos) in &accepted {
                        let path = unsafe { glyph.as_ref().path() }
                            .expect("accepted path glyph owns a path");
                        let translate = origin + pos;
                        let mut matrix = SkMatrix::default();
                        matrix.setScaleTranslate(scale, scale, translate.fX, translate.fY);
                        let mut builder = SkPathBuilder::new();
                        builder.addPath(path, &matrix, AddPathMode::kAppend_AddPathMode);
                        builder.setIsVolatile(true);
                        unsafe {
                            SkCanvas::drawPath_raw(canvas, &builder.detach(None), &path_paint)
                        }?;
                    }
                }
                if !source.is_empty() {
                    unsafe {
                        prepare_for_drawable_drawing(&strike, &source, &mut accepted, &mut rejected)
                    }?;
                    core::mem::swap(&mut source, &mut rejected);
                    for &(glyph, pos) in &accepted {
                        let drawable = unsafe { glyph.as_ref().drawable() }
                            .expect("accepted drawable glyph owns drawable");
                        let translate = origin + pos;
                        let mut matrix = SkMatrix::default();
                        matrix.setScaleTranslate(scale, scale, translate.fX, translate.fY);
                        let count = unsafe { canvas.as_ref() }.getSaveCount();
                        let bounds = matrix.mapRect_value(&drawable.getBounds());
                        let result = (|| {
                            unsafe {
                                core::pin::Pin::new_unchecked(&mut *canvas.as_ptr())
                                    .saveLayer_bounds(Some(&bounds), Some(paint))
                            }?;
                            unsafe { drawable.draw(canvas, Some(&matrix)) }
                        })();
                        let restore = unsafe { SkCanvas::restoreToCount_raw(canvas, count) };
                        restore?;
                        result?;
                    }
                }
            }
            if !source.is_empty() && !position_matrix.hasPerspective() {
                let spec = SkStrikeSpec::MakeMask(
                    font,
                    paint,
                    props,
                    self.fScalerContextFlags,
                    &position_matrix,
                )?;
                let strike = unsafe { spec.findOrCreateStrike() };
                unsafe {
                    prepare_for_direct_mask_drawing(
                        &strike,
                        &position_matrix,
                        &source,
                        &mut accepted,
                        &mut rejected,
                    )
                }?;
                core::mem::swap(&mut source, &mut rejected);
                let masks: Vec<_> = accepted
                    .iter()
                    .map(|&(glyph, pos)| (unsafe { glyph.as_ref() }, pos))
                    .collect();
                unsafe { bitmap.paintMasks(&masks, paint) }?;
            }
            if !source.is_empty() {
                let scale_spec = SkStrikeSpec::MakeMask(
                    font,
                    paint,
                    props,
                    self.fScalerContextFlags,
                    &SkMatrix::I(),
                )?;
                let mut metrics = unsafe { SkBulkGlyphMetrics::new(&scale_spec) };
                let ids: Vec<_> = source.iter().map(|&(id, _)| id).collect();
                let glyphs = unsafe { metrics.glyphs(&ids) };
                let mut max_scale = crate::include::core::SkScalar::SK_ScalarMin;
                for (&glyph, &(_, pos)) in glyphs.iter().zip(&source) {
                    let glyph = unsafe { glyph.as_ref() };
                    if glyph.isEmpty() {
                        continue;
                    }
                    let rect = glyph.rect();
                    // Native makeOffset returns a new rectangle and its result is
                    // intentionally unused in the pinned implementation.
                    let _ = rect.makeOffsetVec(origin + pos);
                    let mut corners = [SkPoint::make(0.0, 0.0); 4];
                    position_matrix.mapRectToQuad(&mut corners, &rect);
                    let scale = (corners[1] - corners[0]).length() / rect.width();
                    if max_scale < scale {
                        max_scale = scale;
                    }
                    let scale = (corners[2] - corners[1]).length() / rect.height();
                    if max_scale < scale {
                        max_scale = scale;
                    }
                    let scale = (corners[3] - corners[2]).length() / rect.width();
                    if max_scale < scale {
                        max_scale = scale;
                    }
                    let scale = (corners[0] - corners[3]).length() / rect.height();
                    if max_scale < scale {
                        max_scale = scale;
                    }
                }
                if max_scale <= 0.0 {
                    continue;
                }
                if max_scale * font.getSize() > 256.0 {
                    max_scale = 256.0 / font.getSize();
                }
                let cache_scale = SkMatrix::Scale(max_scale, max_scale);
                let spec = SkStrikeSpec::MakeMask(
                    font,
                    paint,
                    props,
                    self.fScalerContextFlags,
                    &cache_scale,
                )?;
                let strike = unsafe { spec.findOrCreateStrike() };
                unsafe {
                    prepare_for_direct_bitmap_drawing(
                        &strike,
                        &position_matrix,
                        &source,
                        &mut accepted,
                        &mut rejected,
                    )
                }?;
                let inverse = 1.0 / max_scale;
                for &(glyph, pos) in &accepted {
                    let mask = unsafe { glyph.as_ref() }.mask();
                    if mask.fFormat != Format::kARGB32_Format {
                        continue;
                    }
                    let mut bm = SkBitmap::default();
                    unsafe {
                        bm.installPixels(
                            &SkImageInfo::MakeN32Premul(
                                mask.fBounds.width(),
                                mask.fBounds.height(),
                            ),
                            mask.fImage.cast_mut().cast(),
                            mask.fRowBytes as usize,
                            None,
                            core::ptr::null_mut(),
                        )
                    };
                    bm.setImmutable();
                    let pos = origin
                        + pos
                        + SkPoint::make(mask.fBounds.fLeft as f32, mask.fBounds.fTop as f32)
                            * inverse;
                    let mut translate = SkMatrix::TranslateVector(pos);
                    translate.preScale(inverse, inverse);
                    unsafe {
                        bitmap.drawBitmap(&bm,&translate,None,
                        &crate::include::core::SkSamplingOptions::SkSamplingOptions::with_filter(
                            crate::include::core::SkSamplingOptions::SkFilterMode::kLinear),paint,None)
                    }?;
                }
            }
        }
        Ok(())
    }
}
