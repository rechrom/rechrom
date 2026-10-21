//! Encoded image metadata; header and actual cpp implementation share this target.
use crate::include::private::SkHdrMetadata::Metadata;
use crate::modules::skcms::src::skcms_public::skcms_ICCProfile;
use crate::src::codec::SkCodecPriv::SkCodecs::ColorProfile;
use crate::src::core::{SkColorSpace::SkColorSpace, SkData::SkData, SkImageInfo::SkImageInfo};
use std::sync::Arc;
// Source: include/private/SkEncodedInfo.h:34
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Alpha(pub core::ffi::c_int);
impl Alpha {
    // Source: include/private/SkEncodedInfo.h:35
    pub const kOpaque_Alpha: Self = Self(0);
    // Source: include/private/SkEncodedInfo.h:36
    pub const kUnpremul_Alpha: Self = Self(1);
    // Source: include/private/SkEncodedInfo.h:40
    pub const kBinary_Alpha: Self = Self(2);
}
// Source: include/private/SkEncodedInfo.h:51
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(pub core::ffi::c_int);
impl Color {
    // Source: include/private/SkEncodedInfo.h:53
    pub const kGray_Color: Self = Self(0);
    // Source: include/private/SkEncodedInfo.h:56
    pub const kGrayAlpha_Color: Self = Self(1);
    // Source: include/private/SkEncodedInfo.h:63
    pub const kXAlpha_Color: Self = Self(2);
    // Source: include/private/SkEncodedInfo.h:70
    pub const k565_Color: Self = Self(3);
    // Source: include/private/SkEncodedInfo.h:73
    pub const kPalette_Color: Self = Self(4);
    // Source: include/private/SkEncodedInfo.h:76
    pub const kRGB_Color: Self = Self(5);
    // Source: include/private/SkEncodedInfo.h:77
    pub const kRGBA_Color: Self = Self(6);
    // Source: include/private/SkEncodedInfo.h:80
    pub const kBGR_Color: Self = Self(7);
    // Source: include/private/SkEncodedInfo.h:81
    pub const kBGRX_Color: Self = Self(8);
    // Source: include/private/SkEncodedInfo.h:82
    pub const kBGRA_Color: Self = Self(9);
    // Source: include/private/SkEncodedInfo.h:85
    pub const kYUV_Color: Self = Self(10);
    // Source: include/private/SkEncodedInfo.h:88
    pub const kYUVA_Color: Self = Self(11);
    // Source: include/private/SkEncodedInfo.h:96
    pub const kInvertedCMYK_Color: Self = Self(12);
    // Source: include/private/SkEncodedInfo.h:97
    pub const kYCCK_Color: Self = Self(13);
}
// Source: include/private/SkEncodedInfo.h:32
// Private field order supplies native reverse destruction without a Drop impl.
// No C++ layout/ABI is claimed; the native constructor expression order stays below.
pub struct SkEncodedInfo {
    // Source: include/private/SkEncodedInfo.h:246
    fHdrMetadata: Metadata,
    // Source: include/private/SkEncodedInfo.h:245
    fColorProfile: Option<Box<ColorProfile>>,
    // Source: include/private/SkEncodedInfo.h:244
    fColorDepth: u8,
    // Source: include/private/SkEncodedInfo.h:243
    fBitsPerComponent: u8,
    // Source: include/private/SkEncodedInfo.h:242
    fAlpha: Alpha,
    // Source: include/private/SkEncodedInfo.h:241
    fColor: Color,
    // Source: include/private/SkEncodedInfo.h:240
    fHeight: i32,
    // Source: include/private/SkEncodedInfo.h:239
    fWidth: i32,
}
impl SkEncodedInfo {
    // Source: include/private/SkEncodedInfo.h:123
    pub fn width(&self) -> i32 {
        self.fWidth
    }
    // Source: include/private/SkEncodedInfo.h:124
    pub fn height(&self) -> i32 {
        self.fHeight
    }
    // Source: include/private/SkEncodedInfo.h:125
    pub fn color(&self) -> Color {
        self.fColor
    }
    // Source: include/private/SkEncodedInfo.h:126
    pub fn alpha(&self) -> Alpha {
        self.fAlpha
    }
    // Source: include/private/SkEncodedInfo.h:137
    pub fn bitsPerComponent(&self) -> u8 {
        self.fBitsPerComponent
    }
    // Source: include/private/SkEncodedInfo.h:175
    pub fn getColorDepth(&self) -> u8 {
        self.fColorDepth
    }
    // Source: include/private/SkEncodedInfo.h:127
    pub fn opaque(&self) -> bool {
        self.fAlpha == Alpha::kOpaque_Alpha
    }
    // Source: include/private/SkEncodedInfo.h:133
    pub fn colorProfile(&self) -> Option<&ColorProfile> {
        self.fColorProfile.as_deref()
    }
    // Source: include/private/SkEncodedInfo.h:181
    pub fn getHdrMetadata(&self) -> &Metadata {
        &self.fHdrMetadata
    }
    // Source: include/private/SkEncodedInfo.h:139
    pub fn bitsPerPixel(&self) -> u8 {
        match self.fColor {
            Color::kGray_Color | Color::kPalette_Color => self.fBitsPerComponent,
            Color::kXAlpha_Color | Color::kGrayAlpha_Color => {
                self.fBitsPerComponent.wrapping_mul(2)
            }
            Color::kRGB_Color | Color::kBGR_Color | Color::kYUV_Color | Color::k565_Color => {
                self.fBitsPerComponent.wrapping_mul(3)
            }
            Color::kRGBA_Color
            | Color::kBGRA_Color
            | Color::kBGRX_Color
            | Color::kYUVA_Color
            | Color::kInvertedCMYK_Color
            | Color::kYCCK_Color => self.fBitsPerComponent.wrapping_mul(4),
            _ => {
                crate::SkASSERT!(@source b"include/private/SkEncodedInfo.h\0".as_ptr(),161; @condition b"false\0".as_ptr(); false);
                0
            }
        }
    }
    // Source: include/private/SkEncodedInfo.h:193
    fn VerifyColor(color: Color, alpha: Alpha, bitsPerComponent: i32) {
        let _ = alpha;
        let _ = bitsPerComponent;
        match color {
            Color::kGray_Color => {
                crate::SkASSERT!(@source b"include/private/SkEncodedInfo.h\0".as_ptr(),200; @condition b"kOpaque_Alpha == alpha\0".as_ptr(); Alpha::kOpaque_Alpha == alpha);
                return;
            }
            Color::kGrayAlpha_Color => {
                crate::SkASSERT!(@source b"include/private/SkEncodedInfo.h\0".as_ptr(),203; @condition b"kOpaque_Alpha != alpha\0".as_ptr(); Alpha::kOpaque_Alpha != alpha);
                return;
            }
            Color::kPalette_Color => {
                crate::SkASSERT!(@source b"include/private/SkEncodedInfo.h\0".as_ptr(),206; @condition b"16 != bitsPerComponent\0".as_ptr(); 16 != bitsPerComponent);
                return;
            }
            Color::kRGB_Color | Color::kBGR_Color | Color::kBGRX_Color => {
                crate::SkASSERT!(@source b"include/private/SkEncodedInfo.h\0".as_ptr(),200; @condition b"kOpaque_Alpha == alpha\0".as_ptr(); Alpha::kOpaque_Alpha == alpha);
                crate::SkASSERT!(@source b"include/private/SkEncodedInfo.h\0".as_ptr(),212; @condition b"bitsPerComponent >= 8\0".as_ptr(); bitsPerComponent >= 8);
                return;
            }
            Color::kYUV_Color | Color::kInvertedCMYK_Color | Color::kYCCK_Color => {
                crate::SkASSERT!(@source b"include/private/SkEncodedInfo.h\0".as_ptr(),200; @condition b"kOpaque_Alpha == alpha\0".as_ptr(); Alpha::kOpaque_Alpha == alpha);
                crate::SkASSERT!(@source b"include/private/SkEncodedInfo.h\0".as_ptr(),218; @condition b"8 == bitsPerComponent\0".as_ptr(); 8 == bitsPerComponent);
                return;
            }
            Color::kRGBA_Color => {
                crate::SkASSERT!(@source b"include/private/SkEncodedInfo.h\0".as_ptr(),212; @condition b"bitsPerComponent >= 8\0".as_ptr(); bitsPerComponent >= 8);
                return;
            }
            Color::kBGRA_Color | Color::kYUVA_Color => {
                crate::SkASSERT!(@source b"include/private/SkEncodedInfo.h\0".as_ptr(),218; @condition b"8 == bitsPerComponent\0".as_ptr(); 8 == bitsPerComponent);
                return;
            }
            Color::kXAlpha_Color => {
                crate::SkASSERT!(@source b"include/private/SkEncodedInfo.h\0".as_ptr(),228; @condition b"kUnpremul_Alpha == alpha\0".as_ptr(); Alpha::kUnpremul_Alpha == alpha);
                crate::SkASSERT!(@source b"include/private/SkEncodedInfo.h\0".as_ptr(),218; @condition b"8 == bitsPerComponent\0".as_ptr(); 8 == bitsPerComponent);
                return;
            }
            Color::k565_Color => {
                crate::SkASSERT!(@source b"include/private/SkEncodedInfo.h\0".as_ptr(),200; @condition b"kOpaque_Alpha == alpha\0".as_ptr(); Alpha::kOpaque_Alpha == alpha);
                crate::SkASSERT!(@source b"include/private/SkEncodedInfo.h\0".as_ptr(),218; @condition b"8 == bitsPerComponent\0".as_ptr(); 8 == bitsPerComponent);
                return;
            }
            _ => {}
        }
        crate::SkASSERT!(@source b"include/private/SkEncodedInfo.h\0".as_ptr(),236; @condition b"false\0".as_ptr(); false);
    }
}
// Source: src/codec/SkEncodedInfo.cpp:12
impl SkEncodedInfo {
    pub fn makeImageInfo(&self) -> SkImageInfo {
        use crate::include::core::{SkAlphaType::*, SkColorType::*};
        let ct = if self.fColor == Color::kGray_Color {
            kGray_8_SkColorType
        } else if self.fColor == Color::kXAlpha_Color {
            kAlpha_8_SkColorType
        } else if self.fColor == Color::k565_Color {
            kRGB_565_SkColorType
        } else {
            kN32_SkColorType
        };
        let alpha = if self.fAlpha == Alpha::kOpaque_Alpha {
            kOpaque_SkAlphaType
        } else {
            kUnpremul_SkAlphaType
        };
        let mut cs = if let Some(profile) = &self.fColorProfile {
            profile.getExactColorSpace()
        } else {
            Some(SkColorSpace::MakeSRGB())
        };
        if cs.is_none() {
            cs = Some(SkColorSpace::MakeSRGB());
        }
        SkImageInfo::MakeWithColorSpace(self.fWidth, self.fHeight, ct, alpha, cs)
    }
    // Source: include/private/SkEncodedInfo.h:100-101
    // Source: src/codec/SkEncodedInfo.cpp:27-30
    pub fn Make(
        width: i32,
        height: i32,
        color: Color,
        alpha: Alpha,
        bitsPerComponent: i32,
    ) -> Self {
        Self::Make_profile(width, height, color, alpha, bitsPerComponent, None)
    }
    // Source: include/private/SkEncodedInfo.h:103-105
    // Source: src/codec/SkEncodedInfo.cpp:33-38
    pub fn Make_profile(
        width: i32,
        height: i32,
        color: Color,
        alpha: Alpha,
        bitsPerComponent: i32,
        profile: Option<Box<ColorProfile>>,
    ) -> Self {
        Self::Make_profile_depth(
            width,
            height,
            color,
            alpha,
            bitsPerComponent,
            profile,
            bitsPerComponent,
        )
    }
    // Source: include/private/SkEncodedInfo.h:107-109
    // Source: src/codec/SkEncodedInfo.cpp:41-46
    pub fn Make_profile_depth(
        width: i32,
        height: i32,
        color: Color,
        alpha: Alpha,
        bitsPerComponent: i32,
        profile: Option<Box<ColorProfile>>,
        colorDepth: i32,
    ) -> Self {
        Self::Make_hdr(
            width,
            height,
            color,
            alpha,
            bitsPerComponent,
            colorDepth,
            profile,
            &Metadata::MakeEmpty(),
        )
    }
    // Source: include/private/SkEncodedInfo.h:111-114
    // Source: src/codec/SkEncodedInfo.cpp:49-60
    pub fn Make_hdr(
        width: i32,
        height: i32,
        color: Color,
        alpha: Alpha,
        bitsPerComponent: i32,
        colorDepth: i32,
        profile: Option<Box<ColorProfile>>,
        hdrMetadata: &Metadata,
    ) -> Self {
        crate::SkASSERT!(@source b"src/codec/SkEncodedInfo.cpp\0".as_ptr(),52; @condition b"1 == bitsPerComponent || 2 == bitsPerComponent || 4 == bitsPerComponent || 8 == bitsPerComponent || 16 == bitsPerComponent\0".as_ptr(); bitsPerComponent==1||bitsPerComponent==2||bitsPerComponent==4||bitsPerComponent==8||bitsPerComponent==16);
        Self::VerifyColor(color, alpha, bitsPerComponent);
        // Actual SkToU8 declaration/body is a direct dependency; no replacement cast.
        Self::new_fields(
            width,
            height,
            color,
            alpha,
            crate::include::private::SkTo::SkToU8(bitsPerComponent),
            crate::include::private::SkTo::SkToU8(colorDepth),
            profile,
            hdrMetadata,
        )
    }
    // Source: src/codec/SkEncodedInfo.cpp:62
    pub fn profile(&self) -> *const skcms_ICCProfile {
        self.fColorProfile
            .as_ref()
            .map_or(core::ptr::null(), |profile| profile.profile())
    }
    // Source: src/codec/SkEncodedInfo.cpp:67
    pub fn profileData(&self) -> Option<Arc<SkData>> {
        self.fColorProfile
            .as_ref()
            .and_then(|profile| profile.data())
    }
    // Source: include/private/SkEncodedInfo.h:171-172
    // Source: src/codec/SkEncodedInfo.cpp:72-76
    pub fn copy(&self) -> Self {
        Self::new_fields(
            self.fWidth,
            self.fHeight,
            self.fColor,
            self.fAlpha,
            self.fBitsPerComponent,
            self.fColorDepth,
            self.fColorProfile
                .as_ref()
                .map(|profile| ColorProfile::clone(profile.as_ref())),
            &self.fHdrMetadata,
        )
    }
    // Source: src/codec/SkEncodedInfo.cpp:78
    // Native lvalue source stays live: scalars retain values, unique_ptr becomes
    // null, and each optional metadata object retains its engagement.
    pub fn move_from(orig: &mut Self) -> Self {
        let fWidth = orig.fWidth;
        let fHeight = orig.fHeight;
        let fColor = orig.fColor;
        let fAlpha = orig.fAlpha;
        let fBitsPerComponent = orig.fBitsPerComponent;
        let fColorDepth = orig.fColorDepth;
        let fColorProfile = orig.fColorProfile.take();
        let fHdrMetadata = Metadata::move_from(&mut orig.fHdrMetadata);
        Self {
            fWidth,
            fHeight,
            fColor,
            fAlpha,
            fBitsPerComponent,
            fColorDepth,
            fColorProfile,
            fHdrMetadata,
        }
    }
    // Source: src/codec/SkEncodedInfo.cpp:79
    pub fn move_assign(&mut self, orig: &mut Self) -> &mut Self {
        self.fWidth = orig.fWidth;
        self.fHeight = orig.fHeight;
        self.fColor = orig.fColor;
        self.fAlpha = orig.fAlpha;
        self.fBitsPerComponent = orig.fBitsPerComponent;
        self.fColorDepth = orig.fColorDepth;
        // libc++ unique_ptr::operator= releases the source before resetting target.
        self.fColorProfile = orig.fColorProfile.take();
        self.fHdrMetadata.move_assign(&mut orig.fHdrMetadata);
        self
    }
    // Source: src/codec/SkEncodedInfo.cpp:79
    /// Safety: pointers identify live exclusive complete objects; they may be
    /// equal for the actual native self-move case. No other overlapping objects.
    pub unsafe fn move_assign_raw(dst: core::ptr::NonNull<Self>, src: core::ptr::NonNull<Self>) {
        if dst == src {
            // unique_ptr self-move releases then resets the same pointer, preserving
            // its object. Scalars stay unchanged; optional/vector self-moves proceed.
            unsafe {
                (*dst.as_ptr()).fHdrMetadata.self_move_members();
            }
        } else {
            unsafe {
                (&mut *dst.as_ptr()).move_assign(&mut *src.as_ptr());
            }
        }
    }
    // Source: include/private/SkEncodedInfo.h:185
    // Source: src/codec/SkEncodedInfo.cpp:80
    // Default destruction is implemented by the actual private Rust field order:
    // HDR metadata first, then unique ColorProfile, with no manual Drop move ban.
    // Source: include/private/SkEncodedInfo.h:188-191
    // Source: src/codec/SkEncodedInfo.cpp:82-93
    fn new_fields(
        width: i32,
        height: i32,
        color: Color,
        alpha: Alpha,
        bitsPerComponent: u8,
        colorDepth: u8,
        profile: Option<Box<ColorProfile>>,
        hdrMetadata: &Metadata,
    ) -> Self {
        Self {
            fWidth: width,
            fHeight: height,
            fColor: color,
            fAlpha: alpha,
            fBitsPerComponent: bitsPerComponent,
            fColorDepth: colorDepth,
            fColorProfile: profile,
            fHdrMetadata: hdrMetadata.clone(),
        }
    }
}

