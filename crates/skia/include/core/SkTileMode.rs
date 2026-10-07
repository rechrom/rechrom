//! Tile modes correspond to include/core/SkTileMode.h.
//! Rust variant order preserves migrated layout; no C++ ABI value is promised.
/// A shader spreading mode.
#[derive(Copy, Clone, Default, PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum SkTileMode {
    /// Replicate the edge color if the shader draws outside of its
    /// original bounds.
    #[default]
    kClamp,

    /// Repeat the shader's image horizontally and vertically, alternating
    /// mirror images so that adjacent images always seam.
    kMirror,

    /// Repeat the shader's image horizontally and vertically.
    kRepeat,
}

#[allow(non_upper_case_globals)]
impl SkTileMode {
    pub const Pad: Self = Self::kClamp;
    pub const Reflect: Self = Self::kMirror;
    pub const Repeat: Self = Self::kRepeat;
}
pub use SkTileMode as SpreadMode;
