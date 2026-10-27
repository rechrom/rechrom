//! Canonical cc paint blend-mode value.
//!
//! Paint owns the recorded semantic value. Concrete Skia backends translate it
//! only when executing a raster command.
#![allow(non_camel_case_types, non_snake_case)]

// Source: include/core/SkBlendMode.h:35-69 and cc/paint/paint_flags.h.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BlendMode {
    kClear,
    kSrc,
    kDst,
    kSrcOver,
    kDstOver,
    kSrcIn,
    kDstIn,
    kSrcOut,
    kDstOut,
    kSrcATop,
    kDstATop,
    kXor,
    kPlus,
    kModulate,
    kScreen,
    kOverlay,
    kDarken,
    kLighten,
    kColorDodge,
    kColorBurn,
    kHardLight,
    kSoftLight,
    kDifference,
    kExclusion,
    kMultiply,
    kHue,
    kSaturation,
    kColor,
    kLuminosity,
}

impl BlendMode {
    pub fn Name(self) -> &'static str {
        match self {
            Self::kClear => "Clear",
            Self::kSrc => "Src",
            Self::kDst => "Dst",
            Self::kSrcOver => "SrcOver",
            Self::kDstOver => "DstOver",
            Self::kSrcIn => "SrcIn",
            Self::kDstIn => "DstIn",
            Self::kSrcOut => "SrcOut",
            Self::kDstOut => "DstOut",
            Self::kSrcATop => "SrcATop",
            Self::kDstATop => "DstATop",
            Self::kXor => "Xor",
            Self::kPlus => "Plus",
            Self::kModulate => "Modulate",
            Self::kScreen => "Screen",
            Self::kOverlay => "Overlay",
            Self::kDarken => "Darken",
            Self::kLighten => "Lighten",
            Self::kColorDodge => "ColorDodge",
            Self::kColorBurn => "ColorBurn",
            Self::kHardLight => "HardLight",
            Self::kSoftLight => "SoftLight",
            Self::kDifference => "Difference",
            Self::kExclusion => "Exclusion",
            Self::kMultiply => "Multiply",
            Self::kHue => "Hue",
            Self::kSaturation => "Saturation",
            Self::kColor => "Color",
            Self::kLuminosity => "Luminosity",
        }
    }
}
