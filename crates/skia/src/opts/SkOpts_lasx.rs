#[cfg(all(target_arch = "loongarch64", not(SK_ENABLE_OPTIMIZE_SIZE)))]
// Source: src/opts/SkOpts_lasx.cpp:16-31
pub(crate) unsafe fn Init_lasx(
    selected: &mut (
        Option<crate::src::core::SkOpts::PipelineDispatch>,
        crate::src::core::SkOpts::PipelineDispatch,
    ),
) {
    let high = crate::src::core::SkOpts::lasx_highp_dispatch();
    let low = crate::src::core::SkOpts::lasx_lowp_dispatch();
    // Native complete table/terminal/runner/stride replacement publishes as a pair.
    *selected = (Some(low), high);
}
