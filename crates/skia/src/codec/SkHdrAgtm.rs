// Source: src/codec/SkHdrAgtm.cpp
use super::SkHdrAgtmPriv::AgtmHelpers::Weighting;
use super::SkHdrMetadata::{
    AdaptiveGlobalToneMap, ColorGainFunction, ComponentMixingFunction,
    ContentLightLevelInformation, ControlPoint, GainCurve, HeadroomAdaptiveToneMap,
    MasteringDisplayColorVolume, Metadata,
};
use crate::src::core::{
    SkColor::SkColor4f,
    SkColorFilter::SkColorFilter,
    SkColorSpace::{SkColorSpace, SkNamedPrimaries, SkNamedTransferFn},
};
use std::sync::Arc;
unsafe extern "C" {
    fn log2f(x: f32) -> f32;
    fn exp2f(x: f32) -> f32;
    fn logf(x: f32) -> f32;
}
// C++ std::min/max return the first operand on unordered/equal comparison.
fn min(a: f32, b: f32) -> f32 {
    if b < a {
        b
    } else {
        a
    }
}
fn max(a: f32, b: f32) -> f32 {
    if a < b {
        b
    } else {
        a
    }
}
// Source: src/codec/SkHdrAgtm.cpp:18
const kMinHdrHeadroom: f32 = 0.0;
const kMaxHdrHeadroom: f32 = 6.0;
const kMaxLinearHdrHeadroom: f32 = 64.0;
// Source: src/codec/SkHdrAgtm.cpp:24
fn in_unit_interval(x: f32) -> bool {
    x >= 0.0 && x <= 1.0
}
// Source: src/codec/SkHdrAgtm.cpp:29
const gAgtmSKSL: &str = concat!(
    "uniform half scale_factor;",
    "uniform shader curve_xym;",
    "uniform half weight_i;",
    "uniform half4 mix_rgbx_i;",
    "uniform half4 mix_Mmcx_i;",
    "uniform half curve_texcoord_y_i;",
    "uniform half curve_N_cp_i;",
    "uniform half weight_j;",
    "uniform half4 mix_rgbx_j;",
    "uniform half4 mix_Mmcx_j;",
    "uniform half curve_texcoord_y_j;",
    "uniform half curve_N_cp_j;",
    "half3 EvalComponentMixing(half3 color, half4 rgbx, half4 Mmcx) {",
    "half common = dot(rgbx.rgb, color) +",
    "Mmcx[0] * max(max(color.r, color.g), color.b) +",
    "Mmcx[1] * min(min(color.r, color.g), color.b);",
    "return Mmcx[2] * color + half3(common);",
    "}",
    "half EvalGainCurve(half x, half curve_texcoord_y, half curve_N_cp) {",
    "half c_min = 0.0;",
    "half4 xym_min = curve_xym.eval(half2(c_min + 0.5, curve_texcoord_y));",
    "if (x <= xym_min.x) {",
    "return xym_min.y;",
    "}",
    "half c_max = curve_N_cp - 1.0;",
    "half4 xym_max = curve_xym.eval(half2(c_max + 0.5, curve_texcoord_y));",
    "if (x >= xym_max.x) {",
    "return xym_max.y + log2(xym_max.x / x);",
    "}",
    "for (int step = 0; step < 5; ++step) {",
    "if (c_max - c_min <= 1.0) {",
    "break;",
    "}",
    "half c_mid = ceil(0.5 * (c_min + c_max));",
    "half4 xym_mid = curve_xym.eval(half2(c_mid + 0.5, curve_texcoord_y));",
    "if (x == xym_mid.x) {",
    "return xym_mid.y;",
    "} else if (x < xym_mid.x) {",
    "c_max = c_mid;",
    "xym_max = xym_mid;",
    "} else {",
    "c_min = c_mid;",
    "xym_min = xym_mid;",
    "}",
    "}",
    "half h = xym_max.x - xym_min.x;",
    "half mHat_min = xym_min.z * h;",
    "half mHat_max = xym_max.z * h;",
    "half c3 =  2.0 * xym_min.y + mHat_min - 2.0 * xym_max.y + mHat_max;",
    "half c2 = -3.0 * xym_min.y + 3.0 * xym_max.y - 2.0 * mHat_min - mHat_max;",
    "half c1 = mHat_min;",
    "half c0 = xym_min.y;",
    "half t = (x - xym_min.x) / h;",
    "return ((c3*t + c2)*t + c1)*t + c0;",
    "}",
    "half3 EvalColorGainFunction(half3 color,",
    "half4 mix_rgbx, half4 mix_Mmcx,",
    "float curve_texcoord_y, float curve_N_cp) {",
    "half3 M = EvalComponentMixing(color, mix_rgbx, mix_Mmcx);",
    "if (mix_Mmcx.b == 0.0) {",
    "return half3(EvalGainCurve(M.r, curve_texcoord_y, curve_N_cp));",
    "}",
    "return half3(EvalGainCurve(M.r, curve_texcoord_y, curve_N_cp),",
    "EvalGainCurve(M.g, curve_texcoord_y, curve_N_cp),",
    "EvalGainCurve(M.b, curve_texcoord_y, curve_N_cp));",
    "}",
    "half4 main(half4 color) {",
    "color.rgb *= scale_factor;",
    "if (weight_i > 0.0) {",
    "float a_inv = (color.a == 0.0) ? 1.0 : 1.0 / color.a;",
    "half3 G = half3(0.0);",
    "G += weight_i * EvalColorGainFunction(color.rgb * a_inv,",
    "mix_rgbx_i, mix_Mmcx_i,",
    "curve_texcoord_y_i, curve_N_cp_i);",
    "if (weight_j > 0.0) {",
    "G += weight_j * EvalColorGainFunction(color.rgb * a_inv,",
    "mix_rgbx_j, mix_Mmcx_j,",
    "curve_texcoord_y_j, curve_N_cp_j);",
    "}",
    "color.rgb *= exp2(G);",
    "}",
    "return color;",
    "}"
);
// Source: src/codec/SkHdrAgtm.cpp:132
fn agtm_runtime_effect() -> Option<Arc<crate::src::core::SkRuntimeEffect::SkRuntimeEffect>> {
    use crate::src::core::SkRuntimeEffect::SkRuntimeEffect;
    static effect: std::sync::OnceLock<Option<Arc<SkRuntimeEffect>>> = std::sync::OnceLock::new();
    effect.get_or_init(||{
        let mut result=SkRuntimeEffect::MakeForColorFilterDefault(gAgtmSKSL.as_bytes().to_vec());
        result.errorText.push(0);
        crate::SkASSERTF!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),135;@condition b"result.effect\0".as_ptr();result.effect.is_some(),"Agtm shader log:\n%s\n",result.errorText.as_ptr());
        result.effect
    }).clone()
}
// Source: src/codec/SkHdrAgtm.cpp:146
pub fn EvaluateComponentMixingFunction(mix: &ComponentMixingFunction, c: &SkColor4f) -> SkColor4f {
    // Preserve native Formula (9), including its min(max(R,G),B) expression.
    let common = mix.fRed * c.fR
        + mix.fGreen * c.fG
        + mix.fBlue * c.fB
        + mix.fMax * max(max(c.fR, c.fG), c.fB)
        + mix.fMin * min(max(c.fR, c.fG), c.fB);
    if mix.fComponent == 0.0 {
        return SkColor4f {
            fR: common,
            fG: common,
            fB: common,
            fA: c.fA,
        };
    }
    SkColor4f {
        fR: mix.fComponent * c.fR + common,
        fG: mix.fComponent * c.fG + common,
        fB: mix.fComponent * c.fB + common,
        fA: c.fA,
    }
}
// Source: src/codec/SkHdrAgtm.cpp:167
pub fn EvaluateGainCurve(gainCurve: &GainCurve, x: f32) -> f32 {
    let cp = &gainCurve.fControlPoints;
    let N = cp.len();
    crate::SkASSERT!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),172;@condition b"N > 0 && N <= 32\0".as_ptr();N>0&&N<=32);
    let mut i = 0;
    if x <= cp[i].fX {
        return cp[i].fY;
    }
    let mut j = N - 1;
    if x >= cp[j].fX {
        return cp[j].fY + unsafe { log2f(cp[j].fX / x) };
    }
    while j - i > 1 {
        let m = (i + j) / 2;
        if x < cp[m].fX {
            j = m;
        } else {
            i = m;
        }
    }
    let x_i = cp[i].fX;
    let y_i = cp[i].fY;
    let x_j = cp[j].fX;
    let y_j = cp[j].fY;
    let h_i = x_j - x_i;
    let mHat_i = cp[i].fM * h_i;
    let mHat_j = cp[j].fM * h_i;
    if h_i == 0.0 {
        return y_i;
    }
    let c3 = 2.0 * y_i + mHat_i - 2.0 * y_j + mHat_j;
    let c2 = -3.0 * y_i + 3.0 * y_j - 2.0 * mHat_i - mHat_j;
    let c1 = mHat_i;
    let c0 = y_i;
    let t = (x - x_i) / h_i;
    ((c3 * t + c2) * t + c1) * t + c0
}
// Source: src/codec/SkHdrAgtm.cpp:220
pub fn EvaluateColorGainFunction(gain: &ColorGainFunction, c: &SkColor4f) -> SkColor4f {
    let m = EvaluateComponentMixingFunction(&gain.fComponentMixing, c);
    let mut result = SkColor4f {
        fR: 0.0,
        fG: 0.0,
        fB: 0.0,
        fA: c.fA,
    };
    result.fR = EvaluateGainCurve(&gain.fGainCurve, m.fR);
    if m.fR == m.fG && m.fG == m.fB {
        result.fG = result.fR;
        result.fB = result.fR;
    } else {
        result.fG = EvaluateGainCurve(&gain.fGainCurve, m.fG);
        result.fB = EvaluateGainCurve(&gain.fGainCurve, m.fB);
    }
    result
}
// Source: src/codec/SkHdrAgtm.cpp:235
pub fn PopulateSlopeFromPCHIP(gainCurve: &mut GainCurve) {
    let cp = &mut gainCurve.fControlPoints;
    crate::SkASSERT_RELEASE!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),237;@condition b"cp.size() >= AdaptiveGlobalToneMap::GainCurve::kMinNumControlPoints && cp.size() <= AdaptiveGlobalToneMap::GainCurve::kMaxNumControlPoints\0".as_ptr();cp.len()>=GainCurve::kMinNumControlPoints&&cp.len()<=GainCurve::kMaxNumControlPoints);
    let N = cp.len();
    let mut h = [0.0f32; 32];
    let mut s = [0.0f32; 32];
    for i in 0..N - 1 {
        h[i] = cp[i + 1].fX - cp[i].fX;
    }
    for i in 0..N - 1 {
        s[i] = if h[i] != 0.0 {
            (cp[i + 1].fY - cp[i].fY) / h[i]
        } else {
            0.0
        };
    }
    fn three_point_finite_difference(h0: f32, h1: f32, s0: f32, s1: f32) -> f32 {
        let num = (2.0 * h0 + h1) * s0 - h0 * s1;
        let den = h0 + h1;
        let mut m = num / den;
        if m.is_sign_negative() != s0.is_sign_negative() {
            m = 0.0;
        } else if m.is_sign_negative() != s1.is_sign_negative() && m.abs() > 3.0 * s0.abs() {
            m = 3.0 * s0;
        }
        m
    }
    for i in 0..N {
        if i >= 1 && i + 1 < N && cp[i - 1].fX < cp[i].fX && cp[i].fX < cp[i + 1].fX {
            if s[i - 1].is_sign_negative() != s[i].is_sign_negative() {
                cp[i].fM = 0.0;
            } else {
                let num = 3.0 * (h[i - 1] + h[i]) * s[i - 1] * s[i];
                let den = (2.0 * h[i - 1] + h[i]) * s[i - 1] + (h[i - 1] + 2.0 * h[i]) * s[i];
                cp[i].fM = num / den;
            }
        } else if i + 1 < N && cp[i].fX < cp[i + 1].fX {
            cp[i].fM = if i + 2 < N && cp[i + 1].fX < cp[i + 2].fX {
                three_point_finite_difference(h[i], h[i + 1], s[i], s[i + 1])
            } else {
                s[i]
            };
        } else if i >= 1 && cp[i - 1].fX < cp[i].fX {
            cp[i].fM = if i >= 2 && cp[i - 2].fX < cp[i - 1].fX {
                three_point_finite_difference(h[i - 1], h[i - 2], s[i - 1], s[i - 2])
            } else {
                s[i - 1]
            };
        } else {
            cp[i].fM = 0.0;
        }
    }
}
// Source: src/codec/SkHdrAgtm.cpp:304
pub unsafe fn MakeGainCurveXYMImage(
    hatm: &HeadroomAdaptiveToneMap,
) -> Result<Option<Arc<dyn crate::src::image::SkImage::SkImage>>, crate::RasterError> {
    use crate::include::core::{SkAlphaType::SkAlphaType, SkColorType::SkColorType};
    use crate::src::core::{SkBitmap::SkBitmap, SkImageInfo::SkImageInfo};
    if hatm.fAlternateImages.is_empty() {
        return Ok(None);
    }
    let mut maxNumControlPoints = 1;
    for alt in &hatm.fAlternateImages {
        maxNumControlPoints =
            maxNumControlPoints.max(alt.fColorGainFunction.fGainCurve.fControlPoints.len());
    }
    let mut bm32 = SkBitmap::default();
    bm32.allocPixels(&SkImageInfo::Make(
        maxNumControlPoints as i32,
        hatm.fAlternateImages.len() as i32,
        SkColorType::kRGBA_F32_SkColorType,
        SkAlphaType::kPremul_SkAlphaType,
    ));
    for (a, alt) in hatm.fAlternateImages.iter().enumerate() {
        for (c, cp) in alt
            .fColorGainFunction
            .fGainCurve
            .fControlPoints
            .iter()
            .enumerate()
        {
            let xymX = bm32.getAddr(c as i32, a as i32).cast::<f32>();
            unsafe {
                *xymX = cp.fX;
                *xymX.add(1) = cp.fY;
                *xymX.add(2) = cp.fM;
                *xymX.add(3) = 1.0;
            }
        }
    }
    let mut bm16 = SkBitmap::default();
    bm16.allocPixels(
        &bm32
            .info()
            .makeColorType(SkColorType::kRGBA_F16_SkColorType),
    );
    if !unsafe { bm32.readPixels(bm16.pixmap(), 0, 0) }? {
        return Ok(None);
    }
    bm16.setImmutable();
    Ok(unsafe { crate::src::image::SkImage_RasterFactories::RasterFromBitmap(&bm16) })
}
// Source: src/codec/SkHdrAgtm.cpp:341
pub fn PopulateUsingRwtmo(hatm: &mut HeadroomAdaptiveToneMap) {
    hatm.fGainApplicationSpacePrimaries = SkNamedPrimaries::kRec2020;
    if hatm.fBaselineHdrHeadroom == 0.0 {
        hatm.fAlternateImages.clear();
        return;
    }
    hatm.fAlternateImages.resize_with(2, Default::default);
    hatm.fAlternateImages[0].fHdrHeadroom = 0.0;
    hatm.fAlternateImages[1].fHdrHeadroom = unsafe { log2f(8.0 / 3.0) }
        * min(
            hatm.fBaselineHdrHeadroom / unsafe { log2f(1000.0 / 203.0) },
            1.0,
        );
    for a in 0..hatm.fAlternateImages.len() {
        let headroom = hatm.fAlternateImages[a].fHdrHeadroom;
        let gain = &mut hatm.fAlternateImages[a].fColorGainFunction;
        *gain = ColorGainFunction::default();
        gain.fComponentMixing.fMax = 1.0;
        let yWhite = if a == 1 {
            1.0
        } else {
            1.0 - 0.5
                * min(
                    hatm.fBaselineHdrHeadroom / unsafe { log2f(1000.0 / 203.0) },
                    1.0,
                )
        };
        let kappa = 0.65;
        let xKnee = 1.0;
        let yKnee = yWhite;
        let xMax = unsafe { exp2f(hatm.fBaselineHdrHeadroom) };
        let yMax = unsafe { exp2f(headroom) };
        let xMid = (1.0 - kappa) * xKnee + kappa * (xKnee * yMax / yKnee);
        let yMid = (1.0 - kappa) * yKnee + kappa * yMax;
        let xA = xKnee - 2.0 * xMid + xMax;
        let yA = yKnee - 2.0 * yMid + yMax;
        let xB = 2.0 * xMid - 2.0 * xKnee;
        let yB = 2.0 * yMid - 2.0 * yKnee;
        let xC = xKnee;
        let yC = yKnee;
        let cubic = &mut gain.fGainCurve;
        cubic.fControlPoints.resize(8, ControlPoint::default());
        for c in 0..cubic.fControlPoints.len() {
            let t = c as f32 / (cubic.fControlPoints.len() as f32 - 1.0);
            let x = xC + t * (xB + t * xA);
            let y = yC + t * (yB + t * yA);
            let m = (2.0 * yA * t + yB) / (2.0 * xA * t + xB);
            cubic.fControlPoints[c].fX = x;
            cubic.fControlPoints[c].fY = unsafe { log2f(y / x) };
            cubic.fControlPoints[c].fM = (x * m - y) / (unsafe { logf(2.0) } * x * y);
        }
    }
}
// Source: src/codec/SkHdrAgtm.cpp:401
pub fn ComputeWeighting(hatm: &HeadroomAdaptiveToneMap, targetedHdrHeadroom: f32) -> Weighting {
    let mut result = Weighting::default();
    let mut N = 0;
    let mut H = [0.0f32; 5];
    let mut indices = [0u8; 5];
    for i in 0..hatm.fAlternateImages.len() {
        if N == i && hatm.fBaselineHdrHeadroom < hatm.fAlternateImages[i].fHdrHeadroom {
            indices[N] = Weighting::kInvalidIndex;
            H[N] = hatm.fBaselineHdrHeadroom;
            N += 1;
        }
        indices[N] = i as u8;
        H[N] = hatm.fAlternateImages[i].fHdrHeadroom;
        N += 1;
    }
    if N == hatm.fAlternateImages.len() {
        indices[N] = Weighting::kInvalidIndex;
        H[N] = hatm.fBaselineHdrHeadroom;
        N += 1;
    }
    if targetedHdrHeadroom <= H[0] {
        result.fWeight[0] = 1.0;
        result.fAlternateImageIndex[0] = indices[0];
    } else if targetedHdrHeadroom >= H[N - 1] {
        result.fWeight[0] = 1.0;
        result.fAlternateImageIndex[0] = indices[N - 1];
    } else {
        let mut i = 0;
        while i < N - 1 {
            if H[i] <= targetedHdrHeadroom && targetedHdrHeadroom <= H[i + 1] {
                break;
            }
            i += 1;
        }
        result.fWeight[0] = (targetedHdrHeadroom - H[i + 1]) / (H[i] - H[i + 1]);
        result.fWeight[1] = 1.0 - result.fWeight[0];
        result.fAlternateImageIndex[0] = indices[i];
        result.fAlternateImageIndex[1] = indices[i + 1];
    }
    for i in 0..2 {
        if result.fAlternateImageIndex[i] == Weighting::kInvalidIndex {
            result.fWeight[i] = 0.0;
        } else if result.fWeight[i] == 0.0 {
            result.fAlternateImageIndex[i] = Weighting::kInvalidIndex;
        }
    }
    if result.fWeight[1] > result.fWeight[0] {
        result.fWeight.swap(0, 1);
        result.fAlternateImageIndex.swap(0, 1);
    }
    result
}
// Source: src/codec/SkHdrAgtm.cpp:473
pub fn ApplyGain(
    hatm: &HeadroomAdaptiveToneMap,
    colors: &mut [SkColor4f],
    targetedHdrHeadroom: f32,
) {
    crate::SkASSERT!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),476;@condition b"Validate(hatm)\0".as_ptr();ValidateHeadroom(hatm));
    let weighting = ComputeWeighting(hatm, targetedHdrHeadroom);
    if weighting.fWeight[0] == 0.0 {
        return;
    } else if weighting.fWeight[1] == 0.0 {
        let gain =
            &hatm.fAlternateImages[weighting.fAlternateImageIndex[0] as usize].fColorGainFunction;
        let w = weighting.fWeight[0];
        for C in colors {
            let G = EvaluateColorGainFunction(gain, C);
            *C = SkColor4f {
                fR: C.fR * unsafe { exp2f(w * G.fR) },
                fG: C.fG * unsafe { exp2f(w * G.fG) },
                fB: C.fB * unsafe { exp2f(w * G.fB) },
                fA: C.fA,
            };
        }
    } else {
        let gain0 =
            &hatm.fAlternateImages[weighting.fAlternateImageIndex[0] as usize].fColorGainFunction;
        let w0 = weighting.fWeight[0];
        let gain1 =
            &hatm.fAlternateImages[weighting.fAlternateImageIndex[1] as usize].fColorGainFunction;
        let w1 = weighting.fWeight[1];
        for C in colors {
            let G0 = EvaluateColorGainFunction(gain0, C);
            let G1 = EvaluateColorGainFunction(gain1, C);
            *C = SkColor4f {
                fR: C.fR * unsafe { exp2f(w0 * G0.fR + w1 * G1.fR) },
                fG: C.fG * unsafe { exp2f(w0 * G0.fG + w1 * G1.fG) },
                fB: C.fB * unsafe { exp2f(w0 * G0.fB + w1 * G1.fB) },
                fA: C.fA,
            };
        }
    }
}
// Source: src/codec/SkHdrAgtm.cpp:520
pub fn GetGainApplicationSpace(hatm: &HeadroomAdaptiveToneMap) -> Option<Arc<SkColorSpace>> {
    let mut toXYZD50 = crate::modules::skcms::src::skcms_public::skcms_Matrix3x3 {
        vals: [[0.0; 3]; 3],
    };
    if !hatm.fGainApplicationSpacePrimaries.toXYZD50(&mut toXYZD50) {
        return None;
    }
    SkColorSpace::MakeRGB(&SkNamedTransferFn::kLinear, &toXYZD50)
}

