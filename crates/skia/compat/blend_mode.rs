//! Compatibility constants for the former Rust blend-mode variant names.
use crate::include::core::SkBlendMode::SkBlendMode;
#[allow(non_upper_case_globals)]
impl SkBlendMode {
    pub const Clear: Self = Self::kClear;
    pub const Source: Self = Self::kSrc;
    pub const Destination: Self = Self::kDst;
    pub const SourceOver: Self = Self::kSrcOver;
    pub const DestinationOver: Self = Self::kDstOver;
    pub const SourceIn: Self = Self::kSrcIn;
    pub const DestinationIn: Self = Self::kDstIn;
    pub const SourceOut: Self = Self::kSrcOut;
    pub const DestinationOut: Self = Self::kDstOut;
    pub const SourceAtop: Self = Self::kSrcATop;
    pub const DestinationAtop: Self = Self::kDstATop;
    pub const Xor: Self = Self::kXor;
    pub const Plus: Self = Self::kPlus;
    pub const Modulate: Self = Self::kModulate;
    pub const Screen: Self = Self::kScreen;
    pub const Overlay: Self = Self::kOverlay;
    pub const Darken: Self = Self::kDarken;
    pub const Lighten: Self = Self::kLighten;
    pub const ColorDodge: Self = Self::kColorDodge;
    pub const ColorBurn: Self = Self::kColorBurn;
    pub const HardLight: Self = Self::kHardLight;
    pub const SoftLight: Self = Self::kSoftLight;
    pub const Difference: Self = Self::kDifference;
    pub const Exclusion: Self = Self::kExclusion;
    pub const Multiply: Self = Self::kMultiply;
    pub const Hue: Self = Self::kHue;
    pub const Saturation: Self = Self::kSaturation;
    pub const Color: Self = Self::kColor;
    pub const Luminosity: Self = Self::kLuminosity;
}
