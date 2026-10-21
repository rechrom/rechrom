use std::sync::Arc;

// Source: src/core/SkMaskGamma.h:32-34
// Source: src/core/SkMaskGamma.h:37-37
// Source: src/core/SkMaskGamma.h:39-39
pub trait SkColorSpaceLuminance: Sync {
    fn to_luma(&self, gamma: f32, luminance: f32) -> f32;
    fn from_luma(&self, gamma: f32, luma: f32) -> f32;
}

// Source: src/core/SkMaskGamma.h:42-52
pub fn compute_luminance(gamma: f32, color: u32) -> u32 {
    use crate::src::core::SkColorData::{SK_LUM_COEFF_B, SK_LUM_COEFF_G, SK_LUM_COEFF_R};
    let luminance = fetch_luminance(gamma);
    let r = luminance.to_luma(gamma, ((color >> 16) & 255) as f32 / 255.);
    let g = luminance.to_luma(gamma, ((color >> 8) & 255) as f32 / 255.);
    let b = luminance.to_luma(gamma, (color & 255) as f32 / 255.);
    let luma = r * SK_LUM_COEFF_R + g * SK_LUM_COEFF_G + b * SK_LUM_COEFF_B;
    debug_assert!(luma <= 1.);
    ((luminance.from_luma(gamma, luma) * 255.) as f64 + 0.5).floor() as i32 as u32
}

// Source: src/core/SkMaskGamma.h:64-83
pub fn sk_t_scale255<const N: u32>(mut base: u32) -> u32 {
    match N {
        1 => base * 0xff,
        2 => base * 0x55,
        4 => base * 0x11,
        8 => base,
        _ => {
            assert!((1..=8).contains(&N));
            base <<= 8 - N;
            let mut lum = base;
            let mut i = N;
            while i < 8 {
                lum |= base >> i;
                i += N;
            }
            lum
        }
    }
}

// Source: src/core/SkMaskGamma.h:102-102
// Source: src/core/SkMaskGamma.h:104-104
// Source: src/core/SkMaskGamma.h:107-107
pub struct SkTMaskGamma<const R: u32, const G: u32, const B: u32> {
    // Source: src/core/SkMaskGamma.h:186-186
    gamma_tables: Option<Box<[u8]>>,
}
impl<const R: u32, const G: u32, const B: u32> SkTMaskGamma<R, G, B> {
    pub const fn linear() -> Self {
        Self { gamma_tables: None }
    }
    // Source: src/core/SkMaskGamma.h:117-126
    pub fn new(contrast: f32, device_gamma: f32) -> Self {
        let mut tables = vec![0; Self::TABLE_NUM_ELEMENTS].into_boxed_slice();
        let convert = fetch_luminance(device_gamma);
        for i in 0..Self::NUM_TABLES {
            // Rust cannot use an expression involving generic parameters as a const argument.
            let base = (i as u32) << (8 - Self::MAX_LUM_BITS);
            let mut lum = base;
            let mut bit = Self::MAX_LUM_BITS;
            while bit < 8 {
                lum |= base >> bit;
                bit += Self::MAX_LUM_BITS;
            }
            let row: &mut [u8; 256] = (&mut tables[i * 256..(i + 1) * 256]).try_into().unwrap();
            build_correcting_lut(row, lum, contrast, convert, device_gamma);
        }
        Self {
            gamma_tables: Some(tables),
        }
    }
    // Source: src/core/SkMaskGamma.h:129-134
    pub fn canonical_color(color: u32) -> u32 {
        0xff000000
            | sk_t_scale255::<R>(((color >> 16) & 255) >> (8 - R)) << 16
            | sk_t_scale255::<G>(((color >> 8) & 255) >> (8 - G)) << 8
            | sk_t_scale255::<B>((color & 255) >> (8 - B))
    }
    // Source: src/core/SkMaskGamma.h:150-153
    pub fn gamma_table_dimensions(&self) -> (usize, usize) {
        (Self::TABLE_WIDTH, Self::NUM_TABLES)
    }
    // Source: src/core/SkMaskGamma.h:159-161
    pub const fn gamma_table_size_in_bytes(&self) -> usize {
        Self::TABLE_NUM_ELEMENTS
    }
    // Source: src/core/SkMaskGamma.h:168-170
    // Source: src/core/SkMaskGamma.h:172-176
    // Source: src/core/SkMaskGamma.h:178-180
    pub fn gamma_tables(&self) -> Option<&[u8]> {
        self.gamma_tables.as_deref()
    }
    const MAX_LUM_BITS: u32 = if B > R {
        if B > G {
            B
        } else {
            G
        }
    } else if R > G {
        R
    } else {
        G
    };
    const NUM_TABLES: usize = 1 << Self::MAX_LUM_BITS;
    const TABLE_WIDTH: usize = 256;
    const TABLE_NUM_ELEMENTS: usize = Self::NUM_TABLES * Self::TABLE_WIDTH;
    pub fn is_linear(&self) -> bool {
        self.gamma_tables.is_none()
    }
}

