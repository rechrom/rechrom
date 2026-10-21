#[cfg(all(target_arch = "loongarch64", not(SK_ENABLE_OPTIMIZE_SIZE)))]
// Source: src/core/SkBitmapProcState_opts_lasx.cpp:27-29
pub(crate) unsafe fn Init_BitmapProcState_lasx(d: &mut super::SkBitmapProcState_opts::Dispatch) {
    d.dx = crate::src::opts::SkBitmapProcState_opts::lasx::S32_alpha_D32_filter_DX;
}
