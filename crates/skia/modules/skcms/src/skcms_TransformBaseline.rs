// Copyright 2018 Google LLC. BSD-style license.

// ARM64: the N=4 NEON baseline. Elsewhere the SKCMS_PORTABLE scalar instance,
// which computes the same per-lane results as the 4-lane vector baseline.
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
#[cfg(any(skcms_portable, not(target_arch = "aarch64")))]
pub const N: usize = 1;
#[cfg(any(skcms_portable, not(target_arch = "aarch64")))]
pub type V<T> = T;
// Source: modules/skcms/src/skcms_TransformBaseline.cc:46-46
// The real scalar source instance, not the NEON four-lane program.
#[cfg(any(skcms_portable, not(target_arch = "aarch64")))]
mod portable_instance {
    crate::modules::skcms::src::Transform_inl::instantiate_transform_portable_backend!();
}
#[cfg(any(skcms_portable, not(target_arch = "aarch64")))]
pub use portable_instance::run_program;
