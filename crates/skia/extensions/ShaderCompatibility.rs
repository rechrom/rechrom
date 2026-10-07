//! Migrated raster shader convenience types without exact current upstream Skia counterparts.
//! FilterQuality approximates sampling options; PixmapPaint combines draw-image paint settings.
use crate::raster::BlendMode;
/// Controls how much filtering to be done when transforming images.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FilterQuality {
    /// Nearest-neighbor. Low quality, but fastest.
    Nearest,
    /// Bilinear.
    Bilinear,
    /// Bicubic. High quality, but slow.
    Bicubic,
}

/// Controls how a pixmap should be blended.
///
/// Like `Paint`, but for `Pixmap`.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct PixmapPaint {
    /// Pixmap opacity.
    ///
    /// Must be in 0..=1 range.
    ///
    /// Default: 1.0
    pub opacity: f32,

    /// Pixmap blending mode.
    ///
    /// Default: SourceOver
    pub blend_mode: BlendMode,

    /// Specifies how much filtering to be done when transforming images.
    ///
    /// Default: Nearest
    pub quality: FilterQuality,
}

impl Default for PixmapPaint {
    fn default() -> Self {
        PixmapPaint {
            opacity: 1.0,
            blend_mode: BlendMode::default(),
            quality: FilterQuality::Nearest,
        }
    }
}
