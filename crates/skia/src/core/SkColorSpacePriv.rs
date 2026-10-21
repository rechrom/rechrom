// Copyright 2016 Google Inc. BSD-style license.
// Source: src/core/SkColorSpacePriv.h:67-73
// Native namespace reopenings expose these same existing CICP producers.
pub use super::SkColorSpace::{SkNamedPrimaries, SkNamedTransferFn};
use crate::modules::skcms::src::skcms_public::{skcms_Matrix3x3, skcms_TransferFunction};
// Source: src/core/SkColorSpacePriv.h:15-19
pub const gNarrow_toXYZD50: skcms_Matrix3x3 = skcms_Matrix3x3 {
    vals: [
        [0.190974, 0.404865, 0.368380],
        [0.114746, 0.582937, 0.302318],
        [0.032925, 0.153615, 0.638669],
    ],
};
// Source: src/core/SkColorSpacePriv.h:21-23
pub fn color_space_almost_equal(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}
// Source: src/core/SkColorSpacePriv.h:27-29
pub fn transfer_fn_almost_equal(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.001
}
// Source: src/core/SkColorSpacePriv.h:31-39
pub fn is_almost_srgb(coeffs: &skcms_TransferFunction) -> bool {
    transfer_fn_almost_equal(SkNamedTransferFn::kSRGB.a, coeffs.a)
        && transfer_fn_almost_equal(SkNamedTransferFn::kSRGB.b, coeffs.b)
        && transfer_fn_almost_equal(SkNamedTransferFn::kSRGB.c, coeffs.c)
        && transfer_fn_almost_equal(SkNamedTransferFn::kSRGB.d, coeffs.d)
        && transfer_fn_almost_equal(SkNamedTransferFn::kSRGB.e, coeffs.e)
        && transfer_fn_almost_equal(SkNamedTransferFn::kSRGB.f, coeffs.f)
        && transfer_fn_almost_equal(SkNamedTransferFn::kSRGB.g, coeffs.g)
}
// Source: src/core/SkColorSpacePriv.h:41-47
pub fn is_almost_2dot2(coeffs: &skcms_TransferFunction) -> bool {
    transfer_fn_almost_equal(1.0, coeffs.a)
        && transfer_fn_almost_equal(0.0, coeffs.b)
        && transfer_fn_almost_equal(0.0, coeffs.e)
        && transfer_fn_almost_equal(2.2, coeffs.g)
        && coeffs.d <= 0.0
}
// Source: src/core/SkColorSpacePriv.h:49-49
// Source: src/core/SkColorSpacePriv.h:51-56
// Source: src/core/SkColorSpacePriv.h:59-62
// Source: src/core/SkColorSpacePriv.h:64-65
pub fn is_almost_linear(coeffs: &skcms_TransferFunction) -> bool {
    let linearExp = transfer_fn_almost_equal(1.0, coeffs.a)
        && transfer_fn_almost_equal(0.0, coeffs.b)
        && transfer_fn_almost_equal(0.0, coeffs.e)
        && transfer_fn_almost_equal(1.0, coeffs.g)
        && coeffs.d <= 0.0;
    let linearFn = transfer_fn_almost_equal(1.0, coeffs.c)
        && transfer_fn_almost_equal(0.0, coeffs.f)
        && coeffs.d >= 1.0;
    linearExp || linearFn
}

// Source: src/core/SkColorSpacePriv.h:77-78
// Borrow the original process-resident singleton allocation; no Arc clone/ref
// or replacement color-space owner is introduced by the native raw projection.
pub fn sk_srgb_singleton() -> &'static super::SkColorSpace::SkColorSpace {
    super::SkColorSpace::sk_srgb_singleton().as_ref()
}
pub fn sk_srgb_linear_singleton() -> &'static super::SkColorSpace::SkColorSpace {
    super::SkColorSpace::sk_srgb_linear_singleton().as_ref()
}
