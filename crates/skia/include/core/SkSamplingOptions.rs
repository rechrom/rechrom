// Source: include/core/SkSamplingOptions.h:16-18
// Source: include/core/SkSamplingOptions.h:20-22
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkFilterMode {
    kNearest,
    kLinear,
}
impl SkFilterMode {
    pub const kLast: Self = Self::kLinear;
}
pub const kSkFilterModeCount: i32 = SkFilterMode::kLast as i32 + 1;
// Source: include/core/SkSamplingOptions.h:24-27
// Source: include/core/SkSamplingOptions.h:29-31
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkMipmapMode {
    kNone,
    kNearest,
    kLinear,
}
impl SkMipmapMode {
    pub const kLast: Self = Self::kLinear;
}
pub const kSkMipmapModeCount: i32 = SkMipmapMode::kLast as i32 + 1;
// Source: include/core/SkSamplingOptions.h:50-51
// Source: include/core/SkSamplingOptions.h:54-56
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkCubicResampler {
    pub B: f32,
    pub C: f32,
}
impl SkCubicResampler {
    pub const fn Mitchell() -> Self {
        Self {
            B: 1.0 / 3.0,
            C: 1.0 / 3.0,
        }
    }
}
impl SkCubicResampler {
    pub const fn CatmullRom() -> Self {
        Self {
            B: 0.0,
            C: 1.0 / 2.0,
        }
    }
}
// Source: include/core/SkSamplingOptions.h:58-63
// Source: include/core/SkSamplingOptions.h:65-66
// Private fields/read-only getters preserve native const public member semantics; whole-value Copy replaces native copy construction.
#[derive(Clone, Copy, Debug)]
pub struct SkSamplingOptions {
    maxAniso: i32,
    useCubic: bool,
    cubic: SkCubicResampler,
    filter: SkFilterMode,
    mipmap: SkMipmapMode,
}
impl Default for SkSamplingOptions {
    fn default() -> Self {
        Self::new()
    }
}
impl SkSamplingOptions {
    pub const fn new() -> Self {
        Self {
            maxAniso: 0,
            useCubic: false,
            cubic: SkCubicResampler { B: 0.0, C: 0.0 },
            filter: SkFilterMode::kNearest,
            mipmap: SkMipmapMode::kNone,
        }
    }
}
impl SkSamplingOptions {
    pub const fn maxAniso(&self) -> i32 {
        self.maxAniso
    }
}
impl SkSamplingOptions {
    pub const fn useCubic(&self) -> bool {
        self.useCubic
    }
}
impl SkSamplingOptions {
    pub const fn cubic(&self) -> SkCubicResampler {
        self.cubic
    }
}
impl SkSamplingOptions {
    pub const fn filter(&self) -> SkFilterMode {
        self.filter
    }
}
impl SkSamplingOptions {
    pub const fn mipmap(&self) -> SkMipmapMode {
        self.mipmap
    }
}
impl SkSamplingOptions {
    // Source: include/core/SkSamplingOptions.h:67-71
    pub fn assign_from(&mut self, that: &Self) -> &mut Self {
        *self = *that;
        self
    }
}
impl SkSamplingOptions {
    // Source: include/core/SkSamplingOptions.h:73-75
    pub const fn with_filter_mipmap(fm: SkFilterMode, mm: SkMipmapMode) -> Self {
        Self {
            filter: fm,
            mipmap: mm,
            ..Self::new()
        }
    }
}
impl SkSamplingOptions {
    // Source: include/core/SkSamplingOptions.h:79-81
    pub const fn with_filter(fm: SkFilterMode) -> Self {
        Self {
            filter: fm,
            mipmap: SkMipmapMode::kNone,
            ..Self::new()
        }
    }
}
impl SkSamplingOptions {
    // Source: include/core/SkSamplingOptions.h:83-85
    pub const fn with_cubic(c: SkCubicResampler) -> Self {
        Self {
            useCubic: true,
            cubic: c,
            ..Self::new()
        }
    }
}
impl SkSamplingOptions {
    // Source: include/core/SkSamplingOptions.h:87-89
    pub const fn Aniso(maxAniso: i32) -> Self {
        Self::with_aniso(if maxAniso < 1 { 1 } else { maxAniso })
    }
}
// Source: include/core/SkSamplingOptions.h:91-99
impl PartialEq for SkSamplingOptions {
    fn eq(&self, other: &Self) -> bool {
        self.maxAniso == other.maxAniso
            && self.useCubic == other.useCubic
            && self.cubic.B == other.cubic.B
            && self.cubic.C == other.cubic.C
            && self.filter == other.filter
            && self.mipmap == other.mipmap
    }
}
impl SkSamplingOptions {
    // Source: include/core/SkSamplingOptions.h:101-101
    pub fn isAniso(&self) -> bool {
        self.maxAniso != 0
    }
}
impl SkSamplingOptions {
    // Source: include/core/SkSamplingOptions.h:104-104
    const fn with_aniso(maxAniso: i32) -> Self {
        Self {
            maxAniso,
            ..Self::new()
        }
    }
}
