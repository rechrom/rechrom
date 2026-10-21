#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    not(SK_ENABLE_OPTIMIZE_SIZE)
))]
// Source: src/opts/SkOpts_ml3.cpp:16-31
pub(crate) unsafe fn Init_ml3(
    selected: &mut (
        Option<crate::src::core::SkOpts::PipelineDispatch>,
        crate::src::core::SkOpts::PipelineDispatch,
    ),
) {
    let high = crate::src::core::SkOpts::avx2_highp_dispatch();
    let low = crate::src::core::SkOpts::x86_avx2_lowp_stages_dispatch();
    // Each descriptor carries the native full stage table, terminal, runner and stride.
    *selected = (Some(low), high);
}
