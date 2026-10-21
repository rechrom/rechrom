use crate::src::core::SkBlendMode::{SkBlendMode, SkBlendModeCoeff, SkBlendMode_AsCoeff};
// Source: src/core/SkPaintPriv.h:21-25
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ShaderOverrideOpacity {
    kNone_ShaderOverrideOpacity,
    kOpaque_ShaderOverrideOpacity,
    kNotOpaque_ShaderOverrideOpacity,
}
// Source: src/core/SkPaintPriv.cpp:36-39
fn changes_alpha(paint: &crate::src::core::SkPaint::SkPaint) -> bool {
    paint
        .getColorFilter()
        .is_some_and(|filter| !filter.isAlphaUnchanged())
}
// Source: src/core/SkPaintPriv.cpp:41-50
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
enum SrcColorOpacity {
    kOpaque_SrcColorOpacity = 0,
    kTransparentBlack_SrcColorOpacity = 1,
    kTransparentAlpha_SrcColorOpacity = 2,
    kUnknown_SrcColorOpacity = 3,
}
// Source: src/core/SkPaintPriv.cpp:52-81
fn blend_mode_is_opaque(mode: SkBlendMode, opacity: SrcColorOpacity) -> bool {
    let Some((src, dst)) = SkBlendMode_AsCoeff(mode) else {
        return false;
    };
    match src {
        SkBlendModeCoeff::kDA
        | SkBlendModeCoeff::kDC
        | SkBlendModeCoeff::kIDA
        | SkBlendModeCoeff::kIDC => return false,
        _ => {}
    }
    match dst {
        SkBlendModeCoeff::kZero => true,
        SkBlendModeCoeff::kISA => opacity == SrcColorOpacity::kOpaque_SrcColorOpacity,
        SkBlendModeCoeff::kSA => {
            opacity == SrcColorOpacity::kTransparentBlack_SrcColorOpacity
                || opacity == SrcColorOpacity::kTransparentAlpha_SrcColorOpacity
        }
        SkBlendModeCoeff::kSC => opacity == SrcColorOpacity::kTransparentBlack_SrcColorOpacity,
        _ => false,
    }
}
// Source: src/core/SkPaintPriv.cpp:83-111
pub fn Overwrites(
    paint: Option<&crate::src::core::SkPaint::SkPaint>,
    override_opacity: ShaderOverrideOpacity,
) -> bool {
    use ShaderOverrideOpacity::*;
    use SrcColorOpacity::*;
    let Some(paint) = paint else {
        return override_opacity != kNotOpaque_ShaderOverrideOpacity;
    };
    let mut opacity = kUnknown_SrcColorOpacity;
    if !changes_alpha(paint) {
        let alpha = paint.getAlpha();
        if alpha == 255
            && override_opacity != kNotOpaque_ShaderOverrideOpacity
            && paint.getShader().is_none_or(|shader| shader.isOpaque())
        {
            opacity = kOpaque_SrcColorOpacity;
        } else if alpha == 0 {
            opacity =
                if override_opacity == kNone_ShaderOverrideOpacity && paint.getShader().is_none() {
                    kTransparentBlack_SrcColorOpacity
                } else {
                    kTransparentAlpha_SrcColorOpacity
                };
        }
    }
    let Some(mode) = paint.asBlendMode() else {
        return false;
    };
    blend_mode_is_opaque(mode, opacity)
}
// Source: src/core/SkPaintPriv.cpp:113-131
pub fn ShouldDither(
    paint: &crate::src::core::SkPaint::SkPaint,
    dst_ct: crate::include::core::SkColorType::SkColorType,
) -> bool {
    use crate::include::core::SkColorType::SkColorType::*;
    if !paint.isDither() {
        return false;
    }
    if dst_ct == kUnknown_SkColorType {
        return false;
    }
    if dst_ct == kRGB_565_SkColorType || dst_ct == kARGB_4444_SkColorType {
        return true;
    }
    paint.getImageFilter().is_some()
        || paint.getMaskFilter().is_some()
        || paint
            .getShader()
            .is_some_and(|shader| !shader.isConstant(None))
}

// Source: src/core/SkPaintPriv.cpp:135-151
// Adapted: canonical CPU color filtering is fallible while stages remain incomplete.
// Propagate its real error; only a nonconstant shader produces native false.
fn just_a_color(
    paint: &crate::src::core::SkPaint::SkPaint,
    color: Option<&mut crate::src::core::SkColor::SkColor4f>,
) -> Result<bool, crate::src::effects::colorfilters::SkColorFilterBase::ColorFilterEvaluationError>
{
    let mut c = paint.getColor4f();
    if paint
        .getShader()
        .is_some_and(|shader| !shader.asLuminanceColor(Some(&mut c)))
    {
        return Ok(false);
    }
    if let Some(filter) = paint.getColorFilter() {
        c = filter.filterColor4f(&c, None, None)?;
    }
    if let Some(color) = color {
        *color = c;
    }
    Ok(true)
}
// Source: src/core/SkPaintPriv.cpp:153-159
// Adapted: expose the canonical filter executor's real failure rather than substitute a color.
pub fn ComputeLuminanceColor(
    paint: &crate::src::core::SkPaint::SkPaint,
) -> Result<
    crate::src::core::SkColor::SkColor,
    crate::src::effects::colorfilters::SkColorFilterBase::ColorFilterEvaluationError,
