//! SkPaint raster style subset; Rust lifetimes and convenience methods adapt
//! ownership to the directly migrated raster code, without C++ ABI claims.
use crate::raster::{BlendMode, Color, ColorSpace, Shader};
#[derive(Clone, PartialEq, Debug)]
pub struct SkPaint<'a> {
    /// A paint shader.
    ///
    /// Default: black color
    pub shader: Shader<'a>,

    /// SkPaint blending mode.
    ///
    /// Default: SourceOver
    pub blend_mode: BlendMode,

    /// Enables anti-aliased painting.
    ///
    /// Default: true
    pub anti_alias: bool,

    /// Colorspace for blending.
    ///
    /// This enables gamma correction during the blend operation.  While skia supports
    /// full color-space conversions, we only support a few (simple) cases.  Note that
    /// any color space other than Linear will force using the high-quality pipeline.
    ///
    /// Default: Linear
    pub colorspace: ColorSpace,

    /// Forces the high quality/precision rendering pipeline.
    ///
    /// `tiny-skia`, just like Skia, has two rendering pipelines:
    /// one uses `f32` and another one uses `u16`. `u16` one is usually way faster,
    /// but less precise. Which can lead to slight differences.
    ///
    /// By default, `tiny-skia` will choose the pipeline automatically,
    /// depending on a blending mode and other parameters.
    /// But you can force the high quality one using this flag.
    ///
    /// This feature is especially useful during testing.
    ///
    /// Unlike high quality pipeline, the low quality one doesn't support all
    /// rendering stages, therefore we cannot force it like hq one.
    ///
    /// Default: false
    pub force_hq_pipeline: bool,
}
impl Default for SkPaint<'_> {
    fn default() -> Self {
        SkPaint {
            shader: Shader::SolidColor(Color::BLACK),
            blend_mode: BlendMode::default(),
            anti_alias: true,
            colorspace: ColorSpace::default(),
            force_hq_pipeline: false,
        }
    }
}
impl SkPaint<'_> {
    /// Sets a paint source to a solid color.
    pub fn set_color(&mut self, color: Color) {
        self.shader = Shader::SolidColor(color);
    }

    /// Sets a paint source to a solid color.
    ///
    /// `self.shader = Shader::SolidColor(Color::from_rgba8(50, 127, 150, 200));` shorthand.
    pub fn set_color_rgba8(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.set_color(Color::from_rgba8(r, g, b, a))
    }

    /// Checks that the paint source is a solid color.
    pub fn is_solid_color(&self) -> bool {
        matches!(self.shader, Shader::SolidColor(_))
    }
}
pub use SkPaint as Paint;
