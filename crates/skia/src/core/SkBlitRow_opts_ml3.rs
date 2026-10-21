#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    not(SK_ENABLE_OPTIMIZE_SIZE)
))]
// Source: src/core/SkBlitRow_opts_ml3.cpp:27-30
pub(crate) unsafe fn Init_BlitRow_ml3(d: &mut super::SkBlitRow_opts::Dispatch) {
    d.color32 = crate::src::opts::SkBlitRow_opts::blit_row_color32_raw;
    d.opaque = crate::src::opts::SkBlitRow_opts::x86::blit_row_s32a_opaque_avx2;
}
