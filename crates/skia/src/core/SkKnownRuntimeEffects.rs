// Copyright 2024 Google LLC. BSD-style license.
// Source: src/core/SkKnownRuntimeEffects.h:27-39
pub const kSkiaBuiltInReservedCnt: i32 = 500;
pub const kSkiaKnownRuntimeEffectsReservedCnt: i32 = 500;
pub const kUserDefinedKnownRuntimeEffectsReservedCnt: i32 = 100;
pub const kSkiaKnownRuntimeEffectsStart: i32 = kSkiaBuiltInReservedCnt;
pub const kSkiaKnownRuntimeEffectsEnd: i32 =
    kSkiaKnownRuntimeEffectsStart + kSkiaKnownRuntimeEffectsReservedCnt;
pub const kUserDefinedKnownRuntimeEffectsStart: i32 = kSkiaKnownRuntimeEffectsEnd;
pub const kUserDefinedKnownRuntimeEffectsEnd: i32 =
    kUserDefinedKnownRuntimeEffectsStart + kUserDefinedKnownRuntimeEffectsReservedCnt;
pub const kUnknownRuntimeEffectIDStart: i32 = kUserDefinedKnownRuntimeEffectsEnd;
// Source: src/core/SkKnownRuntimeEffects.h:49-80
impl StableKey {
    pub const kInvalid: Self = Self::kStart;
    pub const k1DBlurBase: Self = Self(501);
    pub const k1DBlur4: Self = Self::k1DBlurBase;
    pub const k1DBlur8: Self = Self(502);
    pub const k1DBlur12: Self = Self(503);
    pub const k1DBlur16: Self = Self(504);
    pub const k1DBlur20: Self = Self(505);
    pub const k1DBlur28: Self = Self(506);
    pub const k2DBlurBase: Self = Self(507);
    pub const k2DBlur4: Self = Self::k2DBlurBase;
    pub const k2DBlur8: Self = Self(508);
    pub const k2DBlur12: Self = Self(509);
    pub const k2DBlur16: Self = Self(510);
    pub const k2DBlur20: Self = Self(511);
    pub const k2DBlur28: Self = Self(512);
    pub const kBlend: Self = Self(513);
    pub const kDecal: Self = Self(514);
    pub const kDisplacement: Self = Self(515);
    pub const kLighting: Self = Self(516);
    pub const kLinearMorphology: Self = Self(517);
    pub const kMagnifier: Self = Self(518);
    pub const kMatrixConvUniforms: Self = Self(519);
    pub const kMatrixConvTexSm: Self = Self(520);
    pub const kMatrixConvTexLg: Self = Self(521);
    pub const kNormal: Self = Self(522);
    pub const kSparseMorphology: Self = Self(523);
    pub const kArithmetic: Self = Self(524);
    pub const kHighContrast: Self = Self(525);
    pub const kLerp: Self = Self(526);
    pub const kLuma: Self = Self(527);
    pub const kOverdraw: Self = Self(528);
}
// Source: src/core/SkKnownRuntimeEffects.h:85-97
// Adapted: Rust forbids duplicate enum discriminants. A transparent uint32 value
// retains every native alias and arbitrary uint32 cast identity; associated
// constants above stay in the exact native macro expansion order.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StableKey(pub u32);
impl StableKey {
    pub const kStart: Self = Self(kSkiaKnownRuntimeEffectsStart as u32);
    pub const kLast: Self = Self::kOverdraw;
}
// Source: src/core/SkKnownRuntimeEffects.h:99-100
pub const kStableKeyCnt: i32 = StableKey::kLast.0 as i32 - StableKey::kStart.0 as i32 + 1;
// Source: src/core/SkKnownRuntimeEffects.h:104-106
const _: () = {
    assert!(StableKey::kStart.0 as u32 != 0);
    assert!((StableKey::kLast.0 as i32) < kSkiaKnownRuntimeEffectsEnd);
};
// Source: src/core/SkKnownRuntimeEffects.h:123-137
const _: () = {
    assert!(StableKey::kInvalid.0 as u32 == StableKey::kStart.0 as u32);
    assert!(StableKey::k1DBlur4.0 as u32 == StableKey::k1DBlurBase.0 as u32 + 0);
    assert!(StableKey::k1DBlur8.0 as u32 == StableKey::k1DBlurBase.0 as u32 + 1);
    assert!(StableKey::k1DBlur12.0 as u32 == StableKey::k1DBlurBase.0 as u32 + 2);
    assert!(StableKey::k1DBlur16.0 as u32 == StableKey::k1DBlurBase.0 as u32 + 3);
    assert!(StableKey::k1DBlur20.0 as u32 == StableKey::k1DBlurBase.0 as u32 + 4);
    assert!(StableKey::k1DBlur28.0 as u32 == StableKey::k1DBlurBase.0 as u32 + 5);
    assert!(StableKey::k2DBlur4.0 as u32 == StableKey::k2DBlurBase.0 as u32 + 0);
    assert!(StableKey::k2DBlur8.0 as u32 == StableKey::k2DBlurBase.0 as u32 + 1);
    assert!(StableKey::k2DBlur12.0 as u32 == StableKey::k2DBlurBase.0 as u32 + 2);
    assert!(StableKey::k2DBlur16.0 as u32 == StableKey::k2DBlurBase.0 as u32 + 3);
    assert!(StableKey::k2DBlur20.0 as u32 == StableKey::k2DBlurBase.0 as u32 + 4);
    assert!(StableKey::k2DBlur28.0 as u32 == StableKey::k2DBlurBase.0 as u32 + 5);
};
use super::SkRuntimeEffect::{Options, SkRuntimeEffect};
use super::SkRuntimeEffectPriv::{self, SkMakeRuntimeEffect};
use std::sync::{Arc, OnceLock};
// Source: src/core/SkKnownRuntimeEffects.cpp:19-25
fn get_options(key: StableKey) -> Options<'static> {
    let mut options = Options::default();
    SkRuntimeEffectPriv::SetStableKeyOnOptions(&mut options, key.0);
    SkRuntimeEffectPriv::AllowPrivateAccess(&mut options);
    options
}
// Source: src/core/SkKnownRuntimeEffects.cpp:28-28
const kMaxBlurSamples: i32 = 28;
// Source: src/core/SkKnownRuntimeEffects.cpp:30-62
fn make_blur_1D_shader(kernelWidth: i32, key: StableKey) -> Arc<SkRuntimeEffect> {
    crate::SkASSERT!(@source b"src/core/SkKnownRuntimeEffects.cpp\0".as_ptr(),33; @condition b"kernelWidth <= kMaxBlurSamples\0".as_ptr(); kernelWidth <= kMaxBlurSamples);
    crate::SkASSERT!(@source b"src/core/SkKnownRuntimeEffects.cpp\0".as_ptr(),36; @condition b"kernelWidth % 2 == 0\0".as_ptr(); kernelWidth % 2 == 0);
    let source = unsafe {
        crate::src::core::SkString::SkStringPrintf(b"const int kMaxUniformKernelSize = %d / 2;const int kMaxLoopLimit = %d / 2;uniform half4 offsetsAndKernel[kMaxUniformKernelSize];uniform half2 dir;uniform shader child;half4 main(float2 coord) {half4 sum = half4(0);for (int i = 0; i < kMaxLoopLimit; ++i) {half4 s = offsetsAndKernel[i];sum += s.y * child.eval(coord + s.x*dir);sum += s.w * child.eval(coord + s.z*dir);}return sum;}\0".as_ptr(),kMaxBlurSamples as core::ffi::c_int,kernelWidth as core::ffi::c_int)
    };
    SkMakeRuntimeEffect(
        SkRuntimeEffect::MakeForShader,
        unsafe { core::slice::from_raw_parts(source.data(), source.size()) },
        get_options(key),
    )
}
// Source: src/core/SkKnownRuntimeEffects.cpp:64-101
fn make_blur_2D_shader(maxKernelSize: i32, key: StableKey) -> Arc<SkRuntimeEffect> {
    crate::SkASSERT!(@source b"src/core/SkKnownRuntimeEffects.cpp\0".as_ptr(),67; @condition b"maxKernelSize % 4 == 0\0".as_ptr(); maxKernelSize % 4 == 0);
    let source = unsafe {
        crate::src::core::SkString::SkStringPrintf(b"const int kMaxUniformKernelSize = %d / 4;const int kMaxUniformOffsetsSize = 2*kMaxUniformKernelSize;const int kMaxLoopLimit = %d / 4;uniform half4 kernel[kMaxUniformKernelSize];uniform half4 offsets[kMaxUniformOffsetsSize];uniform shader child;half4 main(float2 coord) {half4 sum = half4(0);for (int i = 0; i < kMaxLoopLimit; ++i) {half4 k = kernel[i];half4 o = offsets[2*i];sum += k.x * child.eval(coord + o.xy);sum += k.y * child.eval(coord + o.zw);o = offsets[2*i + 1];sum += k.z * child.eval(coord + o.xy);sum += k.w * child.eval(coord + o.zw);}return sum;}\0".as_ptr(),kMaxBlurSamples as core::ffi::c_int,maxKernelSize as core::ffi::c_int)
    };
    SkMakeRuntimeEffect(
        SkRuntimeEffect::MakeForShader,
        unsafe { core::slice::from_raw_parts(source.data(), source.size()) },
        get_options(key),
    )
}
// Source: src/core/SkKnownRuntimeEffects.cpp:103-116
fn make_blend_shader() -> Arc<SkRuntimeEffect> {
    SkMakeRuntimeEffect(SkRuntimeEffect::MakeForShader,"uniform shader s, d;uniform blender b;half4 main(float2 xy) {return b.eval(s.eval(xy), d.eval(xy));}".as_bytes(),get_options(StableKey::kBlend))
}
// Source: src/core/SkKnownRuntimeEffects.cpp:118-133
fn make_lerp_shader() -> Arc<SkRuntimeEffect> {
    SkMakeRuntimeEffect(SkRuntimeEffect::MakeForColorFilter,"uniform colorFilter cf0;uniform colorFilter cf1;uniform half weight;half4 main(half4 color) {return mix(cf0.eval(color), cf1.eval(color), weight);}".as_bytes(),get_options(StableKey::kLerp))
}
// Source: src/core/SkKnownRuntimeEffects.cpp:135-139
#[derive(Clone, Copy)]
enum MatrixConvolutionImpl {
    kUniformBased,
    kTextureBasedSm,
    kTextureBasedLg,
}
// Source: src/core/SkKnownRuntimeEffects.cpp:145-245
fn make_matrix_conv_shader(
    implementation: MatrixConvolutionImpl,
    key: StableKey,
) -> Arc<SkRuntimeEffect> {
    use crate::src::effects::imagefilters::SkMatrixConvolutionImageFilter as K;
    let header="uniform int2 size;uniform int2 offset;uniform half2 gainAndBias;uniform int convolveAlpha;uniform shader child;half4 main(float2 coord) {half4 sum = half4(0);half origAlpha = 0;int2 kernelPos = int2(0);for (int i = 0; i < kMaxKernelSize; ++i) {if (kernelPos.y >= size.y) { break; }\0";
    let accumulate="half4 c = child.eval(coord + half2(kernelPos) - half2(offset));if (convolveAlpha == 0) {if (kernelPos == offset) {origAlpha = c.a;}c = unpremul(c);}sum += c*k;kernelPos.x += 1;if (kernelPos.x >= size.x) {kernelPos.x = 0;kernelPos.y += 1;}\0";
    let footer="}half4 color = sum*gainAndBias.x + gainAndBias.y;if (convolveAlpha == 0) {color = half4(color.rgb*origAlpha, origAlpha);} else {color.a = saturate(color.a);}color.rgb = clamp(color.rgb, 0, color.a);return color;}\0";
    let source = match implementation {
        MatrixConvolutionImpl::kUniformBased => unsafe {
            crate::src::core::SkString::SkStringPrintf(b"const int kMaxKernelSize = %d / 4;uniform half4 kernel[kMaxKernelSize];%shalf4 k4 = kernel[i];for (int j = 0; j < 4; ++j) {if (kernelPos.y >= size.y) { break; }half k = k4[j];%s}%s\0".as_ptr(),K::kMaxUniformKernelSize as core::ffi::c_int,header.as_ptr(),accumulate.as_ptr(),footer.as_ptr())
        },
        MatrixConvolutionImpl::kTextureBasedSm => unsafe {
            crate::src::core::SkString::SkStringPrintf(b"const int kMaxKernelSize = %d;uniform shader kernel;uniform half2 innerGainAndBias;%shalf k = kernel.eval(half2(half(i) + 0.5, 0.5)).a;k = k * innerGainAndBias.x + innerGainAndBias.y;%s%s\0".as_ptr(),K::kSmallKernelSize as core::ffi::c_int,header.as_ptr(),accumulate.as_ptr(),footer.as_ptr())
        },
        MatrixConvolutionImpl::kTextureBasedLg => unsafe {
            crate::src::core::SkString::SkStringPrintf(b"const int kMaxKernelSize = %d;uniform shader kernel;uniform half2 innerGainAndBias;%shalf k = kernel.eval(half2(half(i) + 0.5, 0.5)).a;k = k * innerGainAndBias.x + innerGainAndBias.y;%s%s\0".as_ptr(),K::kLargeKernelSize as core::ffi::c_int,header.as_ptr(),accumulate.as_ptr(),footer.as_ptr())
        },
    };
    SkMakeRuntimeEffect(
        SkRuntimeEffect::MakeForShader,
        unsafe { core::slice::from_raw_parts(source.data(), source.size()) },
        get_options(key),
    )
}
// Source: src/core/SkKnownRuntimeEffects.cpp:247-261
fn make_decal_shader() -> Arc<SkRuntimeEffect> {
    SkMakeRuntimeEffect(SkRuntimeEffect::MakeForShader,"uniform shader image;uniform float4 decalBounds;half4 main(float2 coord) {return sk_decal(image, coord, decalBounds);}".as_bytes(),get_options(StableKey::kDecal))
}
// Source: src/core/SkKnownRuntimeEffects.cpp:263-283
fn make_displacement_shader() -> Arc<SkRuntimeEffect> {
    SkMakeRuntimeEffect(SkRuntimeEffect::MakeForShader,"uniform shader displMap;uniform shader colorMap;uniform half2 scale;uniform half4 xSelect;uniform half4 ySelect;half4 main(float2 coord) {return sk_displacement(displMap, colorMap, coord, scale, xSelect, ySelect);}".as_bytes(),get_options(StableKey::kDisplacement))
}
// Source: src/core/SkKnownRuntimeEffects.cpp:285-317
fn make_lighting_shader() -> Arc<SkRuntimeEffect> {
    SkMakeRuntimeEffect(SkRuntimeEffect::MakeForShader,"uniform shader normalMap;uniform half4 materialAndLightType;uniform half4 lightPosAndSpotFalloff;uniform half4 lightDirAndSpotCutoff;uniform half3 lightColor;half4 main(float2 coord) {return sk_lighting(normalMap, coord,materialAndLightType.x,materialAndLightType.y,materialAndLightType.z,materialAndLightType.w,lightPosAndSpotFalloff.xyz,lightPosAndSpotFalloff.w,lightDirAndSpotCutoff.xyz,lightDirAndSpotCutoff.w,lightColor);}".as_bytes(),get_options(StableKey::kLighting))
}
// Source: src/core/SkKnownRuntimeEffects.cpp:319-335
fn make_linear_morphology_shader() -> Arc<SkRuntimeEffect> {
    SkMakeRuntimeEffect(SkRuntimeEffect::MakeForShader,"uniform shader child;uniform half2 offset;uniform half flip;uniform int radius;half4 main(float2 coord) {return sk_linear_morphology(child, coord, offset, flip, radius);}".as_bytes(),get_options(StableKey::kLinearMorphology))
}
// Source: src/core/SkKnownRuntimeEffects.cpp:337-353
fn make_magnifier_shader() -> Arc<SkRuntimeEffect> {
    SkMakeRuntimeEffect(SkRuntimeEffect::MakeForShader,"uniform shader src;uniform float4 lensBounds;uniform float4 zoomXform;uniform float2 invInset;half4 main(float2 coord) {return sk_magnifier(src, coord, lensBounds, zoomXform, invInset);}".as_bytes(),get_options(StableKey::kMagnifier))
}
// Source: src/core/SkKnownRuntimeEffects.cpp:355-370
fn make_normal_shader() -> Arc<SkRuntimeEffect> {
    SkMakeRuntimeEffect(SkRuntimeEffect::MakeForShader,"uniform shader alphaMap;uniform float4 edgeBounds;uniform half negSurfaceDepth;half4 main(float2 coord) {return sk_normal(alphaMap, coord, edgeBounds, negSurfaceDepth);}".as_bytes(),get_options(StableKey::kNormal))
}
// Source: src/core/SkKnownRuntimeEffects.cpp:372-387
fn make_sparse_morphology_shader() -> Arc<SkRuntimeEffect> {
    SkMakeRuntimeEffect(SkRuntimeEffect::MakeForShader,"uniform shader child;uniform half2 offset;uniform half flip;half4 main(float2 coord) {return sk_sparse_morphology(child, coord, offset, flip);}".as_bytes(),get_options(StableKey::kSparseMorphology))
}
// Source: src/core/SkKnownRuntimeEffects.cpp:389-403
fn make_arithmetic_blender() -> Arc<SkRuntimeEffect> {
    SkMakeRuntimeEffect(SkRuntimeEffect::MakeForBlender,"uniform half4 k;uniform half pmClamp;half4 main(half4 src, half4 dst) {return sk_arithmetic_blend(src, dst, k, pmClamp);}".as_bytes(),get_options(StableKey::kArithmetic))
}
// Source: src/core/SkKnownRuntimeEffects.cpp:405-417
fn make_high_contrast_color_filter() -> Arc<SkRuntimeEffect> {
    SkMakeRuntimeEffect(SkRuntimeEffect::MakeForColorFilter,"uniform half grayscale, invertStyle, contrast;half4 main(half4 color) {return half4(sk_high_contrast(color.rgb, grayscale, invertStyle, contrast), color.a);}".as_bytes(),get_options(StableKey::kHighContrast))
}
// Source: src/core/SkKnownRuntimeEffects.cpp:419-430
fn make_luma_color_filter() -> Arc<SkRuntimeEffect> {
    SkMakeRuntimeEffect(
        SkRuntimeEffect::MakeForColorFilter,
        "half4 main(half4 color) {return sk_luma(color.rgb);}".as_bytes(),
        get_options(StableKey::kLuma),
    )
}
// Source: src/core/SkKnownRuntimeEffects.cpp:432-445
fn make_overdraw_color_filter() -> Arc<SkRuntimeEffect> {
    SkMakeRuntimeEffect(SkRuntimeEffect::MakeForColorFilter,"uniform half4 color0, color1, color2, color3, color4, color5;half4 main(half4 color) {return sk_overdraw(color.a, color0, color1, color2, color3, color4, color5);}".as_bytes(),get_options(StableKey::kOverdraw))
}

