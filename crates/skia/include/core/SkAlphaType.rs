// Copyright 2022 Google LLC. BSD-style license.
// Source: include/core/SkAlphaType.h:26-26
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkAlphaType {
    // Source: include/core/SkAlphaType.h:27-27
    kUnknown_SkAlphaType,
    // Source: include/core/SkAlphaType.h:28-28
    kOpaque_SkAlphaType,
    // Source: include/core/SkAlphaType.h:29-29
    kPremul_SkAlphaType,
    // Source: include/core/SkAlphaType.h:30-30
    kUnpremul_SkAlphaType,
}
// Associated constants preserve C++ duplicate-valued enumerators.
// Source: include/core/SkAlphaType.h:31-31
pub const kLastEnum_SkAlphaType: SkAlphaType = SkAlphaType::kUnpremul_SkAlphaType;
pub use SkAlphaType::*;
// Source: include/core/SkAlphaType.h:41-43
pub fn SkAlphaTypeIsOpaque(at: SkAlphaType) -> bool {
    at == kOpaque_SkAlphaType
}