// Native unscoped member enumerators retain SkEncodedInfo::name lookup.
impl SkEncodedInfo {
    // Source: include/private/SkEncodedInfo.h:35
    pub const kOpaque_Alpha: Alpha = Alpha::kOpaque_Alpha;
    // Source: include/private/SkEncodedInfo.h:36
    pub const kUnpremul_Alpha: Alpha = Alpha::kUnpremul_Alpha;
    // Source: include/private/SkEncodedInfo.h:40
    pub const kBinary_Alpha: Alpha = Alpha::kBinary_Alpha;
    // Source: include/private/SkEncodedInfo.h:53
    pub const kGray_Color: Color = Color::kGray_Color;
    // Source: include/private/SkEncodedInfo.h:56
    pub const kGrayAlpha_Color: Color = Color::kGrayAlpha_Color;
    // Source: include/private/SkEncodedInfo.h:63
    pub const kXAlpha_Color: Color = Color::kXAlpha_Color;
    // Source: include/private/SkEncodedInfo.h:70
    pub const k565_Color: Color = Color::k565_Color;
    // Source: include/private/SkEncodedInfo.h:73
    pub const kPalette_Color: Color = Color::kPalette_Color;
    // Source: include/private/SkEncodedInfo.h:76
    pub const kRGB_Color: Color = Color::kRGB_Color;
    // Source: include/private/SkEncodedInfo.h:77
    pub const kRGBA_Color: Color = Color::kRGBA_Color;
    // Source: include/private/SkEncodedInfo.h:80
    pub const kBGR_Color: Color = Color::kBGR_Color;
    // Source: include/private/SkEncodedInfo.h:81
    pub const kBGRX_Color: Color = Color::kBGRX_Color;
    // Source: include/private/SkEncodedInfo.h:82
    pub const kBGRA_Color: Color = Color::kBGRA_Color;
    // Source: include/private/SkEncodedInfo.h:85
    pub const kYUV_Color: Color = Color::kYUV_Color;
    // Source: include/private/SkEncodedInfo.h:88
    pub const kYUVA_Color: Color = Color::kYUVA_Color;
    // Source: include/private/SkEncodedInfo.h:96
    pub const kInvertedCMYK_Color: Color = Color::kInvertedCMYK_Color;
    // Source: include/private/SkEncodedInfo.h:97
    pub const kYCCK_Color: Color = Color::kYCCK_Color;
}
