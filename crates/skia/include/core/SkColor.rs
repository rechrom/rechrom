//! Rust subset of include/core/SkColor.h. See ../../LICENSE.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkColor4f {
    pub fR: f32,
    pub fG: f32,
    pub fB: f32,
    pub fA: f32,
}
impl SkColor4f {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            fR: r,
            fG: g,
            fB: b,
            fA: a,
        }
    }
}