/// Safety: performs native pixel writes and runtime uniform writes under their
/// original exclusive local-owner leases; errors from pixel conversion propagate.
// Source: src/codec/SkHdrAgtm.cpp:529
pub unsafe fn MakeColorFilter(
    hatm: &HeadroomAdaptiveToneMap,
    targetedHdrHeadroom: f32,
    scaleFactor: f32,
) -> Result<Option<Arc<dyn SkColorFilter>>, crate::RasterError> {
    use crate::src::core::SkRuntimeEffect::{ChildPtr, SkRuntimeShaderBuilder};
    let weighting = ComputeWeighting(hatm, targetedHdrHeadroom);
    let Some(effect) = agtm_runtime_effect() else {
        return Ok(None);
    };
    let mut builder = SkRuntimeShaderBuilder::new(effect);
    unsafe {
        builder.uniform(b"scale_factor").assign(&scaleFactor);
    }
    for a in 0..2 {
        let weight_str: [&[u8]; 2] = [b"weight_i", b"weight_j"];
        unsafe {
            builder.uniform(weight_str[a]).assign(&weighting.fWeight[a]);
        }
        if weighting.fWeight[a] == 0.0 {
            continue;
        }
        let gain =
            &hatm.fAlternateImages[weighting.fAlternateImageIndex[a] as usize].fColorGainFunction;
        let mix_rgbx_str: [&[u8]; 2] = [b"mix_rgbx_i", b"mix_rgbx_j"];
        unsafe {
            builder.uniform(mix_rgbx_str[a]).assign(&SkColor4f {
                fR: gain.fComponentMixing.fRed,
                fG: gain.fComponentMixing.fGreen,
                fB: gain.fComponentMixing.fBlue,
                fA: 0.0,
            });
        }
        let mix_Mmcx_str: [&[u8]; 2] = [b"mix_Mmcx_i", b"mix_Mmcx_j"];
        unsafe {
            builder.uniform(mix_Mmcx_str[a]).assign(&SkColor4f {
                fR: gain.fComponentMixing.fMax,
                fG: gain.fComponentMixing.fMin,
                fB: gain.fComponentMixing.fComponent,
                fA: 0.0,
            });
        }
        let curve_texcoord_y_str: [&[u8]; 2] = [b"curve_texcoord_y_i", b"curve_texcoord_y_j"];
        unsafe {
            builder
                .uniform(curve_texcoord_y_str[a])
                .assign(&(weighting.fAlternateImageIndex[a] as f32 + 0.5));
        }
        let curve_N_cp_str: [&[u8]; 2] = [b"curve_N_cp_i", b"curve_N_cp_j"];
        unsafe {
            builder
                .uniform(curve_N_cp_str[a])
                .assign(&(gain.fGainCurve.fControlPoints.len() as f32));
        }
    }
    if let Some(gainCurvesXYM) = unsafe { MakeGainCurveXYMImage(hatm) }? {
        use crate::include::core::SkSamplingOptions::{SkFilterMode, SkSamplingOptions};
        let child = gainCurvesXYM
            .makeRawShader_default(&SkSamplingOptions::with_filter(SkFilterMode::kNearest));
        if let Some(child) = child {
            builder.child(b"curve_xym").assign(ChildPtr::Shader(child));
        } else {
            builder.child(b"curve_xym").assign_null();
        }
    }
    let Some(gainApplicationColorSpace) = GetGainApplicationSpace(hatm) else {
        return Ok(None);
    };
    let filter = builder.makeColorFilter();
    crate::SkASSERT!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),586;@condition b"filter\0".as_ptr();filter.is_some());
    // Native dereference requires the actual shader compiler to have produced a filter.
    let filter = unsafe { filter.unwrap_unchecked() };
    Ok(filter.makeWithWorkingColorSpace(Some(gainApplicationColorSpace)))
}
// Source: src/codec/SkHdrAgtm.cpp:591
fn get_max_luminance(metadata: &Metadata) -> f32 {
    if unsafe { metadata.getContentLightLevelInformation(core::ptr::null_mut()) } {
        let mut clli = ContentLightLevelInformation::default();
        if unsafe { metadata.getContentLightLevelInformation(&mut clli) } && clli.fMaxCLL > 0.0 {
            return clli.fMaxCLL;
        }
    }
    if unsafe { metadata.getMasteringDisplayColorVolume(core::ptr::null_mut()) } {
        let mut mdcv = MasteringDisplayColorVolume::default();
        if unsafe { metadata.getMasteringDisplayColorVolume(&mut mdcv) }
            && mdcv.fMaximumDisplayMasteringLuminance > 0.0
        {
            return mdcv.fMaximumDisplayMasteringLuminance;
        }
    }
    1000.0
}
// Source: src/codec/SkHdrAgtm.cpp:608
pub fn PopulateToneMapAgtmParams(
    metadata: &Metadata,
    inputColorSpace: Option<&SkColorSpace>,
    outAgtm: Option<&mut AdaptiveGlobalToneMap>,
    outScaleFactor: Option<&mut f32>,
) -> bool {
    use crate::modules::skcms::{
        skcms::skcms_TransferFunction_getType,
        src::skcms_public::{skcms_TFType, skcms_TransferFunction},
    };
    let mut inputIsPqOrHlg = false;
    let mut inputPqOrHlgWhite = AdaptiveGlobalToneMap::kDefaultHdrReferenceWhite;
    if let Some(inputColorSpace) = inputColorSpace {
        let mut trfn = SkNamedTransferFn::kLinear;
        inputColorSpace.transferFn(&mut trfn);
        match skcms_TransferFunction_getType(&trfn) {
            skcms_TFType::skcms_TFType_PQ | skcms_TFType::skcms_TFType_HLG => {
                inputIsPqOrHlg = true;
                inputPqOrHlgWhite = trfn.a;
            }
            _ => {}
        }
    }
    let mut agtm = AdaptiveGlobalToneMap::default();
    let hadAgtmMetadata = unsafe { metadata.getAdaptiveGlobalToneMap(&mut agtm) };
    if !inputIsPqOrHlg {
        if !hadAgtmMetadata || agtm.fHeadroomAdaptiveToneMap.is_none() {
            return false;
        }
    }
    if !hadAgtmMetadata {
        agtm.fHdrReferenceWhite = inputPqOrHlgWhite;
    }
    if agtm.fHeadroomAdaptiveToneMap.is_none() {
        let mut hatm = HeadroomAdaptiveToneMap::default();
        hatm.fBaselineHdrHeadroom = unsafe {
            log2f(max(
                get_max_luminance(metadata) / agtm.fHdrReferenceWhite,
                1.0,
            ))
        };
        PopulateUsingRwtmo(&mut hatm);
        agtm.fHeadroomAdaptiveToneMap = Some(hatm);
    }
    if let Some(outAgtm) = outAgtm {
        outAgtm.clone_from(&agtm);
    }
    if let Some(outScaleFactor) = outScaleFactor {
        *outScaleFactor = if inputIsPqOrHlg {
            inputPqOrHlgWhite / agtm.fHdrReferenceWhite
        } else {
            1.0
        };
    }
    true
}
// Source: src/codec/SkHdrAgtm.cpp:668
pub fn Validate(agtm: &AdaptiveGlobalToneMap) -> bool {
    if agtm.fHdrReferenceWhite < 0.0 || agtm.fHdrReferenceWhite > 10000.0 {
        crate::SkCodecPrintf!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),670;b"Agtm validation failed: HdrReferenceWhite invalid\n\0".as_ptr());
        return false;
    }
    if let Some(hatm) = &agtm.fHeadroomAdaptiveToneMap {
        if !ValidateHeadroom(hatm) {
            return false;
        }
    }
    true
}
// Source: src/codec/SkHdrAgtm.cpp:681
pub fn ValidateHeadroom(hatm: &HeadroomAdaptiveToneMap) -> bool {
    if hatm.fBaselineHdrHeadroom < kMinHdrHeadroom || hatm.fBaselineHdrHeadroom > kMaxHdrHeadroom {
        crate::SkCodecPrintf!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),687;b"Agtm validation failed: H_baseline invalid\n\0".as_ptr());
        return false;
    }
    if hatm.fAlternateImages.len() > HeadroomAdaptiveToneMap::kMaxNumAlternateImages {
        crate::SkCodecPrintf!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),693;b"Agtm validation failed: NumAlternateImages invalid\n\0".as_ptr());
        return false;
    }
    for alt in &hatm.fAlternateImages {
        if alt.fHdrHeadroom == hatm.fBaselineHdrHeadroom {
            crate::SkCodecPrintf!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),699;b"Agtm validation failed: Alternate headroom equal to baseline\n\0".as_ptr());
            return false;
        }
    }
    for a in 1..hatm.fAlternateImages.len() {
        if hatm.fAlternateImages[a].fHdrHeadroom <= hatm.fAlternateImages[a - 1].fHdrHeadroom {
            crate::SkCodecPrintf!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),707;b"Agtm validation failed: Alternate headrooms not ascending\n\0".as_ptr());
            return false;
        }
    }
    let p = &hatm.fGainApplicationSpacePrimaries;
    if !in_unit_interval(p.fRX)
        || !in_unit_interval(p.fRY)
        || !in_unit_interval(p.fGX)
        || !in_unit_interval(p.fGY)
        || !in_unit_interval(p.fBX)
        || !in_unit_interval(p.fBY)
        || !in_unit_interval(p.fWX)
        || !in_unit_interval(p.fWY)
    {
        crate::SkCodecPrintf!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),720;b"Agtm validation failed: chromaticities not in unit interval\n\0".as_ptr());
        return false;
    }
    for alt in &hatm.fAlternateImages {
        for cp in &alt.fColorGainFunction.fGainCurve.fControlPoints {
            if alt.fHdrHeadroom > hatm.fBaselineHdrHeadroom {
                if cp.fY < 0.0 {
                    crate::SkCodecPrintf!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),734;b"Agtm validation failed: negative Y value in inverse tone map\n\0".as_ptr());
                    return false;
                }
            } else if cp.fY > 0.0 {
                crate::SkCodecPrintf!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),740;b"Agtm validation failed: positive Y value in tone map\n\0".as_ptr());
                return false;
            }
        }
    }
    for alt in &hatm.fAlternateImages {
        let mix = &alt.fColorGainFunction.fComponentMixing;
        if !in_unit_interval(mix.fRed)
            || !in_unit_interval(mix.fGreen)
            || !in_unit_interval(mix.fBlue)
            || !in_unit_interval(mix.fMax)
            || !in_unit_interval(mix.fMin)
            || !in_unit_interval(mix.fComponent)
        {
            crate::SkCodecPrintf!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),754;b"Agtm validation failed: mix coefficients not in unit interval\n\0".as_ptr());
            return false;
        }
        let sum = mix.fRed + mix.fGreen + mix.fBlue + mix.fMax + mix.fMin + mix.fComponent;
        if sum < 0.99999 || sum > 1.00001 {
            crate::SkCodecPrintf!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),761;b"Agtm validation failed: mix coefficients don't sum to 1\n\0".as_ptr());
            return false;
        }
    }
    for alt in &hatm.fAlternateImages {
        let curve = &alt.fColorGainFunction.fGainCurve;
        if curve.fControlPoints.len() < GainCurve::kMinNumControlPoints
            || curve.fControlPoints.len() > GainCurve::kMaxNumControlPoints
        {
            crate::SkCodecPrintf!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),772;b"Agtm validation failed: invalid number of control points\n\0".as_ptr());
            return false;
        }
        for c in 0..curve.fControlPoints.len() {
            if curve.fControlPoints[c].fX < 0.0
                || curve.fControlPoints[c].fX > kMaxLinearHdrHeadroom
            {
                crate::SkCodecPrintf!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),779;b"Agtm validation failed: invalid X value\n\0".as_ptr());
                return false;
            }
            if curve.fControlPoints[c].fY < -kMaxHdrHeadroom
                || curve.fControlPoints[c].fY > kMaxHdrHeadroom
            {
                crate::SkCodecPrintf!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),785;b"Agtm validation failed: invalid Y value\n\0".as_ptr());
                return false;
            }
            if c > 0 {
                if curve.fControlPoints[c].fX < curve.fControlPoints[c - 1].fX {
                    crate::SkCodecPrintf!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),791;b"Agtm validation failed: curve X is not non-decreasing\n\0".as_ptr());
                    return false;
                }
                if curve.fControlPoints[c].fX == curve.fControlPoints[c - 1].fX
                    && curve.fControlPoints[c].fY != curve.fControlPoints[c - 1].fY
                {
                    crate::SkCodecPrintf!(@source b"src/codec/SkHdrAgtm.cpp\0".as_ptr(),797;b"Agtm validation failed: curve has Y discontinuity\n\0".as_ptr());
                    return false;
                }
            }
        }
    }
    true
}
impl AdaptiveGlobalToneMap {
    // Source: src/codec/SkHdrAgtm.cpp:808
    pub fn toString(&self) -> crate::src::core::SkString::SkString {
        use crate::src::core::SkString::SkStringPrintf;
        let mut result = unsafe {
            SkStringPrintf(
                b"{hdrReferenceWhite:%f\0".as_ptr(),
                self.fHdrReferenceWhite as f64,
            )
        };
        let Some(hatm) = &self.fHeadroomAdaptiveToneMap else {
            result.append_view(b"}");
            return result;
        };
        let headroom = unsafe {
            SkStringPrintf(
                b", baselineHdrHeadroom:%f\0".as_ptr(),
                hatm.fBaselineHdrHeadroom as f64,
            )
        };
        result.append_string(&headroom);
        result.append_view(b", alternateHdrHeadrooms:[");
        for a in 0..hatm.fAlternateImages.len() {
            let value = unsafe {
                SkStringPrintf(
                    b"%f\0".as_ptr(),
                    hatm.fAlternateImages[a].fHdrHeadroom as f64,
                )
            };
            result.append_string(&value);
            if a != hatm.fAlternateImages.len() - 1 {
                result.append_view(b", ");
            }
        }
        result.append_view(b"]}");
        result
    }
    // Source: src/codec/SkHdrAgtm.cpp:828
    pub fn isValid(&self) -> bool {
        Validate(self)
    }
}
