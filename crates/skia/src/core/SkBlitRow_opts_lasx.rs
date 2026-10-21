#[cfg(all(target_arch = "loongarch64", not(SK_ENABLE_OPTIMIZE_SIZE)))]
// Source: src/core/SkBlitRow_opts_lasx.cpp:27-30
pub(crate) unsafe fn Init_BlitRow_lasx(d: &mut super::SkBlitRow_opts::Dispatch) {
    d.color32 = crate::src::opts::SkBlitRow_opts::blit_row_color32_raw;
    d.opaque = crate::src::opts::SkBlitRow_opts::loong::blit_row_s32a_opaque_lasx_raw;
}
