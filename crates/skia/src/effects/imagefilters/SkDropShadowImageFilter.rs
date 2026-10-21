use crate::include::core::SkSize::SkSize;
use crate::src::core::{
    SkColor::SkColor4f, SkColorSpace::SkColorSpace, SkImageFilter::SkImageFilter, SkPoint::SkPoint,
    SkRect::SkRect,
};
use std::sync::Arc;
// Source: src/effects/imagefilters/SkDropShadowImageFilter.cpp:34-52
// Source: src/effects/imagefilters/SkDropShadowImageFilter.cpp:57-71
// Current Chromium CPU configuration does not define the legacy Blend-for-shadow
// switch; the native active branch constructs Merge. Serialization is separate.
fn make_drop_shadow_graph(
    offset: SkPoint,
    sigma: SkSize,
    color: SkColor4f,
    colorSpace: Option<Arc<SkColorSpace>>,
    shadowOnly: bool,
    input: Option<Arc<dyn SkImageFilter>>,
    crop: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    use crate::include::core::SkTileMode::SkTileMode::kDecal;
    let mut filter = input.clone();
    filter = super::SkBlurImageFilter::Blur(sigma.fWidth, sigma.fHeight, kDecal, filter, None);
    filter = super::SkColorFilterImageFilter::ColorFilter(
        crate::src::effects::colorfilters::SkBlendModeColorFilter::Blend(
            &color,
            colorSpace,
            crate::src::core::SkBlendMode::SkBlendMode::kSrcIn,
        ),
        filter,
        None,
    );
    filter = super::SkMatrixTransformImageFilter::MatrixTransform(
        &crate::src::core::SkMatrix::SkMatrix::Translate(offset.fX, offset.fY),
        crate::include::core::SkSamplingOptions::SkSamplingOptions::with_filter(
            crate::include::core::SkSamplingOptions::SkFilterMode::kLinear,
        ),
        filter,
    );
    if !shadowOnly {
        filter = super::SkMergeImageFilter::Merge(Some(&[filter, input]), None);
    }
    if let Some(crop) = crop {
        filter = super::SkCropImageFilter::Crop(crop, kDecal, filter);
    }
    filter
}
// Source: src/effects/imagefilters/SkDropShadowImageFilter.cpp:73-94
fn legacy_drop_shadow_create_proc(
    buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
) -> crate::src::core::SkFlattenable::FactoryResult {
    if !buffer.isVersionLT(crate::src::core::SkPicturePriv::kDropShadowImageFilterComposition) {
        return Ok(None);
    }
    let (child, crop) =
        crate::src::core::SkImageFilter_Base::SkImageFilter_Base::Unflatten(buffer)?;
    let dx = buffer.readScalar();
    let dy = buffer.readScalar();
    let sx = buffer.readScalar();
    let sy = buffer.readScalar();
    let color = SkColor4f::FromColor(buffer.readColor());
    let only = buffer.read32LE(1) != 0;
    Ok(make_drop_shadow_graph(
        SkPoint::make(dx, dy),
        SkSize::Make(sx, sy),
        color,
        None,
        only,
        child,
        crop,
    )
    .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
}
// Source: src/effects/imagefilters/SkDropShadowImageFilter.cpp:99-105
pub fn DropShadow(
    dx: f32,
    dy: f32,
    sigmaX: f32,
    sigmaY: f32,
    color: SkColor4f,
    colorSpace: Option<Arc<SkColorSpace>>,
    input: Option<Arc<dyn SkImageFilter>>,
    cropRect: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    make_drop_shadow_graph(
        SkPoint::make(dx, dy),
        SkSize::Make(sigmaX, sigmaY),
        color,
        colorSpace,
        false,
        input,
        cropRect,
    )
}
// Source: src/effects/imagefilters/SkDropShadowImageFilter.cpp:107-112
pub fn DropShadowOnly(
    dx: f32,
    dy: f32,
    sigmaX: f32,
    sigmaY: f32,
    color: SkColor4f,
    colorSpace: Option<Arc<SkColorSpace>>,
    input: Option<Arc<dyn SkImageFilter>>,
    cropRect: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    make_drop_shadow_graph(
        SkPoint::make(dx, dy),
        SkSize::Make(sigmaX, sigmaY),
        color,
        colorSpace,
        true,
        input,
        cropRect,
    )
}
// Source: src/effects/imagefilters/SkDropShadowImageFilter.cpp:115-118

/// Safety: exclusive initialization before global factory publication, with
/// capacity in the actual fixed table and no concurrent registry observers.
pub unsafe fn SkRegisterLegacyDropShadowImageFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkDropShadowImageFilter",
            legacy_drop_shadow_create_proc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkDropShadowImageFilterImpl",
            legacy_drop_shadow_create_proc,
        );
    }
}
