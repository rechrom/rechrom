//! SkColor4f value (third_party/skia/include/core/SkColor.h) carried by cc
//! layers, display lists and commit state. Moved from compositor_draw, which
//! re-exports it.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color4f {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}
