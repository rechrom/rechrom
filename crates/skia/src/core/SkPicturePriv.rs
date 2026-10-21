// Copyright 2018 Google LLC. BSD-style license.
// Native Picture helpers and version constants. Serialization implementations
// remain separate from these retained-owner projections.
// Source: src/core/SkPicturePriv.h:34-34
pub const kDefaultRecursionLimit: i32 = 100;
// Source: src/core/SkPicturePriv.h:43-45
pub fn AsSkBigPicture(
    picture: &crate::include::core::SkRefCnt::sk_sp<dyn super::SkPicture::SkPicture>,
) -> Option<&super::SkBigPicture::SkBigPicture> {
    picture.asSkBigPicture()
}
// Source: src/core/SkPicturePriv.h:47-50
pub fn MakeSharedID(pictureID: u32) -> u64 {
    let sharedID = u32::from_be_bytes(*b"pict") as u64;
    (sharedID << 32) | u64::from(pictureID)
}
// Source: src/core/SkPicturePriv.h:52-54
pub fn AddedToCache(pic: &dyn super::SkPicture::SkPicture) {
    pic.picture_state()
        .fAddedToCache
        .store(true, std::sync::atomic::Ordering::SeqCst);
}
// Native namespace declarations bind the definitions retained with SkPicture.cpp.
// The existing Rust rendering-error boundary remains explicit to every caller.
// Source: src/core/SkPicturePriv.h:35-35
// Source: src/core/SkPicturePriv.h:35-35
pub use super::SkPicture::PicturePriv_MakeFromBuffer as MakeFromBuffer;
// Source: src/core/SkPicturePriv.h:40-40
// Source: src/core/SkPicturePriv.h:40-40
pub use super::SkPicture::PicturePriv_Flatten as Flatten;
// Source: src/core/SkPicturePriv.h:132-132
// Rust namespace constants preserve native integer values and duplicate aliases.
pub type Version = i32;
// Source: src/core/SkPicturePriv.h:133-133
pub const kPictureShaderFilterParam_Version: Version = 82;
// Source: src/core/SkPicturePriv.h:134-134
pub const kMatrixImageFilterSampling_Version: Version = 83;
// Source: src/core/SkPicturePriv.h:135-135
pub const kImageFilterImageSampling_Version: Version = 84;
// Source: src/core/SkPicturePriv.h:136-136
pub const kNoFilterQualityShaders_Version: Version = 85;
// Source: src/core/SkPicturePriv.h:137-137
pub const kVerticesRemoveCustomData_Version: Version = 86;
// Source: src/core/SkPicturePriv.h:138-138
pub const kSkBlenderInSkPaint: Version = 87;
// Source: src/core/SkPicturePriv.h:139-139
pub const kBlenderInEffects: Version = 88;
// Source: src/core/SkPicturePriv.h:140-140
pub const kNoExpandingClipOps: Version = 89;
// Source: src/core/SkPicturePriv.h:141-141
pub const kBackdropScaleFactor: Version = 90;
// Source: src/core/SkPicturePriv.h:142-142
pub const kRawImageShaders: Version = 91;
// Source: src/core/SkPicturePriv.h:143-143
pub const kAnisotropicFilter: Version = 92;
// Source: src/core/SkPicturePriv.h:144-144
pub const kBlend4fColorFilter: Version = 93;
// Source: src/core/SkPicturePriv.h:145-145
pub const kNoShaderLocalMatrix: Version = 94;
// Source: src/core/SkPicturePriv.h:146-146
pub const kShaderImageFilterSerializeShader: Version = 95;
// Source: src/core/SkPicturePriv.h:147-147
pub const kRevampMagnifierFilter: Version = 96;
// Source: src/core/SkPicturePriv.h:148-148
pub const kRuntimeImageFilterSampleRadius: Version = 97;
// Source: src/core/SkPicturePriv.h:149-149
pub const kCombineBlendArithmeticFilters: Version = 98;
// Source: src/core/SkPicturePriv.h:150-150
pub const kRemoveLegacyMagnifierFilter: Version = 99;
// Source: src/core/SkPicturePriv.h:151-151
pub const kDropShadowImageFilterComposition: Version = 100;
// Source: src/core/SkPicturePriv.h:152-152
pub const kCropImageFilterSupportsTiling: Version = 101;
// Source: src/core/SkPicturePriv.h:153-153
pub const kConvolutionImageFilterTilingUpdate: Version = 102;
// Source: src/core/SkPicturePriv.h:154-154
pub const kRemoveDeprecatedCropRect: Version = 103;
// Source: src/core/SkPicturePriv.h:155-155
pub const kMultipleFiltersOnSaveLayer: Version = 104;
// Source: src/core/SkPicturePriv.h:156-156
pub const kUnclampedMatrixColorFilter: Version = 105;
// Source: src/core/SkPicturePriv.h:157-157
pub const kSaveLayerBackdropTileMode: Version = 106;
// Source: src/core/SkPicturePriv.h:158-158
pub const kCombineColorShaders: Version = 107;
// Source: src/core/SkPicturePriv.h:159-159
pub const kSerializeStableKeys: Version = 108;
// Source: src/core/SkPicturePriv.h:160-160
pub const kWorkingColorSpaceOutput: Version = 109;
// Source: src/core/SkPicturePriv.h:161-161
pub const kRuntimeImageFilterRestrictedOutput: Version = 110;
// Source: src/core/SkPicturePriv.h:185-185
pub const kMin_Version: Version = kPictureShaderFilterParam_Version;
// Source: src/core/SkPicturePriv.h:186-186
pub const kCurrent_Version: Version = kRuntimeImageFilterRestrictedOutput;

// Source: src/core/SkPicturePriv.h:190-190
pub use super::SkPicture::SkPicture_StreamIsSKP;