// Source: src/core/SkMaskGamma.h:200-204
// Source: src/core/SkMaskGamma.h:206-208
// Source: src/core/SkMaskGamma.h:210-210
// Source: src/core/SkMaskGamma.h:216-217
// Source: src/core/SkMaskGamma.h:219-219
// Source: src/core/SkMaskGamma.h:222-222
// Source: src/core/SkMaskGamma.h:224-227
// Arc replaces sk_sp; row offsets replace self-referential pointers without changing lifetime.
#[derive(Clone)]
// Source: src/core/SkMaskGamma.h:86-86
pub struct SkTMaskPreBlend<const R: u32, const G: u32, const B: u32> {
    parent: Option<Arc<SkTMaskGamma<R, G, B>>>,
    rows: [usize; 3],
}
impl<const R: u32, const G: u32, const B: u32> SkTMaskPreBlend<R, G, B> {
    pub fn new() -> Self {
        Self {
            parent: None,
            rows: [0; 3],
        }
    }
    pub fn is_applicable(&self) -> bool {
        self.parent.is_some()
    }
    pub fn channel(&self, channel: usize) -> Option<&[u8; 256]> {
        let parent = self.parent.as_ref()?;
        let start = self.rows[channel];
        parent
            .gamma_tables
            .as_ref()
            .map(|t| t[start..start + 256].try_into().unwrap())
    }
}
// Source: src/core/SkMaskGamma.h:229-246
impl<const R: u32, const G: u32, const B: u32> SkTMaskGamma<R, G, B> {
    // Source: src/core/SkMaskGamma.h:137-144
    pub fn pre_blend(self: &Arc<Self>, color: u32) -> SkTMaskPreBlend<R, G, B> {
        if self.is_linear() {
            return SkTMaskPreBlend::new();
        }
        let lum_shift = 8 - Self::MAX_LUM_BITS;
        let r = (((color >> 16) & 255) >> lum_shift) as usize * Self::TABLE_WIDTH;
        let g = (((color >> 8) & 255) >> lum_shift) as usize * Self::TABLE_WIDTH;
        let b = ((color & 255) >> lum_shift) as usize * Self::TABLE_WIDTH;
        debug_assert!(
            r < Self::TABLE_NUM_ELEMENTS
                && g < Self::TABLE_NUM_ELEMENTS
                && b < Self::TABLE_NUM_ELEMENTS
        );
        SkTMaskPreBlend {
            parent: Some(Arc::clone(self)),
            rows: [r, g, b],
        }
    }
}
// Source: src/core/SkMaskGamma.h:256-261
pub fn sk_apply_lut_if<const APPLY: bool>(component: u32, lut: Option<&[u8; 256]>) -> u32 {
    if APPLY {
        lut.unwrap()[component as usize] as u32
    } else {
        component
    }
}

