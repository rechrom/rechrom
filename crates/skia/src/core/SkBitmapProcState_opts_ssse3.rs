#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    not(SK_ENABLE_OPTIMIZE_SIZE)
))]
// Source: src/core/SkBitmapProcState_opts_ssse3.cpp:27-29
pub(crate) unsafe fn Init_BitmapProcState_ssse3(d: &mut super::SkBitmapProcState_opts::Dispatch) {
    d.dx = crate::src::opts::SkBitmapProcState_opts::ssse3::S32_alpha_D32_filter_DX;
}
