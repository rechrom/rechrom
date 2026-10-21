#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    not(SK_ENABLE_OPTIMIZE_SIZE)
))]
// Source: src/core/SkMemset_opts_avx.cpp:27-35
pub(crate) unsafe fn Init_Memset_avx(d: &mut super::SkMemset_opts::Dispatch) {
    let avx = super::SkMemset_opts::avx_dispatch();
    d.memset16 = avx.memset16;
    d.memset32 = avx.memset32;
    d.memset64 = avx.memset64;
    d.rect_memset16 = avx.rect_memset16;
    d.rect_memset32 = avx.rect_memset32;
    d.rect_memset64 = avx.rect_memset64;
}
