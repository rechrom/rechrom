// Copyright 2011 Google Inc. BSD-style license.
// All native registration calls retain their original order. Missing concrete
// CreateProc/registration owners remain compile-visible dependencies, never skipped.
// Source: src/ports/SkGlobalInitialization_default.cpp:11-14
#[cfg(sk_disable_effect_deserialization)]
/// Safety: called exclusively by the native initialization once before publication.
pub unsafe fn InitEffects() {
    unsafe {}
}
#[cfg(sk_disable_effect_deserialization)]
/// Safety: called exclusively by the native initialization once before publication.
pub unsafe fn InitImageFilters() {
    unsafe {}
}

// Source: src/ports/SkGlobalInitialization_default.cpp:52-101
#[cfg(not(sk_disable_effect_deserialization))]
/// Safety: called exclusively by the native initialization once before publication.
pub unsafe fn InitEffects() {
    unsafe {
        // Shaders.
        // Source: src/ports/SkGlobalInitialization_default.cpp:54-54
        crate::src::shaders::SkBlendShader::SkRegisterBlendShaderFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:55-55
        crate::src::core::SkFlattenable::Register(
            b"SkColorFilterShader",
            crate::src::shaders::SkColorFilterShader::SkColorFilterShader::CreateProc,
        );
        // Source: src/ports/SkGlobalInitialization_default.cpp:56-56
        crate::src::shaders::SkColorShader::SkRegisterColorShaderFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:57-57
        crate::src::shaders::SkCoordClampShader::SkRegisterCoordClampShaderFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:58-58
        crate::src::shaders::SkEmptyShader::SkRegisterEmptyShaderFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:59-59
        crate::src::core::SkFlattenable::Register(
            b"SkLocalMatrixShader",
            crate::src::shaders::SkLocalMatrixShader::SkLocalMatrixShader::CreateProc,
        );
        // Source: src/ports/SkGlobalInitialization_default.cpp:60-60
        crate::src::core::SkFlattenable::Register(
            b"SkPictureShader",
            crate::src::shaders::SkPictureShader::SkPictureShader::CreateProc,
        );
        // Source: src/ports/SkGlobalInitialization_default.cpp:61-61
        crate::src::shaders::gradients::SkConicalGradient::SkRegisterConicalGradientShaderFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:62-62
        crate::src::shaders::gradients::SkLinearGradient::SkRegisterLinearGradientShaderFlattenable(
        );
        // Source: src/ports/SkGlobalInitialization_default.cpp:63-63
        crate::src::shaders::gradients::SkRadialGradient::SkRegisterRadialGradientShaderFlattenable(
        );
        // Source: src/ports/SkGlobalInitialization_default.cpp:64-64
        crate::src::shaders::gradients::SkSweepGradient::SkRegisterSweepGradientShaderFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:65-65
        crate::src::shaders::SkPerlinNoiseShaderImpl::SkRegisterPerlinNoiseShaderFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:66-66
        crate::src::shaders::SkWorkingColorSpaceShader::SkRegisterWorkingColorSpaceShaderFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:67-67
        crate::src::shaders::SkShaderBase::RegisterFlattenables();
        // Color filters.
        // Source: src/ports/SkGlobalInitialization_default.cpp:70-70
        crate::src::effects::colorfilters::SkMatrixColorFilter::SkRegisterMatrixColorFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:71-71
        crate::src::effects::colorfilters::SkComposeColorFilter::SkRegisterComposeColorFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:72-72
        crate::src::effects::colorfilters::SkBlendModeColorFilter::SkRegisterModeColorFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:73-73
        crate::src::effects::colorfilters::SkColorSpaceXformColorFilter::SkRegisterSkColorSpaceXformColorFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:74-74
        crate::src::effects::colorfilters::SkWorkingFormatColorFilter::SkRegisterWorkingFormatColorFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:75-75
        crate::src::effects::colorfilters::SkTableColorFilter::SkRegisterTableColorFilterFlattenable();
        // Blenders.
        // Source: src/ports/SkGlobalInitialization_default.cpp:78-78
        crate::src::core::SkFlattenable::Register(
            b"SkBlendModeBlender",
            crate::src::core::SkBlendModeBlender::SkBlendModeBlender::CreateProc,
        );
        // Runtime shaders, color filters, and blenders.
        // Source: src/ports/SkGlobalInitialization_default.cpp:81-81
        crate::src::core::SkRuntimeEffect::SkRuntimeEffect::RegisterFlattenables();
        // Mask filters.
        // Source: src/ports/SkGlobalInitialization_default.cpp:84-84
        crate::src::core::SkFlattenable::Register(
            b"SkEmbossMaskFilter",
            crate::src::effects::SkEmbossMaskFilter::SkEmbossMaskFilter::CreateProc,
        );
        // Source: src/ports/SkGlobalInitialization_default.cpp:85-85
        crate::src::core::SkMaskFilter::RegisterFlattenables();
        // Source: src/ports/SkGlobalInitialization_default.cpp:86-86
        crate::src::effects::SkShaderMaskFilterImpl::RegisterFlattenables();
        // Source: src/ports/SkGlobalInitialization_default.cpp:87-87
        crate::src::effects::SkTableMaskFilter::RegisterFlattenables();
        // Path effects.
        // Source: src/ports/SkGlobalInitialization_default.cpp:90-90
        crate::src::effects::SkCornerPathEffect::RegisterFlattenables();
        // Source: src/ports/SkGlobalInitialization_default.cpp:91-91
        crate::src::core::SkFlattenable::Register(
            b"SkDashImpl",
            crate::src::effects::SkDashImpl::SkDashImpl::CreateProc,
        );
        // Source: src/ports/SkGlobalInitialization_default.cpp:92-92
        crate::src::effects::SkDiscretePathEffect::RegisterFlattenables();
        // Source: src/ports/SkGlobalInitialization_default.cpp:93-93
        crate::src::effects::Sk2DPathEffect::SkLine2DPathEffect::RegisterFlattenables();
        // Source: src/ports/SkGlobalInitialization_default.cpp:94-94
        crate::src::effects::Sk2DPathEffect::SkPath2DPathEffect::RegisterFlattenables();
        // Source: src/ports/SkGlobalInitialization_default.cpp:95-95
        crate::src::effects::Sk1DPathEffect::SkPath1DPathEffect::RegisterFlattenables();
        // Source: src/ports/SkGlobalInitialization_default.cpp:96-96
        crate::src::core::SkFlattenable::Register(
            b"SkTrimPE",
            crate::src::effects::SkTrimPE::SkTrimPE::CreateProc,
        );
        // Source: src/ports/SkGlobalInitialization_default.cpp:97-97
        crate::src::core::SkPathEffect::RegisterFlattenables();
        // Misc.
        // Source: src/ports/SkGlobalInitialization_default.cpp:100-100
        crate::src::core::SkFlattenable::Register(
            b"SkRecordedDrawable",
            crate::src::core::SkRecordedDrawable::SkRecordedDrawable::CreateProc,
        );
    }
}
// Source: src/ports/SkGlobalInitialization_default.cpp:110-130
#[cfg(not(sk_disable_effect_deserialization))]
/// Safety: called exclusively by the native initialization once before publication.
pub unsafe fn InitImageFilters() {
    unsafe {
        // Source: src/ports/SkGlobalInitialization_default.cpp:111-111
        crate::src::effects::imagefilters::SkBlendImageFilter::SkRegisterBlendImageFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:112-112
        crate::src::effects::imagefilters::SkBlurImageFilter::SkRegisterBlurImageFilterFlattenable(
        );
        // Source: src/ports/SkGlobalInitialization_default.cpp:113-113
        crate::src::effects::imagefilters::SkColorFilterImageFilter::SkRegisterColorFilterImageFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:114-114
        crate::src::effects::imagefilters::SkComposeImageFilter::SkRegisterComposeImageFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:115-115
        crate::src::effects::imagefilters::SkCropImageFilter::SkRegisterCropImageFilterFlattenable(
        );
        // Source: src/ports/SkGlobalInitialization_default.cpp:116-116
        crate::src::effects::imagefilters::SkDisplacementMapImageFilter::SkRegisterDisplacementMapImageFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:117-117
        crate::src::effects::imagefilters::SkImageImageFilter::SkRegisterImageImageFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:118-118
        crate::src::effects::imagefilters::SkLightingImageFilter::SkRegisterLightingImageFilterFlattenables();
        // Source: src/ports/SkGlobalInitialization_default.cpp:119-119
        crate::src::effects::imagefilters::SkMagnifierImageFilter::SkRegisterMagnifierImageFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:120-120
        crate::src::effects::imagefilters::SkMatrixConvolutionImageFilter::SkRegisterMatrixConvolutionImageFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:121-121
        crate::src::effects::imagefilters::SkMatrixTransformImageFilter::SkRegisterMatrixTransformImageFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:122-122
        crate::src::effects::imagefilters::SkMergeImageFilter::SkRegisterMergeImageFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:123-123
        crate::src::effects::imagefilters::SkMorphologyImageFilter::SkRegisterMorphologyImageFilterFlattenables();
        // Source: src/ports/SkGlobalInitialization_default.cpp:124-124
        crate::src::effects::imagefilters::SkPictureImageFilter::SkRegisterPictureImageFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:125-125
        crate::src::effects::imagefilters::SkRuntimeImageFilter::SkRegisterRuntimeImageFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:126-126
        crate::src::effects::imagefilters::SkShaderImageFilter::SkRegisterShaderImageFilterFlattenable();
        // Source: src/ports/SkGlobalInitialization_default.cpp:127-127
        crate::src::core::SkFlattenable::Register(
            b"SkLocalMatrixImageFilter",
            crate::src::core::SkLocalMatrixImageFilter::SkLocalMatrixImageFilter::CreateProc,
        );
        // Source: src/ports/SkGlobalInitialization_default.cpp:129-129
        crate::src::effects::imagefilters::SkDropShadowImageFilter::SkRegisterLegacyDropShadowImageFilterFlattenable();
    }
}