// Source: src/core/SkMaskGamma.cpp:17-26
struct LinearLuminance;
impl SkColorSpaceLuminance for LinearLuminance {
    fn to_luma(&self, gamma: f32, luminance: f32) -> f32 {
        debug_assert_eq!(gamma, 1.);
        luminance
    }
    fn from_luma(&self, gamma: f32, luma: f32) -> f32 {
        debug_assert_eq!(gamma, 1.);
        luma
    }
}
// Source: src/core/SkMaskGamma.cpp:28-35
struct GammaLuminance;
impl SkColorSpaceLuminance for GammaLuminance {
    fn to_luma(&self, gamma: f32, luminance: f32) -> f32 {
        luminance.powf(gamma)
    }
    fn from_luma(&self, gamma: f32, luma: f32) -> f32 {
        luma.powf(1. / gamma)
    }
}
// Source: src/core/SkMaskGamma.cpp:37-39
// Source: src/core/SkMaskGamma.cpp:42-49
// Source: src/core/SkMaskGamma.cpp:52-58
struct SrgbLuminance;
impl SkColorSpaceLuminance for SrgbLuminance {
    fn to_luma(&self, gamma: f32, luminance: f32) -> f32 {
        debug_assert_eq!(gamma, 0.);
        if luminance <= 0.04045 {
            return luminance / 12.92;
        }
        ((luminance + 0.055) / 1.055).powf(2.4)
    }
    fn from_luma(&self, gamma: f32, luma: f32) -> f32 {
        debug_assert_eq!(gamma, 0.);
        if luma <= 0.0031308 {
            return luma * 12.92;
        }
        1.055 * luma.powf(1. / 2.4) - 0.055
    }
}
// Source: src/core/SkMaskGamma.cpp:60-63
// Source: src/core/SkMaskGamma.cpp:65-72
// Source: src/core/SkMaskGamma.h:55-55
pub fn fetch_luminance(gamma: f32) -> &'static dyn SkColorSpaceLuminance {
    static LINEAR: LinearLuminance = LinearLuminance;
    static GAMMA: GammaLuminance = GammaLuminance;
    static SRGB: SrgbLuminance = SrgbLuminance;
    if gamma == 0. {
        &SRGB
    } else if gamma == 1. {
        &LINEAR
    } else {
        &GAMMA
    }
}
// Source: src/core/SkMaskGamma.cpp:74-76
fn apply_contrast(srca: f32, contrast: f32) -> f32 {
    srca + ((1. - srca) * contrast * srca)
}
// Source: src/core/SkMaskGamma.cpp:78-83
// Source: src/core/SkMaskGamma.cpp:88-89
// Source: src/core/SkMaskGamma.cpp:92-92
// Source: src/core/SkMaskGamma.cpp:96-103
// Source: src/core/SkMaskGamma.cpp:105-106
// Source: src/core/SkMaskGamma.cpp:111-114
// Source: src/core/SkMaskGamma.cpp:117-119
// Source: src/core/SkMaskGamma.cpp:122-123
// Source: src/core/SkMaskGamma.cpp:125-128
// Source: src/core/SkMaskGamma.h:88-89
pub fn build_correcting_lut(
    table: &mut [u8; 256],
    src_i: u32,
    contrast: f32,
    dst_convert: &dyn SkColorSpaceLuminance,
    dst_gamma: f32,
) {
    let src_convert = dst_convert;
    let src_gamma = dst_gamma;
    let src = src_i as f32 / 255.;
    let lin_src = src_convert.to_luma(src_gamma, src);
    let dst = 1. - src;
    let lin_dst = dst_convert.to_luma(dst_gamma, dst);
    let adjusted_contrast = contrast * lin_dst;
    if (src - dst).abs() < 1. / 256. {
        let mut ii = 0.;
        for i in 0..256 {
            let raw_srca = ii / 255.;
            let srca = apply_contrast(raw_srca, adjusted_contrast);
            table[i] = ((255. * srca) as f64 + 0.5).floor() as i32 as u8;
            ii += 1.;
        }
    } else {
        let mut ii = 0.;
        for i in 0..256 {
            let raw_srca = ii / 255.;
            let srca = apply_contrast(raw_srca, adjusted_contrast);
            debug_assert!(srca <= 1.);
            let dsta = 1. - srca;
            let lin_out = lin_src * srca + dsta * lin_dst;
            debug_assert!(lin_out <= 1.);
            let out = dst_convert.from_luma(dst_gamma, lin_out);
            let result = (out - dst) / (src - dst);
            debug_assert!(((255. * result) as f64 + 0.5).floor() <= 255.);
            table[i] = ((255. * result) as f64 + 0.5).floor() as i32 as u8;
            ii += 1.;
        }
    }
}
