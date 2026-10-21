use super::SkBitmapProcState::SampleProc32;
#[derive(Clone, Copy)]
pub(crate) struct Dispatch {
    pub dx: SampleProc32,
    pub dxdy: SampleProc32,
}
// Source: src/core/SkBitmapProcState_opts.cpp:21-22
fn defaults() -> Dispatch {
    use crate::include::private::SkFeatures as F;
    use crate::src::opts::SkBitmapProcState_opts as k;
    let mut d = Dispatch {
        dx: k::S32_alpha_D32_filter_DX,
        dxdy: k::S32_alpha_D32_filter_DXDY,
    };
    // DEFINE_DEFAULT uses the native configured CPU level, not Rust's profile
    // or an independent target-feature test. Higher levels share this kernel.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let level = F::SK_CPU_X64_LEVEL.unwrap_or(0);
        if level >= F::SK_CPU_X64_LEVEL_SSSE3 {
            d.dx = k::ssse3::S32_alpha_D32_filter_DX;
        } else if level >= F::SK_CPU_X64_LEVEL_SSE2 {
            d.dx = k::sse2::S32_alpha_D32_filter_DX;
        }
    }
    #[cfg(target_arch = "loongarch64")]
    {
        let level = F::SK_CPU_LSX_LEVEL.unwrap_or(0);
        if level >= F::SK_CPU_LSX_LEVEL_LASX {
            d.dx = k::lasx::S32_alpha_D32_filter_DX;
        } else if level >= F::SK_CPU_LSX_LEVEL_LSX {
            d.dx = k::lsx::S32_alpha_D32_filter_DX;
        }
    }
    d
}
// Source: src/core/SkBitmapProcState_opts.cpp:24-25
// C++ declarations bind the actual target TU producers; no replacement Init.
#[cfg(all(target_arch = "loongarch64", not(SK_ENABLE_OPTIMIZE_SIZE)))]
use super::SkBitmapProcState_opts_lasx::Init_BitmapProcState_lasx;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    not(SK_ENABLE_OPTIMIZE_SIZE)
))]
use super::SkBitmapProcState_opts_ssse3::Init_BitmapProcState_ssse3;

// Source: src/core/SkBitmapProcState_opts.cpp:27-40
fn init(d: &mut Dispatch) -> bool {
    #[cfg(not(SK_ENABLE_OPTIMIZE_SIZE))]
    {
        use crate::include::private::SkFeatures as F;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if F::SK_CPU_X86.is_some() {
            // Source: src/core/SkBitmapProcState_opts.cpp:31-32
            if F::SK_CPU_X64_LEVEL.unwrap_or(0) < F::SK_CPU_X64_LEVEL_SSSE3 {
                if super::SkCpu::Supports(super::SkCpu::SkX64::SSSE3) {
                    unsafe {
                        Init_BitmapProcState_ssse3(d);
                    }
                }
            }
        }
        #[cfg(target_arch = "loongarch64")]
        if F::SK_CPU_LOONGARCH.is_some() {
            // Source: src/core/SkBitmapProcState_opts.cpp:35-36
            if F::SK_CPU_LSX_LEVEL.unwrap_or(0) < F::SK_CPU_LSX_LEVEL_LASX {
                if super::SkCpu::Supports(super::SkCpu::SkLoongArch::ASX) {
                    unsafe {
                        Init_BitmapProcState_lasx(d);
                    }
                }
            }
        }
    }
    true
}
// Adapted storage/publication: the native function-local initialized flag and
// writable function pointers share one immutable published Dispatch in Rust.
// Function-pointer fields contain the actual selected native kernel producers.
fn selected() -> &'static Dispatch {
    static D: std::sync::OnceLock<Dispatch> = std::sync::OnceLock::new();
    D.get_or_init(|| {
        let mut d = defaults();
        let _ = init(&mut d);
        d
    })
}
unsafe fn dispatch_dx(
    s: &super::SkBitmapProcState::SkBitmapProcState,
    xy: &[u32],
    n: i32,
    out: &mut [u32],
) {
    (selected().dx)(s, xy, n, out)
}
unsafe fn dispatch_dxdy(
    s: &super::SkBitmapProcState::SkBitmapProcState,
    xy: &[u32],
    n: i32,
    out: &mut [u32],
) {
    (selected().dxdy)(s, xy, n, out)
}
pub static S32_alpha_D32_filter_DX: Option<SampleProc32> = Some(dispatch_dx);
pub static S32_alpha_D32_filter_DXDY: Option<SampleProc32> = Some(dispatch_dxdy);
// Source: src/core/SkBitmapProcState_opts.cpp:42-44
pub fn Init_BitmapProcState() {
    let _ = selected();
}