// Source: src/core/SkKnownRuntimeEffects.cpp:449-452
pub fn IsSkiaKnownRuntimeEffect(candidate: i32) -> bool {
    candidate >= StableKey::kStart.0 as i32 && candidate <= StableKey::kLast.0 as i32
}
// Source: src/core/SkKnownRuntimeEffects.cpp:454-456
pub fn IsUserDefinedRuntimeEffect(candidate: i32) -> bool {
    candidate >= kUnknownRuntimeEffectIDStart
}
// Source: src/core/SkKnownRuntimeEffects.cpp:458-461
pub fn IsViableUserDefinedKnownRuntimeEffect(candidate: i32) -> bool {
    candidate >= kUserDefinedKnownRuntimeEffectsStart
        && candidate < kUserDefinedKnownRuntimeEffectsEnd
}

// Source: src/core/SkKnownRuntimeEffects.cpp:463-472
pub fn MaybeGetKnownRuntimeEffect(candidate: u32) -> Option<Arc<SkRuntimeEffect>> {
    if IsSkiaKnownRuntimeEffect(candidate as i32) {
        GetKnownRuntimeEffect(StableKey(candidate)).cloned()
    } else {
        None
    }
}
// Source: src/core/SkKnownRuntimeEffects.cpp:474-602
pub fn GetKnownRuntimeEffect(key: StableKey) -> Option<&'static Arc<SkRuntimeEffect>> {
    match key {
        StableKey::kInvalid => None,
        StableKey::k1DBlur4 => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_blur_1D_shader(4, key)))
        }
        StableKey::k1DBlur8 => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_blur_1D_shader(8, key)))
        }
        StableKey::k1DBlur12 => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_blur_1D_shader(12, key)))
        }
        StableKey::k1DBlur16 => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_blur_1D_shader(16, key)))
        }
        StableKey::k1DBlur20 => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_blur_1D_shader(20, key)))
        }
        StableKey::k1DBlur28 => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_blur_1D_shader(28, key)))
        }
        StableKey::k2DBlur4 => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_blur_2D_shader(4, key)))
        }
        StableKey::k2DBlur8 => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_blur_2D_shader(8, key)))
        }
        StableKey::k2DBlur12 => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_blur_2D_shader(12, key)))
        }
        StableKey::k2DBlur16 => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_blur_2D_shader(16, key)))
        }
        StableKey::k2DBlur20 => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_blur_2D_shader(20, key)))
        }
        StableKey::k2DBlur28 => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_blur_2D_shader(28, key)))
        }
        StableKey::kBlend => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_blend_shader()))
        }
        StableKey::kLerp => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_lerp_shader()))
        }
        StableKey::kMatrixConvUniforms => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(
                EFFECT.get_or_init(|| {
                    make_matrix_conv_shader(MatrixConvolutionImpl::kUniformBased, key)
                }),
            )
        }
        StableKey::kMatrixConvTexSm => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| {
                make_matrix_conv_shader(MatrixConvolutionImpl::kTextureBasedSm, key)
            }))
        }
        StableKey::kMatrixConvTexLg => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| {
                make_matrix_conv_shader(MatrixConvolutionImpl::kTextureBasedLg, key)
            }))
        }
        StableKey::kDecal => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_decal_shader()))
        }
        StableKey::kDisplacement => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_displacement_shader()))
        }
        StableKey::kLighting => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_lighting_shader()))
        }
        StableKey::kLinearMorphology => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_linear_morphology_shader()))
        }
        StableKey::kMagnifier => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_magnifier_shader()))
        }
        StableKey::kNormal => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_normal_shader()))
        }
        StableKey::kSparseMorphology => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_sparse_morphology_shader()))
        }
        StableKey::kArithmetic => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_arithmetic_blender()))
        }
        StableKey::kHighContrast => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_high_contrast_color_filter()))
        }
        StableKey::kLuma => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_luma_color_filter()))
        }
        StableKey::kOverdraw => {
            static EFFECT: OnceLock<Arc<SkRuntimeEffect>> = OnceLock::new();
            Some(EFFECT.get_or_init(|| make_overdraw_color_filter()))
        }
        _ => crate::SkUNREACHABLE!(),
    }
}
