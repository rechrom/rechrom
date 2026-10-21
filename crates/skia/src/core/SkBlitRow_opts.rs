use super::SkBlitRow::RawProc32;
#[derive(Clone, Copy)]
pub(crate) struct Dispatch {
    pub color32: unsafe fn(*mut u32, i32, u32),
    pub opaque: RawProc32,
}
// Source: src/core/SkBlitRow_opts.cpp:21-22
fn defaults() -> Dispatch {
    use crate::src::opts::SkBlitRow_opts as k;
    let mut d = Dispatch {
        color32: k::blit_row_color32_raw,
        opaque: k::blit_row_s32a_opaque_raw,
    };
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        d.opaque = k::neon::blit_row_s32a_opaque;
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if cfg!(target_feature = "avx2") {
            d.opaque = k::x86::blit_row_s32a_opaque_avx2;
        } else if cfg!(target_feature = "sse2") {
            d.opaque = k::x86::blit_row_s32a_opaque_sse2;
        }
    }
    #[cfg(target_arch = "loongarch64")]
    {
        if cfg!(target_feature = "lasx") {
            d.opaque = k::loong::blit_row_s32a_opaque_lasx_raw;
        } else if cfg!(target_feature = "lsx") {
            d.opaque = k::loong::blit_row_s32a_opaque_lsx_raw;
        }
    }
    d
}
// Source: src/core/SkBlitRow_opts.cpp:27-40
fn selected() -> &'static Dispatch {
    static D: std::sync::OnceLock<Dispatch> = std::sync::OnceLock::new();
    D.get_or_init(|| {
        let mut d = defaults();
        #[cfg(not(SK_ENABLE_OPTIMIZE_SIZE))]
        {
            #[cfg(any(
                target_arch = "x86",
                target_arch = "x86_64",
                target_arch = "loongarch64"
            ))]
            super::SkCpu::CacheRuntimeFeatures();
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            if !cfg!(target_feature = "avx2") && super::SkCpu::Supports(super::SkCpu::SkX64::ML3) {
                unsafe {
                    super::SkBlitRow_opts_ml3::Init_BlitRow_ml3(&mut d);
                }
            }
            #[cfg(target_arch = "loongarch64")]
            if !cfg!(target_feature = "lasx")
                && super::SkCpu::Supports(super::SkCpu::SkLoongArch::ASX)
            {
                unsafe {
                    super::SkBlitRow_opts_lasx::Init_BlitRow_lasx(&mut d);
                }
            }
        }
        d
    })
}
pub unsafe fn blit_row_color32(p: *mut u32, n: i32, c: u32) {
    (selected().color32)(p, n, c)
}
pub unsafe fn blit_row_s32a_opaque(p: *mut u32, s: *const u32, n: i32, a: u32) {
    (selected().opaque)(p, s, n, a)
}
// Source: src/core/SkBlitRow_opts.cpp:42-44
pub fn Init_BlitRow() {
    let _ = selected();
}
