//! Native sampling option values used by PaintFlags.
#![allow(non_camel_case_types, non_snake_case)]
// Source: include/core/SkSamplingOptions.h:16-31.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkFilterMode {
    #[default]
    kNearest,
    kLinear,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkMipmapMode {
    #[default]
    kNone,
    kNearest,
    kLinear,
}
// Source: include/core/SkSamplingOptions.h:50-56.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkCubicResampler {
    pub B: f32,
    pub C: f32,
}
impl SkCubicResampler {
    pub const fn Mitchell() -> Self {
        Self {
            B: 1. / 3.,
            C: 1. / 3.,
        }
    }
    pub const fn CatmullRom() -> Self {
        Self { B: 0., C: 0.5 }
    }
}
// Source: include/core/SkSamplingOptions.h:58-108.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkSamplingOptions {
    pub maxAniso: i32,
    pub useCubic: bool,
    pub cubic: SkCubicResampler,
    pub filter: SkFilterMode,
    pub mipmap: SkMipmapMode,
}
impl SkSamplingOptions {
    pub fn Filter(filter: SkFilterMode, mipmap: SkMipmapMode) -> Self {
        Self {
            filter,
            mipmap,
            ..Self::default()
        }
    }
    pub fn Cubic(cubic: SkCubicResampler) -> Self {
        Self {
            useCubic: true,
            cubic,
            ..Self::default()
        }
    }
    pub fn Aniso(max: i32) -> Self {
        Self {
            maxAniso: max.max(1),
            ..Self::default()
        }
    }
    pub fn isAniso(&self) -> bool {
        self.maxAniso != 0
    }
}
