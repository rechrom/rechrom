//! Backend-neutral image sampling values carried by Paint policy.
//!
//! Source: include/core/SkSamplingOptions.h:16-31,50-108. Concrete
//! legacy/translated Skia sampling values remain private to raster adapters.
#![allow(non_snake_case, non_camel_case_types)]

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FilterMode {
    #[default]
    kNearest,
    kLinear,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MipmapMode {
    #[default]
    kNone,
    kNearest,
    kLinear,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CubicResampler {
    pub B: f32,
    pub C: f32,
}

impl CubicResampler {
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SamplingOptions {
    pub max_aniso: i32,
    pub use_cubic: bool,
    pub cubic: CubicResampler,
    pub filter: FilterMode,
    pub mipmap: MipmapMode,
}

impl SamplingOptions {
    pub const fn Filter(filter: FilterMode, mipmap: MipmapMode) -> Self {
        Self {
            filter,
            mipmap,
            ..Self::new()
        }
    }

    pub const fn Cubic(cubic: CubicResampler) -> Self {
        Self {
            use_cubic: true,
            cubic,
            ..Self::new()
        }
    }

    pub const fn Aniso(max_aniso: i32) -> Self {
        Self {
            max_aniso: if max_aniso < 1 { 1 } else { max_aniso },
            ..Self::new()
        }
    }

    pub const fn new() -> Self {
        Self {
            max_aniso: 0,
            use_cubic: false,
            cubic: CubicResampler { B: 0., C: 0. },
            filter: FilterMode::kNearest,
            mipmap: MipmapMode::kNone,
        }
    }

    pub const fn isAniso(&self) -> bool {
        self.max_aniso != 0
    }
}