> {
    let mut color = crate::src::core::SkColor::SkColor4f {
        fR: 0.0,
        fG: 0.0,
        fB: 0.0,
        fA: 0.0,
    };
    if !just_a_color(paint, Some(&mut color))? {
        color = crate::src::core::SkColor::SkColor4f {
            fR: 0.5,
            fG: 0.5,
            fB: 0.5,
            fA: 1.0,
        };
    }
    Ok(color.toSkColor())
}
// Source: src/core/SkPaintPriv.cpp:161-175
// Adapted: four native Paint effect fields still retain actual canonical Arc objects;
// the actual color-filter Result propagates before Paint is changed on failure.
pub fn RemoveColorFilter(
    paint: &mut crate::src::core::SkPaint::SkPaint,
    dst_cs: Option<&std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
) -> Result<(), crate::src::effects::colorfilters::SkColorFilterBase::ColorFilterEvaluationError> {
    if let Some(filter) = paint.refColorFilter() {
        if let Some(shader) = paint.refShader() {
            let shader = crate::src::shaders::SkColorFilterShader::SkColorFilterShader::Make(
                Some(shader),
                paint.getAlphaf(),
                Some(filter),
            );
            paint.setShader(shader);
            paint.setAlphaf(1.0);
        } else {
            // MakeSRGB is the real current singleton owner API; its Arc is retained
            // only through the synchronous native color-space conversion operation.
            let srgb = crate::src::core::SkColorSpace::SkColorSpace::MakeSRGB();
            let color = filter.filterColor4f(&paint.getColor4f(), Some(&srgb), dst_cs)?;
            paint.setColor4f(&color, dst_cs.map(std::sync::Arc::as_ref));
        }
        paint.setColorFilter(None);
    }
    Ok(())
}

// Source: src/core/SkPaintPriv.cpp:178-184
#[cfg(sk_debug)]
fn assert_fits_in(value: u32, bit_count: u32) {
    crate::SkASSERT!(@source b"src/core/SkPaintPriv.cpp\0".as_ptr(), 179; @condition b"bitCount > 0 && bitCount <= 32\0".as_ptr(); bit_count>0&&bit_count<=32);
    let mut mask = !0u32;
    mask >>= 32 - bit_count;
    debug_assert_eq!(value & !mask, 0);
}
// Source: src/core/SkPaintPriv.cpp:188-194
const HAS_TYPEFACE_FLAT_FLAG: u32 = 1;
const HAS_EFFECTS_FLAT_FLAG: u32 = 2;
const FLAT_FLAG_MASK: u32 = 3;
// Source: src/core/SkPaintPriv.cpp:198-203
fn shift_bits(value: u32, shift: u32, bits: u32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkPaintPriv.cpp\0".as_ptr(), 199; @condition b"shift + bits <= 32\0".as_ptr(); shift+bits<=32);
    let v = value;
    #[cfg(sk_debug)]
    assert_fits_in(v, bits);
    v << shift
}
// Source: src/core/SkPaintPriv.cpp:205-205
const CUSTOM_BLEND_MODE_SENTINEL: u32 = 255;
// Source: src/core/SkPaintPriv.cpp:217-232
fn pack_v68(paint: &crate::src::core::SkPaint::SkPaint, flat_flags: u32) -> u32 {
    let mut packed = 0;
    let mode = paint
        .asBlendMode()
        .map_or(CUSTOM_BLEND_MODE_SENTINEL, |value| value as u32);
    packed |= shift_bits(
        ((paint.isDither() as u32) << 1) | (paint.isAntiAlias() as u32),
        0,
        8,
    );
    packed |= shift_bits(mode, 8, 8);
    packed |= shift_bits(paint.getStrokeCap() as u32, 16, 2);
    packed |= shift_bits(paint.getStrokeJoin() as u32, 18, 2);
    packed |= shift_bits(paint.getStyle() as u32, 20, 2);
    packed |= shift_bits(0, 22, 2);
    packed |= shift_bits(flat_flags, 24, 8);
    packed
}
// Source: src/core/SkPaintPriv.cpp:234-256
fn unpack_v68(
    paint: &mut crate::src::core::SkPaint::SkPaint,
    mut packed: u32,
    safe: &mut crate::src::core::SkSafeRange::SkSafeRange,
) -> u32 {
    use crate::src::core::SkPaint::{Cap, Join, Style};
    paint.setAntiAlias(packed & 1 != 0);
    paint.setDither(packed & 2 != 0);
    packed >>= 8;
    let mode = packed & 255;
    if mode != CUSTOM_BLEND_MODE_SENTINEL {
        let mode = safe.checkLE(mode as u64, SkBlendMode::kLastMode as u32);
        // Native checkLE guarantees a legal contiguous blend-mode discriminant.
        paint.setBlendMode(unsafe { core::mem::transmute::<u8, SkBlendMode>(mode as u8) });
    }
    packed >>= 8;
    let cap = safe.checkLE((packed & 3) as u64, Cap::kLast_Cap as u32);
    paint.setStrokeCap(match cap {
        0 => Cap::kButt_Cap,
        1 => Cap::kRound_Cap,
        _ => Cap::kSquare_Cap,
    });
    packed >>= 2;
    let join = safe.checkLE((packed & 3) as u64, Join::kLast_Join as u32);
    paint.setStrokeJoin(match join {
        0 => Join::kMiter_Join,
        1 => Join::kRound_Join,
        _ => Join::kBevel_Join,
    });
    packed >>= 2;
    let style = safe.checkLE((packed & 3) as u64, Style::kStrokeAndFill_Style as u32);
    paint.setStyle(match style {
        0 => Style::kFill_Style,
        1 => Style::kStroke_Style,
        _ => Style::kStrokeAndFill_Style,
    });
    packed >>= 2;
    packed >>= 2;
    packed
}
// Source: src/core/SkPaintPriv.cpp:261-287
pub fn Flatten(
    paint: &crate::src::core::SkPaint::SkPaint,
    buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
) -> Result<(), crate::RasterError> {
    use crate::src::core::SkFlattenable::SkFlattenableView;
    let mut flat_flags = 0;
    if paint.getPathEffect().is_some()
        || paint.getShader().is_some()
        || paint.getMaskFilter().is_some()
        || paint.getColorFilter().is_some()
        || paint.getImageFilter().is_some()
        || paint.asBlendMode().is_none()
    {
        flat_flags |= HAS_EFFECTS_FLAT_FLAG;
    }
    buffer.writeScalar(paint.getStrokeWidth());
    buffer.writeScalar(paint.getStrokeMiter());
    buffer.writeColor4f(&paint.getColor4f());
    buffer.write32(pack_v68(paint, flat_flags) as i32);
    if flat_flags & HAS_EFFECTS_FLAT_FLAG != 0 {
        buffer.writeFlattenable(
            paint
                .getPathEffect()
                .map(|value| value as &dyn SkFlattenableView),
        )?;
        buffer.writeFlattenable(
            paint
                .getShader()
                .map(|value| value as &dyn SkFlattenableView),
        )?;
        buffer.writeFlattenable(
            paint
                .getMaskFilter()
                .map(|value| value as &dyn SkFlattenableView),
        )?;
        buffer.writeFlattenable(
            paint
                .getColorFilter()
                .map(|value| value as &dyn SkFlattenableView),
        )?;
        buffer.writeFlattenable(
            paint
                .getImageFilter()
                .map(|value| value as &dyn SkFlattenableView),
        )?;
        buffer.writeFlattenable(
            paint
                .getBlender()
                .map(|value| value as &dyn SkFlattenableView),
        )?;
    }
    Ok(())
}

