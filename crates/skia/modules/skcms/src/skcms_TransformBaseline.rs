// Copyright 2018 Google LLC. BSD-style license.

// Selected native configuration: baselineN4, ARM64 NEON, SKCMS_HAS_MUSTTAIL=0.
// Source: modules/skcms/src/skcms_TransformBaseline.cc:42-43
#[cfg(all(target_arch = "aarch64", not(skcms_portable)))]
pub const N: usize = 4;
// Source: modules/skcms/src/skcms_TransformBaseline.cc:44-44
#[cfg(all(target_arch = "aarch64", not(skcms_portable)))]
pub type V<T: super::skcms_Transform::BaselineVectorLane> =
    <T as super::skcms_Transform::BaselineVectorLane>::Vector;
// Native Clang V<T> aliases use the actual four-lane register representation in Transform_inl.
#[cfg(all(target_arch = "aarch64", not(skcms_portable)))]
pub use super::Transform_inl::{F, I32, U16, U32, U64, U8};
// Source: modules/skcms/src/skcms_TransformBaseline.cc:46-46
// Native textual inclusion instantiates the genuine implementation in this namespace.
// Rust module visibility replaces inclusion without a second algorithm or owner.
#[cfg(all(target_arch = "aarch64", not(skcms_portable)))]
pub use super::Transform_inl::run_program;

// Source: modules/skcms/src/skcms_TransformBaseline.cc:39-40
#[cfg(skcms_portable)]
pub const N: usize = 1;
#[cfg(skcms_portable)]
pub type V<T> = T;
// Source: modules/skcms/src/skcms_TransformBaseline.cc:46-46
// The real scalar source instance, not the NEON four-lane program.
#[cfg(skcms_portable)]
mod portable_instance {
    crate::modules::skcms::src::Transform_inl::instantiate_transform_portable_backend!();
}
#[cfg(skcms_portable)]
pub use portable_instance::run_program;
