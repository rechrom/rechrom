// Copyright 2023 Google LLC. BSD-style license.
// This translation implements the selected ARM64 build configuration. The
// native x86 SSSE3 runtime initializer is explicitly pending, not replaced by
// the ARM64/default path on x86.
// Source: src/core/SkBlitMask_opts.cpp:21-21
#[cfg(target_arch = "aarch64")]
pub static blit_mask_d32_a8: crate::src::core::SkBlitMask::BlitMaskProc =
    crate::src::opts::SkBlitMask_opts::blit_mask_d32_a8;

// Source: src/core/SkBlitMask_opts.cpp:25-34
#[cfg(target_arch = "aarch64")]
fn init() -> bool {
    // In the original selected ARM64 preprocessor branch no x86 CPU probe or
    // SSSE3 initializer runs. The default is the actual native NEON dispatcher.
    true
}
// Source: src/core/SkBlitMask_opts.cpp:36-38
#[cfg(target_arch = "aarch64")]
pub fn Init_BlitMask() {
    // Rust's OnceLock retains native function-local static one-time execution.
    static INITIALIZED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let _ = INITIALIZED.get_or_init(init);
}
