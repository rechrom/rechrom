// Copyright 2023 Google LLC. BSD-style license.
// Source: src/core/SkBlitMask_opts.cpp:21-21
// The default binding is the build's own SK_OPTS_NS implementation: NEON on
// ARM64, the SSE2 baseline (portable SkBlitMask_opts.h path) on x86-64.
pub static blit_mask_d32_a8: crate::src::core::SkBlitMask::BlitMaskProc =
    crate::src::opts::SkBlitMask_opts::blit_mask_d32_a8;

// Source: src/core/SkBlitMask_opts.cpp:25-34
fn init() -> bool {
    // ARM64 runs no CPU probe. On x86-64 the source replaces the default with
    // ssse3::blit_mask_d32_a8 when SkCpu::SSSE3 is available; that SSSE3
    // translation is pending, so the SSE2 baseline default stays installed.
    true
}
// Source: src/core/SkBlitMask_opts.cpp:36-38
pub fn Init_BlitMask() {
    // Rust's OnceLock retains native function-local static one-time execution.
    static INITIALIZED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let _ = INITIALIZED.get_or_init(init);
}
