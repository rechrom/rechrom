// Migrated unchanged in behavior from tiny-skia-0.12.0/src/blend_mode.rs.

/// A blending mode.
#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[allow(non_camel_case_types)]
pub enum SkBlendMode {
    /// Replaces destination with zero: fully transparent.
    kClear,
    /// Replaces destination.
    kSrc,
    /// Preserves destination.
    kDst,
    /// kSrc over destination.
    #[default]
    kSrcOver,
    /// kDst over source.
    kDstOver,
    /// kSrc trimmed inside destination.
    kSrcIn,
    /// kDst trimmed by source.
    kDstIn,
    /// kSrc trimmed outside destination.
    kSrcOut,
    /// kDst trimmed outside source.
    kDstOut,
    /// kSrc inside destination blended with destination.
    kSrcATop,
    /// kDst inside source blended with source.
    kDstATop,
    /// Each of source and destination trimmed outside the other.
    kXor,
    /// Sum of colors.
    kPlus,
    /// Product of premultiplied colors; darkens destination.
    kModulate,
    /// kMultiply inverse of pixels, inverting result; brightens destination.
    kScreen,
    /// kMultiply or screen, depending on destination.
    kOverlay,
    /// Darker of source and destination.
    kDarken,
    /// Lighter of source and destination.
    kLighten,
    /// Brighten destination to reflect source.
    kColorDodge,
    /// kDarken destination to reflect source.
    kColorBurn,
    /// kMultiply or screen, depending on source.
    kHardLight,
    /// kLighten or darken, depending on source.
    kSoftLight,
    /// Subtract darker from lighter with higher contrast.
    kDifference,
    /// Subtract darker from lighter with lower contrast.
    kExclusion,
    /// kMultiply source with destination, darkening image.
    kMultiply,
    /// kHue of source with saturation and luminosity of destination.
    kHue,
    /// kSaturation of source with hue and luminosity of destination.
    kSaturation,
    /// kHue and saturation of source with luminosity of destination.
    kColor,
    /// kLuminosity of source with hue and saturation of destination.
    kLuminosity,
}

// Compatibility name for the Rust raster API.
pub use SkBlendMode as BlendMode;
