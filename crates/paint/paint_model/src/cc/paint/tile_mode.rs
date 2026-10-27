//! Backend-neutral ownership of Skia's four public tile modes.
//!
//! Source: include/core/SkTileMode.h:13-39. Concrete legacy and translated
//! Skia enums are selected only inside their raster backends.

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TileMode {
    #[default]
    kClamp,
    kRepeat,
    kMirror,
    kDecal,
}
