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

impl SkColor4f {
    // Source: src/core/SkColor.cpp:134-144; src/core/SkSwizzlePriv.h:49-60.
    pub fn FromColor(argb: u32) -> Self {
        let scale = 1. / 255.;
        Self {
            fR: ((argb >> 16) & 255) as f32 * scale,
            fG: ((argb >> 8) & 255) as f32 * scale,
            fB: (argb & 255) as f32 * scale,
            fA: (argb >> 24) as f32 * scale,
        }
    }
    pub fn toSkColor(&self) -> u32 {
        let byte = |v: f32| (v * 255. + 0.5).clamp(0., 255.) as u32;
        (byte(self.fA) << 24) | (byte(self.fR) << 16) | (byte(self.fG) << 8) | byte(self.fB)
    }
}