// Source: src/core/SkPaintPriv.cpp:289-331
pub fn Unflatten(
    buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
) -> Result<crate::src::core::SkPaint::SkPaint, crate::RasterError> {
    use crate::include::core::SkRefCnt::sk_sp;
    let mut paint = crate::src::core::SkPaint::SkPaint::default();
    paint.setStrokeWidth(buffer.readScalar());
    paint.setStrokeMiter(buffer.readScalar());
    let mut color = crate::src::core::SkColor::SkColor4f {
        fR: 0.0,
        fG: 0.0,
        fB: 0.0,
        fA: 0.0,
    };
    buffer.readColor4f(&mut color);
    let srgb = crate::src::core::SkColorSpace::SkColorSpace::MakeSRGB();
    paint.setColor4f(&color, Some(&srgb));
    let mut safe = crate::src::core::SkSafeRange::SkSafeRange::new();
    let flat_flags = unpack_v68(&mut paint, buffer.readUInt(), &mut safe);
    if flat_flags & HAS_EFFECTS_FLAT_FLAG == 0 {
        paint.setPathEffect(sk_sp::default());
        paint.setShader(None);
        paint.setMaskFilter(None);
        paint.setColorFilter(None);
        paint.setImageFilter(None);
    } else if buffer.isVersionLT(crate::src::core::SkPicturePriv::kSkBlenderInSkPaint) {
        paint.setPathEffect(buffer.readPathEffect()?);
        paint.setShader(buffer.readShader()?);
        paint.setMaskFilter(buffer.readMaskFilter()?);
        paint.setColorFilter(buffer.readColorFilter()?);
        buffer.read32();
        paint.setImageFilter(buffer.readImageFilter()?);
    } else {
        paint.setPathEffect(buffer.readPathEffect()?);
        paint.setShader(buffer.readShader()?);
        paint.setMaskFilter(buffer.readMaskFilter()?);
        paint.setColorFilter(buffer.readColorFilter()?);
        paint.setImageFilter(buffer.readImageFilter()?);
        paint.setBlender(buffer.readBlender()?);
    }
    if !buffer.validate(safe.ok()) {
        paint.reset();
    }
    Ok(paint)
}
