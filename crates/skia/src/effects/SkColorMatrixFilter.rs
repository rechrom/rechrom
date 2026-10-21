// Copyright 2011 Google Inc. BSD-style license.
// Source: src/effects/SkColorMatrixFilter.cpp:17-24
fn byte_to_unit_float(byte: u32) -> f32 {
    if byte == 0xff {
        1.
    } else {
        byte as f32 * 0.00392156862745_f32
    }
}
// Source: src/effects/SkColorMatrixFilter.cpp:26-43
pub fn Lighting(
    mul: crate::src::core::SkColor::SkColor,
    add: crate::src::core::SkColor::SkColor,
) -> Option<std::sync::Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>> {
    use crate::src::core::SkColor::{SK_ColorBLACK, SkColorGetB, SkColorGetG, SkColorGetR};
    use crate::src::core::SkColorFilter::Clamp;
    use crate::src::effects::colorfilters::SkMatrixColorFilter::MatrixFromColorMatrix;
    let opaqueAlphaMask = SK_ColorBLACK;
    if add & !opaqueAlphaMask == 0 {
        return crate::src::effects::colorfilters::SkBlendModeColorFilter::BlendColor(
            mul | opaqueAlphaMask,
            crate::src::core::SkBlendMode::SkBlendMode::kModulate,
        );
    }
    let mut matrix = super::SkColorMatrix::SkColorMatrix::default();
    matrix.setScale(
        byte_to_unit_float(SkColorGetR(mul)),
        byte_to_unit_float(SkColorGetG(mul)),
        byte_to_unit_float(SkColorGetB(mul)),
        1.,
    );
    matrix.postTranslate(
        byte_to_unit_float(SkColorGetR(add)),
        byte_to_unit_float(SkColorGetG(add)),
        byte_to_unit_float(SkColorGetB(add)),
        0.,
    );
    MatrixFromColorMatrix(&matrix, Clamp::kYes)
}
