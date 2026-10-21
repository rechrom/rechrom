// Copyright 2018 Google LLC. BSD-style license.
// skcms_internals.h:112-123 defines SKCMS_DISABLE_HSW on the selected
// non-x86 ARM64 configuration. This is the ORIGINAL disabled-ISA wrapper;
// The enabled x86 N=8 branch instantiates the actual common program.

use super::skcms_Transform::Op;
use core::ffi::c_void;
// Source: modules/skcms/src/skcms_TransformHsw.cc:41-46
/// Safety: program/context/pixel spans obey the actual baseline interpreter's
/// native pointer, pixel count, layout and retained context lifetime contract.
#[cfg(any(
    skcms_portable,
    not(target_arch = "x86_64"),
    target_os = "android",
    skcms_disable_hsw
))]
pub unsafe fn run_program(
    program: *const Op,
    contexts: *const *const c_void,
    programSize: isize,
    src: *const u8,
    dst: *mut u8,
    n: i32,
    src_bpp: usize,
    dst_bpp: usize,
) {
    unsafe {
        super::skcms_TransformBaseline::run_program(
            program,
            contexts,
            programSize,
            src,
            dst,
            n,
            src_bpp,
            dst_bpp,
        )
    }
}

// Source: modules/skcms/src/skcms_TransformHsw.cc:53-54
// Actual native wide lane storage, not an ARM program or scalar evaluator.
// The common native program is instantiated at this exact width below.
#[cfg(all(
    target_arch = "x86_64",
    not(target_os = "android"),
    not(skcms_portable),
    not(skcms_disable_hsw)
))]
pub mod enabled {
    pub const N: usize = 8;
    pub type V<T> = [T; N];
    // Source: modules/skcms/src/skcms_TransformHsw.cc:56
    crate::modules::skcms::src::Transform_inl::instantiate_transform_wide_backend!(hsw);
}
#[cfg(all(
    target_arch = "x86_64",
    not(target_os = "android"),
    not(skcms_portable),
    not(skcms_disable_hsw)
))]
pub use enabled::{run_program, N, V};
