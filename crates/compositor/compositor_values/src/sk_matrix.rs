//! Flattened SkMatrix value (ui/gfx/geometry/skia_conversions.h) produced by
//! gfx::Transform. Moved from compositor_draw::occlusion, which re-exports it.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkMatrix {
    pub values: [f32; 9],
}

impl SkMatrix {
    pub const fn I() -> Self {
        Self {
            values: [1., 0., 0., 0., 1., 0., 0., 0., 1.],
        }
    }
}
