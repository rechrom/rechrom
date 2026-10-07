//! Raster stage contexts corresponding by responsibility to SkRasterPipelineOpContexts.h.
//! The migrated pipeline still owns contexts together; upstream allocates each stage context.
//! Field names/storage are compatibility adaptations and are not an ABI translation.
pub mod SkRasterPipelineContexts {
    use crate::path::NormalizedF32;
    use crate::raster::wide::u32x8;
    use crate::raster::{Color, SpreadMode};
    use alloc::vec::Vec;
    #[derive(Copy, Clone, Default, Debug)]
    pub struct SamplerCtx {
        pub spread_mode: SpreadMode,
        pub inv_width: f32,
        pub inv_height: f32,
    }

    #[derive(Copy, Clone, Default, Debug)]
    pub struct UniformColorCtx {
        pub r: f32,
        pub g: f32,
        pub b: f32,
        pub a: f32,
        pub rgba: [u16; 4], // [0,255] in a 16-bit lane.
    }

    // A gradient color is an unpremultiplied RGBA not in a 0..1 range.
    // It basically can have any float value.
    #[derive(Copy, Clone, Default, Debug)]
    pub struct GradientColor {
        pub r: f32,
        pub g: f32,
        pub b: f32,
        pub a: f32,
    }

    impl GradientColor {
        pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
            GradientColor { r, g, b, a }
        }
    }

    impl From<Color> for GradientColor {
        fn from(c: Color) -> Self {
            GradientColor {
                r: c.red(),
                g: c.green(),
                b: c.blue(),
                a: c.alpha(),
            }
        }
    }

    #[derive(Copy, Clone, Default, Debug)]
    pub struct EvenlySpaced2StopGradientCtx {
        pub factor: GradientColor,
        pub bias: GradientColor,
    }

    #[derive(Clone, Default, Debug)]
    pub struct GradientCtx {
        /// This value stores the actual colors count.
        /// `factors` and `biases` must store at least 16 values,
        /// since this is the length of a lowp pipeline stage.
        /// So any any value past `len` is just zeros.
        pub len: usize,
        pub factors: Vec<GradientColor>,
        pub biases: Vec<GradientColor>,
        pub t_values: Vec<NormalizedF32>,
    }

    impl GradientCtx {
        pub fn push_const_color(&mut self, color: GradientColor) {
            self.factors.push(GradientColor::new(0.0, 0.0, 0.0, 0.0));
            self.biases.push(color);
        }
    }

    #[derive(Copy, Clone, Default, Debug)]
    pub struct Conical2PtCtx {
        // This context is used only in highp, where we use Tx4.
        pub mask: u32x8,
        pub p0: f32,
        pub p1: f32,
    }

    #[derive(Copy, Clone, Default, Debug)]
    pub struct TileCtx {
        pub scale: f32,
        pub inv_scale: f32, // cache of 1/scale
    }

    pub use Conical2PtCtx as TwoPointConicalGradientCtx;
}
