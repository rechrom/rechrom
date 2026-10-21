// Copyright 2023 Google LLC. BSD-style license.
// Source: src/core/SkBlitMask.h:14-17
// Named Rust type for the native extern function pointer declaration. Its real
// definition/default target/CPU initialization belong to SkBlitMask_opts.cpp,
// now translated for selected ARM64; x86 runtime SSSE3 dispatch remains pending.
pub type BlitMaskProc = unsafe fn(*mut u32, usize, *const u8, usize, u32, i32, i32);
// Source: src/core/SkBlitMask.h:14-19
pub use crate::src::core::SkBlitMask_opts::{blit_mask_d32_a8, Init_BlitMask};
