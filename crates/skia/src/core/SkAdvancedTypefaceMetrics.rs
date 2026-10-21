// Copyright 2011 Google Inc. BSD-style license.
use super::{SkRect::SkIRect, SkString::SkString};
// Source: src/core/SkAdvancedTypefaceMetrics.h:24-26
// Source: src/core/SkAdvancedTypefaceMetrics.h:38-38
// Source: src/core/SkAdvancedTypefaceMetrics.h:50-50
// Source: src/core/SkAdvancedTypefaceMetrics.h:58-58
// Source: src/core/SkAdvancedTypefaceMetrics.h:60-68
// Rust fields must be contiguous; declarations retain native field order.
pub struct SkAdvancedTypefaceMetrics {
    pub fPostScriptName: SkString,
    pub fStyle: StyleFlags,
    pub fType: FontType,
    pub fFlags: FontFlags,
    pub fItalicAngle: i16,
    pub fAscent: i16,
    pub fDescent: i16,
    pub fStemV: i16,
    pub fCapHeight: i16,
    pub fBBox: SkIRect,
}
// Source: src/core/SkAdvancedTypefaceMetrics.h:29-37
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StyleFlags(pub u32);
impl StyleFlags {
    pub const kFixedPitch_Style: Self = Self(0x00000001);
    pub const kSerif_Style: Self = Self(0x00000002);
    pub const kScript_Style: Self = Self(0x00000008);
    pub const kItalic_Style: Self = Self(0x00000040);
    pub const kAllCaps_Style: Self = Self(0x00010000);
    pub const kSmallCaps_Style: Self = Self(0x00020000);
    pub const kForceBold_Style: Self = Self(0x00040000);
}
// Source: src/core/SkAdvancedTypefaceMetrics.h:40-46
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FontType {
    kType1_Font,
    kType1CID_Font,
    kCFF_Font,
    kTrueType_Font,
    kOther_Font,
}
// Source: src/core/SkAdvancedTypefaceMetrics.h:52-57
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FontFlags(pub u8);
impl FontFlags {
    pub const kVariable_FontFlag: Self = Self(1 << 0);
    pub const kNotEmbeddable_FontFlag: Self = Self(1 << 1);
    pub const kNotSubsettable_FontFlag: Self = Self(1 << 2);
    pub const kAltDataFormat_FontFlag: Self = Self(1 << 3);
}
// Native default member initialization, including the zero bitmask values.
impl Default for SkAdvancedTypefaceMetrics {
    fn default() -> Self {
        Self {
            fPostScriptName: SkString::new(),
            fStyle: StyleFlags(0),
            fType: FontType::kOther_Font,
            fFlags: FontFlags(0),
            fItalicAngle: 0,
            fAscent: 0,
            fDescent: 0,
            fStemV: 0,
            fCapHeight: 0,
            fBBox: SkIRect::make_ltrb(0, 0, 0, 0),
        }
    }
}
// Source: src/core/SkAdvancedTypefaceMetrics.h:71-74
macro_rules! bitmask {
    ($t:ty) => {
        impl core::ops::BitOr for $t {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }
        impl core::ops::BitOrAssign for $t {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }
        impl core::ops::BitAnd for $t {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self {
                Self(self.0 & rhs.0)
            }
        }
        impl core::ops::BitAndAssign for $t {
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0;
            }
        }
        impl core::ops::BitXor for $t {
            type Output = Self;
            fn bitxor(self, rhs: Self) -> Self {
                Self(self.0 ^ rhs.0)
            }
        }
        impl core::ops::BitXorAssign for $t {
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0;
            }
        }
        impl core::ops::Not for $t {
            type Output = Self;
            fn not(self) -> Self {
                Self(!self.0)
            }
        }
    };
}
bitmask!(FontFlags);
bitmask!(StyleFlags);
